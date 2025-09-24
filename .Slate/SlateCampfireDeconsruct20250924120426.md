Title: Campfire Deconstruction — 3-Chunk Summary (1–900) — TS 20250924120426

Scope
- Source: zzzzArchive/reference-materials/_LLMcampfiretxt/basecamp-once-campfire-8a5edab282632443.txt (Rails reference)
- Method: 300-line slices with Kiro mapping (Spec → Requirement → Design → Tasks)
- Style: Basecamp conventions (thin views + helpers, model concerns, ActionCable, Stimulus)

What we saw (Chunks 1–3)
1) Realtime & UI
- Channels: presence, typing_notifications, room, unread/read_rooms; JS controllers: presence, typing, turbo_streaming.
- Requirement: Presence + typing TTL and throttling; unread counts; reconnect backfill.

2) Views and partials
- Messages: item/presentation/template; Rooms: show (composer, nav), closed/direct/open; Users: profiles/avatars/push_subscriptions; PWA support.
- Helpers: content_filters, time formatting, mention linking; ActionText OG embed partial.

3) Config/Env/Initializers/Routes
- CSP, session store, time formats, vapid/web_push; routes.rb; env configs; locales.

4) DB/Jobs/Lib
- Migrations for sessions, webhooks, ActiveStorage variants; lib rails_ext filters and OG embeds; web_push pool/notification.

5) Deploy/Deps
- Dockerfile for single-node deploy (jemalloc, ffmpeg, libvips, redis); resque/resque-pool; Hotwire (turbo/stimulus); image_processing; web-push; jbuilder; geared_pagination; net-http-persistent; sentry.

Gaps vs ./.kiro/specs/campfire-rust-rewrite (to confirm and add if absent)
- Presence & typing: explicit TTL/throttle acceptance criteria; reconnect backfill; unread computation and broadcasts.
- Search & sanitize/mentions: filters and mention semantics.
- Push notifications: VAPID key mgmt, triggers (@mention/DM), test endpoint, quiet hours/rate limits.
- Bots/webhooks: signature verification, retries, limits.
- Unfurl/attachments: async fetch, caching, failure handling, sanitized renders.
- Background jobs: queue, retry/backoff, bounded concurrency.
- Deploy: SSL toggle; env wiring; push keys utility.
- Routes: parity table with status codes (prefer 201 on creates).
- Tests: UAT parity for send message, boosts, unread, reconnect.

Immediate MVP tasks (Design → Tasks)
- Presence/Typing: presence store + typing TTL; WS events; throttling.
- Unread/Last seen: persist last_seen; summary endpoint; WS updates.
- Search: FTS5 index; sanitize + mentions; result views.
- Push: subscription CRUD; VAPID keys setup; test push; triggers for mentions/DMs.
- Bots/Webhooks: inbound bot endpoint with HMAC; webhook job + retries.
- Unfurl: OG fetch job + cache; sanitized embed partial.
- Jobs: simple executor w/ retry/backoff; define job types.
- Views: templates + helpers for rooms/messages; fragment endpoints.
- Deploy: Dockerfile + env; service worker + manifest.

References
- Longer notes: .Slate/SlateLongerNotesCampfire20250924120426.md
- Kiro snapshot:
  # .kiro/specs/campfire-rust-rewrite (snapshot)
  design.md
  image.png
  Rails_Legacy.md
  requirements.md
  tasks.md
