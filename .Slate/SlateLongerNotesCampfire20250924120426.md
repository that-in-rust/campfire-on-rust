Title: Campfire Deconstruction — Longer Notes (Chunked) — 2025-09-24 12:04:26
Session TS: 20250924120426
Target: ./zzzzArchive/reference-materials/_LLMcampfiretxt/basecamp-once-campfire-8a5edab282632443.txt
Kiro specs location detected: ./.kiro/specs (campfire-rust-rewrite)

Method
- Read in 300-line chunks and extract concrete capabilities (controllers, models, jobs, JS controllers, channels, etc.).
- Map findings to Kiro: Spec → Requirement → Design → Tasks.
- Track “Missing vs Kiro specs” deltas to accelerate MVP closure.
- Commit this longer notes file after every chunk; synthesize .Slate/SlateCampfireDeconstruct20250924120426.md after each 3 chunks.

Chunk 1 (lines 1–300) — Highlights
Scope observed (directory listing):
- Assets
  - Images/sounds placeholders; stylesheets covering broad UI: base, inputs, layout, messages, panels, sidebar, signup, utilities, etc.
- ActionCable Channels
  - heartbeat_channel, presence_channel, read_rooms_channel, room_channel, typing_notifications_channel, unread_rooms_channel
  - application_cable/{channel.rb, connection.rb}
