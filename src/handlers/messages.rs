use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::errors::MessageError;
use crate::middleware::AuthenticatedUser;
use crate::models::{Message, MessageId, RoomId};
use crate::validation::{CreateMessageRequest, sanitization, validate_request};
use crate::AppState;

#[derive(Deserialize)]
pub struct GetMessagesQuery {
    limit: Option<u32>,
    before: Option<String>, // MessageId as string
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: Message,
}

#[derive(Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<Message>,
    pub has_more: bool,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

/// POST /api/rooms/:room_id/messages
/// 
/// Creates a new message in the specified room with deduplication
/// 
/// # Authentication
/// Requires valid session token via Authorization header or cookie
/// 
/// # Request Body
/// ```json
/// {
///   "content": "Message content (1-10000 chars)",
///   "client_message_id": "uuid-v4-string"
/// }
/// ```
/// 
/// # Response
/// - 201: Message created successfully
/// - 400: Invalid request (bad content, invalid UUID)
/// - 401: Authentication required
/// - 403: User not authorized for room
/// - 500: Internal server error
pub async fn create_message(
    State(state): State<AppState>,
    Path(room_id_str): Path<String>,
    auth_user: AuthenticatedUser,
    Json(request): Json<CreateMessageRequest>,
) -> Result<Response, Response> {
    info!(
        "Creating message in room {} for user {}",
        room_id_str, auth_user.user.id
    );

    // Validate request
    if let Err(validation_error) = validate_request(&request) {
        return Err(validation_error.into_response());
    }
    
    // Parse room_id from path parameter
    let room_id = parse_room_id(&room_id_str)?;

    // Sanitize message content
    let content = sanitization::sanitize_message_content(&request.content);

    // Use message service to create message with deduplication
    match state
        .message_service
        .create_message_with_deduplication(
            content,
            room_id,
            auth_user.user.id,
            request.client_message_id,
        )
        .await
    {
        Ok(message) => {
            info!("Message created successfully: {}", message.id);
            Ok((
                StatusCode::CREATED,
                Json(MessageResponse { message }),
            ).into_response())
        }
        Err(MessageError::Authorization { user_id, room_id }) => {
            warn!("User {} not authorized for room {}", user_id, room_id);
            Err(create_error_response(
                StatusCode::FORBIDDEN,
                "You are not authorized to post messages in this room",
            ))
        }
        Err(MessageError::InvalidContent { reason }) => {
            warn!("Invalid message content: {}", reason);
            Err(create_error_response(
                StatusCode::BAD_REQUEST,
                &format!("Invalid message content: {}", reason),
            ))
        }
        Err(MessageError::ContentTooLong { length }) => {
            warn!("Message content too long: {} chars", length);
            Err(create_error_response(
                StatusCode::BAD_REQUEST,
                &format!("Message content too long: {} chars (max: 10000)", length),
            ))
        }
        Err(MessageError::ContentTooShort) => {
            warn!("Message content too short");
            Err(create_error_response(
                StatusCode::BAD_REQUEST,
                "Message content cannot be empty",
            ))
        }
        Err(MessageError::RateLimit { limit, window }) => {
            warn!("Rate limit exceeded: {} per {}", limit, window);
            Err(create_error_response(
                StatusCode::TOO_MANY_REQUESTS,
                &format!("Rate limit exceeded: {} messages per {}", limit, window),
            ))
        }
        Err(err) => {
            error!("Failed to create message: {:?}", err);
            Err(create_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create message",
            ))
        }
    }
}

/// GET /api/rooms/:room_id/messages
/// 
/// Retrieves message history for the specified room with pagination
/// 
/// # Authentication
/// Requires valid session token via Authorization header or cookie
/// 
/// # Query Parameters
/// - `limit`: Number of messages to retrieve (default: 50, max: 100)
/// - `before`: MessageId to paginate before (optional)
/// 
/// # Response
/// - 200: Messages retrieved successfully
/// - 400: Invalid request (bad UUID, invalid limit)
/// - 401: Authentication required
/// - 403: User not authorized for room
/// - 500: Internal server error
pub async fn get_messages(
    State(state): State<AppState>,
    Path(room_id_str): Path<String>,
    Query(query): Query<GetMessagesQuery>,
    auth_user: AuthenticatedUser,
) -> Result<Response, Response> {
    info!(
        "Getting messages for room {} for user {}",
        room_id_str, auth_user.user.id
    );

    // Parse room_id from path parameter
    let room_id = parse_room_id(&room_id_str)?;

    // Parse and validate limit
    let limit = query.limit.unwrap_or(50);
    if limit > 100 {
        return Err(create_error_response(
            StatusCode::BAD_REQUEST,
            "Limit cannot exceed 100 messages",
        ));
    }

    // Parse before parameter if provided
    let before = if let Some(before_str) = query.before {
        Some(parse_message_id(&before_str)?)
    } else {
        None
    };

    // Use message service to get room messages
    match state
        .message_service
        .get_room_messages(room_id, auth_user.user.id, limit, before)
        .await
    {
        Ok(messages) => {
            info!("Retrieved {} messages for room {}", messages.len(), room_id);
            
            // Determine if there are more messages
            // This is a simple heuristic - if we got the full limit, there might be more
            let has_more = messages.len() as u32 == limit;
            
            Ok((
                StatusCode::OK,
                Json(MessagesResponse { messages, has_more }),
            ).into_response())
        }
        Err(MessageError::Authorization { user_id, room_id }) => {
            warn!("User {} not authorized for room {}", user_id, room_id);
            Err(create_error_response(
                StatusCode::FORBIDDEN,
                "You are not authorized to view messages in this room",
            ))
        }
        Err(err) => {
            error!("Failed to get messages: {:?}", err);
            Err(create_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve messages",
            ))
        }
    }
}

/// Parse room ID from string parameter
fn parse_room_id(room_id_str: &str) -> Result<RoomId, Response> {
    match Uuid::parse_str(room_id_str) {
        Ok(uuid) => Ok(RoomId(uuid)),
        Err(_) => Err(create_error_response(
            StatusCode::BAD_REQUEST,
            "Invalid room ID format",
        )),
    }
}

/// Parse message ID from string parameter
fn parse_message_id(message_id_str: &str) -> Result<MessageId, Response> {
    match Uuid::parse_str(message_id_str) {
        Ok(uuid) => Ok(MessageId(uuid)),
        Err(_) => Err(create_error_response(
            StatusCode::BAD_REQUEST,
            "Invalid message ID format",
        )),
    }
}

/// Create a standardized error response
fn create_error_response(status: StatusCode, message: &str) -> Response {
    let error_response = ErrorResponse {
        error: message.to_string(),
        code: status.as_u16(),
    };

    (status, Json(error_response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_parse_room_id_valid() {
        let uuid = Uuid::new_v4();
        let room_id = parse_room_id(&uuid.to_string()).unwrap();
        assert_eq!(room_id.0, uuid);
    }

    #[test]
    fn test_parse_room_id_invalid() {
        let result = parse_room_id("invalid-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_message_id_valid() {
        let uuid = Uuid::new_v4();
        let message_id = parse_message_id(&uuid.to_string()).unwrap();
        assert_eq!(message_id.0, uuid);
    }

    #[test]
    fn test_parse_message_id_invalid() {
        let result = parse_message_id("invalid-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_error_response() {
        let _response = create_error_response(StatusCode::BAD_REQUEST, "Test error");
        // We can't easily test the response body here without more setup,
        // but we can verify the function doesn't panic
        assert!(true);
    }
}