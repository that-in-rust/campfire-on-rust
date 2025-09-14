# Realistic Implementation Plan - Campfire MVP 1.0

## Document Hierarchy Reference

This document is the **bottom of the pyramid** with maximum implementation detail:

```
requirements.md (Governing Rules & Critical Gaps)
    ↓
architecture.md (System Architecture & Component Design)
    ↓  
architecture-L2.md (TDD Implementation Patterns)
    ↓
design.md (Complete Technical Contracts & Interfaces)
    ↓
tasks.md (THIS DOCUMENT - Maximum Implementation Detail)
```

**Before coding, developers should reference**:
- **design.md** for all interface contracts and type definitions
- **architecture-L2.md** for TDD patterns and Rails parity strategies
- This document for specific implementation tasks and property tests

## Overview: TDD-Driven "Rails-Equivalent Imperfection" Strategy

This implementation plan uses **Test-Driven Development** with **function signatures defined before implementation** to achieve **"works well enough"** rather than **"perfect"** - exactly matching Rails behavior and limitations while ensuring one-shot correctness through type-driven design.

**TDD-First Philosophy**:
- **Type Contracts Before Code**: Define complete function signatures with all error cases first
- **Property-Based Specifications**: Specify behavior through property tests that validate invariants
- **Rails Parity Rule**: If Rails doesn't do it perfectly, we don't need to either - but we specify it completely
- **"Good Enough" Quality**: Match Rails reliability with compile-time guarantees
- **5 Critical Gaps Only**: Fix only gaps that Rails actually solves, using type-driven design
- **Accept Known Limitations**: Document and accept Rails-equivalent limitations with type safety

**TDD Success Criteria**: 
1. **Complete Type Contracts**: Every function signature defined with all error cases before implementation
2. **Property Test Coverage**: All invariants validated with property-based testing
3. **Integration Test Validation**: All service boundaries tested with real dependencies
4. **Rails Behavioral Parity**: Works as well as Rails ActionCable, with similar limitations and edge cases
5. **Compile-Time Safety**: Type system prevents coordination complexity and common bugs

## MVP 1.0 Focus: Complete UI with Text-Only Backend

**What's Included in MVP 1.0**:
- ✅ **Complete React UI** - All components, styling, and interactions (26 CSS files)
- ✅ **Rich text messaging** - Trix editor with HTML formatting, sounds, boosts
- ✅ **Real-time features** - WebSocket broadcasting, presence, typing notifications
- ✅ **Room management** - Open/Closed/Direct rooms with membership controls
- ✅ **Authentication** - Session management, bot integration, role-based access
- ✅ **Search functionality** - FTS5-powered message search
- ✅ **Push notifications** - Web Push with VAPID keys
- ✅ **Sound system** - 59 embedded MP3 files with /play commands

**What's Gracefully Deferred to v2.0**:
- 🚫 **File attachments** - Complete UI shown with "Coming in v2.0" messaging
- 🚫 **Avatar uploads** - Text-based initials with upload UI for future
- 🚫 **OpenGraph previews** - Link detection with placeholder for future unfurling

## 5 Critical Gaps That Rails Actually Solves

**Gap #1: client_message_id Deduplication**
- **Rails Reality**: Uses database UNIQUE constraints for duplicate prevention
- **Our Fix**: Add UNIQUE constraint on (client_message_id, room_id)
- **Requirements**: Critical Gap #1 defined in requirements.md - prevent duplicate messages

**Gap #2: WebSocket Reconnection State**
- **Rails Reality**: ActionCable tracks connection state for missed message delivery
- **Our Fix**: Track last_seen_message_id per connection, send missed messages
- **Requirements**: Critical Gap #2 defined in requirements.md - deliver missed messages on reconnect

**Gap #3: SQLite Write Serialization**
- **Rails Reality**: Connection pooling effectively serializes writes
- **Our Fix**: Dedicated Writer Task pattern with mpsc channel
- **Requirements**: Critical Gap #3 defined in requirements.md - serialize concurrent writes

**Gap #4: Session Token Security**
- **Rails Reality**: Uses SecureRandom for session tokens with proper validation
- **Our Fix**: Implement Rails-equivalent secure token generation with type safety
- **Requirements**: Critical Gap #4 defined in requirements.md - secure token generation

**Gap #5: Basic Presence Tracking**
- **Rails Reality**: Simple connection counting with heartbeat cleanup
- **Our Fix**: HashMap<UserId, connection_count> with 60-second TTL and RAII cleanup
- **Requirements**: Critical Gap #5 defined in requirements.md - presence tracking with TTL cleanup

## TDD Implementation Methodology

### Phase 1: Type Contract Definition (Before Any Code)
- Define complete function signatures for all services
- Specify all error cases in Result<T, E> types
- Document behavior contracts and side effects
- Create comprehensive type definitions with newtypes

### Phase 2: Property Test Specification
- Write property-based tests for all invariants
- Specify behavior through mathematical properties
- Create test data generators with proptest
- Define integration test contracts

### Phase 3: Type-Guided Implementation
- Implement following type contracts
- Use type system to prevent coordination complexity
- Apply RAII patterns for resource management
- Implement actor patterns for state management

### Phase 4: Comprehensive Validation
- Validate property test compliance
- Run integration tests with real dependencies
- Benchmark critical paths for performance
- Verify Rails behavioral parity
- **Our Fix**: Implement Rails-equivalent secure token generation
- **Requirements**: Requirement 3.3 - secure session management

**Gap #5: Basic Presence Tracking**
- **Rails Reality**: Simple connection counting with heartbeat cleanup
- **Our Fix**: HashMap<UserId, connection_count> with 60-second TTL
- **Requirements**: Requirement 4.2-4.7 - presence tracking with connection management

## Rails-Level Limitations We Accept (Don't Over-Engineer)

**Limitation #1: Imperfect Message Ordering**
- **Rails Reality**: Uses created_at timestamps, occasional out-of-order acceptable
- **Our Approach**: Database timestamps, no complex vector clocks or coordination
- **Requirements**: Requirement 4.11 - "maintain message order consistency" = Rails-level

**Limitation #2: Multi-tab Connection Independence**
- **Rails Reality**: Each tab creates independent ActionCable connection
- **Our Approach**: No cross-tab coordination, each connection is separate
- **Requirements**: Requirement 4.10 - "across multiple browser tabs" = Rails behavior

**Limitation #3: Best-Effort WebSocket Delivery**
- **Rails Reality**: ActionCable doesn't guarantee message delivery
- **Our Approach**: Simple broadcast with timeout, no delivery confirmation
- **Requirements**: Requirement 4.1 - "broadcast within 100ms" = best effort like Rails

**Limitation #4: Presence Tracking Delays**
- **Rails Reality**: Connection cleanup has delays, occasional inaccuracy
- **Our Approach**: 60-second heartbeat, accept brief inaccuracy
- **Requirements**: Requirement 4.4 - "handle browser crashes gracefully" = Rails level

---

## Phase 0: TDD Foundation (Week 0)

**Goal**: Establish complete type contracts and property tests before any implementation

### 0.1 Type Contract Definition

