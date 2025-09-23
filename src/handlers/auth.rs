use axum::{
    extract::State,
    http::{header::SET_COOKIE, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use tracing::{error, info, warn};

use crate::errors::AuthError;
use crate::middleware::session::SessionToken;
use crate::models::LoginResponse;
use crate::validation::{LoginRequest, sanitization, validate_request};
use crate::AppState;

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
    Json(request): Json<LoginRequest>,
) -> Response {
    // Validate request
    if let Err(validation_error) = validate_request(&request) {
        return validation_error.into_response();
    }
    
    // Sanitize input
    let email = sanitization::sanitize_plain_text(&request.email);
    let password = request.password; // Don't sanitize passwords
    
    info!("Login attempt for email: {}", email);
    
    // Authenticate user and create session
    let session = match state
        .auth_service
        .authenticate(email.clone(), password)
        .await
    {
        Ok(session) => session,
        Err(auth_error) => {
            warn!("Authentication failed for {}: {}", email, auth_error);
            return auth_error_to_response(auth_error);
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
            return create_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
                "SESSION_VALIDATION_FAILED"
            );
        }
    };

    info!("User {} logged in successfully", user.email);
    
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
    session_token: SessionToken,
) -> Response {
    info!("Logout attempt for session token");
    
    // Revoke the session
    match state
        .auth_service
        .revoke_session(session_token.token.clone())
        .await
    {
        Ok(()) => {
            info!("Session revoked successfully");
            
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

/// Converts AuthError to appropriate HTTP response
fn auth_error_to_response(error: AuthError) -> Response {
    let (status, message, code) = match error {
        AuthError::InvalidCredentials => (
            StatusCode::UNAUTHORIZED,
            "Invalid email or password",
            "INVALID_CREDENTIALS"
        ),
        AuthError::UserNotFound { .. } => (
            StatusCode::UNAUTHORIZED,
            "Invalid email or password", // Don't reveal if user exists
            "INVALID_CREDENTIALS"
        ),
        AuthError::SessionExpired => (
            StatusCode::UNAUTHORIZED,
            "Session expired",
            "SESSION_EXPIRED"
        ),
        AuthError::InvalidEmail { .. } => (
            StatusCode::BAD_REQUEST,
            "Invalid email format",
            "INVALID_EMAIL"
        ),
        AuthError::WeakPassword => (
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 characters long",
            "WEAK_PASSWORD"
        ),
        AuthError::EmailExists { .. } => (
            StatusCode::CONFLICT,
            "Email already exists",
            "EMAIL_EXISTS"
        ),
        AuthError::Database(_) | AuthError::PasswordHash(_) | AuthError::TokenGeneration => {
            error!("Internal auth error: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
                "INTERNAL_ERROR"
            )
        }
    };
    
    create_error_response(status, message, code)
}

/// Creates a standardized error response
fn create_error_response(status: StatusCode, message: &str, code: &str) -> Response {
    let error_body = json!({
        "error": {
            "message": message,
            "code": code,
            "status": status.as_u16()
        },
        "success": false
    });
    
    (status, Json(error_body)).into_response()
}