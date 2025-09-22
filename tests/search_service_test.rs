use campfire_on_rust::{
    CampfireDatabase, SearchService, SearchServiceTrait, RoomService,
};
use campfire_on_rust::models::*;
use campfire_on_rust::services::search::{SearchRequest, SearchError};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_db() -> Arc<CampfireDatabase> {
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    Arc::new(db)
}

async fn create_test_user(db: &CampfireDatabase, name: &str, email: &str) -> User {
    let user = User {
        id: UserId::new(),
        name: name.to_string(),
        email: email.to_string(),
        password_hash: "test_hash".to_string(),
        bio: None,
        admin: false,
        bot_token: None,
        created_at: Utc::now(),
    };
    
    db.writer().create_user(user.clone()).await.unwrap();
    user
}

async fn create_test_room(db: &CampfireDatabase, name: &str, room_type: RoomType) -> Room {
    let room = Room {
        id: RoomId::new(),
        name: name.to_string(),
        topic: None,
        room_type,
        created_at: Utc::now(),
        last_message_at: None,
    };
    
    db.writer().create_room(room.clone()).await.unwrap();
    room
}

async fn create_test_membership(db: &CampfireDatabase, room_id: RoomId, user_id: UserId, level: InvolvementLevel) {
    let membership = Membership {
        room_id,
        user_id,
        involvement_level: level,
        created_at: Utc::now(),
    };
    
    db.writer().create_membership(membership).await.unwrap();
}

async fn create_test_message(db: &CampfireDatabase, room_id: RoomId, user_id: UserId, content: &str) -> Message {
    let message = Message {
        id: MessageId::new(),
        room_id,
        creator_id: user_id,
        content: content.to_string(),
        client_message_id: Uuid::new_v4(),
        created_at: Utc::now(),
    };
    
    db.writer().create_message_with_deduplication(message.clone()).await.unwrap()
}

#[tokio::test]
async fn test_search_messages_success() {
    let db = setup_test_db().await;
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = SearchService::new(db.clone(), room_service);
    
    // Create test user
    let user = create_test_user(&db, "Test User", "test@example.com").await;
    
    // Create test room
    let room = create_test_room(&db, "Test Room", RoomType::Open).await;
    
    // Add user to room
    create_test_membership(&db, room.id, user.id, InvolvementLevel::Member).await;
    
    // Create test messages
    create_test_message(&db, room.id, user.id, "Hello world").await;
    create_test_message(&db, room.id, user.id, "This is a test message").await;
    create_test_message(&db, room.id, user.id, "Another message here").await;
    
    // Search for messages
    let request = SearchRequest {
        query: "test".to_string(),
        limit: Some(10),
        offset: Some(0),
        room_id: None,
    };
    
    let response = search_service.search_messages(user.id, request).await.unwrap();
    
    assert_eq!(response.results.len(), 1);
    assert!(response.results[0].message.content.contains("test"));
    assert_eq!(response.total_count, 1);
    assert_eq!(response.query, "test");
    assert!(!response.has_more);
}

#[tokio::test]
async fn test_search_messages_authorization() {
    let db = setup_test_db().await;
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = SearchService::new(db.clone(), room_service);
    
    // Create two users
    let user1 = create_test_user(&db, "User 1", "user1@example.com").await;
    let user2 = create_test_user(&db, "User 2", "user2@example.com").await;
    
    // Create test room
    let room = create_test_room(&db, "Private Room", RoomType::Closed).await;
    
    // Add only user1 to room
    create_test_membership(&db, room.id, user1.id, InvolvementLevel::Member).await;
    
    // Create test message
    create_test_message(&db, room.id, user1.id, "Secret message").await;
    
    // User1 should find the message
    let request = SearchRequest {
        query: "secret".to_string(),
        limit: Some(10),
        offset: Some(0),
        room_id: None,
    };
    
    let response1 = search_service.search_messages(user1.id, request.clone()).await.unwrap();
    assert_eq!(response1.results.len(), 1);
    
    // User2 should not find the message (no access to room)
    let response2 = search_service.search_messages(user2.id, request).await.unwrap();
    assert_eq!(response2.results.len(), 0);
}

