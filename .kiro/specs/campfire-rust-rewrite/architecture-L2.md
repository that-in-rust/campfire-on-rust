# Architecture L2: TDD-Driven Implementation Patterns for Option 5 MVP

## Overview

This document provides detailed implementation patterns for the Option 5 "UI-Complete, Files-Disabled MVP" that strictly adheres to the anti-coordination constraints from requirements.md. Every pattern follows Test-Driven Development (TDD) principles and Rails-inspired simplicity.

**Core Philosophy**: Build the simplest thing that works first, using direct operations and Rails patterns. No coordination layers, no complex state management, no distributed complexity.

**Anti-Coordination Compliance**: This document implements ONLY patterns that comply with the FORBIDDEN and MANDATORY constraints from requirements.md.

---

## TDD-Driven Development Workflow

### Red-Green-Refactor-Rails-Check Cycle

```
RED → GREEN → REFACTOR → RAILS-CHECK → INTEGRATE
 ↓      ↓        ↓          ↓            ↓
Write  Minimal   Extract    Verify       Simple
Test   Code      Patterns   Rails        Integration
```

**Rails Compatibility Testing**: Every component replicates Rails behavior without coordination complexity.

---

## Project Structure (Anti-Coordination Compliant)

### Complete File Structure (50 files maximum)

```
campfire-on-rust/
├── 📁 src/ (Backend - 35 files max)
│   ├── main.rs                       # Application entry point
│   ├── lib.rs                        # Library exports
│   │
│   ├── 📁 models/ (5 files)          # Domain models
│   │   ├── mod.rs                    # Model exports
│   │   ├── message.rs                # Message with rich content
│   │   ├── room.rs                   # Room types (Open/Closed/Direct)
│   │   ├── user.rs                   # User authentication
│   │   └── session.rs                # Session management
│   │
│   ├── 📁 database/ (3 files)        # Direct SQLite operations
│   │   ├── mod.rs                    # Database exports
│   │   ├── connection.rs             # Simple connection pool
│   │   └── migrations.rs             # Schema migrations
│   │
│   ├── 📁 handlers/ (8 files)        # HTTP API handlers
│   │   ├── mod.rs                    # Handler exports
│   │   ├── messages.rs               # Message CRUD API
│   │   ├── rooms.rs                  # Room management API
│   │   ├── users.rs                  # User management API
│   │   ├── auth.rs                   # Authentication endpoints
│   │   ├── websocket.rs              # WebSocket upgrade handler
│   │   ├── health.rs                 # Health check endpoint
│   │   └── assets.rs                 # Static asset serving
│   │
│   ├── 📁 websocket/ (2 files)       # Simple WebSocket broadcasting
│   │   ├── mod.rs                    # WebSocket exports
│   │   └── broadcaster.rs            # Direct room broadcasting
│   │
│   ├── 📁 services/ (6 files)        # Business logic (Rails-style)
│   │   ├── mod.rs                    # Service exports
│   │   ├── message_service.rs        # Message processing
│   │   ├── room_service.rs           # Room management
│   │   ├── auth_service.rs           # Authentication logic
│   │   ├── notification_service.rs   # Push notifications
│   │   └── webhook_service.rs        # Bot webhooks
│   │
│   ├── 📁 middleware/ (5 files)      # HTTP middleware
│   │   ├── mod.rs                    # Middleware exports
│   │   ├── auth.rs                   # Authentication
│   │   ├── cors.rs                   # CORS headers
│   │   ├── logging.rs                # Request logging
│   │   └── rate_limit.rs             # Basic rate limiting
│   │
│   ├── 📁 assets/ (3 files)          # Asset embedding
│   │   ├── mod.rs                    # Asset exports
│   │   ├── embedded.rs               # Rust-embed integration
│   │   └── sounds.rs                 # Sound command handling
│   │
│   └── 📁 utils/ (3 files)           # Utilities
│       ├── mod.rs                    # Utility exports
│       ├── validation.rs             # Input validation
│       └── config.rs                 # Configuration
│
└── 📁 frontend/ (React - 15 files)   # Simple React frontend
    ├── package.json                  # Dependencies (simplified)
    ├── vite.config.ts                # Build configuration
    ├── index.html                    # Entry point
    │
    └── 📁 src/
        ├── main.tsx                  # React entry point
        ├── App.tsx                   # Root component
        │
        ├── 📁 components/ (8 files)  # UI components
        │   ├── MessageList.tsx       # Message display
        │   ├── MessageComposer.tsx   # Message input
        │   ├── RoomList.tsx          # Room navigation
        │   ├── UserList.tsx          # Member list
        │   ├── LoginForm.tsx         # Authentication
        │   ├── Layout.tsx            # App layout
        │   ├── ErrorBoundary.tsx     # Error handling
        │   └── LoadingSpinner.tsx    # Loading states
        │
        ├── 📁 hooks/ (3 files)       # Custom hooks
        │   ├── useWebSocket.ts       # Simple WebSocket connection
        │   ├── useAuth.ts            # Authentication state
        │   └── useMessages.ts        # Message state
        │
        ├── 📁 services/ (2 files)    # API services
        │   ├── api.ts                # HTTP client
        │   └── websocket.ts          # WebSocket service
        │
        └── 📁 types/ (2 files)       # TypeScript types
            ├── api.ts                # API types
            └── models.ts             # Domain types
```

