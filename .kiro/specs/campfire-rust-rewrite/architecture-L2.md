# Architecture L2: TDD-Driven Code Patterns for Campfire MVP

## Overview

This document provides a comprehensive L2 (Implementation Layer) architecture that maps each requirement to specific Test-Driven Development patterns using idiomatic Rust and React code structures. The entire application is designed as a collection of well-tested, composable code patterns that ensure compile-first success and maintainable, bug-free code.

**Core Philosophy**: Every feature is built using the Red-Green-Refactor TDD cycle, with comprehensive pattern libraries ensuring consistent, idiomatic implementation across the entire codebase.

---

## Table of Contents

1. [TDD Architecture Principles](#tdd-architecture-principles)
2. [Rust Backend Pattern Mapping](#rust-backend-pattern-mapping)
3. [React Frontend Pattern Mapping](#react-frontend-pattern-mapping)
4. [Cross-Cutting Pattern Integration](#cross-cutting-pattern-integration)
5. [Testing Strategy and Implementation](#testing-strategy-and-implementation)
6. [Feature Flag Pattern Architecture](#feature-flag-pattern-architecture)
7. [Performance Pattern Implementation](#performance-pattern-implementation)

---

## TDD Architecture Principles

### 1. Test-First Development Workflow

Every component follows the strict TDD cycle:

```
RED → GREEN → REFACTOR → INTEGRATE
 ↓      ↓        ↓         ↓
Write  Minimal   Extract   Pattern
Test   Code      Patterns  Library
```

#### 1.1 Rust TDD Pattern
```rust
// Step 1: RED - Write failing test
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_message_success() {
        // Arrange
        let mut mock_db = MockDatabase::new();
        mock_db.expect_create_message()
            .returning(|msg| Ok(msg.clone()));
        
        let service = MessageService::new(mock_db);
        
        // Act
        let result = service.create_message(
            "Hello world".to_string(),
            RoomId(Uuid::new_v4()),
            UserId(Uuid::new_v4())
        ).await;
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello world");
    }
}

// Step 2: GREEN - Minimal implementation
pub struct MessageService<D: Database> {
    db: D,
}

impl<D: Database> MessageService<D> {
    pub async fn create_message(
        &self,
        content: String,
        room_id: RoomId,
        creator_id: UserId,
    ) -> Result<Message, MessageError> {
        let message = Message {
            id: MessageId(Uuid::new_v4()),
            content,
            room_id,
            creator_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.db.create_message(&message).await
    }
}

// Step 3: REFACTOR - Extract patterns
// Move to pattern library for reuse
```

#### 1.2 React TDD Pattern
```jsx
// Step 1: RED - Write failing test
import { render, screen, userEvent } from '@testing-library/react';
import { MessageComposer } from './MessageComposer';

test('sends message when form is submitted', async () => {
  const mockSendMessage = jest.fn().mockResolvedValue({ id: '123' });
  const user = userEvent.setup();
  
  render(<MessageComposer onSend={mockSendMessage} />);
  
  const textarea = screen.getByRole('textbox');
  const submitButton = screen.getByRole('button', { name: /send/i });
  
  await user.type(textarea, 'Hello, world!');
  await user.click(submitButton);
  
  expect(mockSendMessage).toHaveBeenCalledWith('Hello, world!');
  expect(textarea).toHaveValue(''); // Should clear after send
});

// Step 2: GREEN - Minimal implementation
function MessageComposer({ onSend }) {
  const [content, setContent] = useState('');
  
  const handleSubmit = async (e) => {
    e.preventDefault();
    await onSend(content);
    setContent('');
  };
  
  return (
    <form onSubmit={handleSubmit}>
      <textarea 
        value={content}
        onChange={(e) => setContent(e.target.value)}
      />
      <button type="submit">Send</button>
    </form>
  );
}

// Step 3: REFACTOR - Extract custom hook pattern
function useMessageComposer(onSend) {
  const [content, setContent] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  
  const handleSubmit = useCallback(async (e) => {
    e.preventDefault();
    if (!content.trim()) return;
    
    try {
      setIsSubmitting(true);
      await onSend(content);
      setContent('');
    } finally {
      setIsSubmitting(false);
    }
  }, [content, onSend]);
  
  return { content, setContent, handleSubmit, isSubmitting };
}
```

---

## Rust Backend Pattern Mapping

### Requirement 1: Rich Text Message System → Actor Pattern + Type Safety

#### 1.1 Message Domain Types (Newtype Pattern)
```rust
// Test-driven domain types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

// Message state machine (Typestate Pattern)
#[derive(Debug, Clone)]
pub enum MessageState {
    Draft { client_id: Uuid },
    Pending { client_id: Uuid, timestamp: DateTime<Utc> },
    Sent { id: MessageId, timestamp: DateTime<Utc> },
    Failed { error: String, retry_count: u8 },
}

// Rich content processing (Zero-cost abstractions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichContent {
    pub html: String,
    pub plain_text: String,
    pub mentions: Vec<UserId>,
    pub sound_commands: Vec<SoundCommand>,
}

impl RichContent {
    pub fn from_input(content: &str) -> Self {
        let mentions = Self::extract_mentions(content);
        let sound_commands = Self::extract_sound_commands(content);
        let html = Self::process_html(content);
        let plain_text = Self::strip_html(&html);
        
        Self { html, plain_text, mentions, sound_commands }
    }
    
    fn extract_mentions(content: &str) -> Vec<UserId> {
        content
            .split_whitespace()
            .filter_map(|word| {
                word.strip_prefix('@')
                    .and_then(|username| resolve_username_to_id(username))
            })
            .collect()
    }
    
    fn extract_sound_commands(content: &str) -> Vec<SoundCommand> {
        if let Some(sound_name) = content.strip_prefix("/play ") {
            if let Some(command) = SoundCommand::from_name(sound_name) {
                return vec![command];
            }
        }
        vec![]
    }
}

// TDD Test Suite
#[cfg(test)]
mod message_tests {
    use super::*;
    
    #[test]
    fn test_rich_content_extracts_mentions() {
        let content = "Hello @alice and @bob!";
        let rich = RichContent::from_input(content);
        
        assert_eq!(rich.mentions.len(), 2);
        assert!(rich.plain_text.contains("Hello"));
    }
    
    #[test]
    fn test_sound_command_detection() {
        let content = "/play bell";
        let rich = RichContent::from_input(content);
        
        assert_eq!(rich.sound_commands.len(), 1);
        assert_eq!(rich.sound_commands[0], SoundCommand::Bell);
    }
}
```

#### 1.2 Message Actor Pattern (Concurrency)
```rust
use tokio::sync::{mpsc, oneshot};

// Message processing actor
pub struct MessageActor {
    receiver: mpsc::Receiver<MessageCommand>,
    db: Arc<dyn Database>,
    broadcaster: Arc<dyn MessageBroadcaster>,
}

#[derive(Debug)]
pub enum MessageCommand {
    Create {
        content: String,
        room_id: RoomId,
        creator_id: UserId,
        client_id: Uuid,
        respond_to: oneshot::Sender<Result<Message, MessageError>>,
    },
    Update {
        id: MessageId,
        content: String,
        respond_to: oneshot::Sender<Result<Message, MessageError>>,
    },
    Delete {
        id: MessageId,
        user_id: UserId,
        respond_to: oneshot::Sender<Result<(), MessageError>>,
    },
}

impl MessageActor {
    pub async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                MessageCommand::Create { content, room_id, creator_id, client_id, respond_to } => {
                    let result = self.handle_create_message(content, room_id, creator_id, client_id).await;
                    let _ = respond_to.send(result);
                }
                MessageCommand::Update { id, content, respond_to } => {
                    let result = self.handle_update_message(id, content).await;
                    let _ = respond_to.send(result);
                }
                MessageCommand::Delete { id, user_id, respond_to } => {
                    let result = self.handle_delete_message(id, user_id).await;
                    let _ = respond_to.send(result);
                }
            }
        }
    }
    
    async fn handle_create_message(
        &self,
        content: String,
        room_id: RoomId,
        creator_id: UserId,
        client_id: Uuid,
    ) -> Result<Message, MessageError> {
        // Validate content
        if content.trim().is_empty() {
            return Err(MessageError::EmptyContent);
        }
        
        // Process rich content
        let rich_content = RichContent::from_input(&content);
        
        // Create message
        let message = Message {
            id: MessageId(Uuid::new_v4()),
            content: rich_content.html,
            plain_text: rich_content.plain_text,
            room_id,
            creator_id,
            client_message_id: client_id,
            mentions: rich_content.mentions,
            sound_commands: rich_content.sound_commands,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Store in database
        let stored_message = self.db.create_message(&message).await?;
        
        // Broadcast to room subscribers
        self.broadcaster.broadcast_message(&stored_message).await?;
        
        Ok(stored_message)
    }
}

// TDD Test Suite for Actor
#[cfg(test)]
mod actor_tests {
    use super::*;
    use tokio::sync::mpsc;
    
    #[tokio::test]
    async fn test_message_actor_creates_message() {
        let (tx, rx) = mpsc::channel(100);
        let mock_db = Arc::new(MockDatabase::new());
        let mock_broadcaster = Arc::new(MockBroadcaster::new());
        
        let actor = MessageActor {
            receiver: rx,
            db: mock_db.clone(),
            broadcaster: mock_broadcaster.clone(),
        };
        
        tokio::spawn(actor.run());
        
        let (respond_tx, respond_rx) = oneshot::channel();
        
        tx.send(MessageCommand::Create {
            content: "Test message".to_string(),
            room_id: RoomId(Uuid::new_v4()),
            creator_id: UserId(Uuid::new_v4()),
            client_id: Uuid::new_v4(),
            respond_to: respond_tx,
        }).await.unwrap();
        
        let result = respond_rx.await.unwrap();
        assert!(result.is_ok());
    }
}
```

### Requirement 2: Room Management → Repository Pattern + STI

#### 2.1 Room Repository Pattern
```rust
// Repository trait for testability
#[async_trait]
pub trait RoomRepository: Send + Sync {
    async fn create_room(&self, room: &Room) -> Result<Room, RoomError>;
    async fn get_room(&self, id: RoomId) -> Result<Option<Room>, RoomError>;
    async fn get_user_rooms(&self, user_id: UserId) -> Result<Vec<Room>, RoomError>;
    async fn update_room(&self, room: &Room) -> Result<Room, RoomError>;
    async fn delete_room(&self, id: RoomId) -> Result<(), RoomError>;
}

// SQLite implementation
pub struct SqliteRoomRepository {
    pool: SqlitePool,
}

#[async_trait]
impl RoomRepository for SqliteRoomRepository {
    async fn create_room(&self, room: &Room) -> Result<Room, RoomError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO rooms (id, name, room_type, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, room_type, created_by, created_at, updated_at
            "#,
            room.id.0,
            room.name,
            room.room_type.to_string(),
            room.created_by.0,
            room.created_at,
            room.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(RoomError::Database)?;
        
        Ok(Room::from_row(row))
    }
}

// Room types with Single Table Inheritance pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomType {
    Open,
    Closed { invited_users: Vec<UserId> },
    Direct { participants: [UserId; 2] },
}

impl RoomType {
    pub fn is_user_allowed(&self, user_id: UserId) -> bool {
        match self {
            RoomType::Open => true,
            RoomType::Closed { invited_users } => invited_users.contains(&user_id),
            RoomType::Direct { participants } => participants.contains(&user_id),
        }
    }
}

// TDD Test Suite
#[cfg(test)]
mod room_repository_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_open_room() {
        let repo = SqliteRoomRepository::new_in_memory().await;
        
        let room = Room {
            id: RoomId(Uuid::new_v4()),
            name: "Test Room".to_string(),
            room_type: RoomType::Open,
            created_by: UserId(Uuid::new_v4()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let result = repo.create_room(&room).await;
        assert!(result.is_ok());
        
        let created_room = result.unwrap();
        assert_eq!(created_room.name, "Test Room");
        assert!(matches!(created_room.room_type, RoomType::Open));
    }
    
    #[test]
    fn test_room_type_permissions() {
        let user1 = UserId(Uuid::new_v4());
        let user2 = UserId(Uuid::new_v4());
        
        let open_room = RoomType::Open;
        assert!(open_room.is_user_allowed(user1));
        
        let closed_room = RoomType::Closed { invited_users: vec![user1] };
        assert!(closed_room.is_user_allowed(user1));
        assert!(!closed_room.is_user_allowed(user2));
        
        let direct_room = RoomType::Direct { participants: [user1, user2] };
        assert!(direct_room.is_user_allowed(user1));
        assert!(direct_room.is_user_allowed(user2));
    }
}
```

### Requirement 3: Authentication → JWT + Session Pattern

#### 3.1 Authentication Service Pattern
```rust
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

// Authentication service with comprehensive error handling
pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    session_store: Arc<dyn SessionStore>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub exp: usize,  // Expiration
    pub iat: usize,  // Issued at
    pub role: UserRole,
    pub session_id: String,
}

impl AuthService {
    pub fn new(secret: &str, session_store: Arc<dyn SessionStore>) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            session_store,
        }
    }
    
    pub async fn authenticate_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, AuthError> {
        // Rate limiting check
        self.check_rate_limit(email).await?;
        
        // Validate credentials
        let user = self.validate_credentials(email, password).await?;
        
        // Create session
        let session = Session {
            id: SessionId(Uuid::new_v4()),
            user_id: user.id,
            created_at: Utc::now(),
            last_active_at: Utc::now(),
            ip_address: None, // Set by middleware
            user_agent: None, // Set by middleware
        };
        
        self.session_store.create_session(&session).await?;
        
        // Generate JWT
        let claims = Claims {
            sub: user.id.0.to_string(),
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            role: user.role,
            session_id: session.id.0.to_string(),
        };
        
        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(AuthError::TokenGeneration)?;
        
        Ok(AuthResponse {
            token,
            user,
            session_id: session.id,
        })
    }
    
    pub async fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(AuthError::InvalidToken)?;
        
        // Verify session is still active
        let session_id = SessionId(
            Uuid::parse_str(&token_data.claims.session_id)
                .map_err(|_| AuthError::InvalidSession)?
        );
        
        if !self.session_store.is_session_active(session_id).await? {
            return Err(AuthError::SessionExpired);
        }
        
        Ok(token_data.claims)
    }
}

// TDD Test Suite
#[cfg(test)]
mod auth_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_successful_authentication() {
        let mock_session_store = Arc::new(MockSessionStore::new());
        let auth_service = AuthService::new("test_secret", mock_session_store);
        
        let result = auth_service.authenticate_user(
            "test@example.com",
            "password123"
        ).await;
        
        assert!(result.is_ok());
        let auth_response = result.unwrap();
        assert!(!auth_response.token.is_empty());
    }
    
    #[tokio::test]
    async fn test_invalid_credentials() {
        let mock_session_store = Arc::new(MockSessionStore::new());
        let auth_service = AuthService::new("test_secret", mock_session_store);
        
        let result = auth_service.authenticate_user(
            "test@example.com",
            "wrong_password"
        ).await;
        
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }
    
    #[tokio::test]
    async fn test_token_validation() {
        let mock_session_store = Arc::new(MockSessionStore::new());
        let auth_service = AuthService::new("test_secret", mock_session_store);
        
        // First authenticate to get a token
        let auth_response = auth_service.authenticate_user(
            "test@example.com",
            "password123"
        ).await.unwrap();
        
        // Then validate the token
        let claims = auth_service.validate_token(&auth_response.token).await;
        assert!(claims.is_ok());
    }
}
```

### Requirement 4: Real-time Communication → WebSocket Actor Pattern

#### 4.1 WebSocket Connection Manager
```rust
use tokio::sync::broadcast;
use axum::extract::ws::{WebSocket, Message as WsMessage};

// Connection manager with actor pattern
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<UserId, Vec<ConnectionHandle>>>>,
    room_subscribers: Arc<RwLock<HashMap<RoomId, broadcast::Sender<RoomEvent>>>>,
}

#[derive(Debug, Clone)]
pub struct ConnectionHandle {
    id: ConnectionId,
    user_id: UserId,
    room_id: RoomId,
    sender: mpsc::UnboundedSender<WsMessage>,
}

#[derive(Debug, Clone, Serialize)]
pub enum RoomEvent {
    MessageCreated(Message),
    MessageUpdated(Message),
    MessageDeleted(MessageId),
    UserJoined(UserId),
    UserLeft(UserId),
    TypingStarted { user_id: UserId },
    TypingEnded { user_id: UserId },
    PresenceUpdate { user_id: UserId, is_present: bool },
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            room_subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn add_connection(
        &self,
        user_id: UserId,
        room_id: RoomId,
        websocket: WebSocket,
    ) -> Result<(), ConnectionError> {
        let connection_id = ConnectionId(Uuid::new_v4());
        let (ws_sender, mut ws_receiver) = websocket.split();
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Create connection handle
        let handle = ConnectionHandle {
            id: connection_id,
            user_id,
            room_id,
            sender: tx,
        };
        
        // Store connection
        {
            let mut connections = self.connections.write().await;
            connections.entry(user_id).or_default().push(handle.clone());
        }
        
        // Subscribe to room events
        let mut room_receiver = self.subscribe_to_room(room_id).await;
        
        // Spawn connection handler
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle outgoing messages
                    msg = rx.recv() => {
                        match msg {
                            Some(ws_msg) => {
                                if ws_sender.send(ws_msg).await.is_err() {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    
                    // Handle room events
                    event = room_receiver.recv() => {
                        match event {
                            Ok(room_event) => {
                                let json = serde_json::to_string(&room_event).unwrap();
                                let ws_msg = WsMessage::Text(json);
                                if tx.send(ws_msg).is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    
                    // Handle incoming WebSocket messages
                    ws_msg = ws_receiver.next() => {
                        match ws_msg {
                            Some(Ok(WsMessage::Text(text))) => {
                                // Handle incoming message
                                if let Err(e) = self.handle_incoming_message(&text, user_id, room_id).await {
                                    tracing::error!("Error handling WebSocket message: {}", e);
                                }
                            }
                            Some(Ok(WsMessage::Close(_))) => break,
                            Some(Err(e)) => {
                                tracing::error!("WebSocket error: {}", e);
                                break;
                            }
                            None => break,
                        }
                    }
                }
            }
            
            // Cleanup connection
            self.remove_connection(user_id, connection_id).await;
        });
        
        // Broadcast user joined event
        self.broadcast_to_room(room_id, RoomEvent::UserJoined(user_id)).await?;
        
        Ok(())
    }
    
    pub async fn broadcast_to_room(&self, room_id: RoomId, event: RoomEvent) -> Result<(), ConnectionError> {
        let room_subscribers = self.room_subscribers.read().await;
        
        if let Some(sender) = room_subscribers.get(&room_id) {
            let _ = sender.send(event); // Ignore if no receivers
        }
        
        Ok(())
    }
    
    async fn subscribe_to_room(&self, room_id: RoomId) -> broadcast::Receiver<RoomEvent> {
        let mut room_subscribers = self.room_subscribers.write().await;
        
        let sender = room_subscribers.entry(room_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(1000);
            tx
        });
        
        sender.subscribe()
    }
}

// TDD Test Suite
#[cfg(test)]
mod connection_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connection_manager_broadcasts_events() {
        let manager = ConnectionManager::new();
        let room_id = RoomId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        
        // Subscribe to room
        let mut receiver = manager.subscribe_to_room(room_id).await;
        
        // Broadcast event
        let event = RoomEvent::MessageCreated(create_test_message());
        manager.broadcast_to_room(room_id, event.clone()).await.unwrap();
        
        // Verify event received
        let received_event = receiver.recv().await.unwrap();
        assert!(matches!(received_event, RoomEvent::MessageCreated(_)));
    }
}
```

---

## React Frontend Pattern Mapping

### Requirement 8: Complete React UI → Component Architecture + Custom Hooks

#### 8.1 Message List Component with TDD
```jsx
// Step 1: RED - Write comprehensive tests
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MessageList } from './MessageList';
import { WebSocketProvider } from '../providers/WebSocketProvider';

describe('MessageList', () => {
  test('renders messages correctly', async () => {
    const mockMessages = [
      { id: '1', content: 'Hello', author: { name: 'Alice' }, createdAt: new Date() },
      { id: '2', content: 'World', author: { name: 'Bob' }, createdAt: new Date() },
    ];
    
    render(
      <WebSocketProvider roomId="room-123">
        <MessageList initialMessages={mockMessages} />
      </WebSocketProvider>
    );
    
    expect(screen.getByText('Hello')).toBeInTheDocument();
    expect(screen.getByText('World')).toBeInTheDocument();
    expect(screen.getByText('Alice')).toBeInTheDocument();
    expect(screen.getByText('Bob')).toBeInTheDocument();
  });
  
  test('handles real-time message updates', async () => {
    const { rerender } = render(
      <WebSocketProvider roomId="room-123">
        <MessageList initialMessages={[]} />
      </WebSocketProvider>
    );
    
    // Simulate WebSocket message
    const mockWebSocket = getMockWebSocket();
    mockWebSocket.simulateMessage({
      type: 'MESSAGE_CREATED',
      payload: { id: '1', content: 'New message', author: { name: 'Charlie' } }
    });
    
    await waitFor(() => {
      expect(screen.getByText('New message')).toBeInTheDocument();
    });
  });
  
  test('shows graceful file upload messaging', () => {
    render(
      <WebSocketProvider roomId="room-123">
        <MessageList initialMessages={[]} />
      </WebSocketProvider>
    );
    
    const fileUploadArea = screen.getByTestId('file-upload-area');
    expect(fileUploadArea).toHaveTextContent('File sharing available in v2.0');
    expect(fileUploadArea).toHaveClass('disabled');
  });
});

// Step 2: GREEN - Implement with custom hooks
function MessageList({ roomId, initialMessages = [] }) {
  const { messages, sendMessage, isConnected } = useRealTimeMessages(roomId, initialMessages);
  const { scrollRef, shouldAutoScroll } = useMessageScroll(messages);
  const { isTyping, typingUsers } = useTypingIndicators(roomId);
  
  return (
    <div className="message-list">
      <div className="connection-status">
        {isConnected ? (
          <span className="connected">Connected</span>
        ) : (
          <span className="disconnected">Reconnecting...</span>
        )}
      </div>
      
      <div ref={scrollRef} className="messages-container">
        {messages.map(message => (
          <MessageItem 
            key={message.id} 
            message={message}
            onEdit={handleEditMessage}
            onDelete={handleDeleteMessage}
          />
        ))}
        
        {isTyping && (
          <TypingIndicator users={typingUsers} />
        )}
      </div>
      
      <MessageComposer 
        onSend={sendMessage}
        disabled={!isConnected}
      />
      
      {/* Feature-flagged file upload area */}
      <FileUploadArea 
        data-testid="file-upload-area"
        className="disabled"
        onUploadAttempt={() => showFeatureMessage('File sharing available in v2.0')}
      />
    </div>
  );
}

// Step 3: REFACTOR - Extract custom hooks
function useRealTimeMessages(roomId, initialMessages) {
  const [messages, setMessages] = useState(initialMessages);
  const [isConnected, setIsConnected] = useState(false);
  const { socket, sendMessage: socketSend } = useWebSocket(roomId);
  
  useEffect(() => {
    if (!socket) return;
    
    const handleMessage = (event) => {
      const data = JSON.parse(event.data);
      
      switch (data.type) {
        case 'MESSAGE_CREATED':
          setMessages(prev => [...prev, data.payload]);
          break;
        case 'MESSAGE_UPDATED':
          setMessages(prev => prev.map(msg => 
            msg.id === data.payload.id ? { ...msg, ...data.payload } : msg
          ));
          break;
        case 'MESSAGE_DELETED':
          setMessages(prev => prev.filter(msg => msg.id !== data.payload.id));
          break;
      }
    };
    
    const handleOpen = () => setIsConnected(true);
    const handleClose = () => setIsConnected(false);
    
    socket.addEventListener('message', handleMessage);
    socket.addEventListener('open', handleOpen);
    socket.addEventListener('close', handleClose);
    
    return () => {
      socket.removeEventListener('message', handleMessage);
      socket.removeEventListener('open', handleOpen);
      socket.removeEventListener('close', handleClose);
    };
  }, [socket]);
  
  const sendMessage = useCallback(async (content) => {
    if (!socket || !isConnected) return;
    
    const message = {
      type: 'SEND_MESSAGE',
      payload: {
        content,
        clientId: crypto.randomUUID(),
        roomId,
      }
    };
    
    socketSend(JSON.stringify(message));
  }, [socket, isConnected, roomId, socketSend]);
  
  return { messages, sendMessage, isConnected };
}

function useMessageScroll(messages) {
  const scrollRef = useRef(null);
  const [shouldAutoScroll, setShouldAutoScroll] = useState(true);
  
  useEffect(() => {
    if (!scrollRef.current || !shouldAutoScroll) return;
    
    const container = scrollRef.current;
    const isNearBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 100;
    
    if (isNearBottom) {
      container.scrollTop = container.scrollHeight;
    }
  }, [messages, shouldAutoScroll]);
  
  useEffect(() => {
    const container = scrollRef.current;
    if (!container) return;
    
    const handleScroll = () => {
      const isNearBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 100;
      setShouldAutoScroll(isNearBottom);
    };
    
    container.addEventListener('scroll', handleScroll);
    return () => container.removeEventListener('scroll', handleScroll);
  }, []);
  
  return { scrollRef, shouldAutoScroll };
}
```

#### 8.2 Feature Flag Component Pattern
```jsx
// Feature flag hook for graceful degradation
function useFeatureFlags() {
  const [flags, setFlags] = useState({
    filesEnabled: false,
    avatarsEnabled: false,
    opengraphEnabled: false,
  });
  
  useEffect(() => {
    // Load feature flags from API or config
    fetch('/api/feature-flags')
      .then(res => res.json())
      .then(setFlags)
      .catch(() => {
        // Default to disabled on error
        console.warn('Failed to load feature flags, using defaults');
      });
  }, []);
  
  return flags;
}

// Feature-flagged component wrapper
function FeatureGate({ feature, children, fallback, upgradeMessage }) {
  const flags = useFeatureFlags();
  const isEnabled = flags[feature];
  
  if (isEnabled) {
    return children;
  }
  
  return (
    <div className="feature-disabled">
      {fallback}
      {upgradeMessage && (
        <div className="upgrade-message">
          <span className="icon">🚀</span>
          <span className="text">{upgradeMessage}</span>
          <button className="learn-more" onClick={() => showFeatureRoadmap(feature)}>
            Learn More
          </button>
        </div>
      )}
    </div>
  );
}

// Usage in components
function FileUploadArea({ onUploadAttempt }) {
  return (
    <FeatureGate 
      feature="filesEnabled"
      upgradeMessage="File sharing available in v2.0"
      fallback={
        <div 
          className="file-upload-placeholder"
          onClick={onUploadAttempt}
        >
          <div className="upload-icon">📎</div>
          <div className="upload-text">
            Drag files here or click to upload
          </div>
        </div>
      }
    >
      <ActualFileUpload />
    </FeatureGate>
  );
}

function AvatarUpload({ user, onAvatarChange }) {
  return (
    <FeatureGate
      feature="avatarsEnabled"
      upgradeMessage="Avatar uploads coming in v2.0"
      fallback={
        <div className="avatar-placeholder">
          <div className="avatar-initials">
            {user.name.split(' ').map(n => n[0]).join('')}
          </div>
          <button className="avatar-upload-disabled">
            Upload Avatar
          </button>
        </div>
      }
    >
      <ActualAvatarUpload user={user} onChange={onAvatarChange} />
    </FeatureGate>
  );
}

// TDD Tests for feature flags
describe('FeatureGate', () => {
  test('renders children when feature is enabled', () => {
    mockFeatureFlags({ filesEnabled: true });
    
    render(
      <FeatureGate feature="filesEnabled">
        <div>Feature content</div>
      </FeatureGate>
    );
    
    expect(screen.getByText('Feature content')).toBeInTheDocument();
  });
  
  test('renders fallback when feature is disabled', () => {
    mockFeatureFlags({ filesEnabled: false });
    
    render(
      <FeatureGate 
        feature="filesEnabled"
        fallback={<div>Feature disabled</div>}
        upgradeMessage="Coming soon"
      >
        <div>Feature content</div>
      </FeatureGate>
    );
    
    expect(screen.getByText('Feature disabled')).toBeInTheDocument();
    expect(screen.getByText('Coming soon')).toBeInTheDocument();
    expect(screen.queryByText('Feature content')).not.toBeInTheDocument();
  });
});
```

### Requirement 13: MVP Feature Flag System → Context + Provider Pattern

#### 13.1 Feature Flag Provider with TDD
```jsx
// Feature flag context and provider
const FeatureFlagContext = createContext();

export function FeatureFlagProvider({ children }) {
  const [flags, setFlags] = useState({
    filesEnabled: false,
    avatarsEnabled: false,
    opengraphEnabled: false,
  });
  
  const [upgradeMessages, setUpgradeMessages] = useState({
    filesEnabled: "File sharing available in v2.0 - Expected March 2025",
    avatarsEnabled: "Avatar uploads coming in v2.0 - Expected February 2025", 
    opengraphEnabled: "Link previews coming in v2.0 - Expected March 2025",
  });
  
  const [userFeedback, setUserFeedback] = useState({});
  
  // Load feature flags from API
  useEffect(() => {
    const loadFeatureFlags = async () => {
      try {
        const response = await fetch('/api/feature-flags');
        const data = await response.json();
        setFlags(data.flags);
        setUpgradeMessages(data.upgradeMessages);
      } catch (error) {
        console.warn('Failed to load feature flags:', error);
        // Use default disabled state
      }
    };
    
    loadFeatureFlags();
  }, []);
  
  // Track feature interest
  const trackFeatureInterest = useCallback((feature, action = 'viewed') => {
    const feedback = {
      feature,
      action,
      timestamp: new Date().toISOString(),
      userId: getCurrentUserId(),
    };
    
    // Send to analytics
    fetch('/api/feature-feedback', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(feedback),
    }).catch(console.error);
    
    // Update local state
    setUserFeedback(prev => ({
      ...prev,
      [feature]: [...(prev[feature] || []), feedback],
    }));
  }, []);
  
  const showFeatureRoadmap = useCallback((feature) => {
    trackFeatureInterest(feature, 'roadmap_viewed');
    // Show modal with feature roadmap
    openFeatureRoadmapModal(feature);
  }, [trackFeatureInterest]);
  
  const value = {
    flags,
    upgradeMessages,
    userFeedback,
    trackFeatureInterest,
    showFeatureRoadmap,
    isEnabled: (feature) => flags[feature] === true,
    getUpgradeMessage: (feature) => upgradeMessages[feature],
  };
  
  return (
    <FeatureFlagContext.Provider value={value}>
      {children}
    </FeatureFlagContext.Provider>
  );
}

export function useFeatureFlags() {
  const context = useContext(FeatureFlagContext);
  if (!context) {
    throw new Error('useFeatureFlags must be used within FeatureFlagProvider');
  }
  return context;
}

// TDD Tests for feature flag provider
describe('FeatureFlagProvider', () => {
  test('provides feature flag state to children', () => {
    const TestComponent = () => {
      const { flags, isEnabled } = useFeatureFlags();
      return (
        <div>
          <span data-testid="files-enabled">{isEnabled('filesEnabled').toString()}</span>
          <span data-testid="avatars-enabled">{isEnabled('avatarsEnabled').toString()}</span>
        </div>
      );
    };
    
    render(
      <FeatureFlagProvider>
        <TestComponent />
      </FeatureFlagProvider>
    );
    
    expect(screen.getByTestId('files-enabled')).toHaveTextContent('false');
    expect(screen.getByTestId('avatars-enabled')).toHaveTextContent('false');
  });
  
  test('tracks feature interest correctly', async () => {
    const mockFetch = jest.fn().mockResolvedValue({ ok: true });
    global.fetch = mockFetch;
    
    const TestComponent = () => {
      const { trackFeatureInterest } = useFeatureFlags();
      return (
        <button onClick={() => trackFeatureInterest('filesEnabled', 'clicked')}>
          Track Interest
        </button>
      );
    };
    
    render(
      <FeatureFlagProvider>
        <TestComponent />
      </FeatureFlagProvider>
    );
    
    const button = screen.getByText('Track Interest');
    await userEvent.click(button);
    
    expect(mockFetch).toHaveBeenCalledWith('/api/feature-feedback', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: expect.stringContaining('"feature":"filesEnabled"'),
    });
  });
});
```

---

## Cross-Cutting Pattern Integration

### 1. Error Handling Pattern Integration

#### 1.1 Unified Error Handling (Rust + React)
```rust
// Rust: Comprehensive error types with context
#[derive(Error, Debug, Serialize)]
pub enum CampfireError {
    #[error("Authentication failed: {reason}")]
    Authentication { reason: String },
    
    #[error("Authorization failed: user {user_id} cannot access {resource}")]
    Authorization { user_id: UserId, resource: String },
    
    #[error("Validation error in {field}: {message}")]
    Validation { field: String, message: String },
    
    #[error("Rate limit exceeded: {limit} requests per {window}")]
    RateLimit { limit: u32, window: String },
    
    #[error("Feature not available: {feature} - {upgrade_message}")]
    FeatureDisabled { feature: String, upgrade_message: String },
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("WebSocket error: {0}")]
    WebSocket(String),
}

impl IntoResponse for CampfireError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            CampfireError::Authentication { .. } => (StatusCode::UNAUTHORIZED, self.to_string()),
            CampfireError::Authorization { .. } => (StatusCode::FORBIDDEN, self.to_string()),
            CampfireError::Validation { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            CampfireError::RateLimit { .. } => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            CampfireError::FeatureDisabled { .. } => (StatusCode::NOT_IMPLEMENTED, self.to_string()),
            CampfireError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            CampfireError::WebSocket(_) => (StatusCode::INTERNAL_SERVER_ERROR, "WebSocket error".to_string()),
        };
        
        let body = Json(serde_json::json!({
            "error": error_message,
            "code": status.as_u16(),
            "timestamp": Utc::now().to_rfc3339(),
        }));
        
        (status, body).into_response()
    }
}
```

```jsx
// React: Error boundary with feature flag integration
function CampfireErrorBoundary({ children }) {
  return (
    <ErrorBoundary
      FallbackComponent={ErrorFallback}
      onError={(error, errorInfo) => {
        // Log to error reporting service
        console.error('Error caught by boundary:', error, errorInfo);
        
        // Track feature-related errors
        if (error.message.includes('feature')) {
          trackFeatureError(error.message);
        }
      }}
    >
      {children}
    </ErrorBoundary>
  );
}

function ErrorFallback({ error, resetErrorBoundary }) {
  const { showFeatureRoadmap } = useFeatureFlags();
  
  // Check if this is a feature-disabled error
  const isFeatureError = error.message.includes('Feature not available');
  
  if (isFeatureError) {
    const feature = extractFeatureFromError(error.message);
    return (
      <div className="error-boundary feature-error">
        <h2>Feature Coming Soon!</h2>
        <p>{error.message}</p>
        <div className="error-actions">
          <button onClick={() => showFeatureRoadmap(feature)}>
            View Roadmap
          </button>
          <button onClick={resetErrorBoundary}>
            Continue
          </button>
        </div>
      </div>
    );
  }
  
  return (
    <div className="error-boundary">
      <h2>Something went wrong</h2>
      <details>
        <summary>Error details</summary>
        <pre>{error.message}</pre>
      </details>
      <button onClick={resetErrorBoundary}>Try again</button>
    </div>
  );
}
```

### 2. Performance Pattern Integration

#### 2.1 Rust Performance Patterns
```rust
// Zero-cost abstractions for message processing
pub fn process_messages_efficiently(
    messages: impl Iterator<Item = Message>,
    user_involvement: Involvement,
    user_id: UserId,
) -> impl Iterator<Item = ProcessedMessage> {
    messages
        .filter(move |msg| match user_involvement {
            Involvement::Everything => true,
            Involvement::Mentions => msg.mentions.contains(&user_id),
            Involvement::Nothing => false,
            Involvement::Invisible => false,
        })
        .map(|msg| ProcessedMessage::from(msg))
        .take(50) // Pagination limit
}

// Efficient WebSocket broadcasting with batching
pub struct BatchedBroadcaster {
    pending_messages: Arc<Mutex<Vec<RoomEvent>>>,
    batch_interval: Duration,
}

impl BatchedBroadcaster {
    pub async fn broadcast_message(&self, event: RoomEvent) {
        {
            let mut pending = self.pending_messages.lock().await;
            pending.push(event);
        }
        
        // Trigger batch send if not already scheduled
        self.schedule_batch_send().await;
    }
    
    async fn schedule_batch_send(&self) {
        tokio::time::sleep(self.batch_interval).await;
        
        let messages = {
            let mut pending = self.pending_messages.lock().await;
            std::mem::take(&mut *pending)
        };
        
        if !messages.is_empty() {
            self.send_batch(messages).await;
        }
    }
}
```

#### 2.2 React Performance Patterns
```jsx
// Virtualized message list for large message counts
import { FixedSizeList as List } from 'react-window';

function VirtualizedMessageList({ messages }) {
  const Row = useCallback(({ index, style }) => (
    <div style={style}>
      <MessageItem message={messages[index]} />
    </div>
  ), [messages]);
  
  return (
    <List
      height={600}
      itemCount={messages.length}
      itemSize={80}
      width="100%"
    >
      {Row}
    </List>
  );
}

// Optimized WebSocket hook with reconnection
function useWebSocket(roomId) {
  const [socket, setSocket] = useState(null);
  const [connectionState, setConnectionState] = useState('connecting');
  const reconnectTimeoutRef = useRef(null);
  const reconnectAttempts = useRef(0);
  
  const connect = useCallback(() => {
    const ws = new WebSocket(`ws://localhost:8080/rooms/${roomId}`);
    
    ws.onopen = () => {
      setConnectionState('connected');
      reconnectAttempts.current = 0;
      setSocket(ws);
    };
    
    ws.onclose = () => {
      setConnectionState('disconnected');
      setSocket(null);
      
      // Exponential backoff reconnection
      const delay = Math.min(1000 * Math.pow(2, reconnectAttempts.current), 30000);
      reconnectAttempts.current += 1;
      
      reconnectTimeoutRef.current = setTimeout(connect, delay);
    };
    
    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      setConnectionState('error');
    };
    
    return ws;
  }, [roomId]);
  
  useEffect(() => {
    const ws = connect();
    
    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.close();
      }
    };
  }, [connect]);
  
  const sendMessage = useCallback((message) => {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(message);
    }
  }, [socket]);
  
  return { socket, connectionState, sendMessage };
}
```

---

## Testing Strategy and Implementation

### 1. Comprehensive TDD Test Suites

#### 1.1 Rust Testing Patterns
```rust
// Property-based testing for message validation
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_message_content_validation(
        content in ".*",
        room_id in any::<Uuid>().prop_map(RoomId),
        user_id in any::<Uuid>().prop_map(UserId),
    ) {
        let result = validate_message_content(&content, room_id, user_id);
        
        if content.trim().is_empty() {
            prop_assert!(result.is_err());
        } else if content.len() > 10000 {
            prop_assert!(result.is_err());
        } else {
            prop_assert!(result.is_ok());
        }
    }
}

