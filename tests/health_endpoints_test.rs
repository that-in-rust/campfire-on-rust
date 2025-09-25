use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use hyper;
use campfire_on_rust::{
    AppState, CampfireDatabase, AuthService, RoomService, MessageService, 
    ConnectionManagerImpl, SearchService, PushNotificationServiceImpl, 
    VapidConfig, BotServiceImpl, health
};
use std::sync::Arc;
use tower::ServiceExt;

async fn create_test_app() -> Router {
    // Initialize health check system
    health::init();
    
    // Initialize database
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let db_arc = Arc::new(db.clone());
    
    // Initialize connection manager
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db_arc.clone()));
    
    // Initialize services
    let auth_service = Arc::new(AuthService::new(db_arc.clone()));
    let room_service = Arc::new(RoomService::new(db_arc.clone()));
    
    // Initialize push notification service
    let vapid_config = VapidConfig::default();
    let push_service = Arc::new(PushNotificationServiceImpl::new(
        db.clone(),
        db.writer(),
        vapid_config,
    ));
    
    // Initialize message service with push notifications
    let message_service = Arc::new(MessageService::with_push_service(
        db_arc.clone(), 
        connection_manager,
        room_service.clone(),
        push_service.clone(),
    ));
    
    let search_service = Arc::new(SearchService::new(
        db_arc.clone(),
        room_service.clone()
    ));
    
    // Initialize bot service
    let bot_service = Arc::new(BotServiceImpl::new(
        db_arc.clone(),
        db.writer(),
        message_service.clone(),
    ));
    
    let setup_service = Arc::new(campfire_on_rust::SetupServiceImpl::new(db.clone()));
    
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

    Router::new()
        .route("/health", axum::routing::get(health::health_check))
        .route("/health/ready", axum::routing::get(health::readiness_check))
        .route("/health/live", axum::routing::get(health::liveness_check))
        .with_state(app_state)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let health_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Verify response structure
    assert!(health_response.get("status").is_some());
    assert!(health_response.get("timestamp").is_some());
    assert!(health_response.get("version").is_some());
    assert!(health_response.get("uptime_seconds").is_some());
    assert!(health_response.get("checks").is_some());
    
    let checks = health_response.get("checks").unwrap();
    assert!(checks.get("database").is_some());
    assert!(checks.get("memory").is_some());
    assert!(checks.get("disk_space").is_some());
}

#[tokio::test]
async fn test_readiness_endpoint() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .uri("/health/ready")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let readiness_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Verify response structure
    assert!(readiness_response.get("ready").is_some());
    assert!(readiness_response.get("timestamp").is_some());
    assert!(readiness_response.get("checks").is_some());
    
    let checks = readiness_response.get("checks").unwrap();
    assert!(checks.get("database").is_some());
    assert!(checks.get("services").is_some());
}

#[tokio::test]
async fn test_liveness_endpoint() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .uri("/health/live")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}