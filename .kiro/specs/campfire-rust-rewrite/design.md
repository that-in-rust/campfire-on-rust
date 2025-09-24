# Design Document - Campfire Rust Rewrite MVP Phase 1

## IMPORTANT FOR VISUALS AND DIAGRAMS

ALL DIAGRAMS WILL BE IN MERMAID ONLY TO ENSURE EASE WITH GITHUB - DO NOT SKIP THAT

## Overview

> v0.1 note: Avatar uploads and Attachments/Uploads have been moved to .Slate/SlateBacklogIdentified20250924120426.md

This design document specifies the architecture for rewriting the Ruby on Rails Campfire chat application in Rust, following the **MVP-First Rigor** pattern with **Rails-Compatible Simplicity**. The design prioritizes proven patterns over architectural innovation, ensuring rapid delivery of a working chat application with Rust's performance benefits.

**Key Design Updates for Requirements 10 & 11:**
- **Demo Experience Architecture**: Basecamp-inspired demo mode with realistic data initialization and multi-user simulation capabilities
- **First-Run Setup System**: Simple admin account creation with environment-based configuration following DHH's radical simplicity philosophy
- **Deployment Strategy**: Single binary with automatic setup detection and Docker-first approach

**Core Design Philosophy:**
- **Rails Parity Rule**: If Rails doesn't do it, we don't do it - replicate Rails patterns exactly using idiomatic Rust
- **Anti-Coordination Mandates**: No coordination layers, event buses, or distributed complexity
- **Single Binary Deployment**: Embedded assets using Rust's compile-time inclusion
- **Type Safety First**: Leverage Rust's type system to prevent bugs at compile-time

## Architecture

### System Architecture Overview

```mermaid
graph TB
    subgraph "Single Rust Binary"
        subgraph "Web Layer"
            HTTP[HTTP Server<br/>Axum]
            WS[WebSocket Handler<br/>tokio-tungstenite]
            STATIC[Static Assets<br/>include_bytes!]
            LANDING[Landing Page<br/>Demo Mode Detection]
            SETUP_UI[First-Run Setup<br/>Admin Creation]
        end
        
        subgraph "Service Layer"
            AUTH[Authentication<br/>Service]
            MSG[Message<br/>Service]
            ROOM[Room<br/>Service]
            USER[User<br/>Service]
            SEARCH[Search<br/>Service]
            PUSH[Push Notification<br/>Service]
            DEMO[Demo Data<br/>Service]
            SETUP[First Run Setup<br/>Service]
        end
        
        subgraph "Data Layer"
            DB[(SQLite Database<br/>rusqlite)]
            FTS[(FTS5 Search<br/>Virtual Table)]
            DEMO_STORE[Demo Data<br/>Templates & Users]
        end
        
        subgraph "Background Tasks"
            WEBHOOK[Webhook<br/>Delivery]
            CLEANUP[Connection<br/>Cleanup]
            DEMO_INIT[Demo Initialization<br/>Async Task]
        end
    end
    
    HTTP --> AUTH
    HTTP --> MSG
    HTTP --> ROOM
    HTTP --> USER
    HTTP --> SEARCH
    HTTP --> SETUP
    HTTP --> LANDING
    HTTP --> SETUP_UI
    
    WS --> MSG
    WS --> ROOM
    WS --> AUTH
    
    AUTH --> DB
    MSG --> DB
    MSG --> FTS
    ROOM --> DB
    USER --> DB
    SEARCH --> FTS
    SETUP --> DB
    DEMO --> DB
    
    MSG --> WEBHOOK
    WS --> CLEANUP
    DEMO_INIT --> DEMO_STORE
    DEMO_STORE --> DB
    
    PUSH --> USER
    
    LANDING --> DEMO
    SETUP_UI --> SETUP
```

### Demo Experience and First-Run Setup Architecture

```mermaid
graph TB
    subgraph "Application Startup Flow"
        START[Application Start]
        ENV_CHECK[Check Environment Variables]
        DB_CHECK[Check Database State]
        
        START --> ENV_CHECK
        ENV_CHECK --> DB_CHECK
    end
    
    subgraph "Demo Mode Flow (CAMPFIRE_DEMO_MODE=true)"
        DEMO_DETECT[Demo Mode Detected]
        DEMO_INIT[Initialize Demo Data]
        LANDING[Professional Landing Page]
        PREVIEW[Live Chat Preview]
        DEMO_LOGIN[One-Click Demo Login]
        MULTI_USER[Multi-User Simulation]
        TOUR[Guided Feature Tour]
        
        DB_CHECK -->|Demo Mode| DEMO_DETECT
        DEMO_DETECT --> DEMO_INIT
        DEMO_INIT --> LANDING
        LANDING --> PREVIEW
        LANDING --> DEMO_LOGIN
        DEMO_LOGIN --> TOUR
        TOUR --> MULTI_USER
    end
    
    subgraph "First-Run Setup Flow (Production)"
        EMPTY_DB[Empty Database Detected]
        SETUP_PAGE[Admin Setup Page]
        ADMIN_CREATE[Create Admin Account]
        SESSION_CREATE[Create Initial Session]
        REDIRECT_CHAT[Redirect to Chat]
        
        DB_CHECK -->|Empty DB & Production| EMPTY_DB
        EMPTY_DB --> SETUP_PAGE
        SETUP_PAGE --> ADMIN_CREATE
        ADMIN_CREATE --> SESSION_CREATE
        SESSION_CREATE --> REDIRECT_CHAT
    end
    
    subgraph "Normal Operation Flow"
        EXISTING_DB[Database Has Users]
        LOGIN_PAGE[Standard Login Page]
        AUTH_FLOW[Authentication Flow]
        CHAT_APP[Chat Application]
        
        DB_CHECK -->|Has Users| EXISTING_DB
        EXISTING_DB --> LOGIN_PAGE
        LOGIN_PAGE --> AUTH_FLOW
        AUTH_FLOW --> CHAT_APP
    end
    
    subgraph "Demo Data Components"
        DEMO_USERS[8 Realistic Demo Users<br/>Admin, PM, Devs, Designers]
        DEMO_ROOMS[7 Diverse Rooms<br/>General, Dev, Design, etc.]
        DEMO_CONVOS[Sample Conversations<br/>@mentions, /play, bots]
        DEMO_BOTS[Bot Integration Examples]
        
        DEMO_INIT --> DEMO_USERS
        DEMO_INIT --> DEMO_ROOMS
        DEMO_INIT --> DEMO_CONVOS
        DEMO_INIT --> DEMO_BOTS
    end
```

