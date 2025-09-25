use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    clock::{DefaultClock, QuantaClock},
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    sync::{Arc, RwLock},
    time::Duration,
};
use tracing::{debug, warn};

use crate::logging::audit::{AuditAction, AuditLogger};

/// Rate limiter for different endpoint types
#[derive(Clone)]
pub struct RateLimitingMiddleware {
    /// General API rate limiter (per IP)
    general_limiter: Arc<RwLock<HashMap<IpAddr, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>>,
    /// Authentication endpoint rate limiter (stricter)
    auth_limiter: Arc<RwLock<HashMap<IpAddr, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>>,
    /// Bot API rate limiter (per bot token)
    bot_limiter: Arc<RwLock<HashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>>,
    /// Configuration
    config: RateLimitConfig,
    /// Audit logger for rate limit violations
    audit_logger: AuditLogger,
}

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// General API requests per minute per IP
    pub general_rpm: u32,
    /// Authentication requests per minute per IP
    pub auth_rpm: u32,
    /// Bot API requests per minute per token
    pub bot_rpm: u32,
    /// Burst allowance (requests that can be made immediately)
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            general_rpm: 60,
            auth_rpm: 10,
            bot_rpm: 100,
            burst_size: 10,
        }
    }
}

impl RateLimitingMiddleware {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            general_limiter: Arc::new(RwLock::new(HashMap::new())),
            auth_limiter: Arc::new(RwLock::new(HashMap::new())),
            bot_limiter: Arc::new(RwLock::new(HashMap::new())),
            config,
            audit_logger: AuditLogger::new(true),
        }
    }

    /// Create rate limiter for general API endpoints
    fn create_general_limiter(&self) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        let quota = Quota::per_minute(NonZeroU32::new(self.config.general_rpm).unwrap())
            .allow_burst(NonZeroU32::new(self.config.burst_size).unwrap());
        Arc::new(RateLimiter::direct(quota))
    }

    /// Create rate limiter for authentication endpoints
    fn create_auth_limiter(&self) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        let quota = Quota::per_minute(NonZeroU32::new(self.config.auth_rpm).unwrap())
            .allow_burst(NonZeroU32::new(5).unwrap()); // Lower burst for auth
        Arc::new(RateLimiter::direct(quota))
    }

    /// Create rate limiter for bot API endpoints
    fn create_bot_limiter(&self) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        let quota = Quota::per_minute(NonZeroU32::new(self.config.bot_rpm).unwrap())
            .allow_burst(NonZeroU32::new(self.config.burst_size * 2).unwrap()); // Higher burst for bots
        Arc::new(RateLimiter::direct(quota))
    }

    /// Get or create rate limiter for IP address
    fn get_ip_limiter(
        &self,
        ip: IpAddr,
        is_auth: bool,
    ) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        let limiter_map = if is_auth {
            &self.auth_limiter
        } else {
            &self.general_limiter
        };

        // Try to get existing limiter
        {
            let read_guard = limiter_map.read().unwrap();
            if let Some(limiter) = read_guard.get(&ip) {
                return Arc::clone(limiter);
            }
        }

        // Create new limiter
        let new_limiter = if is_auth {
            self.create_auth_limiter()
        } else {
            self.create_general_limiter()
        };

        // Store new limiter
        {
            let mut write_guard = limiter_map.write().unwrap();
            write_guard.insert(ip, Arc::clone(&new_limiter));
        }

        new_limiter
    }

    /// Get or create rate limiter for bot token
    fn get_bot_limiter(&self, bot_token: &str) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        // Try to get existing limiter
        {
            let read_guard = self.bot_limiter.read().unwrap();
            if let Some(limiter) = read_guard.get(bot_token) {
                return Arc::clone(limiter);
            }
        }

        // Create new limiter
        let new_limiter = self.create_bot_limiter();

        // Store new limiter
        {
            let mut write_guard = self.bot_limiter.write().unwrap();
            write_guard.insert(bot_token.to_string(), Arc::clone(&new_limiter));
        }

        new_limiter
    }

    /// Check if request should be rate limited
    pub async fn check_rate_limit(&self, request: &Request<axum::body::Body>, addr: SocketAddr) -> Result<(), RateLimitError> {
        let path = request.uri().path();
        let ip = addr.ip();

        // Determine rate limit type based on path
        let (limiter, limit_type) = if path.starts_with("/api/auth/") {
            // Authentication endpoints - stricter limits
            (self.get_ip_limiter(ip, true), "auth")
        } else if path.starts_with("/rooms/") && path.contains("/bot/") {
            // Bot API endpoints - check for bot token
            if let Some(bot_token) = extract_bot_token_from_path(path) {
                (self.get_bot_limiter(&bot_token), "bot")
            } else {
                return Err(RateLimitError::InvalidBotToken);
            }
        } else if path.starts_with("/api/") {
            // General API endpoints
            (self.get_ip_limiter(ip, false), "general")
        } else {
            // Static content and pages - no rate limiting
            return Ok(());
        };

        // Check rate limit
        match limiter.check() {
            Ok(_) => {
                debug!("Rate limit check passed for {} from {}", limit_type, ip);
                Ok(())
            }
            Err(_) => {
                warn!("Rate limit exceeded for {} from {} on path {}", limit_type, ip, path);
                
                // Log rate limit violation
                let mut details = std::collections::HashMap::new();
                details.insert("ip".to_string(), ip.to_string());
                details.insert("path".to_string(), path.to_string());
                details.insert("limit_type".to_string(), limit_type.to_string());
                
                self.audit_logger.log_security_event(
                    AuditAction::RateLimitExceeded,
                    None,
                    Some(&ip.to_string()),
                    details,
                );
                
                Err(RateLimitError::RateLimitExceeded {
                    limit_type: limit_type.to_string(),
                    retry_after: Duration::from_secs(60),
                })
            }
        }
    }

    /// Cleanup old rate limiters (call periodically)
    pub fn cleanup_old_limiters(&self) {
        // This is a simplified cleanup - in production you'd want more sophisticated cleanup
        // based on last access time
        let max_limiters = 10000;
        
        {
            let mut general = self.general_limiter.write().unwrap();
            if general.len() > max_limiters {
                general.clear();
            }
        }
        
        {
            let mut auth = self.auth_limiter.write().unwrap();
            if auth.len() > max_limiters {
                auth.clear();
            }
        }
        
        {
            let mut bot = self.bot_limiter.write().unwrap();
            if bot.len() > max_limiters {
                bot.clear();
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for {limit_type}, retry after {retry_after:?}")]
    RateLimitExceeded {
        limit_type: String,
        retry_after: Duration,
    },
    
    #[error("Invalid or missing bot token")]
    InvalidBotToken,
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        match self {
            RateLimitError::RateLimitExceeded { retry_after, .. } => {
                let mut response = (
                    StatusCode::TOO_MANY_REQUESTS,
                    axum::Json(serde_json::json!({
                        "error": "Rate limit exceeded",
                        "message": "Too many requests. Please slow down.",
                        "retry_after_seconds": retry_after.as_secs()
                    }))
                ).into_response();
                
                // Add Retry-After header
                response.headers_mut().insert(
                    "Retry-After",
                    retry_after.as_secs().to_string().parse().unwrap(),
                );
                
                response
            }
            RateLimitError::InvalidBotToken => {
                (
                    StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::json!({
                        "error": "Invalid bot token",
                        "message": "Bot token is missing or invalid"
                    }))
                ).into_response()
            }
        }
    }
}

