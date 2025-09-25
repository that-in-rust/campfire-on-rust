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

/// Serve first-run setup page with enhanced error handling
/// 
/// # Preconditions
/// - Application is running
/// - Database is accessible
/// 
/// # Postconditions
/// - Returns setup page HTML if first-run detected
/// - Returns redirect to login if setup already complete
/// - Displays clean setup interface with organization branding
/// - Provides detailed error information and recovery options
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
            // Error checking first-run status - provide detailed error page with recovery options
            let error_html = format!(
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Setup Error - Campfire</title>
                    <meta charset="utf-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1">
                    <style>
                        body {{ 
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; 
                            margin: 0; 
                            padding: 40px; 
                            background: #f8f9fa;
                        }}
                        .container {{ 
                            max-width: 600px; 
                            margin: 0 auto; 
                            background: white; 
                            padding: 40px; 
                            border-radius: 8px; 
                            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                        }}
                        .error {{ 
                            background: #fee; 
                            border: 1px solid #fcc; 
                            padding: 20px; 
                            border-radius: 4px; 
                            margin-bottom: 20px;
                        }}
                        .recovery {{ 
                            background: #e8f4fd; 
                            border: 1px solid #bee5eb; 
                            padding: 20px; 
                            border-radius: 4px; 
                            margin-top: 20px;
                        }}
                        .btn {{ 
                            display: inline-block; 
                            padding: 10px 20px; 
                            background: #007bff; 
                            color: white; 
                            text-decoration: none; 
                            border-radius: 4px; 
                            margin-right: 10px;
                        }}
                        .btn:hover {{ background: #0056b3; }}
                        .btn-secondary {{ background: #6c757d; }}
                        .btn-secondary:hover {{ background: #545b62; }}
                        h1 {{ color: #dc3545; }}
                        h2 {{ color: #495057; }}
                    </style>
                </head>
                <body>
                    <div class="container">
                        <h1>Setup Error</h1>
                        <div class="error">
                            <strong>Unable to determine setup status:</strong><br>
                            {}
                        </div>
                        
                        <h2>Recovery Options</h2>
                        <div class="recovery">
                            <p><strong>Try these steps to resolve the issue:</strong></p>
                            <ol>
                                <li>Check if the database is accessible and properly configured</li>
                                <li>Verify environment variables are set correctly</li>
                                <li>Ensure the application has proper file permissions</li>
                                <li>Check the application logs for more detailed error information</li>
                            </ol>
                            
                            <p><strong>Quick Actions:</strong></p>
                            <a href="/health" class="btn">Check System Health</a>
                            <a href="/api/setup/status" class="btn btn-secondary">Check Setup Status (JSON)</a>
                            <a href="/" class="btn btn-secondary">Return to Home</a>
                        </div>
                        
                        <div style="margin-top: 30px; padding-top: 20px; border-top: 1px solid #dee2e6; color: #6c757d; font-size: 14px;">
                            <p><strong>Technical Details:</strong> This error occurred while checking if the application requires first-run setup. 
                            The setup detection process failed, which may indicate database connectivity issues or configuration problems.</p>
                        </div>
                    </div>
                </body>
                </html>
                "#,
                html_escape::encode_text(&e.to_string())
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

/// Create admin account API endpoint with enhanced error handling and validation
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
/// - Provides detailed error information for troubleshooting
pub async fn create_admin_account(
    State(state): State<AppState>,
    Json(request): Json<CreateAdminRequest>,
) -> impl IntoResponse {
    // Pre-validation: Check if setup is still needed
    match state.setup_service.is_first_run().await {
        Ok(false) => {
            // Setup already complete
            let error_response = json!({
                "success": false,
                "error": "SETUP_ALREADY_COMPLETE",
                "message": "Admin account already exists. Setup has been completed.",
                "redirect_url": "/login",
                "recovery_actions": [
                    "Use the existing admin credentials to log in",
                    "If you've forgotten the admin password, check your deployment documentation",
                    "Contact your system administrator for password reset procedures"
                ]
            });
            
            return (StatusCode::CONFLICT, Json(error_response)).into_response();
        }
        Ok(true) => {
            // First run confirmed - continue with account creation
        }
        Err(e) => {
            // Error checking first-run status
            let error_response = json!({
                "success": false,
                "error": "SETUP_VALIDATION_ERROR",
                "message": format!("Unable to validate setup status: {}", e),
                "recovery_actions": [
                    "Check database connectivity",
                    "Verify environment configuration",
                    "Check application logs for detailed error information",
                    "Try refreshing the page and attempting setup again"
                ]
            });
            
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    }
    
    // Attempt to create admin account
    match state.setup_service.create_admin_account(request).await {
        Ok(response) => {
            // Success - set session cookie for immediate login
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
            
            // Validate system health after successful setup
            let system_health = state.setup_service.validate_system_health().await
                .unwrap_or_else(|_| crate::models::SystemHealth {
                    database_connected: true, // We know it worked since we created the account
                    fts_search_available: false,
                    websocket_ready: true,
                    push_notifications_configured: response.deployment_config.vapid_public_key.is_some(),
                    static_assets_embedded: true,
                    admin_account_exists: true,
                });
            
            let success_response = json!({
                "success": true,
                "message": "Admin account created successfully! You are now logged in.",
                "user": {
                    "id": response.user.id,
                    "name": response.user.name,
                    "email": response.user.email,
                    "admin": response.user.admin,
                    "created_at": response.user.created_at
                },
                "session_token": response.session_token,
                "deployment_config": response.deployment_config,
                "system_health": system_health,
                "redirect_url": "/chat",
                "next_steps": [
                    "You can now access the chat interface",
                    "Create additional rooms and invite users",
                    "Configure push notifications if desired",
                    "Set up bot integrations if needed"
                ]
            });
            
            (StatusCode::CREATED, headers, Json(success_response)).into_response()
        }
        Err(e) => {
            // Error creating admin account - provide detailed error information
            let (error_code, recovery_actions) = match &e {
                crate::errors::SetupError::NotFirstRun => (
                    "NOT_FIRST_RUN",
                    vec![
                        "Refresh the page and check if setup is already complete".to_string(),
                        "If you see this error repeatedly, check for database connectivity issues".to_string(),
                        "Try accessing /login to see if an admin account already exists".to_string(),
                    ]
                ),
                crate::errors::SetupError::InvalidEmail { email } => (
                    "INVALID_EMAIL",
                    vec![
                        format!("Provide a valid email address (current: '{}')", email),
                        "Email must contain '@' and a domain (e.g., admin@example.com)".to_string(),
                        "Avoid special characters that might cause issues".to_string(),
                    ]
                ),
                crate::errors::SetupError::WeakPassword { reason } => (
                    "WEAK_PASSWORD",
                    vec![
                        format!("Password requirement: {}", reason),
                        "Use at least 8 characters with letters and numbers".to_string(),
                        "Consider using a password manager for strong passwords".to_string(),
                    ]
                ),
                crate::errors::SetupError::AdminCreationFailed(msg) => (
                    "ADMIN_CREATION_FAILED",
                    vec![
                        "Check database connectivity and permissions".to_string(),
                        "Verify the database schema is properly initialized".to_string(),
                        format!("Technical details: {}", msg),
                        "Check application logs for more information".to_string(),
                    ]
                ),
                crate::errors::SetupError::Database(db_err) => (
                    "DATABASE_ERROR",
                    vec![
                        "Check database connectivity".to_string(),
                        "Verify database file permissions".to_string(),
                        "Ensure sufficient disk space".to_string(),
                        format!("Database error: {}", db_err),
                    ]
                ),
                _ => (
                    "SETUP_ERROR",
                    vec![
                        "Check system health at /health".to_string(),
                        "Verify environment configuration".to_string(),
                        "Check application logs for detailed error information".to_string(),
                        "Try the setup process again".to_string(),
                    ]
                ),
            };
            
            let error_response = json!({
                "success": false,
                "error": error_code,
                "message": e.to_string(),
                "recovery_actions": recovery_actions,
                "support_info": {
                    "health_check_url": "/health",
                    "setup_status_url": "/api/setup/status",
                    "environment_check_url": "/api/setup/environment"
                }
            });
            
            let status_code = match &e {
                crate::errors::SetupError::NotFirstRun => StatusCode::CONFLICT,
                crate::errors::SetupError::InvalidEmail { .. } => StatusCode::BAD_REQUEST,
                crate::errors::SetupError::WeakPassword { .. } => StatusCode::BAD_REQUEST,
                crate::errors::SetupError::AdminCreationFailed(_) => StatusCode::UNPROCESSABLE_ENTITY,
                crate::errors::SetupError::Database(_) => StatusCode::SERVICE_UNAVAILABLE,
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
        CampfireDatabase,
        services::setup::{SetupService, SetupServiceImpl},
        models::CreateAdminRequest,
    };
    use std::sync::Arc;

    async fn create_test_setup_service() -> SetupServiceImpl {
        let database = CampfireDatabase::new("sqlite::memory:").await.unwrap();
        SetupServiceImpl::new(database)
    }
    
    #[tokio::test]
    async fn test_setup_service_first_run() {
        let service = create_test_setup_service().await;
        
        // Should detect first run on empty database
        assert!(service.is_first_run().await.unwrap());
    }
    
    #[tokio::test]
    async fn test_setup_service_admin_creation() {
        let service = create_test_setup_service().await;
        
        let request = CreateAdminRequest {
            email: "admin@example.com".to_string(),
            password: "securepass123".to_string(),
            name: "System Admin".to_string(),
        };
        
        let response = service.create_admin_account(request).await.unwrap();
        
        assert_eq!(response.user.email, "admin@example.com");
        assert_eq!(response.user.name, "System Admin");
        assert!(response.user.admin);
        assert!(!response.session_token.is_empty());
    }
    
    #[tokio::test]
    async fn test_setup_service_invalid_email() {
        let service = create_test_setup_service().await;
        
        let request = CreateAdminRequest {
            email: "invalid-email".to_string(),
            password: "securepass123".to_string(),
            name: "System Admin".to_string(),
        };
        
        let result = service.create_admin_account(request).await;
        assert!(matches!(result, Err(crate::errors::SetupError::InvalidEmail { .. })));
    }
    
    #[tokio::test]
    async fn test_setup_service_weak_password() {
        let service = create_test_setup_service().await;
        
        let request = CreateAdminRequest {
            email: "admin@example.com".to_string(),
            password: "weak".to_string(),
            name: "System Admin".to_string(),
        };
        
        let result = service.create_admin_account(request).await;
        assert!(matches!(result, Err(crate::errors::SetupError::WeakPassword { .. })));
    }
    
    #[tokio::test]
    async fn test_setup_service_system_health() {
        let service = create_test_setup_service().await;
        
        let health = service.validate_system_health().await.unwrap();
        
        assert!(health.database_connected);
        assert!(health.fts_search_available);
        assert!(health.websocket_ready);
        assert!(health.static_assets_embedded);
        assert!(!health.admin_account_exists); // No admin created yet
    }
}