### Technology Stack

**Core Framework:**
- **Web Server**: Axum (async, type-safe routing)
- **WebSocket**: tokio-tungstenite (Rails ActionCable equivalent)
- **Database**: SQLite with rusqlite (direct operations, no ORM)
- **Templates**: Askama (compile-time HTML templates)
- **Async Runtime**: Tokio (for WebSocket and background tasks)

**Key Libraries:**
- **Authentication**: bcrypt for password hashing, secure session tokens
- **Search**: SQLite FTS5 virtual tables
- **Push Notifications**: web-push crate with VAPID keys
- **Error Handling**: thiserror for structured errors, anyhow for application context
- **Serialization**: serde for JSON APIs

### Layered Architecture (L1→L2→L3)

Following the Design101 principles:

**L1 Core (Rust Language Features):**
- Ownership and borrowing for memory safety
- Result<T, E> for error handling
- Newtype pattern for type safety (UserId, RoomId, MessageId)
- RAII for resource management

**L2 Standard Library:**
- Collections (HashMap, Vec) for in-memory state
- Arc<Mutex<T>> for shared mutable state
- Channels (mpsc) for background task communication

**L3 External Dependencies:**
- Tokio for async runtime
- Axum for HTTP server
- SQLite for persistence
- Serde for serialization

## Components and Interfaces

### Core Data Models

```rust
// Type-safe ID wrappers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

// Core domain models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub bio: Option<String>,
    pub admin: bool,
    pub bot_token: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub topic: Option<String>,
    pub room_type: RoomType,
    pub created_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomType {
    Open,    // Anyone can join
    Closed,  // Invitation only
    Direct,  // Two-person direct message
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub room_id: RoomId,
    pub creator_id: UserId,
    pub content: String,
    pub client_message_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Membership {
    pub room_id: RoomId,
    pub user_id: UserId,
    pub involvement_level: InvolvementLevel,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvolvementLevel {
    Member,
    Admin,
}

// Demo and Setup Data Models (Requirements 10 & 11)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoUser {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub role_description: String,
    pub permissions_summary: String,
    pub demo_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoInitStatus {
    pub users_created: u32,
    pub rooms_created: u32,
    pub messages_created: u32,
    pub bots_configured: u32,
    pub initialization_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub database_url: String,
    pub vapid_public_key: Option<String>,
    pub vapid_private_key: Option<String>,
    pub ssl_domain: Option<String>,
    pub session_timeout_hours: u32,
    pub max_message_length: usize,
    pub enable_user_registration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub database_connected: bool,
    pub fts_search_available: bool,
    pub websocket_ready: bool,
    pub push_notifications_configured: bool,
    pub static_assets_embedded: bool,
    pub admin_account_exists: bool,
}
```

### Service Layer Interfaces

