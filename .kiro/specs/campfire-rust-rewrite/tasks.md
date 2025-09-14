# Implementation Plan - Campfire Rust Rewrite MVP

## Overview

This implementation plan follows a **TDD-driven, 3-checkpoint approach** to build the Campfire MVP with maximum correctness and minimal debugging. Each checkpoint builds incrementally, with comprehensive testing ensuring "one-shot correct" implementation.

**Core Principles**:
- **Compile-first correctness**: Strong types prevent bugs at compile time
- **Test-driven development**: Tests as executable specifications
- **Anti-coordination compliance**: Simple, Rails-equivalent patterns only
- **Incremental implementation**: One feature at a time, freeze and move on

---

## Checkpoint 1: Scaffolding & Compile-First Correctness

**Goal**: Create a compiling "walking skeleton" with all module interfaces defined but no logic implemented.

### 1.1 Project Structure Setup

- [ ] **1.1.1 Initialize Rust project structure**
  - Create Cargo.toml with required dependencies (axum, sqlx, tokio, serde)
  - Set up workspace structure following 50-file limit
  - Configure development dependencies (mockall, proptest, criterion)
  - _Requirements: Anti-coordination file limits, tech stack constraints_

- [ ] **1.1.2 Create core module structure**
  - Create models/ directory with mod.rs exports
  - Create handlers/ directory for HTTP endpoints
  - Create services/ directory for business logic
  - Create database/ directory for SQLite operations
  - _Requirements: Rails-style organization, clear module boundaries_

- [ ] **1.1.3 Set up frontend structure**
  - Initialize React project with Vite
  - Configure TypeScript and testing setup
  - Create component directory structure
  - Set up build integration with Rust binary
  - _Requirements: Complete UI with graceful degradation_

### 1.2 Domain Types & Interfaces

- [ ] **1.2.1 Define core domain types**
  - Implement UserId, RoomId, MessageId newtypes for type safety
  - Create User, Room, Message structs with all fields
  - Define RoomType enum (Open, Closed, Direct) with permissions
  - Create error types for each domain (UserError, RoomError, MessageError)
  - _Requirements: Type safety, prevent ID mix-ups at compile time_

- [ ] **1.2.2 Define service interfaces**
  - Create MessageService trait with create/update/delete signatures
  - Create RoomService trait with room management operations
  - Create AuthService trait with login/logout/session operations
  - Create WebSocketBroadcaster trait for real-time messaging
  - _Requirements: Clear interfaces, testable design_

- [ ] **1.2.3 Define database interfaces**
  - Create Database trait with all required operations
  - Define repository traits for each domain entity
  - Create migration types and schema definitions
  - Set up connection pool configuration types
  - _Requirements: Direct SQLite operations, no ORM complexity_

### 1.3 HTTP Handler Signatures

- [ ] **1.3.1 Create message handlers**
  - POST /api/rooms/:id/messages (create message)
  - GET /api/rooms/:id/messages (list messages with pagination)
  - PUT /api/messages/:id (update message)
  - DELETE /api/messages/:id (delete message)
  - _Requirements: RESTful API, Rails-style routing_

