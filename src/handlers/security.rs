use axum::{
    http::{Request, StatusCode},
    response::{IntoResponse, Json},
};
use serde_json::json;
use tracing::debug;

use crate::middleware::security::CsrfProtection;

/// GET /api/security/csrf-token
/// 
/// Generate a new CSRF token for the client
/// 
/// # Response
/// - 200 OK: Returns CSRF token
/// - 500 Internal Server Error: Server error
pub async fn get_csrf_token(request: Request<axum::body::Body>) -> impl IntoResponse {
    // Get CSRF protection from request extensions
    if let Some(csrf) = request.extensions().get::<CsrfProtection>() {
        let token = csrf.generate_token();
        debug!("Generated CSRF token");
        
        (
            StatusCode::OK,
            Json(json!({
                "csrf_token": token,
                "expires_in": 3600 // 1 hour in seconds
            }))
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "CSRF protection not available",
                "message": "Server configuration error"
            }))
        )
    }
}

/// GET /api/security/headers
/// 
/// Get security configuration information for the client
/// 
/// # Response
/// - 200 OK: Returns security configuration
pub async fn get_security_info() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "csrf_protection": true,
            "content_security_policy": true,
            "rate_limiting": true,
            "input_sanitization": true,
            "security_headers": [
                "Content-Security-Policy",
                "X-Content-Type-Options",
                "X-Frame-Options",
                "X-XSS-Protection",
                "Referrer-Policy",
                "Permissions-Policy"
            ]
        }))
    )
}