use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
};
use campfire_on_rust::{
    middleware::{
        rate_limiting::{RateLimitConfig, RateLimitingMiddleware},
        security::{CsrfProtection, BotAbuseProtection},
    },
    validation::sanitization,
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use tower::ServiceExt;

#[tokio::test]
async fn test_rate_limiting_general_api() {
    let config = RateLimitConfig {
        general_rpm: 2, // Very low for testing
        auth_rpm: 1,
        bot_rpm: 3,
        burst_size: 1,
    };
    
    let middleware = RateLimitingMiddleware::new(config);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    
    // Create test request
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/rooms")
        .body(Body::empty())
        .unwrap();
    
    // First request should pass
    assert!(middleware.check_rate_limit(&request, addr).await.is_ok());
    
    // Second request should pass (within burst)
    assert!(middleware.check_rate_limit(&request, addr).await.is_ok());
    
    // Third request should be rate limited
    assert!(middleware.check_rate_limit(&request, addr).await.is_err());
}

#[tokio::test]
async fn test_rate_limiting_auth_endpoints() {
    let config = RateLimitConfig {
        general_rpm: 10,
        auth_rpm: 1, // Very strict for testing
        bot_rpm: 10,
        burst_size: 1,
    };
    
    let middleware = RateLimitingMiddleware::new(config);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    
    // Create auth request
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .body(Body::empty())
        .unwrap();
    
    // First request should pass
    assert!(middleware.check_rate_limit(&request, addr).await.is_ok());
    
    // Second request should be rate limited (stricter auth limits)
    assert!(middleware.check_rate_limit(&request, addr).await.is_err());
}

#[tokio::test]
async fn test_bot_api_rate_limiting() {
    let config = RateLimitConfig {
        general_rpm: 10,
        auth_rpm: 5,
        bot_rpm: 2, // Low for testing
        burst_size: 1,
    };
    
    let middleware = RateLimitingMiddleware::new(config);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    
    // Create bot API request
    let request = Request::builder()
        .method(Method::POST)
        .uri("/rooms/123/bot/test-token/messages")
        .body(Body::empty())
        .unwrap();
    
    // First request should pass
    assert!(middleware.check_rate_limit(&request, addr).await.is_ok());
    
    // Second request should pass (within burst)
    assert!(middleware.check_rate_limit(&request, addr).await.is_ok());
    
    // Third request should be rate limited
    assert!(middleware.check_rate_limit(&request, addr).await.is_err());
}

#[test]
fn test_csrf_token_generation_and_validation() {
    let csrf = CsrfProtection::new();
    
    // Generate token
    let token = csrf.generate_token();
    assert!(!token.is_empty());
    assert_eq!(token.len(), 36); // UUID length with hyphens
    
    // Validate token (should work once)
    assert!(csrf.validate_token(&token));
    
    // Validate same token again (should fail - one-time use)
    assert!(!csrf.validate_token(&token));
    
    // Validate invalid token
    assert!(!csrf.validate_token("invalid-token"));
}

#[test]
fn test_bot_abuse_protection() {
    let protection = BotAbuseProtection::new();
    let bot_token = "test-bot-token";
    
    // Initial check should pass
    assert!(protection.check_bot_abuse(bot_token).is_ok());
    
    // Record multiple errors
    for _ in 0..15 {
        protection.record_bot_error(bot_token);
    }
    
    // Should be blocked after too many errors
    assert!(protection.check_bot_abuse(bot_token).is_err());
}

#[test]
fn test_input_sanitization() {
    // Test basic HTML sanitization
    let malicious_input = "<script>alert('xss')</script><b>Bold text</b>";
    let sanitized = sanitization::sanitize_message_content(malicious_input);
    assert!(!sanitized.contains("<script>"));
    assert!(sanitized.contains("<b>Bold text</b>"));
    
    // Test plain text sanitization
    let html_input = "<b>Bold</b> text";
    let plain = sanitization::sanitize_plain_text(html_input);
    assert_eq!(plain, "Bold text");
    
    // Test comprehensive input validation
    let dangerous_input = "SELECT * FROM users; DROP TABLE users;";
    let result = sanitization::validate_and_sanitize_input(dangerous_input, 1000);
    assert!(result.is_err());
    
    // Test XSS prevention
    let xss_input = "javascript:alert('xss')";
    let result = sanitization::validate_and_sanitize_input(xss_input, 1000);
    assert!(result.is_err());
    
    // Test path traversal prevention
    let path_traversal = "../../../etc/passwd";
    let result = sanitization::validate_and_sanitize_input(path_traversal, 1000);
    assert!(result.is_err());
    
    // Test valid input
    let valid_input = "Hello, world! This is a normal message.";
    let result = sanitization::validate_and_sanitize_input(valid_input, 1000);
    assert!(result.is_ok());
}

#[test]
fn test_bot_token_validation() {
    // Valid bot token
    assert!(sanitization::validate_bot_token("valid-bot-token-123").is_ok());
    
    // Invalid characters
    assert!(sanitization::validate_bot_token("invalid@token").is_err());
    
    // Too short
    assert!(sanitization::validate_bot_token("short").is_err());
    
    // Too long
    let long_token = "a".repeat(101);
    assert!(sanitization::validate_bot_token(&long_token).is_err());
}

#[test]
fn test_url_sanitization() {
    // Valid URLs
    assert!(sanitization::sanitize_url("https://example.com").is_ok());
    assert!(sanitization::sanitize_url("http://example.com/path").is_ok());
    
    // Invalid protocols
    assert!(sanitization::sanitize_url("javascript:alert('xss')").is_err());
    assert!(sanitization::sanitize_url("data:text/html,<script>alert('xss')</script>").is_err());
    
    // Missing protocol
    assert!(sanitization::sanitize_url("example.com").is_err());
}

#[test]
fn test_email_sanitization() {
    let email = "  Test@Example.COM  ";
    let sanitized = sanitization::sanitize_email(email);
    assert_eq!(sanitized, "test@example.com");
    
    let html_email = "<script>alert('xss')</script>test@example.com";
    let sanitized = sanitization::sanitize_email(html_email);
    assert_eq!(sanitized, "test@example.com");
}

#[tokio::test]
async fn test_security_headers_middleware() {
    use axum::{
        middleware,
        response::Response,
        routing::get,
        Router,
    };
    use campfire_on_rust::middleware::security::security_headers_middleware;
    use tower::ServiceExt;
    
    async fn test_handler() -> &'static str {
        "Hello, World!"
    }
    
    let app = Router::new()
        .route("/", get(test_handler))
        .layer(middleware::from_fn(security_headers_middleware));
    
    let request = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Check that security headers are present
    assert!(response.headers().contains_key("Content-Security-Policy"));
    assert!(response.headers().contains_key("X-Content-Type-Options"));
    assert!(response.headers().contains_key("X-Frame-Options"));
    assert!(response.headers().contains_key("X-XSS-Protection"));
    assert!(response.headers().contains_key("Referrer-Policy"));
    assert!(response.headers().contains_key("Permissions-Policy"));
    
    // Check specific header values
    assert_eq!(
        response.headers().get("X-Content-Type-Options").unwrap(),
        "nosniff"
    );
    assert_eq!(
        response.headers().get("X-Frame-Options").unwrap(),
        "DENY"
    );
}

