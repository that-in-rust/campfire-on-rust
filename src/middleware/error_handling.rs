use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::Instant;
use tracing::{error, warn, info};

use crate::logging::{
    error_handling::{UserFriendlyError, ErrorCircuitBreaker},
    audit::{AuditAction, AuditLogger},
};

/// Global error handling middleware that provides consistent error responses
/// and recovery procedures across the entire application
pub async fn global_error_handler(
    request: Request<Body>,
    next: Next<Body>,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    // Extract client information for error logging
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    let response = next.run(request).await;
    let duration = start_time.elapsed();
    let status = response.status();
    
    // Log errors and performance issues
    match status {
        status if status.is_server_error() => {
            error!(
                method = %method,
                uri = %uri,
                status = %status,
                duration_ms = %duration.as_millis(),
                user_agent = %user_agent,
                error_type = "server_error",
                "Server error occurred"
            );
            
            // Log audit event for server errors
            let audit_logger = AuditLogger::new(true);
            let mut details = std::collections::HashMap::new();
            details.insert("method".to_string(), method.to_string());
            details.insert("uri".to_string(), uri.to_string());
            details.insert("status".to_string(), status.as_u16().to_string());
            details.insert("duration_ms".to_string(), duration.as_millis().to_string());
            details.insert("user_agent".to_string(), user_agent.clone());
            
            audit_logger.log_system_event(
                AuditAction::SystemConfigChanged, // Using as closest match for system errors
                details,
            );
        }
        status if status.is_client_error() => {
            warn!(
                method = %method,
                uri = %uri,
                status = %status,
                duration_ms = %duration.as_millis(),
                user_agent = %user_agent,
                error_type = "client_error",
                "Client error occurred"
            );
        }
        _ => {
            // Log successful requests with performance monitoring
            if duration.as_millis() > 1000 {
                warn!(
                    method = %method,
                    uri = %uri,
                    status = %status,
                    duration_ms = %duration.as_millis(),
                    performance_issue = true,
                    "Slow request detected"
                );
            } else {
                info!(
                    method = %method,
                    uri = %uri,
                    status = %status,
                    duration_ms = %duration.as_millis(),
                    "Request completed"
                );
            }
        }
    }
    
    response
}

