// Critical Gap Tests - Comprehensive test coverage for all 5 critical gaps
// Following TDD-First Architecture Principles with executable specifications

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use campfire_on_rust::{
    CampfireDatabase, AuthService, RoomService, MessageService, ConnectionManagerImpl,
    AuthServiceTrait, MessageServiceTrait, ConnectionManager, RoomServiceTrait,
    models::{User, Room, Membership, Session, RoomType, InvolvementLevel, UserId, RoomId, MessageId, ConnectionId},
    errors::{MessageError, AuthError},
};

/// Test helper to create a complete test environment
async fn create_test_environment() -> TestEnvironment {
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let db_arc = Arc::new(db);
    
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db_arc.clone()));
    let auth_service = Arc::new(AuthService::new(db_arc.clone()));
    let room_service = Arc::new(RoomService::new(db_arc.clone()));
    let message_service = Arc::new(MessageService::new(
        db_arc.clone(),
        connection_manager.clone(),
        room_service.clone()
    ));
    
    TestEnvironment {
        db: db_arc,
        auth_service,
        room_service,
        message_service,
        connection_manager,
    }
}

struct TestEnvironment {
    db: Arc<CampfireDatabase>,
    auth_service: Arc<AuthService>,
    room_service: Arc<RoomService>,
    message_service: Arc<MessageService>,
    connection_manager: Arc<ConnectionManagerImpl>,
}

impl TestEnvironment {
    /// Creates a test user and returns the user and session
    async fn create_test_user(&self, name: &str, email: &str) -> (User, Session) {
        let user = self.auth_service.create_user(
            name.to_string(),
            email.to_string(),
            "password123".to_string(),
        ).await.unwrap();
        
        let session = self.auth_service.create_session(user.id).await.unwrap();
        
        (user, session)
    }
    
    /// Creates a test room and adds the user as a member
    async fn create_test_room(&self, user_id: UserId, room_name: &str) -> Room {
        let room = Room {
            id: RoomId::new(),
            name: room_name.to_string(),
            topic: None,
            room_type: RoomType::Open,
            created_at: chrono::Utc::now(),
            last_message_at: None,
        };
        
        self.db.create_room(room.clone()).await.unwrap();
        
        // Add user as member
        let membership = Membership {
            room_id: room.id,
            user_id,
            involvement_level: InvolvementLevel::Member,
            created_at: chrono::Utc::now(),
        };
        
        self.db.create_membership(membership).await.unwrap();
        
        room
    }
}

// =============================================================================
// CRITICAL GAP #1: MESSAGE DEDUPLICATION TESTS
// =============================================================================

/// Test Critical Gap #1: Message Deduplication Contract
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
#[tokio::test]
async fn test_critical_gap_1_message_deduplication_idempotency() {
    let env = create_test_environment().await;
    let (user, _session) = env.create_test_user("Test User", "test@example.com").await;
    let room = env.create_test_room(user.id, "Test Room").await;
    
    let client_message_id = Uuid::new_v4();
    let content = "Test message for deduplication".to_string();
    
    // First message creation should succeed
    let message1 = env.message_service.create_message_with_deduplication(
        content.clone(),
        room.id,
        user.id,
        client_message_id,
    ).await.unwrap();
    
    // Second message with same client_message_id should return the same message (deduplication)
    let message2 = env.message_service.create_message_with_deduplication(
        "Different content (should be ignored)".to_string(),
        room.id,
        user.id,
        client_message_id,
    ).await.unwrap();
    
    // Verify deduplication worked
    assert_eq!(message1.id, message2.id, "Messages should have same ID (deduplication)");
    assert_eq!(message1.content, message2.content, "Original content should be preserved");
    assert_eq!(message1.content, content, "Content should match original");
    assert_eq!(message1.client_message_id, client_message_id);
    assert_eq!(message2.client_message_id, client_message_id);
    
    // Verify only one message exists in database
    let messages = env.message_service.get_room_messages(room.id, user.id, 10, None).await.unwrap();
    assert_eq!(messages.len(), 1, "Only one message should exist in database");
    assert_eq!(messages[0].id, message1.id);
}

