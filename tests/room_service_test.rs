use campfire_on_rust::{CampfireDatabase, RoomService, RoomServiceTrait};
use campfire_on_rust::models::{
    User, UserId, RoomId, RoomType, InvolvementLevel, CreateRoomRequest
};
use campfire_on_rust::errors::RoomError;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

async fn create_test_db() -> CampfireDatabase {
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    db
}

async fn create_test_user(db: &CampfireDatabase, email: &str, name: &str) -> UserId {
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
    
    db.create_user(user.clone()).await.unwrap();
    user.id
}

#[tokio::test]
async fn test_create_room_success() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let creator_id = create_test_user(&db, "creator@test.com", "Creator").await;
    
    let room = room_service.create_room(
        "Test Room".to_string(),
        Some("A test room".to_string()),
        RoomType::Open,
        creator_id,
    ).await.unwrap();
    
    assert_eq!(room.name, "Test Room");
    assert_eq!(room.topic, Some("A test room".to_string()));
    assert!(matches!(room.room_type, RoomType::Open));
    
    // Verify creator is admin member
    let access = room_service.check_room_access(room.id, creator_id).await.unwrap();
    assert!(matches!(access, Some(InvolvementLevel::Admin)));
}

#[tokio::test]
async fn test_create_room_invalid_name() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let creator_id = create_test_user(&db, "creator@test.com", "Creator").await;
    
    // Empty name
    let result = room_service.create_room(
        "".to_string(),
        None,
        RoomType::Open,
        creator_id,
    ).await;
    
    assert!(matches!(result, Err(RoomError::InvalidName { .. })));
    
    // Name too long
    let long_name = "a".repeat(101);
    let result = room_service.create_room(
        long_name,
        None,
        RoomType::Open,
        creator_id,
    ).await;
    
    assert!(matches!(result, Err(RoomError::InvalidName { .. })));
}

#[tokio::test]
async fn test_create_room_trims_whitespace() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let creator_id = create_test_user(&db, "creator@test.com", "Creator").await;
    
    let room = room_service.create_room(
        "  Test Room  ".to_string(),
        Some("  A test room  ".to_string()),
        RoomType::Open,
        creator_id,
    ).await.unwrap();
    
    assert_eq!(room.name, "Test Room");
    assert_eq!(room.topic, Some("A test room".to_string()));
}

#[tokio::test]
async fn test_add_member_success() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let creator_id = create_test_user(&db, "creator@test.com", "Creator").await;
    let member_id = create_test_user(&db, "member@test.com", "Member").await;
    
    // Create room
    let room = room_service.create_room(
        "Test Room".to_string(),
        None,
        RoomType::Closed,
        creator_id,
    ).await.unwrap();
    
    // Add member
    room_service.add_member(
        room.id,
        member_id,
        creator_id,
        InvolvementLevel::Member,
    ).await.unwrap();
    
    // Verify member has access
    let access = room_service.check_room_access(room.id, member_id).await.unwrap();
    assert!(matches!(access, Some(InvolvementLevel::Member)));
}

#[tokio::test]
async fn test_add_member_not_authorized() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let creator_id = create_test_user(&db, "creator@test.com", "Creator").await;
    let member_id = create_test_user(&db, "member@test.com", "Member").await;
    let other_user_id = create_test_user(&db, "other@test.com", "Other").await;
    
    // Create closed room
    let room = room_service.create_room(
        "Test Room".to_string(),
        None,
        RoomType::Closed,
        creator_id,
    ).await.unwrap();
    
    // Try to add member as non-admin user
    let result = room_service.add_member(
        room.id,
        member_id,
        other_user_id,
        InvolvementLevel::Member,
    ).await;
    
    assert!(matches!(result, Err(RoomError::NotAuthorized { .. })));
}

#[tokio::test]
async fn test_add_member_already_member() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let creator_id = create_test_user(&db, "creator@test.com", "Creator").await;
    let member_id = create_test_user(&db, "member@test.com", "Member").await;
    
    // Create room
    let room = room_service.create_room(
        "Test Room".to_string(),
        None,
        RoomType::Closed,
        creator_id,
    ).await.unwrap();
    
    // Add member
    room_service.add_member(
        room.id,
        member_id,
        creator_id,
        InvolvementLevel::Member,
    ).await.unwrap();
    
    // Try to add same member again
    let result = room_service.add_member(
        room.id,
        member_id,
        creator_id,
        InvolvementLevel::Member,
    ).await;
    
    assert!(matches!(result, Err(RoomError::AlreadyMember { .. })));
}

