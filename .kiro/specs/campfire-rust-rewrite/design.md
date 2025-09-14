# Campfire Rust Rewrite - Design Document

## Overview: TDD-Driven Type Contracts

This document defines the complete type contracts, function signatures, and error hierarchies for the Campfire Rust rewrite. Following the improved LLM workflow, we establish all interfaces before any implementation to ensure compile-first success and architectural correctness.

**Design Philosophy:**
- **Type Contracts First**: Complete function signatures with all error cases defined upfront
- **Rails Parity**: Every interface mirrors Rails behavior exactly, no improvements
- **Anti-Coordination**: Direct function calls, no async coordination between components
- **Phantom Types**: Use type system to prevent invalid state transitions

## Core Domain Types

### Newtype IDs (Compile-Time Safety)

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// User identifier - prevents mixing up with other ID types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i64);

/// Room identifier - type-safe room references
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub i64);

/// Message identifier - prevents ID confusion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub i64);

/// Session identifier - secure session tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub i64);

/// WebSocket connection identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(pub i64);

/// Bot token for API authentication
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BotToken(pub String);
```

### Domain Models with Complete Field Specifications

```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// User model - matches Rails User schema exactly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email_address: String,
    pub name: String,
    pub password_digest: String,  // bcrypt hash
    pub role: UserRole,
    pub active: bool,
    pub bot_token: Option<BotToken>,
    pub webhook_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User roles - matches Rails enum exactly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Member = 0,
    Administrator = 1,
    Bot = 2,
}

/// Room model - supports Rails STI pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub room_type: RoomType,
    pub creator_id: UserId,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Room types - Rails Single Table Inheritance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomType {
    Open,
    Closed { invited_users: Vec<UserId> },
    Direct { participants: [UserId; 2] },  // Exactly 2 participants
}

/// Message model with phantom types for state safety
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<State = Persisted> {
    pub id: MessageId,
    pub client_message_id: Uuid,  // Critical Gap #1: deduplication
    pub content: String,
    pub room_id: RoomId,
    pub creator_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    _state: std::marker::PhantomData<State>,
}

/// Message states for type safety
pub struct Draft;
pub struct Validated;
pub struct Persisted;

/// Room membership with involvement levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Membership {
    pub id: i64,
    pub user_id: UserId,
    pub room_id: RoomId,
    pub involvement: Involvement,
    pub connections: i32,  // Critical Gap #5: presence tracking
    pub connected_at: Option<DateTime<Utc>>,
    pub unread_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Involvement levels - Rails enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Involvement {
    Invisible = 0,  // Hidden from sidebar
    Nothing = 1,    // No notifications
    Mentions = 2,   // @mention notifications only
    Everything = 3, // All message notifications
}

/// Session model for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub token: String,  // Critical Gap #4: secure token
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_active_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// WebSocket connection with state machine
#[derive(Debug, Clone)]
pub struct WebSocketConnection<State> {
    pub id: ConnectionId,
    pub user_id: Option<UserId>,
    pub room_id: Option<RoomId>,
    pub last_seen_message_id: Option<MessageId>,  // Critical Gap #2: reconnection
    pub connected_at: DateTime<Utc>,
    _state: std::marker::PhantomData<State>,
}

/// WebSocket connection states
pub struct Connected;
pub struct Authenticated { pub user_id: UserId }
pub struct Subscribed { pub room_id: RoomId }
```

## Comprehensive Error Type Hierarchy

```rust
use thiserror::Error;

/// Message service errors - all validation, database, and authorization cases
#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Database operation failed: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Message validation failed: {field} - {message}")]
    Validation { field: String, message: String },
    
    #[error("User {user_id:?} not authorized for room {room_id:?}")]
    Authorization { user_id: UserId, room_id: RoomId },
    
    #[error("Room {room_id:?} not found")]
    RoomNotFound { room_id: RoomId },
    
    #[error("Message {message_id:?} not found")]
    MessageNotFound { message_id: MessageId },
    
    #[error("Content too long: {length} characters (max 10000)")]
    ContentTooLong { length: usize },
    
    #[error("Empty message content not allowed")]
    EmptyContent,
    
    #[error("Duplicate client message ID handled")]
    DuplicateClientId { existing_message_id: MessageId },
    
    #[error("Database writer unavailable")]
    WriterUnavailable,
}

/// Room service errors - access control and membership errors
#[derive(Error, Debug)]
pub enum RoomError {
    #[error("Database operation failed: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Room validation failed: {field} - {message}")]
    Validation { field: String, message: String },
    
    #[error("User {user_id:?} not authorized for this operation")]
    NotAuthorized { user_id: UserId },
    
    #[error("Room {room_id:?} not found")]
    NotFound { room_id: RoomId },
    
    #[error("User {user_id:?} not found")]
    UserNotFound { user_id: UserId },
    
    #[error("Membership already exists for user {user_id:?} in room {room_id:?}")]
    MembershipExists { user_id: UserId, room_id: RoomId },
    
    #[error("Direct room already exists between users")]
    DirectRoomExists { room_id: RoomId },
    
    #[error("Cannot modify system room")]
    SystemRoomProtected,
    
    #[error("Room name too long: {length} characters (max 100)")]
    NameTooLong { length: usize },
}

