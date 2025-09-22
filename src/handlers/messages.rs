use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::models::{CreateMessageRequest, Message, MessageId, RoomId};
use crate::AppState;

#[derive(Deserialize)]
pub struct GetMessagesQuery {
    limit: Option<u32>,
    before: Option<String>, // MessageId as string
}

/// POST /api/rooms/:room_id/messages
/// 
/// Creates a new message in the specified room with deduplication
pub async fn create_message(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    Json(request): Json<CreateMessageRequest>,
) -> Result<Json<Message>, StatusCode> {
    // TODO: Parse room_id
    // TODO: Extract user from session
    // TODO: Use message service to create message with deduplication
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// GET /api/rooms/:room_id/messages
/// 
/// Retrieves message history for the specified room
pub async fn get_messages(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    Query(query): Query<GetMessagesQuery>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    // TODO: Parse room_id
    // TODO: Extract user from session
    // TODO: Use message service to get room messages
    
    Err(StatusCode::NOT_IMPLEMENTED)
}