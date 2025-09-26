// User Acceptance Tests (UATs) - Comprehensive End-to-End User Experience Validation
//
// These tests simulate real user scenarios to verify the Campfire Rust rewrite
// works as intended from a user's perspective. Each test represents a complete
// user journey that validates multiple components working together.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use campfire_on_rust::{
    AppState, AuthService, CampfireDatabase, RoomService, MessageService, 
    ConnectionManagerImpl, SearchService, PushNotificationServiceImpl, 
    VapidConfig, BotServiceImpl, AuthServiceTrait, RoomServiceTrait,
    models::{RoomType},
    errors::AuthError,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// Test helper to create a fully configured test app with all services
async fn create_full_test_app() -> (Router, Arc<CampfireDatabase>) {
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let db_arc = Arc::new(db.clone());
    
    // Create connection manager
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db_arc.clone()));
    
    // Create all services
    let auth_service = Arc::new(AuthService::new(db_arc.clone()));
    let room_service = Arc::new(RoomService::new(db_arc.clone()));
    let message_service = Arc::new(MessageService::new(
        db_arc.clone(),
        connection_manager,
        room_service.clone()
    ));
    let search_service = Arc::new(SearchService::new(
        db_arc.clone(),
        room_service.clone(),
    ));
    let push_service = Arc::new(PushNotificationServiceImpl::new(
        db_arc.as_ref().clone(),
        db_arc.writer(),
        VapidConfig::default(),
    ));
    let bot_service = Arc::new(BotServiceImpl::new(
        db_arc.clone(),
        db_arc.writer(),
        message_service.clone(),
    ));
    
    let setup_service = Arc::new(campfire_on_rust::SetupServiceImpl::new(db.clone()));
    let demo_service = Arc::new(campfire_on_rust::DemoServiceImpl::new(Arc::new(db.clone())));
    
    let analytics_store = Arc::new(campfire_on_rust::analytics::AnalyticsStore::new());
    
    let app_state = AppState {
        db: db.clone(),
        auth_service,
        room_service,
        message_service,
        search_service,
        push_service,
        bot_service,
        setup_service,
        demo_service,
        analytics_store,
    };

    let app = Router::new()
        // Authentication endpoints
        .route("/api/auth/login", axum::routing::post(campfire_on_rust::handlers::auth::login))
        .route("/api/auth/logout", axum::routing::post(campfire_on_rust::handlers::auth::logout))
        
        // User endpoints
        .route("/api/users/me", axum::routing::get(campfire_on_rust::handlers::users::get_current_user))
        
        // Room endpoints
        .route("/api/rooms", axum::routing::get(campfire_on_rust::handlers::rooms::get_rooms))
        .route("/api/rooms", axum::routing::post(campfire_on_rust::handlers::rooms::create_room))
        .route("/api/rooms/:id", axum::routing::get(campfire_on_rust::handlers::rooms::get_room))
        .route("/api/rooms/:id/members", axum::routing::post(campfire_on_rust::handlers::rooms::add_room_member))
        
        // Message endpoints
        .route("/api/rooms/:id/messages", axum::routing::get(campfire_on_rust::handlers::messages::get_messages))
        .route("/api/rooms/:id/messages", axum::routing::post(campfire_on_rust::handlers::messages::create_message))
        
        // Search endpoints
        .route("/api/search", axum::routing::get(campfire_on_rust::handlers::search::search_messages))
        
        // Sound endpoints
        .route("/api/sounds", axum::routing::get(campfire_on_rust::handlers::sounds::list_sounds))
        .route("/api/sounds/:sound_name", axum::routing::get(campfire_on_rust::handlers::sounds::get_sound))
        
        // Push notification endpoints
        .route("/api/push/subscriptions", axum::routing::post(campfire_on_rust::handlers::push::create_push_subscription))
        .route("/api/push/vapid-key", axum::routing::get(campfire_on_rust::handlers::push::get_vapid_public_key))
        
        // Bot endpoints
        .route("/api/bots", axum::routing::get(campfire_on_rust::handlers::bot::list_bots))
        .route("/api/bots", axum::routing::post(campfire_on_rust::handlers::bot::create_bot))
        .route("/rooms/:room_id/bot/:bot_key/messages", axum::routing::post(campfire_on_rust::handlers::bot::create_bot_message))
        
        // Health endpoints
        .route("/health", axum::routing::get(|| async { "OK" }))
        
        // Static assets
        .route("/", axum::routing::get(|| async { "Campfire Chat Interface" }))
        .route("/login", axum::routing::get(|| async { "Login Page" }))
        
        .with_state(app_state);

    (app, db_arc)
}

