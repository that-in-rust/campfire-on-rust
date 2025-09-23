use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use campfire_on_rust::{
    AppState, CampfireDatabase, AuthService, RoomService, MessageService, 
    ConnectionManagerImpl, SearchService, AuthServiceTrait,
};
use campfire_on_rust::models::*;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;
use chrono::Utc;

async fn setup_test_app() -> (Router, Arc<CampfireDatabase>, User, String) {
    let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db.clone()));
    
    let auth_service = Arc::new(AuthService::new(db.clone()));
    let room_service = Arc::new(RoomService::new(db.clone()));
    let message_service = Arc::new(MessageService::new(
        db.clone(),
        connection_manager,
        room_service.clone(),
    ));
    let search_service = Arc::new(SearchService::new(
        db.clone(),
        room_service.clone(),
    ));
    
    let push_service = Arc::new(campfire_on_rust::PushNotificationServiceImpl::new(
        db.as_ref().clone(),
        db.writer(),
        campfire_on_rust::VapidConfig::default(),
    ));
    let bot_service = Arc::new(campfire_on_rust::BotServiceImpl::new(
        db.clone(),
        db.writer(),
        message_service.clone(),
    ));
    
    let app_state = AppState {
        db: CampfireDatabase::new(":memory:").await.unwrap(),
        auth_service: auth_service.clone(),
        room_service: room_service.clone(),
        message_service,
        search_service,
        push_service,
        bot_service,
    };
    
    let app = Router::new()
        .route("/api/search", axum::routing::get(campfire_on_rust::handlers::search::search_messages))
        .with_state(app_state);
    
    // Create test user and session
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
    
    let session = auth_service.create_session(user.id).await.unwrap();
    
    (app, db, user, session.token)
}

async fn create_test_room_with_messages(
    db: &CampfireDatabase,
    user: &User,
    room_name: &str,
    messages: &[&str],
) -> Room {
    let room = Room {
        id: RoomId::new(),
        name: room_name.to_string(),
        topic: None,
        room_type: RoomType::Open,
        created_at: Utc::now(),
        last_message_at: None,
    };
    
    db.writer().create_room(room.clone()).await.unwrap();
    
    let membership = Membership {
        room_id: room.id,
        user_id: user.id,
        involvement_level: InvolvementLevel::Member,
        created_at: Utc::now(),
    };
    
    db.writer().create_membership(membership).await.unwrap();
    
    for content in messages {
        let message = Message {
            id: MessageId::new(),
            room_id: room.id,
            creator_id: user.id,
            content: content.to_string(),
            client_message_id: Uuid::new_v4(),
            created_at: Utc::now(),
            html_content: None,
            mentions: Vec::new(),
            sound_commands: Vec::new(),
        };
        
        db.writer().create_message_with_deduplication(message).await.unwrap();
    }
    
    room
}

#[tokio::test]
async fn test_search_messages_success() {
    let (app, db, user, session_token) = setup_test_app().await;
    
    // Create test room with messages
    create_test_room_with_messages(
        &db,
        &user,
        "Test Room",
        &["Hello world", "This is a test message", "Another message"],
    ).await;
    
    // Make search request
    let request = Request::builder()
        .method("GET")
        .uri("/api/search?q=test")
        .header("Authorization", format!("Bearer {}", session_token))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["results"].as_array().unwrap().len(), 1);
    assert!(json["results"][0]["message"]["content"].as_str().unwrap().contains("test"));
    assert_eq!(json["total_count"], 1);
    assert_eq!(json["query"], "test");
}

#[tokio::test]
async fn test_search_messages_with_pagination() {
    let (app, db, user, session_token) = setup_test_app().await;
    
    // Create test room with multiple messages
    create_test_room_with_messages(
        &db,
        &user,
        "Test Room",
        &[
            "Test message 1",
            "Test message 2", 
            "Test message 3",
            "Test message 4",
            "Test message 5",
        ],
    ).await;
    
    // Make search request with pagination
    let request = Request::builder()
        .method("GET")
        .uri("/api/search?q=test&limit=2&offset=0")
        .header("Authorization", format!("Bearer {}", session_token))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["results"].as_array().unwrap().len(), 2);
    assert_eq!(json["total_count"], 5);
    assert_eq!(json["limit"], 2);
    assert_eq!(json["offset"], 0);
    assert_eq!(json["has_more"], true);
}

