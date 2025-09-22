# Implementation Plan - Campfire Rust Rewrite MVP 1.0



This implementation plan follows the TDD-First philosophy with Rails-Compatible Simplicity, focusing on building a working chat application using proven patterns adapted to Rust.

## IMPORTANT FOR VISUALS AND DIAGRAMS

ALL DIAGRAMS WILL BE IN MERMAID ONLY TO ENSURE EASE WITH GITHUB - DO NOT SKIP THAT

## Overview

**Strategic Focus**: Build complete text-based Campfire functionality using Rails-equivalent patterns in idiomatic Rust, prioritizing working software over architectural innovation.

**Core Philosophy**: 
- **Rails Parity Rule**: If Rails doesn't do it, we don't do it - replicate Rails patterns exactly using idiomatic Rust
- **Anti-Coordination Mandates**: No coordination layers, event buses, or distributed complexity
- **Single Binary Deployment**: Embedded assets using Rust's compile-time inclusion
- **Type Safety First**: Leverage Rust's type system to prevent bugs at compile-time

## 5 Critical Gaps That Must Be Solved

**Gap #1: client_message_id Deduplication** - UNIQUE constraint on (client_message_id, room_id)
**Gap #2: WebSocket Reconnection State** - Track last_seen_message_id per connection
**Gap #3: SQLite Write Serialization** - Dedicated Writer Task pattern with mpsc channel
**Gap #4: Session Token Security** - Rails-equivalent secure token generation
**Gap #5: Basic Presence Tracking** - HashMap<UserId, connection_count> with 60-second TTL

## Implementation Tasks

### Phase 1: Core Foundation (Week 1)

- [ ] **1.1 Project Structure and Domain Models**
  - Create src/main.rs with basic Axum server setup
  - Implement core domain types: UserId, RoomId, MessageId with newtype pattern
  - Create User, Room, Message structs with proper serialization
  - Set up error types: MessageError, RoomError, AuthError, ConnectionError
  - _Requirements: Requirement 0.1-0.12 (Anti-coordination architecture)_

- [ ] **1.2 Database Schema and Migrations**
  - Create SQLite database schema with all tables from design.md
  - Implement UNIQUE constraint on (client_message_id, room_id) for Critical Gap #1
  - Set up FTS5 virtual table for message search
  - Create database migration system with sqlx
  - _Requirements: Critical Gap #1, Requirement 7.5 (FTS5 search)_

