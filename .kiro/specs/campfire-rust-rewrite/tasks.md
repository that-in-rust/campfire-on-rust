# Implementation Plan - Campfire Rust Rewrite MVP 1.0

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

**Gap #1: client_message_id Deduplication** - UNIQUE constraint on (client_message_id, room_id) ✅ IMPLEMENTED
**Gap #2: WebSocket Reconnection State** - Track last_seen_message_id per connection ✅ IMPLEMENTED  
**Gap #3: SQLite Write Serialization** - Dedicated Writer Task pattern with mpsc channel ❌ PENDING
**Gap #4: Session Token Security** - Rails-equivalent secure token generation ✅ IMPLEMENTED
**Gap #5: Basic Presence Tracking** - HashMap<UserId, connection_count> with 60-second TTL ✅ IMPLEMENTED

## Current Implementation Status

### ✅ COMPLETED TASKS

- [x] **1.1 Project Structure and Domain Models**
  - ✅ Created src/main.rs with basic Axum server setup
  - ✅ Implemented core domain types: UserId, RoomId, MessageId with newtype pattern
  - ✅ Created User, Room, Message structs with proper serialization
  - ✅ Set up comprehensive error types: MessageError, RoomError, AuthError, ConnectionError
  - ✅ Added From/Into implementations and Display traits for ergonomic conversions
  - _Requirements: Requirement 0.1-0.12 (Anti-coordination architecture)_

- [x] **1.2 Database Schema and Migrations**
  - ✅ Created SQLite database schema with all tables from design.md
  - ✅ Implemented UNIQUE constraint on (client_message_id, room_id) for Critical Gap #1
  - ✅ Set up FTS5 virtual table for message search with triggers
  - ✅ Created database migration system with direct SQLite operations
  - ✅ Implemented complete database operations for users, sessions, and messages
  - _Requirements: Critical Gap #1, Requirement 7.5 (FTS5 search)_

