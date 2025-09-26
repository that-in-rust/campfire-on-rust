use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    Router,
};
use campfire_on_rust::{
    middleware::{
        rate_limiting::{RateLimitConfig, RateLimitingMiddleware},
        security::{CsrfProtection, BotAbuseProtection},
    },
    validation::sanitization,
    AppState,
    database::CampfireDatabase,
    services::{
        auth::AuthService,
        message::MessageService,
        room::RoomService,
        connection::ConnectionManagerImpl,
        search::SearchService,
        push::PushNotificationService,
        bot::BotService,
    },
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tower::ServiceExt;
use uuid::Uuid;

/// Comprehensive security audit test suite
/// 
/// This module tests all security boundaries and validates protection
/// against common web vulnerabilities as specified in Task 36.

#[tokio::test]
async fn test_authentication_boundary_validation() {
    let app_state = create_test_app_state().await;
    let app = create_app(app_state);
    
    // Test 1: Unauthenticated access to protected endpoints
    let protected_endpoints = vec![
        ("/api/rooms", Method::GET),
        ("/api/rooms", Method::POST),
        ("/api/messages", Method::POST),
        ("/api/users/me", Method::GET),
    ];
    
    for (path, method) in protected_endpoints {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Endpoint {} {} should require authentication",
            method,
            path
        );
    }
    
    // Test 2: Invalid token access
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/rooms")
        .header(header::AUTHORIZATION, "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    // Test 3: Expired token access (would need to create expired token)
    // This would be tested in integration tests with actual database
}

#[tokio::test]
async fn test_authorization_boundary_validation() {
    let app_state = create_test_app_state().await;
    
    // Create two users
    let user1 = app_state.auth_service.create_user(
        "User One".to_string(),
        "user1@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    let user2 = app_state.auth_service.create_user(
        "User Two".to_string(),
        "user2@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    // Create sessions for both users
    let session1 = app_state.auth_service.create_session(user1.id).await.unwrap();
    let session2 = app_state.auth_service.create_session(user2.id).await.unwrap();
    
    // Create a private room for user1
    let room = app_state.room_service.create_room(
        "Private Room".to_string(),
        None,
        campfire_on_rust::models::RoomType::Closed,
        user1.id,
    ).await.unwrap();
    
    let app = create_app(app_state);
    
    // Test: User2 should not be able to access User1's private room
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/rooms/{}/messages", room.id.0))
        .header(header::AUTHORIZATION, format!("Bearer {}", session2.token))
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "User should not access rooms they're not a member of"
    );
}

#[tokio::test]
async fn test_xss_protection() {
    // Test XSS prevention in input sanitization
    let xss_payloads = vec![
        "<script>alert('xss')</script>",
        "javascript:alert('xss')",
        "<img src=x onerror=alert('xss')>",
        "<svg onload=alert('xss')>",
        "data:text/html,<script>alert('xss')</script>",
        "<iframe src=javascript:alert('xss')></iframe>",
        "<object data=javascript:alert('xss')></object>",
        "<embed src=javascript:alert('xss')>",
        "<link rel=stylesheet href=javascript:alert('xss')>",
        "<style>@import 'javascript:alert(\"xss\")';</style>",
    ];
    
    for payload in xss_payloads {
        // Test message content sanitization
        let sanitized = sanitization::sanitize_message_content(payload);
        assert!(
            !sanitized.contains("<script>") && 
            !sanitized.contains("javascript:") &&
            !sanitized.contains("onerror=") &&
            !sanitized.contains("onload="),
            "XSS payload not properly sanitized: {} -> {}",
            payload,
            sanitized
        );
        
        // Test comprehensive input validation
        let result = sanitization::validate_and_sanitize_input(payload, 1000);
        assert!(
            result.is_err(),
            "XSS payload should be rejected: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_csrf_protection() {
    let csrf = CsrfProtection::new();
    
    // Test CSRF token generation and validation
    let token = csrf.generate_token();
    assert!(!token.is_empty());
    assert_eq!(token.len(), 36); // UUID length
    
    // Token should validate once
    assert!(csrf.validate_token(&token));
    
    // Same token should not validate again (one-time use)
    assert!(!csrf.validate_token(&token));
    
    // Invalid tokens should not validate
    assert!(!csrf.validate_token("invalid-token"));
    assert!(!csrf.validate_token(""));
    
    // Test CSRF protection requirements
    assert!(CsrfProtection::needs_csrf_protection(&Method::POST, "/api/rooms"));
    assert!(CsrfProtection::needs_csrf_protection(&Method::PUT, "/api/rooms/123"));
    assert!(CsrfProtection::needs_csrf_protection(&Method::DELETE, "/api/rooms/123"));
    
    // Auth endpoints should not need CSRF (different protection)
    assert!(!CsrfProtection::needs_csrf_protection(&Method::POST, "/api/auth/login"));
    
    // Bot endpoints should not need CSRF (token auth)
    assert!(!CsrfProtection::needs_csrf_protection(&Method::POST, "/rooms/123/bot/token/messages"));
    
    // GET requests should not need CSRF
    assert!(!CsrfProtection::needs_csrf_protection(&Method::GET, "/api/rooms"));
}

#[tokio::test]
async fn test_sql_injection_protection() {
    // Test SQL injection prevention in input validation
    let sql_injection_payloads = vec![
        "'; DROP TABLE users; --",
        "' OR '1'='1",
        "' UNION SELECT * FROM users --",
        "'; INSERT INTO users VALUES ('hacker', 'hack@evil.com'); --",
        "' OR 1=1 --",
        "admin'--",
        "admin'/*",
        "' OR 'x'='x",
        "'; EXEC xp_cmdshell('dir'); --",
        "' AND (SELECT COUNT(*) FROM users) > 0 --",
    ];
    
    for payload in sql_injection_payloads {
        let result = sanitization::validate_and_sanitize_input(payload, 1000);
        assert!(
            result.is_err(),
            "SQL injection payload should be rejected: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_path_traversal_protection() {
    // Test path traversal prevention
    let path_traversal_payloads = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
        "....//....//....//etc/passwd",
        "..%2f..%2f..%2fetc%2fpasswd",
        "..%252f..%252f..%252fetc%252fpasswd",
        "..\\..\\..\\etc\\passwd",
        "....\\\\....\\\\....\\\\etc\\passwd",
    ];
    
    for payload in path_traversal_payloads {
        let result = sanitization::validate_and_sanitize_input(payload, 1000);
        assert!(
            result.is_err(),
            "Path traversal payload should be rejected: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_rate_limiting_security() {
    let config = RateLimitConfig {
        general_rpm: 5,
        auth_rpm: 2,
        bot_rpm: 10,
        burst_size: 2,
    };
    
    let middleware = RateLimitingMiddleware::new(config);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    
    // Test general API rate limiting
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/rooms")
        .body(Body::empty())
        .unwrap();
    
    // First few requests should pass (within burst)
    assert!(middleware.check_rate_limit(&request, addr).await.is_ok());
    assert!(middleware.check_rate_limit(&request, addr).await.is_ok());
    
    // Subsequent requests should be rate limited
    assert!(middleware.check_rate_limit(&request, addr).await.is_err());
    
    // Test auth endpoint rate limiting (stricter)
    let auth_request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .body(Body::empty())
        .unwrap();
    
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)), 8080);
    
    // Auth endpoints have stricter limits
    assert!(middleware.check_rate_limit(&auth_request, addr2).await.is_ok());
    assert!(middleware.check_rate_limit(&auth_request, addr2).await.is_ok());
    assert!(middleware.check_rate_limit(&auth_request, addr2).await.is_err());
}

#[tokio::test]
async fn test_bot_api_security() {
    let protection = BotAbuseProtection::new();
    let bot_token = "test-bot-token";
    
    // Initial requests should pass
    assert!(protection.check_bot_abuse(bot_token).is_ok());
    
    // Record multiple errors to trigger abuse protection
    for _ in 0..15 {
        protection.record_bot_error(bot_token);
    }
    
    // Bot should be blocked after too many errors
    assert!(protection.check_bot_abuse(bot_token).is_err());
    
    // Test bot token validation
    assert!(sanitization::validate_bot_token("valid-bot-token-123").is_ok());
    assert!(sanitization::validate_bot_token("invalid@token").is_err());
    assert!(sanitization::validate_bot_token("short").is_err());
    
    let long_token = "a".repeat(101);
    assert!(sanitization::validate_bot_token(&long_token).is_err());
}

#[tokio::test]
async fn test_session_security() {
    let app_state = create_test_app_state().await;
    
    // Test secure token generation
    let user = app_state.auth_service.create_user(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    // Generate multiple sessions to test token uniqueness
    let mut tokens = Vec::new();
    for _ in 0..10 {
        let session = app_state.auth_service.create_session(user.id).await.unwrap();
        tokens.push(session.token);
    }
    
    // All tokens should be unique
    for i in 0..tokens.len() {
        for j in (i + 1)..tokens.len() {
            assert_ne!(
                tokens[i], tokens[j],
                "Session tokens should be unique"
            );
        }
    }
    
    // All tokens should have sufficient entropy
    for token in &tokens {
        assert!(
            token.len() >= 32,
            "Session token should have sufficient length: {}",
            token.len()
        );
        
        // Should be URL-safe (no special characters)
        assert!(
            !token.contains('+') && !token.contains('/') && !token.contains('='),
            "Session token should be URL-safe: {}",
            token
        );
    }
}

#[tokio::test]
async fn test_input_validation_edge_cases() {
    // Test null byte injection
    let null_byte_input = "test\0malicious";
    let result = sanitization::validate_and_sanitize_input(null_byte_input, 1000);
    assert!(result.is_err(), "Null bytes should be rejected");
    
    // Test extremely long input
    let long_input = "a".repeat(100000);
    let result = sanitization::validate_and_sanitize_input(&long_input, 1000);
    assert!(result.is_err(), "Overly long input should be rejected");
    
    // Test Unicode normalization attacks
    let unicode_attack = "test\u{202e}gnol_yrev_si_siht";
    let sanitized = sanitization::sanitize_user_input(unicode_attack);
    assert!(!sanitized.contains('\u{202e}'), "Unicode control characters should be removed");
    
    // Test email validation edge cases
    let invalid_emails = vec![
        "plainaddress",
        "@missingdomain.com",
        "missing@.com",
        "missing@domain",
        "spaces @domain.com",
        "test@domain..com",
        "test@.domain.com",
    ];
    
    for email in invalid_emails {
        let sanitized = sanitization::sanitize_email(email);
        // Should not crash and should produce safe output
        assert!(!sanitized.contains('<'));
        assert!(!sanitized.contains('>'));
    }
}

#[tokio::test]
async fn test_security_headers() {
    let app_state = create_test_app_state().await;
    let app = create_app(app_state);
    
    // Test that security headers are present
    let request = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Check for security headers
    let headers = response.headers();
    
    // Content Security Policy should be present
    assert!(headers.contains_key("Content-Security-Policy"));
    
    // X-Content-Type-Options should be nosniff
    assert_eq!(
        headers.get("X-Content-Type-Options").unwrap(),
        "nosniff"
    );
    
    // X-Frame-Options should be DENY
    assert_eq!(
        headers.get("X-Frame-Options").unwrap(),
        "DENY"
    );
    
    // X-XSS-Protection should be enabled
    assert_eq!(
        headers.get("X-XSS-Protection").unwrap(),
        "1; mode=block"
    );
}

#[tokio::test]
async fn test_password_security() {
    let app_state = create_test_app_state().await;
    
    // Test password strength requirements
    let weak_passwords = vec![
        "",
        "1",
        "12",
        "123",
        "1234",
        "12345",
        "123456",
        "1234567", // Less than 8 characters
    ];
    
    for password in weak_passwords {
        let result = app_state.auth_service.create_user(
            "Test User".to_string(),
            format!("test{}@example.com", password.len()),
            password.to_string(),
        ).await;
        
        assert!(
            result.is_err(),
            "Weak password should be rejected: '{}'",
            password
        );
    }
    
    // Test that passwords are properly hashed
    let user = app_state.auth_service.create_user(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "strongpassword123".to_string(),
    ).await.unwrap();
    
    // Password hash should not contain the original password
    assert!(!user.password_hash.contains("strongpassword123"));
    
    // Password hash should be bcrypt format
    assert!(user.password_hash.starts_with("$2"));
}

#[tokio::test]
async fn test_url_validation_security() {
    // Test URL sanitization for webhook URLs and other URL inputs
    let malicious_urls = vec![
        "javascript:alert('xss')",
        "data:text/html,<script>alert('xss')</script>",
        "vbscript:msgbox('xss')",
        "file:///etc/passwd",
        "ftp://malicious.com/",
        "ldap://malicious.com/",
        "gopher://malicious.com/",
    ];
    
    for url in malicious_urls {
        let result = sanitization::sanitize_url(url);
        assert!(
            result.is_err(),
            "Malicious URL should be rejected: {}",
            url
        );
    }
    
    // Valid URLs should pass
    let valid_urls = vec![
        "https://example.com",
        "http://example.com/path",
        "https://api.example.com/webhook",
    ];
    
    for url in valid_urls {
        let result = sanitization::sanitize_url(url);
        assert!(
            result.is_ok(),
            "Valid URL should be accepted: {}",
            url
        );
    }
}

#[tokio::test]
async fn test_content_type_validation() {
    // Test that the application properly validates content types
    // and rejects potentially dangerous content
    
    let app_state = create_test_app_state().await;
    let app = create_app(app_state);
    
    // Test with malicious content type
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from(r#"{"email":"test@example.com","password":"test123"}"#))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    // Should reject non-JSON content type for JSON endpoints
    assert_ne!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_timing_attack_resistance() {
    let app_state = create_test_app_state().await;
    
    // Create a user for testing
    app_state.auth_service.create_user(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    // Test that authentication timing is consistent for valid vs invalid users
    // This is a basic test - in practice you'd need more sophisticated timing analysis
    
    let start = std::time::Instant::now();
    let _ = app_state.auth_service.authenticate(
        "test@example.com".to_string(),
        "wrongpassword".to_string(),
    ).await;
    let valid_user_time = start.elapsed();
    
    let start = std::time::Instant::now();
    let _ = app_state.auth_service.authenticate(
        "nonexistent@example.com".to_string(),
        "wrongpassword".to_string(),
    ).await;
    let invalid_user_time = start.elapsed();
    
    // Times should be reasonably similar (within an order of magnitude)
    // This is a basic check - real timing attack prevention requires more sophisticated measures
    let ratio = if valid_user_time > invalid_user_time {
        valid_user_time.as_nanos() as f64 / invalid_user_time.as_nanos() as f64
    } else {
        invalid_user_time.as_nanos() as f64 / valid_user_time.as_nanos() as f64
    };
    
    assert!(
        ratio < 10.0,
        "Authentication timing difference too large: {:?} vs {:?}",
        valid_user_time,
        invalid_user_time
    );
}

// Helper function to create test app state
async fn create_test_app_state() -> AppState {
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    
    let auth_service = Arc::new(campfire_on_rust::AuthService::new(Arc::new(db.clone())));
    let room_service = Arc::new(campfire_on_rust::RoomService::new(Arc::new(db.clone())));
    let connection_manager = Arc::new(campfire_on_rust::ConnectionManagerImpl::new(Arc::new(db.clone())));
    let message_service = Arc::new(campfire_on_rust::MessageService::new(
        Arc::new(db.clone()),
        connection_manager.clone(),
        room_service.clone(),
    ));
    let search_service = Arc::new(campfire_on_rust::SearchService::new(
        Arc::new(db.clone()),
        room_service.clone(),
    ));
    
    let vapid_config = campfire_on_rust::VapidConfig {
        public_key: "test_public_key".to_string(),
        private_key: "test_private_key".to_string(),
        subject: "mailto:test@example.com".to_string(),
    };
    let push_service = Arc::new(campfire_on_rust::PushNotificationServiceImpl::new(
        db.clone(),
        db.writer(),
        vapid_config,
    ));
    let bot_service = Arc::new(campfire_on_rust::BotServiceImpl::new(
        Arc::new(db.clone()),
        db.writer(),
        message_service.clone(),
    ));
    let setup_service = Arc::new(campfire_on_rust::SetupServiceImpl::new(db.clone()));
    let demo_service = Arc::new(campfire_on_rust::DemoServiceImpl::new(Arc::new(db.clone())));
    
    AppState {
        db,
        auth_service,
        room_service,
        message_service,
        search_service,
        push_service,
        bot_service,
        setup_service,
        demo_service,
    }
}