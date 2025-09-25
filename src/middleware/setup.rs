use axum::{
    extract::State,
    http::{header, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::{debug, warn};

use crate::AppState;

/// Middleware to handle first-run setup detection and redirection
/// 
/// This middleware checks if the application is in a first-run state and automatically
/// redirects users to the setup page when needed, except for setup-related endpoints.
/// 
/// # Preconditions
/// - Application is running with valid database connection
/// - Setup service is available in AppState
/// 
/// # Postconditions
/// - Redirects to /setup if first-run detected (except for setup endpoints)
/// - Allows normal request processing if setup is complete
/// - Handles setup detection errors gracefully
pub async fn setup_detection_middleware<B>(
    State(state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let uri = request.uri();
    let path = uri.path();
    
    // Skip setup detection for setup-related endpoints and static assets
    if is_setup_related_path(path) {
        debug!("Skipping setup detection for path: {}", path);
        return next.run(request).await;
    }
    
    // Check if this is a first-run scenario
    match state.setup_service.is_first_run().await {
        Ok(true) => {
            // First run detected - redirect to setup page
            debug!("First-run detected, redirecting to setup page");
            
            (StatusCode::FOUND, [(header::LOCATION, "/setup")], "").into_response()
        }
        Ok(false) => {
            // Setup complete - continue with normal request processing
            debug!("Setup complete, continuing with normal request processing");
            next.run(request).await
        }
        Err(e) => {
            // Error checking setup status - log warning and continue
            // This prevents setup detection errors from breaking the application
            warn!("Failed to check first-run status: {}", e);
            next.run(request).await
        }
    }
}

/// Check if a path is related to setup functionality and should bypass setup detection
/// 
/// Setup-related paths include:
/// - /setup (setup page)
/// - /api/setup/* (setup API endpoints)
/// - /health/* (health check endpoints)
/// - /static/* (static assets)
/// - /manifest.json (PWA manifest)
fn is_setup_related_path(path: &str) -> bool {
    path == "/setup" 
        || path.starts_with("/api/setup/")
        || path.starts_with("/health")
        || path.starts_with("/static/")
        || path == "/manifest.json"
}

/// Middleware to validate setup completion for protected endpoints
/// 
/// This middleware ensures that certain endpoints are only accessible after
/// setup is complete. It's more restrictive than the general setup detection
/// middleware and is applied to specific protected routes.
/// 
/// # Preconditions
/// - Application is running with valid database connection
/// - Setup service is available in AppState
/// 
/// # Postconditions
/// - Returns 503 Service Unavailable if setup is not complete
/// - Allows request processing if setup is complete
/// - Handles setup validation errors gracefully
pub async fn setup_completion_middleware<B>(
    State(state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    // Check if setup is complete
    match state.setup_service.is_first_run().await {
        Ok(false) => {
            // Setup is complete - continue with request processing
            debug!("Setup validation passed, continuing with request");
            next.run(request).await
        }
        Ok(true) => {
            // Setup not complete - return service unavailable
            debug!("Setup not complete, returning service unavailable");
            
            let error_response = serde_json::json!({
                "error": "SETUP_REQUIRED",
                "message": "Application setup is required before accessing this endpoint",
                "setup_url": "/setup"
            });
            
            (StatusCode::SERVICE_UNAVAILABLE, axum::Json(error_response)).into_response()
        }
        Err(e) => {
            // Error checking setup status - return internal server error
            warn!("Failed to validate setup completion: {}", e);
            
            let error_response = serde_json::json!({
                "error": "SETUP_VALIDATION_ERROR",
                "message": format!("Failed to validate setup status: {}", e)
            });
            
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(error_response)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_setup_related_path() {
        // Setup-related paths should return true
        assert!(is_setup_related_path("/setup"));
        assert!(is_setup_related_path("/api/setup/status"));
        assert!(is_setup_related_path("/api/setup/admin"));
        assert!(is_setup_related_path("/health"));
        assert!(is_setup_related_path("/health/ready"));
        assert!(is_setup_related_path("/static/css/campfire.css"));
        assert!(is_setup_related_path("/manifest.json"));
        
        // Non-setup paths should return false
        assert!(!is_setup_related_path("/"));
        assert!(!is_setup_related_path("/login"));
        assert!(!is_setup_related_path("/chat"));
        assert!(!is_setup_related_path("/api/auth/login"));
        assert!(!is_setup_related_path("/api/rooms"));
        assert!(!is_setup_related_path("/ws"));
    }
}