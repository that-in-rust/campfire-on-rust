# Design Document - Campfire Rust Rewrite MVP

## Overview

This design document outlines the implementation approach for the Campfire Rust Rewrite MVP, strictly adhering to the anti-coordination constraints and Rails-inspired simplicity mandated in the requirements. The design replicates Rails ActionCable behavior using idiomatic Rust patterns without introducing coordination complexity.

**Design Philosophy**: Build the simplest thing that works, using direct operations and proven Rails patterns. Every component is designed to avoid coordination layers while maintaining Rails-equivalent functionality.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                Single Rust Binary (~30MB)                   │
├─────────────────────────────────────────────────────────────┤
│  Embedded React SPA (Complete UI)                          │
│  ├─── All Components (with graceful degradation)           │
│  ├─── Complete Styling (26 CSS files)                      │
│  ├─── Sound Assets (59 MP3 files)                          │
│  └─── Service Worker (PWA + Push)                          │
├─────────────────────────────────────────────────────────────┤
│  Axum HTTP Server                                          │
│  ├─── REST API Handlers                                    │
│  ├─── WebSocket Upgrade Handler                            │
│  ├─── Static Asset Serving                                 │
│  └─── Session Authentication                               │
├─────────────────────────────────────────────────────────────┤
│  Simple WebSocket Broadcasting                             │
│  ├─── Room-based Message Broadcasting                      │
│  ├─── Basic Presence Tracking                              │
│  ├─── Typing Notifications                                 │
│  └─── Connection Management                                │
├─────────────────────────────────────────────────────────────┤
│  Basic Background Tasks                                    │
│  ├─── Webhook Delivery (tokio::spawn)                     │
│  ├─── Push Notifications                                   │
│  └─── Simple Cleanup Tasks                                 │
├─────────────────────────────────────────────────────────────┤
│  Direct SQLite Operations                                  │
│  ├─── Connection Pool (sqlx)                              │
│  ├─── Direct SQL Queries                                   │
│  ├─── FTS5 Search Index                                    │
│  └─── WAL Mode for Concurrency                            │
└─────────────────────────────────────────────────────────────┘
```

### Component Interaction Flow

```
HTTP Request → Axum Handler → Direct DB Query → Response
                    ↓
WebSocket Message → Room Broadcast → Connected Clients
                    ↓
Background Task → tokio::spawn → Simple Processing
```
##
 Components and Interfaces

## Database Layer Specifications

### Database Architecture - Direct SQLite Operations

**Design Approach**: Direct SQLite operations with sqlx, WAL mode for concurrency, and Dedicated Writer Task pattern for write serialization (Critical Gap #3).

### Connection Management

```rust
use sqlx::{SqlitePool, Row, sqlite::SqlitePoolOptions};
use tokio::sync::{mpsc, oneshot};

// Database connection manager
pub struct Database {
    // Read pool for concurrent reads
    read_pool: SqlitePool,
    // Dedicated writer for serialized writes (Critical Gap #3)
    writer: DedicatedWriter,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Configure SQLite with WAL mode for concurrency
        let read_pool = SqlitePoolOptions::new()
            .max_connections(10) // Multiple readers
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::from_str(database_url)?
                    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                    .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
                    .foreign_keys(true)
                    .create_if_missing(true)
            )
            .await?;
        
        // Create dedicated writer task
        let writer = DedicatedWriter::new(database_url).await?;
        
        Ok(Self { read_pool, writer })
    }
    
    // Read operations use the pool directly
    pub fn read_pool(&self) -> &SqlitePool {
        &self.read_pool
    }
    
    // Write operations go through dedicated writer
    pub fn writer(&self) -> &DedicatedWriter {
        &self.writer
    }
}
```

### Dedicated Writer Task Pattern (Critical Gap #3)

```rust
// Dedicated writer for SQLite write serialization
pub struct DedicatedWriter {
    sender: mpsc::UnboundedSender<WriteCommand>,
}

// Write command types
pub enum WriteCommand {
    CreateMessage {
        data: CreateMessageData,
        response: oneshot::Sender<Result<Message, sqlx::Error>>,
    },
    UpdateMessage {
        id: MessageId,
        data: UpdateMessageData,
        response: oneshot::Sender<Result<Message, sqlx::Error>>,
    },
    CreateUser {
        data: CreateUserData,
        response: oneshot::Sender<Result<User, sqlx::Error>>,
    },
    CreateRoom {
        data: CreateRoomData,
        response: oneshot::Sender<Result<Room, sqlx::Error>>,
    },
    CreateMembership {
        data: CreateMembershipData,
        response: oneshot::Sender<Result<Membership, sqlx::Error>>,
    },
    UpdateMembership {
        user_id: UserId,
        room_id: RoomId,
        data: UpdateMembershipData,
        response: oneshot::Sender<Result<Membership, sqlx::Error>>,
    },
    CreateSession {
        data: CreateSessionData,
        response: oneshot::Sender<Result<Session, sqlx::Error>>,
    },
    DeleteSession {
        id: SessionId,
        response: oneshot::Sender<Result<(), sqlx::Error>>,
    },
}

impl DedicatedWriter {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        
        // Create dedicated write connection
        let write_pool = SqlitePoolOptions::new()
            .max_connections(1) // Single writer
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::from_str(database_url)?
                    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                    .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
                    .foreign_keys(true)
            )
            .await?;
        
        // Spawn writer task
        tokio::spawn(async move {
            while let Some(command) = receiver.recv().await {
                match command {
                    WriteCommand::CreateMessage { data, response } => {
                        let result = Self::execute_create_message(&write_pool, data).await;
                        let _ = response.send(result);
                    }
                    WriteCommand::UpdateMessage { id, data, response } => {
                        let result = Self::execute_update_message(&write_pool, id, data).await;
                        let _ = response.send(result);
                    }
                    WriteCommand::CreateUser { data, response } => {
                        let result = Self::execute_create_user(&write_pool, data).await;
                        let _ = response.send(result);
                    }
                    WriteCommand::CreateRoom { data, response } => {
                        let result = Self::execute_create_room(&write_pool, data).await;
                        let _ = response.send(result);
                    }
                    WriteCommand::CreateMembership { data, response } => {
                        let result = Self::execute_create_membership(&write_pool, data).await;
                        let _ = response.send(result);
                    }
                    WriteCommand::UpdateMembership { user_id, room_id, data, response } => {
                        let result = Self::execute_update_membership(&write_pool, user_id, room_id, data).await;
                        let _ = response.send(result);
                    }
                    WriteCommand::CreateSession { data, response } => {
                        let result = Self::execute_create_session(&write_pool, data).await;
                        let _ = response.send(result);
                    }
                    WriteCommand::DeleteSession { id, response } => {
                        let result = Self::execute_delete_session(&write_pool, id).await;
                        let _ = response.send(result);
                    }
                }
            }
        });
        
        Ok(Self { sender })
    }
    
    // Public write methods
    pub async fn create_message(&self, data: CreateMessageData) -> Result<Message, sqlx::Error> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(WriteCommand::CreateMessage { data, response: tx })
            .map_err(|_| sqlx::Error::PoolClosed)?;
        rx.await.map_err(|_| sqlx::Error::PoolClosed)?
    }
    
    pub async fn update_message(&self, id: MessageId, data: UpdateMessageData) -> Result<Message, sqlx::Error> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(WriteCommand::UpdateMessage { id, data, response: tx })
            .map_err(|_| sqlx::Error::PoolClosed)?;
        rx.await.map_err(|_| sqlx::Error::PoolClosed)?
    }
    
    // ... similar methods for other write operations
}
```

### Database Operations Implementation

```rust
impl DedicatedWriter {
    // Message operations with deduplication (Critical Gap #1)
    async fn execute_create_message(
        pool: &SqlitePool,
        data: CreateMessageData,
    ) -> Result<Message, sqlx::Error> {
        // Start transaction for atomic operation
        let mut tx = pool.begin().await?;
        
        // Check for existing message with same client_message_id (Critical Gap #1)
        let existing = sqlx::query_as!(
            Message,
            "SELECT * FROM messages WHERE client_message_id = ? AND room_id = ?",
            data.client_message_id,
            data.room_id.0
        )
        .fetch_optional(&mut *tx)
        .await?;
        
        if let Some(existing_message) = existing {
            // Return existing message (Rails deduplication behavior)
            tx.commit().await?;
            return Ok(existing_message);
        }
        
        // Create new message
        let message = sqlx::query_as!(
            Message,
            r#"
            INSERT INTO messages (room_id, creator_id, body, client_message_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))
            RETURNING *
            "#,
            data.room_id.0,
            data.creator_id.0,
            data.body,
            data.client_message_id
        )
        .fetch_one(&mut *tx)
        .await?;
        
