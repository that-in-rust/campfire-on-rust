use axum::{
    response::Json,
};

use crate::middleware::session::AuthenticatedUser;
use crate::models::User;

/// GET /api/users/me
/// 
/// Returns current authenticated user information
pub async fn get_current_user(
    auth_user: AuthenticatedUser,
) -> Json<User> {
    // The middleware has already validated the session and extracted the user
    Json(auth_user.user)
}