#[test]
fn test_csrf_needs_protection() {
    use campfire_on_rust::middleware::security::CsrfProtection;
    use axum::http::Method;
    
    // POST requests to API should need protection
    assert!(CsrfProtection::needs_csrf_protection(&Method::POST, "/api/rooms"));
    
    // Auth endpoints should not need CSRF (they have other protection)
    assert!(!CsrfProtection::needs_csrf_protection(&Method::POST, "/api/auth/login"));
    
    // Bot endpoints should not need CSRF (they use token auth)
    assert!(!CsrfProtection::needs_csrf_protection(&Method::POST, "/rooms/123/bot/token/messages"));
    
    // GET requests should not need protection
    assert!(!CsrfProtection::needs_csrf_protection(&Method::GET, "/api/rooms"));
    
    // Non-API endpoints should not need protection
    assert!(!CsrfProtection::needs_csrf_protection(&Method::POST, "/login"));
}

#[tokio::test]
async fn test_rate_limiting_cleanup() {
    let config = RateLimitConfig::default();
    let middleware = RateLimitingMiddleware::new(config);
    
    // Add some limiters
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)), 8080);
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/rooms")
        .body(Body::empty())
        .unwrap();
    
    // Make requests to create limiters
    let _ = middleware.check_rate_limit(&request, addr1).await;
    let _ = middleware.check_rate_limit(&request, addr2).await;
    
    // Cleanup should not panic
    middleware.cleanup_old_limiters();
}

#[test]
fn test_bot_abuse_protection_cleanup() {
    let protection = BotAbuseProtection::new();
    
    // Add some bot metrics
    protection.record_bot_error("bot1");
    protection.record_bot_error("bot2");
    
    // Cleanup should not panic
    protection.cleanup_old_metrics();
}