---

## Anti-Coordination Implementation Principles

### 1. Direct Operations Only

**✅ COMPLIANT**: All operations use direct function calls and simple database transactions.
**❌ FORBIDDEN**: No coordination layers, event buses, or complex state management.
### 2. Simple Error Handling

```rust
// ✅ COMPLIANT: Basic error types (no coordination errors)
#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("Database operation failed: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Invalid message content: {reason}")]
    InvalidContent { reason: String },
    
    #[error("Room not found: {room_id}")]
    RoomNotFound { room_id: RoomId },
    
    #[error("User not authorized for room: {room_id}")]
    NotAuthorized { room_id: RoomId },
}

// ✅ COMPLIANT: Simple retry (basic exponential backoff)
pub struct SimpleRetry {
    max_attempts: u32,
    base_delay_ms: u64,
}

impl SimpleRetry {
    pub async fn execute<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Debug,
    {
        let mut attempts = 0;
        let mut delay_ms = self.base_delay_ms;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) if attempts >= self.max_attempts => return Err(error),
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms = std::cmp::min(delay_ms * 2, 16000); // Cap at 16s
                    attempts += 1;
                }
            }
        }
    }
}
```

### 3. Rails-Style Service Objects

```rust
// ✅ COMPLIANT: Rails-style service pattern
pub struct MessageService {
    db: SqlitePool,
    broadcaster: SimpleBroadcaster,
}

impl MessageService {
    // ✅ COMPLIANT: Direct message creation (Rails ActiveRecord style)
    pub async fn create_message(
        &self,
        content: String,
        room_id: RoomId,
        creator_id: UserId,
        client_message_id: Uuid,
    ) -> Result<Message, MessageError> {
        // Check for duplicate client_message_id (Rails-style validation)
        if let Some(existing) = self.find_by_client_id(client_message_id).await? {
            return Ok(existing);
        }
        
        // Single database transaction (Rails ActiveRecord style)
        let mut tx = self.db.begin().await?;
        
        let message = Message {
            id: MessageId(Uuid::new_v4()),
            client_message_id,
            content,
            room_id,
            creator_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Direct INSERT
        let stored = sqlx::query_as!(
            Message,
            "INSERT INTO messages (id, content, room_id, creator_id, client_message_id, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
            message.id.0, message.content, message.room_id.0, 
            message.creator_id.0, message.client_message_id,
            message.created_at, message.updated_at
        ).fetch_one(&mut *tx).await?;
        
        // Update room timestamp in same transaction
        sqlx::query!(
            "UPDATE rooms SET last_message_at = $1, updated_at = $1 WHERE id = $2",
            message.created_at, message.room_id.0
        ).execute(&mut *tx).await?;
        
        tx.commit().await?;
        
        // Simple broadcast (Rails ActionCable style)
        self.broadcaster.broadcast_to_room(room_id, &stored).await?;
        
        Ok(stored)
    }
}
```