```rust
// Message Service - Core chat functionality
#[async_trait]
pub trait MessageService: Send + Sync {
    /// Creates message with deduplication (Critical Gap #1)
    /// 
    /// # Preconditions
    /// - User authenticated with room access
    /// - Content: 1-10000 chars, sanitized HTML
    /// - client_message_id: valid UUID
    /// 
    /// # Postconditions  
    /// - Returns Ok(Message) on success
    /// - Inserts row into 'messages' table
    /// - Updates room.last_message_at timestamp
    /// - Broadcasts to room subscribers via WebSocket
    /// - Deduplication: returns existing if client_message_id exists
    /// 
    /// # Error Conditions
    /// - MessageError::Authorization if user lacks room access
    /// - MessageError::InvalidContent if content violates constraints
    /// - MessageError::Database on persistence failure
    async fn create_message_with_deduplication(
        &self,
        content: String,
        room_id: RoomId,
        user_id: UserId,
        client_message_id: Uuid,
    ) -> Result<Message, MessageError>;
    
    /// Retrieves message history for a room
    async fn get_room_messages(
        &self,
        room_id: RoomId,
        user_id: UserId,
        limit: u32,
        before: Option<MessageId>,
    ) -> Result<Vec<Message>, MessageError>;
    
    /// Broadcasts message to room subscribers
    async fn broadcast_message(
        &self,
        message: &Message,
        room_id: RoomId,
    ) -> Result<(), BroadcastError>;
}

// Room Service - Room management
#[async_trait]
pub trait RoomService: Send + Sync {
    /// Creates a new room
    async fn create_room(
        &self,
        name: String,
        topic: Option<String>,
        room_type: RoomType,
        creator_id: UserId,
    ) -> Result<Room, RoomError>;
    
    /// Adds user to room
    async fn add_member(
        &self,
        room_id: RoomId,
        user_id: UserId,
        added_by: UserId,
        involvement_level: InvolvementLevel,
    ) -> Result<(), RoomError>;
    
    /// Checks if user has access to room
    async fn check_room_access(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<Option<InvolvementLevel>, RoomError>;
    
    /// Gets rooms for user
    async fn get_user_rooms(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Room>, RoomError>;
}

// Authentication Service - Session management
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Authenticates user with email/password
    async fn authenticate(
        &self,
        email: String,
        password: String,
    ) -> Result<Session, AuthError>;
    
    /// Creates secure session token (Critical Gap #4)
    async fn create_session(
        &self,
        user_id: UserId,
    ) -> Result<Session, AuthError>;
    
    /// Validates session token
    async fn validate_session(
        &self,
        token: String,
    ) -> Result<User, AuthError>;
    
    /// Revokes session
    async fn revoke_session(
        &self,
        token: String,
    ) -> Result<(), AuthError>;
}

// WebSocket Connection Manager - Real-time features
#[async_trait]
pub trait ConnectionManager: Send + Sync {
    /// Adds WebSocket connection for user
    async fn add_connection(
        &self,
        user_id: UserId,
        connection_id: ConnectionId,
        sender: WebSocketSender,
    ) -> Result<(), ConnectionError>;
    
    /// Removes WebSocket connection
    async fn remove_connection(
        &self,
        connection_id: ConnectionId,
    ) -> Result<(), ConnectionError>;
    
    /// Broadcasts message to room subscribers
    async fn broadcast_to_room(
        &self,
        room_id: RoomId,
        message: WebSocketMessage,
    ) -> Result<(), BroadcastError>;
    
    /// Gets presence information for room (Critical Gap #5)
    async fn get_room_presence(
        &self,
        room_id: RoomId,
    ) -> Result<Vec<UserId>, ConnectionError>;
    
    /// Handles missed messages on reconnection (Critical Gap #2)
    async fn send_missed_messages(
        &self,
        user_id: UserId,
        connection_id: ConnectionId,
        last_seen_message_id: Option<MessageId>,
    ) -> Result<(), ConnectionError>;
}

// Demo Data Service - Basecamp-inspired demo experience (Requirement 10)
#[async_trait]
pub trait DemoService: Send + Sync {
    /// Detects if demo mode should be enabled
    /// 
    /// # Preconditions
    /// - Environment variables checked (CAMPFIRE_DEMO_MODE)
    /// - Database state assessed
    /// 
    /// # Postconditions
    /// - Returns true if demo mode should be active
    /// - Considers environment settings and database state
    async fn should_enable_demo_mode(&self) -> Result<bool, DemoError>;
    
    /// Initializes complete demo data set
    /// 
    /// # Preconditions
    /// - Demo mode is enabled
    /// - Database is accessible
    /// 
    /// # Postconditions
    /// - Creates 8 realistic demo users with varied roles
    /// - Creates 7 diverse rooms (General, Development, Design, etc.)
    /// - Generates sample conversations with @mentions and /play commands
    /// - Includes bot integration examples
    /// - Returns initialization status
    async fn initialize_demo_data(&self) -> Result<DemoInitStatus, DemoError>;
    
    /// Gets demo user credentials for one-click login
    async fn get_demo_users(&self) -> Result<Vec<DemoUser>, DemoError>;
    
    /// Checks if demo data exists and is complete
    async fn verify_demo_data_integrity(&self) -> Result<bool, DemoError>;
}

// First-Run Setup Service - Basecamp-style admin setup (Requirement 11)
#[async_trait]
pub trait SetupService: Send + Sync {
    /// Detects if this is a first-run scenario
    /// 
    /// # Preconditions
    /// - Database is accessible
    /// - Not in demo mode
    /// 
    /// # Postconditions
    /// - Returns true if no users exist in database
    /// - Indicates first-run setup is needed
    async fn is_first_run(&self) -> Result<bool, SetupError>;
    
    /// Creates initial admin account
    /// 
    /// # Preconditions
    /// - First-run condition verified
    /// - Valid email and password provided
    /// - Email format validated
    /// - Password strength requirements met
    /// 
    /// # Postconditions
    /// - Creates admin user with full permissions
    /// - Marks user as primary administrator
    /// - Returns created user and session token
    /// - Enables subsequent normal login flow
    async fn create_admin_account(
        &self,
        email: String,
        password: String,
        name: String,
    ) -> Result<(User, SessionToken), SetupError>;
    
    /// Gets environment-based configuration
    async fn get_deployment_config(&self) -> Result<DeploymentConfig, SetupError>;
    
    /// Validates system readiness for production
    async fn validate_system_health(&self) -> Result<SystemHealth, SetupError>;
}
```

### Database Schema

