# Realistic Implementation Plan - Campfire MVP 1.0

## Overview: "Rails-Equivalent Imperfection" Strategy

This implementation plan focuses on **"works well enough"** rather than **"perfect"** - exactly matching Rails behavior and limitations. We implement only what Rails actually does, accepting Rails-level imperfections as acceptable for MVP.

**Core Philosophy**:
- **Rails Parity Rule**: If Rails doesn't do it perfectly, we don't need to either
- **"Good Enough" Quality**: Match Rails reliability, not theoretical perfection
- **5 Critical Gaps Only**: Fix only gaps that Rails actually solves
- **Accept Known Limitations**: Document and accept Rails-equivalent limitations

**Success Criteria**: Works as well as Rails ActionCable, with similar limitations and edge cases.

## MVP 1.0 Focus: Complete UI with Text-Only Backend

**What's Included in MVP 1.0**:
- ✅ **Complete React UI** - All components, styling, and interactions (26 CSS files)
- ✅ **Rich text messaging** - Trix editor with HTML formatting, sounds, boosts
- ✅ **Real-time features** - WebSocket broadcasting, presence, typing notifications
- ✅ **Room management** - Open/Closed/Direct rooms with membership controls
- ✅ **Authentication** - Session management, bot integration, role-based access
- ✅ **Search functionality** - FTS5-powered message search
- ✅ **Push notifications** - Web Push with VAPID keys
- ✅ **Sound system** - 59 embedded MP3 files with /play commands

**What's Gracefully Deferred to v2.0**:
- 🚫 **File attachments** - Complete UI shown with "Coming in v2.0" messaging
- 🚫 **Avatar uploads** - Text-based initials with upload UI for future
- 🚫 **OpenGraph previews** - Link detection with placeholder for future unfurling

## 5 Critical Gaps That Rails Actually Solves

**Gap #1: client_message_id Deduplication**
- **Rails Reality**: Uses database UNIQUE constraints for duplicate prevention
- **Our Fix**: Add UNIQUE constraint on (client_message_id, room_id)
- **Requirements**: Requirement 1.14 - prevent duplicate messages from rapid clicking

**Gap #2: WebSocket Reconnection State**
- **Rails Reality**: ActionCable tracks connection state for missed message delivery
- **Our Fix**: Track last_seen_message_id per connection, send missed messages
- **Requirements**: Requirement 4.14 - state reconciliation after reconnection

**Gap #3: SQLite Write Serialization**
- **Rails Reality**: Connection pooling effectively serializes writes
- **Our Fix**: Dedicated Writer Task pattern with mpsc channel
- **Requirements**: Requirement 4.11 - handle concurrent updates properly

**Gap #4: Session Token Security**
- **Rails Reality**: Uses SecureRandom for session tokens with proper validation
- **Our Fix**: Implement Rails-equivalent secure token generation
- **Requirements**: Requirement 3.3 - secure session management

**Gap #5: Basic Presence Tracking**
- **Rails Reality**: Simple connection counting with heartbeat cleanup
- **Our Fix**: HashMap<UserId, connection_count> with 60-second TTL
- **Requirements**: Requirement 4.2-4.7 - presence tracking with connection management

## Rails-Level Limitations We Accept (Don't Over-Engineer)

**Limitation #1: Imperfect Message Ordering**
- **Rails Reality**: Uses created_at timestamps, occasional out-of-order acceptable
- **Our Approach**: Database timestamps, no complex vector clocks or coordination
- **Requirements**: Requirement 4.11 - "maintain message order consistency" = Rails-level

**Limitation #2: Multi-tab Connection Independence**
- **Rails Reality**: Each tab creates independent ActionCable connection
- **Our Approach**: No cross-tab coordination, each connection is separate
- **Requirements**: Requirement 4.10 - "across multiple browser tabs" = Rails behavior

**Limitation #3: Best-Effort WebSocket Delivery**
- **Rails Reality**: ActionCable doesn't guarantee message delivery
- **Our Approach**: Simple broadcast with timeout, no delivery confirmation
- **Requirements**: Requirement 4.1 - "broadcast within 100ms" = best effort like Rails

**Limitation #4: Presence Tracking Delays**
- **Rails Reality**: Connection cleanup has delays, occasional inaccuracy
- **Our Approach**: 60-second heartbeat, accept brief inaccuracy
- **Requirements**: Requirement 4.4 - "handle browser crashes gracefully" = Rails level