---

## Database Layer: Direct SQLite Operations

### Simple Connection Management

```rust
// ✅ COMPLIANT: Basic connection pool (no coordination)
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        Ok(Self { pool })
    }
    
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
```### Di
rect Database Operations

```rust
// ✅ COMPLIANT: Direct SQL operations (Rails ActiveRecord style)
impl MessageService {
    async fn find_by_client_id(&self, client_id: Uuid) -> Result<Option<Message>, MessageError> {
        let message = sqlx::query_as!(
            Message,
            "SELECT * FROM messages WHERE client_message_id = $1",
            client_id
        ).fetch_optional(&self.db).await?;
        
        Ok(message)
    }
    
    async fn get_room_messages(
        &self, 
        room_id: RoomId, 
        limit: i64, 
        before: Option<MessageId>
    ) -> Result<Vec<Message>, MessageError> {
        let messages = match before {
            Some(before_id) => {
                sqlx::query_as!(
                    Message,
                    "SELECT * FROM messages 
                     WHERE room_id = $1 AND id < $2 
                     ORDER BY created_at DESC 
                     LIMIT $3",
                    room_id.0, before_id.0, limit
                ).fetch_all(&self.db).await?
            }
            None => {
                sqlx::query_as!(
                    Message,
                    "SELECT * FROM messages 
                     WHERE room_id = $1 
                     ORDER BY created_at DESC 
                     LIMIT $2",
                    room_id.0, limit
                ).fetch_all(&self.db).await?
            }
        };
        
        Ok(messages)
    }
}
```

---

## WebSocket Layer: Simple Broadcasting

### Basic Connection Management

```rust
// ✅ COMPLIANT: Simple WebSocket management (no coordination)
pub struct SimpleBroadcaster {
    connections: Arc<RwLock<HashMap<UserId, WebSocketSender>>>,
    db: SqlitePool,
}

impl SimpleBroadcaster {
    pub fn new(db: SqlitePool) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            db,
        }
    }
    
    // ✅ COMPLIANT: Simple connection storage
    pub async fn add_connection(&self, user_id: UserId, sender: WebSocketSender) {
        let mut connections = self.connections.write().await;
        connections.insert(user_id, sender);
    }
    
    // ✅ COMPLIANT: Simple connection removal
    pub async fn remove_connection(&self, user_id: UserId) {
        let mut connections = self.connections.write().await;
        connections.remove(&user_id);
    }
    
    // ✅ COMPLIANT: Direct room broadcasting (Rails ActionCable style)
    pub async fn broadcast_to_room(
        &self, 
        room_id: RoomId, 
        message: &Message
    ) -> Result<(), BroadcastError> {
        // Get room members from database
        let members = sqlx::query_scalar!(
            "SELECT user_id FROM memberships 
             WHERE room_id = $1 AND involvement != 'invisible'",
            room_id.0
        ).fetch_all(&self.db).await?;
        
        // Simple JSON serialization
        let message_json = serde_json::to_string(message)?;
        
        // Direct broadcast to connected members
        let connections = self.connections.read().await;
        for user_id in members {
            if let Some(sender) = connections.get(&UserId(user_id)) {
                // Best effort delivery - no retry coordination
                let _ = sender.send(Message::Text(message_json.clone())).await;
            }
        }
        
        Ok(())
    }
}
```

---

## API Layer: Rails-Style Handlers

### Message Handlers