/// Authentication service errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Database operation failed: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User not found: {email}")]
    UserNotFound { email: String },
    
    #[error("User account is deactivated")]
    AccountDeactivated,
    
    #[error("Session not found or expired")]
    SessionNotFound,
    
    #[error("Invalid session token")]
    InvalidToken,
    
    #[error("Rate limit exceeded: {attempts} attempts in {window_minutes} minutes")]
    RateLimitExceeded { attempts: u32, window_minutes: u32 },
    
    #[error("Invalid join code format")]
    InvalidJoinCode,
    
    #[error("Email already registered: {email}")]
    EmailExists { email: String },
    
    #[error("Bot token invalid or expired")]
    InvalidBotToken,
    
    #[error("Password validation failed: {reason}")]
    PasswordValidation { reason: String },
}

/// WebSocket and presence errors
#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    
    #[error("Connection not authenticated")]
    NotAuthenticated,
    
    #[error("Connection not subscribed to room")]
    NotSubscribed,
    
    #[error("User {user_id:?} not authorized for room {room_id:?}")]
    NotAuthorized { user_id: UserId, room_id: RoomId },
    
    #[error("Room {room_id:?} not found")]
    RoomNotFound { room_id: RoomId },
    
    #[error("Connection {connection_id:?} not found")]
    ConnectionNotFound { connection_id: ConnectionId },
    
    #[error("Broadcast failed: {reason}")]
    BroadcastFailed { reason: String },
    
    #[error("Presence tracking error: {reason}")]
    PresenceError { reason: String },
    
    #[error("Connection limit exceeded")]
    ConnectionLimitExceeded,
}

/// Webhook and bot integration errors
#[derive(Error, Debug)]
pub enum WebhookError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Webhook timeout after {seconds} seconds")]
    Timeout { seconds: u64 },
    
    #[error("Invalid webhook URL: {url}")]
    InvalidUrl { url: String },
    
    #[error("Webhook response too large: {size} bytes")]
    ResponseTooLarge { size: usize },
    
    #[error("Invalid response content type: {content_type}")]
    InvalidContentType { content_type: String },
    
    #[error("Bot not found: {bot_id:?}")]
    BotNotFound { bot_id: UserId },
    
    #[error("Webhook delivery failed: {status_code}")]
    DeliveryFailed { status_code: u16 },
}
```

## Service Trait Interfaces

### MessageService - Complete Interface Contract

```rust
use async_trait::async_trait;

/// Message service - handles all message operations with Rails parity
#[async_trait]
pub trait MessageService: Send + Sync {
    /// Creates a message with automatic deduplication based on client_message_id
    /// 
    /// # Critical Gap #1: Deduplication
    /// If client_message_id already exists in room, returns existing message
    /// 
    /// # Arguments
    /// * `content` - Message content (1-10000 characters)
    /// * `room_id` - Target room identifier
    /// * `creator_id` - Message creator identifier  
    /// * `client_message_id` - Client-generated UUID for deduplication
    ///
    /// # Returns
    /// * `Ok(Message<Persisted>)` - Created or existing message
    /// * `Err(MessageError::Validation)` - Invalid input parameters
    /// * `Err(MessageError::Authorization)` - User cannot access room
    /// * `Err(MessageError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Updates room.last_message_at timestamp
    /// * Broadcasts message to room subscribers via WebSocket
    /// * Updates FTS5 search index
    async fn create_message_with_deduplication(
        &self,
        content: String,
        room_id: RoomId,
        creator_id: UserId,
        client_message_id: Uuid,
    ) -> Result<Message<Persisted>, MessageError>;

    /// Gets messages since a specific message ID (for reconnection)
    /// 
    /// # Critical Gap #2: Reconnection State
    /// Returns all messages after the given ID in chronological order
    /// 
    /// # Arguments
    /// * `room_id` - Room to get messages from
    /// * `since_message_id` - Get messages after this ID
    /// * `limit` - Maximum number of messages (default 50)
    ///
    /// # Returns
    /// * `Ok(Vec<Message<Persisted>>)` - Messages in chronological order
    /// * `Err(MessageError::Authorization)` - User cannot access room
    /// * `Err(MessageError::Database)` - Database operation failed
    async fn get_messages_since(
        &self,
        room_id: RoomId,
        since_message_id: MessageId,
        user_id: UserId,
        limit: Option<u32>,
    ) -> Result<Vec<Message<Persisted>>, MessageError>;

    /// Gets recent messages for a room (pagination support)
    /// 
    /// # Arguments
    /// * `room_id` - Room to get messages from
    /// * `before_message_id` - Get messages before this ID (for pagination)
    /// * `limit` - Maximum number of messages (default 50)
    /// * `user_id` - User requesting messages (for authorization)
    ///
    /// # Returns
    /// * `Ok(Vec<Message<Persisted>>)` - Messages in reverse chronological order
    /// * `Err(MessageError::Authorization)` - User cannot access room
    /// * `Err(MessageError::Database)` - Database operation failed
    async fn get_recent_messages(
        &self,
        room_id: RoomId,
        before_message_id: Option<MessageId>,
        user_id: UserId,
        limit: Option<u32>,
    ) -> Result<Vec<Message<Persisted>>, MessageError>;

    /// Updates an existing message (creator or admin only)
    /// 
    /// # Arguments
    /// * `message_id` - Message to update
    /// * `new_content` - New message content
    /// * `user_id` - User requesting update
    ///
    /// # Returns
    /// * `Ok(Message<Persisted>)` - Updated message
    /// * `Err(MessageError::Authorization)` - User cannot edit this message
    /// * `Err(MessageError::MessageNotFound)` - Message doesn't exist
    /// * `Err(MessageError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Broadcasts update to room subscribers
    /// * Updates FTS5 search index
    async fn update_message(
        &self,
        message_id: MessageId,
        new_content: String,
        user_id: UserId,
    ) -> Result<Message<Persisted>, MessageError>;

