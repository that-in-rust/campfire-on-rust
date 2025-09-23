use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use uuid::Uuid;

use crate::errors::RoomError;
use crate::middleware::session::AuthenticatedUser;
use crate::models::{Room, RoomId};
use crate::validation::{CreateRoomRequest, AddRoomMemberRequest, sanitization, validate_request};
use crate::AppState;

/// GET /api/rooms
/// 
/// Returns list of rooms the current user has access to
/// 
/// # Authentication
/// Requires valid session token via Authorization header or cookie
/// 
/// # Response
/// - 200: JSON array of Room objects the user has access to
/// - 401: Invalid or missing authentication token
/// - 500: Internal server error
pub async fn get_rooms(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Room>>, RoomApiError> {
    // Get rooms for the authenticated user
    let rooms = state
        .room_service
        .get_user_rooms(auth_user.user.id)
        .await
        .map_err(RoomApiError::from)?;

    Ok(Json(rooms))
}

/// POST /api/rooms
/// 
/// Creates a new room with the authenticated user as admin
/// 
/// # Authentication
/// Requires valid session token via Authorization header or cookie
/// 
/// # Request Body
/// ```json
/// {
///   "name": "Room Name",
///   "topic": "Optional room topic",
///   "room_type": "Open" | "Closed" | "Direct"
/// }
/// ```
/// 
/// # Response
/// - 201: JSON Room object for the created room
/// - 400: Invalid request data (name too long, invalid room type, etc.)
/// - 401: Invalid or missing authentication token
/// - 500: Internal server error
pub async fn create_room(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(request): Json<CreateRoomRequest>,
) -> Result<(StatusCode, Json<Room>), RoomApiError> {
    // Validate request
    if let Err(validation_error) = validate_request(&request) {
        return Err(RoomApiError::ValidationError(validation_error));
    }
    
    // Sanitize input
    let name = sanitization::sanitize_room_name(&request.name);
    let topic = request.topic.map(|t| sanitization::sanitize_user_input(&t));
    
    // Parse room type
    let room_type = request.room_type.parse()
        .map_err(|_| RoomApiError::InvalidRoomType { room_type: request.room_type })?;
    
    // Create room using the room service
    let room = state
        .room_service
        .create_room(
            name,
            topic,
            room_type,
            auth_user.user.id,
        )
        .await
        .map_err(RoomApiError::from)?;

    Ok((StatusCode::CREATED, Json(room)))
}

/// GET /api/rooms/:id
/// 
/// Gets details for a specific room
/// 
/// # Authentication
/// Requires valid session token via Authorization header or cookie
/// 
/// # Path Parameters
/// - id: UUID of the room
/// 
/// # Response
/// - 200: JSON Room object
/// - 400: Invalid room ID format
/// - 401: Invalid or missing authentication token
/// - 403: User does not have access to this room
/// - 404: Room not found
/// - 500: Internal server error
pub async fn get_room(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(room_id_str): Path<String>,
) -> Result<Json<Room>, RoomApiError> {
    // Parse room ID
    let room_id = parse_room_id(&room_id_str)?;

    // Check if user has access to the room
    let access_level = state
        .room_service
        .check_room_access(room_id, auth_user.user.id)
        .await
        .map_err(RoomApiError::from)?;

    if access_level.is_none() {
        return Err(RoomApiError::AccessDenied { room_id });
    }

    // Get room details from database
    let room = state
        .db
        .get_room_by_id(room_id)
        .await
        .map_err(RoomApiError::from)?;

    match room {
        Some(room) => Ok(Json(room)),
        None => Err(RoomApiError::NotFound { room_id }),
    }
}

/// POST /api/rooms/:id/members
/// 
/// Adds a member to a room
/// 
/// # Authentication
/// Requires valid session token via Authorization header or cookie
/// User must be an admin of the room to add members
/// 
/// # Path Parameters
/// - id: UUID of the room
/// 
/// # Request Body
/// ```json
/// {
///   "user_id": "uuid-of-user-to-add",
///   "involvement_level": "Member" | "Admin"
/// }
/// ```
/// 
/// # Response
/// - 201: Member added successfully
/// - 400: Invalid request data or room ID format
/// - 401: Invalid or missing authentication token
/// - 403: User does not have permission to add members
/// - 404: Room or user not found
/// - 409: User is already a member of the room
/// - 500: Internal server error
pub async fn add_room_member(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(room_id_str): Path<String>,
    Json(request): Json<AddRoomMemberRequest>,
) -> Result<StatusCode, RoomApiError> {
    // Validate request
    if let Err(validation_error) = validate_request(&request) {
        return Err(RoomApiError::ValidationError(validation_error));
    }
    
    // Parse room ID
    let room_id = parse_room_id(&room_id_str)?;

    // Parse user ID and involvement level
    let user_id = request.user_id.into();
    let involvement_level = request.involvement_level.parse()
        .map_err(|_| RoomApiError::InvalidInvolvementLevel { level: request.involvement_level })?;

    // Add member using room service
    state
        .room_service
        .add_member(room_id, user_id, auth_user.user.id, involvement_level)
        .await
        .map_err(RoomApiError::from)?;

    Ok(StatusCode::CREATED)
}

/// Helper function to parse room ID from string
fn parse_room_id(room_id_str: &str) -> Result<RoomId, RoomApiError> {
    Uuid::parse_str(room_id_str)
        .map(RoomId::from)
        .map_err(|_| RoomApiError::InvalidRoomId {
            room_id: room_id_str.to_string(),
        })
}



/// Room API specific errors with proper HTTP status codes
#[derive(Debug)]
pub enum RoomApiError {
    InvalidRoomId { room_id: String },
    InvalidUserId { user_id: String },
    InvalidRoomType { room_type: String },
    InvalidInvolvementLevel { level: String },
    NotFound { room_id: RoomId },
    AccessDenied { room_id: RoomId },
    Database(sqlx::Error),
    RoomService(RoomError),
    ValidationError(crate::validation::ValidationErrorResponse),
}

impl From<RoomError> for RoomApiError {
    fn from(room_error: RoomError) -> Self {
        RoomApiError::RoomService(room_error)
    }
}

impl From<crate::errors::DatabaseError> for RoomApiError {
    fn from(db_error: crate::errors::DatabaseError) -> Self {
        match db_error {
            crate::errors::DatabaseError::Connection(sqlx_error) => RoomApiError::Database(sqlx_error),
            _ => RoomApiError::Database(sqlx::Error::RowNotFound), // Convert other errors to generic sqlx error
        }
    }
}

impl IntoResponse for RoomApiError {
    fn into_response(self) -> Response {
        let (status, error_message, error_code) = match self {
            RoomApiError::InvalidRoomId { room_id } => (
                StatusCode::BAD_REQUEST,
                format!("Invalid room ID format: {}", room_id),
                "INVALID_ROOM_ID",
            ),
            RoomApiError::InvalidUserId { user_id } => (
                StatusCode::BAD_REQUEST,
                format!("Invalid user ID format: {}", user_id),
                "INVALID_USER_ID",
            ),
            RoomApiError::InvalidRoomType { room_type } => (
                StatusCode::BAD_REQUEST,
                format!("Invalid room type: {}", room_type),
                "INVALID_ROOM_TYPE",
            ),
            RoomApiError::InvalidInvolvementLevel { level } => (
                StatusCode::BAD_REQUEST,
                format!("Invalid involvement level: {}", level),
                "INVALID_INVOLVEMENT_LEVEL",
            ),
            RoomApiError::ValidationError(validation_error) => {
                return validation_error.into_response();
            },
            RoomApiError::NotFound { room_id } => (
                StatusCode::NOT_FOUND,
                format!("Room not found: {}", room_id),
                "ROOM_NOT_FOUND",
            ),
            RoomApiError::AccessDenied { room_id } => (
                StatusCode::FORBIDDEN,
                format!("Access denied to room: {}", room_id),
                "ACCESS_DENIED",
            ),
            RoomApiError::Database(db_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", db_error),
                "DATABASE_ERROR",
            ),
            RoomApiError::RoomService(room_error) => match room_error {
                RoomError::NotFound { room_id } => (
                    StatusCode::NOT_FOUND,
                    format!("Room not found: {}", room_id),
                    "ROOM_NOT_FOUND",
                ),
                RoomError::NotAuthorized { user_id, room_id } => (
                    StatusCode::FORBIDDEN,
                    format!("User {} not authorized for room {}", user_id, room_id),
                    "NOT_AUTHORIZED",
                ),
                RoomError::AlreadyMember { user_id, room_id } => (
                    StatusCode::CONFLICT,
                    format!("User {} is already a member of room {}", user_id, room_id),
                    "ALREADY_MEMBER",
                ),
                RoomError::InvalidName { reason } => (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid room name: {}", reason),
                    "INVALID_ROOM_NAME",
                ),
                RoomError::Database(db_error) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", db_error),
                    "DATABASE_ERROR",
                ),
            },
        };

        let body = Json(json!({
            "error": error_message,
            "code": error_code,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}