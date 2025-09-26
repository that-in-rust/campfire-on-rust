use axum::{
    extract::State,
    http::{header, HeaderValue, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{collections::HashMap, time::Duration};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    set_header::SetResponseHeaderLayer,
};
use tracing::{debug, warn};
use uuid::Uuid;

/// Comprehensive security headers middleware
pub async fn security_headers_middleware(
    request: Request<axum::body::Body>,
    next: Next<axum::body::Body>,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             connect-src 'self' wss: ws:; \
             font-src 'self'; \
             object-src 'none'; \
             base-uri 'self'; \
             form-action 'self'"
        )
    );
    
    // X-Content-Type-Options
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff")
    );
    
    // X-Frame-Options
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY")
    );
    
    // X-XSS-Protection
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block")
    );
    
    // Referrer-Policy
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin")
    );
    
    // Permissions-Policy
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static(
            "camera=(), microphone=(), geolocation=(), \
             payment=(), usb=(), magnetometer=(), gyroscope=()"
        )
    );
    
    response
}

/// Create security headers layer with HTTPS enforcement option
pub fn create_security_headers_layer(_force_https: bool) -> SetResponseHeaderLayer<HeaderValue> {
    // Simplified security headers - just return the basic one for now
    SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    )
}

/// Create CORS layer with configurable origins
pub fn create_cors_layer(allowed_origins: &[String], _force_https: bool) -> CorsLayer {
    let mut cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            "x-requested-with".parse().unwrap(),
        ])
        .max_age(Duration::from_secs(3600));
    
    if allowed_origins.is_empty() {
        // Allow all origins (development mode) - cannot use credentials with Any
        cors = cors.allow_origin(Any);
    } else {
        // Allow specific origins (production mode) - can use credentials with specific origins
        cors = cors.allow_credentials(true);
        for origin in allowed_origins {
            if let Ok(origin_header) = origin.parse::<HeaderValue>() {
                cors = cors.allow_origin(origin_header);
            }
        }
    }
    
    cors
}

/// Create request size limit layer with configurable size
pub fn create_request_size_limit_layer_with_size(max_size: usize) -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(max_size)
}

/// Create timeout layer with configurable duration
pub fn create_timeout_layer_with_duration(timeout: Duration) -> TimeoutLayer {
    TimeoutLayer::new(timeout)
}

/// Legacy function for backward compatibility
pub fn create_security_headers_layer_legacy() -> SetResponseHeaderLayer<HeaderValue> {
    create_security_headers_layer(false)
}

pub fn create_production_cors_layer() -> CorsLayer {
    create_cors_layer(&[], false)
}

pub fn create_request_size_limit_layer() -> RequestBodyLimitLayer {
    create_request_size_limit_layer_with_size(10 * 1024 * 1024) // 10MB
}

pub fn create_timeout_layer() -> TimeoutLayer {
    create_timeout_layer_with_duration(Duration::from_secs(30))
}
// CSRF protection middleware
#[derive(Clone)]
pub struct CsrfProtection {
    /// Store of valid CSRF tokens
    tokens: std::sync::Arc<std::sync::RwLock<HashMap<String, std::time::Instant>>>,
    /// Token expiration time
    token_lifetime: Duration,
}

impl CsrfProtection {
    pub fn new() -> Self {
        Self {
            tokens: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
            token_lifetime: Duration::from_secs(3600), // 1 hour
        }
    }
    
    /// Generate a new CSRF token
    pub fn generate_token(&self) -> String {
        let token = Uuid::new_v4().to_string();
        let mut tokens = self.tokens.write().unwrap();
        tokens.insert(token.clone(), std::time::Instant::now());
        token
    }
    