- [x] **1.3 Authentication Service (Critical Gap #4)**
  - ✅ Implemented secure session token generation using cryptographically secure random
  - ✅ Created AuthService with complete login/logout/create_user functionality
  - ✅ Set up session storage in SQLite with proper expiration
  - ✅ Implemented password hashing with bcrypt and validation
  - ✅ Added comprehensive tests for token security and authentication flows
  - _Requirements: Critical Gap #4, Requirement 3.1-3.4 (authentication)_

- [x] **2.1 Message Service with Deduplication (Critical Gap #1)**
  - ✅ Implemented MessageService trait from design.md with complete contract
  - ✅ Created message creation with client_message_id deduplication
  - ✅ Handle UNIQUE constraint violations gracefully with existing message return
  - ✅ Added message validation (1-10000 chars, HTML sanitization with ammonia)
  - ✅ Added comprehensive tests for deduplication and validation
  - ✅ Integrated with ConnectionManager for WebSocket broadcasting
  - _Requirements: Critical Gap #1, Requirement 1.1-1.4 (message creation)_

- [x] **3.1 WebSocket Connection Manager (Critical Gap #2 & #5)**
  - ✅ Implemented ConnectionManager trait and complete implementation
  - ✅ Created connection tracking with HashMap<ConnectionId, ConnectionInfo>
  - ✅ Implemented presence tracking with HashMap<UserId, connection_count>
  - ✅ Added 60-second TTL cleanup for disconnected users with background task
  - ✅ Implemented last_seen_message_id tracking for reconnection state
  - ✅ Added comprehensive tests for presence, reconnection, and broadcasting
  - _Requirements: Critical Gap #2, #5, Requirement 4.1-4.11 (real-time features)_

- [x] **3.2 WebSocket Message Broadcasting**
  - ✅ Implemented room-based message broadcasting with serialization
  - ✅ Created WebSocket message types for different events (NewMessage, Presence, etc.)
  - ✅ Added message broadcasting integration in MessageService
  - ✅ Implemented partial failure handling and error reporting
  - _Requirements: Requirement 4.1-4.3 (real-time messaging)_

## 🚧 REMAINING TASKS

### Phase 1: Complete HTTP API Integration

- [x] **1.4 Session Extraction Middleware**
  - Create session extraction middleware for Axum
  - Implement session token parsing from Authorization header or cookies
  - Add user authentication state to request context
  - Handle session validation errors with proper HTTP status codes
  - _Requirements: Requirement 3.1-3.4 (authentication endpoints)_

- [ ] **1.5 Complete Authentication HTTP Handlers**
  - Integrate AuthService into AppState
  - Complete POST /api/auth/login handler with proper error handling
  - Complete POST /api/auth/logout handler with session revocation
  - Complete GET /api/users/me handler with session validation
  - Add proper JSON error responses and HTTP status codes
  - _Requirements: Requirement 3.1-3.4 (authentication endpoints)_

### Phase 2: Room Management System

- [ ] **2.2 Complete Room Service Implementation**
  - Implement create_room with database operations and validation
  - Add room membership system with involvement levels (Member/Admin)
  - Implement add_member with proper authorization checks
  - Complete check_room_access with database queries
  - Implement get_user_rooms with membership filtering
  - Add database operations for rooms and room_memberships tables
  - _Requirements: Requirement 2.1-2.7 (room management)_

- [ ] **2.3 Room API Endpoints**
  - Complete GET /api/rooms handler with user room filtering
  - Complete POST /api/rooms handler with room creation
  - Add proper UUID parsing and validation for room IDs
  - Integrate RoomService into AppState and handlers
  - Add room access validation for all room operations
  - _Requirements: Requirement 2.1-2.7 (room management APIs)_

### Phase 3: Message API Integration

- [ ] **3.3 Complete Message API Endpoints**
  - Complete POST /api/rooms/:id/messages handler with MessageService
  - Complete GET /api/rooms/:id/messages handler with pagination
  - Add proper UUID parsing for room_id and message_id parameters
  - Integrate MessageService into AppState
  - Add room access validation for message operations
  - _Requirements: Requirement 1.1-1.4 (message APIs)_

- [ ] **3.4 Database Writer Pattern (Critical Gap #3)**
  - Implement single writer task with mpsc channel for write serialization
  - Create DatabaseWriter trait for all write operations
  - Ensure all writes go through the single writer to prevent SQLite conflicts
  - Add proper error handling and retry logic for write operations
  - Refactor existing database operations to use the writer pattern
  - _Requirements: Critical Gap #3, Requirement 0.1 (direct function calls)_

### Phase 4: WebSocket Real-Time Features

- [ ] **4.1 WebSocket Handler Implementation**
  - Implement WebSocket upgrade handler in Axum (/ws endpoint)
  - Add WebSocket message parsing and routing
  - Integrate ConnectionManager with actual WebSocket connections
  - Handle connection lifecycle (connect, disconnect, error) properly
  - Add authentication for WebSocket connections
  - _Requirements: Requirement 4.1-4.11 (real-time features)_

- [ ] **4.2 Complete Missed Messages Implementation (Critical Gap #2)**
  - Complete missed message delivery on reconnection in ConnectionManager
  - Implement database queries for messages since last_seen_message_id
  - Add proper error handling for reconnection scenarios
  - Test reconnection flow with message history delivery
  - _Requirements: Critical Gap #2, Requirement 4.8-4.11 (connection management)_

- [ ] **4.3 Typing Indicators and Enhanced Presence**
  - Implement typing notification system with WebSocket messages
  - Add presence change broadcasting to room members
  - Create typing start/stop WebSocket message handlers
  - Enhance presence tracking with room-specific presence
  - _Requirements: Requirement 4.4-4.7 (presence tracking)_

### Phase 5: Advanced Features

- [ ] **5.1 Full-Text Search Implementation**
  - Create SearchService using existing SQLite FTS5 setup
  - Implement search API endpoint: GET /api/search?q=query
  - Add search result ranking and pagination
  - Test FTS5 integration with message indexing and triggers
  - Add search result authorization (only show accessible messages)
  - _Requirements: Requirement 7.1-7.5 (search functionality)_

- [ ] **5.2 Rich Text and Sound System**
  - Enhance HTML sanitization for rich text features (bold, italic, links)
  - Add support for @mentions with user linking and notifications
  - Embed MP3 sound files using rust-embed crate
  - Create /play command processing for sounds in message content
  - Add sound playback WebSocket messages
  - _Requirements: Requirement 5.1-5.4 (rich text), 6.1-6.3 (sound system)_

- [ ] **5.3 Push Notifications**
  - Add web-push crate dependency and implement Web Push with VAPID keys
  - Create push notification service with subscription management
  - Add notification preferences per user in database
  - Implement notification triggers for mentions and DMs
  - Add push notification endpoints for subscription management
  - _Requirements: Requirement 8.1-8.6 (push notifications)_

- [ ] **5.4 Bot API Integration**
  - Implement bot authentication with API tokens (using existing bot_token field)
  - Create bot-specific endpoints for message posting
  - Add bot user type validation and permissions
  - Implement webhook delivery for bot integrations
  - Add bot management endpoints
  - _Requirements: Requirement 9.1-9.4 (bot integration)_

### Phase 6: Frontend and Production

- [ ] **6.1 Static Asset Serving**
  - Add rust-embed crate dependency for asset embedding
  - Embed frontend assets (HTML, CSS, JS) at compile time
  - Serve static files from embedded assets with proper MIME types
  - Add asset caching headers and compression
  - Create basic HTML templates for the chat interface
  - _Requirements: Requirement 8.7 (embedded assets)_

- [ ] **6.2 Rate Limiting and Security**
  - Add governor crate dependency and implement rate limiting middleware
  - Enhance CORS middleware configuration for production
  - Implement request size limits and timeout handling
  - Add security headers (CSP, HSTS, X-Frame-Options, etc.)
  - Add input validation and sanitization for all endpoints
  - _Requirements: Requirement 3.4 (rate limiting), 0.1-0.12 (security)_

- [ ] **6.3 Graceful Shutdown and Health Checks**
  - Implement graceful shutdown handling with signal handling
  - Enhance health check endpoint with database connectivity checks
  - Create proper resource cleanup on shutdown (connections, tasks)
  - Add startup validation and readiness checks
  - Add metrics and monitoring endpoints
  - _Requirements: Requirement 0.1-0.12 (operational requirements)_

- [ ] **6.4 Production Deployment Preparation**
  - Create single binary build configuration with embedded assets
  - Add configuration management with environment variables
  - Enhance structured logging with tracing and log levels
  - Create Docker container for deployment
  - Add database backup and migration scripts
  - _Requirements: Requirement 0.1-0.12 (single binary deployment)_

## Testing Strategy

Each remaining task should include:
- **Unit Tests**: Test individual functions and components
- **Integration Tests**: Test service boundaries and database operations
- **Property Tests**: Test invariants with proptest for critical gaps
- **End-to-End Tests**: Test complete user journeys

## Critical Gap Status

- **Gap #1**: ✅ COMPLETE - Message deduplication implemented and tested with UNIQUE constraint
- **Gap #2**: ✅ COMPLETE - WebSocket reconnection state tracking implemented with last_seen_message_id
- **Gap #3**: ❌ PENDING - SQLite write serialization needs implementation (Task 3.4)
- **Gap #4**: ✅ COMPLETE - Secure session token generation implemented and tested with crypto-secure random
- **Gap #5**: ✅ COMPLETE - Presence tracking with TTL cleanup implemented and tested with background task

## Success Criteria

- All tests pass (unit, integration, property, e2e)
- Single binary deployment works with embedded assets
- WebSocket real-time messaging functions correctly
- All 5 critical gaps are solved and tested
- Rails behavioral parity achieved for core features
- HTTP API endpoints work with proper authentication
- Room management system functions correctly

## Next Priority Tasks

1. **Session Extraction Middleware** (Task 1.4) - Essential for API authentication
2. **Complete Authentication Handlers** (Task 1.5) - Required for user login/logout
3. **Complete Room Service** (Task 2.2) - Required for room management
4. **Room API Endpoints** (Task 2.3) - Essential for room operations
5. **Message API Endpoints** (Task 3.3) - Critical for message functionality
6. **Database Writer Pattern** (Task 3.4) - Critical Gap #3 implementation

## Implementation Notes

**Current State**: The core services (AuthService, MessageService, ConnectionManager) are fully implemented with comprehensive tests. The database schema is complete with all necessary tables and constraints. The main remaining work is:

1. **HTTP API Integration**: Connect the implemented services to HTTP handlers
2. **Room Management**: Complete the RoomService implementation 
3. **WebSocket Integration**: Connect the ConnectionManager to actual WebSocket handlers
4. **Write Serialization**: Implement Critical Gap #3 for SQLite write safety

**Architecture Status**: The foundation is solid with proper error handling, type safety, and Rails-equivalent patterns. The remaining tasks focus on integration and completing the user-facing API.