- [ ] **1.3 Authentication Service (Critical Gap #4)**
  - Implement secure session token generation using cryptographically secure random
  - Create AuthService with login/logout functionality
  - Set up session storage in SQLite with proper expiration
  - Implement password hashing with bcrypt
  - _Requirements: Critical Gap #4, Requirement 3.1-3.4 (authentication)_

- [ ] **1.4 Basic HTTP API Endpoints**
  - POST /api/auth/login - user authentication
  - POST /api/auth/logout - session termination
  - GET /api/users/me - current user info
  - Create middleware for session validation
  - _Requirements: Requirement 3.1-3.4 (authentication endpoints)_

### Phase 2: Core Chat Functionality (Week 2)

- [ ] **2.1 Message Service with Deduplication (Critical Gap #1)**
  - Implement MessageService trait from design.md
  - Create message creation with client_message_id deduplication
  - Handle UNIQUE constraint violations gracefully
  - Add message validation (1-10000 chars, HTML sanitization)
  - _Requirements: Critical Gap #1, Requirement 1.1-1.4 (message creation)_

- [ ] **2.2 Room Service and Membership**
  - Implement RoomService trait with room creation/management
  - Create room membership system with involvement levels
  - Add room access control checks
  - Implement Open/Closed/Direct room types
  - _Requirements: Requirement 2.1-2.7 (room management)_

- [ ] **2.3 Database Writer Pattern (Critical Gap #3)**
  - Implement single writer task with mpsc channel for write serialization
  - Create DatabaseWriter trait for all write operations
  - Ensure all writes go through the single writer to prevent conflicts
  - Add proper error handling and retry logic
  - _Requirements: Critical Gap #3, Requirement 0.1 (direct function calls)_

- [ ] **2.4 Message API Endpoints**
  - POST /api/rooms/:id/messages - create message with deduplication
  - GET /api/rooms/:id/messages - retrieve message history
  - GET /api/rooms - list user's rooms
  - POST /api/rooms - create new room
  - _Requirements: Requirement 1.1-1.4, 2.1-2.7 (message and room APIs)_

### Phase 3: Real-Time Features (Week 3)

- [ ] **3.1 WebSocket Connection Manager (Critical Gap #2 & #5)**
  - Implement WebSocket handler with tokio-tungstenite
  - Create ConnectionManager for tracking active connections
  - Implement presence tracking with HashMap<UserId, connection_count>
  - Add 60-second TTL cleanup for disconnected users
  - _Requirements: Critical Gap #2, #5, Requirement 4.1-4.11 (real-time features)_

- [ ] **3.2 WebSocket Message Broadcasting**
  - Implement room-based message broadcasting
  - Create WebSocket message types for different events
  - Add message broadcasting when new messages are created
  - Implement typing notifications
  - _Requirements: Requirement 4.1-4.3 (real-time messaging)_

- [ ] **3.3 Reconnection and Missed Messages (Critical Gap #2)**
  - Track last_seen_message_id per WebSocket connection
  - Implement missed message delivery on reconnection
  - Handle connection drops gracefully
  - Add reconnection state management
  - _Requirements: Critical Gap #2, Requirement 4.8-4.11 (connection management)_

- [ ] **3.4 Presence and Typing Indicators**
  - Implement user presence tracking in rooms
  - Add typing notification system
  - Create presence cleanup on connection close
  - Broadcast presence changes to room members
  - _Requirements: Critical Gap #5, Requirement 4.4-4.7 (presence tracking)_

### Phase 4: Advanced Features (Week 4)

- [ ] **4.1 Full-Text Search (FTS5)**
  - Implement search service using SQLite FTS5
  - Create search API endpoint: GET /api/search?q=query
  - Add search indexing for new messages
  - Implement search result ranking and pagination
  - _Requirements: Requirement 7.1-7.5 (search functionality)_

- [ ] **4.2 Rich Text and Sound System**
  - Implement HTML content sanitization with ammonia
  - Add support for @mentions with user linking
  - Embed MP3 sound files using rust-embed
  - Create /play command processing for sounds
  - _Requirements: Requirement 5.1-5.4 (rich text), 6.1-6.3 (sound system)_

- [ ] **4.3 Push Notifications**
  - Implement Web Push with VAPID keys using web-push crate
  - Create push notification service
  - Add notification preferences per user
  - Implement notification triggers for mentions and DMs
  - _Requirements: Requirement 8.1-8.6 (push notifications)_

- [ ] **4.4 Bot API Integration**
  - Implement bot authentication with API tokens
  - Create bot-specific endpoints for message posting
  - Add bot user type and permissions
  - Implement webhook delivery for bot integrations
  - _Requirements: Requirement 9.1-9.4 (bot integration)_

### Phase 5: Frontend and Polish (Week 5)

- [ ] **5.1 Static Asset Serving**
  - Embed React frontend assets using rust-embed
  - Serve static files from embedded assets
  - Implement proper MIME type handling
  - Add asset caching headers
  - _Requirements: Requirement 8.7 (embedded assets)_

- [ ] **5.2 Rate Limiting and Security**
  - Implement rate limiting using governor crate
  - Add CORS middleware with proper configuration
  - Implement request size limits
  - Add security headers (CSP, HSTS, etc.)
  - _Requirements: Requirement 3.4 (rate limiting), 0.1-0.12 (security)_

- [ ] **5.3 Graceful Shutdown and Health Checks**
  - Implement graceful shutdown handling
  - Add health check endpoint: GET /health
  - Create proper resource cleanup on shutdown
  - Add startup validation and readiness checks
  - _Requirements: Requirement 0.1-0.12 (operational requirements)_

- [ ] **5.4 Production Deployment Preparation**
  - Create single binary build with embedded assets
  - Add configuration management with environment variables
  - Implement structured logging with tracing
  - Create Docker container for deployment
  - _Requirements: Requirement 0.1-0.12 (single binary deployment)_

## Testing Strategy

Each task should include:
- **Unit Tests**: Test individual functions and components
- **Integration Tests**: Test service boundaries and database operations
- **Property Tests**: Test invariants with proptest for critical gaps
- **End-to-End Tests**: Test complete user journeys

## Critical Gap Validation

Each critical gap must have specific tests:
- **Gap #1**: Test message deduplication with concurrent requests
- **Gap #2**: Test WebSocket reconnection with missed message delivery
- **Gap #3**: Test concurrent write operations are properly serialized
- **Gap #4**: Test session token security and validation
- **Gap #5**: Test presence tracking with connection cleanup

## Success Criteria

- All tests pass (unit, integration, property, e2e)
- Single binary deployment works
- WebSocket real-time messaging functions correctly
- All 5 critical gaps are solved and tested
- Rails behavioral parity achieved for core features