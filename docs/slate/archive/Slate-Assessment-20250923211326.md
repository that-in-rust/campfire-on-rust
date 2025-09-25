# Slate Snapshot — 2025-09-23 21:13:26

Executive summary
- Campfire-on-Rust is a single-binary Axum app with feature-flagged subsystems (websockets, search, sounds, push, bot).
- Parseltongue gives us fast architectural orientation, precise impact scopes, and auto-generated artifacts (graph + contexts).
- The UI/user journey is split between regular mode and demo mode and is mapped to concrete routes (see below).

Parseltongue assessment (benefits, limits, when to use)
- Strengths
  - Instant orientation: Ingest + architecture.html gives a working mental model in minutes.
  - Impact clarity: “uses” and “calls” quantify change risk before edits; great for PRs and refactors.
  - Context packs: generate-context outputs compactly describe dependencies and edges for traits/structs/functions.
  - Fast iteration: HTML graph generation and queries are effectively real-time at this repo’s size.
- Limits
  - Name sensitivity: Queries require exact entity names; use debug --graph | grep to discover names first.
  - Generics/macros: Limited expansion details; analyze concrete implementors and usage sites instead.
  - Some query types demand a target (e.g., find-cycles), so prefer generate-context + uses/calls for broad scans.
- Best uses in this repo
  - Change impact on MessageService/RoomService/AuthService before shipping.
  - Debugging “where is this called” and “who depends on this type.”
  - Documentation: architecture.html + selected context files for onboarding.
- Overall: Worth integrating into daily dev flow (pre-PR impact, onboarding, debugging). Low setup, high signal.

Creative parseltongue usage in this snapshot
- Impact radar (measured counts)
  - AppState: uses = 11
  - AuthService: uses = 5
  - RoomService: uses = 4
  - MessageService: uses = 4
  - SearchService: uses = 4
  - PushNotificationServiceImpl: uses = 4
  - BotServiceImpl: uses = 5
  - ConnectionManagerImpl: uses = 4
  - create_message_with_deduplication: callers = 17
- Interpreting the radar
  - High “uses” = higher centrality; treat changes as medium/high risk and test broadly.
  - Many callers = entry point; keep contracts stable, consider adapters for breaking changes.
  - Message creation is hot (17 callers) → prioritize UAT coverage and backward-compatibility.

Route inventory (from src/main.rs)
- Pages & assets
  - GET  / → pages::serve_root_page (demo-aware)
  - GET  /chat → assets::serve_chat_interface
  - GET  /login → pages::serve_login_page (demo-aware)
  - GET  /demo → assets::serve_demo_page
  - GET  /manifest.json → assets::serve_manifest
  - GET  /static/*path → assets::serve_static_asset
- Demo API
  - GET  /api/demo/status → pages::demo_status
  - POST /api/demo/initialize → pages::initialize_demo
- Health & metrics
  - GET  /health, /health/ready, /health/live
  - GET  <metrics endpoint>, GET /metrics/summary (if metrics enabled)
- WebSocket
  - GET  /ws → handlers::websocket::websocket_handler (if websockets enabled)
- Core API
  - POST /api/auth/login → auth::login
  - POST /api/auth/logout → auth::logout
  - GET  /api/users/me → users::get_current_user
  - GET  /api/rooms → rooms::get_rooms
  - POST /api/rooms → rooms::create_room
  - GET  /api/rooms/:id → rooms::get_room
  - POST /api/rooms/:id/members → rooms::add_room_member
  - GET  /api/rooms/:id/messages → messages::get_messages
  - POST /api/rooms/:id/messages → messages::create_message
- Search (if enabled)
  - GET  /api/search → search::search_messages
- Sounds (if enabled)
  - GET  /api/sounds
  - GET  /api/sounds/:sound_name
  - GET  /api/sounds/:sound_name/info
- Push (if enabled)
  - POST /api/push/subscriptions
  - DELETE /api/push/subscriptions/:id
  - GET /api/push/preferences
  - PUT /api/push/preferences
  - GET /api/push/vapid-key
  - POST /api/push/test (debug only)
- Bot API (if enabled)
  - GET/POST /api/bots
  - GET/PUT/DELETE /api/bots/:id
  - POST /rooms/:room_id/bot/:bot_key/messages

User journey (screen-by-screen, mapped)
- Mode detection
  - GET / and GET /login are demo-aware based on presence of admin@campfire.demo.
  - Demo mode shows demo landing/login pages; regular mode shows standard login/chat.
- Login
  - GET /login → page template (demo or regular)
  - POST /api/auth/login → 200, sets session cookie; POST /api/auth/logout → 200, clears cookie
- Rooms index
  - GET /api/rooms populates sidebar/dashboard; create room with POST /api/rooms (201)
  - GET /api/rooms/:id checks access and returns room metadata (authorize view)
  - POST /api/rooms/:id/members (201) for admin membership ops
- Room chat
  - GET /api/rooms/:id/messages delivers history {messages, has_more}
  - POST /api/rooms/:id/messages deduplicates via client_message_id, returns 201
  - GET /ws authenticates via query/header/cookie; events: CreateMessage, Join/LeaveRoom, Typing start/stop, UpdateLastSeen; broadcasts for messages/presence
- DMs
  - Use room_type=Direct with room creation; flow mirrors regular rooms
- Search
  - GET /api/search across authorized rooms (sanitized query, validated constraints)
- Push & sounds
  - Push endpoints manage subscriptions/preferences and expose VAPID key if configured
  - Sounds endpoints list and serve sound metadata; in-chat /play uses this
- Demo journeys
  - GET / → /demo landing and demo-aware login
  - /api/demo/status, /api/demo/initialize for one-click demo population
  - Pre-seeded rooms/messages to evaluate features quickly

Artifacts generated
- Visualization: parseltongue_workspace/analysis_20250923193618/architecture.html
- Contexts: parseltongue_workspace/analysis_20250923193618/context_*.{txt,json}
- ISG snapshot at ingestion time: 541 nodes, 894 edges

Actionable next uses of parseltongue
- Pre-change risk checks (MessageService/RoomService/AuthService): run uses + calls and paste counts into PRs.
- Debugging playbook: query calls <function>, generate-context <function>, follow in architecture.html to find stateful edges (DB, WS).
- Documentation: regenerate architecture.html on notable merges; commit alongside CHANGELOG.

Notes and decisions
- Favor 201 Created for create endpoints (rooms/messages/members).
- Keep WS auth flexible (token in query/header/cookie) to simplify clients.
- Demo mode remains feature-flagged; can be disabled without code churn.