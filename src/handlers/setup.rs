use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Json},
};
use serde_json::json;

use crate::{
    AppState,
    models::CreateAdminRequest,
};

/// Serve first-run setup page
/// 
/// # Preconditions
/// - Application is running
/// - Database is accessible
/// 
/// # Postconditions
/// - Returns setup page HTML if first-run detected
/// - Returns redirect to login if setup already complete
/// - Displays clean setup interface with organization branding
pub async fn serve_setup_page(State(state): State<AppState>) -> impl IntoResponse {
    // Check if this is a first-run scenario
    match state.setup_service.is_first_run().await {
        Ok(true) => {
            // First run - serve setup page
            let html = include_str!("../../templates/setup.html");
            
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            
            (StatusCode::OK, headers, Html(html)).into_response()
        }
        Ok(false) => {
            // Not first run - redirect to login
            let mut headers = HeaderMap::new();
            headers.insert(
                header::LOCATION,
                HeaderValue::from_static("/login"),
            );
            
            (StatusCode::FOUND, headers, Html("")).into_response()
        }
        Err(e) => {
            // Error checking first-run status
            let error_html = format!(
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Setup Error - Campfire</title>
                    <style>
                        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; }}
                        .error {{ background: #fee; border: 1px solid #fcc; padding: 20px; border-radius: 4px; }}
                    </style>
                </head>
                <body>
                    <div class="error">
                        <h1>Setup Error</h1>
                        <p>Unable to determine setup status: {}</p>
                        <p><a href="/health">Check system health</a></p>
                    </div>
                </body>
                </html>
                "#,
                e
            );
            
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            
            (StatusCode::INTERNAL_SERVER_ERROR, headers, Html(error_html)).into_response()
        }
    }
}

