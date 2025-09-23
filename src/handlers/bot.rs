use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::errors::BotError;
use crate::middleware::session::AuthenticatedUser;
use crate::models::*;
use crate::validation::{CreateBotRequest, CreateBotMessageRequest, sanitization, validate_request};
use crate::AppState;

/// GET /api/bots
/// 
/// List all active bots (admin only)
/// 
/// # Authentication
/// Requires valid session token and admin privileges
/// 
/// # Response
/// - 200 OK: Returns list of bots
/// - 401 Unauthorized: Invalid or missing session token
/// - 403 Forbidden: User is not an admin
/// - 500 Internal Server Error: Server error
pub async fn list_bots(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Response {
    // Check admin privileges
    if !auth_user.user.admin {
        warn!("Non-admin user {} attempted to list bots", auth_user.user.id);
        return create_error_response(
            StatusCode::FORBIDDEN,
            "Admin privileges required",
            "INSUFFICIENT_PRIVILEGES"
        );
    }
    
    match state.bot_service.list_bots().await {
        Ok(bots) => {
            info!("Listed {} bots for admin {}", bots.len(), auth_user.user.id);
            (StatusCode::OK, Json(json!({
                "bots": bots,
                "success": true
            }))).into_response()
        }
        Err(bot_error) => {
            error!("Failed to list bots: {}", bot_error);
            bot_error_to_response(bot_error)
        }
    }
}

/// POST /api/bots
/// 
/// Create a new bot (admin only)
/// 
/// # Request Body
/// ```json
/// {
///   "name": "My Bot",
///   "webhook_url": "https://example.com/webhook" // optional
/// }
/// ```
/// 
/// # Authentication
/// Requires valid session token and admin privileges
/// 
/// # Response
/// - 201 Created: Bot created successfully
/// - 400 Bad Request: Invalid request format or bot name
/// - 401 Unauthorized: Invalid or missing session token
/// - 403 Forbidden: User is not an admin
/// - 500 Internal Server Error: Server error
pub async fn create_bot(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(request): Json<CreateBotRequest>,
) -> Response {
    // Check admin privileges
    if !auth_user.user.admin {
        warn!("Non-admin user {} attempted to create bot", auth_user.user.id);
        return create_error_response(
            StatusCode::FORBIDDEN,
            "Admin privileges required",
            "INSUFFICIENT_PRIVILEGES"
        );
    }
    
    // Validate request
    if let Err(validation_error) = validate_request(&request) {
        return validation_error.into_response();
    }
    
    // Sanitize input
    let name = sanitization::sanitize_user_input(&request.name);
    let _description = request.description.map(|d| sanitization::sanitize_user_input(&d));
    let webhook_url = request.webhook_url.map(|url| sanitization::sanitize_user_input(&url));
    
    info!("Creating bot '{}' for admin {}", name, auth_user.user.id);
    
    match state.bot_service.create_bot(name, webhook_url).await {
        Ok(bot) => {
            info!("Created bot: {} ({})", bot.name, bot.id);
            (StatusCode::CREATED, Json(json!({
                "bot": bot,
                "success": true
            }))).into_response()
        }
        Err(bot_error) => {
            error!("Failed to create bot: {}", bot_error);
            bot_error_to_response(bot_error)
        }
    }
}

/// GET /api/bots/:id
/// 
/// Get bot details (admin only)
/// 
/// # Authentication
/// Requires valid session token and admin privileges
/// 
/// # Response
/// - 200 OK: Returns bot details
/// - 401 Unauthorized: Invalid or missing session token
/// - 403 Forbidden: User is not an admin
/// - 404 Not Found: Bot not found
/// - 500 Internal Server Error: Server error
pub async fn get_bot(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(bot_id): Path<Uuid>,
) -> Response {
    // Check admin privileges
    if !auth_user.user.admin {
        warn!("Non-admin user {} attempted to get bot {}", auth_user.user.id, bot_id);
        return create_error_response(
            StatusCode::FORBIDDEN,
            "Admin privileges required",
            "INSUFFICIENT_PRIVILEGES"
        );
    }
    
    let bot_user_id = UserId(bot_id);
    
    match state.bot_service.get_bot(bot_user_id).await {
        Ok(Some(bot)) => {
            (StatusCode::OK, Json(json!({
                "bot": bot,
                "success": true
            }))).into_response()
        }
        Ok(None) => {
            create_error_response(
                StatusCode::NOT_FOUND,
                "Bot not found",
                "BOT_NOT_FOUND"
            )
        }
        Err(bot_error) => {
            error!("Failed to get bot {}: {}", bot_id, bot_error);
            bot_error_to_response(bot_error)
        }
    }
}

/// PUT /api/bots/:id
/// 
/// Update bot details (admin only)
/// 
/// # Request Body
/// ```json
/// {
///   "name": "Updated Bot Name", // optional
///   "webhook_url": "https://example.com/new-webhook" // optional, empty string to remove
/// }
/// ```
/// 
/// # Authentication
/// Requires valid session token and admin privileges
/// 
/// # Response
/// - 200 OK: Bot updated successfully
/// - 400 Bad Request: Invalid request format
/// - 401 Unauthorized: Invalid or missing session token
/// - 403 Forbidden: User is not an admin
/// - 404 Not Found: Bot not found
/// - 500 Internal Server Error: Server error
pub async fn update_bot(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(bot_id): Path<Uuid>,
    Json(request): Json<UpdateBotRequest>,
) -> Response {
    // Check admin privileges
    if !auth_user.user.admin {
        warn!("Non-admin user {} attempted to update bot {}", auth_user.user.id, bot_id);
        return create_error_response(
            StatusCode::FORBIDDEN,
            "Admin privileges required",
            "INSUFFICIENT_PRIVILEGES"
        );
    }
    
    let bot_user_id = UserId(bot_id);
    
    info!("Updating bot {} for admin {}", bot_id, auth_user.user.id);
    
    match state.bot_service.update_bot(
        bot_user_id,
        request.name,
        request.webhook_url,
    ).await {
        Ok(bot) => {
            info!("Updated bot: {} ({})", bot.name, bot.id);
            (StatusCode::OK, Json(json!({
                "bot": bot,
                "success": true
            }))).into_response()
        }
        Err(bot_error) => {
            error!("Failed to update bot {}: {}", bot_id, bot_error);
            bot_error_to_response(bot_error)
        }
    }
}

/// DELETE /api/bots/:id
/// 
/// Delete (deactivate) a bot (admin only)
/// 
/// # Authentication
/// Requires valid session token and admin privileges
/// 
/// # Response
/// - 200 OK: Bot deleted successfully
/// - 401 Unauthorized: Invalid or missing session token
/// - 403 Forbidden: User is not an admin
/// - 404 Not Found: Bot not found
/// - 500 Internal Server Error: Server error
pub async fn delete_bot(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(bot_id): Path<Uuid>,
) -> Response {
    // Check admin privileges
    if !auth_user.user.admin {
        warn!("Non-admin user {} attempted to delete bot {}", auth_user.user.id, bot_id);
        return create_error_response(
            StatusCode::FORBIDDEN,
            "Admin privileges required",
            "INSUFFICIENT_PRIVILEGES"
        );
    }
    
    let bot_user_id = UserId(bot_id);
    
    info!("Deleting bot {} for admin {}", bot_id, auth_user.user.id);
    
    match state.bot_service.delete_bot(bot_user_id).await {
        Ok(()) => {
            info!("Deleted bot: {}", bot_id);
            (StatusCode::OK, Json(json!({
                "message": "Bot deleted successfully",
                "success": true
            }))).into_response()
        }
        Err(bot_error) => {
            error!("Failed to delete bot {}: {}", bot_id, bot_error);
            bot_error_to_response(bot_error)
        }
    }
}

/// POST /api/bots/:id/reset-token
/// 
/// Reset bot API token (admin only)
/// 
/// # Authentication
/// Requires valid session token and admin privileges
/// 
/// # Response
/// - 200 OK: Token reset successfully, returns new bot key
/// - 401 Unauthorized: Invalid or missing session token
/// - 403 Forbidden: User is not an admin
/// - 404 Not Found: Bot not found
/// - 500 Internal Server Error: Server error
pub async fn reset_bot_token(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(bot_id): Path<Uuid>,
) -> Response {
    // Check admin privileges
    if !auth_user.user.admin {
        warn!("Non-admin user {} attempted to reset bot token {}", auth_user.user.id, bot_id);
        return create_error_response(
            StatusCode::FORBIDDEN,
            "Admin privileges required",
            "INSUFFICIENT_PRIVILEGES"
        );
    }
    
    let bot_user_id = UserId(bot_id);
    
    info!("Resetting bot token {} for admin {}", bot_id, auth_user.user.id);
    
    match state.bot_service.reset_bot_token(bot_user_id).await {
        Ok(new_token) => {
            let new_bot_key = format!("{}-{}", bot_id, new_token);
            info!("Reset bot token: {}", bot_id);
            (StatusCode::OK, Json(json!({
                "bot_key": new_bot_key,
                "message": "Bot token reset successfully",
                "success": true
            }))).into_response()
        }
        Err(bot_error) => {
            error!("Failed to reset bot token {}: {}", bot_id, bot_error);
            bot_error_to_response(bot_error)
        }
    }
}

/// POST /rooms/:room_id/bot/:bot_key/messages
/// 
/// Create a message from a bot (bot API endpoint)
/// 
/// # Request Body
/// Plain text body or JSON:
/// ```json
/// {
///   "body": "Hello from bot!"
/// }
/// ```
/// 
/// # Authentication
/// Uses bot_key in URL path for authentication
/// 
/// # Response
/// - 201 Created: Message created successfully
/// - 400 Bad Request: Invalid request format
/// - 401 Unauthorized: Invalid bot key
/// - 403 Forbidden: Bot not authorized for room
/// - 404 Not Found: Room not found
/// - 500 Internal Server Error: Server error
pub async fn create_bot_message(
    State(state): State<AppState>,
    Path((room_id, bot_key)): Path<(Uuid, String)>,
    Json(request): Json<CreateBotMessageRequest>,
) -> Response {
    let room_id = RoomId(room_id);
    
    info!("Bot message creation attempt for room {} with key {}", room_id, bot_key);
    
    // Authenticate bot
    let bot_user = match state.bot_service.authenticate_bot(&bot_key).await {
        Ok(user) => user,
        Err(BotError::InvalidToken) => {
            warn!("Invalid bot key used: {}", bot_key);
            return create_error_response(
                StatusCode::UNAUTHORIZED,
                "Invalid bot key",
                "INVALID_BOT_KEY"
            );
        }
        Err(e) => {
            error!("Bot authentication error: {}", e);
            return create_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Authentication error",
                "AUTH_ERROR"
            );
        }
    };
    
    // Validate request
    if let Err(validation_error) = validate_request(&request) {
        return validation_error.into_response();
    }
    
    // Sanitize message content
    let content = sanitization::sanitize_message_content(&request.content);
    
    // Create message
    match state.bot_service.create_bot_message(bot_user.id, room_id, content).await {
        Ok(message) => {
            info!("Bot {} created message {} in room {}", bot_user.id, message.id, room_id);
            
            // Return location header like Rails
            let location = format!("/rooms/{}/messages/{}", room_id.0, message.id.0);
            
            (
                StatusCode::CREATED,
                [("Location", location.as_str())],
                Json(json!({
                    "message": message,
                    "success": true
                }))
            ).into_response()
        }
        Err(bot_error) => {
            error!("Failed to create bot message: {}", bot_error);
            bot_error_to_response(bot_error)
        }
    }
}

