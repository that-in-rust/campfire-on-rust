use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use tracing::info;

use crate::middleware::session::AuthenticatedUser;

/// GET /api/users/me
/// 
/// Returns current authenticated user information
/// 
/// # Authentication
/// Requires valid session token in Authorization header or cookie
/// 
/// # Response
/// - 200 OK: Returns user information
/// - 401 Unauthorized: Invalid or missing session token
/// 
/// # Response Body
/// ```json
/// {
///   "id": "uuid",
///   "name": "User Name",
///   "email": "user@example.com",
///   "bio": "Optional bio",
///   "admin": false,
///   "created_at": "2023-01-01T00:00:00Z"
/// }
/// ```
pub async fn get_current_user(
    auth_user: AuthenticatedUser,
) -> Response {
    info!("Fetching current user info for user: {}", auth_user.user.email);
    
    // Create response with user data (excluding sensitive fields)
    let user_response = json!({
        "id": auth_user.user.id,
        "name": auth_user.user.name,
        "email": auth_user.user.email,
        "bio": auth_user.user.bio,
        "admin": auth_user.user.admin,
        "created_at": auth_user.user.created_at,
        // Exclude password_hash and bot_token for security
    });
    
    (StatusCode::OK, Json(user_response)).into_response()
}