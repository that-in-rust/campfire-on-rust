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