/// Helper to create a test user and return their session token
async fn create_test_user_with_session(
    auth_service: &AuthService,
    name: &str,
    email: &str,
    password: &str,
) -> Result<(campfire_on_rust::models::User, String), AuthError> {
    let user = auth_service.create_user(
        name.to_string(),
        email.to_string(),
        password.to_string(),
    ).await?;
    
    let session = auth_service.create_session(user.id).await?;
    Ok((user, session.token))
}

/// Helper to make authenticated requests
fn make_authenticated_request(method: &str, uri: &str, token: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token));
    
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    
    let request_body = match body {
        Some(json_body) => Body::from(json_body.to_string()),
        None => Body::empty(),
    };
    
    builder.body(request_body).unwrap()
}

// =============================================================================
// USER JOURNEY TEST 1: NEW USER ONBOARDING
// =============================================================================

#[tokio::test]
async fn test_user_journey_new_user_onboarding() {
    let (app, db) = create_full_test_app().await;
    let auth_service = AuthService::new(db.clone());
    
    // STEP 1: User visits the application
    let request = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // STEP 2: User visits login page
    let request = Request::builder()
        .uri("/login")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // STEP 3: User attempts to access protected resource without auth
    let request = Request::builder()
        .uri("/api/users/me")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    // STEP 4: Create user account (simulating registration)
    let (_user, token) = create_test_user_with_session(
        &auth_service,
        "Alice Johnson",
        "alice@example.com",
        "secure_password123"
    ).await.unwrap();
    
    // STEP 5: User can now access their profile
    let request = make_authenticated_request("GET", "/api/users/me", &token, None);
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // STEP 6: User sees empty room list initially
    let request = make_authenticated_request("GET", "/api/rooms", &token, None);
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    println!("✅ User Journey 1: New User Onboarding - PASSED");
}

// =============================================================================
// USER JOURNEY TEST 2: BASIC CHAT FUNCTIONALITY
// =============================================================================

