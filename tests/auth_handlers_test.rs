use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use campfire_on_rust::{handlers, AppState, CampfireDatabase, AuthService, RoomService, MessageService, ConnectionManagerImpl, AuthServiceTrait};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`

async fn create_test_app() -> (Router, Arc<CampfireDatabase>) {
    // Create test database
    let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
    
    // Create connection manager
    let connection_manager = Arc::new(ConnectionManagerImpl::new());
    
    // Create services
    let auth_service = Arc::new(AuthService::new(db.clone()));
    let room_service = Arc::new(RoomService::new(db.clone()));
    let message_service = Arc::new(MessageService::new(
        db.clone(),
        connection_manager,
        room_service.clone()
    ));
    
    let app_state = AppState {
        db: (*db).clone(),
        auth_service,
        room_service,
        message_service,
    };

    let router = Router::new()
        .route("/api/auth/login", axum::routing::post(handlers::auth::login))
        .route("/api/auth/logout", axum::routing::post(handlers::auth::logout))
        .route("/api/users/me", axum::routing::get(handlers::users::get_current_user))
        .with_state(app_state);
        
    (router, db)
}

#[tokio::test]
async fn test_login_success() {
    let (app, db) = create_test_app().await;
    
    // Create auth service using the same database
    let auth_service = campfire_on_rust::services::auth::AuthService::new(db);
    
    let email = "test@example.com";
    let password = "password123";
    
    auth_service.create_user(
        "Test User".to_string(),
        email.to_string(),
        password.to_string(),
    ).await.unwrap();

    let login_request = json!({
        "email": email,
        "password": password
    });

    let request = Request::builder()
        .uri("/api/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(login_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let (app, _db) = create_test_app().await;

    let login_request = json!({
        "email": "nonexistent@example.com",
        "password": "wrongpassword"
    });

    let request = Request::builder()
        .uri("/api/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(login_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_missing_credentials() {
    let (app, _db) = create_test_app().await;

    // Test with empty email
    let login_request = json!({
        "email": "",
        "password": "password123"
    });

    let request = Request::builder()
        .uri("/api/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(login_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_logout_success() {
    let (app, db) = create_test_app().await;
    
    // Create auth service using the same database
    let auth_service = campfire_on_rust::services::auth::AuthService::new(db);
    
    let user = auth_service.create_user(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    let session = auth_service.create_session(user.id).await.unwrap();

    let request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .header("Authorization", format!("Bearer {}", session.token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_logout_without_token() {
    let (app, _db) = create_test_app().await;

    let request = Request::builder()
        .uri("/api/auth/logout")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_current_user_success() {
    let (app, db) = create_test_app().await;
    
    // Create auth service using the same database
    let auth_service = campfire_on_rust::services::auth::AuthService::new(db);
    
    let user = auth_service.create_user(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    let session = auth_service.create_session(user.id).await.unwrap();

    let request = Request::builder()
        .uri("/api/users/me")
        .header("Authorization", format!("Bearer {}", session.token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_current_user_unauthorized() {
    let (app, _db) = create_test_app().await;

    let request = Request::builder()
        .uri("/api/users/me")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}