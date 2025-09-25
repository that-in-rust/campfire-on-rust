use axum::{
    extract::{State, ConnectInfo},
    http::{header::SET_COOKIE, StatusCode, HeaderMap},
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::net::SocketAddr;
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::errors::AuthError;
use crate::middleware::session::SessionToken;
use crate::models::LoginResponse;
use crate::validation::{LoginRequest, sanitization, validate_request};
use crate::logging::{audit::{AuditAction, AuditLogger}, error_handling::handle_auth_error};
use crate::{AppState, audit_user_action, audit_security_event, log_audit_event};

/// POST /api/auth/login
/// 
/// Authenticates user with email/password and returns session token
/// 
/// # Request Body
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "password123"
/// }
/// ```
/// 
/// # Response
/// - 200 OK: Authentication successful, returns user and session token
/// - 400 Bad Request: Invalid request format
/// - 401 Unauthorized: Invalid credentials
/// - 500 Internal Server Error: Server error
pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Response {
    // Extract client information for audit logging
    let ip_address = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Initialize audit logger
    let audit_logger = AuditLogger::new(true); // TODO: Get from config

    // Validate request
    if let Err(validation_error) = validate_request(&request) {
        // Log failed login attempt due to validation
        let mut details = HashMap::new();
        details.insert("reason".to_string(), "validation_failed".to_string());
        details.insert("email".to_string(), request.email.clone());
        
        audit_logger.log_security_event(
            AuditAction::Login,
            None,
            Some(&ip_address),
            details,
        );
        
        return validation_error.into_response();
    }
    
    // Sanitize input
    let email = sanitization::sanitize_plain_text(&request.email);
    let password = request.password; // Don't sanitize passwords
    
    info!("Login attempt for email: {} from IP: {}", email, ip_address);
    
    // Authenticate user and create session
    let session = match state
        .auth_service
        .authenticate(email.clone(), password)
        .await
    {
        Ok(session) => session,
        Err(auth_error) => {
            warn!("Authentication failed for {}: {}", email, auth_error);
            
            // Log failed login attempt
            let mut details = HashMap::new();
            details.insert("email".to_string(), email.clone());
            details.insert("error".to_string(), auth_error.to_string());
            details.insert("user_agent".to_string(), user_agent);
            
            audit_logger.log_security_event(
                AuditAction::Login,
                None,
                Some(&ip_address),
                details,
            );
            
            return handle_auth_error(auth_error, Some("login")).into_response();
        }
    };

    // Get user information (session was just created, so this should succeed)
    let user = match state
        .auth_service
        .validate_session(session.token.clone())
        .await
    {
        Ok(user) => user,
        Err(auth_error) => {
            error!("Failed to validate newly created session: {}", auth_error);
            
            // Log session validation failure
            let mut details = HashMap::new();
            details.insert("email".to_string(), email.clone());
            details.insert("error".to_string(), "session_validation_failed".to_string());
            
            audit_logger.log_security_event(
                AuditAction::Login,
                None,
                Some(&ip_address),
                details,
            );
            
            return handle_auth_error(
                AuthError::TokenGeneration,
                Some("session_validation")
            ).into_response();
        }
    };

    info!("User {} logged in successfully from IP: {}", user.email, ip_address);
    
    // Log successful login
    let mut details = HashMap::new();
    details.insert("email".to_string(), user.email.clone());
    details.insert("user_agent".to_string(), user_agent);
    details.insert("session_id".to_string(), session.token.clone());
    
    audit_logger.log_user_action(
        AuditAction::Login,
        user.id,
        "session",
        Some(session.token.clone()),
        details,
    );
    
    let response = LoginResponse {
        user,
        session_token: session.token.clone(),
    };

    // Set session cookie for automatic authentication
    let cookie = format!(
        "session_token={}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}",
        session.token,
        30 * 24 * 60 * 60 // 30 days in seconds
    );

    let mut response = (StatusCode::OK, Json(response)).into_response();
    response.headers_mut().insert(SET_COOKIE, cookie.parse().unwrap());
    response
}

/// POST /api/auth/logout
/// 
/// Revokes the current session token
/// 
/// # Authentication
/// Requires valid session token in Authorization header or cookie
/// 
/// # Response
/// - 200 OK: Logout successful
/// - 401 Unauthorized: Invalid or missing session token
/// - 500 Internal Server Error: Server error
pub async fn logout(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    session_token: SessionToken,
) -> Response {
    let ip_address = addr.ip().to_string();
    let audit_logger = AuditLogger::new(true); // TODO: Get from config
    
    info!("Logout attempt for session token from IP: {}", ip_address);
    
    // Get user info before revoking session for audit logging
    let user_info = state
        .auth_service
        .validate_session(session_token.token.clone())
        .await
        .ok();
    
    // Revoke the session
    match state
        .auth_service
        .revoke_session(session_token.token.clone())
        .await
    {
        Ok(()) => {
            info!("Session revoked successfully");
            
            // Log successful logout
            if let Some(user) = &user_info {
                let mut details = HashMap::new();
                details.insert("email".to_string(), user.email.clone());
                details.insert("session_id".to_string(), session_token.token.clone());
                
                audit_logger.log_user_action(
                    AuditAction::Logout,
                    user.id,
                    "session",
                    Some(session_token.token.clone()),
                    details,
                );
            }
            
            // Clear session cookie
            let clear_cookie = "session_token=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0";
            
            let mut response = (
                StatusCode::OK,
                Json(json!({ 
                    "message": "Logged out successfully",
                    "success": true
                }))
            ).into_response();
            
            response.headers_mut().insert(SET_COOKIE, clear_cookie.parse().unwrap());
            response
        }
        Err(auth_error) => {
            error!("Failed to revoke session: {}", auth_error);
            
            // Log logout attempt failure
            if let Some(user) = &user_info {
                let mut details = HashMap::new();
                details.insert("email".to_string(), user.email.clone());
                details.insert("session_id".to_string(), session_token.token.clone());
                details.insert("error".to_string(), auth_error.to_string());
                
                audit_logger.log_security_event(
                    AuditAction::Logout,
                    Some(user.id),
                    Some(&ip_address),
                    details,
                );
            }
            
            // Even if revocation fails, we should return success to the client
            // The session might already be expired or invalid
            
            // Clear session cookie anyway
            let clear_cookie = "session_token=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0";
            
            let mut response = (
                StatusCode::OK,
                Json(json!({ 
                    "message": "Logged out successfully",
                    "success": true
                }))
            ).into_response();
            
            response.headers_mut().insert(SET_COOKIE, clear_cookie.parse().unwrap());
            response
        }
    }
}

// Note: auth_error_to_response and create_error_response functions removed
// Now using the enhanced error handling from logging::error_handling module