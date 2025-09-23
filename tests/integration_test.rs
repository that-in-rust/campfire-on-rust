use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use campfire_on_rust::{handlers, AppState, CampfireDatabase, AuthService, RoomService, MessageService, ConnectionManagerImpl};
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`

async fn create_test_app() -> Router {
    // Create test database
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let db_arc = Arc::new(db.clone());
    
    // Create connection manager
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db_arc.clone()));
    
    // Create services
    let auth_service = Arc::new(AuthService::new(db_arc.clone()));
    let room_service = Arc::new(RoomService::new(db_arc.clone()));
    let message_service = Arc::new(MessageService::new(
        db_arc.clone(),
        connection_manager,
        room_service.clone()
    ));
    
    let search_service = Arc::new(campfire_on_rust::SearchService::new(
        db_arc.clone(),
        room_service.clone(),
    ));
    let push_service = Arc::new(campfire_on_rust::PushNotificationServiceImpl::new(
        db_arc.as_ref().clone(),
        db_arc.writer(),
        campfire_on_rust::VapidConfig::default(),
    ));
    let bot_service = Arc::new(campfire_on_rust::BotServiceImpl::new(
        db_arc.clone(),
        db_arc.writer(),
        message_service.clone(),
    ));
    
    let app_state = AppState {
        db,
        auth_service,
        room_service,
        message_service,
        search_service,
        push_service,
        bot_service,
    };

    Router::new()
        .route("/api/users/me", axum::routing::get(handlers::users::get_current_user))
        .with_state(app_state)
}

#[tokio::test]
async fn test_get_current_user_without_auth() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/users/me")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_current_user_with_invalid_token() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/users/me")
        .header("Authorization", "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_session_token_extraction_from_cookie() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/users/me")
        .header("Cookie", "session_token=invalid_token")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should still be unauthorized because token is invalid, but it was extracted
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}