```sql
-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    bio TEXT,
    admin BOOLEAN NOT NULL DEFAULT FALSE,
    bot_token TEXT UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Rooms table
CREATE TABLE rooms (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    topic TEXT,
    room_type TEXT NOT NULL CHECK (room_type IN ('open', 'closed', 'direct')),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_message_at DATETIME
);

-- Messages table with deduplication constraint (Critical Gap #1)
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    room_id TEXT NOT NULL REFERENCES rooms(id),
    creator_id TEXT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    client_message_id TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(client_message_id, room_id) -- Prevents duplicate messages
);

-- Room memberships
CREATE TABLE room_memberships (
    room_id TEXT NOT NULL REFERENCES rooms(id),
    user_id TEXT NOT NULL REFERENCES users(id),
    involvement_level TEXT NOT NULL CHECK (involvement_level IN ('member', 'admin')),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (room_id, user_id)
);

-- Sessions table for authentication
CREATE TABLE sessions (
    token TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL
);

-- Demo and setup tracking tables (Requirements 10 & 11)
CREATE TABLE system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Track demo initialization status
CREATE TABLE demo_status (
    component TEXT PRIMARY KEY,
    initialized BOOLEAN NOT NULL DEFAULT FALSE,
    initialization_time DATETIME,
    metadata TEXT -- JSON for additional info
);

-- Enhanced users table with demo and admin flags
-- (Note: This extends the existing users table structure)
-- ALTER TABLE users ADD COLUMN is_demo_user BOOLEAN NOT NULL DEFAULT FALSE;
-- ALTER TABLE users ADD COLUMN is_primary_admin BOOLEAN NOT NULL DEFAULT FALSE;
-- ALTER TABLE users ADD COLUMN role_description TEXT;
-- ALTER TABLE users ADD COLUMN demo_context TEXT;

-- FTS5 virtual table for message search
CREATE VIRTUAL TABLE messages_fts USING fts5(
    content,
    content=messages,
    content_rowid=id
);

-- Triggers to keep FTS5 in sync
CREATE TRIGGER messages_fts_insert AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
END;

CREATE TRIGGER messages_fts_delete AFTER DELETE ON messages BEGIN
    DELETE FROM messages_fts WHERE rowid = old.id;
END;

CREATE TRIGGER messages_fts_update AFTER UPDATE ON messages BEGIN
    DELETE FROM messages_fts WHERE rowid = old.id;
    INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
END;
```

## Data Models

### Type Safety Through Newtypes

All domain IDs use the newtype pattern to prevent ID confusion:

```rust
// Prevents accidentally using UserId where RoomId is expected
impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<UserId> for Uuid {
    fn from(user_id: UserId) -> Self {
        user_id.0
    }
}
```

### State Machines for Message Processing

```rust
// Type-state pattern for message validation
pub struct Message<State> {
    pub id: MessageId,
    pub room_id: RoomId,
    pub creator_id: UserId,
    pub content: String,
    pub client_message_id: Uuid,
    pub created_at: DateTime<Utc>,
    _state: PhantomData<State>,
}

pub struct Unvalidated;
pub struct Validated;
pub struct Persisted;

impl Message<Unvalidated> {
    pub fn validate(self) -> Result<Message<Validated>, ValidationError> {
        // Content length validation (1-10000 chars)
        if self.content.is_empty() || self.content.len() > 10000 {
            return Err(ValidationError::InvalidContentLength);
        }
        
        // HTML sanitization
        let sanitized_content = sanitize_html(&self.content);
        
        Ok(Message {
            id: self.id,
            room_id: self.room_id,
            creator_id: self.creator_id,
            content: sanitized_content,
            client_message_id: self.client_message_id,
            created_at: self.created_at,
            _state: PhantomData,
        })
    }
}

impl Message<Validated> {
    pub async fn persist(self, db: &Database) -> Result<Message<Persisted>, DatabaseError> {
        // Database insertion with deduplication
        // Returns existing message if client_message_id already exists
        todo!()
    }
}
```

## Error Handling

### Comprehensive Error Hierarchies

```rust
// Library-level errors using thiserror
#[derive(Error, Debug)]
pub enum MessageError {
    #[error("User {user_id} not authorized for room {room_id}")]
    Authorization { user_id: UserId, room_id: RoomId },
    
    #[error("Invalid content: {reason}")]
    InvalidContent { reason: String },
    
    #[error("Content too long: {length} chars (max: 10000)")]
    ContentTooLong { length: usize },
    
    #[error("Content too short: must not be empty")]
    ContentTooShort,
    
    #[error("Database operation failed: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("WebSocket broadcast failed: {0}")]
    Broadcast(#[from] BroadcastError),
    
    #[error("Rate limit exceeded: {limit} messages per {window}")]
    RateLimit { limit: u32, window: String },
}

#[derive(Error, Debug)]
pub enum RoomError {
    #[error("Room not found: {room_id}")]
    NotFound { room_id: RoomId },
    
    #[error("User {user_id} already member of room {room_id}")]
    AlreadyMember { user_id: UserId, room_id: RoomId },
    
    #[error("User {user_id} not authorized to add members to room {room_id}")]
    NotAuthorized { user_id: UserId, room_id: RoomId },
    
    #[error("Database operation failed: {0}")]
    Database(#[from] rusqlite::Error),
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Session not found or expired")]
    SessionExpired,
    
    #[error("User not found: {email}")]
    UserNotFound { email: String },
    
    #[error("Database operation failed: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Password hashing failed: {0}")]
    PasswordHash(#[from] bcrypt::BcryptError),
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Connection not found: {connection_id}")]
    NotFound { connection_id: ConnectionId },
    
    #[error("WebSocket send failed: {0}")]
    SendFailed(String),
    
    #[error("User {user_id} not found")]
    UserNotFound { user_id: UserId },
}

#[derive(Error, Debug)]
pub enum BroadcastError {
    #[error("No connections found for room {room_id}")]
    NoConnections { room_id: RoomId },
    
    #[error("Failed to serialize message: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("WebSocket send failed to {connection_count} connections")]
    PartialFailure { connection_count: usize },
}

// Demo and Setup Error Types (Requirements 10 & 11)

#[derive(Error, Debug)]
pub enum DemoError {
    #[error("Demo mode not enabled in environment")]
    DemoModeDisabled,
    
    #[error("Demo data initialization failed: {reason}")]
    InitializationFailed { reason: String },
    
    #[error("Demo data integrity check failed: missing {component}")]
    IntegrityCheckFailed { component: String },
    
    #[error("Demo user creation failed: {0}")]
    UserCreationFailed(String),
    
    #[error("Demo conversation generation failed: {0}")]
    ConversationGenerationFailed(String),
    
    #[error("Database operation failed: {0}")]
    Database(#[from] rusqlite::Error),
}

#[derive(Error, Debug)]
pub enum SetupError {
    #[error("Not a first-run scenario: users already exist")]
    NotFirstRun,
    
    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },
    
    #[error("Password too weak: {reason}")]
    WeakPassword { reason: String },
    
    #[error("Admin account creation failed: {0}")]
    AdminCreationFailed(String),
    
    #[error("Environment configuration invalid: {field}")]
    InvalidConfiguration { field: String },
    
    #[error("System health check failed: {component}")]
    HealthCheckFailed { component: String },
    
    #[error("Database operation failed: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Session creation failed: {0}")]
    SessionCreation(#[from] AuthError),
}
```

