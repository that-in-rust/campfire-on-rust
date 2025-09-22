use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

use crate::models::{LoginRequest, LoginResponse};
use crate::services::auth::AuthServiceTrait;
use crate::AppState;

/// POST /api/auth/login
/// 
/// Authenticates user with email/password and returns session token
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // TODO: Implement auth service in AppState
    // For now, return a placeholder response
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// POST /api/auth/logout
/// 
/// Revokes the current session token
pub async fn logout(
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Extract session token from headers
    // TODO: Revoke session using auth service
    
    Ok(Json(json!({ "message": "Logged out successfully" })))
}