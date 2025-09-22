use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

use crate::middleware::session::SessionToken;
use crate::models::{LoginRequest, LoginResponse};
use crate::AppState;

/// POST /api/auth/login
/// 
/// Authenticates user with email/password and returns session token
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Authenticate user and create session
    let session = state
        .auth_service
        .authenticate(request.email, request.password)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Get user information
    let user = state
        .auth_service
        .validate_session(session.token.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = LoginResponse {
        user,
        session_token: session.token,
    };

    Ok(Json(response))
}

/// POST /api/auth/logout
/// 
/// Revokes the current session token
pub async fn logout(
    State(state): State<AppState>,
    session_token: SessionToken,
) -> Result<Json<Value>, StatusCode> {
    // Revoke the session
    state
        .auth_service
        .revoke_session(session_token.token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(json!({ "message": "Logged out successfully" })))
}