- [ ] **1.3.2 Create room handlers**
  - GET /api/rooms (list user's rooms)
  - POST /api/rooms (create room)
  - GET /api/rooms/:id (get room details)
  - PUT /api/rooms/:id (update room)
  - DELETE /api/rooms/:id (delete room)
  - _Requirements: Room management, membership handling_

- [ ] **1.3.3 Create auth handlers**
  - POST /api/auth/login (authenticate user)
  - POST /api/auth/logout (end session)
  - GET /api/auth/me (current user info)
  - POST /api/auth/register (create account - first run only)
  - _Requirements: Session-based auth, Rails-style security_

- [ ] **1.3.4 Create WebSocket handlers**
  - WebSocket upgrade endpoint at /ws
  - Connection authentication and room subscription
  - Message broadcasting infrastructure
  - Presence tracking endpoints
  - _Requirements: ActionCable-equivalent real-time features_

### 1.4 Stub Implementations

- [ ] **1.4.1 Create service stubs**
  - Implement all service traits with todo!() or simple returns
  - Ensure all function signatures compile correctly
  - Add comprehensive documentation for each method
  - Verify interfaces make sense for intended use cases
  - _Requirements: Freeze interfaces before implementation_

- [ ] **1.4.2 Create database stubs**
  - Implement Database trait with in-memory HashMap for testing
  - Create migration runner that sets up SQLite schema
  - Implement connection pool with basic configuration
  - Add health check functionality
  - _Requirements: SQLite with WAL mode, FTS5 search_

- [ ] **1.4.3 Create handler stubs**
  - Implement all HTTP handlers with placeholder responses
  - Set up middleware stack (auth, CORS, rate limiting)
  - Configure static asset serving for React app
  - Add basic error handling and logging
  - _Requirements: Complete API surface, graceful degradation_

### 1.5 Compilation & Basic Testing

- [ ] **1.5.1 Ensure clean compilation**
  - Fix all compiler errors and warnings
  - Run cargo clippy and address all suggestions
  - Set up CI/CD pipeline with compilation checks
  - Configure development environment with hot reload
  - _Requirements: Compile-first correctness_

- [ ] **1.5.2 Basic smoke tests**
  - Create health check endpoint test
  - Test server startup and shutdown
  - Verify static asset serving works
  - Test basic WebSocket connection
  - _Requirements: Walking skeleton functionality_

---

## Checkpoint 2: Core Chat Flow Implementation

**Goal**: Implement the essential "send message in room" flow end-to-end with full testing.

### 2.1 Integration Test Suite

- [ ] **2.1.1 Write end-to-end chat flow test**
  - Test: user login → join room → send message → receive via WebSocket
  - Use real HTTP client and WebSocket client for testing
  - Verify message persistence and real-time broadcasting
  - Test graceful degradation for disabled features (file uploads)
  - _Requirements: Core user journey, MVP scope validation_

- [ ] **2.1.2 Write authentication flow tests**
  - Test successful login with valid credentials
  - Test failed login with invalid credentials
  - Test session cookie creation and validation
  - Test logout and session cleanup
  - _Requirements: Rails-style session management_

- [ ] **2.1.3 Write room management tests**
  - Test room creation for different types (Open, Closed, Direct)
  - Test room membership and permissions
  - Test room listing and filtering
  - Test room deletion and cleanup
  - _Requirements: Room types and membership management_

### 2.2 Core Domain Implementation

- [ ] **2.2.1 Implement MessageService**
  - Implement create_message with validation and persistence
  - Add duplicate detection using client_message_id
  - Implement message updates and deletions with permissions
  - Add rich text processing (mentions, sound commands)
  - _Requirements: Rich text messaging, optimistic UI support_

- [ ] **2.2.2 Implement AuthService**
  - Implement password hashing with bcrypt
  - Create session management with secure tokens
  - Add rate limiting for login attempts (10 per 3 minutes)
  - Implement session validation and cleanup
  - _Requirements: Security, rate limiting, session management_

- [ ] **2.2.3 Implement RoomService**
  - Implement room creation with proper type handling
  - Add membership management and permissions
  - Implement room listing with user filtering
  - Add presence tracking and connection counting
  - _Requirements: Room types, membership, presence tracking_

### 2.3 Database Layer Implementation

- [ ] **2.3.1 Implement SQLite database layer**
  - Set up SQLite connection pool with WAL mode
  - Implement all CRUD operations with sqlx
  - Add database migrations and schema management
  - Create FTS5 search index for messages
  - _Requirements: Direct SQLite operations, FTS5 search_

- [ ] **2.3.2 Implement data persistence**
  - Create all database tables with proper indexes
  - Implement foreign key constraints and relationships
  - Add data validation at database level
  - Implement backup and recovery procedures
  - _Requirements: Data integrity, Rails-compatible schema_

### 2.4 WebSocket Broadcasting

- [ ] **2.4.1 Implement simple WebSocket broadcaster**
  - Create room-based message broadcasting (ActionCable-style)
  - Implement connection management with cleanup
  - Add presence tracking with connection counting
  - Implement typing notifications
  - _Requirements: Real-time communication, Rails ActionCable equivalent_

- [ ] **2.4.2 Implement WebSocket handlers**
  - Handle WebSocket upgrade and authentication
  - Implement room subscription and unsubscription
  - Add message broadcasting to room subscribers
  - Implement connection cleanup on disconnect
  - _Requirements: WebSocket connection management_

### 2.5 HTTP API Implementation

- [ ] **2.5.1 Implement message API endpoints**
  - POST /api/rooms/:id/messages with validation
  - GET /api/rooms/:id/messages with pagination
  - PUT /api/messages/:id with permission checks
  - DELETE /api/messages/:id with authorization
  - _Requirements: RESTful API, proper error handling_

- [ ] **2.5.2 Implement authentication middleware**
  - Session validation middleware for protected routes
  - CSRF protection with bot API bypass
  - Rate limiting middleware implementation
  - Error handling and logging middleware
  - _Requirements: Security middleware, Rails-style protection_

---

## Checkpoint 3: MVP Feature Parity

**Goal**: Complete all MVP features with comprehensive testing and graceful degradation.

### 3.1 Advanced Features Implementation

- [ ] **3.1.1 Implement search functionality**
  - Set up FTS5 full-text search with Porter stemming
  - Implement search API with proper ranking
  - Add search result pagination and filtering
  - Optimize search performance for large message volumes
  - _Requirements: Full search functionality, FTS5 implementation_

- [ ] **3.1.2 Implement bot integration**
  - Create bot authentication with API keys
  - Implement webhook delivery with 7-second timeout
  - Add bot response processing (text and binary)
  - Create bot management interface for administrators
  - _Requirements: Bot integration, webhook system_

- [ ] **3.1.3 Implement push notifications**
  - Set up WebPush with VAPID keys
  - Implement subscription management
  - Add notification payload creation and delivery
  - Create service worker for PWA support
  - _Requirements: Push notifications, PWA support_

### 3.2 Frontend Implementation

- [ ] **3.2.1 Implement React components**
  - Create MessageList component with virtualization
  - Implement MessageComposer with rich text support
  - Build RoomList component with real-time updates
  - Create UserList component with presence indicators
  - _Requirements: Complete React UI, real-time updates_

- [ ] **3.2.2 Implement feature flag components**
  - Create FeatureGate component for graceful degradation
  - Implement file upload placeholders with upgrade messaging
  - Add avatar upload placeholders with text initials
  - Create upgrade messaging system for disabled features
  - _Requirements: Graceful degradation, professional appearance_

- [ ] **3.2.3 Implement WebSocket client**
  - Create WebSocket connection with automatic reconnection
  - Implement message handling and state synchronization
  - Add typing indicators and presence updates
  - Create optimistic UI updates with rollback
  - _Requirements: Real-time frontend, connection management_

### 3.3 Security & Performance

- [ ] **3.3.1 Implement security measures**
  - Add input validation and sanitization
  - Implement CSRF protection and rate limiting
  - Add SQL injection prevention with parameterized queries
  - Create security headers and CORS configuration
  - _Requirements: Security implementation, input validation_

- [ ] **3.3.2 Performance optimization**
  - Optimize database queries with proper indexing
  - Implement connection pooling and prepared statements
  - Add caching for frequently accessed data
  - Optimize WebSocket broadcasting for multiple connections
  - _Requirements: Performance targets, scalability_

### 3.4 Testing & Quality Assurance

- [ ] **3.4.1 Comprehensive test suite**
  - Unit tests for all service methods
  - Integration tests for all API endpoints
  - Property tests for data validation and invariants
  - End-to-end tests for complete user workflows
  - _Requirements: Test coverage, quality assurance_

- [ ] **3.4.2 Performance testing**
  - Load testing for concurrent users and messages
  - WebSocket connection stress testing
  - Database performance testing with large datasets
  - Memory usage and resource consumption testing
  - _Requirements: Performance validation, resource limits_

### 3.5 Deployment & Documentation

- [ ] **3.5.1 Production deployment setup**
  - Create Docker container with embedded assets
  - Set up database volume mounting (never in container)
  - Configure environment variables and secrets
  - Create deployment scripts and documentation
  - _Requirements: Single binary deployment, database safety_

- [ ] **3.5.2 Documentation and monitoring**
  - Create API documentation with examples
  - Add deployment and configuration guides
  - Implement health checks and monitoring endpoints
  - Create troubleshooting and maintenance documentation
  - _Requirements: Operational readiness, maintainability_

---

## Success Criteria

### Checkpoint 1 Success
- [ ] All modules compile without errors or warnings
- [ ] Basic health check endpoint returns 200
- [ ] Static assets serve correctly
- [ ] All interfaces are defined and documented

### Checkpoint 2 Success
- [ ] End-to-end chat flow test passes
- [ ] Users can authenticate and join rooms
- [ ] Messages can be sent and received in real-time
- [ ] Core domain logic is fully tested

### Checkpoint 3 Success
- [ ] All MVP requirements are implemented and tested
- [ ] Performance targets are met (Rails-equivalent)
- [ ] Security measures are in place and tested
- [ ] System is ready for production deployment

---

## Anti-Coordination Compliance Checklist

Throughout implementation, ensure compliance with anti-coordination mandates:

- [ ] **No coordination layers**: Direct function calls only
- [ ] **No event buses**: Simple WebSocket broadcasting only
- [ ] **No complex state machines**: Basic enums and simple state only
- [ ] **File limit compliance**: Maximum 50 files total
- [ ] **Line limit compliance**: Maximum 500 lines per file
- [ ] **Rails parity rule**: If Rails doesn't do it, we don't do it
- [ ] **Simple error handling**: Basic Result<T, E> patterns only
- [ ] **Direct database operations**: No ORM, direct SQL with sqlx

---

## Kiro Integration Strategy

### Agent Hooks Setup
- [ ] **On save .rs files**: Run cargo check and clippy
- [ ] **On save test files**: Run relevant test suite
- [ ] **On create migration**: Generate corresponding query macros
- [ ] **On save frontend files**: Run TypeScript compilation and tests

### Kiro Usage Patterns
- [ ] **Scaffolding**: Use Kiro to generate module templates and signatures
- [ ] **Test generation**: Use Kiro to create tests from requirements
- [ ] **Implementation**: Use Kiro as pair-programmer for TDD cycles
- [ ] **Code review**: Use Kiro to check anti-coordination compliance

This implementation plan ensures a systematic, test-driven approach to building the Campfire MVP while maintaining strict adherence to anti-coordination principles and achieving "one-shot correct" implementation through comprehensive testing and incremental development.