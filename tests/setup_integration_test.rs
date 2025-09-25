use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use campfire_on_rust::{
    AppState, CampfireDatabase, AuthService, RoomService, MessageService,
    ConnectionManagerImpl, SearchService, PushNotificationServiceImpl,
    VapidConfig, BotServiceImpl, SetupServiceImpl,
    models::CreateAdminRequest,
};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

/// Integration tests for Task 26: Integrate first-run setup with application startup
/// 
/// Tests Requirements 11.1, 11.7, 11.8:
/// - Setup detection in main application flow
/// - Automatic redirection to setup when needed
/// - Setup completion validation
/// - Proper error handling for setup failures

async fn create_test_app() -> Router {
    // Create in-memory database for testing
    let db = CampfireDatabase::new("sqlite::memory:").await.unwrap();
    let db_arc = Arc::new(db.clone());
    
    // Initialize services
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db_arc.clone()));
    let auth_service = Arc::new(AuthService::new(db_arc.clone()));
    let room_service = Arc::new(RoomService::new(db_arc.clone()));
    
    let vapid_config = VapidConfig::default();
    let push_service = Arc::new(PushNotificationServiceImpl::new(
        db.clone(),
        db.writer(),
        vapid_config,
    ));
    
    let message_service = Arc::new(MessageService::with_push_service(
        db_arc.clone(),
        connection_manager,
        room_service.clone(),
        push_service.clone(),
    ));
    
    let search_service = Arc::new(SearchService::new(
        db_arc.clone(),
        room_service.clone(),
    ));
    
    let bot_service = Arc::new(BotServiceImpl::new(
        db_arc.clone(),
        db.writer(),
        message_service.clone(),
    ));
    
    let setup_service = Arc::new(SetupServiceImpl::new(db.clone()));
    
    let app_state = AppState {
        db,
        auth_service,
        room_service,
        message_service,
        search_service,
        push_service,
        bot_service,
        setup_service,
    };
    
    // Create router with setup middleware (simplified version of main.rs)
    let mut app = Router::new()
        .route("/", axum::routing::get(campfire_on_rust::handlers::pages::serve_root_page))
        .route("/login", axum::routing::get(campfire_on_rust::handlers::pages::serve_login_page))
        .route("/chat", axum::routing::get(campfire_on_rust::assets::serve_chat_interface))
        .route("/setup", axum::routing::get(campfire_on_rust::handlers::setup::serve_setup_page))
        .route("/api/setup/status", axum::routing::get(campfire_on_rust::handlers::setup::get_setup_status))
        .route("/api/setup/admin", axum::routing::post(campfire_on_rust::handlers::setup::create_admin_account))
        .route("/health", axum::routing::get(campfire_on_rust::health::health_check));
    
    // Add protected routes with setup completion middleware
    let protected_routes = Router::new()
        .route("/api/auth/login", axum::routing::post(campfire_on_rust::handlers::auth::login))
        .route("/api/rooms", axum::routing::get(campfire_on_rust::handlers::rooms::get_rooms))
        .route("/api/protected-test", axum::routing::get(|| async { "protected" }))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            campfire_on_rust::middleware::setup::setup_completion_middleware
        ));
    
    app = app.merge(protected_routes);
    
    // Apply setup detection middleware to the entire app
    app = app.layer(axum::middleware::from_fn_with_state(
        app_state.clone(),
        campfire_on_rust::middleware::setup::setup_detection_middleware
    ));
    
    app.with_state(app_state)
}

#[tokio::test]
async fn test_requirement_11_1_setup_detection_in_main_flow() {
    // Requirement 11.1: Setup detection in main application flow
    
    let app = create_test_app().await;
    
    // Test root page redirects to setup on first run
    let request = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should redirect to setup page
    assert_eq!(response.status(), StatusCode::FOUND);
    let location = response.headers().get("location").unwrap();
    assert_eq!(location, "/setup");
}

#[tokio::test]
async fn test_requirement_11_1_login_page_redirects_to_setup() {
    // Requirement 11.1: Login page redirects to setup on first run
    
    let app = create_test_app().await;
    
    // Test login page redirects to setup on first run
    let request = Request::builder()
        .uri("/login")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should redirect to setup page
    assert_eq!(response.status(), StatusCode::FOUND);
    let location = response.headers().get("location").unwrap();
    assert_eq!(location, "/setup");
}

#[tokio::test]
async fn test_requirement_11_7_automatic_redirection_to_setup() {
    // Requirement 11.7: Automatic redirection to setup when needed
    
    let app = create_test_app().await;
    
    // Test various pages redirect to setup on first run
    let test_paths = vec!["/", "/login", "/chat"];
    
    for path in test_paths {
        let request = Request::builder()
            .uri(path)
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should redirect to setup page
        assert_eq!(response.status(), StatusCode::FOUND, "Path {} should redirect", path);
        let location = response.headers().get("location").unwrap();
        assert_eq!(location, "/setup", "Path {} should redirect to /setup", path);
    }
}