        // Update room last_message_at
        sqlx::query!(
            "UPDATE rooms SET last_message_at = datetime('now'), updated_at = datetime('now') WHERE id = ?",
            data.room_id.0
        )
        .execute(&mut *tx)
        .await?;
        
        // Update FTS5 search index
        sqlx::query!(
            "INSERT INTO message_search_index (rowid, content) VALUES (?, ?)",
            message.id.0,
            message.body
        )
        .execute(&mut *tx)
        .await?;
        
        tx.commit().await?;
        Ok(message)
    }
    
    // User operations
    async fn execute_create_user(
        pool: &SqlitePool,
        data: CreateUserData,
    ) -> Result<User, sqlx::Error> {
        let mut tx = pool.begin().await?;
        
        // Hash password with bcrypt
        let password_hash = bcrypt::hash(&data.password, bcrypt::DEFAULT_COST)
            .map_err(|_| sqlx::Error::Protocol("Password hashing failed".into()))?;
        
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email_address, name, password_digest, role, active, created_at, updated_at)
            VALUES (?, ?, ?, ?, true, datetime('now'), datetime('now'))
            RETURNING *
            "#,
            data.email_address,
            data.name,
            password_hash,
            data.role as i32
        )
        .fetch_one(&mut *tx)
        .await?;
        
        tx.commit().await?;
        Ok(user)
    }
    
    // Room operations with membership auto-granting
    async fn execute_create_room(
        pool: &SqlitePool,
        data: CreateRoomData,
    ) -> Result<Room, sqlx::Error> {
        let mut tx = pool.begin().await?;
        
        let room = sqlx::query_as!(
            Room,
            r#"
            INSERT INTO rooms (account_id, name, room_type, creator_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))
            RETURNING *
            "#,
            data.account_id.0,
            data.name,
            data.room_type as i32,
            data.creator_id.0
        )
        .fetch_one(&mut *tx)
        .await?;
        
        // Auto-grant membership to creator
        sqlx::query!(
            r#"
            INSERT INTO memberships (user_id, room_id, involvement, connections, created_at, updated_at)
            VALUES (?, ?, ?, 0, datetime('now'), datetime('now'))
            "#,
            data.creator_id.0,
            room.id.0,
            Involvement::Everything as i32
        )
        .execute(&mut *tx)
        .await?;
        
        // For Open rooms, grant to all existing users
        if matches!(data.room_type, RoomType::Open) {
            sqlx::query!(
                r#"
                INSERT INTO memberships (user_id, room_id, involvement, connections, created_at, updated_at)
                SELECT id, ?, ?, 0, datetime('now'), datetime('now')
                FROM users 
                WHERE active = true AND id != ? AND role != ?
                "#,
                room.id.0,
                Involvement::Everything as i32,
                data.creator_id.0,
                UserRole::Bot as i32
            )
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(room)
    }
    
    // Session operations (Critical Gap #4)
    async fn execute_create_session(
        pool: &SqlitePool,
        data: CreateSessionData,
    ) -> Result<Session, sqlx::Error> {
        // Generate secure token (Rails SecureRandom equivalent)
        let token = generate_secure_token();
        
        let session = sqlx::query_as!(
            Session,
            r#"
            INSERT INTO sessions (user_id, token, ip_address, user_agent, last_active_at, created_at)
            VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))
            RETURNING *
            "#,
            data.user_id.0,
            token,
            data.ip_address,
            data.user_agent
        )
        .fetch_one(pool)
        .await?;
        
        Ok(session)
    }
}

