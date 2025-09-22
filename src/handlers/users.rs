use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};

use crate::models::User;
use crate::AppState;

/// GET /api/users/me
/// 
/// Returns current authenticated user information
pub async fn get_current_user(
    State(state): State<AppState>,
) -> Result<Json<User>, StatusCode> {
    // TODO: Extract session token from headers
    // TODO: Validate session and get user
    
    Err(StatusCode::NOT_IMPLEMENTED)
}