    /// Deletes a message (creator or admin only)
    /// 
    /// # Arguments
    /// * `message_id` - Message to delete
    /// * `user_id` - User requesting deletion
    ///
    /// # Returns
    /// * `Ok(())` - Message deleted successfully
    /// * `Err(MessageError::Authorization)` - User cannot delete this message
    /// * `Err(MessageError::MessageNotFound)` - Message doesn't exist
    /// * `Err(MessageError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Broadcasts deletion to room subscribers
    /// * Removes from FTS5 search index
    async fn delete_message(
        &self,
        message_id: MessageId,
        user_id: UserId,
    ) -> Result<(), MessageError>;

    /// Searches messages using FTS5 (Rails equivalent)
    /// 
    /// # Arguments
    /// * `query` - Search query string
    /// * `user_id` - User performing search (for room access filtering)
    /// * `room_id` - Optional room filter
    /// * `limit` - Maximum results (default 50)
    ///
    /// # Returns
    /// * `Ok(Vec<Message<Persisted>>)` - Matching messages
    /// * `Err(MessageError::Database)` - Search operation failed
    async fn search_messages(
        &self,
        query: String,
        user_id: UserId,
        room_id: Option<RoomId>,
        limit: Option<u32>,
    ) -> Result<Vec<Message<Persisted>>, MessageError>;

    /// Validates message content (Rails equivalent rules)
    /// 
    /// # Arguments
    /// * `content` - Content to validate
    ///
    /// # Returns
    /// * `Ok(())` - Content is valid
    /// * `Err(MessageError::Validation)` - Content validation failed
    fn validate_content(&self, content: &str) -> Result<(), MessageError>;
}
```

### RoomService - Complete Interface Contract

```rust
/// Room service - handles room management with Rails STI pattern
#[async_trait]
pub trait RoomService: Send + Sync {
    /// Creates a new room with automatic membership granting
    /// 
    /// # Arguments
    /// * `name` - Room name (1-100 characters)
    /// * `room_type` - Type of room (Open/Closed/Direct)
    /// * `creator_id` - User creating the room
    /// * `initial_members` - For closed rooms, initial member list
    ///
    /// # Returns
    /// * `Ok(Room)` - Created room
    /// * `Err(RoomError::Validation)` - Invalid room parameters
    /// * `Err(RoomError::DirectRoomExists)` - Direct room already exists
    /// * `Err(RoomError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Creates memberships for appropriate users
    /// * For Open rooms: grants membership to all active users
    /// * For Direct rooms: enforces singleton pattern
    /// * Broadcasts room creation to affected users
    async fn create_room(
        &self,
        name: String,
        room_type: RoomType,
        creator_id: UserId,
        initial_members: Option<Vec<UserId>>,
    ) -> Result<Room, RoomError>;

    /// Finds or creates a direct room between two users
    /// 
    /// # Arguments
    /// * `user1_id` - First participant
    /// * `user2_id` - Second participant
    ///
    /// # Returns
    /// * `Ok(Room)` - Existing or newly created direct room
    /// * `Err(RoomError::UserNotFound)` - One of the users doesn't exist
    /// * `Err(RoomError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Creates room if it doesn't exist
    /// * Sets involvement to Everything for both users
    /// * Auto-generates name from member list
    async fn find_or_create_direct_room(
        &self,
        user1_id: UserId,
        user2_id: UserId,
    ) -> Result<Room, RoomError>;

    /// Gets all rooms accessible to a user
    /// 
    /// # Arguments
    /// * `user_id` - User to get rooms for
    /// * `include_invisible` - Whether to include invisible memberships
    ///
    /// # Returns
    /// * `Ok(Vec<Room>)` - Accessible rooms
    /// * `Err(RoomError::Database)` - Database operation failed
    async fn get_user_rooms(
        &self,
        user_id: UserId,
        include_invisible: bool,
    ) -> Result<Vec<Room>, RoomError>;

    /// Grants membership to a user (admin or creator only)
    /// 
    /// # Arguments
    /// * `room_id` - Room to grant access to
    /// * `user_id` - User to grant membership
    /// * `granter_id` - User granting membership (must be admin/creator)
    /// * `involvement` - Initial involvement level
    ///
    /// # Returns
    /// * `Ok(Membership)` - Created membership
    /// * `Err(RoomError::NotAuthorized)` - Granter cannot modify this room
    /// * `Err(RoomError::MembershipExists)` - User already has membership
    /// * `Err(RoomError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Broadcasts membership change to affected users
    /// * For Open rooms: auto-grants to new users joining account
    async fn grant_membership(
        &self,
        room_id: RoomId,
        user_id: UserId,
        granter_id: UserId,
        involvement: Involvement,
    ) -> Result<Membership, RoomError>;

    /// Revokes membership from a user (admin or creator only)
    /// 
    /// # Arguments
    /// * `room_id` - Room to revoke access from
    /// * `user_id` - User to revoke membership
    /// * `revoker_id` - User revoking membership (must be admin/creator)
    ///
    /// # Returns
    /// * `Ok(())` - Membership revoked successfully
    /// * `Err(RoomError::NotAuthorized)` - Revoker cannot modify this room
    /// * `Err(RoomError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Closes WebSocket connections for revoked user
    /// * Broadcasts membership change to affected users
    async fn revoke_membership(
        &self,
        room_id: RoomId,
        user_id: UserId,
        revoker_id: UserId,
    ) -> Result<(), RoomError>;