/// Get setup status API endpoint
/// 
/// # Preconditions
/// - Application is running
/// - Database is accessible
/// 
/// # Postconditions
/// - Returns JSON with setup status information
/// - Includes first-run detection, admin existence, system health
pub async fn get_setup_status(State(state): State<AppState>) -> impl IntoResponse {
    match state.setup_service.get_setup_status().await {
        Ok(status) => {
            Json(status).into_response()
        }
        Err(e) => {
            let error_response = json!({
                "error": "SETUP_STATUS_ERROR",
                "message": format!("Failed to get setup status: {}", e),
                "is_first_run": false,
                "admin_exists": false,
                "system_health": {
                    "database_connected": false,
                    "fts_search_available": false,
                    "websocket_ready": false,
                    "push_notifications_configured": false,
                    "static_assets_embedded": true,
                    "admin_account_exists": false
                }
            });
            
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

/// Create admin account API endpoint
/// 
/// # Preconditions
/// - First-run condition verified
/// - Valid email and password provided in request body
/// - Email format validated
/// - Password strength requirements met
/// 
/// # Postconditions
/// - Creates admin user with full permissions
/// - Marks user as primary administrator
/// - Returns created user and session token
/// - Enables subsequent normal login flow
pub async fn create_admin_account(
    State(state): State<AppState>,
    Json(request): Json<CreateAdminRequest>,
) -> impl IntoResponse {
    match state.setup_service.create_admin_account(request).await {
        Ok(response) => {
            // Set session cookie for immediate login
            let cookie_value = format!(
                "campfire_session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
                response.session_token,
                24 * 60 * 60 // 24 hours in seconds
            );
            
            let mut headers = HeaderMap::new();
            headers.insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&cookie_value).unwrap(),
            );
            
            let success_response = json!({
                "success": true,
                "message": "Admin account created successfully",
                "user": {
                    "id": response.user.id,
                    "name": response.user.name,
                    "email": response.user.email,
                    "admin": response.user.admin
                },
                "session_token": response.session_token,
                "deployment_config": response.deployment_config,
                "redirect_url": "/chat"
            });
            
            (StatusCode::CREATED, headers, Json(success_response)).into_response()
        }
        Err(e) => {
            let error_code = match &e {
                crate::errors::SetupError::NotFirstRun => "NOT_FIRST_RUN",
                crate::errors::SetupError::InvalidEmail { .. } => "INVALID_EMAIL",
                crate::errors::SetupError::WeakPassword { .. } => "WEAK_PASSWORD",
                crate::errors::SetupError::AdminCreationFailed(_) => "ADMIN_CREATION_FAILED",
                _ => "SETUP_ERROR",
            };
            
            let error_response = json!({
                "success": false,
                "error": error_code,
                "message": e.to_string()
            });
            
            let status_code = match &e {
                crate::errors::SetupError::NotFirstRun => StatusCode::CONFLICT,
                crate::errors::SetupError::InvalidEmail { .. } => StatusCode::BAD_REQUEST,
                crate::errors::SetupError::WeakPassword { .. } => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            
            (status_code, Json(error_response)).into_response()
        }
    }
}

/// Validate environment configuration endpoint
/// 
/// # Preconditions
/// - Application is running
/// - Environment variables accessible
/// 
/// # Postconditions
/// - Returns deployment configuration with validation status
/// - Includes environment variable validation results
pub async fn validate_environment(State(state): State<AppState>) -> impl IntoResponse {
    match state.setup_service.get_deployment_config().await {
        Ok(config) => {
            // Validate system health
            let health = state.setup_service.validate_system_health().await
                .unwrap_or_else(|_| crate::models::SystemHealth {
                    database_connected: false,
                    fts_search_available: false,
                    websocket_ready: false,
                    push_notifications_configured: false,
                    static_assets_embedded: true,
                    admin_account_exists: false,
                });
            
            let validation_response = json!({
                "valid": true,
                "deployment_config": config,
                "system_health": health,
                "recommendations": {
                    "vapid_keys": if config.vapid_public_key.is_some() && config.vapid_private_key.is_some() {
                        "Push notifications configured"
                    } else {
                        "Consider setting CAMPFIRE_VAPID_PUBLIC_KEY and CAMPFIRE_VAPID_PRIVATE_KEY for push notifications"
                    },
                    "ssl_domain": if config.ssl_domain.is_some() {
                        "SSL domain configured for automatic HTTPS"
                    } else {
                        "Consider setting CAMPFIRE_SSL_DOMAIN for automatic SSL with Let's Encrypt"
                    },
                    "user_registration": if config.enable_user_registration {
                        "User registration is enabled"
                    } else {
                        "User registration is disabled - only admin can create accounts"
                    }
                }
            });
            
            Json(validation_response).into_response()
        }
        Err(e) => {
            let error_response = json!({
                "valid": false,
                "error": "ENVIRONMENT_VALIDATION_ERROR",
                "message": format!("Environment validation failed: {}", e)
            });
            
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        database::Database,
        services::setup::SetupServiceImpl,
        models::CreateAdminRequest,
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use std::sync::Arc;

    async fn create_test_app_state() -> AppState {
        let database = Database::new("sqlite::memory:").await.unwrap();
        let setup_service = Arc::new(SetupServiceImpl::new(database.clone()));
        
        AppState {
            db: database,
            setup_service,
            // Add other required fields with mock implementations
            connection_manager: Arc::new(crate::services::connection::ConnectionManagerImpl::new()),
            message_service: Arc::new(crate::services::message::MessageServiceImpl::new(
                database.clone(),
                Arc::new(crate::services::connection::ConnectionManagerImpl::new()),
            )),
            // ... other services would be mocked here
        }
    }
    
    #[tokio::test]
    async fn test_serve_setup_page_first_run() {
        let state = create_test_app_state().await;
        
        // Should serve setup page on first run
        let response = serve_setup_page(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Should contain HTML content type
        let content_type = response.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().contains("text/html"));
    }
    
    #[tokio::test]
    async fn test_get_setup_status() {
        let state = create_test_app_state().await;
        
        let response = get_setup_status(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Response should be JSON
        let content_type = response.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().contains("application/json"));
    }
    
    #[tokio::test]
    async fn test_create_admin_account_success() {
        let state = create_test_app_state().await;
        
        let request = CreateAdminRequest {
            email: "admin@example.com".to_string(),
            password: "securepass123".to_string(),
            name: "System Admin".to_string(),
        };
        
        let response = create_admin_account(State(state), Json(request)).await.into_response();
        assert_eq!(response.status(), StatusCode::CREATED);
        
        // Should set session cookie
        let set_cookie = response.headers().get("set-cookie");
        assert!(set_cookie.is_some());
        assert!(set_cookie.unwrap().to_str().unwrap().contains("campfire_session="));
    }
    
    #[tokio::test]
    async fn test_create_admin_account_invalid_email() {
        let state = create_test_app_state().await;
        
        let request = CreateAdminRequest {
            email: "invalid-email".to_string(),
            password: "securepass123".to_string(),
            name: "System Admin".to_string(),
        };
        
        let response = create_admin_account(State(state), Json(request)).await.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    
    #[tokio::test]
    async fn test_create_admin_account_weak_password() {
        let state = create_test_app_state().await;
        
        let request = CreateAdminRequest {
            email: "admin@example.com".to_string(),
            password: "weak".to_string(),
            name: "System Admin".to_string(),
        };
        
        let response = create_admin_account(State(state), Json(request)).await.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    
    #[tokio::test]
    async fn test_validate_environment() {
        let state = create_test_app_state().await;
        
        let response = validate_environment(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        
        // Response should be JSON
        let content_type = response.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().contains("application/json"));
    }
}