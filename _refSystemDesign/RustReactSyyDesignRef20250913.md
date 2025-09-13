Here’s a pragmatic playbook for converting a **Rails full‑stack app** into a **Rust backend + React frontend**—from “how to carve up the monolith” down to “which crates and idioms to reach for,” plus the potholes to avoid.

---

## 0) The migration strategy (top level)

**Adopt an incremental migration, not a rewrite.**

* Use the **Strangler Fig** pattern: route some requests to new Rust/React slices while the rest still hit Rails. Over time, “strangle” the old endpoints behind a façade/proxy until the Rails tree is gone. ([martinfowler.com][1])
* Insert a **BFF (Backend‑for‑Frontend)** to tailor APIs per client (web, mobile) and to hide backend churn during the migration. This reduces over/under‑fetching and isolates frontend needs from backend internals. ([Sam Newman][2])

**Practical steps**

1. Put a reverse proxy (nginx/Traefik) or API gateway in front.
2. Start with a leaf feature (e.g., a read‑only page) and route it to React + Rust.
3. Move write paths next; buffer risk with **contract tests** (Pact) between frontend and backend so refactors don’t break callers. ([Pact Docs][3])
4. Avoid dual‑writes. If you must share data, introduce CDC (e.g., Debezium) or a single writer until tables migrate.
5. Keep **API versions** explicit during cutover; use OpenAPI for schema and change discipline. Generate typed TS clients from the spec (e.g., `openapi-generator -g typescript-fetch`). ([OpenAPI Generator][4])

---

## 1) System design choices

**Monolith first, services later.** Start with a modular Rust **monolith** (workspace with crates) using **hexagonal (ports & adapters)** so data stores and transport are plugins. Later, split along clear **bounded contexts** if needed. ([Alistair Cockburn][5])

**Protocol shape:**

* **REST** is simple and frontend‑friendly; document with OpenAPI.
* **gRPC** (Tonic) excels for internal service‑to‑service calls and streaming; expose **grpc‑web** if you truly need it in browsers. ([Docs.rs][6])

**Backend‑for‑Frontend (BFF)** siting: co‑locate it with your Rust web server (shared code + tower layers) or as a separate service; keep it thin: mapping, pagination, composition, and auth decoration. ([Sam Newman][2])

---

## 2) Rust backend architecture (frameworks, layers, and idioms)

**Framework & runtime**

* **axum** on **Tokio** is the default these days. Axum leans on the **Tower** ecosystem for middleware (timeouts, CORS, auth, tracing), so “before\_action” in Rails maps to **Tower layers** wrapped around routes. ([Docs.rs][7])
* Add middleware like **CORS** via `tower-http::CorsLayer` and **Trace** via `TraceLayer`. ([Docs.rs][8])

**Concurrency & async gotchas**

* Don’t block the async reactor. For CPU‑heavy or blocking I/O (e.g., bcrypt/argon2, large file ops), offload to **`tokio::task::spawn_blocking`**. This prevents starving other futures. ([Docs.rs][9])

**Project layout** (hexagonal):

* `domain` crate(s): business types, behaviors, invariants.
* `adapters` crate(s): DB repositories (SQLx/Diesel), cache (Redis), messaging (Kafka/Rabbit/NATS).
* `api` crate: axum router, extractors, DTOs (serde).
* `app` crate (the binary): wire everything via dependency injection (constructors), load config, serve.

**Configuration**

* Use layered config with `config` or `figment` and `serde` for typed settings. Keep secrets in env/secret stores; keep a single “settings.rs” that all crates import. ([Crates][10])

**Data access**

* **SQLx**: async, hand‑written SQL, **compile‑time checked** queries (online or **offline mode** with `cargo sqlx prepare`). Great when you want explicit SQL and top‑tier type safety. ([GitHub][11])
* **Diesel**: synchronous but **compile‑time safe query builder/ORM**. If you prefer a “Rusty ActiveRecord,” Diesel enforces schema safety at compile time. ([diesel.rs][12])
* Migrations: use each tool’s CLI and gate deploys with “migrate up → healthcheck → flip traffic”.

**Caching & sessions**

* Redis works well (connection pools via e.g., `deadpool-redis`). Prefer cookie sessions for browser apps. Tie session IDs to server data; don’t push identity into long‑lived JWTs unless you need statelessness across many edges.

**Background jobs & messaging**

* **Job runners**: **Apalis** integrates with AMQP/Redis and Tower middleware. ([Docs.rs][13])
* **Brokers**: RabbitMQ (**lapin**), Kafka (**rust‑rdkafka**), NATS (official client). Pick for your latency/delivery needs. ([Docs.rs][14])

**Observability**

* Structured logs and spans with `tracing`; add **OpenTelemetry** exporters for traces/metrics; add `TraceLayer` to HTTP. (Axum/Tower ecosystem.)

**Error handling**