    /// Updates user's involvement level in a room
    /// 
    /// # Arguments
    /// * `room_id` - Room to update involvement in
    /// * `user_id` - User whose involvement to update
    /// * `new_involvement` - New involvement level
    ///
    /// # Returns
    /// * `Ok(Membership)` - Updated membership
    /// * `Err(RoomError::NotAuthorized)` - User cannot access this room
    /// * `Err(RoomError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Broadcasts involvement change to user's connections
    /// * Updates sidebar display for user
    async fn update_involvement(
        &self,
        room_id: RoomId,
        user_id: UserId,
        new_involvement: Involvement,
    ) -> Result<Membership, RoomError>;

    /// Marks room as read for a user (clears unread_at)
    /// 
    /// # Arguments
    /// * `room_id` - Room to mark as read
    /// * `user_id` - User marking room as read
    ///
    /// # Returns
    /// * `Ok(())` - Room marked as read
    /// * `Err(RoomError::NotAuthorized)` - User cannot access this room
    /// * `Err(RoomError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Clears unread_at timestamp
    /// * Broadcasts read state change to user's connections
    async fn mark_room_as_read(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<(), RoomError>;

    /// Updates room when it receives a message (Rails pattern)
    /// 
    /// # Arguments
    /// * `room_id` - Room that received message
    /// * `message` - The message that was received
    ///
    /// # Returns
    /// * `Ok(())` - Room updated successfully
    /// * `Err(RoomError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Updates last_message_at timestamp
    /// * Sets unread_at for disconnected visible members (excluding creator)
    async fn receive_message(
        &self,
        room_id: RoomId,
        message: &Message<Persisted>,
    ) -> Result<(), RoomError>;

    /// Validates room parameters (Rails equivalent rules)
    /// 
    /// # Arguments
    /// * `name` - Room name to validate
    /// * `room_type` - Room type to validate
    ///
    /// # Returns
    /// * `Ok(())` - Parameters are valid
    /// * `Err(RoomError::Validation)` - Validation failed
    fn validate_room_params(&self, name: &str, room_type: &RoomType) -> Result<(), RoomError>;
}
```

I'll continue with the remaining service interfaces in the next part. This establishes the foundation with complete type contracts for the core domain models and the first two major services. 

Would you like me to continue with the AuthService, WebSocketBroadcaster, and other service interfaces?
###
 AuthService - Complete Interface Contract

```rust
/// Authentication service - handles user auth with Rails session management
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Authenticates user with email and password (Rails equivalent)
    /// 
    /// # Arguments
    /// * `email` - User email address
    /// * `password` - Plain text password
    /// * `ip_address` - Client IP for session tracking
    /// * `user_agent` - Client user agent for session tracking
    ///
    /// # Returns
    /// * `Ok(Session)` - Created session with secure token
    /// * `Err(AuthError::InvalidCredentials)` - Wrong email/password
    /// * `Err(AuthError::AccountDeactivated)` - User account is disabled
    /// * `Err(AuthError::RateLimitExceeded)` - Too many login attempts
    /// * `Err(AuthError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Creates session record with secure token (Critical Gap #4)
    /// * Updates user's last_active_at timestamp
    /// * Logs security event
    async fn authenticate_user(
        &self,
        email: String,
        password: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<Session, AuthError>;

    /// Authenticates bot with bot key (id-token format)
    /// 
    /// # Arguments
    /// * `bot_key` - Bot key in "id-token" format
    ///
    /// # Returns
    /// * `Ok(User)` - Authenticated bot user
    /// * `Err(AuthError::InvalidBotToken)` - Invalid or expired bot token
    /// * `Err(AuthError::AccountDeactivated)` - Bot account is disabled
    /// * `Err(AuthError::Database)` - Database operation failed
    async fn authenticate_bot(&self, bot_key: String) -> Result<User, AuthError>;

    /// Validates session token and returns user
    /// 
    /// # Arguments
    /// * `session_token` - Session token from cookie
    ///
    /// # Returns
    /// * `Ok((User, Session))` - Valid user and session
    /// * `Err(AuthError::SessionNotFound)` - Invalid or expired session
    /// * `Err(AuthError::AccountDeactivated)` - User account is disabled
    /// * `Err(AuthError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Updates session's last_active_at if > 1 hour old
    async fn validate_session(&self, session_token: String) -> Result<(User, Session), AuthError>;

    /// Creates new user account with join code verification
    /// 
    /// # Arguments
    /// * `email` - User email address
    /// * `name` - User display name
    /// * `password` - Plain text password (will be hashed)
    /// * `join_code` - Account join code (XXXX-XXXX-XXXX format)
    ///
    /// # Returns
    /// * `Ok(User)` - Created user account
    /// * `Err(AuthError::InvalidJoinCode)` - Wrong join code
    /// * `Err(AuthError::EmailExists)` - Email already registered
    /// * `Err(AuthError::PasswordValidation)` - Password too weak
    /// * `Err(AuthError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Hashes password with bcrypt
    /// * Auto-grants memberships to all Open rooms
    /// * Creates default user role (member)
    async fn create_user_account(
        &self,
        email: String,
        name: String,
        password: String,
        join_code: String,
    ) -> Result<User, AuthError>;

    /// Creates bot user account (admin only)
    /// 
    /// # Arguments
    /// * `name` - Bot display name
    /// * `webhook_url` - Optional webhook URL for bot responses
    /// * `creator_id` - Admin user creating the bot
    ///
    /// # Returns
    /// * `Ok((User, BotToken))` - Created bot and its token
    /// * `Err(AuthError::NotAuthorized)` - Creator is not admin
    /// * `Err(AuthError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Generates secure bot token (SecureRandom.alphanumeric(12))
    /// * Sets user role to Bot
    /// * Creates email with UUID suffix for uniqueness
    async fn create_bot_account(
        &self,
        name: String,
        webhook_url: Option<String>,
        creator_id: UserId,
    ) -> Result<(User, BotToken), AuthError>;

    /// Destroys user session (logout)
    /// 
    /// # Arguments
    /// * `session_token` - Session token to destroy
    ///
    /// # Returns
    /// * `Ok(())` - Session destroyed successfully
    /// * `Err(AuthError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Removes session record from database
    /// * Removes push notification subscription
    async fn destroy_session(&self, session_token: String) -> Result<(), AuthError>;

    /// Deactivates user account (admin only)
    /// 
    /// # Arguments
    /// * `user_id` - User to deactivate
    /// * `admin_id` - Admin performing deactivation
    ///
    /// # Returns
    /// * `Ok(())` - User deactivated successfully
    /// * `Err(AuthError::NotAuthorized)` - Admin lacks permission
    /// * `Err(AuthError::UserNotFound)` - User doesn't exist
    /// * `Err(AuthError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Closes all remote connections
    /// * Deletes non-direct memberships
    /// * Anonymizes email with UUID suffix
    /// * Sets active=false
    /// * Deletes all sessions
    async fn deactivate_user(
        &self,
        user_id: UserId,
        admin_id: UserId,
    ) -> Result<(), AuthError>;

    /// Resets bot token (admin only)
    /// 
    /// # Arguments
    /// * `bot_id` - Bot user to reset token for
    /// * `admin_id` - Admin performing reset
    ///
    /// # Returns
    /// * `Ok(BotToken)` - New bot token
    /// * `Err(AuthError::NotAuthorized)` - Admin lacks permission
    /// * `Err(AuthError::UserNotFound)` - Bot doesn't exist
    /// * `Err(AuthError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Generates new secure token
    /// * Invalidates old token immediately
    async fn reset_bot_token(
        &self,
        bot_id: UserId,
        admin_id: UserId,
    ) -> Result<BotToken, AuthError>;

    /// Generates secure session token (Critical Gap #4)
    /// 
    /// # Returns
    /// * `String` - Cryptographically secure token (Rails SecureRandom equivalent)
    fn generate_secure_token(&self) -> String;

    /// Validates password strength (Rails equivalent rules)
    /// 
    /// # Arguments
    /// * `password` - Password to validate
    ///
    /// # Returns
    /// * `Ok(())` - Password meets requirements
    /// * `Err(AuthError::PasswordValidation)` - Password too weak
    fn validate_password(&self, password: &str) -> Result<(), AuthError>;

    /// Validates join code format (XXXX-XXXX-XXXX)
    /// 
    /// # Arguments
    /// * `join_code` - Join code to validate
    /// * `account_join_code` - Expected join code for account
    ///
    /// # Returns
    /// * `Ok(())` - Join code is valid
    /// * `Err(AuthError::InvalidJoinCode)` - Join code is invalid
    fn validate_join_code(&self, join_code: &str, account_join_code: &str) -> Result<(), AuthError>;
}
```

### WebSocketBroadcaster - Complete Interface Contract

```rust
/// WebSocket broadcaster - handles real-time communication (Rails ActionCable equivalent)
#[async_trait]
pub trait WebSocketBroadcaster: Send + Sync {
    /// Adds a WebSocket connection to the broadcaster
    /// 
    /// # Arguments
    /// * `connection` - WebSocket connection in Connected state
    ///
    /// # Returns
    /// * `Ok(ConnectionId)` - Assigned connection ID
    /// * `Err(ConnectionError::ConnectionLimitExceeded)` - Too many connections
    ///
    /// # Side Effects
    /// * Stores connection for future broadcasts
    /// * Starts heartbeat monitoring
    async fn add_connection(
        &self,
        connection: WebSocketConnection<Connected>,
    ) -> Result<ConnectionId, ConnectionError>;