// Integration testing with test database
#[tokio::test]
async fn test_message_creation_integration() {
    let test_db = TestDatabase::new().await;
    let message_service = MessageService::new(test_db.clone());
    
    // Create test user and room
    let user = test_db.create_test_user().await;
    let room = test_db.create_test_room().await;
    
    // Test message creation
    let message = message_service.create_message(
        "Integration test message".to_string(),
        room.id,
        user.id,
    ).await.unwrap();
    
    // Verify message was stored
    let stored_message = test_db.get_message(message.id).await.unwrap();
    assert_eq!(stored_message.content, "Integration test message");
    
    // Verify room received the message
    let room_messages = test_db.get_room_messages(room.id).await.unwrap();
    assert_eq!(room_messages.len(), 1);
}

// Concurrency testing with loom
#[cfg(loom)]
mod loom_tests {
    use super::*;
    use loom::sync::Arc;
    use loom::thread;
    
    #[test]
    fn test_concurrent_message_creation() {
        loom::model(|| {
            let message_service = Arc::new(MessageService::new(MockDatabase::new()));
            
            let handles: Vec<_> = (0..2).map(|i| {
                let service = message_service.clone();
                thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        service.create_message(
                            format!("Message {}", i),
                            RoomId(Uuid::new_v4()),
                            UserId(Uuid::new_v4()),
                        ).await
                    })
                })
            }).collect();
            
            for handle in handles {
                handle.join().unwrap().unwrap();
            }
        });
    }
}
```

#### 1.2 React Testing Patterns
```jsx
// Component integration testing
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ChatInterface } from './ChatInterface';

