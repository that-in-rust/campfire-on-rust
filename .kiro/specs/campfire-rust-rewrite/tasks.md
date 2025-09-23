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

- [x] **1.5 Complete Authentication HTTP Handlers**
  - Integrate AuthService into AppState
  - Complete POST /api/auth/login handler with proper error handling
  - Complete POST /api/auth/logout handler with session revocation
  - Complete GET /api/users/me handler with session validation
  - Add proper JSON error responses and HTTP status codes
  - _Requirements: Requirement 3.1-3.4 (authentication endpoints)_

### Phase 2: Room Management System

- [x] **2.2 Complete Room Service Implementation**
  - Implement create_room with database operations and validation
  - Add room membership system with involvement levels (Member/Admin)
  - Implement add_member with proper authorization checks
  - Complete check_room_access with database queries
  - Implement get_user_rooms with membership filtering
  - Add database operations for rooms and room_memberships tables
  - _Requirements: Requirement 2.1-2.7 (room management)_

- [x] **2.3 Room API Endpoints**
  - Complete GET /api/rooms handler with user room filtering
  - Complete POST /api/rooms handler with room creation
  - Add proper UUID parsing and validation for room IDs
  - Integrate RoomService into AppState and handlers
  - Add room access validation for all room operations
  - _Requirements: Requirement 2.1-2.7 (room management APIs)_

### Phase 3: Message API Integration

- [x] **3.3 Complete Message API Endpoints**
  - Complete POST /api/rooms/:id/messages handler with MessageService
  - Complete GET /api/rooms/:id/messages handler with pagination
  - Add proper UUID parsing for room_id and message_id parameters
  - Integrate MessageService into AppState
  - Add room access validation for message operations
  - _Requirements: Requirement 1.1-1.4 (message APIs)_