    /// Authenticates a WebSocket connection
    /// 
    /// # Arguments
    /// * `connection_id` - Connection to authenticate
    /// * `session_token` - Session token from cookie
    ///
    /// # Returns
    /// * `Ok(WebSocketConnection<Authenticated>)` - Authenticated connection
    /// * `Err(ConnectionError::NotAuthenticated)` - Invalid session
    /// * `Err(ConnectionError::ConnectionNotFound)` - Connection doesn't exist
    ///
    /// # Side Effects
    /// * Updates connection state to Authenticated
    /// * Associates connection with user
    async fn authenticate_connection(
        &self,
        connection_id: ConnectionId,
        session_token: String,
    ) -> Result<WebSocketConnection<Authenticated>, ConnectionError>;

    /// Subscribes connection to a room
    /// 
    /// # Arguments
    /// * `connection_id` - Authenticated connection
    /// * `room_id` - Room to subscribe to
    ///
    /// # Returns
    /// * `Ok(WebSocketConnection<Subscribed>)` - Subscribed connection
    /// * `Err(ConnectionError::NotAuthorized)` - User cannot access room
    /// * `Err(ConnectionError::RoomNotFound)` - Room doesn't exist
    ///
    /// # Side Effects
    /// * Updates connection state to Subscribed
    /// * Increments presence count for user in room (Critical Gap #5)
    /// * Clears unread_at for user in room
    /// * Broadcasts presence update to room
    async fn subscribe_to_room(
        &self,
        connection_id: ConnectionId,
        room_id: RoomId,
    ) -> Result<WebSocketConnection<Subscribed>, ConnectionError>;