#[tokio::test]
async fn test_search_messages_room_specific() {
    let db = setup_test_db().await;
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = SearchService::new(db.clone(), room_service);
    
    // Create test user
    let user = create_test_user(&db, "Test User", "test@example.com").await;
    
    // Create two test rooms
    let room1 = create_test_room(&db, "Room 1", RoomType::Open).await;
    let room2 = create_test_room(&db, "Room 2", RoomType::Open).await;
    
    // Add user to both rooms
    create_test_membership(&db, room1.id, user.id, InvolvementLevel::Member).await;
    create_test_membership(&db, room2.id, user.id, InvolvementLevel::Member).await;
    
    // Create messages in both rooms
    create_test_message(&db, room1.id, user.id, "Hello from room 1").await;
    create_test_message(&db, room2.id, user.id, "Hello from room 2").await;
    
    // Search in specific room
    let request = SearchRequest {
        query: "hello".to_string(),
        limit: Some(10),
        offset: Some(0),
        room_id: Some(room1.id),
    };
    
    let response = search_service.search_messages(user.id, request).await.unwrap();
    
    assert_eq!(response.results.len(), 1);
    assert!(response.results[0].message.content.contains("room 1"));
    assert_eq!(response.results[0].message.room_id, room1.id);
}

#[tokio::test]
async fn test_search_messages_pagination() {
    let db = setup_test_db().await;
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = SearchService::new(db.clone(), room_service);
    
    // Create test user
    let user = create_test_user(&db, "Test User", "test@example.com").await;
    
    // Create test room
    let room = create_test_room(&db, "Test Room", RoomType::Open).await;
    
    // Add user to room
    create_test_membership(&db, room.id, user.id, InvolvementLevel::Member).await;
    
    // Create multiple test messages
    for i in 1..=5 {
        create_test_message(&db, room.id, user.id, &format!("Test message {}", i)).await;
    }
    
    // Search with pagination
    let request = SearchRequest {
        query: "test".to_string(),
        limit: Some(2),
        offset: Some(0),
        room_id: None,
    };
    
    let response = search_service.search_messages(user.id, request).await.unwrap();
    
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.total_count, 5);
    assert_eq!(response.limit, 2);
    assert_eq!(response.offset, 0);
    assert!(response.has_more);
    
    // Test second page
    let request2 = SearchRequest {
        query: "test".to_string(),
        limit: Some(2),
        offset: Some(2),
        room_id: None,
    };
    
    let response2 = search_service.search_messages(user.id, request2).await.unwrap();
    
    assert_eq!(response2.results.len(), 2);
    assert_eq!(response2.total_count, 5);
    assert_eq!(response2.limit, 2);
    assert_eq!(response2.offset, 2);
    assert!(response2.has_more);
}

#[tokio::test]
async fn test_search_messages_query_validation() {
    let db = setup_test_db().await;
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = SearchService::new(db.clone(), room_service);
    
    let user = create_test_user(&db, "Test User", "test@example.com").await;
    
    // Test empty query
    let request = SearchRequest {
        query: "".to_string(),
        limit: Some(10),
        offset: Some(0),
        room_id: None,
    };
    
    let result = search_service.search_messages(user.id, request).await;
    assert!(matches!(result, Err(SearchError::InvalidQuery { .. })));
    
    // Test query too short
    let request = SearchRequest {
        query: "a".to_string(),
        limit: Some(10),
        offset: Some(0),
        room_id: None,
    };
    
    let result = search_service.search_messages(user.id, request).await;
    assert!(matches!(result, Err(SearchError::QueryTooShort)));
    
    // Test query too long
    let request = SearchRequest {
        query: "a".repeat(101),
        limit: Some(10),
        offset: Some(0),
        room_id: None,
    };
    
    let result = search_service.search_messages(user.id, request).await;
    assert!(matches!(result, Err(SearchError::QueryTooLong)));
}