/// Converts BotError to appropriate HTTP response
fn bot_error_to_response(error: BotError) -> Response {
    let (status, message, code) = match error {
        BotError::InvalidToken => (
            StatusCode::UNAUTHORIZED,
            "Invalid bot token",
            "INVALID_BOT_TOKEN"
        ),
        BotError::NotFound { .. } => (
            StatusCode::NOT_FOUND,
            "Bot not found",
            "BOT_NOT_FOUND"
        ),
        BotError::NotABot { .. } => (
            StatusCode::FORBIDDEN,
            "User is not a bot",
            "NOT_A_BOT"
        ),
        BotError::TokenExists => (
            StatusCode::CONFLICT,
            "Bot token already exists",
            "TOKEN_EXISTS"
        ),
        BotError::InvalidWebhookUrl { .. } => (
            StatusCode::BAD_REQUEST,
            "Invalid webhook URL",
            "INVALID_WEBHOOK_URL"
        ),
        BotError::InvalidName { .. } => (
            StatusCode::BAD_REQUEST,
            "Invalid bot name",
            "INVALID_BOT_NAME"
        ),
        BotError::WebhookDeliveryFailed { .. } 
        | BotError::WebhookTimeout { .. }
        | BotError::Database(_)
        | BotError::HttpRequest(_)
        | BotError::JsonSerialization(_) => {
            error!("Internal bot error: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
                "INTERNAL_ERROR"
            )
        }
    };
    
    create_error_response(status, message, code)
}

/// Creates a standardized error response
fn create_error_response(status: StatusCode, message: &str, code: &str) -> Response {
    let error_body = json!({
        "error": {
            "message": message,
            "code": code,
            "status": status.as_u16()
        },
        "success": false
    });
    
    (status, Json(error_body)).into_response()
}