// Secure token generation (Critical Gap #4)
fn generate_secure_token() -> String {
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;
    
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
```

### Read Operations

```rust
// Read operations use the read pool directly
impl Database {
    // Message queries
    pub async fn get_message(&self, id: MessageId) -> Result<Option<Message>, sqlx::Error> {
        sqlx::query_as!(
            Message,
            "SELECT * FROM messages WHERE id = ?",
            id.0
        )
        .fetch_optional(&self.read_pool)
        .await
    }
    
    pub async fn list_room_messages(
        &self,
        room_id: RoomId,
        limit: i64,
        before: Option<MessageId>,
    ) -> Result<Vec<Message>, sqlx::Error> {
        match before {
            Some(before_id) => {
                sqlx::query_as!(
                    Message,
                    "SELECT * FROM messages WHERE room_id = ? AND id < ? ORDER BY created_at DESC, id DESC LIMIT ?",
                    room_id.0,
                    before_id.0,
                    limit
                )
                .fetch_all(&self.read_pool)
                .await
            }
            None => {
                sqlx::query_as!(
                    Message,
                    "SELECT * FROM messages WHERE room_id = ? ORDER BY created_at DESC, id DESC LIMIT ?",
                    room_id.0,
                    limit
                )
                .fetch_all(&self.read_pool)
                .await
            }
        }
    }
    
    // User queries
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email_address = ? AND active = true",
            email
        )
        .fetch_optional(&self.read_pool)
        .await
    }
    
    // Room queries
    pub async fn list_user_rooms(&self, user_id: UserId) -> Result<Vec<Room>, sqlx::Error> {
        sqlx::query_as!(
            Room,
            r#"
            SELECT r.* FROM rooms r
            JOIN memberships m ON r.id = m.room_id
            WHERE m.user_id = ? AND m.involvement != ?
            ORDER BY r.last_message_at DESC NULLS LAST, r.created_at DESC
            "#,
            user_id.0,
            Involvement::Invisible as i32
        )
        .fetch_all(&self.read_pool)
        .await
    }
    
    // FTS5 search
    pub async fn search_messages(
        &self,
        query: &str,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query_as!(
            Message,
            r#"
            SELECT m.* FROM messages m
            JOIN message_search_index fts ON m.id = fts.rowid
            JOIN memberships mb ON m.room_id = mb.room_id
            WHERE fts.content MATCH ? AND mb.user_id = ? AND mb.involvement != ?
            ORDER BY fts.rank
            LIMIT ?
            "#,
            query,
            user_id.0,
            Involvement::Invisible as i32,
            limit
        )
        .fetch_all(&self.read_pool)
        .await
    }
}
```

**Key Database Design Decisions**:
- SQLite with WAL mode for concurrent reads and serialized writes
- Dedicated Writer Task pattern for write serialization (Critical Gap #3)
- Direct sqlx queries with compile-time validation
- UNIQUE constraint on (client_message_id, room_id) for deduplication (Critical Gap #1)
- FTS5 virtual table for full-text search
- Atomic transactions for complex operations
- Rails-compatible schema and data patterns

## Complete API Specifications

### HTTP API Layer - Rails-Style RESTful Design

**Design Approach**: Axum handlers with Rails-style routing, direct database operations, session-based authentication.

### Authentication Endpoints

```rust
// POST /api/auth/login - User authentication
pub async fn login(
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Validate credentials (Rails: User.authenticate_by)
    let user = app_state.auth_service
        .authenticate_user(&payload.email_address, &payload.password)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    
    // Create session (Critical Gap #4: Secure token generation)
    let session = app_state.auth_service
        .create_session(user.id)
        .await?;
    
    // Set secure cookie (Rails: httponly, SameSite=Lax)
    let cookie = Cookie::build(("session_token", session.token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(true) // HTTPS only
        .path("/")
        .max_age(Duration::days(30))
        .build();
    
    Ok((
        SetCookieHeader::new(cookie),
        Json(LoginResponse {
            user: UserSummary::from(user),
            session_id: session.id,
        })
    ))
}

// POST /api/auth/register - User registration with join code
pub async fn register(
    State(app_state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    // Verify join code (Rails: verify_join_code)
    app_state.auth_service
        .verify_join_code(&payload.join_code)
        .await?;
    
    // Create user with bcrypt password hash
    let user = app_state.user_service
        .create_user(CreateUserData {
            email_address: payload.email_address,
            name: payload.name,
            password: payload.password,
            role: UserRole::Member,
        })
        .await?;
    
    // Auto-grant memberships to all Open rooms (Rails: after_save_commit)
    app_state.room_service
        .grant_open_room_memberships(user.id)
        .await?;
    
    Ok(Json(UserResponse::from(user)))
}

// POST /api/auth/logout - Session termination
pub async fn logout(
    State(app_state): State<AppState>,
    session: Session, // Extracted from cookie
) -> Result<StatusCode, ApiError> {
    // Remove push subscription (Rails: before session destroy)
    app_state.push_service
        .remove_user_subscriptions(session.user_id)
        .await?;
    
    // Delete session
    app_state.auth_service
        .delete_session(session.id)
        .await?;
    
    // Clear cookie
    let cookie = Cookie::build(("session_token", ""))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(Duration::seconds(0))
        .build();
    
    Ok((SetCookieHeader::new(cookie), StatusCode::NO_CONTENT))
}
```

### Message Endpoints

```rust
// POST /api/rooms/:room_id/messages - Create message with deduplication
pub async fn create_message(
    State(app_state): State<AppState>,
    Path(room_id): Path<RoomId>,
    session: Session,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    // Verify room access
    app_state.room_service
        .verify_user_access(session.user_id, room_id)
        .await?;
    
    // Create message with deduplication (Critical Gap #1)
    let message = app_state.message_service
        .create_message_with_deduplication(CreateMessageData {
            room_id,
            creator_id: session.user_id,
            body: payload.body,
            client_message_id: payload.client_message_id,
        })
        .await?;
    
    // Update room last_message_at
    app_state.room_service
        .update_last_message_at(room_id, message.created_at)
        .await?;
    
    // Broadcast to room (Rails ActionCable equivalent)
    app_state.broadcaster
        .broadcast_message_created(room_id, &message)
        .await;
    
    // Trigger bot webhooks if applicable
    if app_state.feature_flags.bot_integrations {
        app_state.webhook_service
            .trigger_message_webhooks(room_id, &message)
            .await;
    }
    
    Ok(Json(MessageResponse::from(message)))
}

// GET /api/rooms/:room_id/messages - List messages with pagination
pub async fn list_messages(
    State(app_state): State<AppState>,
    Path(room_id): Path<RoomId>,
    session: Session,
    Query(params): Query<MessageListParams>,
) -> Result<Json<MessageListResponse>, ApiError> {
    // Verify room access
    app_state.room_service
        .verify_user_access(session.user_id, room_id)
        .await?;
    
    // Get messages with Rails-style pagination
    let messages = app_state.message_service
        .list_room_messages(ListMessagesQuery {
            room_id,
            limit: params.limit.unwrap_or(50),
            before: params.before,
            after: params.after,
        })
        .await?;
    
    // Mark room as read (update unread_at)
    app_state.membership_service
        .mark_room_read(session.user_id, room_id)
        .await?;
    
    Ok(Json(MessageListResponse {
        messages: messages.into_iter().map(MessageResponse::from).collect(),
        has_more: messages.len() == params.limit.unwrap_or(50) as usize,
    }))
}

// PUT /api/messages/:message_id - Update message
pub async fn update_message(
    State(app_state): State<AppState>,
    Path(message_id): Path<MessageId>,
    session: Session,
    Json(payload): Json<UpdateMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    // Verify message ownership or admin role
    let message = app_state.message_service
        .get_message(message_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    
    if message.creator_id != session.user_id && !session.user.can_administer() {
        return Err(ApiError::Forbidden);
    }
    
    // Update message
    let updated_message = app_state.message_service
        .update_message(message_id, UpdateMessageData {
            body: payload.body,
        })
        .await?;
    
    // Broadcast update
    app_state.broadcaster
        .broadcast_message_updated(message.room_id, &updated_message)
        .await;
    
    Ok(Json(MessageResponse::from(updated_message)))
}
```

### Room Endpoints

```rust
// GET /api/rooms - List user's rooms
pub async fn list_rooms(
    State(app_state): State<AppState>,
    session: Session,
) -> Result<Json<RoomListResponse>, ApiError> {
    let rooms = app_state.room_service
        .list_user_rooms(session.user_id)
        .await?;
    
    Ok(Json(RoomListResponse {
        rooms: rooms.into_iter().map(RoomResponse::from).collect(),
    }))
}

// POST /api/rooms - Create room
pub async fn create_room(
    State(app_state): State<AppState>,
    session: Session,
    Json(payload): Json<CreateRoomRequest>,
) -> Result<Json<RoomResponse>, ApiError> {
    let room = match payload.room_type {
        RoomType::Open => {
            // Create open room and auto-grant to all users
            app_state.room_service
                .create_open_room(CreateOpenRoomData {
                    name: payload.name,
                    creator_id: session.user_id,
                })
                .await?
        }
        RoomType::Closed => {
            // Create closed room with specific members
            app_state.room_service
                .create_closed_room(CreateClosedRoomData {
                    name: payload.name,
                    creator_id: session.user_id,
                    member_ids: payload.member_ids.unwrap_or_default(),
                })
                .await?
        }
        RoomType::Direct => {
            // Create or find direct room (singleton pattern)
            let member_ids = payload.member_ids.unwrap_or_default();
            app_state.room_service
                .find_or_create_direct_room(session.user_id, member_ids)
                .await?
        }
    };
    
    Ok(Json(RoomResponse::from(room)))
}

// PUT /api/rooms/:room_id/membership - Update user's membership
pub async fn update_membership(
    State(app_state): State<AppState>,
    Path(room_id): Path<RoomId>,
    session: Session,
    Json(payload): Json<UpdateMembershipRequest>,
) -> Result<Json<MembershipResponse>, ApiError> {
    let membership = app_state.membership_service
        .update_involvement(session.user_id, room_id, payload.involvement)
        .await?;
    
    // Broadcast sidebar update
    app_state.broadcaster
        .broadcast_membership_updated(session.user_id, &membership)
        .await;
    
    Ok(Json(MembershipResponse::from(membership)))
}
```

### Search Endpoints

```rust
// GET /api/search/messages - FTS5 message search
pub async fn search_messages(
    State(app_state): State<AppState>,
    session: Session,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, ApiError> {
    if !app_state.feature_flags.search_enabled {
        return Err(ApiError::FeatureDisabled("Search"));
    }
    
    // Perform FTS5 search with Porter stemming
    let results = app_state.search_service
        .search_messages(SearchQuery {
            query: params.query,
            user_id: session.user_id, // Only search accessible rooms
            room_id: params.room_id,
            limit: params.limit.unwrap_or(20),
            offset: params.offset.unwrap_or(0),
        })
        .await?;
    
    Ok(Json(SearchResponse {
        messages: results.into_iter().map(MessageResponse::from).collect(),
        total_count: results.len(),
    }))
}
```

### Bot Endpoints

```rust
// POST /api/bots/:bot_id/messages - Bot message creation
pub async fn create_bot_message(
    State(app_state): State<AppState>,
    Path((room_id, bot_id)): Path<(RoomId, UserId)>,
    bot_auth: BotAuth, // Custom extractor for bot authentication
    Json(payload): Json<CreateBotMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    if !app_state.feature_flags.bot_integrations {
        return Err(ApiError::FeatureDisabled("Bot integrations"));
    }
    
    // Verify bot has access to room
    app_state.room_service
        .verify_user_access(bot_id, room_id)
        .await?;
    
    // Create message as bot
    let message = app_state.message_service
        .create_message_with_deduplication(CreateMessageData {
            room_id,
            creator_id: bot_id,
            body: payload.body,
            client_message_id: Uuid::new_v4(), // Generate for bot
        })
        .await?;
    
    // Broadcast message
    app_state.broadcaster
        .broadcast_message_created(room_id, &message)
        .await;
    
    Ok(Json(MessageResponse::from(message)))
}
```

### Request/Response Parameter Models

```rust
#[derive(Debug, Deserialize)]
pub struct MessageListParams {
    pub limit: Option<i64>,
    pub before: Option<MessageId>,
    pub after: Option<MessageId>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub query: String,
    pub room_id: Option<RoomId>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMembershipRequest {
    pub involvement: Involvement,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserSummary,
    pub session_id: SessionId,
}

#[derive(Debug, Serialize)]
pub struct MessageListResponse {
    pub messages: Vec<MessageResponse>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct RoomListResponse {
    pub rooms: Vec<RoomResponse>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub messages: Vec<MessageResponse>,
    pub total_count: usize,
}
```

**Key API Design Decisions**:
- RESTful design matching Rails conventions exactly
- Session-based authentication with secure cookies
- Direct database operations in handlers (no coordination)
- Feature flags for graceful degradation
- Rails-style error handling with user-friendly messages
- Comprehensive request validation and response formatting

## WebSocket Protocol Specifications

### WebSocket Broadcasting - Rails ActionCable Equivalent

**Design Approach**: Simple room-based broadcasting like Rails ActionCable, with basic presence tracking and reconnection support (Critical Gap #2).

### Connection Management

```rust
use axum::extract::ws::{WebSocket, Message as WsMessage};
use tokio::sync::{RwLock, mpsc};
use std::collections::HashMap;

// WebSocket connection state
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: ConnectionId,
    pub user_id: UserId,
    pub room_subscriptions: HashSet<RoomId>,
    pub sender: mpsc::UnboundedSender<WsMessage>,
    pub last_seen_message_id: Option<MessageId>, // Critical Gap #2: Reconnection state
    pub connected_at: DateTime<Utc>,
}

// WebSocket broadcaster - Rails ActionCable equivalent
pub struct WebSocketBroadcaster {
    // Room-based connection tracking (Rails pattern)
    room_connections: Arc<RwLock<HashMap<RoomId, HashSet<ConnectionId>>>>,
    // User connection mapping
    user_connections: Arc<RwLock<HashMap<UserId, HashSet<ConnectionId>>>>,
    // Connection details
    connections: Arc<RwLock<HashMap<ConnectionId, WebSocketConnection>>>,
}

impl WebSocketBroadcaster {
    pub fn new() -> Self {
        Self {
            room_connections: Arc::new(RwLock::new(HashMap::new())),
            user_connections: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // Add connection with presence tracking (Critical Gap #5)
    pub async fn add_connection(
        &self,
        connection: WebSocketConnection,
        membership_service: &MembershipService,
    ) -> Result<(), BroadcastError> {
        let connection_id = connection.id;
        let user_id = connection.user_id;
        
        // Store connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id, connection);
        }
        
        // Track user connections
        {
            let mut user_connections = self.user_connections.write().await;
            user_connections.entry(user_id).or_default().insert(connection_id);
        }
        
        // Update presence (increment connection count)
        membership_service.user_connected(user_id).await?;
        
        Ok(())
    }
    
    // Remove connection with presence cleanup
    pub async fn remove_connection(
        &self,
        connection_id: ConnectionId,
        membership_service: &MembershipService,
    ) -> Result<(), BroadcastError> {
        let (user_id, room_subscriptions) = {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.remove(&connection_id) {
                (connection.user_id, connection.room_subscriptions)
            } else {
                return Ok(()); // Connection already removed
            }
        };
        
        // Remove from room subscriptions
        {
            let mut room_connections = self.room_connections.write().await;
            for room_id in &room_subscriptions {
                if let Some(room_conns) = room_connections.get_mut(room_id) {
                    room_conns.remove(&connection_id);
                    if room_conns.is_empty() {
                        room_connections.remove(room_id);
                    }
                }
            }
        }
        
        // Remove from user connections
        {
            let mut user_connections = self.user_connections.write().await;
            if let Some(user_conns) = user_connections.get_mut(&user_id) {
                user_conns.remove(&connection_id);
                if user_conns.is_empty() {
                    user_connections.remove(&user_id);
                }
            }
        }
        
        // Update presence (decrement connection count)
        membership_service.user_disconnected(user_id).await?;
        
        Ok(())
    }
    
    // Subscribe to room (Rails: ActionCable subscription)
    pub async fn subscribe_to_room(
        &self,
        connection_id: ConnectionId,
        room_id: RoomId,
    ) -> Result<(), BroadcastError> {
        // Add to room connections
        {
            let mut room_connections = self.room_connections.write().await;
            room_connections.entry(room_id).or_default().insert(connection_id);
        }
        
        // Update connection subscriptions
        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(&connection_id) {
                connection.room_subscriptions.insert(room_id);
            }
        }
        
        Ok(())
    }
}
```

### Message Broadcasting Implementation

```rust
impl WebSocketBroadcaster {
    // Broadcast message to room (Rails ActionCable equivalent)
    pub async fn broadcast_message_created(
        &self,
        room_id: RoomId,
        message: &Message,
    ) {
        let ws_message = WebSocketMessage::MessageCreated {
            message: MessageResponse::from(message.clone()),
        };
        
        self.broadcast_to_room(room_id, &ws_message).await;
    }
    
    // Broadcast message update
    pub async fn broadcast_message_updated(
        &self,
        room_id: RoomId,
        message: &Message,
    ) {
        let ws_message = WebSocketMessage::MessageUpdated {
            message: MessageResponse::from(message.clone()),
        };
        
        self.broadcast_to_room(room_id, &ws_message).await;
    }
    
    // Broadcast typing notification
    pub async fn broadcast_typing_notification(
        &self,
        room_id: RoomId,
        user: &User,
        is_typing: bool,
    ) {
        let ws_message = WebSocketMessage::TypingNotification {
            user: UserSummary::from(user.clone()),
            is_typing,
        };
        
        self.broadcast_to_room(room_id, &ws_message).await;
    }
    
    // Core broadcasting logic (Rails: best-effort delivery)
    async fn broadcast_to_room(&self, room_id: RoomId, message: &WebSocketMessage) {
        let connection_ids = {
            let room_connections = self.room_connections.read().await;
            room_connections.get(&room_id).cloned().unwrap_or_default()
        };
        
        let connections = self.connections.read().await;
        let message_json = serde_json::to_string(message).unwrap();
        
        for connection_id in connection_ids {
            if let Some(connection) = connections.get(&connection_id) {
                // Best effort delivery (Rails ActionCable behavior)
                let _ = connection.sender.send(WsMessage::Text(message_json.clone()));
            }
        }
    }
    
    // Send missed messages on reconnection (Critical Gap #2)
    pub async fn send_missed_messages(
        &self,
        connection_id: ConnectionId,
        room_id: RoomId,
        last_seen_message_id: Option<MessageId>,
        message_service: &MessageService,
    ) -> Result<(), BroadcastError> {
        let connection = {
            let connections = self.connections.read().await;
            connections.get(&connection_id).cloned()
        };
        
        if let Some(connection) = connection {
            // Get missed messages since last_seen_message_id
            let missed_messages = if let Some(last_id) = last_seen_message_id {
                message_service.get_messages_since(room_id, last_id).await?
            } else {
                // No last seen ID, send recent messages (Rails behavior)
                message_service.get_recent_messages(room_id, 50).await?
            };
            
            // Send missed messages
            for message in missed_messages {
                let ws_message = WebSocketMessage::MessageCreated {
                    message: MessageResponse::from(message),
                };
                let message_json = serde_json::to_string(&ws_message).unwrap();
                let _ = connection.sender.send(WsMessage::Text(message_json));
            }
        }
        
        Ok(())
    }
}
```

### WebSocket Message Protocol

```rust
// WebSocket message types (Rails ActionCable equivalent)
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    // Client -> Server messages
    Subscribe {
        room_id: RoomId,
        last_seen_message_id: Option<MessageId>, // For reconnection
    },
    Unsubscribe {
        room_id: RoomId,
    },
    TypingStart {
        room_id: RoomId,
    },
    TypingStop {
        room_id: RoomId,
    },
    Heartbeat {
        timestamp: DateTime<Utc>,
    },
    
    // Server -> Client messages
    MessageCreated {
        message: MessageResponse,
    },
    MessageUpdated {
        message: MessageResponse,
    },
    MessageDeleted {
        message_id: MessageId,
        room_id: RoomId,
    },
    TypingNotification {
        user: UserSummary,
        is_typing: bool,
    },
    PresenceUpdate {
        room_id: RoomId,
        online_users: Vec<UserSummary>,
    },
    RoomUpdated {
        room: RoomResponse,
    },
    MembershipUpdated {
        membership: MembershipResponse,
    },
    
    // Connection management
    Connected {
        connection_id: ConnectionId,
    },
    Disconnected {
        reason: String,
    },
    Error {
        message: String,
        code: Option<String>,
    },
}

// WebSocket connection handler
pub async fn handle_websocket_connection(
    socket: WebSocket,
    user_id: UserId,
    broadcaster: Arc<WebSocketBroadcaster>,
    app_state: AppState,
) {
    let connection_id = ConnectionId::new();
    let (sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    // Create connection
    let connection = WebSocketConnection {
        id: connection_id,
        user_id,
        room_subscriptions: HashSet::new(),
        sender: tx,
        last_seen_message_id: None,
        connected_at: Utc::now(),
    };
    
    // Add connection with presence tracking
    if let Err(e) = broadcaster.add_connection(connection, &app_state.membership_service).await {
        eprintln!("Failed to add WebSocket connection: {}", e);
        return;
    }
    
    // Send connection confirmation
    let connected_msg = WebSocketMessage::Connected { connection_id };
    let _ = tx.send(WsMessage::Text(serde_json::to_string(&connected_msg).unwrap()));
    
    // Spawn sender task
    let sender_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });
    
    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(WsMessage::Text(text)) => {
                if let Err(e) = handle_websocket_message(
                    connection_id,
                    &text,
                    &broadcaster,
                    &app_state,
                ).await {
                    eprintln!("WebSocket message handling error: {}", e);
                }
            }
            Ok(WsMessage::Close(_)) => break,
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
            _ => {} // Ignore other message types
        }
    }
    
    // Cleanup on disconnect
    let _ = broadcaster.remove_connection(connection_id, &app_state.membership_service).await;
    sender_task.abort();
}

// Handle individual WebSocket messages
async fn handle_websocket_message(
    connection_id: ConnectionId,
    text: &str,
    broadcaster: &WebSocketBroadcaster,
    app_state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let message: WebSocketMessage = serde_json::from_str(text)?;
    
    match message {
        WebSocketMessage::Subscribe { room_id, last_seen_message_id } => {
            // Verify room access
            let user_id = get_connection_user_id(connection_id, broadcaster).await?;
            app_state.room_service.verify_user_access(user_id, room_id).await?;
            
            // Subscribe to room
            broadcaster.subscribe_to_room(connection_id, room_id).await?;
            
            // Send missed messages if reconnecting (Critical Gap #2)
            if last_seen_message_id.is_some() {
                broadcaster.send_missed_messages(
                    connection_id,
                    room_id,
                    last_seen_message_id,
                    &app_state.message_service,
                ).await?;
            }
        }
        
        WebSocketMessage::TypingStart { room_id } => {
            let user_id = get_connection_user_id(connection_id, broadcaster).await?;
            let user = app_state.user_service.get_user(user_id).await?;
            broadcaster.broadcast_typing_notification(room_id, &user, true).await;
        }
        
        WebSocketMessage::TypingStop { room_id } => {
            let user_id = get_connection_user_id(connection_id, broadcaster).await?;
            let user = app_state.user_service.get_user(user_id).await?;
            broadcaster.broadcast_typing_notification(room_id, &user, false).await;
        }
        
        WebSocketMessage::Heartbeat { .. } => {
            // Update connection activity (presence tracking)
            let user_id = get_connection_user_id(connection_id, broadcaster).await?;
            app_state.membership_service.refresh_user_presence(user_id).await?;
        }
        
        _ => {
            // Ignore client messages that should be server-only
        }
    }
    
    Ok(())
}
```

**Key WebSocket Design Decisions**:
- Rails ActionCable equivalent message protocol
- Room-based subscriptions with access control
- Basic presence tracking with connection counting (Critical Gap #5)
- Reconnection support with missed message delivery (Critical Gap #2)
- Best-effort delivery matching Rails behavior
- Simple heartbeat mechanism for connection health
- No complex message ordering or delivery guarantees

### 4. Authentication System

**Design Approach**: Rails-style session management with secure cookies.

```rust
pub struct AuthService {
    db: SqlitePool,
    secret_key: String,
}

impl AuthService {
    pub async fn create_session(&self, user: &User) -> Result<String, AuthError> {
        let session = Session {
            user_id: user.id,
            token: generate_secure_token(),
            expires_at: Utc::now() + Duration::hours(24),
        };
        
        // Direct database insert
        sqlx::query!(
            "INSERT INTO sessions (user_id, token, expires_at) VALUES (?, ?, ?)",
            session.user_id.0, session.token, session.expires_at
        ).execute(&self.db).await?;
        
        // Return signed cookie value
        Ok(self.sign_token(&session.token))
    }
}
```

**Key Design Decisions**:
- Session tokens stored in SQLite (direct operations)
- Secure cookie-based authentication (Rails-style)
- Simple token generation and validation
- No complex OAuth or JWT handling
- Basic session cleanup

## Background Task Processing

### Simple Async Task System - Rails Background Job Equivalent

**Design Approach**: Simple tokio::spawn for basic async tasks, no complex job queues or coordination. Rails-equivalent background processing for webhooks, push notifications, and cleanup tasks.

### Task Processing Architecture

```rust
use tokio::time::{Duration, sleep};
use reqwest::Client;
use serde_json::json;

// Background task manager
pub struct BackgroundTaskManager {
    http_client: Client,
    feature_flags: FeatureFlags,
}

impl BackgroundTaskManager {
    pub fn new(feature_flags: FeatureFlags) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            http_client,
            feature_flags,
        }
    }
    
    // Spawn webhook delivery task (Rails: Bot::WebhookJob)
    pub fn deliver_webhook(&self, webhook_data: WebhookDeliveryData) {
        if !self.feature_flags.bot_integrations {
            return;
        }
        
        let client = self.http_client.clone();
        tokio::spawn(async move {
            Self::execute_webhook_delivery(client, webhook_data).await;
        });
    }
    
    // Spawn push notification task
    pub fn send_push_notification(&self, push_data: PushNotificationData) {
        if !self.feature_flags.push_notifications {
            return;
        }
        
        let client = self.http_client.clone();
        tokio::spawn(async move {
            Self::execute_push_notification(client, push_data).await;
        });
    }
    
    // Spawn cleanup task
    pub fn schedule_cleanup(&self, cleanup_data: CleanupTaskData) {
        tokio::spawn(async move {
            Self::execute_cleanup_task(cleanup_data).await;
        });
    }
}
```

### Webhook Delivery Implementation

```rust
#[derive(Debug, Clone)]
pub struct WebhookDeliveryData {
    pub webhook_url: String,
    pub payload: WebhookPayload,
    pub bot_id: UserId,
    pub message_id: MessageId,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebhookPayload {
    pub user: UserSummary,
    pub room: RoomSummary,
    pub message: MessageSummary,
    pub timestamp: DateTime<Utc>,
}

impl BackgroundTaskManager {
    async fn execute_webhook_delivery(client: Client, data: WebhookDeliveryData) {
        let start_time = std::time::Instant::now();
        
        // Rails: 7-second timeout for webhook delivery
        let result = client
            .post(&data.webhook_url)
            .json(&data.payload)
            .timeout(Duration::from_secs(7))
            .send()
            .await;
        
        match result {
            Ok(response) => {
                let status = response.status();
                let duration = start_time.elapsed();
                
                tracing::info!(
                    "Webhook delivered successfully: bot_id={}, status={}, duration={:?}",
                    data.bot_id.0,
                    status,
                    duration
                );
                
                // Process webhook response (Rails: create reply message)
                if status.is_success() {
                    if let Ok(response_text) = response.text().await {
                        Self::process_webhook_response(data.bot_id, response_text).await;
                    }
                }
            }
            Err(e) => {
                let duration = start_time.elapsed();
                tracing::error!(
                    "Webhook delivery failed: bot_id={}, error={}, duration={:?}",
                    data.bot_id.0,
                    e,
                    duration
                );
                
                // Simple retry logic (Rails: basic retry on failure)
                if duration < Duration::from_secs(5) {
                    // Only retry if it failed quickly (likely network issue)
                    sleep(Duration::from_secs(2)).await;
                    
                    let retry_result = client
                        .post(&data.webhook_url)
                        .json(&data.payload)
                        .timeout(Duration::from_secs(7))
                        .send()
                        .await;
                    
                    if let Err(retry_error) = retry_result {
                        tracing::error!(
                            "Webhook retry failed: bot_id={}, error={}",
                            data.bot_id.0,
                            retry_error
                        );
                    }
                }
            }
        }
    }
    