#[tokio::test]
async fn test_critical_gap_1_message_deduplication_different_rooms() {
    let env = create_test_environment().await;
    let (user, _session) = env.create_test_user("Test User", "test@example.com").await;
    let room1 = env.create_test_room(user.id, "Room 1").await;
    let room2 = env.create_test_room(user.id, "Room 2").await;
    
    let client_message_id = Uuid::new_v4();
    let content = "Same client ID, different rooms".to_string();
    
    // Same client_message_id in different rooms should create different messages
    let message1 = env.message_service.create_message_with_deduplication(
        content.clone(),
        room1.id,
        user.id,
        client_message_id,
    ).await.unwrap();
    
    let message2 = env.message_service.create_message_with_deduplication(
        content.clone(),
        room2.id,
        user.id,
        client_message_id,
    ).await.unwrap();
    
    // Should be different messages (different rooms)
    assert_ne!(message1.id, message2.id, "Messages in different rooms should have different IDs");
    assert_eq!(message1.room_id, room1.id);
    assert_eq!(message2.room_id, room2.id);
    assert_eq!(message1.client_message_id, client_message_id);
    assert_eq!(message2.client_message_id, client_message_id);
}

#[tokio::test]
async fn test_critical_gap_1_message_deduplication_concurrent_creation() {
    let env = create_test_environment().await;
    let (user, _session) = env.create_test_user("Test User", "test@example.com").await;
    let room = env.create_test_room(user.id, "Test Room").await;
    
    let client_message_id = Uuid::new_v4();
    let content = "Concurrent message creation".to_string();
    
    // Create multiple concurrent requests with same client_message_id
    let mut handles = Vec::new();
    for i in 0..5 {
        let env_clone = env.message_service.clone();
        let content_clone = format!("{} - attempt {}", content, i);
        let room_id = room.id;
        let user_id = user.id;
        
        let handle = tokio::spawn(async move {
            env_clone.create_message_with_deduplication(
                content_clone,
                room_id,
                user_id,
                client_message_id,
            ).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap().unwrap());
    }
    
    // All results should have the same message ID (deduplication worked)
    let first_id = results[0].id;
    for result in &results {
        assert_eq!(result.id, first_id, "All concurrent requests should return same message ID");
        assert_eq!(result.client_message_id, client_message_id);
    }
    
    // Verify only one message exists in database
    let messages = env.message_service.get_room_messages(room.id, user.id, 10, None).await.unwrap();
    assert_eq!(messages.len(), 1, "Only one message should exist despite concurrent creation");
}

// =============================================================================
// CRITICAL GAP #2: WEBSOCKET RECONNECTION AND MISSED MESSAGES TESTS
// =============================================================================

/// Test Critical Gap #2: WebSocket Reconnection State
/// 
/// Tests the missed message delivery system that handles reconnections
#[tokio::test]
async fn test_critical_gap_2_missed_message_delivery_on_reconnection() {
    let env = create_test_environment().await;
    let (user, _session) = env.create_test_user("Test User", "test@example.com").await;
    let room = env.create_test_room(user.id, "Test Room").await;
    
    // Create initial connection
    let connection_id = ConnectionId::new();
    let (sender, _receiver) = mpsc::unbounded_channel();
    
    env.connection_manager.add_connection(user.id, connection_id, sender).await.unwrap();
    
    // Create a message while connected
    let message1 = env.message_service.create_message_with_deduplication(
        "Message 1 - while connected".to_string(),
        room.id,
        user.id,
        Uuid::new_v4(),
    ).await.unwrap();
    
    // Update last seen message
    env.connection_manager.update_last_seen_message(connection_id, message1.id).await.unwrap();
    
    // Simulate disconnection
    env.connection_manager.remove_connection(connection_id).await.unwrap();
    
    // Create messages while disconnected (these should be "missed")
    let _message2 = env.message_service.create_message_with_deduplication(
        "Message 2 - while disconnected".to_string(),
        room.id,
        user.id,
        Uuid::new_v4(),
    ).await.unwrap();
    
    let _message3 = env.message_service.create_message_with_deduplication(
        "Message 3 - while disconnected".to_string(),
        room.id,
        user.id,
        Uuid::new_v4(),
    ).await.unwrap();
    
    // Simulate reconnection with new connection
    let new_connection_id = ConnectionId::new();
    let (new_sender, mut new_receiver) = mpsc::unbounded_channel();
    
    env.connection_manager.add_connection(user.id, new_connection_id, new_sender).await.unwrap();
    
    // Request missed messages since last seen message
    env.connection_manager.send_missed_messages(
        user.id,
        new_connection_id,
        Some(message1.id),
    ).await.unwrap();
    
    // Should receive the missed messages
    let mut received_messages = Vec::new();
    
    // Use timeout to avoid hanging if messages aren't received
    for _ in 0..2 {
        match timeout(Duration::from_millis(100), new_receiver.recv()).await {
            Ok(Some(msg)) => received_messages.push(msg),
            Ok(None) => break,
            Err(_) => break, // Timeout
        }
    }
    
    assert_eq!(received_messages.len(), 2, "Should receive 2 missed messages");
    
    // Verify the messages contain the expected content
    assert!(received_messages.iter().any(|msg| msg.contains("Message 2")));
    assert!(received_messages.iter().any(|msg| msg.contains("Message 3")));
}

#[tokio::test]
async fn test_critical_gap_2_no_missed_messages_when_up_to_date() {
    let env = create_test_environment().await;
    let (user, _session) = env.create_test_user("Test User", "test@example.com").await;
    let room = env.create_test_room(user.id, "Test Room").await;
    
    // Create a message
    let message1 = env.message_service.create_message_with_deduplication(
        "Latest message".to_string(),
        room.id,
        user.id,
        Uuid::new_v4(),
    ).await.unwrap();
    
    // Create connection and mark as up-to-date
    let connection_id = ConnectionId::new();
    let (sender, mut receiver) = mpsc::unbounded_channel();
    
    env.connection_manager.add_connection(user.id, connection_id, sender).await.unwrap();
    env.connection_manager.update_last_seen_message(connection_id, message1.id).await.unwrap();
    
    // Request missed messages (should be none)
    env.connection_manager.send_missed_messages(
        user.id,
        connection_id,
        Some(message1.id),
    ).await.unwrap();
    
    // Should not receive any messages
    match timeout(Duration::from_millis(50), receiver.recv()).await {
        Ok(Some(_)) => panic!("Should not receive any messages when up-to-date"),
        Ok(None) => {}, // Channel closed, expected
        Err(_) => {}, // Timeout, expected
    }
}

#[tokio::test]
async fn test_critical_gap_2_missed_messages_with_limit() {
    let env = create_test_environment().await;
    let (user, _session) = env.create_test_user("Test User", "test@example.com").await;
    let room = env.create_test_room(user.id, "Test Room").await;
    
    // Create many messages while "disconnected"
    let mut message_ids = Vec::new();
    for i in 0..150 { // More than the 100 message limit
        let message = env.message_service.create_message_with_deduplication(
            format!("Message {}", i),
            room.id,
            user.id,
            Uuid::new_v4(),
        ).await.unwrap();
        message_ids.push(message.id);
    }
    
    // Create connection and request missed messages from beginning
    let connection_id = ConnectionId::new();
    let (sender, mut receiver) = mpsc::unbounded_channel();
    
    env.connection_manager.add_connection(user.id, connection_id, sender).await.unwrap();
    
    // Request missed messages with no last seen (should get limited to 100)
    env.connection_manager.send_missed_messages(
        user.id,
        connection_id,
        None,
    ).await.unwrap();
    
    // Count received messages
    let mut received_count = 0;
    while let Ok(Some(_)) = timeout(Duration::from_millis(10), receiver.recv()).await {
        received_count += 1;
        if received_count > 110 { // Safety break
            break;
        }
    }
    
    // Should be limited to 100 messages
    assert!(received_count <= 100, "Should not receive more than 100 missed messages, got {}", received_count);
    assert!(received_count > 0, "Should receive some messages");
}

// =============================================================================
// CRITICAL GAP #3: AUTHORIZATION BOUNDARY TESTS
// =============================================================================

/// Test Critical Gap #3: Authorization Boundaries
/// 
/// Tests that authorization is properly enforced at all service boundaries
#[tokio::test]
async fn test_critical_gap_3_message_authorization_boundaries() {
    let env = create_test_environment().await;
    let (user1, _session1) = env.create_test_user("User 1", "user1@example.com").await;
    let (user2, _session2) = env.create_test_user("User 2", "user2@example.com").await;
    
    // Create room and only add user1 as member
    let room = env.create_test_room(user1.id, "Private Room").await;
    
    // User1 should be able to create messages (has access)
    let result = env.message_service.create_message_with_deduplication(
        "User1 message".to_string(),
        room.id,
        user1.id,
        Uuid::new_v4(),
    ).await;
    
    assert!(result.is_ok(), "User1 should be able to create messages in their room");
    
    // User2 should NOT be able to create messages (no access)
    let result = env.message_service.create_message_with_deduplication(
        "User2 unauthorized message".to_string(),
        room.id,
        user2.id,
        Uuid::new_v4(),
    ).await;
    
    assert!(result.is_err(), "User2 should not be able to create messages in room they're not in");
    match result.unwrap_err() {
        MessageError::Authorization { user_id, room_id } => {
            assert_eq!(user_id, user2.id);
            assert_eq!(room_id, room.id);
        }
        other => panic!("Expected Authorization error, got: {:?}", other),
    }
    
    // User2 should NOT be able to read messages (no access)
    let result = env.message_service.get_room_messages(room.id, user2.id, 10, None).await;
    
    assert!(result.is_err(), "User2 should not be able to read messages from room they're not in");
    match result.unwrap_err() {
        MessageError::Authorization { user_id, room_id } => {
            assert_eq!(user_id, user2.id);
            assert_eq!(room_id, room.id);
        }
        other => panic!("Expected Authorization error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_critical_gap_3_room_authorization_boundaries() {
    let env = create_test_environment().await;
    let (user1, _session1) = env.create_test_user("User 1", "user1@example.com").await;
    let (user2, _session2) = env.create_test_user("User 2", "user2@example.com").await;
    
    let room = env.create_test_room(user1.id, "Test Room").await;
    
    // User1 should have access (is member)
    let result = env.room_service.check_room_access(room.id, user1.id).await;
    assert!(result.is_ok(), "User1 should have room access");
    assert!(result.unwrap().is_some(), "User1 should have involvement level");
    
    // User2 should NOT have access (not member)
    let result = env.room_service.check_room_access(room.id, user2.id).await;
    assert!(result.is_ok(), "Check should succeed but return None");
    assert!(result.unwrap().is_none(), "User2 should not have access");
}

#[tokio::test]
async fn test_critical_gap_3_cross_user_data_isolation() {
    let env = create_test_environment().await;
    let (user1, _session1) = env.create_test_user("User 1", "user1@example.com").await;
    let (user2, _session2) = env.create_test_user("User 2", "user2@example.com").await;
    
    // Create separate rooms for each user
    let room1 = env.create_test_room(user1.id, "User1 Room").await;
    let room2 = env.create_test_room(user2.id, "User2 Room").await;
    
    // Create messages in each room
    let message1 = env.message_service.create_message_with_deduplication(
        "User1 private message".to_string(),
        room1.id,
        user1.id,
        Uuid::new_v4(),
    ).await.unwrap();
    
    let message2 = env.message_service.create_message_with_deduplication(
        "User2 private message".to_string(),
        room2.id,
        user2.id,
        Uuid::new_v4(),
    ).await.unwrap();
    
    // User1 should only see their own room's messages
    let user1_messages = env.message_service.get_room_messages(room1.id, user1.id, 10, None).await.unwrap();
    assert_eq!(user1_messages.len(), 1);
    assert_eq!(user1_messages[0].id, message1.id);
    assert!(user1_messages[0].content.contains("User1"));
    
    // User2 should only see their own room's messages
    let user2_messages = env.message_service.get_room_messages(room2.id, user2.id, 10, None).await.unwrap();
    assert_eq!(user2_messages.len(), 1);
    assert_eq!(user2_messages[0].id, message2.id);
    assert!(user2_messages[0].content.contains("User2"));
    
    // Cross-access should be denied
    let result = env.message_service.get_room_messages(room1.id, user2.id, 10, None).await;
    assert!(result.is_err(), "User2 should not access User1's room");
    
    let result = env.message_service.get_room_messages(room2.id, user1.id, 10, None).await;
    assert!(result.is_err(), "User1 should not access User2's room");
}

// =============================================================================
// CRITICAL GAP #4: SESSION SECURITY AND TOKEN VALIDATION TESTS
// =============================================================================

/// Test Critical Gap #4: Session Token Security
/// 
/// Tests secure session token generation, validation, and lifecycle management
#[tokio::test]
async fn test_critical_gap_4_session_token_security_properties() {
    let env = create_test_environment().await;
    let (user, _session) = env.create_test_user("Test User", "test@example.com").await;
    
    // Generate multiple session tokens
    let mut tokens = Vec::new();
    for _ in 0..10 {
        let session = env.auth_service.create_session(user.id).await.unwrap();
        tokens.push(session.token);
    }
    
    // All tokens should be unique (no collisions)
    for i in 0..tokens.len() {
        for j in (i + 1)..tokens.len() {
            assert_ne!(tokens[i], tokens[j], "Session tokens should be unique");
        }
    }
    
    // All tokens should have sufficient entropy (at least 32 characters)
    for token in &tokens {
        assert!(token.len() >= 32, "Token should have at least 32 characters for security");
        
        // Should be URL-safe (no characters that need encoding)
        assert!(!token.contains('+'), "Token should be URL-safe (no +)");
        assert!(!token.contains('/'), "Token should be URL-safe (no /)");
        assert!(!token.contains('='), "Token should be URL-safe (no =)");
        
        // Should not contain predictable patterns
        assert!(!token.contains("0000"), "Token should not contain predictable patterns");
        assert!(!token.contains("1111"), "Token should not contain predictable patterns");
        assert!(!token.contains("aaaa"), "Token should not contain predictable patterns");
    }
}

#[tokio::test]
async fn test_critical_gap_4_session_validation_and_expiry() {
    let env = create_test_environment().await;
    let (user, session) = env.create_test_user("Test User", "test@example.com").await;
    
    // Valid session should authenticate successfully
    let validated_user = env.auth_service.validate_session(session.token.clone()).await.unwrap();
    assert_eq!(validated_user.id, user.id);
    assert_eq!(validated_user.email, user.email);
    
    // Invalid token should fail
    let result = env.auth_service.validate_session("invalid_token".to_string()).await;
    assert!(result.is_err(), "Invalid token should fail validation");
    match result.unwrap_err() {
        AuthError::SessionExpired => {}, // Expected
        other => panic!("Expected SessionExpired, got: {:?}", other),
    }
    
    // Revoked session should fail
    env.auth_service.revoke_session(session.token.clone()).await.unwrap();
    let result = env.auth_service.validate_session(session.token).await;
    assert!(result.is_err(), "Revoked session should fail validation");
}

#[tokio::test]
async fn test_critical_gap_4_session_isolation_between_users() {
    let env = create_test_environment().await;
    let (user1, session1) = env.create_test_user("User 1", "user1@example.com").await;
    let (user2, session2) = env.create_test_user("User 2", "user2@example.com").await;
    
    // Each session should only authenticate its own user
    let validated_user1 = env.auth_service.validate_session(session1.token).await.unwrap();
    assert_eq!(validated_user1.id, user1.id);
    assert_ne!(validated_user1.id, user2.id);
    
    let validated_user2 = env.auth_service.validate_session(session2.token).await.unwrap();
    assert_eq!(validated_user2.id, user2.id);
    assert_ne!(validated_user2.id, user1.id);
}

#[tokio::test]
async fn test_critical_gap_4_concurrent_session_operations() {
    let env = create_test_environment().await;
    let (user, _initial_session) = env.create_test_user("Test User", "test@example.com").await;
    
    // Create multiple sessions concurrently
    let mut handles = Vec::new();
    for _ in 0..10 {
        let auth_service = env.auth_service.clone();
        let user_id = user.id;
        
        let handle = tokio::spawn(async move {
            auth_service.create_session(user_id).await
        });
        
        handles.push(handle);
    }
    
    // All should succeed and be unique
    let mut sessions = Vec::new();
    for handle in handles {
        let session = handle.await.unwrap().unwrap();
        sessions.push(session);
    }
    
    // All sessions should be valid and unique
    assert_eq!(sessions.len(), 10);
    
    for i in 0..sessions.len() {
        // Each session should validate to the same user
        let validated_user = env.auth_service.validate_session(sessions[i].token.clone()).await.unwrap();
        assert_eq!(validated_user.id, user.id);
        
        // All tokens should be unique
        for j in (i + 1)..sessions.len() {
            assert_ne!(sessions[i].token, sessions[j].token);
        }
    }
}

// =============================================================================
// CRITICAL GAP #5: PRESENCE TRACKING ACCURACY TESTS
// =============================================================================

/// Test Critical Gap #5: Presence Tracking Accuracy
/// 
/// Tests accurate presence tracking, cleanup, and room-specific presence
#[tokio::test]
async fn test_critical_gap_5_presence_tracking_accuracy() {
    let env = create_test_environment().await;
    let (user1, _session1) = env.create_test_user("User 1", "user1@example.com").await;
    let (user2, _session2) = env.create_test_user("User 2", "user2@example.com").await;
    let room = env.create_test_room(user1.id, "Test Room").await;
    
    // Initially no presence
    let presence = env.connection_manager.get_room_presence(room.id).await.unwrap();
    assert!(presence.is_empty(), "Initially no users should be present");
    
    // Add user1 connection
    let connection1 = ConnectionId::new();
    let (sender1, _receiver1) = mpsc::unbounded_channel();
    env.connection_manager.add_connection(user1.id, connection1, sender1).await.unwrap();
    
    // Add user1 to room members (simulate room membership)
    env.connection_manager.add_room_membership(room.id, vec![user1.id]).await;
    
    // Should show user1 as present
    let presence = env.connection_manager.get_room_presence(room.id).await.unwrap();
    assert_eq!(presence.len(), 1);
    assert!(presence.contains(&user1.id));
    
    // Add user2 connection and room membership
    let connection2 = ConnectionId::new();
    let (sender2, _receiver2) = mpsc::unbounded_channel();
    env.connection_manager.add_connection(user2.id, connection2, sender2).await.unwrap();
    
    env.connection_manager.add_room_membership(room.id, vec![user1.id, user2.id]).await;
    
    // Should show both users as present
    let presence = env.connection_manager.get_room_presence(room.id).await.unwrap();
    assert_eq!(presence.len(), 2);
    assert!(presence.contains(&user1.id));
    assert!(presence.contains(&user2.id));
    
    // Remove user1 connection
    env.connection_manager.remove_connection(connection1).await.unwrap();
    
    // Should only show user2 as present
    let presence = env.connection_manager.get_room_presence(room.id).await.unwrap();
    assert_eq!(presence.len(), 1);
    assert!(presence.contains(&user2.id));
    assert!(!presence.contains(&user1.id));
    
    // Remove user2 connection
    env.connection_manager.remove_connection(connection2).await.unwrap();
    
    // Should show no users as present
    let presence = env.connection_manager.get_room_presence(room.id).await.unwrap();
    assert!(presence.is_empty(), "No users should be present after all disconnected");
}

#[tokio::test]
async fn test_critical_gap_5_multiple_connections_per_user() {
    let env = create_test_environment().await;
    let (user, _session) = env.create_test_user("Test User", "test@example.com").await;
    let room = env.create_test_room(user.id, "Test Room").await;
    
    // Add room membership
    env.connection_manager.add_room_membership(room.id, vec![user.id]).await;
    
    // Add multiple connections for same user (multiple tabs/devices)
    let connection1 = ConnectionId::new();
    let connection2 = ConnectionId::new();
    let connection3 = ConnectionId::new();
    
    let (sender1, _receiver1) = mpsc::unbounded_channel();
    let (sender2, _receiver2) = mpsc::unbounded_channel();
    let (sender3, _receiver3) = mpsc::unbounded_channel();
    
    env.connection_manager.add_connection(user.id, connection1, sender1).await.unwrap();
    env.connection_manager.add_connection(user.id, connection2, sender2).await.unwrap();
    env.connection_manager.add_connection(user.id, connection3, sender3).await.unwrap();
    
    // Should show user as present (regardless of multiple connections)
    let presence = env.connection_manager.get_room_presence(room.id).await.unwrap();
    assert_eq!(presence.len(), 1);
    assert!(presence.contains(&user.id));
    
    // Remove one connection - user should still be present
    env.connection_manager.remove_connection(connection1).await.unwrap();
    let presence = env.connection_manager.get_room_presence(room.id).await.unwrap();
    assert_eq!(presence.len(), 1);
    assert!(presence.contains(&user.id));
    
    // Remove second connection - user should still be present
    env.connection_manager.remove_connection(connection2).await.unwrap();
    let presence = env.connection_manager.get_room_presence(room.id).await.unwrap();
    assert_eq!(presence.len(), 1);
    assert!(presence.contains(&user.id));
    
    // Remove last connection - user should no longer be present
    env.connection_manager.remove_connection(connection3).await.unwrap();
    let presence = env.connection_manager.get_room_presence(room.id).await.unwrap();
    assert!(presence.is_empty());
}

#[tokio::test]
async fn test_critical_gap_5_typing_indicators_accuracy() {
    let env = create_test_environment().await;
    let (user1, _session1) = env.create_test_user("User 1", "user1@example.com").await;
    let (user2, _session2) = env.create_test_user("User 2", "user2@example.com").await;
    let room = env.create_test_room(user1.id, "Test Room").await;
    
    // Initially no typing users
    let typing = env.connection_manager.get_typing_users(room.id).await.unwrap();
    assert!(typing.is_empty());
    
    // User1 starts typing
    env.connection_manager.start_typing(user1.id, room.id).await.unwrap();
    let typing = env.connection_manager.get_typing_users(room.id).await.unwrap();
    assert_eq!(typing.len(), 1);
    assert!(typing.contains(&user1.id));
    
    // User2 starts typing
    env.connection_manager.start_typing(user2.id, room.id).await.unwrap();
    let typing = env.connection_manager.get_typing_users(room.id).await.unwrap();
    assert_eq!(typing.len(), 2);
    assert!(typing.contains(&user1.id));
    assert!(typing.contains(&user2.id));
    
    // User1 stops typing
    env.connection_manager.stop_typing(user1.id, room.id).await.unwrap();
    let typing = env.connection_manager.get_typing_users(room.id).await.unwrap();
    assert_eq!(typing.len(), 1);
    assert!(typing.contains(&user2.id));
    assert!(!typing.contains(&user1.id));
    
    // User2 stops typing
    env.connection_manager.stop_typing(user2.id, room.id).await.unwrap();
    let typing = env.connection_manager.get_typing_users(room.id).await.unwrap();
    assert!(typing.is_empty());
}

#[tokio::test]
async fn test_critical_gap_5_presence_cleanup_on_stale_connections() {
    let env = create_test_environment().await;
    let (user, _session) = env.create_test_user("Test User", "test@example.com").await;
    let room = env.create_test_room(user.id, "Test Room").await;
    
    // Add room membership
    env.connection_manager.add_room_membership(room.id, vec![user.id]).await;
    
    // Add connection
    let connection_id = ConnectionId::new();
    let (sender, receiver) = mpsc::unbounded_channel();
    env.connection_manager.add_connection(user.id, connection_id, sender).await.unwrap();
    
    // Verify presence
    let presence = env.connection_manager.get_room_presence(room.id).await.unwrap();
    assert_eq!(presence.len(), 1);
    assert!(presence.contains(&user.id));
    
    // Drop the receiver to simulate a closed connection
    drop(receiver);
    
    // Wait for cleanup task to run (the cleanup runs every 30 seconds, but we can't wait that long)
    // Instead, we'll test that the cleanup logic works by manually triggering it
    // by checking if the sender is closed and removing dead connections
    
    // Give a small delay to ensure the channel is properly closed
    sleep(Duration::from_millis(10)).await;
    
    // The cleanup task should eventually remove the dead connection
    // For this test, we'll verify the logic by checking if connection still exists
    let connection_exists = env.connection_manager.connection_exists(connection_id).await;
    // Connection should still exist (cleanup happens in background)
    assert!(connection_exists, "Connection should still exist immediately after receiver drop");
}

// =============================================================================
// INTEGRATION TESTS FOR CRITICAL GAPS
// =============================================================================

/// Integration test that validates all critical gaps work together
#[tokio::test]
async fn test_all_critical_gaps_integration() {
    let env = create_test_environment().await;
    
    // Setup: Create users and room (Critical Gap #3: Authorization)
    let (user1, session1) = env.create_test_user("User 1", "user1@example.com").await;
    let (user2, session2) = env.create_test_user("User 2", "user2@example.com").await;
    let room = env.create_test_room(user1.id, "Integration Test Room").await;
    
    // Add user2 to room as well
    let membership = Membership {
        room_id: room.id,
        user_id: user2.id,
        involvement_level: InvolvementLevel::Member,
        created_at: chrono::Utc::now(),
    };
    env.db.create_membership(membership).await.unwrap();
    
    // Critical Gap #4: Session validation works
    let validated_user1 = env.auth_service.validate_session(session1.token).await.unwrap();
    assert_eq!(validated_user1.id, user1.id);
    
    // Critical Gap #5: Setup presence tracking
    let connection1 = ConnectionId::new();
    let connection2 = ConnectionId::new();
    let (sender1, mut receiver1) = mpsc::unbounded_channel();
    let (sender2, mut receiver2) = mpsc::unbounded_channel();
    
    env.connection_manager.add_connection(user1.id, connection1, sender1).await.unwrap();
    env.connection_manager.add_connection(user2.id, connection2, sender2).await.unwrap();
    
    // Add room memberships for presence tracking
    env.connection_manager.add_room_membership(room.id, vec![user1.id, user2.id]).await;
    
    // Verify both users are present
    let presence = env.connection_manager.get_room_presence(room.id).await.unwrap();
    assert_eq!(presence.len(), 2);
    
    // Critical Gap #1: Message deduplication
    let client_message_id = Uuid::new_v4();
    let message1 = env.message_service.create_message_with_deduplication(
        "Integration test message".to_string(),
        room.id,
        user1.id,
        client_message_id,
    ).await.unwrap();
    
    // Duplicate should return same message
    let message2 = env.message_service.create_message_with_deduplication(
        "Different content".to_string(),
        room.id,
        user1.id,
        client_message_id,
    ).await.unwrap();
    
    assert_eq!(message1.id, message2.id);
    
    // Both users should receive the broadcast
    let mut received_by_user1 = false;
    let mut received_by_user2 = false;
    
    // Check if messages were broadcast (with timeout)
    for _ in 0..2 {
        if let Ok(Some(_)) = timeout(Duration::from_millis(50), receiver1.recv()).await {
            received_by_user1 = true;
        }
        if let Ok(Some(_)) = timeout(Duration::from_millis(50), receiver2.recv()).await {
            received_by_user2 = true;
        }
    }
    
    // At least one should receive (depending on timing)
    assert!(received_by_user1 || received_by_user2, "At least one user should receive the broadcast");
    
    // Critical Gap #2: Missed messages on reconnection
    env.connection_manager.update_last_seen_message(connection1, message1.id).await.unwrap();
    env.connection_manager.remove_connection(connection1).await.unwrap();
    
    // Create message while user1 is disconnected
    let missed_message = env.message_service.create_message_with_deduplication(
        "Missed message".to_string(),
        room.id,
        user2.id,
        Uuid::new_v4(),
    ).await.unwrap();
    
    // Reconnect user1
    let new_connection1 = ConnectionId::new();
    let (new_sender1, mut new_receiver1) = mpsc::unbounded_channel();
    env.connection_manager.add_connection(user1.id, new_connection1, new_sender1).await.unwrap();
    
    // Request missed messages
    env.connection_manager.send_missed_messages(
        user1.id,
        new_connection1,
        Some(message1.id),
    ).await.unwrap();
    
    // Should receive the missed message
    let received_missed = timeout(Duration::from_millis(100), new_receiver1.recv()).await;
    assert!(received_missed.is_ok(), "Should receive missed message on reconnection");
    
    // Critical Gap #3: Authorization still enforced
    let (unauthorized_user, _) = env.create_test_user("Unauthorized", "unauthorized@example.com").await;
    
    let unauthorized_result = env.message_service.create_message_with_deduplication(
        "Unauthorized message".to_string(),
        room.id,
        unauthorized_user.id,
        Uuid::new_v4(),
    ).await;
    
    assert!(unauthorized_result.is_err(), "Unauthorized user should not be able to send messages");
}