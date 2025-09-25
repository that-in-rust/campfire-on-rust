use axum::{
    extract::{Path, Query, State, ConnectInfo},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::errors::MessageError;
use crate::middleware::AuthenticatedUser;
use crate::models::{Message, MessageId, RoomId};
use crate::validation::{CreateMessageRequest, sanitization, validate_request};
use crate::logging::{audit::{AuditAction, AuditLogger}, error_handling::handle_message_error};
use crate::{AppState, log_performance_warning, log_business_event};

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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    auth_user: AuthenticatedUser,
    Json(request): Json<CreateMessageRequest>,
) -> Result<Response, Response> {
    let start_time = Instant::now();
    let ip_address = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    let audit_logger = AuditLogger::new(true); // TODO: Get from config

    info!(
        "Creating message in room {} for user {} from IP: {}",
        room_id_str, auth_user.user.id, ip_address
    );

    // Validate request
    if let Err(validation_error) = validate_request(&request) {
        // Log validation failure
        let mut details = HashMap::new();
        details.insert("room_id".to_string(), room_id_str.clone());
        details.insert("user_id".to_string(), auth_user.user.id.to_string());
        details.insert("error".to_string(), "validation_failed".to_string());
        
        audit_logger.log_security_event(
            AuditAction::MessageCreated,
            Some(auth_user.user.id),
            Some(&ip_address),
            details,
        );
        
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
            content.clone(),
            room_id,
            auth_user.user.id,
            request.client_message_id,
        )
        .await
    {
        Ok(message) => {
            let duration = start_time.elapsed();
            
            // Check for performance issues
            if duration.as_millis() > 1000 {
                log_performance_warning!("message_creation", duration, std::time::Duration::from_millis(1000));
            }
            
            info!("Message created successfully: {} in {:?}", message.id, duration);
            
            // Log successful message creation
            let mut details = HashMap::new();
            details.insert("message_id".to_string(), message.id.to_string());
            details.insert("room_id".to_string(), room_id.to_string());
            details.insert("content_length".to_string(), content.len().to_string());
            details.insert("client_message_id".to_string(), request.client_message_id.to_string());
            details.insert("duration_ms".to_string(), duration.as_millis().to_string());
            
            audit_logger.log_user_action(
                AuditAction::MessageCreated,
                auth_user.user.id,
                "message",
                Some(message.id.to_string()),
                details,
            );
            
            // Log business event for analytics
            log_business_event!("message_sent", auth_user.user.id, format!("room:{}, length:{}", room_id, content.len()));
            
            Ok((
                StatusCode::CREATED,
                Json(MessageResponse { message }),
            ).into_response())
        }
        Err(message_error) => {
            let duration = start_time.elapsed();
            
            // Log failed message creation
            let mut details = HashMap::new();
            details.insert("room_id".to_string(), room_id.to_string());
            details.insert("user_id".to_string(), auth_user.user.id.to_string());
            details.insert("error".to_string(), message_error.to_string());
            details.insert("duration_ms".to_string(), duration.as_millis().to_string());
            details.insert("user_agent".to_string(), user_agent.to_string());
            
            // Determine if this is a security event
            let is_security_event = matches!(
                message_error,
                MessageError::Authorization { .. } | MessageError::RateLimit { .. }
            );
            
            if is_security_event {
                audit_logger.log_security_event(
                    AuditAction::MessageCreated,
                    Some(auth_user.user.id),
                    Some(&ip_address),
                    details,
                );
            } else {
                audit_logger.log_user_action(
                    AuditAction::MessageCreated,
                    auth_user.user.id,
                    "message",
                    None::<String>,
                    details,
                );
            }
            
            warn!("Failed to create message: {:?} in {:?}", message_error, duration);
            
            Err(handle_message_error(message_error, Some("create_message")).into_response())
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    auth_user: AuthenticatedUser,
) -> Result<Response, Response> {
    let start_time = Instant::now();
    let ip_address = addr.ip().to_string();
    
    info!(
        "Getting messages for room {} for user {} from IP: {}",
        room_id_str, auth_user.user.id, ip_address
    );

    // Parse room_id from path parameter
    let room_id = parse_room_id(&room_id_str)?;

    // Parse and validate limit
    let limit = query.limit.unwrap_or(50);
    if limit > 100 {
        return Err(handle_message_error(
            MessageError::InvalidContent { 
                reason: "Limit cannot exceed 100 messages".to_string() 
            },
            Some("get_messages")
        ).into_response());
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
            let duration = start_time.elapsed();
            
            // Check for performance issues
            if duration.as_millis() > 500 {
                log_performance_warning!("message_retrieval", duration, std::time::Duration::from_millis(500));
            }
            
            info!("Retrieved {} messages for room {} in {:?}", messages.len(), room_id, duration);
            
            // Log message retrieval for audit (only for large requests or slow queries)
            if limit > 50 || duration.as_millis() > 1000 {
                let audit_logger = AuditLogger::new(true);
                let mut details = HashMap::new();
                details.insert("room_id".to_string(), room_id.to_string());
                details.insert("limit".to_string(), limit.to_string());
                details.insert("messages_returned".to_string(), messages.len().to_string());
                details.insert("duration_ms".to_string(), duration.as_millis().to_string());
                
                audit_logger.log_user_action(
                    AuditAction::MessageCreated, // Using MessageCreated as closest match
                    auth_user.user.id,
                    "message_query",
                    Some(room_id.to_string()),
                    details,
                );
            }
            
            // Determine if there are more messages
            // This is a simple heuristic - if we got the full limit, there might be more
            let has_more = messages.len() as u32 == limit;
            
            Ok((
                StatusCode::OK,
                Json(MessagesResponse { messages, has_more }),
            ).into_response())
        }
        Err(message_error) => {
            let duration = start_time.elapsed();
            
            warn!("Failed to get messages: {:?} in {:?}", message_error, duration);
            
            // Log failed message retrieval for security events
            if matches!(message_error, MessageError::Authorization { .. }) {
                let audit_logger = AuditLogger::new(true);
                let mut details = HashMap::new();
                details.insert("room_id".to_string(), room_id.to_string());
                details.insert("error".to_string(), message_error.to_string());
                details.insert("duration_ms".to_string(), duration.as_millis().to_string());
                
                audit_logger.log_security_event(
                    AuditAction::UnauthorizedAccess,
                    Some(auth_user.user.id),
                    Some(&ip_address),
                    details,
                );
            }
            
            Err(handle_message_error(message_error, Some("get_messages")).into_response())
        }
    }
}

/// Parse room ID from string parameter
fn parse_room_id(room_id_str: &str) -> Result<RoomId, Response> {
    match Uuid::parse_str(room_id_str) {
        Ok(uuid) => Ok(RoomId(uuid)),
        Err(_) => Err(handle_message_error(
            MessageError::InvalidContent { 
                reason: "Invalid room ID format".to_string() 
            },
            Some("parse_room_id")
        ).into_response()),
    }
}

/// Parse message ID from string parameter
fn parse_message_id(message_id_str: &str) -> Result<MessageId, Response> {
    match Uuid::parse_str(message_id_str) {
        Ok(uuid) => Ok(MessageId(uuid)),
        Err(_) => Err(handle_message_error(
            MessageError::InvalidContent { 
                reason: "Invalid message ID format".to_string() 
            },
            Some("parse_message_id")
        ).into_response()),
    }
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
    fn test_error_handling() {
        // Test that error handling functions work correctly
        let error = MessageError::InvalidContent { 
            reason: "Test error".to_string() 
        };
        let user_friendly = handle_message_error(error, Some("test"));
        assert_eq!(user_friendly.status, StatusCode::BAD_REQUEST);
        assert_eq!(user_friendly.code, "INVALID_CONTENT");
    }
}