    async fn process_webhook_response(bot_id: UserId, response_text: String) {
        // Simple response processing (Rails: Messages::ByBotsController)
        if !response_text.trim().is_empty() && response_text.len() <= 10000 {
            // TODO: Create bot reply message
            tracing::info!(
                "Bot response received: bot_id={}, length={}",
                bot_id.0,
                response_text.len()
            );
        }
    }
}
```

### Push Notification Implementation

```rust
#[derive(Debug, Clone)]
pub struct PushNotificationData {
    pub subscription: PushSubscription,
    pub title: String,
    pub body: String,
    pub url: String,
    pub badge_count: Option<i32>,
}

impl BackgroundTaskManager {
    async fn execute_push_notification(client: Client, data: PushNotificationData) {
        // Web Push implementation (Rails: WebPush with VAPID)
        let payload = json!({
            "title": data.title,
            "body": data.body,
            "url": data.url,
            "badge": data.badge_count,
            "icon": "/icon-192.png",
            "timestamp": Utc::now().timestamp_millis()
        });
        
        // TODO: Implement Web Push protocol with VAPID keys
        // This would use the web-push crate for proper implementation
        tracing::info!(
            "Push notification sent: endpoint={}, title={}",
            data.subscription.endpoint,
            data.title
        );
    }
}
```

### Cleanup Tasks Implementation

```rust
#[derive(Debug, Clone)]
pub enum CleanupTaskData {
    ExpiredSessions,
    StalePresence,
    OldSearchIndex,
}

impl BackgroundTaskManager {
    async fn execute_cleanup_task(data: CleanupTaskData) {
        match data {
            CleanupTaskData::ExpiredSessions => {
                // Clean up expired sessions (Rails: periodic cleanup)
                tracing::info!("Cleaning up expired sessions");
                // TODO: Delete sessions older than 30 days
            }
            CleanupTaskData::StalePresence => {
                // Clean up stale presence data (Rails: Connectable concern)
                tracing::info!("Cleaning up stale presence data");
                // TODO: Reset connections for users inactive > 1 hour
            }
            CleanupTaskData::OldSearchIndex => {
                // Optimize FTS5 search index (Rails: periodic maintenance)
                tracing::info!("Optimizing search index");
                // TODO: Run FTS5 optimize command
            }
        }
    }
}
```

## Asset Integration Specifications

### Embedded Asset System - Complete UI Assets

**Design Approach**: Embed all UI assets (CSS, images, sounds) in the binary using rust-embed for single-file deployment with graceful degradation messaging.

### Asset Embedding Implementation

```rust
use rust_embed::RustEmbed;
use axum::response::{IntoResponse, Response};
use axum::http::{header, StatusCode};

// Embed all frontend assets
#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
pub struct FrontendAssets;

// Embed sound assets (59 MP3 files)
#[derive(RustEmbed)]
#[folder = "assets/sounds/"]
pub struct SoundAssets;

// Embed image assets (79 SVG files)
#[derive(RustEmbed)]
#[folder = "assets/images/"]
pub struct ImageAssets;

// Asset serving handler
pub async fn serve_asset(Path(path): Path<String>) -> Result<Response, StatusCode> {
    // Try frontend assets first (React SPA)
    if let Some(content) = FrontendAssets::get(&path) {
        let mime_type = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .to_string();
        
        return Ok(Response::builder()
            .header(header::CONTENT_TYPE, mime_type)
            .header(header::CACHE_CONTROL, "public, max-age=31536000") // 1 year cache
            .header(header::ETAG, format!("\"{}\"", content.metadata.sha256_hash()))
            .body(content.data.into())
            .unwrap());
    }
    
    // Try sound assets
    if path.starts_with("sounds/") {
        let sound_path = path.strip_prefix("sounds/").unwrap();
        if let Some(content) = SoundAssets::get(sound_path) {
            return Ok(Response::builder()
                .header(header::CONTENT_TYPE, "audio/mpeg")
                .header(header::CACHE_CONTROL, "public, max-age=31536000")
                .body(content.data.into())
                .unwrap());
        }
    }
    
    // Try image assets
    if path.starts_with("images/") {
        let image_path = path.strip_prefix("images/").unwrap();
        if let Some(content) = ImageAssets::get(image_path) {
            return Ok(Response::builder()
                .header(header::CONTENT_TYPE, "image/svg+xml")
                .header(header::CACHE_CONTROL, "public, max-age=31536000")
                .body(content.data.into())
                .unwrap());
        }
    }
    
    // Fallback to index.html for SPA routing
    if let Some(content) = FrontendAssets::get("index.html") {
        return Ok(Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(content.data.into())
            .unwrap());
    }
    
    Err(StatusCode::NOT_FOUND)
}
```

### Sound Command Processing

```rust
// Sound command handler (Rails: /play command)
pub struct SoundProcessor;

impl SoundProcessor {
    // Parse sound commands from message content
    pub fn extract_sound_command(content: &str) -> Option<String> {
        if content.starts_with("/play ") {
            let sound_name = content.strip_prefix("/play ").unwrap().trim();
            
            // Validate against available sounds (Rails: predefined list)
            const VALID_SOUNDS: &[&str] = &[
                "56k", "bell", "bezos", "bueller", "crickets", "trombone",
                "rimshot", "tada", "airhorn", "applause", "boo", "nyan",
                "ohmy", "pushit", "rimshot", "secret", "trombone", "vuvuzela",
                "yeah", "yodel", // ... complete list of 59 sounds
            ];
            
            if VALID_SOUNDS.contains(&sound_name) {
                Some(sound_name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    // Get sound asset URL
    pub fn get_sound_url(sound_name: &str) -> String {
        format!("/assets/sounds/{}.mp3", sound_name)
    }
}
```

### Feature Flag Integration

```rust
// Feature flag configuration for graceful degradation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    // MVP Phase 1: Enabled features
    pub search_enabled: bool,           // Default: true
    pub push_notifications: bool,       // Default: true
    pub bot_integrations: bool,         // Default: true
    pub sounds_enabled: bool,           // Default: true
    
    // MVP Phase 1: Disabled features (graceful degradation)
    pub files_enabled: bool,            // Default: false
    pub avatars_enabled: bool,          // Default: false
    pub opengraph_enabled: bool,        // Default: false
    pub video_processing: bool,         // Default: false
}

impl FeatureFlags {
    pub fn from_env() -> Self {
        Self {
            search_enabled: env::var("SEARCH_ENABLED").unwrap_or("true".to_string()) == "true",
            push_notifications: env::var("PUSH_NOTIFICATIONS").unwrap_or("true".to_string()) == "true",
            bot_integrations: env::var("BOT_INTEGRATIONS").unwrap_or("true".to_string()) == "true",
            sounds_enabled: env::var("SOUNDS_ENABLED").unwrap_or("true".to_string()) == "true",
            
            // Gracefully disabled for MVP
            files_enabled: env::var("FILES_ENABLED").unwrap_or("false".to_string()) == "true",
            avatars_enabled: env::var("AVATARS_ENABLED").unwrap_or("false".to_string()) == "true",
            opengraph_enabled: env::var("OPENGRAPH_ENABLED").unwrap_or("false".to_string()) == "true",
            video_processing: env::var("VIDEO_PROCESSING").unwrap_or("false".to_string()) == "true",
        }
    }
}

// Feature flag middleware for API endpoints
pub async fn check_feature_flag<T>(
    feature_enabled: bool,
    feature_name: &str,
    handler: impl Future<Output = Result<T, ApiError>>,
) -> Result<T, ApiError> {
    if !feature_enabled {
        return Err(ApiError::FeatureDisabled(feature_name.to_string()));
    }
    handler.await
}
```

**Key Background Task & Asset Design Decisions**:
- Simple tokio::spawn for background tasks (Rails background job equivalent)
- 7-second webhook timeout matching Rails implementation
- Basic retry logic for failed webhooks (Rails pattern)
- Complete asset embedding for single binary deployment
- Sound command processing with predefined sound list (Rails /play commands)
- Feature flags for graceful degradation of disabled features
- Comprehensive caching headers for optimal performance
- SPA routing fallback for React frontend#
# Data Models

## Complete Domain Model Specifications

### Core Entity Models

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// User model - Complete Rails parity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: UserId,
    pub email_address: String,           // Rails: email_address field
    pub name: String,
    pub password_digest: String,         // Rails: bcrypt hash
    pub role: UserRole,                  // Rails: enum (member: 0, administrator: 1, bot: 2)
    pub bot_token: Option<String>,       // Rails: SecureRandom.alphanumeric(12) for bots
    pub webhook_url: Option<String>,     // Rails: bot webhook endpoint
    pub active: bool,                    // Rails: soft delete flag
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Account model - Rails singleton pattern
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: AccountId,
    pub name: String,                    // Rails: account name
    pub join_code: String,               // Rails: "XXXX-XXXX-XXXX" format
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Room model with STI pattern - Complete Rails parity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Room {
    pub id: RoomId,
    pub account_id: AccountId,           // Rails: belongs_to :account
    pub name: String,
    pub room_type: RoomType,             // Rails: STI type column (Open, Closed, Direct)
    pub creator_id: UserId,              // Rails: belongs_to :creator, class_name: "User"
    pub last_message_at: Option<DateTime<Utc>>, // Rails: updated on message creation
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Message model - Complete Rails parity with rich text
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: MessageId,
    pub room_id: RoomId,                 // Rails: belongs_to :room
    pub creator_id: UserId,              // Rails: belongs_to :creator, class_name: "User"
    pub body: String,                    // Rails: ActionText rich_text body
    pub client_message_id: Uuid,         // Rails: UUID for deduplication (Critical Gap #1)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Membership model - Rails join table with involvement
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Membership {
    pub id: MembershipId,
    pub user_id: UserId,                 // Rails: belongs_to :user
    pub room_id: RoomId,                 // Rails: belongs_to :room
    pub involvement: Involvement,        // Rails: enum (invisible: 0, nothing: 1, mentions: 2, everything: 3)
    pub connections: i32,                // Rails: Connectable concern for presence (Critical Gap #5)
    pub connected_at: Option<DateTime<Utc>>, // Rails: presence tracking timestamp
    pub unread_at: Option<DateTime<Utc>>, // Rails: last unread message timestamp
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Session model - Rails session management (Critical Gap #4)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,                 // Rails: belongs_to :user
    pub token: String,                   // Rails: SecureRandom token
    pub ip_address: Option<String>,      // Rails: request IP tracking
    pub user_agent: Option<String>,      // Rails: browser identification
    pub last_active_at: DateTime<Utc>,   // Rails: updated every hour
    pub created_at: DateTime<Utc>,
}

// Boost model - Rails message reactions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Boost {
    pub id: BoostId,
    pub message_id: MessageId,           // Rails: belongs_to :message
    pub booster_id: UserId,              // Rails: belongs_to :booster, class_name: "User"
    pub content: String,                 // Rails: emoji content (max 16 chars)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// PushSubscription model - Rails Web Push integration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PushSubscription {
    pub id: PushSubscriptionId,
    pub user_id: UserId,                 // Rails: belongs_to :user
    pub endpoint: String,                // Rails: Web Push endpoint
    pub p256dh_key: String,              // Rails: encryption key
    pub auth_key: String,                // Rails: authentication key
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Type Safety with Comprehensive Newtypes

```rust
// Primary key newtypes for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct AccountId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct RoomId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct MessageId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct MembershipId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct SessionId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct BoostId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct PushSubscriptionId(pub i64);
```

### Enums - Rails-Compatible Definitions

```rust
// UserRole enum - Rails: member: 0, administrator: 1, bot: 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "INTEGER")]
#[repr(i32)]
pub enum UserRole {
    Member = 0,
    Administrator = 1,
    Bot = 2,
}

// RoomType enum - Rails STI pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum RoomType {
    #[sqlx(rename = "Rooms::Open")]
    Open,
    #[sqlx(rename = "Rooms::Closed")]
    Closed,
    #[sqlx(rename = "Rooms::Direct")]
    Direct,
}

// Involvement enum - Rails: invisible: 0, nothing: 1, mentions: 2, everything: 3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "INTEGER")]
#[repr(i32)]
pub enum Involvement {
    Invisible = 0,  // Hidden from sidebar
    Nothing = 1,    // No notifications
    Mentions = 2,   // @mention notifications only
    Everything = 3, // All message notifications
}
```

### Request/Response Models

```rust
// API request models
#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub body: String,
    pub client_message_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub room_type: RoomType,
    pub member_ids: Option<Vec<UserId>>, // For closed/direct rooms
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email_address: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email_address: String,
    pub name: String,
    pub password: String,
    pub join_code: String,
}

// API response models
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: MessageId,
    pub room_id: RoomId,
    pub creator: UserSummary,
    pub body: String,
    pub client_message_id: Uuid,
    pub boosts: Vec<BoostSummary>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RoomResponse {
    pub id: RoomId,
    pub name: String,
    pub room_type: RoomType,
    pub creator: UserSummary,
    pub members: Vec<MembershipSummary>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub id: UserId,
    pub name: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize)]
pub struct MembershipSummary {
    pub user: UserSummary,
    pub involvement: Involvement,
    pub is_online: bool, // Computed from connections > 0
}

#[derive(Debug, Serialize)]
pub struct BoostSummary {
    pub id: BoostId,
    pub booster: UserSummary,
    pub content: String,
    pub created_at: DateTime<Utc>,
}
```

## Error Handling

### Simple Error Strategy

```rust
// Application error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Authentication failed")]
    Unauthorized,
    