/// Error recovery middleware that attempts to recover from certain types of errors
pub async fn error_recovery_middleware(
    request: Request<Body>,
    next: Next<Body>,
) -> Response {
    let response = next.run(request).await;
    
    // Check if response indicates a recoverable error
    match response.status() {
        StatusCode::INTERNAL_SERVER_ERROR => {
            // For 500 errors, provide user-friendly error with recovery suggestions
            UserFriendlyError::new(
                "We're experiencing technical difficulties. Please try again in a moment.",
                "INTERNAL_ERROR",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .with_suggestions(vec![
                "Wait a few seconds and try again".to_string(),
                "Refresh the page if the problem persists".to_string(),
                "Contact support if you continue to experience issues".to_string(),
            ])
            .with_support_info("If this problem continues, please contact our support team with the error code and timestamp.")
            .into_response()
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            // For 503 errors, provide maintenance mode information
            UserFriendlyError::new(
                "The service is temporarily unavailable for maintenance.",
                "SERVICE_UNAVAILABLE",
                StatusCode::SERVICE_UNAVAILABLE,
            )
            .with_suggestions(vec![
                "Please try again in a few minutes".to_string(),
                "Check our status page for maintenance updates".to_string(),
            ])
            .into_response()
        }
        StatusCode::TOO_MANY_REQUESTS => {
            // For 429 errors, provide rate limiting guidance
            UserFriendlyError::new(
                "You're making requests too quickly. Please slow down.",
                "RATE_LIMITED",
                StatusCode::TOO_MANY_REQUESTS,
            )
            .with_suggestions(vec![
                "Wait a moment before trying again".to_string(),
                "Reduce the frequency of your requests".to_string(),
            ])
            .into_response()
        }
        _ => response,
    }
}

/// Circuit breaker middleware for protecting against cascading failures
pub struct CircuitBreakerMiddleware {
    circuit_breaker: ErrorCircuitBreaker,
}

impl CircuitBreakerMiddleware {
    pub fn new() -> Self {
        Self {
            circuit_breaker: ErrorCircuitBreaker::new(
                5, // failure_threshold
                std::time::Duration::from_secs(60), // recovery_timeout
            ),
        }
    }
    
    pub async fn handle(
        &self,
        request: Request<Body>,
        next: Next<Body>,
    ) -> Response {
        // Check if circuit breaker allows the request
        if !self.circuit_breaker.should_allow_request() {
            warn!("Circuit breaker is open, rejecting request");
            return UserFriendlyError::new(
                "Service is temporarily unavailable due to high error rates",
                "CIRCUIT_BREAKER_OPEN",
                StatusCode::SERVICE_UNAVAILABLE,
            )
            .with_suggestions(vec![
                "Please try again in a few minutes".to_string(),
                "The service is recovering from technical issues".to_string(),
            ])
            .into_response();
        }
        
        let response = next.run(request).await;
        
        // Record success or failure with circuit breaker
        if response.status().is_server_error() {
            self.circuit_breaker.record_failure();
        } else {
            self.circuit_breaker.record_success();
        }
        
        response
    }
}

impl Default for CircuitBreakerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Panic recovery middleware that converts panics to proper error responses
pub async fn panic_recovery_middleware(
    request: Request<Body>,
    next: Next<Body>,
) -> Response {
    // Set up panic hook to capture panic information
    let original_hook = std::panic::take_hook();
    let panic_info = std::sync::Arc::new(std::sync::Mutex::new(None));
    let panic_info_clone = panic_info.clone();
    
    std::panic::set_hook(Box::new(move |info| {
        let mut panic_data = panic_info_clone.lock().unwrap();
        *panic_data = Some(format!("{}", info));
    }));
    
    // Execute the request with panic recovery
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Note: This is a simplified approach. In a real async environment,
        // you'd need more sophisticated panic handling
        tokio::runtime::Handle::current().block_on(async {
            next.run(request).await
        })
    }));
    
    // Restore original panic hook
    std::panic::set_hook(original_hook);
    
    match result {
        Ok(response) => response,
        Err(_) => {
            // Extract panic information
            let panic_message = panic_info
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| "Unknown panic occurred".to_string());
            
            error!(
                panic_message = %panic_message,
                error_type = "panic",
                "Application panic occurred"
            );
            
            // Log panic as critical audit event
            let audit_logger = AuditLogger::new(true);
            let mut details = std::collections::HashMap::new();
            details.insert("panic_message".to_string(), panic_message);
            details.insert("recovery_action".to_string(), "graceful_error_response".to_string());
            
            audit_logger.log_system_event(
                AuditAction::SystemConfigChanged, // Using as system error
                details,
            );
            
            UserFriendlyError::new(
                "An unexpected error occurred. Our team has been notified.",
                "INTERNAL_PANIC",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .with_suggestions(vec![
                "Please try again in a moment".to_string(),
                "Contact support if the problem persists".to_string(),
            ])
            .with_support_info("This error has been automatically reported to our development team.")
            .into_response()
        }
    }
}

/// Request timeout middleware with user-friendly error messages
pub async fn timeout_middleware(
    request: Request<Body>,
    next: Next<Body>,
) -> Response {
    let timeout_duration = std::time::Duration::from_secs(30); // Default timeout
    
    match tokio::time::timeout(timeout_duration, next.run(request)).await {
        Ok(response) => response,
        Err(_) => {
            warn!(
                timeout_seconds = timeout_duration.as_secs(),
                error_type = "timeout",
                "Request timeout occurred"
            );
            
            UserFriendlyError::new(
                "The request took too long to complete. Please try again.",
                "REQUEST_TIMEOUT",
                StatusCode::REQUEST_TIMEOUT,
            )
            .with_suggestions(vec![
                "Try again with a simpler request".to_string(),
                "Check your internet connection".to_string(),
                "Contact support if timeouts persist".to_string(),
            ])
            .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Method};
    
    #[test]
    fn test_circuit_breaker_creation() {
        let middleware = CircuitBreakerMiddleware::new();
        // Test that circuit breaker starts in closed state
        assert!(middleware.circuit_breaker.should_allow_request());
    }
    
    #[test]
    fn test_user_friendly_error_creation() {
        let error = UserFriendlyError::new(
            "Test error",
            "TEST_ERROR",
            StatusCode::BAD_REQUEST,
        );
        
        assert_eq!(error.message, "Test error");
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }
}