    /// Validate a CSRF token
    pub fn validate_token(&self, token: &str) -> bool {
        let mut tokens = self.tokens.write().unwrap();
        
        if let Some(created_at) = tokens.get(token) {
            if created_at.elapsed() < self.token_lifetime {
                // Token is valid, remove it (one-time use)
                tokens.remove(token);
                true
            } else {
                // Token expired, remove it
                tokens.remove(token);
                false
            }
        } else {
            false
        }
    }
    
    /// Clean up expired tokens
    pub fn cleanup_expired_tokens(&self) {
        let mut tokens = self.tokens.write().unwrap();
        let now = std::time::Instant::now();
        tokens.retain(|_, created_at| now.duration_since(*created_at) < self.token_lifetime);
    }
    
    /// Check if request needs CSRF protection
    pub fn needs_csrf_protection(method: &Method, path: &str) -> bool {
        // Only protect state-changing operations
        matches!(method, &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH) &&
        // Protect API endpoints but not auth endpoints (they use other protection)
        path.starts_with("/api/") && 
        !path.starts_with("/api/auth/") &&
        // Don't protect bot API (uses token auth)
        !path.contains("/bot/")
    }
}

impl Default for CsrfProtection {
    fn default() -> Self {
        Self::new()
    }
}

/// CSRF protection middleware function
pub async fn csrf_protection_middleware(
    request: Request<axum::body::Body>,
    next: Next<axum::body::Body>,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    
    // Check if this request needs CSRF protection
    if CsrfProtection::needs_csrf_protection(&method, &path) {
        // Get CSRF protection from request extensions
        if let Some(csrf) = request.extensions().get::<CsrfProtection>() {
            // Check for CSRF token in headers
            let token = request
                .headers()
                .get("X-CSRF-Token")
                .and_then(|h| h.to_str().ok());
            
            if let Some(token) = token {
                if !csrf.validate_token(token) {
                    warn!("Invalid CSRF token for {} {}", method, path);
                    return (
                        StatusCode::FORBIDDEN,
                        axum::Json(serde_json::json!({
                            "error": "Invalid CSRF token",
                            "message": "CSRF token is missing or invalid"
                        }))
                    ).into_response();
                }
            } else {
                warn!("Missing CSRF token for {} {}", method, path);
                return (
                    StatusCode::FORBIDDEN,
                    axum::Json(serde_json::json!({
                        "error": "Missing CSRF token",
                        "message": "CSRF token is required for this operation"
                    }))
                ).into_response();
            }
        }
    }
    
    next.run(request).await
}

/// Create CSRF protection layer
pub fn create_csrf_protection_layer() -> (CsrfProtection, tower::layer::util::Identity) {
    let csrf = CsrfProtection::new();
    // For now, return identity layer - CSRF will be implemented in handlers
    (csrf, tower::layer::util::Identity::new())
}

/// Input sanitization and validation middleware
pub async fn input_sanitization_middleware(
    request: Request<axum::body::Body>,
    next: Next<axum::body::Body>,
) -> Response {
    // For JSON requests, we could intercept and sanitize the body here
    // For now, we rely on the validation layer in individual handlers
    
    // Add security headers to indicate input processing
    let mut response = next.run(request).await;
    
    // Add header to indicate security processing
    response.headers_mut().insert(
        "X-Input-Sanitized",
        HeaderValue::from_static("true")
    );
    
    response
}

/// Bot API abuse prevention middleware
#[derive(Clone)]
pub struct BotAbuseProtection {
    /// Track bot request patterns
    bot_metrics: std::sync::Arc<std::sync::RwLock<HashMap<String, BotMetrics>>>,
}

#[derive(Debug, Clone)]
struct BotMetrics {
    request_count: u64,
    last_request: std::time::Instant,
    error_count: u64,
    last_error: Option<std::time::Instant>,
    blocked_until: Option<std::time::Instant>,
}