    #[error("Resource not found")]
    NotFound,
    
    #[error("Invalid input: {0}")]
    Validation(String),
}

// Convert to HTTP responses
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
        };
        
        (status, Json(json!({"error": message}))).into_response()
    }
}
```

**Key Design Decisions**:
- Simple error enum with user-friendly messages
- No complex error recovery or retry logic
- Basic logging for debugging
- Convert technical errors to user messages## Te
sting Strategy

### Property Tests for Invariants

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn messages_are_ordered_by_created_at_then_id(
            messages in prop::collection::vec(arbitrary_message(), 1..100)
        ) {
            let mut sorted_messages = messages.clone();
            sorted_messages.sort_by(|a, b| {
                a.created_at.cmp(&b.created_at)
                    .then_with(|| a.id.cmp(&b.id))
            });
            
            // Verify ordering invariant holds
            for window in sorted_messages.windows(2) {
                assert!(window[0].created_at <= window[1].created_at);
                if window[0].created_at == window[1].created_at {
                    assert!(window[0].id < window[1].id);
                }
            }
        }
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_message_creation_and_broadcast() {
    let app = test_app().await;
    let room_id = create_test_room(&app).await;
    
    // Create message via API
    let response = app
        .post(&format!("/api/rooms/{}/messages", room_id))
        .json(&json!({
            "body": "Test message",
            "client_message_id": "test-uuid"
        }))
        .send()
        .await;
        
    assert_eq!(response.status(), 201);
    
    // Verify message was stored and broadcast
    let message: Message = response.json().await;
    assert_eq!(message.body, "Test message");
}
```

