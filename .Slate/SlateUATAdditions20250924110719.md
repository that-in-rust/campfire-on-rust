Title: UAT Additions — WS Reconnect/Presence + Message Dedup (SlateUATAdditions20250924110719)

Scope
- Fortify hot paths prior to release.

Tests to add
- WebSocket reconnect + missed delivery
  - Precondition: user joined room; last_seen recorded
  - Steps: connect → JoinRoom → disconnect → others post → reconnect → verify missed messages delivered and UpdateLastSeen processed; presence leave/join broadcasts observed
- Presence and typing
  - Steps: A and B join → StartTyping/StopTyping from A → B sees typing indicators; leave events broadcast
- Message dedup (HTTP + WS)
  - Steps: send same client_message_id twice rapidly → only one message stored; second returns existing; 201 Created; timeline contains a single instance
- Rate limit and bounds
  - Content length boundaries (1..=10000) and any request rate constraints → 400/429 as appropriate
- Room access guards
  - Unauthorized GET /api/rooms/:id/messages → 403; non-existent → 404
- WS auth modes
  - Query vs Authorization: Bearer vs Cookie session_token

Expected results
- Consistent 201 on create; strict errors on unauthorized/invalid inputs
- Presence/typing and reconnect flows visible over WS