- [x] **3.4 Database Writer Pattern (Critical Gap #3)**
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

- [x] **4.2 Complete Missed Messages Implementation (Critical Gap #2)**
  - ✅ Complete missed message delivery on reconnection in ConnectionManager
  - ✅ Implement database queries for messages since last_seen_message_id
  - ✅ Add proper error handling for reconnection scenarios
  - ✅ Test reconnection flow with message history delivery
  - _Requirements: Critical Gap #2, Requirement 4.8-4.11 (connection management)_

- [x] **4.3 Typing Indicators and Enhanced Presence**
  - Implement typing notification system with WebSocket messages
  - Add presence change broadcasting to room members
  - Create typing start/stop WebSocket message handlers
  - Enhance presence tracking with room-specific presence
  - _Requirements: Requirement 4.4-4.7 (presence tracking)_

### Phase 5: Advanced Features

- [x] **5.1 Full-Text Search Implementation**
  - Create SearchService using existing SQLite FTS5 setup
  - Implement search API endpoint: GET /api/search?q=query
  - Add search result ranking and pagination
  - Test FTS5 integration with message indexing and triggers
  - Add search result authorization (only show accessible messages)
  - _Requirements: Requirement 7.1-7.5 (search functionality)_

- [x] **5.2 Rich Text and Sound System**
  - Enhance HTML sanitization for rich text features (bold, italic, links)
  - Add support for @mentions with user linking and notifications
  - Embed MP3 sound files using rust-embed crate
  - Create /play command processing for sounds in message content
  - Add sound playback WebSocket messages
  - _Requirements: Requirement 5.1-5.4 (rich text), 6.1-6.3 (sound system)_

- [x] **5.3 Push Notifications**
  - Add web-push crate dependency and implement Web Push with VAPID keys
  - Create push notification service with subscription management
  - Add notification preferences per user in database
  - Implement notification triggers for mentions and DMs
  - Add push notification endpoints for subscription management
  - _Requirements: Requirement 8.1-8.6 (push notifications)_

- [x] **5.4 Bot API Integration**
  - Implement bot authentication with API tokens (using existing bot_token field)
  - Create bot-specific endpoints for message posting
  - Add bot user type validation and permissions
  - Implement webhook delivery for bot integrations
  - Add bot management endpoints
  - _Requirements: Requirement 9.1-9.4 (bot integration)_

### Phase 6: Frontend and Production

- [x] **6.1 Static Asset Serving**
  - Add rust-embed crate dependency for asset embedding
  - Embed frontend assets (HTML, CSS, JS) at compile time
  - Serve static files from embedded assets with proper MIME types
  - Add asset caching headers and compression
  - Create basic HTML templates for the chat interface
  - _Requirements: Requirement 8.7 (embedded assets)_

- [x] **6.2 Rate Limiting and Security**
  - Add governor crate dependency and implement rate limiting middleware
  - Enhance CORS middleware configuration for production
  - Implement request size limits and timeout handling
  - Add security headers (CSP, HSTS, X-Frame-Options, etc.)
  - Add input validation and sanitization for all endpoints
  - _Requirements: Requirement 3.4 (rate limiting), 0.1-0.12 (security)_

- [x] **6.3 Graceful Shutdown and Health Checks**
  - Implement graceful shutdown handling with signal handling
  - Enhance health check endpoint with database connectivity checks
  - Create proper resource cleanup on shutdown (connections, tasks)
  - Add startup validation and readiness checks
  - Add metrics and monitoring endpoints
  - _Requirements: Requirement 0.1-0.12 (operational requirements)_

- [x] **6.4 Production Deployment Preparation**
  - Create single binary build configuration with embedded assets
  - Add configuration management with environment variables
  - Enhance structured logging with tracing and log levels
  - Create Docker container for deployment
  - Add database backup and migration scripts
  - _Requirements: Requirement 0.1-0.12 (single binary deployment)_



# README UPDATE

- [x] Update All README and docs with Mermaid diagrams referencing the steering docs - Create README and documentation - take help of (.kiro/steering/mermaid-troubleshooting.md;.kiro/steering/mermaid-syntax-guide.md;kiro/steering/mermaid-status-report.md;.kiro/steering/mermaid-design-patterns.md) x Can we make the README minimalistic x Minto Pyramid Principle - starting from essence at the top and then adding details and lower layers x also all the mermaid diagrams should follow guidance of steering docs x .kiro/steering/mermaid-design-patterns.md x .kiro/steering/mermaid-status-report.md x .kiro/steering/mermaid-syntax-guide.md x .kiro/steering/mermaid-troubleshooting.md

- [x] Run .kiro/tree-with-wc.sh to analyze repository structure + Clean up any unnecessary files or directories - instead of deleting place them in zzzzArchive folder

- [x] In README also add hyperlinks to offline HTML versions of the interface so that users can get an idea of what the interface looks like - at least 10 such HTMLs and if possible screenshots of those to make them feel it

- [x] Cany you study basecamp-once-campfire-8a5edab282632443.txt 1000 lines at a time (will take you 21 shots), to see how you can be inspired to improve your documentation, and also create a doc with differences of current version with the original campfire shared by dhh + jason fried + basecamp team; also add a thank you basecamp md file conveying gratitude for sharing this brilliant piece of code; use mermaid diagrams wherever you can

## Phase 7: Documentation Enhancement (Inspired by Basecamp Campfire)

### 7.1 Core Documentation Improvements (Week 1)

- [x] **7.1.1 Enhanced README with Quick Start Guide**
  - Rewrite README.md with Basecamp-inspired structure and clarity
  - Add quick start section with single-command setup
  - Include feature overview with screenshots/demos
  - Add troubleshooting section for common setup issues
  - Create getting started tutorial with step-by-step instructions
  - _Requirements: Based on Basecamp's excellent README structure_

- [x] **7.1.2 Complete API Documentation with Examples**
  - Create comprehensive API reference documentation in docs/api-reference.md
  - Add request/response examples for every endpoint
  - Include authentication requirements and error responses
  - Add cURL examples and code snippets for common operations
  - Document rate limiting and usage guidelines
  - _Requirements: Inspired by Rails routing and controller documentation patterns_

- [x] **7.1.3 Production Deployment Guide**
  - Create detailed deployment guide in docs/deployment-guide.md
  - Add Docker deployment instructions with environment variables
  - Include reverse proxy setup (nginx/Caddy) with SSL configuration
  - Add monitoring and health check setup instructions
  - Create backup and restore procedures documentation
  - _Requirements: Based on Basecamp's production deployment patterns_

- [x] **7.1.4 Basic Troubleshooting Documentation**
  - Create docs/troubleshooting.md with common issues and solutions
  - Add WebSocket connection troubleshooting section
  - Include database issues and resolution steps
  - Add performance troubleshooting guide
  - Create FAQ section with answers to common questions
  - _Requirements: Inspired by Basecamp's error handling patterns_

### 7.2 Feature-Specific Documentation (Week 2)

- [x] **7.2.1 Real-time Features Documentation**
  - Create docs/websocket-guide.md explaining WebSocket implementation
  - Document connection lifecycle and authentication flow
  - Add examples of WebSocket message types and handling
  - Include reconnection logic and missed message delivery
  - Add presence tracking and typing indicators documentation
  - _Requirements: Based on ActionCable patterns from original Campfire_

- [x] **7.2.2 Authentication System Guide**
  - Create docs/authentication-guide.md with complete auth documentation
  - Document session management and token security
  - Add password requirements and security best practices
  - Include bot authentication and API key management
  - Add session expiration and cleanup documentation
  - _Requirements: Inspired by Rails authentication concerns_

- [x] **7.2.3 Search System Documentation**
  - Create docs/search-guide.md explaining FTS5 integration
  - Document search query syntax and capabilities
  - Add search result ranking and pagination details
  - Include search performance optimization tips
  - Add examples of complex search queries
  - _Requirements: Based on Basecamp's search implementation patterns_

- [x] **7.2.4 Bot Integration Tutorial**
  - Create docs/bot-integration.md with complete bot setup guide
  - Add webhook configuration and payload documentation
  - Include bot authentication and permissions
  - Add examples of bot implementations in different languages
  - Document bot rate limiting and best practices
  - _Requirements: Inspired by Basecamp's webhook system_

### 7.3 Advanced Topics Documentation (Week 3)

- [x] **7.3.1 Performance Tuning Guide**
  - Create docs/performance-guide.md with optimization strategies
  - Document SQLite WAL mode and connection pooling
  - Add WebSocket connection limits and backpressure handling
  - Include memory usage monitoring and optimization
  - Add database indexing and query optimization tips
  - _Requirements: Based on production patterns from original Campfire_

- [x] **7.3.2 Security Best Practices**
  - Create docs/security-guide.md with comprehensive security documentation
  - Document authentication security and session management
  - Add input validation and sanitization guidelines
  - Include network security and TLS configuration
  - Add security headers and CSRF protection documentation
  - _Requirements: Inspired by Basecamp's security-conscious implementation_

- [x] **7.3.3 Migration Documentation**
  - Create docs/migration-guide.md for Rails to Rust migration
  - Document data export procedures from Rails Campfire
  - Add data transformation scripts and procedures
  - Include data integrity verification steps
  - Add rollback procedures and contingency planning
  - _Requirements: Help teams migrate from original Campfire_

- [x] **7.3.4 Scaling Considerations**
  - Create docs/scaling-guide.md with scaling strategies
  - Document single-instance optimization techniques
  - Add multi-instance deployment considerations
  - Include load balancing and session sharing strategies
  - Add capacity planning and resource monitoring
  - _Requirements: Based on production scaling patterns_

### 7.4 Developer Experience Documentation (Week 4)

- [x] **7.4.1 Code Examples and Patterns**
  - Create docs/code-examples.md with common usage patterns
  - Add message creation and room management examples
  - Include WebSocket connection and broadcasting examples
  - Add authentication and authorization code samples
  - Create testing examples and best practices
  - _Requirements: Inspired by Rails helper modules and concerns_

- [x] **7.4.2 Testing Guide with Examples**
  - Create docs/testing-guide.md with comprehensive testing documentation
  - Add unit testing examples for services and handlers
  - Include integration testing with real dependencies
  - Add WebSocket testing strategies and examples
  - Document property-based testing for invariants
  - _Requirements: Based on Basecamp's comprehensive test suite_

- [x] **7.4.3 Contributing Guidelines**
  - Create CONTRIBUTING.md with development workflow
  - Add code style guidelines and formatting requirements
  - Include pull request process and review guidelines
  - Add issue reporting templates and procedures
  - Document development environment setup
  - _Requirements: Inspired by Basecamp's open source contribution model_

- [x] Last moment check of recompiling, TDD, running and eyeballing

- [x] Last moment user journey tests - UATs that you can simulate - the best you can so you know it actually worst as intended
 
- [ ] If there are in md docs in the documentation or elsewhere, make sure you read them and add at least 1 or more mermaid diagrams based on the steering docs x minimalistic x Minto Pyramid Principle - starting from essence at the top and then adding details and lower layers x also all the mermaid diagrams should follow guidance of steering docs x .kiro/steering/mermaid-design-patterns.md x .kiro/steering/mermaid-status-report.md x .kiro/steering/mermaid-syntax-guide.md x .kiro/steering/mermaid-troubleshooting.md