    /// Handles WebSocket reconnection with state sync (Critical Gap #2)
    /// 
    /// # Arguments
    /// * `connection_id` - Reconnecting connection
    /// * `last_seen_message_id` - Last message ID client received
    ///
    /// # Returns
    /// * `Ok(Vec<Message<Persisted>>)` - Missed messages since last_seen
    /// * `Err(ConnectionError::ConnectionNotFound)` - Connection doesn't exist
    ///
    /// # Side Effects
    /// * Sends missed messages to client
    /// * Updates connection's last_seen_message_id
    /// * Restores room subscriptions
    async fn handle_reconnection(
        &self,
        connection_id: ConnectionId,
        last_seen_message_id: Option<MessageId>,
    ) -> Result<Vec<Message<Persisted>>, ConnectionError>;

    /// Broadcasts message to all room subscribers (Rails ActionCable pattern)
    /// 
    /// # Arguments
    /// * `room_id` - Room to broadcast to
    /// * `message` - Message to broadcast
    ///
    /// # Returns
    /// * `Ok(u32)` - Number of connections message was sent to
    /// * `Err(ConnectionError::BroadcastFailed)` - Broadcast operation failed
    ///
    /// # Side Effects
    /// * Sends message to all connected room members
    /// * Updates last_seen_message_id for all connections
    /// * Logs failed sends but continues (best-effort delivery)
    async fn broadcast_to_room(
        &self,
        room_id: RoomId,
        message: &Message<Persisted>,
    ) -> Result<u32, ConnectionError>;

    /// Broadcasts typing notification to room
    /// 
    /// # Arguments
    /// * `room_id` - Room to broadcast to
    /// * `user_id` - User who is typing
    /// * `is_typing` - Whether user started or stopped typing
    ///
    /// # Returns
    /// * `Ok(())` - Notification broadcast successfully
    /// * `Err(ConnectionError::BroadcastFailed)` - Broadcast failed
    ///
    /// # Side Effects
    /// * Sends typing notification to room subscribers (excluding sender)
    /// * Throttles notifications to prevent spam
    /// * Clears typing state after 5 seconds of inactivity
    async fn broadcast_typing_notification(
        &self,
        room_id: RoomId,
        user_id: UserId,
        is_typing: bool,
    ) -> Result<(), ConnectionError>;

    /// Broadcasts presence update to room
    /// 
    /// # Arguments
    /// * `room_id` - Room to broadcast to
    /// * `user_id` - User whose presence changed
    /// * `is_present` - Whether user is now present or absent
    ///
    /// # Returns
    /// * `Ok(())` - Presence update broadcast successfully
    /// * `Err(ConnectionError::BroadcastFailed)` - Broadcast failed
    ///
    /// # Side Effects
    /// * Updates presence tracking for user (Critical Gap #5)
    /// * Sends presence update to room subscribers
    /// * Handles 5-second delay for visibility changes
    async fn broadcast_presence_update(
        &self,
        room_id: RoomId,
        user_id: UserId,
        is_present: bool,
    ) -> Result<(), ConnectionError>;

    /// Removes WebSocket connection
    /// 
    /// # Arguments
    /// * `connection_id` - Connection to remove
    ///
    /// # Returns
    /// * `Ok(())` - Connection removed successfully
    /// * `Err(ConnectionError::ConnectionNotFound)` - Connection doesn't exist
    ///
    /// # Side Effects
    /// * Decrements presence count for user in subscribed rooms
    /// * Broadcasts presence updates if user goes offline
    /// * Cleans up connection resources
    async fn remove_connection(&self, connection_id: ConnectionId) -> Result<(), ConnectionError>;

    /// Refreshes connection heartbeat (prevents timeout)
    /// 
    /// # Arguments
    /// * `connection_id` - Connection to refresh
    ///
    /// # Returns
    /// * `Ok(())` - Heartbeat refreshed successfully
    /// * `Err(ConnectionError::ConnectionNotFound)` - Connection doesn't exist
    ///
    /// # Side Effects
    /// * Updates connection's last_active timestamp
    /// * Prevents connection from being cleaned up
    async fn refresh_heartbeat(&self, connection_id: ConnectionId) -> Result<(), ConnectionError>;

    /// Gets online users for a room (presence tracking)
    /// 
    /// # Arguments
    /// * `room_id` - Room to get online users for
    ///
    /// # Returns
    /// * `Ok(Vec<UserId>)` - Currently online users in room
    /// * `Err(ConnectionError::RoomNotFound)` - Room doesn't exist
    async fn get_online_users(&self, room_id: RoomId) -> Result<Vec<UserId>, ConnectionError>;

    /// Cleans up stale connections (background task)
    /// 
    /// # Returns
    /// * `Ok(u32)` - Number of connections cleaned up
    ///
    /// # Side Effects
    /// * Removes connections inactive for > 60 seconds
    /// * Updates presence tracking for affected users
    /// * Broadcasts presence updates for users who went offline
    async fn cleanup_stale_connections(&self) -> Result<u32, ConnectionError>;
}
```

### DatabaseWriter - Write Serialization Interface (Critical Gap #3)

```rust
/// Database writer - serializes all write operations (Critical Gap #3)
#[async_trait]
pub trait DatabaseWriter: Send + Sync {
    /// Submits a write operation to the serialized writer queue
    /// 
    /// # Arguments
    /// * `operation` - Write operation to execute
    ///
    /// # Returns
    /// * `Ok(WriteResult)` - Operation completed successfully
    /// * `Err(MessageError::WriterUnavailable)` - Writer task is not running
    /// * `Err(MessageError::Database)` - Database operation failed
    ///
    /// # Side Effects
    /// * Queues operation for sequential execution
    /// * Ensures all writes are serialized (Rails connection pool equivalent)
    async fn submit_write(&self, operation: WriteOperation) -> Result<WriteResult, MessageError>;

