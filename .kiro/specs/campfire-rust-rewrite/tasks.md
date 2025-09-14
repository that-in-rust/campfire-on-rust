# Realistic Implementation Plan - Campfire MVP 1.0

## Overview: TDD-Driven "Rails-Equivalent Imperfection" Strategy

This implementation plan uses **Test-Driven Development** with **function signatures defined before implementation** to achieve **"works well enough"** rather than **"perfect"** - exactly matching Rails behavior and limitations while ensuring one-shot correctness through type-driven design.

**TDD-First Philosophy**:
- **Type Contracts Before Code**: Define complete function signatures with all error cases first
- **Property-Based Specifications**: Specify behavior through property tests that validate invariants
- **Rails Parity Rule**: If Rails doesn't do it perfectly, we don't need to either - but we specify it completely
- **"Good Enough" Quality**: Match Rails reliability with compile-time guarantees
- **5 Critical Gaps Only**: Fix only gaps that Rails actually solves, using type-driven design
- **Accept Known Limitations**: Document and accept Rails-equivalent limitations with type safety

**TDD Success Criteria**: 
1. **Complete Type Contracts**: Every function signature defined with all error cases before implementation
2. **Property Test Coverage**: All invariants validated with property-based testing
3. **Integration Test Validation**: All service boundaries tested with real dependencies
4. **Rails Behavioral Parity**: Works as well as Rails ActionCable, with similar limitations and edge cases
5. **Compile-Time Safety**: Type system prevents coordination complexity and common bugs

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
- **Our Fix**: Implement Rails-equivalent secure token generation with type safety
- **Requirements**: Requirement 3.4 - rate limit login attempts, secure session management

**Gap #5: Basic Presence Tracking**
- **Rails Reality**: Simple connection counting with heartbeat cleanup
- **Our Fix**: HashMap<UserId, connection_count> with 60-second TTL and RAII cleanup
- **Requirements**: Requirement 4.2 - track connections count with 60-second TTL

## TDD Implementation Methodology

### Phase 1: Type Contract Definition (Before Any Code)
- Define complete function signatures for all services
- Specify all error cases in Result<T, E> types
- Document behavior contracts and side effects
- Create comprehensive type definitions with newtypes

### Phase 2: Property Test Specification
- Write property-based tests for all invariants
- Specify behavior through mathematical properties
- Create test data generators with proptest
- Define integration test contracts

### Phase 3: Type-Guided Implementation
- Implement following type contracts
- Use type system to prevent coordination complexity
- Apply RAII patterns for resource management
- Implement actor patterns for state management

### Phase 4: Comprehensive Validation
- Validate property test compliance
- Run integration tests with real dependencies
- Benchmark critical paths for performance
- Verify Rails behavioral parity
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

## Phase 0: TDD Foundation (Week 0)

**Goal**: Establish complete type contracts and property tests before any implementation

### 0.1 Type Contract Definition

- [ ] **0.1.1 Define complete service trait interfaces**
  - MessageService trait with all methods and error cases
  - RoomService trait with membership management methods
  - AuthService trait with session and user management
  - WebSocketBroadcaster trait with connection management
  - _Requirements: All service boundaries defined before implementation_

- [ ] **0.1.2 Create comprehensive error type hierarchy**
  - MessageError with all validation, database, and authorization cases
  - RoomError with access control and membership errors
  - AuthError with authentication and session errors
  - ConnectionError with WebSocket and presence errors
  - _Requirements: Complete error case enumeration for Result<T, E> types_

- [ ] **0.1.3 Define domain model type contracts**
  - Message<State> with phantom types for Draft/Validated/Persisted states
  - WebSocketConnection<State> with Connected/Authenticated/Subscribed states
  - User, Room, Membership with complete field specifications
  - All newtypes: UserId, RoomId, MessageId, SessionId, ConnectionId
  - _Requirements: Type system prevents invalid state transitions_

### 0.2 Property Test Specification