#[tokio::test]
async fn test_search_messages_ranking() {
    let db = setup_test_db().await;
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = SearchService::new(db.clone(), room_service);
    
    // Create test user
    let user = create_test_user(&db, "Test User", "test@example.com").await;
    
    // Create test room
    let room = create_test_room(&db, "Test Room", RoomType::Open).await;
    
    // Add user to room
    create_test_membership(&db, room.id, user.id, InvolvementLevel::Member).await;
    
    // Create messages with different relevance
    create_test_message(&db, room.id, user.id, "rust programming language").await;
    create_test_message(&db, room.id, user.id, "rust rust rust programming").await; // More matches
    create_test_message(&db, room.id, user.id, "python programming").await;
    
    // Search for "rust"
    let request = SearchRequest {
        query: "rust".to_string(),
        limit: Some(10),
        offset: Some(0),
        room_id: None,
    };
    
    let response = search_service.search_messages(user.id, request).await.unwrap();
    
    assert_eq!(response.results.len(), 2);
    
    // Results should be ranked by relevance (more "rust" mentions = higher rank)
    assert!(response.results[0].rank >= response.results[1].rank);
    assert!(response.results[0].message.content.contains("rust rust rust"));
}

#[tokio::test]
async fn test_search_messages_snippet_generation() {
    let db = setup_test_db().await;
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = SearchService::new(db.clone(), room_service);
    
    // Create test user
    let user = create_test_user(&db, "Test User", "test@example.com").await;
    
    // Create test room
    let room = create_test_room(&db, "Test Room", RoomType::Open).await;
    
    // Add user to room
    create_test_membership(&db, room.id, user.id, InvolvementLevel::Member).await;
    
    // Create a long message
    let long_content = "This is a very long message that contains the word programming in the middle of a lot of other text that should be truncated in the snippet to show only the relevant part around the search term.";
    create_test_message(&db, room.id, user.id, long_content).await;
    
    // Search for "programming"
    let request = SearchRequest {
        query: "programming".to_string(),
        limit: Some(10),
        offset: Some(0),
        room_id: None,
    };
    
    let response = search_service.search_messages(user.id, request).await.unwrap();
    
    assert_eq!(response.results.len(), 1);
    
    let snippet = &response.results[0].snippet;
    assert!(snippet.contains("programming"));
    assert!(snippet.len() < long_content.len()); // Should be truncated
    assert!(snippet.contains("...")); // Should have ellipsis
}

#[tokio::test]
async fn test_search_room_messages() {
    let db = setup_test_db().await;
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = SearchService::new(db.clone(), room_service);
    
    // Create test user
    let user = create_test_user(&db, "Test User", "test@example.com").await;
    
    // Create test room
    let room = create_test_room(&db, "Test Room", RoomType::Open).await;
    
    // Add user to room
    create_test_membership(&db, room.id, user.id, InvolvementLevel::Member).await;
    
    // Create test message
    create_test_message(&db, room.id, user.id, "Hello world").await;
    
    // Search room messages
    let response = search_service.search_room_messages(
        user.id,
        room.id,
        "hello".to_string(),
        10,
        0,
    ).await.unwrap();
    
    assert_eq!(response.results.len(), 1);
    assert!(response.results[0].message.content.contains("Hello"));
    assert_eq!(response.results[0].message.room_id, room.id);
}

#[tokio::test]
async fn test_search_no_results() {
    let db = setup_test_db().await;
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = SearchService::new(db.clone(), room_service);
    
    // Create test user
    let user = create_test_user(&db, "Test User", "test@example.com").await;
    
    // Create test room
    let room = create_test_room(&db, "Test Room", RoomType::Open).await;
    
    // Add user to room
    create_test_membership(&db, room.id, user.id, InvolvementLevel::Member).await;
    
    // Create test message
    create_test_message(&db, room.id, user.id, "Hello world").await;
    
    // Search for non-existent term
    let request = SearchRequest {
        query: "nonexistent".to_string(),
        limit: Some(10),
        offset: Some(0),
        room_id: None,
    };
    
    let response = search_service.search_messages(user.id, request).await.unwrap();
    
    assert_eq!(response.results.len(), 0);
    assert_eq!(response.total_count, 0);
    assert!(!response.has_more);
}