---

## Phase 1: Core Infrastructure (Week 1)

**Goal**: Get basic server running with Rails-equivalent patterns

### 1.1 Project Setup

- [ ] **1.1.1 Initialize Rust project with anti-coordination constraints**
  - Create Cargo.toml with minimal dependencies: axum, sqlx, tokio, serde, rust-embed
  - Set up basic project structure (≤50 files total per Requirement 0.8)
  - Configure development environment with Rails-equivalent patterns
  - _Requirements: Requirement 0.1 - direct function calls, single-threaded logic_

- [ ] **1.1.2 Create type-safe domain models**
  - UserId(i64), RoomId(i64), MessageId(i64) newtypes for type safety
  - User, Room, Message structs matching Rails schema from Requirement 7.2
  - UserRole enum (member: 0, administrator: 1, bot: 2) from Requirement 3.7
  - Basic AppError enum with user-friendly messages per Requirement 0.4
  - _Requirements: Requirement 3.7 - role enum, Requirement 0.4 - simple error handling_

- [ ] **1.1.3 Set up SQLite database with Critical Gap Fix #1**
  - Create schema matching Rails conventions from Requirement 7.2
  - Add UNIQUE constraint on (client_message_id, room_id) - **Critical Gap Fix #1**
  - Set up WAL mode and connection pooling per Requirement 6.5
  - Create FTS5 search index for messages per Requirement 7.5
  - _Requirements: Requirement 1.14 - prevent duplicates, Requirement 7.5 - FTS5 search_

### 1.2 Basic HTTP Server

- [ ] **1.2.1 Create Axum server with embedded assets**
  - Basic health check endpoint
  - Embedded React SPA serving using rust-embed per Requirement 8.7
  - CORS and basic middleware matching Rails patterns
  - Static asset serving with proper caching headers per Requirement 6.7
  - _Requirements: Requirement 8.7 - embedded assets, Requirement 6.7 - static serving_

- [ ] **1.2.2 Implement Rails-style session authentication - Critical Gap Fix #4**
  - Secure token generation using SecureRandom equivalent per Requirement 3.3
  - Cookie-based sessions with httponly SameSite=Lax per Requirement 3.3
  - Basic login/logout endpoints matching Rails SessionsController
  - Rate limiting (10 attempts per 3 minutes) per Requirement 3.4
  - _Requirements: Requirement 3.3 - session management, Requirement 3.4 - rate limiting_

---

## Phase 2: Core Chat Functionality (Week 2)

**Goal**: Basic message sending/receiving that works "well enough"

### 2.1 Database Operations with Write Serialization

- [ ] **2.1.1 Implement dedicated writer pattern - Critical Gap Fix #3**
  - Single writer task with mpsc channel for write serialization
  - All writes go through single task (Rails connection pool equivalent)
  - Read operations can be concurrent per Requirement 6.5
  - Handle SQLite SQLITE_BUSY errors gracefully without complex retry
  - _Requirements: Requirement 4.11 - concurrent updates, Requirement 6.5 - connection pooling_