## Testing Strategy

### Test-Driven Development Approach

Following the TDD-First philosophy from the requirements:

**1. Type Contracts Before Code**: Define complete function signatures with all error cases first
**2. Property-Based Specifications**: Specify behavior through property tests that validate invariants
**3. Rails Parity Rule**: If Rails doesn't do it perfectly, we don't need to either - but we specify it completely
**4. Integration Test Validation**: All service boundaries tested with real dependencies

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    mod unit_tests {
        use super::*;
        
        #[test]
        fn test_message_validation() {
            // Test individual validation functions
        }
        
        #[test]
        fn test_user_id_newtype_safety() {
            // Test that UserId and RoomId cannot be confused
        }
    }
    
    mod integration_tests {
        use super::*;
        
        #[tokio::test]
        async fn test_message_creation_flow() {
            // Test complete message creation with database
        }
        
        #[tokio::test]
        async fn test_websocket_broadcast() {
            // Test real-time message broadcasting
        }
    }
    
    mod property_tests {
        use super::*;
        use proptest::prelude::*;
        
        proptest! {
            #[test]
            fn message_deduplication_idempotent(
                content in "[a-zA-Z0-9 ]{1,1000}",
                room_id in any::<Uuid>().prop_map(RoomId),
                user_id in any::<Uuid>().prop_map(UserId),
                client_id in any::<Uuid>(),
            ) {
                // Property: Same client_message_id always returns same message
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let service = create_test_service().await;
                    
                    let msg1 = service.create_message_with_deduplication(
                        content.clone(), room_id, user_id, client_id
                    ).await.unwrap();
                    
                    let msg2 = service.create_message_with_deduplication(
                        "different content".to_string(), room_id, user_id, client_id
                    ).await.unwrap();
                    
                    // Same client_message_id should return same message
                    prop_assert_eq!(msg1.id, msg2.id);
                    prop_assert_eq!(msg1.content, msg2.content); // Original preserved
                });
            }
        }
    }
}
```

### Critical Gap Testing

Each of the 5 critical gaps identified in requirements must have specific tests:

### Demo Experience and First-Run Setup Testing (Requirements 10 & 11)

```rust
#[tokio::test]
async fn test_demo_mode_detection_and_initialization() {
    // Test Requirement 10.1: Demo mode detection and landing page
    std::env::set_var("CAMPFIRE_DEMO_MODE", "true");
    let demo_service = create_test_demo_service().await;
    
    assert!(demo_service.should_enable_demo_mode().await.unwrap());
    
    // Test demo data initialization
    let init_status = demo_service.initialize_demo_data().await.unwrap();
    assert_eq!(init_status.users_created, 8); // 8 realistic demo users
    assert_eq!(init_status.rooms_created, 7); // 7 diverse rooms
    assert!(init_status.messages_created > 0); // Sample conversations
    assert!(init_status.bots_configured > 0); // Bot integration examples
}

#[tokio::test]
async fn test_multi_user_demo_simulation() {
    // Test Requirement 10.5: Multi-user simulation capability
    let demo_service = create_test_demo_service().await;
    demo_service.initialize_demo_data().await.unwrap();
    
    let demo_users = demo_service.get_demo_users().await.unwrap();
    assert_eq!(demo_users.len(), 8);
    
    // Verify different user roles and contexts
    let admin_user = demo_users.iter().find(|u| u.role_description.contains("Admin")).unwrap();
    let pm_user = demo_users.iter().find(|u| u.role_description.contains("Product Manager")).unwrap();
    
    assert!(admin_user.permissions_summary.contains("Full Permissions"));
    assert!(pm_user.demo_context.contains("Planning Focus"));
}

#[tokio::test]
async fn test_first_run_setup_detection() {
    // Test Requirement 11.1: First-run detection
    let setup_service = create_test_setup_service().await;
    let empty_db = create_empty_test_database().await;
    
    assert!(setup_service.is_first_run().await.unwrap());
    
    // Test admin account creation
    let (admin_user, session_token) = setup_service.create_admin_account(
        "admin@example.com".to_string(),
        "SecurePassword123!".to_string(),
        "System Administrator".to_string(),
    ).await.unwrap();
    
    assert!(admin_user.admin);
    assert!(!session_token.token.is_empty());
    assert!(session_token.expires_at > Utc::now());
}