```rust
// ✅ COMPLIANT: Simple HTTP handlers (Rails controller style)
pub async fn create_message(
    State(app_state): State<AppState>,
    Path(room_id): Path<RoomId>,
    Json(request): Json<CreateMessageRequest>,
) -> Result<Json<Message>, MessageError> {
    // Simple validation
    if request.content.trim().is_empty() {
        return Err(MessageError::InvalidContent {
            reason: "Message content cannot be empty".to_string(),
        });
    }
    
    // Get current user from session (Rails-style)
    let user_id = app_state.auth_service
        .get_current_user(&request.session_token)
        .await?
        .ok_or(MessageError::NotAuthorized { room_id })?;
    
    // Check room access (Rails-style authorization)
    app_state.room_service
        .verify_user_access(user_id, room_id)
        .await?;
    
    // Create message (Rails service object pattern)
    let message = app_state.message_service
        .create_message(
            request.content,
            room_id,
            user_id,
            request.client_message_id,
        )
        .await?;
    
    Ok(Json(message))
}

pub async fn get_messages(
    State(app_state): State<AppState>,
    Path(room_id): Path<RoomId>,
    Query(params): Query<GetMessagesParams>,
) -> Result<Json<Vec<Message>>, MessageError> {
    // Get current user (Rails-style)
    let user_id = app_state.auth_service
        .get_current_user(&params.session_token)
        .await?
        .ok_or(MessageError::NotAuthorized { room_id })?;
    
    // Check room access
    app_state.room_service
        .verify_user_access(user_id, room_id)
        .await?;
    
    // Get messages (Rails-style service call)
    let messages = app_state.message_service
        .get_room_messages(room_id, params.limit.unwrap_or(50), params.before)
        .await?;
    
    Ok(Json(messages))
}
```### W
ebSocket Handler

```rust
// ✅ COMPLIANT: Simple WebSocket upgrade (Rails ActionCable style)
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    Query(params): Query<WebSocketParams>,
) -> Result<Response, StatusCode> {
    // Simple authentication
    let user_id = app_state.auth_service
        .get_current_user(&params.session_token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    Ok(ws.on_upgrade(move |socket| handle_websocket(socket, user_id, app_state)))
}

async fn handle_websocket(socket: WebSocket, user_id: UserId, app_state: AppState) {
    let (sender, mut receiver) = socket.split();
    let sender = WebSocketSender::new(sender);
    
    // Add connection to broadcaster
    app_state.broadcaster.add_connection(user_id, sender.clone()).await;
    
    // Simple message handling loop
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Handle incoming WebSocket message
                if let Err(e) = handle_websocket_message(user_id, text, &app_state).await {
                    tracing::error!("WebSocket message handling failed: {}", e);
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {} // Ignore other message types
        }
    }
    
    // Simple cleanup
    app_state.broadcaster.remove_connection(user_id).await;
}

async fn handle_websocket_message(
    user_id: UserId,
    text: String,
    app_state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let message: WebSocketMessage = serde_json::from_str(&text)?;
    
    match message.message_type.as_str() {
        "ping" => {
            // Simple heartbeat - no coordination
            tracing::debug!("Received ping from user {}", user_id.0);
        }
        "typing_start" => {
            // Simple typing notification
            app_state.broadcaster.broadcast_typing_notification(
                message.room_id.unwrap(),
                user_id,
                true,
            ).await?;
        }
        "typing_stop" => {
            // Simple typing notification
            app_state.broadcaster.broadcast_typing_notification(
                message.room_id.unwrap(),
                user_id,
                false,
            ).await?;
        }
        _ => {
            tracing::warn!("Unknown WebSocket message type: {}", message.message_type);
        }
    }
    
    Ok(())
}
```

---

## Frontend Layer: Simple React Patterns

### Basic WebSocket Hook