- [ ] **2.1.2 Rich text message CRUD with Rails patterns**
  - create_message with client_message_id deduplication (Critical Gap Fix #1)
  - Support HTML body with Trix formatting per Requirement 1.2
  - get_messages with before/after pagination per Requirement 1.7
  - Handle UNIQUE constraint violations by returning existing message per Requirement 1.14
  - _Requirements: Requirement 1.1-1.2 - message creation, Requirement 1.7 - pagination_

- [ ] **2.1.3 Room management with STI pattern**
  - Room types: Open, Closed, Direct using STI per Requirement 2.10
  - Membership management with involvement levels per Requirement 2.4
  - Simple permission checks (creator/admin or member) per Requirement 3.12
  - Auto-grant Open room memberships per Requirement 2.1
  - _Requirements: Requirement 2.1-2.12 - room management, Requirement 3.12 - authorization_

### 2.2 WebSocket Broadcasting (Rails ActionCable Equivalent)

- [ ] **2.2.1 ActionCable-style connection management**
  - HashMap<RoomId, Vec<WebSocketSender>> for room-based connections
  - Authenticate via session cookies per Requirement 4.8
  - Basic connection cleanup on disconnect (accept Rails-level imperfection)
  - Reject unauthorized connections per Requirement 4.9
  - _Requirements: Requirement 4.8-4.9 - WebSocket authentication and authorization_

- [ ] **2.2.2 Turbo Streams message broadcasting**
  - Broadcast messages using Turbo Streams format per Requirement 4.1
  - broadcast_append_to for new messages, broadcast_replace_to for edits
  - Best-effort delivery within 100ms per Requirement 4.1 (Rails limitation)
  - Simple timeout handling (7 seconds max like webhook timeout)
  - _Requirements: Requirement 4.1 - Turbo Streams broadcasting within 100ms_

- [ ] **2.2.3 Basic reconnection with state sync - Critical Gap Fix #2**
  - Track last_seen_message_id per connection for missed messages
  - Send missed events since last known state per Requirement 4.14
  - Prevent duplicate message delivery during reconnection
  - Accept race conditions in edge cases (Rails ActionCable limitation)
  - _Requirements: Requirement 4.14 - state reconciliation after reconnection_

---

## Phase 3: Real-time Features (Week 3)

**Goal**: "Good enough" real-time features matching Rails quality

### 3.1 Presence Tracking (Rails-Equivalent Imperfection) - Critical Gap Fix #5

- [ ] **3.1.1 Connectable concern pattern for presence**
  - Track connections count per membership per Requirement 4.2
  - connected_at timestamp with 60-second TTL per Requirement 4.2
  - Increment atomically on present, decrement on disconnected per Requirement 4.3-4.4
  - Accept occasional inaccuracy during network hiccups (Rails limitation)
  - _Requirements: Requirement 4.2-4.4 - presence tracking with connection counting_

- [ ] **3.1.2 Heartbeat and visibility management**
  - Refresh connection every 50 seconds per Requirement 4.5
  - 5-second delay for visibility changes per Requirement 4.7
  - Handle tab switching/minimization with present/absent actions
  - Background cleanup for stale connections (Rails equivalent)
  - _Requirements: Requirement 4.5 - refresh heartbeat, Requirement 4.7 - visibility handling_

### 3.2 Typing Notifications (Rails ActionCable Pattern)

- [ ] **3.2.1 TypingNotificationsChannel implementation**
  - Broadcast start/stop actions with user attributes per Requirement 4.6
  - Throttle notifications to prevent spam (Rails pattern)
  - Track active typers per room with simple HashMap
  - Clear indicators on message send per Requirement 4.6
  - _Requirements: Requirement 4.6 - typing notifications with throttling_

- [ ] **3.2.2 Auto-cleanup typing state**
  - Clear typing after 5 seconds of inactivity (Rails timeout pattern)
  - Simple timer-based cleanup without complex coordination
  - Accept edge cases like network interruptions (Rails limitation)
  - Handle multiple connections per user gracefully
  - _Requirements: Requirement 4.6 - typing notification management_

---

## Phase 4: Frontend Integration (Week 4)

**Goal**: React frontend that works with Rails-equivalent backend

### 4.1 Complete React UI with Graceful Degradation

- [ ] **4.1.1 Rich text message interface**
  - Trix editor with HTML formatting per Requirement 1.2
  - Sound commands (/play soundname) with 59 embedded MP3s per Requirement 1.5
  - Message boosts with emoji content per Requirement 1.6
  - Emoji-only message detection and enlargement per Requirement 1.9
  - _Requirements: Requirement 1.2-1.6, 1.9 - rich text messaging features_

- [ ] **4.1.2 Complete file upload UI with v2.0 messaging**
  - Drag-and-drop zones with professional styling per Requirement 1.4
  - "File sharing available in v2.0" messaging per Requirement 1.4
  - Avatar upload UI with text-based initials per Requirement 3.6
  - Maintain all CSS styling and components for future functionality
  - _Requirements: Requirement 1.4 - graceful file degradation, Requirement 3.6 - avatar UI_

### 4.2 ActionCable-Equivalent WebSocket Integration

- [ ] **4.2.1 Connection management with Rails patterns**
  - Auto-reconnect with exponential backoff per Requirement 1.13
  - Connection state synchronization per Requirement 4.11
  - Show connection status to user (Rails ActionCable behavior)
  - Handle WebSocket connection loss within 60 seconds per Requirement 4.13
  - _Requirements: Requirement 1.13 - retry logic, Requirement 4.13 - connection loss detection_

- [ ] **4.2.2 Real-time updates via Turbo Streams**
  - Receive Turbo Streams messages per Requirement 4.1
  - Update presence indicators per Requirement 4.2-4.7
  - Show typing notifications per Requirement 4.6
  - Maintain message order consistency per Requirement 4.11 (Rails-level)
  - _Requirements: Requirement 4.1 - Turbo Streams, Requirement 4.11 - message consistency_

---

## Phase 5: Additional MVP Features (Week 5)

### 5.1 Bot Integration System

- [ ] **5.1.1 Bot authentication and management**
  - User.create_bot! with SecureRandom.alphanumeric(12) token per Requirement 5.1
  - Bot authentication via "id-token" format per Requirement 5.2
  - Accounts::BotsController for administrator management per Requirement 5.10
  - Bot token reset functionality per Requirement 5.8
  - _Requirements: Requirement 5.1-5.2, 5.8, 5.10 - bot creation and management_

- [ ] **5.1.2 Webhook delivery system**
  - Bot::WebhookJob for @mention and DM triggers per Requirement 5.3
  - Webhook payload with user, room, message details per Requirement 5.4
  - 7-second timeout with async delivery per Requirement 5.5
  - Handle webhook responses and create reply messages per Requirement 5.6-5.7
  - _Requirements: Requirement 5.3-5.7 - webhook system_

### 5.2 Search and Additional Features

- [ ] **5.2.1 FTS5 message search**
  - SQLite FTS5 with Porter stemming per Requirement 6.9
  - Sub-millisecond search times with proper indexing
  - Search across message content with room filtering
  - Maintain search index during message operations
  - _Requirements: Requirement 6.9 - FTS5 search performance_

- [ ] **5.2.2 Push notification system**
  - Web Push with VAPID keys for message notifications
  - Service worker integration for PWA functionality
  - @mention notifications per involvement level
  - Push subscription management in database
  - _Requirements: Requirement 8.8 - push notifications_

## What We're NOT Implementing (Rails Doesn't Do It Either)

### ❌ Perfect Message Ordering (Rails Limitation #1)
- **Rails Reality**: Rails uses created_at timestamps, accepts occasional out-of-order
- **Our Approach**: Use database timestamps per Requirement 4.11, accept Rails-level ordering
- **Requirements**: Requirement 4.11 - "maintain message order consistency" = Rails-level

### ❌ Perfect Multi-tab Coordination (Rails Limitation #2)
- **Rails Reality**: Rails doesn't coordinate multiple tabs perfectly
- **Our Approach**: Each tab is independent connection per Requirement 4.10 (Rails behavior)
- **Requirements**: Requirement 4.10 - "across multiple browser tabs" = Rails behavior

### ❌ Guaranteed Message Delivery (Rails Limitation #3)
- **Rails Reality**: ActionCable is best-effort, no delivery guarantees
- **Our Approach**: Best-effort broadcast per Requirement 4.1, log failures
- **Requirements**: Requirement 4.1 - "broadcast within 100ms" = best effort like Rails

### ❌ Perfect Presence Accuracy (Rails Limitation #4)
- **Rails Reality**: Rails presence has delays and edge cases
- **Our Approach**: "Good enough" presence per Requirement 4.2-4.4 with known limitations
- **Requirements**: Requirement 4.4 - "handle browser crashes gracefully" = Rails level

---

## Known Limitations (Rails-Equivalent)

### Connection Management
- Multiple tabs create multiple connections (Rails behavior)
- Connection cleanup may be delayed (Rails limitation)
- Presence may be briefly inaccurate (Rails limitation)

### Message Handling
- Rare duplicate messages possible in edge cases (Rails has this)
- Message ordering by timestamp, not perfect sequence (Rails approach)
- WebSocket failures require client retry (Rails behavior)

### Performance
- Single SQLite database limits concurrent writes (Rails has similar limits)
- Memory usage grows with connections (Rails limitation)
- No horizontal scaling (Rails single-server equivalent)

---

## Success Metrics

### Functional Requirements
- [ ] Users can send/receive messages in real-time
- [ ] Basic presence tracking works "most of the time"
- [ ] Reconnection works after network issues
- [ ] No obvious duplicate messages under normal use
- [ ] Session authentication works reliably

### Performance Requirements (Rails-Equivalent)
- [ ] Handle 50 concurrent users (Rails single-server equivalent)
- [ ] Message delivery within 1 second under normal load
- [ ] Reconnection within 5 seconds
- [ ] Memory usage stays reasonable (< 100MB)

### Reliability Requirements
- [ ] Works reliably for 8-hour sessions
- [ ] Handles network hiccups gracefully
- [ ] Database corruption recovery
- [ ] Graceful degradation under load

---

## Implementation Notes

### Code Organization
- Maximum 500 lines per file
- Rails-style modules: models/, handlers/, services/
- Single responsibility per module
- No circular dependencies

### Error Handling
- Result<T, E> patterns throughout
- User-friendly error messages
- Log technical details, show simple messages to users
- No complex retry logic (Rails keeps it simple)

### Testing Strategy
- Unit tests for core business logic
- Integration tests for API endpoints
- WebSocket connection tests
- Database constraint tests
- No mocking - use real SQLite in tests

### Deployment
- Single binary with embedded React assets
- SQLite database in mounted volume
- Environment variables for configuration
- Simple Docker container (Rails equivalent)

---

## Timeline Summary

**Week 1**: Core infrastructure with 5 critical gap fixes
**Week 2**: Database operations and ActionCable-equivalent WebSocket broadcasting  
**Week 3**: Real-time features with Rails-equivalent quality and limitations
**Week 4**: Complete React UI with graceful v2.0 degradation messaging
**Week 5**: Bot integration, search, and push notifications

**Total**: 5 weeks for fully functional Rails-equivalent chat application with complete UI

## Anti-Coordination Compliance Checklist

### ✅ MANDATORY PATTERNS (Per Requirement 0)
- [ ] Direct SQLite operations with basic connection pooling
- [ ] Basic WebSocket broadcasting (ActionCable-style)
- [ ] Rails-style session management with secure cookies
- [ ] Simple Result<T, E> error handling with user-friendly messages
- [ ] Direct function calls, no async coordination between components
- [ ] Single binary deployment with embedded assets

### 🚫 FORBIDDEN PATTERNS (Will Cause Spec Rejection)
- [ ] ❌ NO coordination layers, coordinators, or event buses
- [ ] ❌ NO distributed transactions, sagas, or event sourcing
- [ ] ❌ NO circuit breakers, retry queues, or complex error recovery
- [ ] ❌ NO cross-tab coordination or global state synchronization
- [ ] ❌ NO microservices, service mesh, or distributed architecture
- [ ] ❌ NO message queues, event streams, or async coordination

### 📏 COMPLEXITY LIMITS (Per Requirement 0.8)
- [ ] Maximum 50 total files in entire codebase
- [ ] No file over 500 lines
- [ ] Maximum 3 async operations per request
- [ ] No more than 2 levels of error handling
- [ ] Single database connection pool

## 5 Critical Gaps Implementation Summary

**Gap #1: client_message_id Deduplication** ✅
- UNIQUE constraint on (client_message_id, room_id) prevents duplicates
- Handle constraint violations by returning existing message
- _Task: 2.1.2 - message CRUD with deduplication_

**Gap #2: WebSocket Reconnection State** ✅
- Track last_seen_message_id per connection
- Send missed messages on reconnect
- _Task: 2.2.3 - reconnection with state sync_

**Gap #3: SQLite Write Serialization** ✅
- Dedicated Writer Task pattern with mpsc channel
- All writes serialized through single task
- _Task: 2.1.1 - dedicated writer pattern_

**Gap #4: Session Token Security** ✅
- SecureRandom equivalent for token generation
- Rails-style httponly SameSite=Lax cookies
- _Task: 1.2.2 - session authentication_

**Gap #5: Basic Presence Tracking** ✅
- Connection counting with 60-second TTL
- Heartbeat refresh every 50 seconds
- _Task: 3.1.1-3.1.2 - presence tracking_

This plan is **realistic and achievable** because it:
1. **Fixes only the 5 critical gaps** that Rails actually solves
2. **Accepts Rails-level limitations** rather than over-engineering
3. **Follows anti-coordination constraints** strictly
4. **Provides complete UI** with graceful v2.0 messaging
5. **References specific requirements** for every task