* Library‑style errors via `thiserror`; app‑level via `anyhow`. Map to HTTP responses (`IntoResponse`) with consistent problem details. Avoid `unwrap()/expect()` in handlers; surface context and return typed errors.

**Security basics**

* Password hashing: Argon2.
* Auth tokens: if you do JWT, keep **short TTL**, rotate keys, and avoid putting tokens in `localStorage` (XSS). Prefer **HttpOnly, Secure, SameSite cookies** for browser auth; mitigate CSRF (see below). ([MDN Web Docs][15])
* Rate limiting: tower‑governor or custom Tower layer.
* Authorization: policy engines like **Oso** or **Casbin** if you need declarative RBAC/ABAC. (Keep enforcement in middleware and per‑handler checks.) ([Docs.rs][16])

**API documentation & types**

* Generate OpenAPI (e.g., utoipa) and **typed TS clients** with **openapi‑generator’s `typescript-fetch`** or `openapi-fetch` to keep frontend compile‑safe. ([OpenAPI Generator][4])

**Testing**

* Unit + property tests (`proptest`) for domain invariants.
* Integration tests spin up **real deps** using **testcontainers‑rs** (Postgres/Redis/Kafka) so you test the same stack you deploy. ([Docs.rs][17])

---

## 3) React frontend architecture

**Rendering strategy**

* Choose SPA (Vite) or app‑router SSR (Next.js). SSR improves TTFB/SEO; SPA is simpler. Both pair well with a BFF.

**Data fetching**

* Centralize remote state with **TanStack Query** (caching, dedupe, retries, stale‑while‑revalidate). Co‑locate queries with components; let the BFF give you **exact** shapes the UI needs.

**Type safety**

* Generate API clients and types from OpenAPI (above). Keep Zod (or similar) for runtime validation of untyped inputs (e.g., from 3rd‑party APIs). ([OpenAPI Generator][4])

**Security on the frontend**

* **Don’t store auth tokens in `localStorage`**; it’s trivially readable via XSS. Favor **HttpOnly** cookies with `Secure` and an appropriate `SameSite` setting; treat CSRF explicitly (see below). ([MDN Web Docs][18])
* **CSRF**: if you use cookie‑based sessions, implement token‑based CSRF defenses (synchronizer or double‑submit); `SameSite` helps but is **not** a complete defense. ([OWASP Cheat Sheet Series][19])
* Keep a **strict CSP**, escape untrusted content, avoid `dangerouslySetInnerHTML` unless sanitized.

---

## 4) Rails → Rust/React concept map (mental translation)

| Rails thing                          | Rust/React counterpart                                                                                 |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------ |
| Controller filters (`before_action`) | Tower middleware layers on axum routes (auth, rate‑limit, logging) ([Docs.rs][7])                      |
| ActiveRecord models                  | SQLx repositories (explicit SQL) or Diesel schema/query builder ([GitHub][11])                         |
| ERB/Haml views                       | React components; SSR/CSR via Next.js/Vite                                                             |
| CSRF helpers                         | CSRF tokens + `SameSite` cookies + server validation (OWASP guidance) ([OWASP Cheat Sheet Series][19]) |
| `config/` & credentials              | `config`/`figment` + env secrets, typed settings (serde) ([Crates][10])                                |
| Sidekiq/ActiveJob                    | Apalis + AMQP/Kafka/NATS; or cron scheduler (e.g., apalis + tokio) ([Docs.rs][13])                     |

---

## 5) Anti‑patterns to avoid (the “Rails‑shaped holes”)

**Architecture**

* **Big‑bang rewrites.** Strangle incrementally and verify with contract tests; version your APIs. ([martinfowler.com][1])
* **Premature microservices.** Start modular monolith; split only when the pain is real.
* **BFF that becomes a second monolith.** Keep it thin: aggregation, mapping, auth.

**Rust specifics**

* **Blocking in async** handlers (file I/O, hashing, templating) → use `spawn_blocking`. ([Docs.rs][9])
* **`unwrap()`/`expect()` in request paths.** Propagate typed errors; log with context.
* **Global mutable state** or casual `Arc<Mutex<...>>` everywhere. Prefer per‑request state, fine‑grained locks, or immutability + message passing.
* **Leaky lifetimes & long‑held locks** in handlers (deadlocks under load). Keep critical sections tiny.
* **Ad‑hoc auth** (rolling your own crypto/JWT) and long‑lived tokens. Use vetted crates and short TTLs; prefer HttpOnly cookies for browser apps. ([MDN Web Docs][15])

**React specifics**

* **Ad‑hoc `fetch` scattered across components.** Centralize with TanStack Query; coalesce requests in the BFF.
* **Tokens in `localStorage`.** Opens the door for XSS‑driven session theft. Use cookies + CSRF protection. ([MDN Web Docs][18])

---

## 6) Security checklist (browser apps)