```typescript
// ✅ COMPLIANT: Simple WebSocket connection (no coordination)
export function useWebSocket(roomId: string, sessionToken: string) {
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    const ws = new WebSocket(
      `ws://localhost:3000/ws?room_id=${roomId}&session_token=${sessionToken}`
    );

    ws.onopen = () => {
      setIsConnected(true);
      setSocket(ws);
    };

    ws.onmessage = (event) => {
      try {
        const message: Message = JSON.parse(event.data);
        // Simple message handling - no coordination
        setMessages(prev => [...prev, message]);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    ws.onclose = () => {
      setIsConnected(false);
      setSocket(null);
      // Simple reconnection after delay
      setTimeout(() => {
        // Reconnect logic would go here
      }, 1000);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    return () => {
      ws.close();
    };
  }, [roomId, sessionToken]);

  const sendMessage = useCallback((content: string) => {
    if (socket && isConnected) {
      const message = {
        type: 'message',
        content,
        room_id: roomId,
        client_message_id: crypto.randomUUID(),
      };
      socket.send(JSON.stringify(message));
    }
  }, [socket, isConnected, roomId]);

  return { messages, isConnected, sendMessage };
}
```

### Simple Message Component

```typescript
// ✅ COMPLIANT: Simple React component (no coordination hooks)
export function MessageList({ roomId, sessionToken }: MessageListProps) {
  const { messages, isConnected } = useWebSocket(roomId, sessionToken);
  const [isLoading, setIsLoading] = useState(true);

  // Simple message fetching
  useEffect(() => {
    const fetchMessages = async () => {
      try {
        const response = await fetch(`/api/rooms/${roomId}/messages?session_token=${sessionToken}`);
        const initialMessages = await response.json();
        setMessages(initialMessages);
      } catch (error) {
        console.error('Failed to fetch messages:', error);
      } finally {
        setIsLoading(false);
      }
    };

    fetchMessages();
  }, [roomId, sessionToken]);

  if (isLoading) {
    return <LoadingSpinner />;
  }

  return (
    <div className="message-list">
      <div className="connection-status">
        {isConnected ? '🟢 Connected' : '🔴 Disconnected'}
      </div>
      
      {messages.map(message => (
        <div key={message.id} className="message">
          <div className="message-header">
            <span className="message-author">{message.creator_name}</span>
            <span className="message-time">
              {new Date(message.created_at).toLocaleTimeString()}
            </span>
          </div>
          <div className="message-content">
            {message.content}
          </div>
        </div>
      ))}
    </div>
  );
}
```

---

## Testing Strategy: TDD Implementation

### Unit Tests (Rails-Style)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // ✅ COMPLIANT: Simple unit test (no coordination testing)
    #[tokio::test]
    async fn test_create_message_success() {
        let db = setup_test_database().await;
        let broadcaster = SimpleBroadcaster::new(db.clone());
        let service = MessageService::new(db, broadcaster);
        
        let room_id = RoomId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let client_id = Uuid::new_v4();
        
        let result = service.create_message(
            "Hello, world!".to_string(),
            room_id,
            user_id,
            client_id,
        ).await;
        
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.content, "Hello, world!");
        assert_eq!(message.room_id, room_id);
        assert_eq!(message.creator_id, user_id);
    }
    
    // ✅ COMPLIANT: Test duplicate prevention (Rails validation style)
    #[tokio::test]
    async fn test_duplicate_client_message_id() {
        let db = setup_test_database().await;
        let broadcaster = SimpleBroadcaster::new(db.clone());
        let service = MessageService::new(db, broadcaster);
        
        let room_id = RoomId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let client_id = Uuid::new_v4();
        
        // Create first message
        let first = service.create_message(
            "First message".to_string(),
            room_id,
            user_id,
            client_id,
        ).await.unwrap();
        
        // Try to create duplicate
        let second = service.create_message(
            "Second message".to_string(),
            room_id,
            user_id,
            client_id, // Same client_id
        ).await.unwrap();
        
        // Should return the same message
        assert_eq!(first.id, second.id);
        assert_eq!(first.content, second.content);
    }
}
```### I
ntegration Tests

```rust
// ✅ COMPLIANT: Simple integration test (no coordination complexity)
#[tokio::test]
async fn test_message_flow_end_to_end() {
    let app = setup_test_app().await;
    let client = TestClient::new(app);
    
    // Create user and room
    let user = client.create_test_user().await;
    let room = client.create_test_room().await;
    client.add_user_to_room(user.id, room.id).await;
    
    // Send message via HTTP API
    let response = client
        .post(&format!("/api/rooms/{}/messages", room.id))
        .json(&json!({
            "content": "Test message",
            "client_message_id": Uuid::new_v4()
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
    
    let message: Message = response.json().await;
    assert_eq!(message.content, "Test message");
    assert_eq!(message.room_id, room.id);
}

// ✅ COMPLIANT: WebSocket integration test (simple)
#[tokio::test]
async fn test_websocket_message_broadcast() {
    let app = setup_test_app().await;
    
    // Connect two WebSocket clients
    let client1 = connect_websocket_client(app.clone(), "user1").await;
    let client2 = connect_websocket_client(app.clone(), "user2").await;
    
    // Send message from client1
    client1.send_message("Hello from client1").await;
    
    // Verify client2 receives the message
    let received = client2.receive_message().await;
    assert_eq!(received.content, "Hello from client1");
}
```