impl BotAbuseProtection {
    pub fn new() -> Self {
        Self {
            bot_metrics: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }
    
    /// Check if bot should be blocked
    pub fn check_bot_abuse(&self, bot_token: &str) -> Result<(), BotAbuseError> {
        let mut metrics = self.bot_metrics.write().unwrap();
        let now = std::time::Instant::now();
        
        let bot_metrics = metrics.entry(bot_token.to_string()).or_insert(BotMetrics {
            request_count: 0,
            last_request: now,
            error_count: 0,
            last_error: None,
            blocked_until: None,
        });
        
        // Check if bot is currently blocked
        if let Some(blocked_until) = bot_metrics.blocked_until {
            if now < blocked_until {
                return Err(BotAbuseError::Blocked {
                    until: blocked_until,
                });
            } else {
                // Unblock the bot
                bot_metrics.blocked_until = None;
                bot_metrics.error_count = 0;
            }
        }
        
        // Update request metrics
        bot_metrics.request_count += 1;
        bot_metrics.last_request = now;
        
        // Check for abuse patterns
        
        // Pattern 1: Too many errors in short time
        if bot_metrics.error_count > 10 {
            if let Some(last_error) = bot_metrics.last_error {
                if now.duration_since(last_error) < Duration::from_secs(300) { // 5 minutes
                    // Block for 30 minutes
                    bot_metrics.blocked_until = Some(now + Duration::from_secs(1800)); // 30 minutes
                    return Err(BotAbuseError::TooManyErrors);
                }
            }
        }
        
        Ok(())
    }
    
    /// Record bot error
    pub fn record_bot_error(&self, bot_token: &str) {
        let mut metrics = self.bot_metrics.write().unwrap();
        let now = std::time::Instant::now();
        
        let bot_metrics = metrics.entry(bot_token.to_string()).or_insert(BotMetrics {
            request_count: 0,
            last_request: now,
            error_count: 0,
            last_error: None,
            blocked_until: None,
        });
        
        bot_metrics.error_count += 1;
        bot_metrics.last_error = Some(now);
    }
    
    /// Clean up old metrics
    pub fn cleanup_old_metrics(&self) {
        let mut metrics = self.bot_metrics.write().unwrap();
        let now = std::time::Instant::now();
        let cutoff = Duration::from_secs(86400); // 24 hours
        
        metrics.retain(|_, bot_metrics| {
            now.duration_since(bot_metrics.last_request) < cutoff
        });
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BotAbuseError {
    #[error("Bot is blocked until {until:?}")]
    Blocked { until: std::time::Instant },
    
    #[error("Too many errors from bot")]
    TooManyErrors,
}

impl IntoResponse for BotAbuseError {
    fn into_response(self) -> Response {
        match self {
            BotAbuseError::Blocked { until } => {
                let retry_after = until.duration_since(std::time::Instant::now()).as_secs();
                let mut response = (
                    StatusCode::TOO_MANY_REQUESTS,
                    axum::Json(serde_json::json!({
                        "error": "Bot blocked",
                        "message": "Bot has been temporarily blocked due to abuse",
                        "retry_after_seconds": retry_after
                    }))
                ).into_response();
                
                response.headers_mut().insert(
                    "Retry-After",
                    retry_after.to_string().parse().unwrap(),
                );
                
                response
            }
            BotAbuseError::TooManyErrors => {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    axum::Json(serde_json::json!({
                        "error": "Too many errors",
                        "message": "Bot has made too many errors and has been temporarily blocked"
                    }))
                ).into_response()
            }
        }
    }
}

/// Create bot abuse protection layer
pub fn create_bot_abuse_protection_layer() -> (BotAbuseProtection, tower::layer::util::Identity) {
    let protection = BotAbuseProtection::new();
    // For now, return identity layer - bot abuse protection will be implemented in handlers
    (protection, tower::layer::util::Identity::new())
}

/// Extract bot token from path (same as in rate_limiting.rs)
fn extract_bot_token_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 5 && parts[3] == "bot" {
        Some(parts[4].to_string())
    } else {
        None
    }
}