#[tokio::test]
async fn test_search_messages_room_specific() {
    let (app, db, user, session_token) = setup_test_app().await;
    
    // Create two rooms with different messages
    let room1 = create_test_room_with_messages(
        &db,
        &user,
        "Room 1",
        &["Hello from room 1"],
    ).await;
    
    create_test_room_with_messages(
        &db,
        &user,
        "Room 2",
        &["Hello from room 2"],
    ).await;
    
    // Search in specific room
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/api/search?q=hello&room_id={}", room1.id.0))
        .header("Authorization", format!("Bearer {}", session_token))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["results"].as_array().unwrap().len(), 1);
    assert!(json["results"][0]["message"]["content"].as_str().unwrap().contains("room 1"));
}

#[tokio::test]
async fn test_search_messages_unauthorized() {
    let (app, _db, _user, _session_token) = setup_test_app().await;
    
    // Make search request without authentication
    let request = Request::builder()
        .method("GET")
        .uri("/api/search?q=test")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_search_messages_invalid_query() {
    let (app, _db, _user, session_token) = setup_test_app().await;
    
    // Make search request with empty query
    let request = Request::builder()
        .method("GET")
        .uri("/api/search?q=")
        .header("Authorization", format!("Bearer {}", session_token))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["type"], "invalid_query");
}

#[tokio::test]
async fn test_search_messages_query_too_short() {
    let (app, _db, _user, session_token) = setup_test_app().await;
    
    // Make search request with query too short
    let request = Request::builder()
        .method("GET")
        .uri("/api/search?q=a")
        .header("Authorization", format!("Bearer {}", session_token))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["type"], "query_too_short");
}

#[tokio::test]
async fn test_search_messages_query_too_long() {
    let (app, _db, _user, session_token) = setup_test_app().await;
    
    // Make search request with query too long
    let long_query = "a".repeat(101);
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/api/search?q={}", long_query))
        .header("Authorization", format!("Bearer {}", session_token))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["type"], "query_too_long");
}

#[tokio::test]
async fn test_search_messages_no_results() {
    let (app, db, user, session_token) = setup_test_app().await;
    
    // Create test room with messages
    create_test_room_with_messages(
        &db,
        &user,
        "Test Room",
        &["Hello world", "This is a message"],
    ).await;
    
    // Search for non-existent term
    let request = Request::builder()
        .method("GET")
        .uri("/api/search?q=nonexistent")
        .header("Authorization", format!("Bearer {}", session_token))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["results"].as_array().unwrap().len(), 0);
    assert_eq!(json["total_count"], 0);
    assert_eq!(json["has_more"], false);
}

#[tokio::test]
async fn test_search_messages_authorization_filtering() {
    let (app, db, user, session_token) = setup_test_app().await;
    
    // Create another user
    let other_user = User {
        id: UserId::new(),
        name: "Other User".to_string(),
        email: "other@example.com".to_string(),
        password_hash: "test_hash".to_string(),
        bio: None,
        admin: false,
        bot_token: None,
        created_at: Utc::now(),
    };
    
    db.writer().create_user(other_user.clone()).await.unwrap();
    
    // Create a private room with other user only
    let private_room = Room {
        id: RoomId::new(),
        name: "Private Room".to_string(),
        topic: None,
        room_type: RoomType::Closed,
        created_at: Utc::now(),
        last_message_at: None,
    };
    
    db.writer().create_room(private_room.clone()).await.unwrap();
    
    let private_membership = Membership {
        room_id: private_room.id,
        user_id: other_user.id,
        involvement_level: InvolvementLevel::Member,
        created_at: Utc::now(),
    };
    
    db.writer().create_membership(private_membership).await.unwrap();
    
    // Create message in private room
    let private_message = Message {
        id: MessageId::new(),
        room_id: private_room.id,
        creator_id: other_user.id,
        content: "Secret message".to_string(),
        client_message_id: Uuid::new_v4(),
        created_at: Utc::now(),
        html_content: None,
        mentions: Vec::new(),
        sound_commands: Vec::new(),
    };
    
    db.writer().create_message_with_deduplication(private_message).await.unwrap();
    
    // Create accessible room for test user
    create_test_room_with_messages(
        &db,
        &user,
        "Public Room",
        &["Public message"],
    ).await;
    
    // Search should not return private message
    let request = Request::builder()
        .method("GET")
        .uri("/api/search?q=message")
        .header("Authorization", format!("Bearer {}", session_token))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    // Should only find the public message, not the secret one
    assert_eq!(json["results"].as_array().unwrap().len(), 1);
    assert!(json["results"][0]["message"]["content"].as_str().unwrap().contains("Public"));
    assert!(!json["results"][0]["message"]["content"].as_str().unwrap().contains("Secret"));
}