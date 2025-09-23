use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use campfire_on_rust::{AppState, CampfireDatabase, AuthService, RoomService, MessageService, ConnectionManagerImpl, SearchService, PushNotificationServiceImpl, VapidConfig, BotServiceImpl};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

async fn create_test_app() -> Router {
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let db_arc = Arc::new(db.clone());
    
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
        .route("/api/bots", axum::routing::get(campfire_on_rust::handlers::bot::list_bots))
        .route("/api/bots", axum::routing::post(campfire_on_rust::handlers::bot::create_bot))
        .route("/api/bots/:id", axum::routing::get(campfire_on_rust::handlers::bot::get_bot))
        .route("/api/bots/:id", axum::routing::put(campfire_on_rust::handlers::bot::update_bot))
        .route("/api/bots/:id", axum::routing::delete(campfire_on_rust::handlers::bot::delete_bot))
        .route("/api/bots/:id/reset-token", axum::routing::post(campfire_on_rust::handlers::bot::reset_bot_token))
        .route("/rooms/:room_id/bot/:bot_key/messages", axum::routing::post(campfire_on_rust::handlers::bot::create_bot_message))
        .with_state(app_state)
}

#[tokio::test]
async fn test_create_bot_without_admin() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/bots")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "name": "Test Bot",
            "webhook_url": "https://example.com/webhook"
        }).to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should fail because no session/admin auth
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_bot_message_creation_invalid_key() {
    let app = create_test_app().await;
    
    let room_id = uuid::Uuid::new_v4();
    let invalid_bot_key = "invalid-key";
    
    let request = Request::builder()
        .method("POST")
        .uri(&format!("/rooms/{}/bot/{}/messages", room_id, invalid_bot_key))
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "body": "Hello from bot!"
        }).to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should fail with unauthorized due to invalid bot key
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_bot_message_creation_empty_content() {
    let app = create_test_app().await;
    
    let room_id = uuid::Uuid::new_v4();
    let fake_bot_key = format!("{}-faketoken123", uuid::Uuid::new_v4());
    
    let request = Request::builder()
        .method("POST")
        .uri(&format!("/rooms/{}/bot/{}/messages", room_id, fake_bot_key))
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "body": ""
        }).to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should fail with bad request due to empty content
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_bot_message_creation_plain_text() {
    let app = create_test_app().await;
    
    let room_id = uuid::Uuid::new_v4();
    let fake_bot_key = format!("{}-faketoken123", uuid::Uuid::new_v4());
    
    let request = Request::builder()
        .method("POST")
        .uri(&format!("/rooms/{}/bot/{}/messages", room_id, fake_bot_key))
        .header("content-type", "text/plain")
        .body(Body::from("Hello from bot!"))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should fail with unauthorized due to invalid bot key, but content parsing should work
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_bots_without_admin() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("GET")
        .uri("/api/bots")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should fail because no session/admin auth
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_bot_without_admin() {
    let app = create_test_app().await;
    
    let bot_id = uuid::Uuid::new_v4();
    
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/api/bots/{}", bot_id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should fail because no session/admin auth
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_bot_without_admin() {
    let app = create_test_app().await;
    
    let bot_id = uuid::Uuid::new_v4();
    
    let request = Request::builder()
        .method("PUT")
        .uri(&format!("/api/bots/{}", bot_id))
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "name": "Updated Bot Name"
        }).to_string()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should fail because no session/admin auth
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_bot_without_admin() {
    let app = create_test_app().await;
    
    let bot_id = uuid::Uuid::new_v4();
    
    let request = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/bots/{}", bot_id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should fail because no session/admin auth
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_reset_bot_token_without_admin() {
    let app = create_test_app().await;
    
    let bot_id = uuid::Uuid::new_v4();
    
    let request = Request::builder()
        .method("POST")
        .uri(&format!("/api/bots/{}/reset-token", bot_id))
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should fail because no session/admin auth
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}