#[tokio::test]
async fn test_environment_based_configuration() {
    // Test Requirement 11.5-11.6: Environment variable configuration
    std::env::set_var("VAPID_PUBLIC_KEY", "test_public_key");
    std::env::set_var("VAPID_PRIVATE_KEY", "test_private_key");
    std::env::set_var("SSL_DOMAIN", "campfire.example.com");
    std::env::set_var("SESSION_TIMEOUT_HOURS", "48");
    
    let config = AppConfig::from_env().unwrap();
    
    assert_eq!(config.vapid_public_key, Some("test_public_key".to_string()));
    assert_eq!(config.vapid_private_key, Some("test_private_key".to_string()));
    assert_eq!(config.ssl_domain, Some("campfire.example.com".to_string()));
    assert_eq!(config.session_timeout_hours, 48);
}

#[tokio::test]
async fn test_system_health_validation() {
    // Test Requirement 11.10: System health monitoring
    let setup_service = create_test_setup_service().await;
    let health = setup_service.validate_system_health().await.unwrap();
    
    assert!(health.database_connected);
    assert!(health.fts_search_available);
    assert!(health.websocket_ready);
    assert!(health.static_assets_embedded);
}

#[tokio::test]
async fn test_demo_data_integrity_verification() {
    // Test Requirement 10.6: Demo data integrity checking
    let demo_service = create_test_demo_service().await;
    
    // Initially no demo data
    assert!(!demo_service.verify_demo_data_integrity().await.unwrap());
    
    // After initialization, integrity should pass
    demo_service.initialize_demo_data().await.unwrap();
    assert!(demo_service.verify_demo_data_integrity().await.unwrap());
}
```

```rust
#[tokio::test]
async fn test_critical_gap_1_message_deduplication() {
    // Test UNIQUE constraint on (client_message_id, room_id)
    let service = create_test_service().await;
    let client_id = Uuid::new_v4();
    
    // First message should succeed
    let msg1 = service.create_message_with_deduplication(
        "First message".to_string(),
        room_id,
        user_id,
        client_id,
    ).await.unwrap();
    
    // Second message with same client_id should return existing
    let msg2 = service.create_message_with_deduplication(
        "Second message".to_string(),
        room_id,
        user_id,
        client_id,
    ).await.unwrap();
    
    assert_eq!(msg1.id, msg2.id);
    assert_eq!(msg1.content, "First message"); // Original preserved
}

#[tokio::test]
async fn test_critical_gap_2_websocket_reconnection() {
    // Test missed message delivery on reconnection
    let connection_manager = create_test_connection_manager().await;
    
    // Simulate connection drop and reconnection
    // Verify missed messages are delivered
    todo!()
}

