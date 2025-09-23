use campfire_on_rust::{CampfireDatabase, SearchService, RoomService};
use campfire_on_rust::models::*;
use campfire_on_rust::services::search::{SearchRequest, SearchServiceTrait};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_basic_search_functionality() {
    // Setup test database
    let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = SearchService::new(db.clone(), room_service);
    
    // Create test user
    let user = User {
        id: UserId::new(),
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "test_hash".to_string(),
        bio: None,
        admin: false,
        bot_token: None,
        created_at: Utc::now(),
    };
    
    db.writer().create_user(user.clone()).await.unwrap();
    
    // Create test room
    let room = Room {
        id: RoomId::new(),
        name: "Test Room".to_string(),
        topic: None,
        room_type: RoomType::Open,
        created_at: Utc::now(),
        last_message_at: None,
    };
    
    db.writer().create_room(room.clone()).await.unwrap();
    
    // Add user to room
    let membership = Membership {
        room_id: room.id,
        user_id: user.id,
        involvement_level: InvolvementLevel::Member,
        created_at: Utc::now(),
    };
    
    db.writer().create_membership(membership).await.unwrap();
    
    // Create test messages
    let message1 = Message {
        id: MessageId::new(),
        room_id: room.id,
        creator_id: user.id,
        content: "Hello world".to_string(),
        client_message_id: Uuid::new_v4(),
        created_at: Utc::now(),
        html_content: None,
        mentions: Vec::new(),
        sound_commands: Vec::new(),
    };
    
    let message2 = Message {
        id: MessageId::new(),
        room_id: room.id,
        creator_id: user.id,
        content: "This is a test message".to_string(),
        client_message_id: Uuid::new_v4(),
        created_at: Utc::now(),
        html_content: None,
        mentions: Vec::new(),
        sound_commands: Vec::new(),
    };
    
    db.writer().create_message_with_deduplication(message1).await.unwrap();
    db.writer().create_message_with_deduplication(message2).await.unwrap();
    
    // Test search
    let request = SearchRequest {
        query: "test".to_string(),
        limit: Some(10),
        offset: Some(0),
        room_id: None,
    };
    
    let response = search_service.search_messages(user.id, request).await.unwrap();
    
    // Verify results
    assert_eq!(response.results.len(), 1);
    assert!(response.results[0].message.content.contains("test"));
    assert_eq!(response.total_count, 1);
    assert_eq!(response.query, "test");
    assert!(!response.has_more);
    
    println!("✅ Basic search functionality test passed!");
}

#[tokio::test]
async fn test_search_validation() {
    let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = SearchService::new(db.clone(), room_service);
    
    let user = User {
        id: UserId::new(),
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "test_hash".to_string(),
        bio: None,
        admin: false,
        bot_token: None,
        created_at: Utc::now(),
    };
    
    db.writer().create_user(user.clone()).await.unwrap();
    
    // Test query too short
    let request = SearchRequest {
        query: "a".to_string(),
        limit: Some(10),
        offset: Some(0),
        room_id: None,
    };
    
    let result = search_service.search_messages(user.id, request).await;
    assert!(result.is_err());
    
    // Test query too long
    let request = SearchRequest {
        query: "a".repeat(101),
        limit: Some(10),
        offset: Some(0),
        room_id: None,
    };
    
    let result = search_service.search_messages(user.id, request).await;
    assert!(result.is_err());
    
    println!("✅ Search validation test passed!");
}