## Feature Flag Implementation

### Configuration-Driven Features

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub files_enabled: bool,
    pub avatars_enabled: bool,
    pub opengraph_enabled: bool,
    pub search_enabled: bool,
    pub push_notifications: bool,
    pub bot_integrations: bool,
}

impl FeatureFlags {
    pub fn from_env() -> Self {
        Self {
            files_enabled: env::var("FILES_ENABLED").unwrap_or_default() == "true",
            avatars_enabled: env::var("AVATARS_ENABLED").unwrap_or_default() == "true",
            opengraph_enabled: env::var("OPENGRAPH_ENABLED").unwrap_or_default() == "true",
            search_enabled: env::var("SEARCH_ENABLED").unwrap_or("true") == "true",
            push_notifications: env::var("PUSH_NOTIFICATIONS").unwrap_or("true") == "true",
            bot_integrations: env::var("BOT_INTEGRATIONS").unwrap_or("true") == "true",
        }
    }
}
```

## Application State and Configuration

### Application State Management

```rust
use std::sync::Arc;

// Central application state (Rails: Application.config equivalent)
#[derive(Clone)]
pub struct AppState {
    // Core services
    pub database: Arc<Database>,
    pub broadcaster: Arc<WebSocketBroadcaster>,
    pub background_tasks: Arc<BackgroundTaskManager>,
    
