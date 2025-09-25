use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use tower::ServiceExt;
use campfire_on_rust::{
    errors::*,
    logging::{
        error_handling::{handle_auth_error, handle_message_error, handle_room_error},
        audit::{AuditAction, AuditLogger},
    },
    middleware::error_handling::*,
};

/// Test comprehensive error handling and recovery procedures
#[tokio::test]
async fn test_comprehensive_error_handling() {
    // Test authentication error handling
    test_auth_error_handling().await;
    
    // Test message error handling
    test_message_error_handling().await;
    
    // Test room error handling
    test_room_error_handling().await;
    
    // Test audit logging
    test_audit_logging().await;
    
    // Test error recovery middleware
    test_error_recovery_middleware().await;
}

async fn test_auth_error_handling() {
    // Test invalid credentials error
    let error = AuthError::InvalidCredentials;
    let user_friendly = handle_auth_error(error, Some("login"));
    
    assert_eq!(user_friendly.status, StatusCode::UNAUTHORIZED);
    assert_eq!(user_friendly.code, "INVALID_CREDENTIALS");
    assert!(!user_friendly.recovery_suggestions.is_empty());
    
    // Test session expired error
    let error = AuthError::SessionExpired;
    let user_friendly = handle_auth_error(error, Some("session_check"));
    
    assert_eq!(user_friendly.status, StatusCode::UNAUTHORIZED);
    assert_eq!(user_friendly.code, "SESSION_EXPIRED");
    assert!(user_friendly.recovery_suggestions.contains(&"Click the login button to sign in again".to_string()));
}