/// Extract bot token from bot API path
/// Path format: /rooms/{room_id}/bot/{bot_token}/messages
fn extract_bot_token_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 5 && parts[3] == "bot" {
        Some(parts[4].to_string())
    } else {
        None
    }
}

/// Middleware function for rate limiting
pub async fn rate_limiting_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next<axum::body::Body>,
) -> Response {
    // Get rate limiter from request extensions (set during app setup)
    let rate_limiter = request
        .extensions()
        .get::<RateLimitingMiddleware>()
        .cloned();

    if let Some(limiter) = rate_limiter {
        if let Err(rate_limit_error) = limiter.check_rate_limit(&request, addr).await {
            return rate_limit_error.into_response();
        }
    }

    next.run(request).await
}

/// Create rate limiting layer with configuration
pub fn create_rate_limiting_layer(config: RateLimitConfig) -> tower::layer::util::Identity {
    // For now, return identity layer - rate limiting will be implemented in a simpler way
    tower::layer::util::Identity::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, Uri};
    use std::net::{IpAddr, Ipv4Addr};

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
            .body(axum::body::Body::empty())
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
            .body(axum::body::Body::empty())
            .unwrap();
        
        // First request should pass
        assert!(middleware.check_rate_limit(&request, addr).await.is_ok());
        
        // Second request should be rate limited (stricter auth limits)
        assert!(middleware.check_rate_limit(&request, addr).await.is_err());
    }

    #[test]
    fn test_extract_bot_token_from_path() {
        assert_eq!(
            extract_bot_token_from_path("/rooms/123/bot/abc123/messages"),
            Some("abc123".to_string())
        );
        
        assert_eq!(
            extract_bot_token_from_path("/api/rooms"),
            None
        );
        
        assert_eq!(
            extract_bot_token_from_path("/rooms/123/messages"),
            None
        );
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
            .body(axum::body::Body::empty())
            .unwrap();
        
        // First request should pass
        assert!(middleware.check_rate_limit(&request, addr).await.is_ok());
        
        // Second request should pass (within burst)
        assert!(middleware.check_rate_limit(&request, addr).await.is_ok());
        
        // Third request should be rate limited
        assert!(middleware.check_rate_limit(&request, addr).await.is_err());
    }
}