- [ ] **0.2.1 Message service property tests**
  - Duplicate client_message_id returns same message (Critical Gap #1)
  - Messages since returns chronological order
  - Search respects room access permissions
  - Message creation is atomic with room timestamp update
  - _Requirements: Property tests validate all message invariants_

- [ ] **0.2.2 WebSocket connection property tests**
  - Reconnection delivers all missed messages (Critical Gap #2)
  - Connection state transitions are valid
  - Presence tracking is eventually consistent
  - Broadcast delivery is best-effort (Rails-equivalent)
  - _Requirements: Property tests validate connection behavior_

- [ ] **0.2.3 Room membership property tests**
  - Room creator always has membership (unless deactivated)
  - Direct rooms have exactly 2 members
  - Open rooms auto-grant to all active users
  - Involvement levels control message visibility
  - _Requirements: Property tests validate membership invariants_

### 0.3 Integration Contract Definition

- [ ] **0.3.1 End-to-end message flow contracts**
  - HTTP message creation → Database write → WebSocket broadcast
  - User authentication → Session creation → Cookie setting
  - Room creation → Membership granting → Sidebar updates
  - File upload UI → Graceful degradation messaging
  - _Requirements: Complete integration test specifications_

- [ ] **0.3.2 Service boundary integration tests**
  - MessageService + RoomService + WebSocketBroadcaster integration
  - AuthService + SessionService + CookieService integration
  - Database writer + FTS5 indexer + presence tracker integration
  - All services tested with real SQLite database
  - _Requirements: All service interactions validated_

## Phase 1: Core Infrastructure (Week 1)

**Goal**: Implement type-guided infrastructure following contracts

### 1.1 Project Setup

- [ ] **1.1.1 Implement type-guided project structure**
  - Create Cargo.toml with TDD dependencies: axum, sqlx, tokio, serde, rust-embed, proptest
  - Implement project structure following type contracts from Phase 0
  - Set up basic project structure (≤50 files total per Requirement 0.8)
  - Configure development environment with property test runners
  - _Requirements: Requirement 0.1 - direct function calls, Phase 0 type contracts_

- [ ] **1.1.2 Implement domain models following type contracts**
  - Implement UserId, RoomId, MessageId newtypes with serde derives
  - Implement User, Room, Message structs matching Phase 0 specifications
  - Implement UserRole enum with database mapping
  - Implement comprehensive error types from Phase 0.1.2
  - _Requirements: Phase 0 type contracts, Requirement 3.7 - role enum_

- [ ] **1.1.3 Implement database layer following contracts**
  - Implement Database struct with read pool and dedicated writer
  - Create schema with UNIQUE constraint on (client_message_id, room_id) - **Critical Gap Fix #1**
  - Implement WAL mode and connection pooling with error handling
  - Create FTS5 search index with compile-time query validation
  - Run property tests to validate database behavior contracts
  - _Requirements: Phase 0 database contracts, Critical Gap #1, Requirement 7.5_

### 1.2 Basic HTTP Server

- [ ] **1.2.1 Implement HTTP server following contracts**
  - Implement Axum server with type-safe extractors and error handling
  - Implement embedded React SPA serving using rust-embed
  - Implement CORS and middleware with comprehensive error handling
  - Implement static asset serving with proper caching headers
  - Run integration tests to validate HTTP behavior contracts
  - _Requirements: Phase 0 HTTP contracts, Requirement 8.7 - embedded assets_

- [ ] **1.2.2 Implement authentication service - Critical Gap Fix #4**
  - Implement AuthService trait following Phase 0 contracts
  - Implement secure token generation with cryptographic randomness
  - Implement cookie-based sessions with security headers
  - Implement login/logout endpoints with comprehensive error handling
  - Run property tests to validate authentication behavior
  - _Requirements: Phase 0 auth contracts, Critical Gap #4, Requirement 3.3_

---

## Phase 2: Core Chat Functionality (Week 2)

**Goal**: Basic message sending/receiving that works "well enough"

### 2.1 Database Operations with Write Serialization

- [ ] **2.1.1 Implement DatabaseWriter actor - Critical Gap Fix #3**
  - Implement DatabaseWriter following Phase 0 contracts
  - Implement single writer task with mpsc channel for write serialization
  - Implement concurrent read operations with proper error handling
  - Handle SQLite errors gracefully with user-friendly messages
  - Run property tests to validate write serialization behavior
  - _Requirements: Phase 0 database contracts, Critical Gap #3, Requirement 4.11_

- [ ] **2.1.2 Implement MessageService following contracts**
  - Implement MessageService trait from Phase 0.1.1
  - Implement create_message_with_deduplication with UNIQUE constraint handling
  - Implement get_messages_since with chronological ordering guarantees
  - Implement search_messages with FTS5 and permission filtering
  - Run property tests from Phase 0.2.1 to validate behavior
  - _Requirements: Phase 0 message contracts, Critical Gap #1, Requirement 1.1-1.7_

- [ ] **2.1.3 Implement RoomService following contracts**
  - Implement RoomService trait from Phase 0.1.1
  - Implement room creation with automatic membership granting
  - Implement membership management with involvement level controls
  - Implement permission checks with type-safe authorization
  - Run property tests from Phase 0.2.3 to validate membership invariants
  - _Requirements: Phase 0 room contracts, Requirement 2.1-2.12, Requirement 3.12_

### 2.2 WebSocket Broadcasting (Rails ActionCable Equivalent)

- [ ] **2.2.1 Implement WebSocketBroadcaster following contracts**
  - Implement WebSocketBroadcaster trait from Phase 0.1.1
  - Implement connection management with type-safe state transitions
  - Implement authentication via session cookies with error handling
  - Implement connection cleanup with RAII patterns
  - Run property tests from Phase 0.2.2 to validate connection behavior
  - _Requirements: Phase 0 WebSocket contracts, Critical Gap #2, Requirement 4.8-4.9_
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