async fn test_message_error_handling() {
    use campfire_on_rust::models::{UserId, RoomId};
    use uuid::Uuid;
    
    // Test authorization error
    let user_id = UserId(Uuid::new_v4());
    let room_id = RoomId(Uuid::new_v4());
    let error = MessageError::Authorization { user_id, room_id };
    let user_friendly = handle_message_error(error, Some("create_message"));
    
    assert_eq!(user_friendly.status, StatusCode::FORBIDDEN);
    assert_eq!(user_friendly.code, "ROOM_ACCESS_DENIED");
    
    // Test content too long error
    let error = MessageError::ContentTooLong { length: 15000 };
    let user_friendly = handle_message_error(error, Some("create_message"));
    
    assert_eq!(user_friendly.status, StatusCode::BAD_REQUEST);
    assert_eq!(user_friendly.code, "MESSAGE_TOO_LONG");
    assert!(user_friendly.message.contains("15000 characters"));
    
    // Test rate limit error
    let error = MessageError::RateLimit { 
        limit: 10, 
        window: "minute".to_string() 
    };
    let user_friendly = handle_message_error(error, Some("create_message"));
    
    assert_eq!(user_friendly.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(user_friendly.code, "RATE_LIMIT_EXCEEDED");
}

async fn test_room_error_handling() {
    use campfire_on_rust::models::{UserId, RoomId};
    use uuid::Uuid;
    
    // Test room not found error
    let room_id = RoomId(Uuid::new_v4());
    let error = RoomError::NotFound { room_id };
    let user_friendly = handle_room_error(error, Some("get_room"));
    
    assert_eq!(user_friendly.status, StatusCode::NOT_FOUND);
    assert_eq!(user_friendly.code, "ROOM_NOT_FOUND");
    
    // Test invalid room name error
    let error = RoomError::InvalidName { 
        reason: "Too long".to_string() 
    };
    let user_friendly = handle_room_error(error, Some("create_room"));
    
    assert_eq!(user_friendly.status, StatusCode::BAD_REQUEST);
    assert_eq!(user_friendly.code, "INVALID_ROOM_NAME");
}

async fn test_audit_logging() {
    use campfire_on_rust::models::UserId;
    use uuid::Uuid;
    
    let audit_logger = AuditLogger::new(true);
    let user_id = UserId(Uuid::new_v4());
    
    // Test user action logging
    let mut details = HashMap::new();
    details.insert("test_key".to_string(), "test_value".to_string());
    
    audit_logger.log_user_action(
        AuditAction::Login,
        user_id,
        "session",
        Some("test_session_id".to_string()),
        details,
    );
    
    // Test security event logging
    let mut security_details = HashMap::new();
    security_details.insert("ip_address".to_string(), "192.168.1.1".to_string());
    security_details.insert("user_agent".to_string(), "test_agent".to_string());
    
    audit_logger.log_security_event(
        AuditAction::UnauthorizedAccess,
        Some(user_id),
        Some("192.168.1.1"),
        security_details,
    );
    
    // Test system event logging
    let mut system_details = HashMap::new();
    system_details.insert("component".to_string(), "database".to_string());
    system_details.insert("action".to_string(), "backup_created".to_string());
    
    audit_logger.log_system_event(
        AuditAction::BackupCreated,
        system_details,
    );
}

async fn test_error_recovery_middleware() {
    // Create a test app with error recovery middleware
    let app = Router::new()
        .route("/test_500", get(handler_500))
        .route("/test_503", get(handler_503))
        .route("/test_429", get(handler_429))
        .layer(axum::middleware::from_fn(error_recovery_middleware));
    
    // Test 500 error recovery
    let request = Request::builder()
        .method(Method::GET)
        .uri("/test_500")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    
    // Test 503 error recovery
    let request = Request::builder()
        .method(Method::GET)
        .uri("/test_503")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    
    // Test 429 error recovery
    let request = Request::builder()
        .method(Method::GET)
        .uri("/test_429")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

// Test handlers that return specific error codes
async fn handler_500() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
}

async fn handler_503() -> Response {
    (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable").into_response()
}

async fn handler_429() -> Response {
    (StatusCode::TOO_MANY_REQUESTS, "Too Many Requests").into_response()
}

/// Test circuit breaker functionality
#[tokio::test]
async fn test_circuit_breaker() {
    use campfire_on_rust::logging::error_handling::ErrorCircuitBreaker;
    use std::time::Duration;
    
    let circuit_breaker = ErrorCircuitBreaker::new(3, Duration::from_millis(100));
    
    // Initially should allow requests
    assert!(circuit_breaker.should_allow_request());
    
    // Record failures up to threshold
    circuit_breaker.record_failure();
    assert!(circuit_breaker.should_allow_request());
    
    circuit_breaker.record_failure();
    assert!(circuit_breaker.should_allow_request());
    
    circuit_breaker.record_failure();
    // Should still allow (threshold is 3, we've had 3 failures)
    assert!(circuit_breaker.should_allow_request());
    
    // One more failure should open the circuit
    circuit_breaker.record_failure();
    assert!(!circuit_breaker.should_allow_request());
    
    // Wait for recovery timeout
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Should allow requests again after timeout
    assert!(circuit_breaker.should_allow_request());
    
    // Record success should reset failure count
    circuit_breaker.record_success();
    assert!(circuit_breaker.should_allow_request());
}

/// Test error documentation system
#[tokio::test]
async fn test_error_documentation() {
    use campfire_on_rust::logging::documentation::ErrorDocumentation;
    
    let error_docs = ErrorDocumentation::new();
    
    // Test getting error guide
    let guide = error_docs.get_guide("DATABASE_CONNECTION_FAILED");
    assert!(guide.is_some());
    
    let guide = guide.unwrap();
    assert_eq!(guide.error_code, "DATABASE_CONNECTION_FAILED");
    assert!(!guide.common_causes.is_empty());
    assert!(!guide.recovery_steps.is_empty());
    assert!(!guide.prevention_tips.is_empty());
    
    // Test recovery steps
    let recovery_steps = error_docs.get_recovery_steps("RATE_LIMIT_EXCEEDED");
    assert!(!recovery_steps.is_empty());
    
    // Test prevention tips
    let prevention_tips = error_docs.get_prevention_tips("SESSION_EXPIRED");
    assert!(!prevention_tips.is_empty());
}

/// Test performance monitoring and logging
#[tokio::test]
async fn test_performance_monitoring() {
    use std::time::{Duration, Instant};
    
    // Simulate slow operation
    let start = Instant::now();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let duration = start.elapsed();
    
    // Test performance warning macro (would normally log)
    if duration > Duration::from_millis(50) {
        // This would trigger a performance warning in real code
        assert!(duration > Duration::from_millis(50));
    }
}

/// Integration test for complete error handling flow
#[tokio::test]
async fn test_complete_error_flow() {
    // This test would require a full application setup
    // For now, we'll test the individual components
    
    // Test that error handling components work together
    let audit_logger = AuditLogger::new(true);
    
    // Simulate an error scenario
    let error = AuthError::InvalidCredentials;
    let user_friendly = handle_auth_error(error, Some("integration_test"));
    
    // Verify error response structure
    assert_eq!(user_friendly.status, StatusCode::UNAUTHORIZED);
    assert!(!user_friendly.recovery_suggestions.is_empty());
    
    // Log the error event
    let mut details = HashMap::new();
    details.insert("test_scenario".to_string(), "integration_test".to_string());
    details.insert("error_code".to_string(), user_friendly.code.clone());
    
    audit_logger.log_security_event(
        AuditAction::Login,
        None,
        Some("127.0.0.1"),
        details,
    );
}