- Controllers (top-level)
  - accounts, application, first_runs, messages, pwa, qr_code, rooms, searches, sessions, unfurl_links, users, welcome
  - Namespaced:
    - accounts/* (bots, custom_styles, join_codes, logos, users; plus bots/keys)
    - autocompletable/users
    - concerns/* (allow_browser, authentication, authorization, room_scoped, set_current_request, set_platform, tracked_room_visit, version_headers; authentication/session_lookup)
    - messages/* (boosts, by_bots)
    - rooms/* (closeds, directs, involvements, opens, refreshes)
    - sessions/transfers
    - users/* (avatars, profiles, push_subscriptions, sidebars; push_subscriptions/test_notifications)
- Helpers
  - Wide helper surface: content_filters, messages, time, translations, users, version, etc. with nested helpers (messages/attachment_presentation, rooms/involvements)
- JavaScript (Stimulus + libs)
  - controllers/* include presence, typing_notifications, turbo_streaming, rooms_list, message UI, notifications, sound, search_results, etc.
  - initializers/* include current/highlight/rich_text
  - lib/* provides autocomplete framework and rich_text unfurling
  - models/* client-side state: client_message, message_paginator, typing_tracker, etc.
- Jobs
  - bot/webhook_job; room/push_message_job
- Models
  - Core: account, membership, message, room, session, user; plus search, sound, push, purchaser, current, first_run
  - Message concerns: attachment, broadcasts, mentionee, pagination, searchable
  - Rooms variants: rooms/{closed, direct, open}
  - Room/message_pusher; push/subscription; user modules (avatar, bot, mentionable, role, transferable)
- Views (start of) accounts/* etc.

Observed Basecamp style
- MVC with ActionCable channels for realtime; Stimulus controllers for UI behaviors.
- Helpers keep views slim; model concerns modularize behaviors (broadcasts, pagination, searchable).
- Jobs isolate async work (push_message, webhook).
- Namespacing by feature domain (accounts, messages, rooms, users).

Initial Kiro Mapping (Spec → Requirement → Design → Tasks)
Note: Cross-reference against ./.kiro/specs/campfire-rust-rewrite.

1) Real-time Presence and Typing
- Spec signals: presence_channel, typing_notifications_channel; JS presence_controller, typing_notifications_controller; room_channel; unread_rooms_channel.
- Requirement: Show who’s online/in-room; per-room typing indicators.
- Design (Rust/Axum):
  - WS messages: JoinRoom, LeaveRoom, StartTyping, StopTyping, UpdatePresence, UnreadCount.
  - Presence registry per room with TTL; typing TTL (2–5s); throttle typing broadcasts.
- Tasks:
  - Presence store (HashMap<RoomId, HashSet<UserId>> with expiry).
  - Typing tracker per room.
  - WS handlers + broadcast fanout.
  - Optional UI events integration.
- Missing vs Kiro: Ensure explicit presence/typing requirements; add if absent.

2) Message Delivery and Unread Tracking
- Spec signals: unread_rooms_channel, read_rooms_channel, room/message_pusher, message_paginator, client-side client_message idempotency.
- Requirement: Accurate unread counts; mark-as-read on view; missed delivery on reconnect.
- Design:
  - last_seen per user+room; unread = created_at > last_seen.
  - Endpoints for mark read; server push updates.
- Tasks:
  - Persist last_seen.
  - GET unread summary; broadcast on changes.
  - Reconnect backfill since last_seen.
- Missing vs Kiro: Verify read/unread coverage; add acceptance criteria.

3) Search and Content Filters
- Spec signals: models/search.rb; helpers/content_filters (sanitize/mentions); messages/searchable.rb; JS search_results_controller.
- Requirement: Full-text message search; sanitized content with mentions/embeds.
- Design: SQLite FTS5; sanitize on render; mention linking.
- Tasks: Ensure FTS5 setup; mention parsing; filter hooks.
- Missing vs Kiro: Add specifics for filters/mentions if not present.

4) Push Notifications
- Spec signals: push/subscription model; users/push_subscriptions_controller; test_notifications; pwa_controller.
- Requirement: Web Push with VAPID; per-device subs; test endpoint.
- Design: Store subscription; VAPID keys; triggers on @mention/DM.
- Tasks: CRUD APIs; validation; test route.
- Missing vs Kiro: Triggers, quiet hours, rate limits.

5) Accounts and Bots
- Spec signals: accounts/* (bots, users, logos, custom_styles, join_codes); bot/webhook_job; messages/by_bots_controller.
- Requirement: Multi-tenant accounts; bot API keys; inbound webhook.
- Design: Account-scoped resources; HMAC/signature; scoped bot keys.
- Tasks: Bot message endpoint; key management; scoping middleware.
- Missing vs Kiro: Security/limits for bots.

6) Media and Unfurling
- Spec signals: message/attachment; content_filters; opengraph fetch pipeline (document/fetch/location/metadata).
- Requirement: URL unfurling; attachments; sanitize + cache.
- Design: Async OG fetch + cache; partial render; rate limit domains.
- Tasks: OG fetcher; background job; cache keys; view partials.
- Missing vs Kiro: Safety & rate-limiting specs.

Next
- Enumerate ./.kiro/specs/campfire-rust-rewrite to bind concrete spec items to the above.
- Continue to Chunk 2 (lines 301–600) and expand mapping with acceptance criteria and tasks.

Chunk 2 (lines 301–600) — Highlights
Views expansion
- Accounts area (bots, users, custom_styles) with CRUD templates and partials; autocompletable users JSON (jbuilder).
- ActionText: _opengraph_embed.html.erb (attachable).
- Layouts: application, lightbox, mailer; ActionText contents partial.
- Messages views: _actions, _message, _presentation, _template, _unrenderable; turbo_stream templates for create/destroy; room_not_found.
- PWA: install/system/browser settings, manifest.json.erb, service_worker.js.
- Rooms: show; subnamespaces for closed/direct/open with form/user partials; refreshes with turbo_stream; show composer/invitation/nav partials.
- Searches: index.html.erb; Sessions: new + incompatible browser + transfers.
- Users: mention partial, avatars (SVG), profiles (membership/transfer), push_subscriptions (partial + index), sidebars (rooms direct/shared).
- Config directory layout: application.rb, boot.rb, routes.rb, initializers (CSP, assets, session_store, time_formats, vapid, web_push), envs, locales.
- DB: structure.sql and migrations (sessions table, webhooks, custom styles, ActiveStorage variants).
- Lib: rails_ext (filters, ActionText OG embeds), restricted_http/private_network_guard, tasks, web_push (notification, pool).
- Tests: broad coverage across controllers, channels, models, system (sending/boosting/unread), helpers; fixtures for accounts/memberships/messages/rooms/sessions/push.

Implications for our MVP (Rust/Axum)
- Server-rendered UI coverage:
  - Rooms: list/show with unread badges; per-room composer/partials; presence indicators.
  - Messages: item/presentation partials; turbo-like updates (we can mirror via HTMX/turbo-like endpoints or WebSocket broadcasts).
  - PWA: manifest + service worker; settings partials.
- Helpers-first approach: formatting/time/mentions/sanitize to keep templates thin.
- Turbo streams: mirror via minimal fragments/WS or HTTP endpoints that return fragments.

Kiro mapping deltas (to validate in ./.kiro/specs/campfire-rust-rewrite)
- Views/UX
  - Requirement: Unread badge dots; room show with composer; turbo-like partial updates.
  - Design: Askama/Minijinja; helper modules; fragment endpoints for partial updates.
  - Tasks:
    - Templates: rooms/show (with composer/nav), messages/_item/_presentation, users/profiles, searches/index.
    - Helpers: time formatting, mention linking, sanitization.
    - Turbo-like flows: create/destroy return fragments; WS to insert/remove DOM nodes.

- PWA + Service Worker
  - Requirement: Manifest and SW; optional install instructions; cache policy.
  - Design: Static endpoints; SW script versioning; cache routes.
  - Tasks: templates + static routes; SW scaffold.

- ActionText/Unfurl presentation parity
  - Requirement: OpenGraph embed partial; safe HTML.
  - Design: OG renderer with sanitized HTML.
  - Tasks: unfurl partials + sanitizer.

- Test scaffolding
  - Requirement: System/UAT for send message, boosts, unread.
  - Tasks: replicate analogous tests in Rust test harness/UAT suite.

Chunk 3 (lines 601–900) — Highlights
README
- Single-tenant app; features: multi-room, DMs, attachments (previews), search, Web Push, @mentions, bot API.
- Docker deploy: single-machine includes web, jobs, caching, file serving, SSL; env vars: SSL_DOMAIN, DISABLE_SSL, VAPID keys, SENTRY_DSN.

config.ru
- Rack entrypoint with Rack::Deflater; Rails app load and load_server.

Dockerfile
- Multi-stage build; jemalloc; installs sqlite3, redis, ffmpeg, libvips; precompiles assets; non-root user; timeouts; exposes 80/443; boot via bin/boot.

Gemfile (key dependencies)
- rails (main), sqlite3, redis
- resque + resque-pool (jobs)
- propshaft + importmap-rails (assets), turbo-rails + stimulus-rails (Hotwire)
- image_processing, web-push, rqrcode, rails_autolink
- geared_pagination (pagination)
- jbuilder (JSON views)
- net-http-persistent (HTTP client for OG fetch, webhooks)
- kredis, platform_agent, thruster
- sentry (observability)
- dev/test: debug, rubocop, faker, brakeman; test: capybara, mocha, selenium-webdriver, webmock

Procfile
- web: thrust bin/start-app; redis server; workers via resque-pool.

Rakefile + .dockerignore
- Rails load_tasks; dockerignore covers logs/tmp/.env, etc.

Implications for our MVP (Rust/Axum)
- Jobs: resque/resque-pool → need a job runner (e.g., Tokio background tasks or lightweight queue) for push/unfurl/webhook.
- Hotwire stack: Turbo/Stimulus → consider HTMX/Turbo-compatible fragments; we already have WebSocket for pushes.
- Pagination: align on geared_pagination-like cursors (we have message pagination endpoints; ensure UI parity).
- Observability: wire Sentry equivalent (optional but useful).
- Docker: Provide a single-image deployment with SSL option (Caddy/Traefik or built-in reverse proxy). Provide envs for VAPID keys.
- HTTP persistent client: ensure pooling for OG fetch and webhooks.

Kiro mapping deltas
- Background jobs
  - Requirement: async message push, OG fetch, webhook invocation.
  - Design: Task queue with retry/backoff; bounded concurrency.
  - Tasks: job trait + executor; jobs for push_message, unfurl_fetch, webhook_call.

- Deployment
  - Requirement: dockerized single-node with SSL, redis-like cache analog (optional).
  - Tasks: Dockerfile; compose; env toggles; VAPID keys script equivalent.

- Libraries parity
  - Requirement: feature coverage without Rails-specific gems.
  - Tasks: confirm alternatives in Rust stack; document deltas.


Chunk 4 (lines 901–1200) — Highlights
Operational concerns and packaging (continuation)
- Public assets and error pages (404/422/500/502) and robots.txt indicate production-friendly defaults.
- GitHub workflows present for CI and publishing images (implies automated builds/tests).
- README and deploy guidance emphasize single-machine Docker deployment with SSL and VAPID configuration.
- Procfile defines processes: web (thrust), redis, workers via resque-pool (jobs). Suggests job queue is first-class.

Implications for our MVP (Rust/Axum)
- Provide production defaults:
  - Public error pages and a minimal robots.txt (or route-based equivalents).
  - CI workflow (optional now), but at least a Docker image publish path.
- Process model:
  - Web server + background worker(s). We can run workers inside the same binary via a background task supervisor or separate processes if needed.
- Redis presence in upstream stack:
  - We can keep SQLite for app data; Redis optional for cache/pubsub. For MVP, keep it minimal (no Redis) unless required for push/presence scaling.

Kiro mapping deltas (to validate in ./.kiro/specs/campfire-rust-rewrite)
- Error handling and ops
  - Requirement: Serve error pages and health endpoints; graceful timeouts.
  - Design: Axum routes for /health; static error pages or templated error handlers; tower timeouts.
  - Tasks: Add health/live/ready endpoints (we have them), ensure error pages exist.

- Jobs & workers
  - Requirement: Dedicated worker execution for push/unfurl/webhooks.
  - Design: Background task executor (Tokio) with bounded concurrency and retry/backoff.
  - Tasks: Worker bootstrap on startup; job scheduling APIs; signal handling for graceful shutdown.

- CI/CD
  - Requirement: Build and optionally publish Docker image; run tests.
  - Tasks: Add GitHub Actions workflow (optional for now); document local build/publish.

Next
- Proceed to Chunk 5 (lines 1201–1500): expect more UI and assets. After Chunk 6, I will update the summary to cover chunks 1–6.

Chunk 5 (lines 1201–1500) — Highlights
CSS and UI foundations (continued)
- ActionText toolbar/editor CSS shows extensive customization: sticky toolbar, themed colors via CSS variables, input sizing, and icon buttons.
- Accessibility considerations: prefers-reduced-motion handling, reasonable defaults for inputs/buttons, and focus on smooth scrolling fallbacks.
- Style system relies on variables (e.g., --color-bg, --color-text) implying a themeable design across panels/messages/composer.
- Asset strategy includes numerous sound thumbnails (webp), indicating a consistent “sounds” feature presentation in UI.

Implications for our MVP (Rust/Axum)
- Theming: adopt a small set of CSS custom properties to theme key surfaces (bg/text/accents).
- Rich text/editor: ensure any composer/editor we use has adequate toolbar behavior and sensible defaults (even if minimal).
- Accessibility: respect reduced-motion preferences; avoid heavy animations by default.
- Sounds feature: confirm our sound endpoints and UI affordances match the intended style (list, info, play/preview).

Kiro mapping deltas (to validate in ./.kiro/specs/campfire-rust-rewrite)
- Theming/Design tokens
  - Requirement: Core CSS variables for colors/spacing to unify UI.
  - Tasks: Add base stylesheet with variables; document in design.md.

- Composer/editor presentation
  - Requirement: Toolbar defaults and link dialogs; safe input sizing.
  - Tasks: Define minimal composer CSS; progressive enhancement later.

- Accessibility
  - Requirement: Reduced-motion handling.
  - Tasks: Global CSS media query; ensure no critical flows depend on animations.

- Sounds UI
  - Requirement: Present available sounds with clear thumbnails; link to sound info/play.
  - Tasks: Template for sounds list/detail and consistent asset paths.

Next
- Continue to Chunk 6 (lines 1501–1800), likely more UI and assets. Then update the summary to include chunks 4–6.

Chunk 6 (lines 1501–1800) — Highlights
Mentions and autocomplete UI (CSS continues)
- ActionText integration includes custom content types for mentions and OG embeds, with toolbar groups toggled per content type.
- .mention component: inline-flex with avatar sizing, strong emphasis, and precise padding/layout rules; z-index layering for toolbar.
- Autocomplete list styling: max sizes, z-index layering, hover/focus visuals, and avatar alignment inside list buttons.
- Code/pre/blockquote/cite theming continues with variables; dark-mode and prefers-reduced-motion are respected across elements.

Implications for our MVP
- Mentions: ensure end-to-end support (detect @mentions in composer, store, render inline with avatar chip, and notify mentioned users with push).
- Autocomplete: deliver minimal UX for user mentions (lightweight endpoint returning users; simple list with keyboard nav).
- Rich text: even if minimal, ensure safe render with code/blockquote styling aligned to our variables.

Kiro mapping deltas (to validate in ./.kiro/specs/campfire-rust-rewrite)
- Mentions
  - Requirement: Create and render @mentions; link to profiles; push notification on mention.
  - Design: Parser on compose; store mention relations; renderer wraps with avatar + name.
  - Tasks: mentions table; parsing on POST; rendering helper; push trigger on mention.

- Autocomplete for mentions
  - Requirement: Autocomplete @username with keyboard navigation.
  - Design: GET /api/users?q= prefix; returns id, name, avatar; debounce client side.
  - Tasks: API route + query; UI list integration.

- Rich text blocks
  - Requirement: style code/blockquote/cite safely; respect dark-mode and reduced-motion.
  - Tasks: base CSS tokens + safe render.

Next
- Summary has been or will be updated to include chunks 4–6 (1–6 roll-up).
Chunk 8 (lines 2101–2400) — Highlights
Buttons, boosts, dialogs, and notification/PWA UX (CSS)
- Buttons: variables for size, padding, borders; icon handling; hover/focus-visible/disabled states unified via tokens; variants like boost actions.
- Boosts: compact chip-like UI with animated expand/collapse and quick-boost grid; delete visibility toggled by expanded state.
- Dialogs: tokenized width with responsive max-inline-size; gradient backdrops; dedicated close control positioning.
- PWA install affordances: conditional show/hide based on display-mode and install-allowed state.
- Notification settings: collapsible help with disclosure icon rotation; image invert in dark mode; inline code/emphasis tokens.

Implications for our MVP
- Buttons: adopt unified .btn token system for consistent feel; ensure icon images are non-selectable and respect hover/focus tokens.
- Boosts: MVP can ship without boosts UI, but leave room for chip-like reactions; keep animation optional for reduced-motion.
- Dialogs: add baseline dialog tokens and close button placement for settings flows.
- PWA/Notifications: provide install hints (optional) and a notifications help section aligned with Web Push preferences UI.

Kiro mapping deltas (to validate in ./.kiro/specs/campfire-rust-rewrite)
- Button system
  - Requirement: Consistent tokens (size, radius, colors, padding) and icon treatment.
  - Tasks: base buttons.css with tokens; document usage.

- Notifications UX
  - Requirement: Settings page with collapsible help; clear instruction for enabling push/permissions.
  - Tasks: template + controller action; asset inversion for dark mode.

- Dialog scaffolding
  - Requirement: Settings dialogs with consistent width/backdrop and accessible close affordance.
  - Tasks: dialog partial + CSS tokens; focus management plan.

- PWA hints
  - Requirement: Conditional installer affordances.
  - Tasks: minimal JS to toggle .pwa--can-install; template partial.

Next
- Continue to Chunk 9 (lines 2401–2700) to capture more foundational UI tokens and components.
Chunk 9 (lines 2401–2700) — Highlights
Avatar/base/layout foundations (CSS continues)
- Avatars: base avatar component, icon variant, grouped avatars with responsive sizing and grid composition; account-logo sizing in nav contexts.
- Base.css: global font stack; common interactive element behavior (outline-size/offset, hover filters, disabled); fieldset/legend layout for settings.
- Turbo integration: turbo-frame and cable stream sources styled as contents to avoid layout shifts.
- Menus, code/pre and version-badge treatments establish cohesive typography and border tokens.

Implications for our MVP
- Avatar system: implement avatar/icon/group patterns with tokens; ensure nav placement support for account logo.
- Settings forms: fieldset/legend patterns and border variants for consistent admin UX.
- Turbo-friendly markup: avoid layout breaks on fragment updates.

Kiro mapping deltas (to validate in ./.kiro/specs/campfire-rust-rewrite)
- Avatar tokens
  - Requirement: Avatar sizes, icon variant, grouping behavior; account logo in nav.
  - Tasks: avatars.css with variables; helpers to render avatar/group.

- Settings UI
  - Requirement: Fieldset/legend style and border variants.
  - Tasks: base.css additions; template helpers.

- Fragment safety
  - Requirement: Layout-safe fragment updates (WS/HTTP).
  - Tasks: guidance in templates; test with live updates.

Next
- After Chunk 9, produce the 7–9 roll-up into the summary to maintain the 3-chunk cadence.

Chunk 11 (lines 3001–3300) — Highlights
CSS/UX continues (code highlighting, colors)
- Code highlighting tokens (keyword/entity/string/comment/etc.) with light/dark variants using OKLCH.
- Consistent background/typography for code/pre blocks.
- Accessibility: theming respects prefers-color-scheme.

Implications for v0.1
- Keep code.css styles or equivalent to maintain visual parity for snippets.
- Ensure base colors are centralized in tokens.

Slice sample (first ~30 lines):
      @media (min-width: 100ch) {
        --sidebar-width: 26vw;
      }
    }
  }
  
  #app-logo {
    display: none;
  
    @media (min-width: 100ch) {
      block-size: var(--footer-height);
      display: grid;
      filter: saturate(0);
      inline-size: 5vw;
      inset: auto auto 0 0;
      opacity: 0.5;
      padding-inline: 1vw;
      place-items: center;
      position: absolute;
      transition: opacity 500ms ease-in-out, filter 500ms ease-in-out;
  
      & img {
        block-size: auto;
        inline-size: 100%;
        max-inline-size: 2.75em;
      }
  
      &:hover {
        filter: none;
        opacity: 1;

Kiro mapping deltas
- Add color/code tokens to base stylesheet and document in design.md.

Chunk 12 (lines 3301–3600) — Highlights
Colors, composer, embeds (continued)
- Colors.css: OKLCH tokens for bg/text/borders/links/selected/alert; dark-mode redefinitions.
- Composer.css: attachment thumbnails/buttons; minimal rich-text toggle under viewport/hover gating; typing indicator placement.
- Embeds.css: OG-embed card layout (title/description responsive styles).

Implications for v0.1
- Colors: adopt OKLCH tokens for light/dark parity.
- Composer: text-only + minimal rich-text toggle; hide upload controls for v0.1.
- Embeds/unfurls: defer (tracked in backlog).

Slice sample (first ~30 lines):
  }
  
  .message__day-separator {
    align-items: center;
    display: none;
    font-size: 0.8rem;
    font-weight: 600;
    grid-area: sep;
    grid-template-columns: 1fr auto 1fr;
    inline-size: 100%;
    margin-block: var(--message-space);
    text-align: center;
    text-transform: uppercase;
    visibility: hidden;
  
    time,
    span {
      padding: 0.66em 2.33ch;
      background-color: var(--color-message-bg);
      border-radius: 3em;
    }
  
    &::after,
    &::before {
      border-top: 2px solid var(--color-message-bg);
      content: "";
    }
  
    .message--first-of-day & {
      display: grid;

Kiro mapping deltas
- Confirm variables in base stylesheet; keep composer minimal; move OG embeds to backlog.