#[tokio::test]
async fn test_add_member_room_not_found() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let creator_id = create_test_user(&db, "creator@test.com", "Creator").await;
    let member_id = create_test_user(&db, "member@test.com", "Member").await;
    
    let fake_room_id = RoomId::new();
    
    let result = room_service.add_member(
        fake_room_id,
        member_id,
        creator_id,
        InvolvementLevel::Member,
    ).await;
    
    assert!(matches!(result, Err(RoomError::NotFound { .. })));
}

#[tokio::test]
async fn test_check_room_access_open_room() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let creator_id = create_test_user(&db, "creator@test.com", "Creator").await;
    let other_user_id = create_test_user(&db, "other@test.com", "Other").await;
    
    // Create open room
    let room = room_service.create_room(
        "Open Room".to_string(),
        None,
        RoomType::Open,
        creator_id,
    ).await.unwrap();
    
    // Creator should be admin
    let access = room_service.check_room_access(room.id, creator_id).await.unwrap();
    assert!(matches!(access, Some(InvolvementLevel::Admin)));
    
    // Other user should have member access to open room
    let access = room_service.check_room_access(room.id, other_user_id).await.unwrap();
    assert!(matches!(access, Some(InvolvementLevel::Member)));
}

#[tokio::test]
async fn test_check_room_access_closed_room() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let creator_id = create_test_user(&db, "creator@test.com", "Creator").await;
    let member_id = create_test_user(&db, "member@test.com", "Member").await;
    let other_user_id = create_test_user(&db, "other@test.com", "Other").await;
    
    // Create closed room
    let room = room_service.create_room(
        "Closed Room".to_string(),
        None,
        RoomType::Closed,
        creator_id,
    ).await.unwrap();
    
    // Add member
    room_service.add_member(
        room.id,
        member_id,
        creator_id,
        InvolvementLevel::Member,
    ).await.unwrap();
    
    // Creator should be admin
    let access = room_service.check_room_access(room.id, creator_id).await.unwrap();
    assert!(matches!(access, Some(InvolvementLevel::Admin)));
    
    // Member should have member access
    let access = room_service.check_room_access(room.id, member_id).await.unwrap();
    assert!(matches!(access, Some(InvolvementLevel::Member)));
    
    // Other user should have no access
    let access = room_service.check_room_access(room.id, other_user_id).await.unwrap();
    assert!(access.is_none());
}

#[tokio::test]
async fn test_check_room_access_room_not_found() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let user_id = create_test_user(&db, "user@test.com", "User").await;
    let fake_room_id = RoomId::new();
    
    let result = room_service.check_room_access(fake_room_id, user_id).await;
    assert!(matches!(result, Err(RoomError::NotFound { .. })));
}

#[tokio::test]
async fn test_get_user_rooms() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let user_id = create_test_user(&db, "user@test.com", "User").await;
    let other_user_id = create_test_user(&db, "other@test.com", "Other").await;
    
    // Create rooms
    let room1 = room_service.create_room(
        "Room 1".to_string(),
        None,
        RoomType::Open,
        user_id,
    ).await.unwrap();
    
    let room2 = room_service.create_room(
        "Room 2".to_string(),
        None,
        RoomType::Closed,
        other_user_id,
    ).await.unwrap();
    
    // Add user to room2
    room_service.add_member(
        room2.id,
        user_id,
        other_user_id,
        InvolvementLevel::Member,
    ).await.unwrap();
    
    // Get user's rooms
    let rooms = room_service.get_user_rooms(user_id).await.unwrap();
    
    assert_eq!(rooms.len(), 2);
    
    // Find rooms by name (order might vary)
    let room1_found = rooms.iter().find(|r| r.name == "Room 1").unwrap();
    let room2_found = rooms.iter().find(|r| r.name == "Room 2").unwrap();
    
    assert_eq!(room1_found.id, room1.id);
    assert_eq!(room2_found.id, room2.id);
}