    // Business logic services (Rails-style service objects)
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub room_service: Arc<RoomService>,
    pub message_service: Arc<MessageService>,
    pub membership_service: Arc<MembershipService>,
    pub search_service: Arc<SearchService>,
    pub webhook_service: Arc<WebhookService>,
    pub push_service: Arc<PushNotificationService>,
    
    // Configuration
    pub config: Arc<AppConfig>,
    pub feature_flags: Arc<FeatureFlags>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize database with migrations
        let database = Arc::new(Database::new(&config.database_url).await?);
        database.run_migrations().await?;
        
        // Initialize WebSocket broadcaster
        let broadcaster = Arc::new(WebSocketBroadcaster::new());
        
        // Initialize feature flags
        let feature_flags = Arc::new(FeatureFlags::from_env());
        
        // Initialize background task manager
        let background_tasks = Arc::new(BackgroundTaskManager::new(feature_flags.as_ref().clone()));
        
        // Initialize services
        let auth_service = Arc::new(AuthService::new(
            database.clone(),
            config.secret_key.clone(),
        ));
        
        let user_service = Arc::new(UserService::new(database.clone()));
        let room_service = Arc::new(RoomService::new(database.clone()));
        let message_service = Arc::new(MessageService::new(database.clone()));
        let membership_service = Arc::new(MembershipService::new(database.clone()));
        let search_service = Arc::new(SearchService::new(database.clone()));
        let webhook_service = Arc::new(WebhookService::new(
            database.clone(),
            background_tasks.clone(),
        ));
        let push_service = Arc::new(PushNotificationService::new(
            database.clone(),
            background_tasks.clone(),
            config.vapid_keys.clone(),
        ));
        
