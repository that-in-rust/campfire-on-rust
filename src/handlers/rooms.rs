use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};

use crate::models::{CreateRoomRequest, Room};
use crate::AppState;

/// GET /api/rooms
/// 
/// Returns list of rooms the current user has access to
pub async fn get_rooms(
    State(state): State<AppState>,
) -> Result<Json<Vec<Room>>, StatusCode> {
    // TODO: Extract user from session
    // TODO: Use room service to get user rooms
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// POST /api/rooms
/// 
/// Creates a new room
pub async fn create_room(
    State(state): State<AppState>,
    Json(request): Json<CreateRoomRequest>,
) -> Result<Json<Room>, StatusCode> {
    // TODO: Extract user from session
    // TODO: Use room service to create room
    
    Err(StatusCode::NOT_IMPLEMENTED)
}