function renderWithProviders(ui, options = {}) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  
  const AllProviders = ({ children }) => (
    <QueryClientProvider client={queryClient}>
      <FeatureFlagProvider>
        <WebSocketProvider roomId="test-room">
          {children}
        </WebSocketProvider>
      </FeatureFlagProvider>
    </QueryClientProvider>
  );
  
  return render(ui, { wrapper: AllProviders, ...options });
}

describe('ChatInterface Integration', () => {
  test('complete message flow works correctly', async () => {
    const user = userEvent.setup();
    
    renderWithProviders(<ChatInterface />);
    
    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByText('Connected')).toBeInTheDocument();
    });
    
    // Type and send message
    const textarea = screen.getByRole('textbox');
    const sendButton = screen.getByRole('button', { name: /send/i });
    
    await user.type(textarea, 'Hello, integration test!');
    await user.click(sendButton);
    
    // Verify message appears
    await waitFor(() => {
      expect(screen.getByText('Hello, integration test!')).toBeInTheDocument();
    });
    
    // Verify textarea is cleared
    expect(textarea).toHaveValue('');
  });
  
  test('feature flag integration works', () => {
    mockFeatureFlags({ filesEnabled: false });
    
    renderWithProviders(<ChatInterface />);
    
    const fileUploadArea = screen.getByTestId('file-upload-area');
    expect(fileUploadArea).toHaveClass('disabled');
    expect(fileUploadArea).toHaveTextContent('File sharing available in v2.0');
  });
});

