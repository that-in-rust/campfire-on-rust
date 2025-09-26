Title: Backlog Identified for v0.1 (Deferred) — TS 20250924120426

These items are intentionally deferred from v0.1 to keep the app reasonably functional with the same look/feel while shipping as a Rust binary.


## Moved to backlog: Push Notifications (refs snapshot)

From design.md:
40:    40	            PUSH[Push Notification<br/>Service]
146:   146	- **Push Notifications**: web-push crate with VAPID keys
933:   933	2. Push notifications with Web Push

From tasks.md:
152:   152	  - Implement typing notification system with WebSocket messages
170:   170	  - Add support for @mentions with user linking and notifications
176:   176	- [x] **5.3 Push Notifications**
177:   177	  - Add web-push crate dependency and implement Web Push with VAPID keys
178:   178	  - Create push notification service with subscription management
179:   179	  - Add notification preferences per user in database
180:   180	  - Implement notification triggers for mentions and DMs
181:   181	  - Add push notification endpoints for subscription management
182:   182	  - _Requirements: Requirement 8.1-8.6 (push notifications)_
437:   437	      - Create VAPID key configuration for push notifications

