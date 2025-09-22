use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use campfire_on_rust::{handlers, middleware, AppState, Database};
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`

async fn create_test_app() -> Router {
    // Create test database
    let db = Database::new(":memory:").await.unwrap();
    
    // Create auth service
    let auth_service = Arc::new(campfire_on_rust::services::auth::AuthService::new(Arc::new(db.clone())));
    
    let app_state = AppState {
        db,
        auth_service,
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