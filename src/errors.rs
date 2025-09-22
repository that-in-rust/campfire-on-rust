use thiserror::Error;
use crate::models::{UserId, RoomId, MessageId, ConnectionId};

// Library-level errors using thiserror for structured, matchable errors
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
    Database(#[from] sqlx::Error),
    
    #[error("WebSocket broadcast failed: {0}")]
    Broadcast(#[from] BroadcastError),
    
    #[error("Rate limit exceeded: {limit} messages per {window}")]
    RateLimit { limit: u32, window: String },
    
    #[error("Message not found: {message_id}")]
    NotFound { message_id: MessageId },
}

// From implementations for error conversion
impl From<DatabaseError> for MessageError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::Connection(e) => MessageError::Database(e),
            _ => MessageError::Database(sqlx::Error::Configuration("Database error".into())),
        }
    }
}

#[derive(Error, Debug)]
pub enum RoomError {
    #[error("Room not found: {room_id}")]
    NotFound { room_id: RoomId },
    
    #[error("User {user_id} already member of room {room_id}")]
    AlreadyMember { user_id: UserId, room_id: RoomId },
    
    #[error("User {user_id} not authorized to add members to room {room_id}")]
    NotAuthorized { user_id: UserId, room_id: RoomId },
    
    #[error("Invalid room name: {reason}")]
    InvalidName { reason: String },
    
    #[error("Database operation failed: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Session not found or expired")]
    SessionExpired,
    
    #[error("User not found: {email}")]
    UserNotFound { email: String },
    
    #[error("Email already exists: {email}")]
    EmailExists { email: String },
    
    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },
    
    #[error("Password too weak")]
    WeakPassword,
    
    #[error("Database operation failed: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Password hashing failed: {0}")]
    PasswordHash(#[from] bcrypt::BcryptError),
    
    #[error("Token generation failed")]
    TokenGeneration,
}

// From implementations for error conversion
impl From<DatabaseError> for AuthError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::Connection(e) => AuthError::Database(e),
            _ => AuthError::Database(sqlx::Error::Configuration("Database error".into())),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Connection not found: {connection_id}")]
    NotFound { connection_id: ConnectionId },
    
    #[error("WebSocket send failed: {reason}")]
    SendFailed { reason: String },
    
    #[error("User {user_id} not found")]
    UserNotFound { user_id: UserId },
    
    #[error("Connection already exists for user {user_id}")]
    AlreadyConnected { user_id: UserId },
    
    #[error("WebSocket protocol error: {0}")]
    Protocol(String),
}

#[derive(Error, Debug)]
pub enum BroadcastError {
    #[error("No connections found for room {room_id}")]
    NoConnections { room_id: RoomId },
    
    #[error("Failed to serialize message: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("WebSocket send failed to {connection_count} connections")]
    PartialFailure { connection_count: usize },
    
    #[error("Room not found: {room_id}")]
    RoomNotFound { room_id: RoomId },
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    Connection(#[from] sqlx::Error),
    
    #[error("Migration failed: {reason}")]
    Migration { reason: String },
    
    #[error("Transaction failed: {reason}")]
    Transaction { reason: String },
    
    #[error("Constraint violation: {constraint}")]
    ConstraintViolation { constraint: String },
    
    #[error("Data integrity error: {reason}")]
    DataIntegrity { reason: String },
    
    #[error("UUID parsing error: {0}")]
    UuidParse(#[from] uuid::Error),
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid content length: must be between 1 and 10000 characters")]
    InvalidContentLength,
    
    #[error("Invalid room name: must be between 1 and 100 characters")]
    InvalidRoomName,
    
    #[error("Invalid user name: must be between 1 and 50 characters")]
    InvalidUserName,
    
    #[error("Invalid email format")]
    InvalidEmailFormat,
    
    #[error("HTML sanitization failed: {reason}")]
    HtmlSanitization { reason: String },
    
    #[error("Required field missing: {field}")]
    RequiredField { field: String },
}

// Application-level result type
pub type Result<T> = std::result::Result<T, anyhow::Error>;

// Conversion implementations for HTTP responses
impl From<MessageError> for axum::http::StatusCode {
    fn from(err: MessageError) -> Self {
        match err {
            MessageError::Authorization { .. } => axum::http::StatusCode::FORBIDDEN,
            MessageError::InvalidContent { .. } 
            | MessageError::ContentTooLong { .. }
            | MessageError::ContentTooShort => axum::http::StatusCode::BAD_REQUEST,
            MessageError::NotFound { .. } => axum::http::StatusCode::NOT_FOUND,
            MessageError::RateLimit { .. } => axum::http::StatusCode::TOO_MANY_REQUESTS,
            MessageError::Database(_) | MessageError::Broadcast(_) => {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl From<RoomError> for axum::http::StatusCode {
    fn from(err: RoomError) -> Self {
        match err {
            RoomError::NotFound { .. } => axum::http::StatusCode::NOT_FOUND,
            RoomError::NotAuthorized { .. } => axum::http::StatusCode::FORBIDDEN,
            RoomError::AlreadyMember { .. } => axum::http::StatusCode::CONFLICT,
            RoomError::InvalidName { .. } => axum::http::StatusCode::BAD_REQUEST,
            RoomError::Database(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<AuthError> for axum::http::StatusCode {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials 
            | AuthError::SessionExpired => axum::http::StatusCode::UNAUTHORIZED,
            AuthError::UserNotFound { .. } => axum::http::StatusCode::NOT_FOUND,
            AuthError::EmailExists { .. } => axum::http::StatusCode::CONFLICT,
            AuthError::InvalidEmail { .. } 
            | AuthError::WeakPassword => axum::http::StatusCode::BAD_REQUEST,
            AuthError::Database(_) 
            | AuthError::PasswordHash(_) 
            | AuthError::TokenGeneration => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}