    /// Gracefully shuts down the writer (drains pending operations)
    /// 
    /// # Returns
    /// * `Ok(u32)` - Number of operations drained
    ///
    /// # Side Effects
    /// * Processes all queued operations before shutdown
    /// * Prevents new operations from being queued
    async fn shutdown(&self) -> Result<u32, MessageError>;
}

/// Write operations that can be serialized
#[derive(Debug)]
pub enum WriteOperation {
    CreateMessage {
        content: String,
        room_id: RoomId,
        creator_id: UserId,
        client_message_id: Uuid,
    },
    UpdateMessage {
        message_id: MessageId,
        new_content: String,
    },
    DeleteMessage {
        message_id: MessageId,
    },
    CreateRoom {
        name: String,
        room_type: RoomType,
        creator_id: UserId,
    },
    UpdateRoomTimestamp {
        room_id: RoomId,
        timestamp: DateTime<Utc>,
    },
    CreateMembership {
        user_id: UserId,
        room_id: RoomId,
        involvement: Involvement,
    },
    UpdateMembership {
        user_id: UserId,
        room_id: RoomId,
        involvement: Option<Involvement>,
        unread_at: Option<DateTime<Utc>>,
        connections: Option<i32>,
        connected_at: Option<DateTime<Utc>>,
    },
    CreateSession {
        user_id: UserId,
        token: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    },
    DeleteSession {
        token: String,
    },
}

/// Results from write operations
#[derive(Debug)]
pub enum WriteResult {
    MessageCreated(Message<Persisted>),
    MessageUpdated(Message<Persisted>),
    MessageDeleted,
    RoomCreated(Room),
    RoomUpdated(Room),
    MembershipCreated(Membership),
    MembershipUpdated(Membership),
    SessionCreated(Session),
    SessionDeleted,
}
```

### NotificationService - Push Notifications Interface

```rust
/// Notification service - handles Web Push notifications
#[async_trait]
pub trait NotificationService: Send + Sync {
    /// Sends push notification to user
    /// 
    /// # Arguments
    /// * `user_id` - User to send notification to
    /// * `title` - Notification title
    /// * `body` - Notification body
    /// * `room_id` - Optional room context
    ///
    /// # Returns
    /// * `Ok(u32)` - Number of devices notified
    /// * `Err(NotificationError)` - Notification failed
    ///
    /// # Side Effects
    /// * Sends Web Push to all user's subscribed devices
    /// * Respects user's involvement level settings
    async fn send_push_notification(
        &self,
        user_id: UserId,
        title: String,
        body: String,
        room_id: Option<RoomId>,
    ) -> Result<u32, NotificationError>;

    /// Subscribes device to push notifications
    /// 
    /// # Arguments
    /// * `user_id` - User subscribing device
    /// * `subscription` - Web Push subscription details
    ///
    /// # Returns
    /// * `Ok(())` - Subscription created successfully
    /// * `Err(NotificationError)` - Subscription failed
    async fn subscribe_device(
        &self,
        user_id: UserId,
        subscription: PushSubscription,
    ) -> Result<(), NotificationError>;

    /// Unsubscribes device from push notifications
    /// 
    /// # Arguments
    /// * `user_id` - User unsubscribing device
    /// * `endpoint` - Push subscription endpoint to remove
    ///
    /// # Returns
    /// * `Ok(())` - Subscription removed successfully
    /// * `Err(NotificationError)` - Unsubscription failed
    async fn unsubscribe_device(
        &self,
        user_id: UserId,
        endpoint: String,
    ) -> Result<(), NotificationError>;
}

#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("Database operation failed: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Push service error: {0}")]
    PushService(String),
    
    #[error("Invalid subscription: {reason}")]
    InvalidSubscription { reason: String },
    
    #[error("User has no push subscriptions")]
    NoSubscriptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub endpoint: String,
    pub keys: PushKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}
```

### WebhookService - Bot Integration Interface

```rust
/// Webhook service - handles bot webhook delivery
#[async_trait]
pub trait WebhookService: Send + Sync {
    /// Delivers webhook to bot
    /// 
    /// # Arguments
    /// * `bot_id` - Bot to deliver webhook to
    /// * `payload` - Webhook payload data
    ///
    /// # Returns
    /// * `Ok(Option<String>)` - Bot response content (if any)
    /// * `Err(WebhookError)` - Webhook delivery failed
    ///
    /// # Side Effects
    /// * POSTs to bot's webhook_url with 7-second timeout
    /// * Processes bot response and creates reply message if applicable
    /// * Logs delivery success/failure
    async fn deliver_webhook(
        &self,
        bot_id: UserId,
        payload: WebhookPayload,
    ) -> Result<Option<String>, WebhookError>;

    /// Checks if webhook should be triggered for message
    /// 
    /// # Arguments
    /// * `message` - Message that was created
    /// * `room` - Room the message was sent to
    ///
    /// # Returns
    /// * `Ok(Vec<UserId>)` - Bot IDs that should receive webhook
    ///
    /// # Side Effects
    /// * Checks for @mentions of bots
    /// * Checks for messages in Direct rooms with bot membership
    async fn get_webhook_targets(
        &self,
        message: &Message<Persisted>,
        room: &Room,
    ) -> Result<Vec<UserId>, WebhookError>;