// Custom hook testing
import { renderHook, act } from '@testing-library/react';
import { useRealTimeMessages } from './useRealTimeMessages';

describe('useRealTimeMessages', () => {
  test('manages message state correctly', async () => {
    const { result } = renderHook(() => 
      useRealTimeMessages('room-123', [])
    );
    
    expect(result.current.messages).toEqual([]);
    expect(result.current.isConnected).toBe(false);
    
    // Simulate WebSocket connection
    act(() => {
      mockWebSocket.simulateOpen();
    });
    
    expect(result.current.isConnected).toBe(true);
    
    // Simulate receiving a message
    act(() => {
      mockWebSocket.simulateMessage({
        type: 'MESSAGE_CREATED',
        payload: { id: '1', content: 'Test message' }
      });
    });
    
    expect(result.current.messages).toHaveLength(1);
    expect(result.current.messages[0].content).toBe('Test message');
  });
});
```

### 2. End-to-End Testing Strategy

#### 2.1 E2E Test Patterns
```javascript
// Playwright E2E tests
import { test, expect } from '@playwright/test';

test.describe('Campfire MVP E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Set up test environment
    await page.goto('/');
    await page.fill('[data-testid="email"]', 'test@example.com');
    await page.fill('[data-testid="password"]', 'password123');
    await page.click('[data-testid="login-button"]');
    
    // Wait for chat interface to load
    await expect(page.locator('[data-testid="chat-interface"]')).toBeVisible();
  });
  
  test('user can send and receive messages', async ({ page }) => {
    // Send a message
    await page.fill('[data-testid="message-input"]', 'Hello from E2E test!');
    await page.click('[data-testid="send-button"]');
    
    // Verify message appears
    await expect(page.locator('text=Hello from E2E test!')).toBeVisible();
    
    // Verify input is cleared
    await expect(page.locator('[data-testid="message-input"]')).toHaveValue('');
  });
  
  test('feature flags show appropriate messaging', async ({ page }) => {
    // Try to access file upload
    await page.click('[data-testid="file-upload-area"]');
    
    // Verify upgrade message appears
    await expect(page.locator('text=File sharing available in v2.0')).toBeVisible();
    
    // Verify roadmap can be accessed
    await page.click('[data-testid="learn-more-button"]');
    await expect(page.locator('[data-testid="feature-roadmap-modal"]')).toBeVisible();
  });
  
  test('real-time updates work correctly', async ({ page, context }) => {
    // Open second tab to simulate another user
    const secondPage = await context.newPage();
    await secondPage.goto('/');
    
    // Login as different user
    await secondPage.fill('[data-testid="email"]', 'user2@example.com');
    await secondPage.fill('[data-testid="password"]', 'password123');
    await secondPage.click('[data-testid="login-button"]');
    
    // Send message from second user
    await secondPage.fill('[data-testid="message-input"]', 'Message from user 2');
    await secondPage.click('[data-testid="send-button"]');
    
    // Verify message appears in first tab
    await expect(page.locator('text=Message from user 2')).toBeVisible();
  });
});
```

---

## Conclusion

This L2 architecture document provides a comprehensive, TDD-driven approach to implementing the Campfire MVP using idiomatic Rust and React patterns. Every component is designed with:

1. **Test-First Development**: All code follows Red-Green-Refactor cycle
2. **Pattern-Based Architecture**: Consistent use of proven patterns
3. **Feature Flag Integration**: Graceful degradation for MVP approach
4. **Performance Optimization**: Zero-cost abstractions and efficient algorithms
5. **Comprehensive Error Handling**: Unified error management across stack
6. **Maintainable Code Structure**: Clear separation of concerns and responsibilities

The architecture ensures that the MVP delivers a professional user experience while maintaining ultra-low costs and providing a clear evolution path to full Rails parity through the 4-phase feature rollout strategy.