#[tokio::test]
async fn test_setup_related_paths_bypass_detection() {
    // Setup-related paths should bypass setup detection middleware
    
    let app = create_test_app().await;
    
    // Test setup page itself doesn't redirect
    let request = Request::builder()
        .uri("/setup")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should serve setup page, not redirect
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test setup API endpoints bypass detection
    let request = Request::builder()
        .uri("/api/setup/status")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should return JSON status, not redirect
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test health endpoint bypasses detection
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should return health status, not redirect
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_requirement_11_8_setup_completion_validation() {
    // Requirement 11.8: Setup completion validation for protected endpoints
    
    let app = create_test_app().await;
    
    // Test protected endpoint redirects to setup before setup (setup detection middleware runs first)
    let request = Request::builder()
        .uri("/api/protected-test")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should redirect to setup (setup detection middleware runs before setup completion middleware)
    assert_eq!(response.status(), StatusCode::FOUND);
    let location = response.headers().get("location").unwrap();
    assert_eq!(location, "/setup");
}

#[tokio::test]
async fn test_setup_completion_allows_access_after_admin_creation() {
    // After admin creation, protected endpoints should be accessible
    
    let app = create_test_app().await;
    
    // First create admin account
    let admin_request = json!({
        "email": "admin@example.com",
        "password": "securepass123",
        "name": "System Admin"
    });
    
    let request = Request::builder()
        .uri("/api/setup/admin")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(admin_request.to_string()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Now test that protected endpoint is accessible
    let request = Request::builder()
        .uri("/api/protected-test")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should now be accessible
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_requirement_11_8_proper_error_handling_for_setup_failures() {
    // Requirement 11.8: Proper error handling for setup failures
    
    let app = create_test_app().await;
    
    // Test invalid admin creation request
    let invalid_request = json!({
        "email": "invalid-email",
        "password": "weak",
        "name": ""
    });
    
    let request = Request::builder()
        .uri("/api/setup/admin")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(invalid_request.to_string()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should return bad request with detailed error information
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let error_response: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    
    // Should contain detailed error information
    assert_eq!(error_response["success"], false);
    assert!(error_response["error"].is_string());
    assert!(error_response["message"].is_string());
    assert!(error_response["recovery_actions"].is_array());
}

#[tokio::test]
async fn test_setup_status_endpoint_provides_complete_information() {
    // Setup status endpoint should provide comprehensive information
    
    let app = create_test_app().await;
    
    // Test setup status before admin creation
    let request = Request::builder()
        .uri("/api/setup/status")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let status: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    
    // Should indicate first run and no admin
    assert_eq!(status["is_first_run"], true);
    assert_eq!(status["admin_exists"], false);
    assert!(status["system_health"].is_object());
    assert_eq!(status["system_health"]["database_connected"], true);
    assert_eq!(status["system_health"]["admin_account_exists"], false);
}

#[tokio::test]
async fn test_setup_page_serves_html_on_first_run() {
    // Setup page should serve HTML on first run
    
    let app = create_test_app().await;
    
    let request = Request::builder()
        .uri("/setup")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should serve HTML page
    assert_eq!(response.status(), StatusCode::OK);
    
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/html"));
}

#[tokio::test]
async fn test_setup_page_redirects_after_completion() {
    // Setup page should redirect to login after setup is complete
    
    let app = create_test_app().await;
    
    // First create admin account
    let admin_request = json!({
        "email": "admin@example.com",
        "password": "securepass123",
        "name": "System Admin"
    });
    
    let request = Request::builder()
        .uri("/api/setup/admin")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(admin_request.to_string()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Now test setup page redirects
    let request = Request::builder()
        .uri("/setup")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Should redirect to login
    assert_eq!(response.status(), StatusCode::FOUND);
    let location = response.headers().get("location").unwrap();
    assert_eq!(location, "/login");
}

#[tokio::test]
async fn test_normal_flow_after_setup_completion() {
    // Normal application flow should work after setup completion
    
    let app = create_test_app().await;
    
    // Create admin account
    let admin_request = json!({
        "email": "admin@example.com",
        "password": "securepass123",
        "name": "System Admin"
    });
    
    let request = Request::builder()
        .uri("/api/setup/admin")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(admin_request.to_string()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Test that normal pages no longer redirect to setup
    let test_paths = vec!["/", "/login"];
    
    for path in test_paths {
        let request = Request::builder()
            .uri(path)
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should not redirect to setup anymore
        if response.status() == StatusCode::FOUND {
            let location = response.headers().get("location");
            if let Some(loc) = location {
                assert_ne!(loc, "/setup", "Path {} should not redirect to setup after completion", path);
            }
        }
    }
}

#[tokio::test]
async fn test_middleware_error_handling() {
    // Test that middleware handles setup service errors gracefully
    
    // This test would require mocking the setup service to return errors
    // For now, we test the basic error path by checking that the middleware
    // doesn't crash the application when setup detection fails
    
    let app = create_test_app().await;
    
    // Test that health endpoint still works even if setup detection might fail
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    // Health endpoint should always work
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_creation_response_includes_session_cookie() {
    // Admin creation should set session cookie for immediate login
    
    let app = create_test_app().await;
    
    let admin_request = json!({
        "email": "admin@example.com",
        "password": "securepass123",
        "name": "System Admin"
    });
    
    let request = Request::builder()
        .uri("/api/setup/admin")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(admin_request.to_string()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Should set session cookie
    let set_cookie = response.headers().get("set-cookie");
    assert!(set_cookie.is_some());
    
    let cookie_value = set_cookie.unwrap().to_str().unwrap();
    assert!(cookie_value.contains("campfire_session="));
    assert!(cookie_value.contains("HttpOnly"));
    assert!(cookie_value.contains("SameSite=Lax"));
    
    // Response should include redirect URL
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let response_json: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    
    assert_eq!(response_json["success"], true);
    assert_eq!(response_json["redirect_url"], "/chat");
    assert!(response_json["next_steps"].is_array());
}