#[tokio::test]
async fn test_get_user_rooms_empty() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let user_id = create_test_user(&db, "user@test.com", "User").await;
    
    let rooms = room_service.get_user_rooms(user_id).await.unwrap();
    assert!(rooms.is_empty());
}

// Property-based test for room name validation
#[tokio::test]
async fn test_room_name_validation_properties() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    let creator_id = create_test_user(&db, "creator@test.com", "Creator").await;
    
    // Test various edge cases
    let max_length_name = "a".repeat(100);
    let too_long_name = "a".repeat(101);
    let test_cases = vec![
        ("", false),                    // Empty
        ("   ", false),                 // Whitespace only
        ("a", true),                    // Single char
        ("Valid Room", true),           // Normal case
        ("  Valid Room  ", true),       // Trimmed
        (max_length_name.as_str(), true),       // Max length
        (too_long_name.as_str(), false),      // Too long
    ];
    
    for (name, should_succeed) in test_cases {
        let result = room_service.create_room(
            name.to_string(),
            None,
            RoomType::Open,
            creator_id,
        ).await;
        
        if should_succeed {
            assert!(result.is_ok(), "Expected success for name: '{}'", name);
        } else {
            assert!(result.is_err(), "Expected failure for name: '{}'", name);
        }
    }
}

// Integration test for complete room workflow
#[tokio::test]
async fn test_complete_room_workflow() {
    let db = create_test_db().await;
    let room_service = RoomService::new(Arc::new(db.clone()));
    
    // Create users
    let admin_id = create_test_user(&db, "admin@test.com", "Admin").await;
    let member1_id = create_test_user(&db, "member1@test.com", "Member1").await;
    let member2_id = create_test_user(&db, "member2@test.com", "Member2").await;
    let outsider_id = create_test_user(&db, "outsider@test.com", "Outsider").await;
    
    // 1. Create room
    let room = room_service.create_room(
        "Project Room".to_string(),
        Some("Discussion for our project".to_string()),
        RoomType::Closed,
        admin_id,
    ).await.unwrap();
    
    // 2. Add members
    room_service.add_member(
        room.id,
        member1_id,
        admin_id,
        InvolvementLevel::Member,
    ).await.unwrap();
    
    room_service.add_member(
        room.id,
        member2_id,
        admin_id,
        InvolvementLevel::Admin,
    ).await.unwrap();
    
    // 3. Verify access levels
    let admin_access = room_service.check_room_access(room.id, admin_id).await.unwrap();
    assert!(matches!(admin_access, Some(InvolvementLevel::Admin)));
    
    let member1_access = room_service.check_room_access(room.id, member1_id).await.unwrap();
    assert!(matches!(member1_access, Some(InvolvementLevel::Member)));
    
    let member2_access = room_service.check_room_access(room.id, member2_id).await.unwrap();
    assert!(matches!(member2_access, Some(InvolvementLevel::Admin)));
    
    let outsider_access = room_service.check_room_access(room.id, outsider_id).await.unwrap();
    assert!(outsider_access.is_none());
    
    // 4. Verify user rooms
    let admin_rooms = room_service.get_user_rooms(admin_id).await.unwrap();
    assert_eq!(admin_rooms.len(), 1);
    assert_eq!(admin_rooms[0].id, room.id);
    
    let member1_rooms = room_service.get_user_rooms(member1_id).await.unwrap();
    assert_eq!(member1_rooms.len(), 1);
    assert_eq!(member1_rooms[0].id, room.id);
    
    let outsider_rooms = room_service.get_user_rooms(outsider_id).await.unwrap();
    assert!(outsider_rooms.is_empty());
    
    // 5. Test member2 can add new members (is admin)
    let new_member_id = create_test_user(&db, "newmember@test.com", "NewMember").await;
    room_service.add_member(
        room.id,
        new_member_id,
        member2_id,  // member2 is admin
        InvolvementLevel::Member,
    ).await.unwrap();
    
    let new_member_access = room_service.check_room_access(room.id, new_member_id).await.unwrap();
    assert!(matches!(new_member_access, Some(InvolvementLevel::Member)));
}