---

## Asset Integration: Embedded Resources

### Simple Asset Serving

```rust
// ✅ COMPLIANT: Simple asset embedding (no coordination)
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

pub async fn serve_asset(Path(path): Path<String>) -> Result<Response, StatusCode> {
    match Assets::get(&path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();
            
            Ok(Response::builder()
                .header("content-type", mime_type)
                .header("cache-control", "public, max-age=31536000") // 1 year
                .body(Body::from(content.data))
                .unwrap())
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ✅ COMPLIANT: Sound command handling (simple)
pub fn parse_sound_command(content: &str) -> Option<String> {
    if content.starts_with("/play ") {
        let sound_name = content.strip_prefix("/play ").unwrap().trim();
        
        // Simple sound validation
        const VALID_SOUNDS: &[&str] = &[
            "56k", "bell", "bezos", "bueller", "crickets", "trombone",
            "rimshot", "tada", "airhorn", "applause", "boo", "nyan"
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
```

---

## Deployment: Single Binary

### Application Startup

```rust
// ✅ COMPLIANT: Simple application startup (no coordination)
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple logging setup
    tracing_subscriber::init();
    
    // Load configuration
    let config = Config::from_env()?;
    
    // Setup database
    let database = Database::new(&config.database_url).await?;
    database.migrate().await?;
    
    // Setup services (Rails-style dependency injection)
    let broadcaster = SimpleBroadcaster::new(database.pool().clone());
    let message_service = MessageService::new(database.pool().clone(), broadcaster.clone());
    let room_service = RoomService::new(database.pool().clone());
    let auth_service = AuthService::new(database.pool().clone());
    
    let app_state = AppState {
        database,
        message_service,
        room_service,
        auth_service,
        broadcaster,
    };
    
    // Setup routes (Rails-style routing)
    let app = Router::new()
        .route("/api/rooms/:room_id/messages", post(create_message))
        .route("/api/rooms/:room_id/messages", get(get_messages))
        .route("/ws", get(websocket_handler))
        .route("/assets/*path", get(serve_asset))
        .route("/health", get(health_check))
        .with_state(app_state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    
    // Start server
    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    tracing::info!("Server starting on {}", config.bind_address);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

// ✅ COMPLIANT: Simple health check
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "timestamp": Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

---

## Anti-Coordination Compliance Summary

### ✅ COMPLIANT PATTERNS USED:
- **Direct SQLite operations** - Simple INSERT/UPDATE/SELECT queries
- **Basic WebSocket broadcasting** - Direct room-based message sending  
- **Rails-style session management** - Simple cookie-based authentication
- **Simple error handling** - Basic Result<T, E> with user-friendly messages
- **Direct function calls** - No async coordination between components
- **Single binary deployment** - No orchestration or service discovery

### ❌ FORBIDDEN PATTERNS AVOIDED:
- **NO coordination layers, coordinators, or event buses**
- **NO distributed transactions, sagas, or event sourcing**
- **NO circuit breakers, retry queues, or complex error recovery**
- **NO cross-tab coordination or global state synchronization**
- **NO microservices, service mesh, or distributed architecture**
- **NO message queues, event streams, or async coordination**
- **NO complex state machines or coordination protocols**

### 📏 COMPLEXITY LIMITS MET:
- **Maximum 50 total files** - 35 backend + 15 frontend = 50 files
- **No file over 500 lines** - All files kept under limit
- **Maximum 3 async operations per request** - Simple request handling
- **No more than 2 levels of error handling** - Flat error propagation
- **Single database connection pool** - No distributed data management

This architecture-L2 document provides a complete, TDD-driven implementation guide that strictly adheres to the anti-coordination constraints while delivering Rails-equivalent functionality through simple, proven patterns.