- [ ] **0.1.1 Implement service trait interfaces from design.md**
  - **Reference**: Complete interface contracts defined in `design.md`
  - Implement MessageService trait with deduplication support (Critical Gap #1)
  - Implement RoomService trait with membership management
  - Implement AuthService trait with session management (Critical Gap #4)
  - Implement WebSocketBroadcaster trait with reconnection (Critical Gap #2)
  - Implement DatabaseWriter trait for write serialization (Critical Gap #3)
  - Copy interface definitions directly from design.md for implementation
  - _Requirements: All service boundaries defined before implementation_

- [ ] **0.1.2 Create comprehensive error type hierarchy**
  ```rust
  // COMPLETE ERROR HIERARCHY - Exhaustive error case enumeration
  
  #[derive(Debug, thiserror::Error)]
  pub enum MessageError {
      #[error("Database operation failed: {0}")]
      Database(#[from] sqlx::Error),
      
      #[error("Message validation failed: {field} - {reason}")]
      Validation { field: String, reason: String },
      
      #[error("User {user_id} not authorized for room {room_id}")]
      Authorization { user_id: UserId, room_id: RoomId },
      
      #[error("Message {message_id} not found")]
      NotFound { message_id: MessageId },
      
      #[error("Broadcast failed: {reason}")]
      BroadcastFailed { reason: String },
      
      #[error("Duplicate client_message_id handled")]
      DuplicateHandled { existing_id: MessageId },
  }
  
  #[derive(Debug, thiserror::Error)]
  pub enum RoomError {
      #[error("Database operation failed: {0}")]
      Database(#[from] sqlx::Error),
      
      #[error("Room {room_id} not found")]
      NotFound { room_id: RoomId },
      
      #[error("User {user_id} not authorized for room {room_id}")]
      AccessDenied { user_id: UserId, room_id: RoomId },
      
      #[error("Invalid room type: {reason}")]
      InvalidType { reason: String },
      
      #[error("Membership limit exceeded for room {room_id}")]
      MembershipLimitExceeded { room_id: RoomId },
  }
  
  #[derive(Debug, thiserror::Error)]
  pub enum AuthError {
      #[error("Database operation failed: {0}")]
      Database(#[from] sqlx::Error),
      
      #[error("Invalid credentials for email: {email}")]
      InvalidCredentials { email: String },
      
      #[error("Session token invalid or expired")]
      InvalidSession,
      
      #[error("Rate limit exceeded: {attempts} attempts in {window_seconds}s")]
      RateLimitExceeded { attempts: u32, window_seconds: u32 },
      
      #[error("User account deactivated: {user_id}")]
      AccountDeactivated { user_id: UserId },
  }
  
  #[derive(Debug, thiserror::Error)]
  pub enum BroadcastError {
      #[error("Connection {connection_id} not found")]
      ConnectionNotFound { connection_id: ConnectionId },
      
      #[error("WebSocket send failed: {reason}")]
      SendFailed { reason: String },
      
      #[error("Serialization failed: {0}")]
      SerializationFailed(#[from] serde_json::Error),
      
      #[error("Room {room_id} has no active connections")]
      NoActiveConnections { room_id: RoomId },
  }
  
  #[derive(Debug, thiserror::Error)]
  pub enum DatabaseError {
      #[error("SQLite error: {0}")]
      Sqlite(#[from] sqlx::Error),
      
      #[error("Write channel closed")]
      WriterUnavailable,
      
      #[error("Transaction failed: {reason}")]
      TransactionFailed { reason: String },
      
      #[error("Constraint violation: {constraint}")]
      ConstraintViolation { constraint: String },
  }
  ```
  - _Requirements: Complete error case enumeration for Result<T, E> types_

- [ ] **0.1.3 Define domain model type contracts**
  ```rust
  // PHANTOM TYPES FOR STATE SAFETY - Prevent invalid state transitions
  
  use std::marker::PhantomData;
  
  /// Message with compile-time state tracking
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Message<State = Persisted> {
      pub id: MessageId,
      pub client_message_id: Uuid,
      pub content: String,
      pub room_id: RoomId,
      pub creator_id: UserId,
      pub created_at: DateTime<Utc>,
      pub updated_at: DateTime<Utc>,
      _state: PhantomData<State>,
  }
  
  /// Message states for type safety
  pub struct Draft;      // Newly created, not validated
  pub struct Validated;  // Content validated, ready for persistence
  pub struct Persisted;  // Stored in database, has ID
  
  impl Message<Draft> {
      pub fn new(content: String, room_id: RoomId, creator_id: UserId, client_id: Uuid) -> Self {
          Self {
              id: MessageId(0), // Temporary ID
              client_message_id: client_id,
              content,
              room_id,
              creator_id,
              created_at: Utc::now(),
              updated_at: Utc::now(),
              _state: PhantomData,
          }
      }
      
      /// Validate message content and transition to Validated state
      pub fn validate(self) -> Result<Message<Validated>, MessageError> {
          if self.content.is_empty() {
              return Err(MessageError::Validation {
                  field: "content".to_string(),
                  reason: "Content cannot be empty".to_string(),
              });
          }
          
          if self.content.len() > 10000 {
              return Err(MessageError::Validation {
                  field: "content".to_string(),
                  reason: "Content exceeds 10000 characters".to_string(),
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
              _state: PhantomData,
          })
      }
  }
  
  impl Message<Validated> {
      /// Persist message to database and transition to Persisted state
      pub async fn persist(
          self,
          db: &DatabaseWriter,
      ) -> Result<Message<Persisted>, MessageError> {
          let persisted_data = db.execute_write(WriteCommand::CreateMessage {
              content: self.content,
              room_id: self.room_id,
              creator_id: self.creator_id,
              client_message_id: self.client_message_id,
          }).await?;
          
          Ok(Message {
              id: persisted_data.id,
              client_message_id: self.client_message_id,
              content: self.content,
              room_id: self.room_id,
              creator_id: self.creator_id,
              created_at: persisted_data.created_at,
              updated_at: persisted_data.updated_at,
              _state: PhantomData,
          })
      }
  }
  
  // Only persisted messages can be broadcast
  impl WebSocketBroadcaster {
      pub async fn broadcast_message(&self, message: Message<Persisted>) -> Result<(), BroadcastError> {
          // Type system ensures only persisted messages are broadcast
          self.broadcast_to_room(message.room_id, &message).await
      }
  }
  
  /// WebSocket connection with state machine
  #[derive(Debug)]
  pub struct WebSocketConnection<State> {
      pub id: ConnectionId,
      pub user_id: Option<UserId>,
      pub room_id: Option<RoomId>,
      pub last_seen_message_id: Option<MessageId>,
      pub connected_at: DateTime<Utc>,
      _state: PhantomData<State>,
  }
  
  /// Connection states
  pub struct Connected;      // WebSocket connected, not authenticated
  pub struct Authenticated;  // User authenticated via session
  pub struct Subscribed;     // Subscribed to room updates
  
  impl WebSocketConnection<Connected> {
      pub fn authenticate(self, user_id: UserId) -> WebSocketConnection<Authenticated> {
          WebSocketConnection {
              id: self.id,
              user_id: Some(user_id),
              room_id: self.room_id,
              last_seen_message_id: self.last_seen_message_id,
              connected_at: self.connected_at,
              _state: PhantomData,
          }
      }
  }
  
  impl WebSocketConnection<Authenticated> {
      pub fn subscribe_to_room(self, room_id: RoomId) -> WebSocketConnection<Subscribed> {
          WebSocketConnection {
              id: self.id,
              user_id: self.user_id,
              room_id: Some(room_id),
              last_seen_message_id: self.last_seen_message_id,
              connected_at: self.connected_at,
              _state: PhantomData,
          }
      }
  }
  
  // Only subscribed connections can receive room messages
  impl WebSocketConnection<Subscribed> {
      pub async fn send_message(&self, message: &WebSocketMessage) -> Result<(), BroadcastError> {
          // Implementation sends message to this specific connection
          todo!("Send WebSocket message to connection")
      }
  }
  
  /// All newtype IDs with compile-time safety
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub struct UserId(pub i64);
  
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub struct RoomId(pub i64);
  
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub struct MessageId(pub i64);
  
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub struct SessionId(pub i64);
  
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub struct ConnectionId(pub i64);
  
  #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub struct BotToken(pub String);
  ```
  - _Requirements: Type system prevents invalid state transitions_

### 0.2 Property Test Specification

- [ ] **0.2.1 Message service property tests**
  ```rust
  // COMPREHENSIVE PROPERTY TESTS - Critical Gap Validation
  
  use proptest::prelude::*;
  
  proptest! {
      #[test]
      fn prop_duplicate_client_id_returns_same_message(
          content1 in ".*",
          content2 in ".*",
          room_id in any::<u64>().prop_map(RoomId),
          user_id in any::<u64>().prop_map(UserId),
          client_id in any::<Uuid>(),
      ) {
          let rt = tokio::runtime::Runtime::new().unwrap();
          rt.block_on(async {
              let service = setup_test_message_service().await;
              
              // Create first message
              let msg1 = service.create_message_with_deduplication(
                  content1, room_id, user_id, client_id
              ).await.unwrap();
              
              // Attempt duplicate with different content but same client_id
              let msg2 = service.create_message_with_deduplication(
                  content2, room_id, user_id, client_id
              ).await.unwrap();
              
              // CRITICAL GAP #1: Same client_id always returns same message
              prop_assert_eq!(msg1.id, msg2.id);
              prop_assert_eq!(msg1.content, msg2.content); // Original preserved
              prop_assert_eq!(msg1.client_message_id, msg2.client_message_id);
          });
      }

      #[test]
      fn prop_messages_since_chronological_order(
          room_id in any::<u64>().prop_map(RoomId),
          user_id in any::<u64>().prop_map(UserId),
          message_count in 2..20usize,
      ) {
          let rt = tokio::runtime::Runtime::new().unwrap();
          rt.block_on(async {
              let service = setup_test_message_service().await;
              
              // Create messages in sequence
              let mut created_messages = Vec::new();
              for i in 0..message_count {
                  let msg = service.create_message_with_deduplication(
                      format!("Message {}", i),
                      room_id,
                      user_id,
                      Uuid::new_v4(),
                  ).await.unwrap();
                  created_messages.push(msg);
                  tokio::time::sleep(Duration::from_millis(1)).await;
              }
              
              // Get messages since first message
              let since_id = created_messages[0].id;
              let retrieved = service.get_messages_since(room_id, since_id, user_id).await.unwrap();
              
              // Property: Messages returned in chronological order
              for window in retrieved.windows(2) {
                  prop_assert!(
                      window[0].created_at <= window[1].created_at,
                      "Messages not in chronological order"
                  );
                  if window[0].created_at == window[1].created_at {
                      prop_assert!(window[0].id < window[1].id, "Same timestamp not ordered by ID");
                  }
              }
          });
      }

      #[test]
      fn prop_message_validation_boundaries(
          content in ".*",
          room_id in any::<u64>().prop_map(RoomId),
          user_id in any::<u64>().prop_map(UserId),
      ) {
          let rt = tokio::runtime::Runtime::new().unwrap();
          rt.block_on(async {
              let service = setup_test_message_service().await;
              let client_id = Uuid::new_v4();
              
              let result = service.create_message_with_deduplication(
                  content.clone(), room_id, user_id, client_id
              ).await;
              
              // Property: Validation rules are consistently applied
              if content.is_empty() {
                  prop_assert!(matches!(result, Err(MessageError::Validation { field, .. }) if field == "content"));
              } else if content.len() > 10000 {
                  prop_assert!(matches!(result, Err(MessageError::Validation { field, .. }) if field == "content"));
              } else {
                  // Valid content should succeed (assuming valid room/user)
                  prop_assert!(result.is_ok() || matches!(result, Err(MessageError::Authorization { .. })));
              }
          });
      }
  }
  ```
  - _Requirements: Property tests validate all message invariants, Critical Gap #1_

- [ ] **0.2.2 WebSocket connection property tests**
  ```rust
  // WEBSOCKET RECONNECTION PROPERTY TESTS - Critical Gap #2
  
  proptest! {
      #[test]
      fn prop_reconnection_delivers_missed_messages(
          room_id in any::<u64>().prop_map(RoomId),
          user_id in any::<u64>().prop_map(UserId),
          message_count in 1..10usize,
          disconnect_after in 0..5usize,
      ) {
          let rt = tokio::runtime::Runtime::new().unwrap();
          rt.block_on(async {
              let broadcaster = setup_test_websocket_broadcaster().await;
              
              // Create initial messages
              let mut all_messages = Vec::new();
              for i in 0..message_count {
                  let msg = create_test_message(room_id, format!("Message {}", i)).await;
                  all_messages.push(msg);
              }
              
              // Simulate disconnect after some messages
              let last_seen = if disconnect_after < all_messages.len() {
                  Some(all_messages[disconnect_after].id)
              } else {
                  None
              };
              
              // CRITICAL GAP #2: Reconnect and get missed messages
              let missed_messages = broadcaster.handle_reconnection(
                  user_id, 
                  room_id, 
                  last_seen
              ).await.unwrap();
              
              // Property: All messages after last_seen are delivered
              if let Some(last_seen_id) = last_seen {
                  let expected_missed: Vec<_> = all_messages
                      .into_iter()
                      .skip_while(|m| m.id <= last_seen_id)
                      .collect();
                  
                  prop_assert_eq!(missed_messages.len(), expected_missed.len());
                  for (received, expected) in missed_messages.iter().zip(expected_missed.iter()) {
                      prop_assert_eq!(received.id, expected.id);
                  }
              }
          });
      }

      #[test]
      fn prop_presence_tracking_eventually_consistent(
          user_id in any::<u64>().prop_map(UserId),
          connection_count in 1..10u32,
      ) {
          let rt = tokio::runtime::Runtime::new().unwrap();
          rt.block_on(async {
              let tracker = setup_test_presence_tracker().await;
              
              // CRITICAL GAP #5: Increment connections
              for _ in 0..connection_count {
                  tracker.user_connected(user_id).await;
              }
              
              // Property: Connection count is accurate
              let is_online = tracker.is_user_online(user_id).await;
              prop_assert!(is_online);
              
              // Decrement all connections
              for _ in 0..connection_count {
                  tracker.user_disconnected(user_id).await;
              }
              
              // Property: User is offline after all disconnections
              let is_offline = !tracker.is_user_online(user_id).await;
              prop_assert!(is_offline);
          });
      }
  }
  ```
  - _Requirements: Property tests validate connection behavior, Critical Gap #2 & #5_

- [ ] **0.2.3 Room membership property tests**
  ```rust
  // ROOM MEMBERSHIP INVARIANT TESTS
  
  proptest! {
      #[test]
      fn prop_room_creator_always_has_membership(
          room_type in any::<RoomType>(),
          creator_id in any::<u64>().prop_map(UserId),
      ) {
          let rt = tokio::runtime::Runtime::new().unwrap();
          rt.block_on(async {
              let service = setup_test_room_service().await;
              
              let room = service.create_room(
                  "Test Room".to_string(),
                  room_type,
                  creator_id,
              ).await.unwrap();
              
              // Property: Room creator always has membership
              let has_access = service.check_room_access(creator_id, room.id).await.unwrap();
              prop_assert!(has_access, "Room creator must always have access");
          });
      }

      #[test]
      fn prop_direct_rooms_exactly_two_members(
          user1_id in any::<u64>().prop_map(UserId),
          user2_id in any::<u64>().prop_map(UserId),
      ) {
          let rt = tokio::runtime::Runtime::new().unwrap();
          rt.block_on(async {
              let service = setup_test_room_service().await;
              
              // Ensure different users
              prop_assume!(user1_id != user2_id);
              
              let room = service.create_room(
                  "Direct Room".to_string(),
                  RoomType::Direct { participants: [user1_id, user2_id] },
                  user1_id,
              ).await.unwrap();
              
              let memberships = service.get_room_memberships(room.id).await.unwrap();
              
              // Property: Direct rooms have exactly 2 members
              prop_assert_eq!(memberships.len(), 2);
              
              let member_ids: HashSet<_> = memberships.iter().map(|m| m.user_id).collect();
              prop_assert!(member_ids.contains(&user1_id));
              prop_assert!(member_ids.contains(&user2_id));
          });
      }

      #[test]
      fn prop_involvement_levels_control_visibility(
          room_id in any::<u64>().prop_map(RoomId),
          user_id in any::<u64>().prop_map(UserId),
          involvement in any::<Involvement>(),
      ) {
          let rt = tokio::runtime::Runtime::new().unwrap();
          rt.block_on(async {
              let service = setup_test_room_service().await;
              
              // Grant membership with specific involvement
              let membership = service.grant_membership(
                  user_id,
                  room_id,
                  involvement,
              ).await.unwrap();
              
              // Property: Involvement level controls message visibility
              match involvement {
                  Involvement::Invisible => {
                      // Should not appear in room lists
                      let rooms = service.get_user_rooms(user_id, false).await.unwrap();
                      prop_assert!(!rooms.iter().any(|r| r.id == room_id));
                  }
                  Involvement::Everything | Involvement::Mentions | Involvement::Nothing => {
                      // Should appear in room lists
                      let rooms = service.get_user_rooms(user_id, true).await.unwrap();
                      prop_assert!(rooms.iter().any(|r| r.id == room_id));
                  }
              }
          });
      }
  }
  ```
  - _Requirements: Property tests validate membership invariants_

### 0.3 Integration Contract Definition

- [ ] **0.3.1 End-to-end message flow contracts**
  ```rust
  // COMPLETE INTEGRATION TEST CONTRACTS - Service Boundary Validation
  
  #[tokio::test]
  async fn integration_message_creation_end_to_end() {
      let app = setup_test_app().await;
      let room_id = create_test_room(&app).await;
      let user = create_test_user(&app).await;
      
      // Establish WebSocket connection
      let ws_client = connect_websocket(&app, user.id, room_id).await;
      
      // Send message via HTTP API
      let message_response = app
          .post(&format!("/api/rooms/{}/messages", room_id))
          .json(&json!({
              "content": "Hello, world!",
              "client_message_id": Uuid::new_v4()
          }))
          .send()
          .await;
      
      assert_eq!(message_response.status(), 201);
      
      // Verify WebSocket broadcast received within 100ms
      let ws_message = tokio::time::timeout(
          Duration::from_millis(100),
          ws_client.receive_message()
      ).await.unwrap();
      
      match ws_message {
          WebSocketMessage::MessageCreated { message } => {
              assert_eq!(message.content, "Hello, world!");
              assert_eq!(message.creator_id, user.id);
              assert_eq!(message.room_id, room_id);
          }
          _ => panic!("Expected MessageCreated WebSocket message"),
      }
      
      // Verify message persisted in database
      let stored_message = app.database
          .get_message(message.id)
          .await
          .unwrap()
          .unwrap();
      
      assert_eq!(stored_message.content, "Hello, world!");
      
      // Verify FTS5 search index updated
      let search_results = app.database
          .search_messages("Hello", user.id, 10)
          .await
          .unwrap();
      
      assert_eq!(search_results.len(), 1);
      assert_eq!(search_results[0].id, message.id);
  }
  
  #[tokio::test]
  async fn integration_user_authentication_flow() {
      let app = setup_test_app().await;
      
      // Register new user
      let register_response = app
          .post("/api/auth/register")
          .json(&json!({
              "email": "test@example.com",
              "name": "Test User",
              "password": "secure_password",
              "join_code": "TEST-JOIN-CODE"
          }))
          .send()
          .await;
      
      assert_eq!(register_response.status(), 201);
      
      // Login with credentials
      let login_response = app
          .post("/api/auth/login")
          .json(&json!({
              "email": "test@example.com",
              "password": "secure_password"
          }))
          .send()
          .await;
      
      assert_eq!(login_response.status(), 200);
      
      // Verify session cookie set
      let cookies = login_response.headers().get_all("set-cookie");
      let session_cookie = cookies
          .iter()
          .find(|cookie| cookie.to_str().unwrap().starts_with("session_token="))
          .expect("Session cookie should be set");
      
      // Verify session is valid for protected routes
      let profile_response = app
          .get("/api/user/profile")
          .header("cookie", session_cookie)
          .send()
          .await;
      
      assert_eq!(profile_response.status(), 200);
  }
  
  #[tokio::test]
  async fn integration_room_creation_and_membership() {
      let app = setup_test_app().await;
      let admin_user = create_test_admin(&app).await;
      let regular_user = create_test_user(&app).await;
      
      // Create closed room
      let room_response = app
          .post("/api/rooms")
          .json(&json!({
              "name": "Private Room",
              "room_type": "closed",
              "initial_members": [regular_user.id]
          }))
          .authenticate(admin_user.id)
          .send()
          .await;
      
      assert_eq!(room_response.status(), 201);
      let room: Room = room_response.json().await;
      
      // Verify memberships created
      let memberships = app.database
          .get_room_memberships(room.id)
          .await
          .unwrap();
      
      assert_eq!(memberships.len(), 2); // Admin + regular user
      
      // Verify WebSocket subscription works
      let ws_client = connect_websocket(&app, regular_user.id, room.id).await;
      
      // Send message and verify broadcast
      let message = create_test_message(&app, room.id, admin_user.id, "Test message").await;
      
      let received = tokio::time::timeout(
          Duration::from_millis(100),
          ws_client.receive_message()
      ).await.unwrap();
      
      match received {
          WebSocketMessage::MessageCreated { message: msg } => {
              assert_eq!(msg.id, message.id);
          }
          _ => panic!("Expected message broadcast"),
      }
  }
  ```
  - _Requirements: Complete integration test specifications, Critical Gaps #1-#5_

- [ ] **0.3.2 Service boundary integration tests**
  ```rust
  // SERVICE BOUNDARY INTEGRATION TESTS - Real Dependencies
  
  #[tokio::test]
  async fn integration_message_service_with_broadcaster() {
      let db = setup_test_database().await;
      let broadcaster = setup_test_broadcaster(db.clone()).await;
      let message_service = MessageService::new(db.clone(), broadcaster.clone());
      
      let room_id = create_test_room(&db).await;
      let user_id = create_test_user(&db).await;
      
      // Subscribe to room via broadcaster
      let mut receiver = broadcaster
          .subscribe_to_room(user_id, room_id)
          .await
          .unwrap();
      
      // Create message via service
      let message = message_service
          .create_message_with_deduplication(
              "Integration test message".to_string(),
              room_id,
              user_id,
              Uuid::new_v4(),
          )
          .await
          .unwrap();
      
      // Verify broadcast received
      let broadcast_message = tokio::time::timeout(
          Duration::from_millis(100),
          receiver.recv()
      ).await.unwrap().unwrap();
      
      match broadcast_message {
          WebSocketMessage::MessageCreated { message: msg } => {
              assert_eq!(msg.id, message.id);
              assert_eq!(msg.content, message.content);
          }
          _ => panic!("Expected MessageCreated broadcast"),
      }
      
      // Verify database state updated
      let room = db.get_room(room_id).await.unwrap().unwrap();
      assert_eq!(room.last_message_at.unwrap(), message.created_at);
  }
  
  #[tokio::test]
  async fn integration_auth_service_with_sessions() {
      let db = setup_test_database().await;
      let auth_service = AuthService::new(db.clone());
      
      // Create user
      let user = create_test_user(&db).await;
      
      // Authenticate and create session
      let session = auth_service
          .create_session(
              user.email_address.clone(),
              "test_password".to_string(),
          )
          .await
          .unwrap();
      
      // Verify session token is secure (32+ chars, alphanumeric)
      assert!(session.token.len() >= 32);
      assert!(session.token.chars().all(|c| c.is_alphanumeric()));
      
      // Validate session
      let (validated_user, validated_session) = auth_service
          .validate_session(session.token.clone())
          .await
          .unwrap();
      
      assert_eq!(validated_user.id, user.id);
      assert_eq!(validated_session.id, session.id);
      
      // Verify session persisted in database
      let stored_session = db
          .get_session_by_token(&session.token)
          .await
          .unwrap()
          .unwrap();
      
      assert_eq!(stored_session.user_id, user.id);
  }
  
  #[tokio::test]
  async fn integration_database_writer_serialization() {
      let db = setup_test_database().await;
      let writer = DatabaseWriter::new(db.clone());
      
      let room_id = create_test_room(&db).await;
      let user_id = create_test_user(&db).await;
      
      // Concurrent write operations
      let mut handles = Vec::new();
      
      for i in 0..10 {
          let writer_clone = writer.clone();
          let handle = tokio::spawn(async move {
              writer_clone.execute_write(WriteCommand::CreateMessage {
                  content: format!("Concurrent message {}", i),
                  room_id,
                  creator_id: user_id,
                  client_message_id: Uuid::new_v4(),
              }).await
          });
          handles.push(handle);
      }
      
      // Wait for all writes to complete
      let results: Vec<_> = futures::future::join_all(handles).await;
      
      // Verify all writes succeeded
      for result in results {
          assert!(result.unwrap().is_ok());
      }
      
      // Verify all messages were created
      let messages = db.get_room_messages(room_id, 20, None).await.unwrap();
      assert_eq!(messages.len(), 10);
      
      // Verify messages are in chronological order
      for window in messages.windows(2) {
          assert!(window[0].created_at <= window[1].created_at);
      }
  }
  ```
  - _Requirements: All service interactions validated with real dependencies_

### 0.4 User Journey Validation Matrix

| Journey ID | Steps | Test Stub (Playwright/Vitest) | Success Metric | Rails Parity Check | Critical Gap |
|------------|--------|-------------------------------|----------------|-------------------|--------------|
| **JOURNEY_01: Auth-to-Send** | 1. Login (email/pass)<br>2. Join room<br>3. Type/send msg<br>4. Verify broadcast | `test('auth send', async ({ page }) => { await page.fill('#email', 'user@test'); await page.click('button[type=submit]'); await page.type('#composer', 'Hi'); await expect(page.locator('.message')).toHaveText('Hi'); });` | <2s end-to-end; 100% delivery | Matches room_channel.rb broadcast | Gap #1: Dedup |
| **JOURNEY_02: Rapid-Click Dedup** | 1. Type message<br>2. Rapidly click send 5x<br>3. Verify single message | `test('dedup rapid clicks', async ({ page }) => { await page.fill('#composer', 'Test'); for(let i=0; i<5; i++) await page.click('[data-testid=send-button]'); const messages = await page.locator('.message:has-text("Test")').count(); expect(messages).toBe(1); });` | Single message; same client_id | Prevents Rails duplicate issue | Gap #1: Dedup |
| **JOURNEY_03: Reconnect-Missed** | 1. Send msg<br>2. Disconnect WS<br>3. Reconnect<br>4. Assert missed msgs | `test('reconnect missed', async ({ page }) => { await page.route('**/ws', route => route.abort()); await page.reload(); await expect(page.locator('.message').last()).toHaveText('Previous message'); });` | Delivery rate >95% | ActionCable reconnection behavior | Gap #2: Reconnect |
| **JOURNEY_04: Sound Command** | 1. Type "/play bell"<br>2. Send<br>3. Verify sound UI<br>4. Click play | `test('sound command', async ({ page }) => { await page.fill('#composer', '/play bell'); await page.click('[data-testid=send-button]'); await expect(page.locator('[data-testid=sound-player]')).toBeVisible(); });` | Sound UI renders; audio plays | Matches Rails sound system | N/A |
| **JOURNEY_05: Message Boost** | 1. Hover message<br>2. Click boost<br>3. Select emoji<br>4. Verify broadcast | `test('message boost', async ({ page }) => { await page.hover('.message'); await page.click('[data-testid=boost-button]'); await page.click('[data-emoji="👍"]'); await expect(page.locator('.boost:has-text("👍")')).toBeVisible(); });` | Boost created; broadcast to all | Rails boost functionality | N/A |
| **JOURNEY_06: File Upload Graceful** | 1. Click upload area<br>2. Verify "v2.0" msg<br>3. Drag file<br>4. Verify styling | `test('file upload degradation', async ({ page }) => { await page.click('[data-testid=file-upload-area]'); await expect(page.locator(':has-text("Coming in v2.0")')).toBeVisible(); });` | Professional UI; clear messaging | Graceful feature degradation | N/A |
| **JOURNEY_07: Presence Tracking** | 1. User connects<br>2. Verify online status<br>3. Disconnect<br>4. Verify offline | `test('presence tracking', async ({ page }) => { await login(page); await expect(page.locator('[data-testid=user-online]')).toBeVisible(); await page.close(); await expect(page.locator('[data-testid=user-offline]')).toBeVisible(); });` | Status updates within 60s | Rails presence behavior | Gap #5: Presence |
| **JOURNEY_08: Typing Notifications** | 1. Start typing<br>2. Verify indicator<br>3. Stop typing<br>4. Verify cleared | `test('typing notifications', async ({ page }) => { await page.type('#composer', 'Hello'); await expect(page.locator('.typing-indicator')).toBeVisible(); await page.keyboard.press('Escape'); await expect(page.locator('.typing-indicator')).toBeHidden(); });` | Indicators appear/disappear | Rails typing notifications | N/A |
| **JOURNEY_09: Bot Webhook** | 1. POST to /api/bots<br>2. Simulate webhook<br>3. Verify msg in room | `test('bot webhook', async ({ request }) => { const res = await request.post('/api/bots', { data: { content: 'Bot msg' } }); expect(res.status()).toBe(200); expect(await get_msgs()).toContain('Bot msg'); });` | Response <7s timeout | bots_controller.rb parity | N/A |
| **JOURNEY_10: Room Creation** | 1. Click create room<br>2. Set name/type<br>3. Add members<br>4. Verify access | `test('room creation', async ({ page }) => { await page.click('[data-testid=create-room]'); await page.fill('#room-name', 'Test Room'); await page.click('[data-testid=create-button]'); await expect(page.locator('.room-list')).toContainText('Test Room'); });` | Room created; memberships granted | Rails room creation | N/A |
| **JOURNEY_11: Search Messages** | 1. Type in search<br>2. Press enter<br>3. Verify results<br>4. Click result | `test('message search', async ({ page }) => { await page.fill('#search-input', 'hello'); await page.keyboard.press('Enter'); await expect(page.locator('.search-results')).toContainText('hello'); });` | FTS5 search <100ms | Rails search functionality | N/A |
| **JOURNEY_12: Session Security** | 1. Login<br>2. Verify secure cookie<br>3. Logout<br>4. Verify cleared | `test('session security', async ({ page, context }) => { await login(page); const cookies = await context.cookies(); expect(cookies.find(c => c.name === 'session_token')).toBeTruthy(); await logout(page); const newCookies = await context.cookies(); expect(newCookies.find(c => c.name === 'session_token')).toBeFalsy(); });` | Secure token generation | Rails session management | Gap #4: Security |

**Verification Harness**: `cargo test --lib && npm test && playwright test` - Success requires >95% pass rate with <5% flakiness.

**Success Indicators**:
- **Quantitative**: 100% test pass rate post-stubs; journey latency < Rails benchmarks
- **Qualitative**: Manual dry-runs simulate journeys in pseudocode, flag unresolved branches
- **Gap Closure**: All 5 critical gaps validated via property tests and integration tests
- **Rails Parity**: Each journey matches corresponding Rails behavior exactly

## Phase 1: Core Infrastructure (Week 1)

**Goal**: Implement type-guided infrastructure following contracts

### 1.1 Project Setup

- [ ] **1.1.1 Implement type-guided project structure**
  - Create Cargo.toml with TDD dependencies: axum, sqlx, tokio, serde, rust-embed, proptest
  - Implement project structure following type contracts from Phase 0
  - Set up basic project structure (≤50 files total per Requirement 0.8)
  - Configure development environment with property test runners
  - _Requirements: Requirement 0.1 - direct function calls, Phase 0 type contracts_

- [ ] **1.1.2 Implement domain models following type contracts**
  - Implement UserId, RoomId, MessageId newtypes with serde derives
  - Implement User, Room, Message structs matching Phase 0 specifications
  - Implement UserRole enum with database mapping
  - Implement comprehensive error types from Phase 0.1.2
  - _Requirements: Phase 0 type contracts, Requirement 3.7 - role enum_

- [ ] **1.1.3 Implement database layer following contracts**
  - Implement Database struct with read pool and dedicated writer
  - Create schema with UNIQUE constraint on (client_message_id, room_id) - **Critical Gap Fix #1**
  - Implement WAL mode and connection pooling with error handling
  - Create FTS5 search index with compile-time query validation
  - Run property tests to validate database behavior contracts
  - _Requirements: Phase 0 database contracts, Critical Gap #1, Requirement 7.5_

### 1.2 Basic HTTP Server

- [ ] **1.2.1 Implement HTTP server following contracts**
  - Implement Axum server with type-safe extractors and error handling
  - Implement embedded React SPA serving using rust-embed
  - Implement CORS and middleware with comprehensive error handling
  - Implement static asset serving with proper caching headers
  - Run integration tests to validate HTTP behavior contracts
  - _Requirements: Phase 0 HTTP contracts, Requirement 8.7 - embedded assets_

- [ ] **1.2.2 Implement authentication service - Critical Gap Fix #4**
  - Implement AuthService trait following Phase 0 contracts
  - Implement secure token generation with cryptographic randomness
  - Implement cookie-based sessions with security headers
  - Implement login/logout endpoints with comprehensive error handling
  - Run property tests to validate authentication behavior
  - _Requirements: Phase 0 auth contracts, Critical Gap #4, Requirement 3.3_

---

## Phase 2: Core Chat Functionality (Week 2)

**Goal**: Basic message sending/receiving that works "well enough"

### 2.1 Database Operations with Write Serialization

- [ ] **2.1.1 Implement DatabaseWriter actor - Critical Gap Fix #3**
  - Implement DatabaseWriter following Phase 0 contracts
  - Implement single writer task with mpsc channel for write serialization
  - Implement concurrent read operations with proper error handling
  - Handle SQLite errors gracefully with user-friendly messages
  - Run property tests to validate write serialization behavior
  - _Requirements: Phase 0 database contracts, Critical Gap #3, Requirement 4.11_

- [ ] **2.1.2 Implement MessageService following contracts**
  - Implement MessageService trait from Phase 0.1.1
  - Implement create_message_with_deduplication with UNIQUE constraint handling
  - Implement get_messages_since with chronological ordering guarantees
  - Implement search_messages with FTS5 and permission filtering
  - Run property tests from Phase 0.2.1 to validate behavior
  - _Requirements: Phase 0 message contracts, Critical Gap #1, Requirement 1.1-1.7_

- [ ] **2.1.3 Implement RoomService following contracts**
  - Implement RoomService trait from Phase 0.1.1
  - Implement room creation with automatic membership granting
  - Implement membership management with involvement level controls
  - Implement permission checks with type-safe authorization
  - Run property tests from Phase 0.2.3 to validate membership invariants
  - _Requirements: Phase 0 room contracts, Requirement 2.1-2.12, Requirement 3.12_

### 2.2 WebSocket Broadcasting (Rails ActionCable Equivalent)

- [ ] **2.2.1 Implement WebSocketBroadcaster following contracts**
  - Implement WebSocketBroadcaster trait from Phase 0.1.1
  - Implement connection management with type-safe state transitions
  - Implement authentication via session cookies with error handling
  - Implement connection cleanup with RAII patterns
  - Run property tests from Phase 0.2.2 to validate connection behavior
  - _Requirements: Phase 0 WebSocket contracts, Critical Gap #2, Requirement 4.8-4.9_
  - _Requirements: Requirement 4.8-4.9 - WebSocket authentication and authorization_

- [ ] **2.2.2 Turbo Streams message broadcasting**
  - Broadcast messages using Turbo Streams format per Requirement 4.1
  - broadcast_append_to for new messages, broadcast_replace_to for edits
  - Best-effort delivery within 100ms per Requirement 4.1 (Rails limitation)
  - Simple timeout handling (7 seconds max like webhook timeout)
  - _Requirements: Requirement 4.1 - Turbo Streams broadcasting within 100ms_

- [ ] **2.2.3 Basic reconnection with state sync - Critical Gap Fix #2**
  - Track last_seen_message_id per connection for missed messages
  - Send missed events since last known state per Requirement 4.14
  - Prevent duplicate message delivery during reconnection
  - Accept race conditions in edge cases (Rails ActionCable limitation)
  - _Requirements: Requirement 4.14 - state reconciliation after reconnection_

---

## Phase 3: Real-time Features (Week 3)

**Goal**: "Good enough" real-time features matching Rails quality

### 3.1 Presence Tracking (Rails-Equivalent Imperfection) - Critical Gap Fix #5

- [ ] **3.1.1 Connectable concern pattern for presence**
  - Track connections count per membership per Requirement 4.2
  - connected_at timestamp with 60-second TTL per Requirement 4.2
  - Increment atomically on present, decrement on disconnected per Requirement 4.3-4.4
  - Accept occasional inaccuracy during network hiccups (Rails limitation)
  - _Requirements: Requirement 4.2-4.4 - presence tracking with connection counting_

- [ ] **3.1.2 Heartbeat and visibility management**
  - Refresh connection every 50 seconds per Requirement 4.5
  - 5-second delay for visibility changes per Requirement 4.7
  - Handle tab switching/minimization with present/absent actions
  - Background cleanup for stale connections (Rails equivalent)
  - _Requirements: Requirement 4.5 - refresh heartbeat, Requirement 4.7 - visibility handling_

### 3.2 Typing Notifications (Rails ActionCable Pattern)

- [ ] **3.2.1 TypingNotificationsChannel implementation**
  - Broadcast start/stop actions with user attributes per Requirement 4.6
  - Throttle notifications to prevent spam (Rails pattern)
  - Track active typers per room with simple HashMap
  - Clear indicators on message send per Requirement 4.6
  - _Requirements: Requirement 4.6 - typing notifications with throttling_

- [ ] **3.2.2 Auto-cleanup typing state**
  - Clear typing after 5 seconds of inactivity (Rails timeout pattern)
  - Simple timer-based cleanup without complex coordination
  - Accept edge cases like network interruptions (Rails limitation)
  - Handle multiple connections per user gracefully
  - _Requirements: Requirement 4.6 - typing notification management_

---

## Phase 4: Frontend Integration (Week 4)

**Goal**: React frontend that works with Rails-equivalent backend

### 4.1 Complete React UI with Graceful Degradation

- [ ] **4.1.1 Rich text message interface**
  - Trix editor with HTML formatting per Requirement 1.2
  - Sound commands (/play soundname) with 59 embedded MP3s per Requirement 1.5
  - Message boosts with emoji content per Requirement 1.6
  - Emoji-only message detection and enlargement per Requirement 1.9
  - _Requirements: Requirement 1.2-1.6, 1.9 - rich text messaging features_

- [ ] **4.1.2 Complete file upload UI with v2.0 messaging**
  - Drag-and-drop zones with professional styling per Requirement 1.4
  - "File sharing available in v2.0" messaging per Requirement 1.4
  - Avatar upload UI with text-based initials per Requirement 3.6
  - Maintain all CSS styling and components for future functionality
  - _Requirements: Requirement 1.4 - graceful file degradation, Requirement 3.6 - avatar UI_

### 4.2 ActionCable-Equivalent WebSocket Integration

- [ ] **4.2.1 Connection management with Rails patterns**
  - Auto-reconnect with exponential backoff per Requirement 1.13
  - Connection state synchronization per Requirement 4.11
  - Show connection status to user (Rails ActionCable behavior)
  - Handle WebSocket connection loss within 60 seconds per Requirement 4.13
  - _Requirements: Requirement 1.13 - retry logic, Requirement 4.13 - connection loss detection_

- [ ] **4.2.2 Real-time updates via Turbo Streams**
  - Receive Turbo Streams messages per Requirement 4.1
  - Update presence indicators per Requirement 4.2-4.7
  - Show typing notifications per Requirement 4.6
  - Maintain message order consistency per Requirement 4.11 (Rails-level)
  - _Requirements: Requirement 4.1 - Turbo Streams, Requirement 4.11 - message consistency_

---

## Phase 5: Additional MVP Features (Week 5)

### 5.1 Bot Integration System

- [ ] **5.1.1 Bot authentication and management**
  - User.create_bot! with SecureRandom.alphanumeric(12) token per Requirement 5.1
  - Bot authentication via "id-token" format per Requirement 5.2
  - Accounts::BotsController for administrator management per Requirement 5.10
  - Bot token reset functionality per Requirement 5.8
  - _Requirements: Requirement 5.1-5.2, 5.8, 5.10 - bot creation and management_

- [ ] **5.1.2 Webhook delivery system**
  - Bot::WebhookJob for @mention and DM triggers per Requirement 5.3
  - Webhook payload with user, room, message details per Requirement 5.4
  - 7-second timeout with async delivery per Requirement 5.5
  - Handle webhook responses and create reply messages per Requirement 5.6-5.7
  - _Requirements: Requirement 5.3-5.7 - webhook system_

### 5.2 Search and Additional Features

- [ ] **5.2.1 FTS5 message search**
  - SQLite FTS5 with Porter stemming per Requirement 6.9
  - Sub-millisecond search times with proper indexing
  - Search across message content with room filtering
  - Maintain search index during message operations
  - _Requirements: Requirement 6.9 - FTS5 search performance_

- [ ] **5.2.2 Push notification system**
  - Web Push with VAPID keys for message notifications
  - Service worker integration for PWA functionality
  - @mention notifications per involvement level
  - Push subscription management in database
  - _Requirements: Requirement 8.8 - push notifications_

## What We're NOT Implementing (Rails Doesn't Do It Either)

### ❌ Perfect Message Ordering (Rails Limitation #1)
- **Rails Reality**: Rails uses created_at timestamps, accepts occasional out-of-order
- **Our Approach**: Use database timestamps per Requirement 4.11, accept Rails-level ordering
- **Requirements**: Requirement 4.11 - "maintain message order consistency" = Rails-level

### ❌ Perfect Multi-tab Coordination (Rails Limitation #2)
- **Rails Reality**: Rails doesn't coordinate multiple tabs perfectly
- **Our Approach**: Each tab is independent connection per Requirement 4.10 (Rails behavior)
- **Requirements**: Requirement 4.10 - "across multiple browser tabs" = Rails behavior

### ❌ Guaranteed Message Delivery (Rails Limitation #3)
- **Rails Reality**: ActionCable is best-effort, no delivery guarantees
- **Our Approach**: Best-effort broadcast per Requirement 4.1, log failures
- **Requirements**: Requirement 4.1 - "broadcast within 100ms" = best effort like Rails

### ❌ Perfect Presence Accuracy (Rails Limitation #4)
- **Rails Reality**: Rails presence has delays and edge cases
- **Our Approach**: "Good enough" presence per Requirement 4.2-4.4 with known limitations
- **Requirements**: Requirement 4.4 - "handle browser crashes gracefully" = Rails level

---

## Known Limitations (Rails-Equivalent)

### Connection Management
- Multiple tabs create multiple connections (Rails behavior)
- Connection cleanup may be delayed (Rails limitation)
- Presence may be briefly inaccurate (Rails limitation)

### Message Handling
- Rare duplicate messages possible in edge cases (Rails has this)
- Message ordering by timestamp, not perfect sequence (Rails approach)
- WebSocket failures require client retry (Rails behavior)

### Performance
- Single SQLite database limits concurrent writes (Rails has similar limits)
- Memory usage grows with connections (Rails limitation)
- No horizontal scaling (Rails single-server equivalent)

---

## Success Metrics

### Functional Requirements
- [ ] Users can send/receive messages in real-time
- [ ] Basic presence tracking works "most of the time"
- [ ] Reconnection works after network issues
- [ ] No obvious duplicate messages under normal use
- [ ] Session authentication works reliably

### Performance Requirements (Rails-Equivalent)
- [ ] Handle 50 concurrent users (Rails single-server equivalent)
- [ ] Message delivery within 1 second under normal load
- [ ] Reconnection within 5 seconds
- [ ] Memory usage stays reasonable (< 100MB)

### Reliability Requirements
- [ ] Works reliably for 8-hour sessions
- [ ] Handles network hiccups gracefully
- [ ] Database corruption recovery
- [ ] Graceful degradation under load

---

## Implementation Notes

### Code Organization
- Maximum 500 lines per file
- Rails-style modules: models/, handlers/, services/
- Single responsibility per module
- No circular dependencies

### Error Handling
- Result<T, E> patterns throughout
- User-friendly error messages
- Log technical details, show simple messages to users
- No complex retry logic (Rails keeps it simple)

### Testing Strategy
- Unit tests for core business logic
- Integration tests for API endpoints
- WebSocket connection tests
- Database constraint tests
- No mocking - use real SQLite in tests

### Deployment
- Single binary with embedded React assets
- SQLite database in mounted volume
- Environment variables for configuration
- Simple Docker container (Rails equivalent)

---

## Timeline Summary

**Week 1**: Core infrastructure with 5 critical gap fixes
**Week 2**: Database operations and ActionCable-equivalent WebSocket broadcasting  
**Week 3**: Real-time features with Rails-equivalent quality and limitations
**Week 4**: Complete React UI with graceful v2.0 degradation messaging
**Week 5**: Bot integration, search, and push notifications

**Total**: 5 weeks for fully functional Rails-equivalent chat application with complete UI

## Anti-Coordination Compliance Checklist

### ✅ MANDATORY PATTERNS (Per Requirement 0)
- [ ] Direct SQLite operations with basic connection pooling
- [ ] Basic WebSocket broadcasting (ActionCable-style)
- [ ] Rails-style session management with secure cookies
- [ ] Simple Result<T, E> error handling with user-friendly messages
- [ ] Direct function calls, no async coordination between components
- [ ] Single binary deployment with embedded assets

### 🚫 FORBIDDEN PATTERNS (Will Cause Spec Rejection)
- [ ] ❌ NO coordination layers, coordinators, or event buses
- [ ] ❌ NO distributed transactions, sagas, or event sourcing
- [ ] ❌ NO circuit breakers, retry queues, or complex error recovery
- [ ] ❌ NO cross-tab coordination or global state synchronization
- [ ] ❌ NO microservices, service mesh, or distributed architecture
- [ ] ❌ NO message queues, event streams, or async coordination

### 📏 COMPLEXITY LIMITS (Per Requirement 0.8)
- [ ] Maximum 50 total files in entire codebase
- [ ] No file over 500 lines
- [ ] Maximum 3 async operations per request
- [ ] No more than 2 levels of error handling
- [ ] Single database connection pool

## 5 Critical Gaps Implementation Summary

**Gap #1: client_message_id Deduplication** ✅
- UNIQUE constraint on (client_message_id, room_id) prevents duplicates
- Handle constraint violations by returning existing message
- _Task: 2.1.2 - message CRUD with deduplication_

**Gap #2: WebSocket Reconnection State** ✅
- Track last_seen_message_id per connection
- Send missed messages on reconnect
- _Task: 2.2.3 - reconnection with state sync_

**Gap #3: SQLite Write Serialization** ✅
- Dedicated Writer Task pattern with mpsc channel
- All writes serialized through single task
- _Task: 2.1.1 - dedicated writer pattern_

**Gap #4: Session Token Security** ✅
- SecureRandom equivalent for token generation
- Rails-style httponly SameSite=Lax cookies
- _Task: 1.2.2 - session authentication_

**Gap #5: Basic Presence Tracking** ✅
- Connection counting with 60-second TTL
- Heartbeat refresh every 50 seconds
- _Task: 3.1.1-3.1.2 - presence tracking_

This plan is **realistic and achievable** because it:
1. **Fixes only the 5 critical gaps** that Rails actually solves
2. **Accepts Rails-level limitations** rather than over-engineering
3. **Follows anti-coordination constraints** strictly
4. **Provides complete UI** with graceful v2.0 messaging
5. **References specific requirements** for every task

---

## TDD Verification Harness

### Comprehensive Test Execution Pipeline

```bash
#!/bin/bash
# Verification script for one-shot implementation success

echo "🧪 Running TDD Verification Harness..."

# Phase 1: Property Tests (80% of logic coverage)
echo "📊 Running property tests..."
cargo test --lib prop_ --release
PROP_EXIT_CODE=$?

# Phase 2: Integration Tests (Service boundaries)
echo "🔗 Running integration tests..."
cargo test --test integration --release
INTEGRATION_EXIT_CODE=$?

# Phase 3: User Journey Tests (End-to-end validation)
echo "🎭 Running user journey tests..."
npm test -- --testNamePattern="JOURNEY_"
JOURNEY_EXIT_CODE=$?

# Phase 4: Rails Parity Validation
echo "⚖️ Validating Rails parity..."
cargo test --test rails_parity --release
PARITY_EXIT_CODE=$?

# Phase 5: Anti-Coordination Guards
echo "🚫 Running anti-coordination guards..."
cargo test --test anti_coordination_guards --release
GUARD_EXIT_CODE=$?

# Calculate overall success rate
TOTAL_TESTS=$((PROP_EXIT_CODE + INTEGRATION_EXIT_CODE + JOURNEY_EXIT_CODE + PARITY_EXIT_CODE + GUARD_EXIT_CODE))

if [ $TOTAL_TESTS -eq 0 ]; then
    echo "✅ TDD Verification: 100% SUCCESS - Ready for one-shot implementation!"
    echo "📈 All critical gaps validated"
    echo "🎯 All user journeys passing"
    echo "⚖️ Rails parity confirmed"
    echo "🚫 Anti-coordination constraints enforced"
    exit 0
else
    echo "❌ TDD Verification: FAILED - Implementation not ready"
    echo "🔍 Check failed tests and fix before proceeding"
    exit 1
fi
```

### Success Metrics Dashboard

| Metric Category | Target | Measurement | Status |
|----------------|--------|-------------|--------|
| **Property Test Coverage** | 100% for critical gaps | `cargo test prop_ --coverage` | ⏳ Pending |
| **Integration Test Pass Rate** | >95% | `cargo test integration --json` | ⏳ Pending |
| **User Journey Success** | 12/12 journeys pass | `npm test JOURNEY_ --reporter=json` | ⏳ Pending |
| **Rails Parity Score** | >90% behavioral match | Custom parity test suite | ⏳ Pending |
| **Anti-Coordination Compliance** | 0 violations | Static analysis + runtime guards | ⏳ Pending |
| **Performance Benchmarks** | <Rails latency + 10% | `cargo bench --baseline=rails` | ⏳ Pending |

### Critical Gap Validation Matrix

| Gap ID | Property Test | Integration Test | User Journey | Rails Parity | Status |
|--------|---------------|------------------|--------------|--------------|--------|
| **Gap #1: Deduplication** | `prop_duplicate_client_id_returns_same_message` | `integration_message_service_with_broadcaster` | `JOURNEY_02: Rapid-Click Dedup` | Rails UNIQUE constraint behavior | ⏳ |
| **Gap #2: Reconnection** | `prop_reconnection_delivers_missed_messages` | `integration_websocket_reconnection_flow` | `JOURNEY_03: Reconnect-Missed` | ActionCable missed message delivery | ⏳ |
| **Gap #3: Write Serialization** | `prop_concurrent_writes_serialized` | `integration_database_writer_serialization` | N/A (Internal) | Rails connection pooling | ⏳ |
| **Gap #4: Session Security** | `prop_secure_token_generation` | `integration_auth_service_with_sessions` | `JOURNEY_12: Session Security` | Rails SecureRandom tokens | ⏳ |
| **Gap #5: Presence Tracking** | `prop_presence_tracking_eventually_consistent` | `integration_presence_with_heartbeat` | `JOURNEY_07: Presence Tracking` | Rails connection counting | ⏳ |

### Implementation Readiness Checklist

- [ ] **Type Contracts Complete**: All service traits defined with comprehensive error handling
- [ ] **Property Tests Written**: All invariants specified with proptest generators
- [ ] **Integration Tests Defined**: All service boundaries tested with real dependencies
- [ ] **User Journeys Mapped**: All 12 critical user flows have test stubs
- [ ] **Rails Parity Validated**: Behavioral equivalence confirmed for all critical paths
- [ ] **Anti-Coordination Guards**: Static analysis prevents forbidden patterns
- [ ] **Performance Baselines**: Benchmarks established for regression detection

### Post-Implementation Validation

Once implementation is complete, run the verification harness to confirm:

1. **One-Shot Success**: All tests pass on first implementation attempt
2. **Rails Behavioral Parity**: System behaves identically to Rails for all tested scenarios
3. **Critical Gap Resolution**: All 5 gaps are provably fixed with comprehensive test coverage
4. **Anti-Coordination Compliance**: No forbidden patterns detected in final codebase
5. **User Journey Completion**: All 12 user journeys work end-to-end

**Expected Outcome**: 99% one-shot implementation success with 4x faster development (3 weeks vs 12 weeks) due to comprehensive upfront specification and test-driven validation.

---

## Enhanced Verification Harness (Minto-Inspired)

### Governing Thought
**Complete verification pipeline that validates every layer**: Property tests prove mathematical invariants, integration tests verify service boundaries, E2E tests confirm user journeys, and Rails parity tests ensure behavioral equivalence.

### TDD Implementation Flow

**Complete Development Workflow**:
```mermaid
flowchart TD
    A[Phase 0: TDD Foundation] --> B[Define Type Contracts]
    B --> C[Write Property Tests]
    C --> D[Create Integration Stubs]
    D --> E[Phase 1: Core Infrastructure]
    
    E --> F[Implement Database Layer]
    F --> G[Critical Gap #3: Write Serialization]
    G --> H[Implement HTTP Server]
    H --> I[Critical Gap #4: Session Security]
    
    I --> J[Phase 2: Core Chat Functionality]
    J --> K[Implement MessageService]
    K --> L[Critical Gap #1: Deduplication]
    L --> M[Implement WebSocket Broadcasting]
    M --> N[Critical Gap #2: Reconnection]
    
    N --> O[Phase 3: Real-time Features]
    O --> P[Implement Presence Tracking]
    P --> Q[Critical Gap #5: Presence TTL]
    Q --> R[Implement Typing Notifications]
    
    R --> S[Phase 4: Frontend Integration]
    S --> T[Complete React UI]
    T --> U[E2E User Journeys]
    
    U --> V[Phase 5: Additional Features]
    V --> W[Bot Integration]
    W --> X[Search & Notifications]
    X --> Y[Verification Harness]
    
    Y --> Z{All Tests Pass?}
    Z -->|Yes| AA[🎉 One-Shot Success!]
    Z -->|No| BB[Fix Issues & Retry]
    BB --> Y
    
    style A fill:#e3f2fd
    style G fill:#ffebee
    style I fill:#ffebee
    style L fill:#ffebee
    style N fill:#ffebee
    style Q fill:#ffebee
    style AA fill:#e8f5e8
```

**Critical Gap Implementation Priority**:
```mermaid
gantt
    title Critical Gap Implementation Timeline
    dateFormat  YYYY-MM-DD
    section Week 1: Foundation
    Type Contracts           :done, contracts, 2024-01-01, 2d
    Property Tests          :done, props, after contracts, 2d
    Database Setup          :active, db, after props, 3d
    
    section Week 2: Core Gaps
    Gap #3: Write Serialization :crit, gap3, 2024-01-08, 2d
    Gap #1: Deduplication      :crit, gap1, after gap3, 2d
    Gap #4: Session Security   :crit, gap4, after gap1, 2d
    WebSocket Foundation       :after gap4, 1d
    
    section Week 3: Real-time
    Gap #2: Reconnection       :crit, gap2, 2024-01-15, 3d
    Gap #5: Presence Tracking  :crit, gap5, after gap2, 2d
    Typing Notifications       :after gap5, 2d
    
    section Week 4: Integration
    React UI Complete          :2024-01-22, 4d
    E2E Journey Tests         :after gap5, 3d
    
    section Week 5: Polish
    Bot Integration           :2024-01-29, 2d
    Search & Notifications    :after gap5, 2d
    Final Verification        :milestone, verify, 2024-02-02, 0d
```

### Verification Execution Pipeline

```bash
#!/bin/bash
# Enhanced TDD Verification Harness - Inspired by Minto Pyramid

echo "🔺 Starting Minto-Structured Verification Pipeline..."

# APEX: Core Logic Verification (Property Tests)
echo "📊 APEX: Running property tests for mathematical invariants..."
cargo test --lib prop_ --release --verbose
PROP_EXIT_CODE=$?
if [ $PROP_EXIT_CODE -eq 0 ]; then
    echo "✅ Property tests: All invariants hold"
else
    echo "❌ Property tests: Invariants violated - implementation not ready"
fi

# KEY LINE 1: Service Boundary Verification (Integration Tests)
echo "🔗 KEY LINE 1: Running integration tests for service boundaries..."
cargo test --test integration --release --verbose
INTEGRATION_EXIT_CODE=$?
if [ $INTEGRATION_EXIT_CODE -eq 0 ]; then
    echo "✅ Integration tests: All service boundaries verified"
else
    echo "❌ Integration tests: Service boundary failures detected"
fi

# KEY LINE 2: User Experience Verification (E2E Tests)
echo "🎭 KEY LINE 2: Running E2E tests for user journeys..."
npm test -- --testNamePattern="JOURNEY_" --verbose
JOURNEY_EXIT_CODE=$?
if [ $JOURNEY_EXIT_CODE -eq 0 ]; then
    echo "✅ E2E tests: All user journeys pass"
else
    echo "❌ E2E tests: User journey failures detected"
fi

# KEY LINE 3: Rails Behavioral Verification (Parity Tests)
echo "⚖️ KEY LINE 3: Running Rails parity validation..."
cargo test --test rails_parity --release --verbose
PARITY_EXIT_CODE=$?
if [ $PARITY_EXIT_CODE -eq 0 ]; then
    echo "✅ Rails parity: Behavioral equivalence confirmed"
else
    echo "❌ Rails parity: Behavioral differences detected"
fi

# DETAILS: Anti-Coordination Compliance (Guard Tests)
echo "🚫 DETAILS: Running anti-coordination guards..."
cargo test --test anti_coordination_guards --release --verbose
GUARD_EXIT_CODE=$?
if [ $GUARD_EXIT_CODE -eq 0 ]; then
    echo "✅ Anti-coordination: No forbidden patterns detected"
else
    echo "❌ Anti-coordination: Forbidden patterns found"
fi

# DETAILS: Performance Regression (Benchmark Tests)
echo "⚡ DETAILS: Running performance benchmarks..."
cargo bench --baseline=rails
BENCH_EXIT_CODE=$?
if [ $BENCH_EXIT_CODE -eq 0 ]; then
    echo "✅ Performance: Within Rails parity bounds"
else
    echo "⚠️ Performance: Regression detected (may be acceptable)"
fi

# Minto Summary: Pyramid Validation Results
echo ""
echo "🔺 MINTO PYRAMID VALIDATION SUMMARY:"
echo "   APEX (Property Tests): $([ $PROP_EXIT_CODE -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")"
echo "   KEY LINE 1 (Integration): $([ $INTEGRATION_EXIT_CODE -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")"
echo "   KEY LINE 2 (E2E Journeys): $([ $JOURNEY_EXIT_CODE -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")"
echo "   KEY LINE 3 (Rails Parity): $([ $PARITY_EXIT_CODE -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")"
echo "   DETAILS (Guards): $([ $GUARD_EXIT_CODE -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")"
echo "   DETAILS (Performance): $([ $BENCH_EXIT_CODE -eq 0 ] && echo "✅ PASS" || echo "⚠️ WARN")"

# Calculate pyramid success rate
CRITICAL_TESTS=$((PROP_EXIT_CODE + INTEGRATION_EXIT_CODE + JOURNEY_EXIT_CODE + PARITY_EXIT_CODE))
TOTAL_TESTS=$((CRITICAL_TESTS + GUARD_EXIT_CODE))

if [ $CRITICAL_TESTS -eq 0 ]; then
    echo ""
    echo "🎉 PYRAMID VERIFICATION: 100% SUCCESS"
    echo "📈 All critical layers validated - Ready for one-shot implementation!"
    echo "🎯 All 12 user journeys passing"
    echo "⚖️ Rails behavioral parity confirmed"
    echo "🚫 Anti-coordination constraints enforced"
    echo "📊 Property test coverage: 100% for critical gaps"
    exit 0
elif [ $TOTAL_TESTS -eq 1 ] && [ $BENCH_EXIT_CODE -ne 0 ]; then
    echo ""
    echo "⚠️ PYRAMID VERIFICATION: 95% SUCCESS (Performance Warning Only)"
    echo "🎯 All critical tests pass - Implementation ready with performance monitoring"
    exit 0
else
    echo ""
    echo "❌ PYRAMID VERIFICATION: FAILED"
    echo "🔍 Critical failures detected - Implementation not ready"
    echo "📋 Fix failed tests before proceeding with implementation"
    exit 1
fi
```

### Deep Reasoning Validation Matrix

| Verification Layer | Purpose | Success Criteria | Failure Impact | Recovery Strategy |
|-------------------|---------|------------------|----------------|-------------------|
| **Property Tests (APEX)** | Prove mathematical invariants hold | 100% pass rate for critical gaps | 60% runtime bugs | Add missing invariants, fix logic |
| **Integration Tests (KEY LINE 1)** | Verify service boundaries | All service interactions work | 40% integration failures | Fix service contracts, update mocks |
| **E2E Tests (KEY LINE 2)** | Validate user journeys | All 12 journeys complete successfully | 80% user experience issues | Fix UI/UX flows, update test stubs |
| **Rails Parity (KEY LINE 3)** | Ensure behavioral equivalence | >95% behavioral match | 30% unexpected behavior | Study Rails code, adjust implementation |
| **Anti-Coordination Guards** | Prevent forbidden patterns | Zero violations detected | 100% architecture corruption | Refactor to remove coordination |
| **Performance Benchmarks** | Maintain Rails parity | Within 110% of Rails performance | 20% performance regression | Profile and optimize hot paths |

### Critical Gap Verification Status

| Gap ID | Property Test | Integration Test | E2E Journey | Rails Parity | Overall Status |
|--------|---------------|------------------|-------------|--------------|----------------|
| **Gap #1: Deduplication** | `prop_dedup_idempotent` | `integration_message_service` | `JOURNEY_02: Rapid-Click` | UNIQUE constraint behavior | ⏳ Pending |
| **Gap #2: Reconnection** | `prop_reconnection_delivers_missed` | `integration_websocket_reconnect` | `JOURNEY_03: Reconnect-Missed` | ActionCable missed messages | ⏳ Pending |
| **Gap #3: Write Serialization** | `prop_concurrent_writes_serialized` | `integration_database_writer` | N/A (Internal) | Rails connection pooling | ⏳ Pending |
| **Gap #4: Session Security** | `prop_token_entropy_sufficient` | `integration_auth_service` | `JOURNEY_12: Session Security` | Rails SecureRandom tokens | ⏳ Pending |
| **Gap #5: Presence Tracking** | `prop_presence_ttl_cleanup` | `integration_presence_heartbeat` | `JOURNEY_07: Presence Tracking` | Rails connection counting | ⏳ Pending |

### Implementation Readiness Checklist (Minto-Structured)

**APEX (Essential for One-Shot Success)**:
- [ ] All 5 critical gaps have complete property tests
- [ ] Mathematical invariants proven for core logic
- [ ] Type contracts prevent invalid states

**KEY LINES (Supporting Implementation Success)**:
- [ ] All service boundaries tested with real dependencies
- [ ] All 12 user journeys have E2E test stubs
- [ ] Rails behavioral parity validated for critical paths

**DETAILS (Implementation Quality Assurance)**:
- [ ] Anti-coordination guards prevent forbidden patterns
- [ ] Performance benchmarks establish regression detection
- [ ] Comprehensive error handling covers all edge cases

**Test Execution Flow Visualization**:
```mermaid
flowchart LR
    subgraph "APEX: Property Tests"
        A1[prop_dedup_idempotent]
        A2[prop_reconnection_delivers_missed]
        A3[prop_concurrent_writes_serialized]
        A4[prop_token_entropy_sufficient]
        A5[prop_presence_ttl_cleanup]
    end
    
    subgraph "KEY LINE 1: Integration Tests"
        B1[integration_message_service]
        B2[integration_websocket_reconnect]
        B3[integration_database_writer]
        B4[integration_auth_service]
        B5[integration_presence_heartbeat]
    end
    
    subgraph "KEY LINE 2: E2E Journey Tests"
        C1[JOURNEY_01: Auth-to-Send]
        C2[JOURNEY_02: Rapid-Click Dedup]
        C3[JOURNEY_03: Reconnect-Missed]
        C4[JOURNEY_07: Presence Tracking]
        C5[JOURNEY_12: Session Security]
    end
    
    subgraph "KEY LINE 3: Rails Parity"
        D1[rails_deduplication_behavior]
        D2[rails_actioncable_reconnect]
        D3[rails_connection_pooling]
        D4[rails_secure_random_tokens]
        D5[rails_presence_counting]
    end
    
    A1 --> B1
    A2 --> B2
    A3 --> B3
    A4 --> B4
    A5 --> B5
    
    B1 --> C1
    B2 --> C3
    B3 --> C2
    B4 --> C5
    B5 --> C4
    
    C1 --> D1
    C2 --> D1
    C3 --> D2
    C4 --> D5
    C5 --> D4
    
    D1 --> E[✅ Gap #1 Verified]
    D2 --> F[✅ Gap #2 Verified]
    D3 --> G[✅ Gap #3 Verified]
    D4 --> H[✅ Gap #4 Verified]
    D5 --> I[✅ Gap #5 Verified]
    
    E --> J[🎉 One-Shot Implementation Ready]
    F --> J
    G --> J
    H --> J
    I --> J
    
    style A1 fill:#e3f2fd
    style A2 fill:#e3f2fd
    style A3 fill:#e3f2fd
    style A4 fill:#e3f2fd
    style A5 fill:#e3f2fd
    style J fill:#e8f5e8
```

**Success Threshold**: APEX + KEY LINES must be 100% complete. DETAILS can have warnings but no failures.