#[tokio::test]
async fn test_critical_gap_3_sqlite_write_serialization() {
    // Test concurrent writes are properly serialized
    let service = create_test_service().await;
    
    // Spawn multiple concurrent write operations
    let handles: Vec<_> = (0..100).map(|i| {
        let service = service.clone();
        tokio::spawn(async move {
            service.create_message_with_deduplication(
                format!("Message {}", i),
                room_id,
                user_id,
                Uuid::new_v4(),
            ).await
        })
    }).collect();
    
    // All operations should succeed without conflicts
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_critical_gap_4_secure_session_tokens() {
    // Test Rails-equivalent secure token generation
    let auth_service = create_test_auth_service().await;
    
    let session1 = auth_service.create_session(user_id).await.unwrap();
    let session2 = auth_service.create_session(user_id).await.unwrap();
    
    // Tokens should be unique and cryptographically secure
    assert_ne!(session1.token, session2.token);
    assert!(session1.token.len() >= 32); // Minimum entropy
}

#[tokio::test]
async fn test_critical_gap_5_presence_tracking() {
    // Test basic presence tracking with TTL cleanup
    let connection_manager = create_test_connection_manager().await;
    
    // Add connection
    connection_manager.add_connection(user_id, connection_id, sender).await.unwrap();
    
    // Verify presence
    let presence = connection_manager.get_room_presence(room_id).await.unwrap();
    assert!(presence.contains(&user_id));
    
    // Simulate connection timeout (60 seconds)
    tokio::time::advance(Duration::from_secs(61)).await;
    
    // Verify cleanup
    let presence = connection_manager.get_room_presence(room_id).await.unwrap();
    assert!(!presence.contains(&user_id));
}
```

## Design Decisions and Rationales

### 1. SQLite Over PostgreSQL for MVP

**Decision**: Use SQLite with rusqlite for direct database operations
**Rationale**: 
- Eliminates deployment complexity (single binary)
- Sufficient for MVP scale (Rails uses similar approach for development)
- FTS5 provides excellent search capabilities
- Direct SQL operations avoid ORM complexity
- Easy to migrate to PostgreSQL later if needed

### 2. Axum Over Other Web Frameworks

**Decision**: Use Axum for HTTP server
**Rationale**:
- Type-safe routing with compile-time validation
- Excellent async performance with Tokio integration
- Tower middleware ecosystem for common functionality
- Clear separation of concerns with extractors
- Strong community adoption and maintenance

### 3. No ORM - Direct SQL Operations

**Decision**: Use rusqlite directly instead of an ORM like Diesel or SQLx
**Rationale**:
- Follows anti-coordination mandate (no complex abstractions)
- Rails uses ActiveRecord, but we prioritize simplicity over feature parity
- Direct control over SQL for performance optimization
- Easier to implement the 5 critical gaps with raw SQL
- Compile-time query validation can be added later if needed

### 4. Embedded Assets Over CDN

**Decision**: Use `include_bytes!` macro to embed all static assets
**Rationale**:
- Single binary deployment (zero coordination overhead)
- No external dependencies for asset serving
- Rust's compile-time inclusion is efficient
- Eliminates asset pipeline complexity
- Easy to switch to CDN later for scaling

### 5. tokio-tungstenite Over Higher-Level WebSocket Libraries

**Decision**: Use tokio-tungstenite directly for WebSocket handling
**Rationale**:
- Direct control over connection management (Critical Gap #2)
- Rails ActionCable equivalent - simple broadcast patterns
- No hidden coordination complexity
- Easy to implement presence tracking (Critical Gap #5)
- Predictable performance characteristics

### 6. Askama Templates Over Runtime Templating

**Decision**: Use Askama for compile-time HTML template compilation
**Rationale**:
- Compile-time template validation prevents runtime errors
- Better performance than runtime template engines
- Type-safe template context
- Familiar syntax similar to Jinja2/Django templates
- Integrates well with Rust's type system

### 7. Session-Based Authentication Over JWT

**Decision**: Use traditional session cookies with database storage
**Rationale**:
- Rails-equivalent pattern (session cookies)
- Simpler to implement securely than JWT
- Easy session revocation
- No coordination complexity
- Familiar security model

### 8. In-Memory Connection Tracking

**Decision**: Use `Arc<Mutex<HashMap>>` for WebSocket connection tracking
**Rationale**:
- Simple implementation of Critical Gap #5 (presence tracking)
- No external dependencies (Redis, etc.)
- Sufficient for single-instance deployment
- Easy to migrate to distributed solution later
- Follows Rails pattern of in-memory session storage

### 9. Background Tasks with Tokio Spawn

**Decision**: Use `tokio::spawn` for background tasks instead of job queues
**Rationale**:
- Follows anti-coordination mandate (no message queues)
- Rails uses similar approach with background jobs
- Sufficient for webhook delivery and cleanup tasks
- No external dependencies
- Easy to monitor and debug

### 10. Graceful Feature Deferrals

**Decision**: Show "Coming in v2.0" messaging for file attachments, avatars, and OpenGraph
**Rationale**:
- Maintains complete UI parity with original Campfire
- Clear user expectations about future features
- Allows focus on core chat functionality
- Provides upgrade path messaging
- Reduces MVP complexity while preserving user experience

### 11. Demo Experience Over Complex Onboarding (Requirement 10)

**Decision**: Implement Basecamp-inspired demo mode with realistic data and multi-user simulation
**Rationale**:
- Follows DHH's philosophy: "the best demo is the real product working well"
- Eliminates complex demo/trial mode transitions
- Provides immediate value assessment without setup complexity
- Enables realistic team collaboration simulation
- Reduces evaluation friction for potential users
- Maintains Rails-equivalent simplicity in implementation

### 12. First-Run Setup Over Complex Configuration (Requirement 11)

**Decision**: Simple admin account creation with environment-based configuration
**Rationale**:
- Follows Basecamp's radical simplicity approach
- Eliminates complex setup wizards and configuration screens
- Gets users into real usage immediately
- Uses convention over configuration principles
- Supports Docker-first deployment patterns
- Maintains single binary deployment benefits

### 13. Environment Detection Over Runtime Configuration

**Decision**: Use environment variables for demo mode and deployment configuration
**Rationale**:
- Follows 12-factor app principles
- Enables Docker and container deployment patterns
- Eliminates runtime configuration complexity
- Supports different deployment scenarios (demo, production, development)
- Maintains Rails-equivalent environment-based configuration
- Reduces coordination complexity in deployment

## Implementation Priority

### Phase 1: Core Infrastructure (Week 1)
1. Database schema and migrations (including demo/setup tables)
2. Basic HTTP server with Axum
3. Authentication service with session management
4. User and room services
5. Basic HTML templates with Askama
6. **First-run setup service and admin account creation (Requirement 11)**

### Phase 2: Real-Time Features (Week 2)
1. WebSocket connection management
2. Message service with deduplication (Critical Gap #1)
3. Real-time message broadcasting
4. Presence tracking (Critical Gap #5)
5. Missed message delivery (Critical Gap #2)
6. **Demo data service and initialization system (Requirement 10)**

### Phase 3: Advanced Features (Week 3)
1. Full-text search with FTS5
2. Push notifications with Web Push
3. Sound system with embedded MP3 files
4. Rich text formatting and @mentions
5. Bot API integration
6. **Demo experience UI: landing page, one-click login, guided tour (Requirement 10)**

### Phase 4: Polish and Testing (Week 4)
1. Comprehensive test suite
2. Performance optimization
3. Security hardening
4. Documentation completion
5. Deployment preparation
6. **Environment-based configuration and Docker deployment (Requirement 11)**
7. **Multi-user demo simulation and realistic conversation generation (Requirement 10)**

## Demo Experience and First-Run Setup Design

### Demo Mode Architecture (Requirement 10)

Following DHH's philosophy of "the best demo is the real product working well," the demo system provides immediate value assessment without complex setup:

```mermaid
graph TB
    subgraph "Demo Data Generation"
        PERSONAS[8 Realistic User Personas]
        ROOMS[7 Diverse Room Types]
        CONVOS[Authentic Conversations]
        BOTS[Bot Integration Examples]
        
        PERSONAS --> ADMIN[Admin User<br/>Full Permissions]
        PERSONAS --> PM[Product Manager<br/>Planning Focus]
        PERSONAS --> DEV1[Senior Developer<br/>Technical Lead]
        PERSONAS --> DEV2[Junior Developer<br/>Learning Mode]
        PERSONAS --> DESIGN1[UX Designer<br/>User Focus]
        PERSONAS --> DESIGN2[Visual Designer<br/>Brand Focus]
        PERSONAS --> SUPPORT[Support Rep<br/>Customer Focus]
        PERSONAS --> MARKETING[Marketing Lead<br/>Growth Focus]
        
        ROOMS --> GENERAL[General Discussion]
        ROOMS --> DEVELOPMENT[Development Team]
        ROOMS --> DESIGN[Design Team]
        ROOMS --> PRODUCT[Product Planning]
        ROOMS --> RANDOM[Random Chat]
        ROOMS --> SUPPORT_ROOM[Customer Support]
        ROOMS --> MARKETING_ROOM[Marketing Team]
        
        CONVOS --> MENTIONS[@mention Examples]
        CONVOS --> SOUNDS[/play Sound Commands]
        CONVOS --> TECHNICAL[Technical Discussions]
        CONVOS --> PLANNING[Product Planning]
        CONVOS --> CASUAL[Casual Team Chat]
        
        BOTS --> WEBHOOK_BOT[Webhook Integration]
        BOTS --> STATUS_BOT[Status Updates]
        BOTS --> DEPLOY_BOT[Deployment Notifications]
    end
    
    subgraph "Demo Experience Flow"
        LANDING[Professional Landing Page]
        PREVIEW[Live Chat Preview]
        LOGIN_DEMO[One-Click Demo Login]
        TOUR[Guided Feature Tour]
        MULTI[Multi-User Simulation]
        
        LANDING --> PREVIEW
        LANDING --> LOGIN_DEMO
        LOGIN_DEMO --> TOUR
        TOUR --> MULTI
    end
```

### First-Run Setup Architecture (Requirement 11)

Implements Basecamp's radical simplicity approach with immediate real usage:

```mermaid
graph TB
    subgraph "Environment Detection"
        ENV_VARS[Environment Variables]
        DEMO_MODE[CAMPFIRE_DEMO_MODE]
        SSL_DOMAIN[SSL_DOMAIN]
        VAPID_KEYS[VAPID_PUBLIC_KEY<br/>VAPID_PRIVATE_KEY]
        DB_URL[DATABASE_URL]
        
        ENV_VARS --> DEMO_MODE
        ENV_VARS --> SSL_DOMAIN
        ENV_VARS --> VAPID_KEYS
        ENV_VARS --> DB_URL
    end
    
    subgraph "Setup Decision Tree"
        START[Application Start]
        CHECK_ENV[Check Environment]
        CHECK_DB[Check Database State]
        
        DEMO_PATH[Demo Mode Path]
        SETUP_PATH[First-Run Setup Path]
        NORMAL_PATH[Normal Operation Path]
        
        START --> CHECK_ENV
        CHECK_ENV --> CHECK_DB
        CHECK_DB -->|Demo Mode + Empty DB| DEMO_PATH
        CHECK_DB -->|Production + Empty DB| SETUP_PATH
        CHECK_DB -->|Has Users| NORMAL_PATH
    end
    
    subgraph "Admin Account Creation"
        SETUP_FORM[Admin Setup Form]
        VALIDATE[Validate Credentials]
        CREATE_ADMIN[Create Admin User]
        CREATE_SESSION[Create Session]
        REDIRECT[Redirect to Chat]
        
        SETUP_PATH --> SETUP_FORM
        SETUP_FORM --> VALIDATE
        VALIDATE --> CREATE_ADMIN
        CREATE_ADMIN --> CREATE_SESSION
        CREATE_SESSION --> REDIRECT
    end
```

### Configuration Management Pattern

```rust
// Environment-based configuration following 12-factor principles
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub demo_mode: bool,
    pub database_url: String,
    pub vapid_public_key: Option<String>,
    pub vapid_private_key: Option<String>,
    pub ssl_domain: Option<String>,
    pub session_timeout_hours: u32,
    pub max_message_length: usize,
    pub enable_user_registration: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            demo_mode: env::var("CAMPFIRE_DEMO_MODE")
                .unwrap_or_default()
                .parse()
                .unwrap_or(false),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:campfire.db".to_string()),
            vapid_public_key: env::var("VAPID_PUBLIC_KEY").ok(),
            vapid_private_key: env::var("VAPID_PRIVATE_KEY").ok(),
            ssl_domain: env::var("SSL_DOMAIN").ok(),
            session_timeout_hours: env::var("SESSION_TIMEOUT_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            max_message_length: env::var("MAX_MESSAGE_LENGTH")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
            enable_user_registration: env::var("ENABLE_USER_REGISTRATION")
                .unwrap_or_default()
                .parse()
                .unwrap_or(false),
        })
    }
}
```

This design provides a solid foundation for implementing the Campfire Rust rewrite while adhering to the anti-coordination constraints and Rails parity requirements specified in the requirements document. The new demo experience and first-run setup components follow DHH's radical simplicity philosophy while providing immediate value to users evaluating or deploying the system.