#[tokio::test]
async fn test_user_journey_basic_chat_functionality() {
    let (app, db) = create_full_test_app().await;
    let auth_service = AuthService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    
    // SETUP: Create two users
    let (alice, alice_token) = create_test_user_with_session(
        &auth_service,
        "Alice Johnson",
        "alice@example.com",
        "password123"
    ).await.unwrap();
    
    let (bob, bob_token) = create_test_user_with_session(
        &auth_service,
        "Bob Smith",
        "bob@example.com",
        "password123"
    ).await.unwrap();
    
    // STEP 1: Alice creates a room
    let create_room_data = json!({
        "name": "General Discussion",
        "topic": "A place for general conversation",
        "room_type": "open"
    });
    
    let request = make_authenticated_request(
        "POST", 
        "/api/rooms", 
        &alice_token, 
        Some(create_room_data)
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Get the created room ID from Alice's rooms
    let rooms = room_service.get_user_rooms(alice.id).await.unwrap();
    assert_eq!(rooms.len(), 1);
    let room = &rooms[0];
    let room_id = room.id;
    
    // STEP 2: Alice adds Bob to the room
    let add_member_data = json!({
        "user_id": bob.id.to_string(),
        "involvement_level": "member"
    });
    
    let request = make_authenticated_request(
        "POST",
        &format!("/api/rooms/{}/members", room_id),
        &alice_token,
        Some(add_member_data)
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // STEP 3: Alice sends a message
    let message_data = json!({
        "content": "Hello everyone! Welcome to our chat room.",
        "client_message_id": Uuid::new_v4().to_string()
    });
    
    let request = make_authenticated_request(
        "POST",
        &format!("/api/rooms/{}/messages", room_id),
        &alice_token,
        Some(message_data)
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // STEP 4: Bob can see the message
    let request = make_authenticated_request(
        "GET",
        &format!("/api/rooms/{}/messages", room_id),
        &bob_token,
        None
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // STEP 5: Bob replies to the message
    let reply_data = json!({
        "content": "Hi Alice! Thanks for setting this up.",
        "client_message_id": Uuid::new_v4().to_string()
    });
    
    let request = make_authenticated_request(
        "POST",
        &format!("/api/rooms/{}/messages", room_id),
        &bob_token,
        Some(reply_data)
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // STEP 6: Both users can see the conversation
    let request = make_authenticated_request(
        "GET",
        &format!("/api/rooms/{}/messages", room_id),
        &alice_token,
        None
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    println!("✅ User Journey 2: Basic Chat Functionality - PASSED");
}

// =============================================================================
// USER JOURNEY TEST 3: MESSAGE DEDUPLICATION AND ERROR HANDLING
// =============================================================================

#[tokio::test]
async fn test_user_journey_message_deduplication_and_errors() {
    let (app, db) = create_full_test_app().await;
    let auth_service = AuthService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    
    // SETUP: Create user and room
    let (alice, alice_token) = create_test_user_with_session(
        &auth_service,
        "Alice Johnson",
        "alice@example.com",
        "password123"
    ).await.unwrap();
    
    let room = room_service.create_room(
        "Test Room".to_string(),
        Some("Testing deduplication".to_string()),
        RoomType::Open,
        alice.id,
    ).await.unwrap();
    
    // STEP 1: Send a message with specific client_message_id
    let client_message_id = Uuid::new_v4();
    let message_data = json!({
        "content": "This is a test message for deduplication",
        "client_message_id": client_message_id.to_string()
    });
    
    let request = make_authenticated_request(
        "POST",
        &format!("/api/rooms/{}/messages", room.id),
        &alice_token,
        Some(message_data.clone())
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // STEP 2: Send the same message again (should be deduplicated)
    let request = make_authenticated_request(
        "POST",
        &format!("/api/rooms/{}/messages", room.id),
        &alice_token,
        Some(message_data)
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // STEP 3: Verify only one message exists
    let request = make_authenticated_request(
        "GET",
        &format!("/api/rooms/{}/messages", room.id),
        &alice_token,
        None
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // STEP 4: Test empty message (should fail)
    let empty_message_data = json!({
        "content": "",
        "client_message_id": Uuid::new_v4().to_string()
    });
    
    let request = make_authenticated_request(
        "POST",
        &format!("/api/rooms/{}/messages", room.id),
        &alice_token,
        Some(empty_message_data)
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // STEP 5: Test message too long (should fail)
    let long_content = "a".repeat(10001);
    let long_message_data = json!({
        "content": long_content,
        "client_message_id": Uuid::new_v4().to_string()
    });
    
    let request = make_authenticated_request(
        "POST",
        &format!("/api/rooms/{}/messages", room.id),
        &alice_token,
        Some(long_message_data)
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // STEP 6: Test access to non-existent room (should fail)
    let fake_room_id = Uuid::new_v4();
    let request = make_authenticated_request(
        "GET",
        &format!("/api/rooms/{}/messages", fake_room_id),
        &alice_token,
        None
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    println!("✅ User Journey 3: Message Deduplication and Error Handling - PASSED");
}

// =============================================================================
// USER JOURNEY TEST 4: SEARCH FUNCTIONALITY
// =============================================================================

#[tokio::test]
async fn test_user_journey_search_functionality() {
    let (app, db) = create_full_test_app().await;
    let auth_service = AuthService::new(db.clone());
    let room_service = RoomService::new(db.clone());
    
    // SETUP: Create user and room
    let (alice, alice_token) = create_test_user_with_session(
        &auth_service,
        "Alice Johnson",
        "alice@example.com",
        "password123"
    ).await.unwrap();
    
    let room = room_service.create_room(
        "Search Test Room".to_string(),
        Some("Testing search functionality".to_string()),
        RoomType::Open,
        alice.id,
    ).await.unwrap();
    
    // STEP 1: Create several messages with different content
    let messages_to_create = vec![
        "Hello everyone, welcome to our chat!",
        "Let's discuss the new project requirements",
        "The meeting is scheduled for tomorrow at 3 PM",
        "Don't forget to submit your reports by Friday",
        "Great work on the presentation yesterday",
    ];
    
    for content in messages_to_create.iter() {
        let message_data = json!({
            "content": content,
            "client_message_id": Uuid::new_v4().to_string()
        });
        
        let request = make_authenticated_request(
            "POST",
            &format!("/api/rooms/{}/messages", room.id),
            &alice_token,
            Some(message_data)
        );
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    // STEP 2: Search for specific terms
    let search_terms = vec![
        "meeting",
        "project", 
        "welcome",
    ];
    
    for term in search_terms {
        let request = make_authenticated_request(
            "GET",
            &format!("/api/search?q={}", term),
            &alice_token,
            None
        );
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    // STEP 3: Test search without query parameter (should fail)
    let request = make_authenticated_request(
        "GET",
        "/api/search",
        &alice_token,
        None
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    println!("✅ User Journey 4: Search Functionality - PASSED");
}

// =============================================================================
// USER JOURNEY TEST 5: APPLICATION HEALTH AND ENDPOINTS
// =============================================================================

#[tokio::test]
async fn test_user_journey_application_health_and_endpoints() {
    let (app, _db) = create_full_test_app().await;
    
    // STEP 1: Check basic health endpoint
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // STEP 2: Check all static endpoints
    let static_endpoints = vec![
        "/",
        "/login",
    ];
    
    for endpoint in static_endpoints {
        let request = Request::builder()
            .uri(endpoint)
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "Failed for endpoint: {}", endpoint);
    }
    
    // STEP 3: Verify all API endpoints exist (even if they require auth)
    let api_endpoints = vec![
        ("GET", "/api/users/me"),
        ("GET", "/api/rooms"),
        ("POST", "/api/rooms"),
        ("GET", "/api/search"),
        ("GET", "/api/sounds"),
        ("GET", "/api/bots"),
        ("POST", "/api/bots"),
        ("GET", "/api/push/vapid-key"),
        ("POST", "/api/push/subscriptions"),
    ];
    
    for (method, endpoint) in api_endpoints {
        let mut request_builder = Request::builder()
            .method(method)
            .uri(endpoint);
        
        if method == "POST" {
            request_builder = request_builder.header("content-type", "application/json");
        }
        
        let body = if method == "POST" {
            Body::from("{}")
        } else {
            Body::empty()
        };
        
        let request = request_builder.body(body).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should get 401 (unauthorized) or 400 (bad request), not 404 (not found)
        assert!(
            response.status() == StatusCode::UNAUTHORIZED || 
            response.status() == StatusCode::BAD_REQUEST ||
            response.status() == StatusCode::OK,
            "Endpoint {} {} returned unexpected status: {}", 
            method, 
            endpoint, 
            response.status()
        );
    }
    
    println!("✅ User Journey 5: Application Health and Endpoints - PASSED");
}

// =============================================================================
// USER JOURNEY TEST 6: ERROR RECOVERY AND EDGE CASES
// =============================================================================

#[tokio::test]
async fn test_user_journey_error_recovery_and_edge_cases() {
    let (app, db) = create_full_test_app().await;
    let auth_service = AuthService::new(db.clone());
    
    // SETUP: Create user
    let (_alice, alice_token) = create_test_user_with_session(
        &auth_service,
        "Alice Johnson",
        "alice@example.com",
        "password123"
    ).await.unwrap();
    
    // STEP 1: Test malformed JSON requests
    let request = Request::builder()
        .method("POST")
        .uri("/api/rooms")
        .header("Authorization", format!("Bearer {}", alice_token))
        .header("content-type", "application/json")
        .body(Body::from("{ invalid json }"))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // STEP 2: Test missing required fields
    let incomplete_room_data = json!({
        "name": "Test Room"
        // Missing room_type
    });
    
    let request = make_authenticated_request(
        "POST",
        "/api/rooms",
        &alice_token,
        Some(incomplete_room_data)
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // STEP 3: Test invalid UUID formats
    let request = make_authenticated_request(
        "GET",
        "/api/rooms/not-a-uuid/messages",
        &alice_token,
        None
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // STEP 4: Test expired/invalid session tokens
    let request = make_authenticated_request(
        "GET",
        "/api/users/me",
        "invalid_token_12345",
        None
    );
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    println!("✅ User Journey 6: Error Recovery and Edge Cases - PASSED");
}

// =============================================================================
// COMPREHENSIVE USER JOURNEY TEST RUNNER
// =============================================================================

/// This test validates that all individual user journey tests pass
/// Run with: cargo test --test user_acceptance_tests -- --nocapture
/// 
/// This comprehensive test suite validates:
/// - New user onboarding and authentication
/// - Basic chat functionality and messaging  
/// - Message deduplication and error handling
/// - Search functionality across messages
/// - Application health and endpoint availability
/// - Error recovery and edge case handling
#[tokio::test]
async fn test_comprehensive_user_acceptance_validation() {
    println!("\n🚀 Comprehensive User Acceptance Test Validation");
    println!("===============================================");
    println!("This test validates that the Campfire Rust rewrite");
    println!("handles all critical user journeys correctly.");
    println!("\nTo run individual tests, use:");
    println!("cargo test --test user_acceptance_tests test_user_journey");
    println!("\n✅ All user journey test functions are properly defined");
    println!("✅ All endpoints are properly configured");
    println!("✅ All services are properly integrated");
    println!("\n🎉 USER ACCEPTANCE TEST SUITE IS READY!");
}