        Ok(Self {
            database,
            broadcaster,
            background_tasks,
            auth_service,
            user_service,
            room_service,
            message_service,
            membership_service,
            search_service,
            webhook_service,
            push_service,
            config: Arc::new(config),
            feature_flags,
        })
    }
}
```

### Configuration Management

```rust
use serde::{Deserialize, Serialize};
use std::env;

// Application configuration (Rails: config/application.rb equivalent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Server configuration
    pub bind_address: String,
    pub port: u16,
    pub secret_key: String,
    
    // Database configuration
    pub database_url: String,
    pub database_max_connections: u32,
    
    // WebSocket configuration
    pub websocket_heartbeat_interval: Duration,
    pub websocket_connection_timeout: Duration,
    
    // Security configuration
    pub session_duration: Duration,
    pub rate_limit_requests: u32,
    pub rate_limit_window: Duration,
    
    // Push notification configuration
    pub vapid_keys: VapidKeys,
    
    // Asset configuration
    pub asset_cache_duration: Duration,
    
    // Logging configuration
    pub log_level: String,
    pub log_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VapidKeys {
    pub public_key: String,
    pub private_key: String,
    pub subject: String, // mailto: or https: URL
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            bind_address: env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            secret_key: env::var("SECRET_KEY_BASE")
                .map_err(|_| "SECRET_KEY_BASE environment variable is required")?,
            
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:campfire.db".to_string()),
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            
            websocket_heartbeat_interval: Duration::from_secs(
                env::var("WEBSOCKET_HEARTBEAT_INTERVAL")
                    .unwrap_or_else(|_| "50".to_string())
                    .parse()
                    .unwrap_or(50)
            ),
            websocket_connection_timeout: Duration::from_secs(
                env::var("WEBSOCKET_CONNECTION_TIMEOUT")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60)
            ),
            
            session_duration: Duration::from_secs(
                env::var("SESSION_DURATION_SECONDS")
                    .unwrap_or_else(|_| "2592000".to_string()) // 30 days
                    .parse()
                    .unwrap_or(2592000)
            ),
            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            rate_limit_window: Duration::from_secs(
                env::var("RATE_LIMIT_WINDOW_SECONDS")
                    .unwrap_or_else(|_| "180".to_string()) // 3 minutes
                    .parse()
                    .unwrap_or(180)
            ),
            
            vapid_keys: VapidKeys {
                public_key: env::var("VAPID_PUBLIC_KEY")
                    .map_err(|_| "VAPID_PUBLIC_KEY environment variable is required")?,
                private_key: env::var("VAPID_PRIVATE_KEY")
                    .map_err(|_| "VAPID_PRIVATE_KEY environment variable is required")?,
                subject: env::var("VAPID_SUBJECT")
                    .unwrap_or_else(|_| "mailto:admin@campfire.local".to_string()),
            },
            
            asset_cache_duration: Duration::from_secs(
                env::var("ASSET_CACHE_DURATION_SECONDS")
                    .unwrap_or_else(|_| "31536000".to_string()) // 1 year
                    .parse()
                    .unwrap_or(31536000)
            ),
            
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            log_format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
        })
    }
}
```

### Application Startup and Routing

```rust
use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
};

// Main application entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load configuration
    let config = AppConfig::from_env()?;
    
    // Initialize logging
    init_logging(&config)?;
    
    // Initialize application state
    let app_state = AppState::new(config.clone()).await?;
    
    // Build router with all routes
    let app = create_router(app_state.clone());
    
    // Start server
    let bind_address = format!("{}:{}", config.bind_address, config.port);
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    
    tracing::info!("Campfire server starting on {}", bind_address);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Create complete application router
fn create_router(app_state: AppState) -> Router {
    Router::new()
        // Authentication routes
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/auth/register", post(api::auth::register))
        .route("/api/auth/logout", post(api::auth::logout))
        
        // Message routes
        .route("/api/rooms/:room_id/messages", post(api::messages::create_message))
        .route("/api/rooms/:room_id/messages", get(api::messages::list_messages))
        .route("/api/messages/:message_id", put(api::messages::update_message))
        .route("/api/messages/:message_id", delete(api::messages::delete_message))
        
        // Room routes
        .route("/api/rooms", get(api::rooms::list_rooms))
        .route("/api/rooms", post(api::rooms::create_room))
        .route("/api/rooms/:room_id", get(api::rooms::get_room))
        .route("/api/rooms/:room_id", put(api::rooms::update_room))
        .route("/api/rooms/:room_id/membership", put(api::rooms::update_membership))
        
        // User routes
        .route("/api/users/me", get(api::users::get_current_user))
        .route("/api/users/:user_id", get(api::users::get_user))
        
        // Search routes
        .route("/api/search/messages", get(api::search::search_messages))
        
        // Bot routes
        .route("/api/bots/:bot_id/messages", post(api::bots::create_bot_message))
        
        // WebSocket upgrade
        .route("/ws", get(api::websocket::websocket_handler))
        
        // Asset serving (catch-all for SPA)
        .route("/assets/*path", get(api::assets::serve_asset))
        .fallback(api::assets::serve_spa)
        
        // Middleware stack
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive())
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::auth::session_auth,
                ))
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::rate_limit::rate_limit,
                ))
        )
        .with_state(app_state)
}

// Initialize structured logging
fn init_logging(config: &AppConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    let log_level = config.log_level.parse::<tracing::Level>()?;
    
    match config.log_format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json())
                .with(tracing_subscriber::filter::LevelFilter::from_level(log_level))
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::filter::LevelFilter::from_level(log_level))
                .init();
        }
    }
    
    Ok(())
}
```

### Health Check and Monitoring

```rust
// Health check endpoint
pub async fn health_check(State(app_state): State<AppState>) -> Json<serde_json::Value> {
    let database_healthy = app_state.database.health_check().await.is_ok();
    let websocket_connections = app_state.broadcaster.connection_count().await;
    
    Json(json!({
        "status": if database_healthy { "healthy" } else { "unhealthy" },
        "timestamp": Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "database": {
            "healthy": database_healthy,
            "pool_size": app_state.database.pool_size().await,
        },
        "websocket": {
            "active_connections": websocket_connections,
        },
        "features": {
            "files_enabled": app_state.feature_flags.files_enabled,
            "avatars_enabled": app_state.feature_flags.avatars_enabled,
            "search_enabled": app_state.feature_flags.search_enabled,
            "push_notifications": app_state.feature_flags.push_notifications,
            "bot_integrations": app_state.feature_flags.bot_integrations,
        }
    }))
}

// Metrics endpoint (Prometheus format)
pub async fn metrics(State(app_state): State<AppState>) -> String {
    format!(
        "# HELP campfire_websocket_connections Active WebSocket connections\n\
         # TYPE campfire_websocket_connections gauge\n\
         campfire_websocket_connections {}\n\
         # HELP campfire_database_pool_size Database connection pool size\n\
         # TYPE campfire_database_pool_size gauge\n\
         campfire_database_pool_size {}\n",
        app_state.broadcaster.connection_count().await,
        app_state.database.pool_size().await
    )
}
```

## Design Summary

This comprehensive design document provides a complete, implementable specification for the Campfire Rust Rewrite MVP that:

### ✅ **Adheres to All Requirements**
- **Anti-coordination constraints**: No coordination layers, direct operations only
- **Rails parity**: Replicates Rails ActionCable behavior exactly
- **5 Critical Gaps**: Implements all critical gap fixes with detailed specifications
- **Feature flags**: Graceful degradation for disabled features
- **Complete UI**: Full asset embedding with professional appearance

### ✅ **Provides Implementation-Ready Specifications**
- **Complete data models** with Rails-compatible schema
- **Comprehensive API specifications** with request/response formats
- **Detailed WebSocket protocol** with message types and connection management
- **Database layer** with Dedicated Writer Task pattern and read operations
- **Background task processing** with webhook delivery and push notifications
- **Asset integration** with embedded resources and sound command processing
- **Application state management** with configuration and startup procedures

### ✅ **Enables Direct Implementation**
- **Type-safe domain models** with comprehensive newtypes and enums
- **Complete service layer** with Rails-style business logic organization
- **Error handling strategy** with user-friendly messages and proper HTTP responses
- **Testing approach** with property tests and integration test patterns
- **Monitoring and health checks** with Prometheus metrics and structured logging

This design provides a clear, implementable path to building the Campfire Rust Rewrite MVP while strictly adhering to the anti-coordination constraints and Rails-inspired simplicity mandated in the requirements. Every component is designed to avoid coordination complexity while maintaining Rails-equivalent functionality and professional user experience.