* Auth cookies: `HttpOnly; Secure; SameSite=Lax` (or `Strict` where UX allows). Use `SameSite=None` **only** with HTTPS and when truly needed (cross‑site iframes). ([MDN Web Docs][15])
* CSRF: token pattern (synchronizer/double submit) on state‑changing requests; don’t rely on SameSite alone. ([OWASP Cheat Sheet Series][19])
* Input validation: serde (server) + runtime schema validators (client).
* Rate‑limit and log suspicious behavior at the BFF and API edges.

---

## 7) Build & delivery patterns

* **Workspaces**: organize server into multiple crates (domain, adapters, api).
* **Docker**: multi‑stage builds; speed up with **cargo‑chef** (cache deps), and use **cross** if you need cross‑compiles. ([GitHub][20])
* Observability: ship traces/metrics (OTel) and structured logs by default.

---

## 8) A crisp migration plan (ordered)

1. **Set the façade:** proxy in front; route `/health` & a static asset through Rust to validate deploys.
2. **Introduce the BFF:** React talks to the BFF; the BFF initially calls Rails. Add Pact/contract tests between them. ([Pact Docs][3])
3. **Port read endpoints first:** low‑risk parity checks; keep Rails DB the source of truth.
4. **Move writes with care:** cut one aggregate at a time; run DB migrations gated by deploy; avoid dual‑writes.
5. **Retire Rails features:** flip traffic in the proxy; monitor SLOs; delete dead code.

---

## 9) What to pick on day one (sane defaults)

* **axum** + Tower; **Tokio** runtime; `tower-http::{TraceLayer,CorsLayer}`. ([Docs.rs][7])
* **SQLx** (or **Diesel** if you love compile‑time query builders); `serde` everywhere. ([GitHub][11])
* **Apalis** for jobs; **lapin / rust-rdkafka / NATS** for messaging if/when needed. ([Docs.rs][13])
* **OpenAPI** + `typescript-fetch` client for React; **TanStack Query** on the frontend. ([OpenAPI Generator][4])
* **Auth**: HttpOnly cookies, CSRF tokens, Argon2 for passwords; keep JWTs short‑lived if used. ([MDN Web Docs][15])
* **Testing**: unit + property tests; **testcontainers** for integration. ([Docs.rs][17])

---

### Closing thought (and next move)

Treat this as a **domain‑first** rewrite, not a line‑for‑line port: sketch your aggregates and invariants, lock the external API contract, and let axum + SQLx do the heavy lifting while React handles the experience. If you share a couple of representative Rails routes/models, I’ll outline the exact Rust/React slices—including tower layers, extractors, DB repo signatures, and a matching React Query setup—to get your first feature across the bridge.

[1]: https://martinfowler.com/bliki/StranglerFigApplication.html?utm_source=chatgpt.com "Strangler Fig"
[2]: https://samnewman.io/patterns/architectural/bff/?utm_source=chatgpt.com "Pattern: Backends For Frontends"
[3]: https://docs.pact.io/implementation_guides/rust/pact_consumer?utm_source=chatgpt.com "Pact test DSL for writing consumer pact tests in Rust"
[4]: https://openapi-generator.tech/docs/generators/typescript-fetch/?utm_source=chatgpt.com "Documentation for the typescript-fetch Generator"
[5]: https://alistair.cockburn.us/hexagonal-architecture?utm_source=chatgpt.com "hexagonal-architecture - Alistair Cockburn"
[6]: https://docs.rs/tonic?utm_source=chatgpt.com "tonic - Rust"
[7]: https://docs.rs/axum/latest/axum/?utm_source=chatgpt.com "axum - Rust"
[8]: https://docs.rs/tower-http/latest/tower_http/cors/index.html?utm_source=chatgpt.com "tower_http::cors - Rust"
[9]: https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html?utm_source=chatgpt.com "spawn_blocking in tokio::task - Rust"
[10]: https://crates.io/crates/config?utm_source=chatgpt.com "config - crates.io: Rust Package Registry"
[11]: https://github.com/launchbadge/sqlx?utm_source=chatgpt.com "launchbadge/sqlx"
[12]: https://diesel.rs/?utm_source=chatgpt.com "Diesel is a Safe, Extensible ORM and Query Builder for Rust"
[13]: https://docs.rs/apalis?utm_source=chatgpt.com "apalis - Rust"
[14]: https://docs.rs/lapin?utm_source=chatgpt.com "lapin - Rust"
[15]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Set-Cookie?utm_source=chatgpt.com "Set-Cookie header - HTTP - MDN - Mozilla"
[16]: https://docs.rs/oso/?utm_source=chatgpt.com "oso - Rust"
[17]: https://docs.rs/testcontainers/latest/testcontainers/?utm_source=chatgpt.com "testcontainers - Rust"
[18]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Cookies?utm_source=chatgpt.com "Using HTTP cookies - MDN - Mozilla"
[19]: https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html?utm_source=chatgpt.com "Cross-Site Request Forgery Prevention Cheat Sheet"
[20]: https://github.com/LukeMathWalker/cargo-chef?utm_source=chatgpt.com "LukeMathWalker/cargo-chef"
