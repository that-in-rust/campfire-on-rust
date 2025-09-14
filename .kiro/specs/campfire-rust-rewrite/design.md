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

### 1. Database Layer

**Design Approach**: Direct SQLite operations with sqlx, no ORM or coordination layer.

```rust
// Connection management
pub struct Database {
    pool: sqlx::SqlitePool,
}

// Direct query patterns
impl Database {
    pub async fn create_message(&self, message: &NewMessage) -> Result<Message, sqlx::Error> {
        sqlx::query_as!(
            Message,
            "INSERT INTO messages (room_id, user_id, body, client_message_id) 
             VALUES (?, ?, ?, ?) RETURNING *",
            message.room_id,
            message.user_id, 
            message.body,
            message.client_message_id
        )
        .fetch_one(&self.pool)
        .await
    }
}
```

**Key Design Decisions**:
- SQLite with WAL mode for basic concurrency
- sqlx for compile-time SQL validation
- Direct queries, no query builder complexity
- Connection pooling for efficiency
- Dedicated Writer Task pattern for write serialization

### 2. HTTP API Layer

**Design Approach**: Axum handlers with Rails-style routing, direct database operations.

```rust
// Message creation handler
pub async fn create_message(
    State(app_state): State<AppState>,
    Path(room_id): Path<RoomId>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<Json<Message>, ApiError> {
    // Direct database operation
    let message = app_state.db.create_message(&NewMessage {
        room_id,
        user_id: current_user.id,
        body: payload.body,
        client_message_id: payload.client_message_id,
    }).await?;
    
    // Direct WebSocket broadcast
    app_state.broadcaster.broadcast_to_room(room_id, &message).await;
    
    Ok(Json(message))
}
```

**Key Design Decisions**:
- RESTful API design matching Rails conventions
- Direct database calls in handlers
- Simple error handling with user-friendly messages
- Session-based authentication
- No complex middleware chains

### 3. WebSocket Broadcasting

**Design Approach**: Simple room-based broadcasting like Rails ActionCable, no coordination complexity.

```rust
pub struct WebSocketBroadcaster {
    connections: Arc<RwLock<HashMap<RoomId, Vec<WebSocketSender>>>>,
}

impl WebSocketBroadcaster {
    pub async fn broadcast_to_room(&self, room_id: RoomId, message: &Message) {
        let connections = self.connections.read().await;
        if let Some(room_connections) = connections.get(&room_id) {
            for sender in room_connections {
                // Best effort delivery - no retry or coordination
                let _ = sender.send(serde_json::to_string(message).unwrap()).await;
            }
        }
    }
}
```

**Key Design Decisions**:
- Simple HashMap for connection tracking (Rails-style)
- Room-based broadcasting only (like ActionCable)
- No global message ordering or coordination
- Direct WebSocket sends to connected clients
- Basic connection cleanup on disconnect

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

### 5. Background Task Processing

**Design Approach**: Simple tokio::spawn for basic async tasks, no complex job queues.

```rust
pub async fn deliver_webhook(webhook_url: String, payload: WebhookPayload) {
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let result = client
            .post(&webhook_url)
            .json(&payload)
            .timeout(Duration::from_secs(7))
            .send()
            .await;
            
        match result {
            Ok(response) => {
                // Handle response simply
                if let Ok(text) = response.text().await {
                    // Process webhook response
                }
            }
            Err(e) => {
                // Log error, no complex retry logic
                eprintln!("Webhook delivery failed: {}", e);
            }
        }
    });
}
```

**Key Design Decisions**:
- tokio::spawn for simple background tasks
- No message queues or complex job systems
- Basic error handling and logging
- Simple timeout handling
- No complex retry mechanisms#
# Data Models

### Core Domain Models

```rust
// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub bot_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Room model with STI pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub room_type: RoomType, // Open, Closed, Direct
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Message model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub room_id: RoomId,
    pub user_id: UserId,
    pub body: String,
    pub client_message_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Type Safety with Newtypes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub i64);
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

This design provides a clear, implementable path to building the Campfire Rust Rewrite MVP while strictly adhering to the anti-coordination constraints and Rails-inspired simplicity mandated in the requirements.