    /// Builds webhook payload for bot
    /// 
    /// # Arguments
    /// * `message` - Message that triggered webhook
    /// * `room` - Room the message was sent to
    /// * `user` - User who sent the message
    ///
    /// # Returns
    /// * `Ok(WebhookPayload)` - Constructed payload
    fn build_webhook_payload(
        &self,
        message: &Message<Persisted>,
        room: &Room,
        user: &User,
    ) -> Result<WebhookPayload, WebhookError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub user: WebhookUser,
    pub room: WebhookRoom,
    pub message: WebhookMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookUser {
    pub id: UserId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRoom {
    pub id: RoomId,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Serialize)]
pub struct WebhookMessage {
    pub id: MessageId,
    pub body: WebhookMessageBody,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMessageBody {
    pub html: String,
    pub plain: String,
}
```

## Implementation State Machine Contracts

### Message State Transitions

```rust
impl Message<Draft> {
    /// Creates a new draft message
    pub fn new_draft(
        content: String,
        room_id: RoomId,
        creator_id: UserId,
        client_message_id: Uuid,
    ) -> Self {
        Self {
            id: MessageId(0), // Will be set on persistence
            client_message_id,
            content,
            room_id,
            creator_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            _state: std::marker::PhantomData,
        }
    }

    /// Validates message content and transitions to Validated state
    pub fn validate(self) -> Result<Message<Validated>, MessageError> {
        if self.content.trim().is_empty() {
            return Err(MessageError::EmptyContent);
        }
        
        if self.content.len() > 10000 {
            return Err(MessageError::ContentTooLong { 
                length: self.content.len() 
            });
        }
        
        Ok(Message {
            id: self.id,
            client_message_id: self.client_message_id,
            content: self.content,
            room_id: self.room_id,
            creator_id: self.creator_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
            _state: std::marker::PhantomData,
        })
    }
}

impl Message<Validated> {
    /// Persists message to database and transitions to Persisted state
    pub async fn persist(
        self,
        writer: &dyn DatabaseWriter,
    ) -> Result<Message<Persisted>, MessageError> {
        let operation = WriteOperation::CreateMessage {
            content: self.content.clone(),
            room_id: self.room_id,
            creator_id: self.creator_id,
            client_message_id: self.client_message_id,
        };
        
        match writer.submit_write(operation).await? {
            WriteResult::MessageCreated(persisted_message) => Ok(persisted_message),
            _ => Err(MessageError::Database(sqlx::Error::RowNotFound)),
        }
    }
}

impl Message<Persisted> {
    /// Only persisted messages can be broadcast
    pub fn can_broadcast(&self) -> bool {
        true
    }
    
    /// Extract mentions from persisted message
    pub fn extract_mentions(&self) -> Vec<String> {
        // Implementation would parse @mentions from content
        todo!("Extract @mentions from message content")
    }
    
    /// Check if message is a sound command
    pub fn is_sound_command(&self) -> bool {
        self.content.starts_with("/play ")
    }
    
    /// Get sound name from command
    pub fn get_sound_name(&self) -> Option<&str> {
        self.content.strip_prefix("/play ")
    }
}
```

### WebSocket Connection State Transitions

```rust
impl WebSocketConnection<Connected> {
    /// Creates a new connected WebSocket connection
    pub fn new_connected(id: ConnectionId) -> Self {
        Self {
            id,
            user_id: None,
            room_id: None,
            last_seen_message_id: None,
            connected_at: Utc::now(),
            _state: std::marker::PhantomData,
        }
    }

    /// Authenticates connection and transitions to Authenticated state
    pub fn authenticate(self, user_id: UserId) -> WebSocketConnection<Authenticated> {
        WebSocketConnection {
            id: self.id,
            user_id: Some(user_id),
            room_id: self.room_id,
            last_seen_message_id: self.last_seen_message_id,
            connected_at: self.connected_at,
            _state: std::marker::PhantomData,
        }
    }
}

impl WebSocketConnection<Authenticated> {
    /// Subscribes to room and transitions to Subscribed state
    pub fn subscribe_to_room(self, room_id: RoomId) -> WebSocketConnection<Subscribed> {
        WebSocketConnection {
            id: self.id,
            user_id: self.user_id,
            room_id: Some(room_id),
            last_seen_message_id: self.last_seen_message_id,
            connected_at: self.connected_at,
            _state: std::marker::PhantomData,
        }
    }
    
    /// Gets authenticated user ID
    pub fn user_id(&self) -> UserId {
        self.user_id.expect("Authenticated connection must have user_id")
    }
}

impl WebSocketConnection<Subscribed> {
    /// Only subscribed connections can receive room messages
    pub fn can_receive_messages(&self) -> bool {
        true
    }
    
    /// Gets subscribed room ID
    pub fn room_id(&self) -> RoomId {
        self.room_id.expect("Subscribed connection must have room_id")
    }
    
    /// Updates last seen message ID for reconnection support
    pub fn update_last_seen(&mut self, message_id: MessageId) {
        self.last_seen_message_id = Some(message_id);
    }
}
```

## Summary

This design document establishes complete type contracts for the Campfire Rust rewrite with:

1. **Complete Domain Models** - All structs match Rails schema exactly
2. **Comprehensive Error Hierarchy** - Every error case enumerated
3. **Service Trait Interfaces** - All methods with full documentation
4. **State Machine Safety** - Phantom types prevent invalid transitions
5. **Critical Gap Solutions** - Type-safe implementations of all 5 gaps
6. **Rails Parity Compliance** - Every interface mirrors Rails behavior

**Next Steps:**
1. Create property-based test specifications for all interfaces
2. Implement integration test contracts for service boundaries  
3. Begin type-guided implementation following these contracts

This foundation ensures compile-first success and prevents coordination complexity through the type system.