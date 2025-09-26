use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    errors::AuthError,
    models::{ConnectionId, MessageId, UserId, WebSocketMessage},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    token: Option<String>,
}

/// Extract session token from headers (simplified version for WebSocket)
fn extract_session_token_from_headers(headers: &HeaderMap) -> Result<String, AuthError> {
    // Try Authorization header first
    if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if !token.is_empty() {
                    return Ok(token.to_string());
                }
            }
        }
    }
    
    // Try Cookie header
    if let Some(cookie_header) = headers.get(axum::http::header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(token) = cookie.strip_prefix("session_token=") {
                    if !token.is_empty() {
                        return Ok(token.to_string());
                    }
                }
            }
        }
    }
    
    Err(AuthError::SessionExpired)
}

/// WebSocket upgrade handler
/// 
/// Handles WebSocket connection upgrade with authentication
/// Supports authentication via:
/// 1. Query parameter: ?token=<session_token>
/// 2. Authorization header: "Bearer <token>"
/// 3. Cookie: "session_token=<token>"
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WebSocketQuery>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Extract session token from query params, headers, or cookies
    let token = if let Some(token) = params.token {
        token
    } else {
        // Try to extract from headers/cookies
        match extract_session_token_from_headers(&headers) {
            Ok(token) => token,
            Err(_) => {
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        }
    };

    // Validate session and get user
    let user = match state.auth_service.validate_session(token).await {
        Ok(user) => user,
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, "Invalid session").into_response();
        }
    };

    info!("WebSocket connection authenticated for user: {}", user.id.0);

    // Upgrade the connection
    ws.on_upgrade(move |socket| handle_websocket(socket, user.id, state))
}

/// Handle individual WebSocket connection
async fn handle_websocket(socket: WebSocket, user_id: UserId, state: AppState) {
    let connection_id = ConnectionId::new();
    
    info!("WebSocket connection established: {} for user: {}", 
          connection_id.0, user_id.0);

    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Create channel for outgoing messages
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Register connection with ConnectionManager
    if let Err(e) = state
        .message_service
        .connection_manager()
        .add_connection(user_id, connection_id, tx.clone())
        .await
    {
        error!("Failed to register WebSocket connection: {}", e);
        return;
    }

    // Spawn task to handle outgoing messages
    let outgoing_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = sender.send(Message::Text(msg)).await {
                warn!("Failed to send WebSocket message: {}", e);
                break;
            }
        }
    });

    // Handle incoming messages
    let state_clone = state.clone();
    let incoming_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = handle_incoming_message(
                        &text, 
                        user_id, 
                        connection_id, 
                        &state_clone
                    ).await {
                        error!("Error handling incoming WebSocket message: {}", e);
                    }
                }
                Ok(Message::Binary(_)) => {
                    warn!("Received binary WebSocket message, ignoring");
                }
                Ok(Message::Ping(data)) => {
                    // Respond to ping with pong
                    if let Err(e) = tx.send(format!("{{\"type\":\"pong\",\"data\":\"{}\"}}", 
                                                   base64::encode(&data))) {
                        warn!("Failed to send pong response: {}", e);
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Pong received, connection is alive
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed by client: {}", connection_id.0);
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = outgoing_task => {
            info!("Outgoing message task completed for connection: {}", connection_id.0);
        }
        _ = incoming_task => {
            info!("Incoming message task completed for connection: {}", connection_id.0);
        }
    }

    // Clean up connection
    if let Err(e) = state
        .message_service
        .connection_manager()
        .remove_connection(connection_id)
        .await
    {
        error!("Failed to remove WebSocket connection: {}", e);
    }

    info!("WebSocket connection closed: {} for user: {}", 
          connection_id.0, user_id.0);
}

/// Handle incoming WebSocket messages
async fn handle_incoming_message(
    text: &str,
    user_id: UserId,
    connection_id: ConnectionId,
    state: &AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse incoming message
    let incoming: IncomingWebSocketMessage = serde_json::from_str(text)?;

    match incoming {
        IncomingWebSocketMessage::CreateMessage { 
            room_id, 
            content, 
            client_message_id 
        } => {
            // Create message through MessageService
            match state
                .message_service
                .create_message_with_deduplication(
                    content,
                    room_id,
                    user_id,
                    client_message_id,
                )
                .await
            {
                Ok(message) => {
                    info!("Message created via WebSocket: {}", message.id.0);
                }
                Err(e) => {
                    error!("Failed to create message via WebSocket: {}", e);
                    
                    // Send error back to client
                    let error_msg = OutgoingWebSocketMessage::Error {
                        message: format!("Failed to create message: {}", e),
                        code: "MESSAGE_CREATION_FAILED".to_string(),
                    };
                    
                    if let Ok(serialized) = serde_json::to_string(&error_msg) {
                        // Try to send error back (best effort)
                        let _ = state
                            .message_service
                            .connection_manager()
                            .send_to_connection(connection_id, serialized)
                            .await;
                    }
                }
            }
        }
        IncomingWebSocketMessage::UpdateLastSeen { message_id } => {
            // Update last seen message for reconnection support (Critical Gap #2)
            if let Err(e) = state
                .message_service
                .connection_manager()
                .update_last_seen_message(connection_id, message_id)
                .await
            {
                warn!("Failed to update last seen message: {}", e);
            }
        }
        IncomingWebSocketMessage::JoinRoom { room_id } => {
            // Verify user has access to room
            match state
                .room_service
                .check_room_access(room_id, user_id)
                .await
            {
                Ok(Some(_)) => {
                    // User has access, send user joined notification
                    let presence_msg = WebSocketMessage::UserJoined {
                        user_id,
                        room_id,
                    };
                    
                    if let Err(e) = state
                        .message_service
                        .connection_manager()
                        .broadcast_to_room(room_id, presence_msg)
                        .await
                    {
                        warn!("Failed to broadcast user joined: {}", e);
                    }
                    
                    // Send updated presence information to room
                    if let Err(e) = state
                        .message_service
                        .connection_manager()
                        .broadcast_presence_update(room_id)
                        .await
                    {
                        warn!("Failed to broadcast presence update: {}", e);
                    }
                }
                Ok(None) => {
                    warn!("User {} attempted to join room {} without access", 
                          user_id.0, room_id.0);
                }
                Err(e) => {
                    error!("Error checking room access: {}", e);
                }
            }
        }
        IncomingWebSocketMessage::LeaveRoom { room_id } => {
            // Send user left notification
            let presence_msg = WebSocketMessage::UserLeft {
                user_id,
                room_id,
            };
            
            if let Err(e) = state
                .message_service
                .connection_manager()
                .broadcast_to_room(room_id, presence_msg)
                .await
            {
                warn!("Failed to broadcast user left: {}", e);
            }
            
            // Send updated presence information to room
            if let Err(e) = state
                .message_service
                .connection_manager()
                .broadcast_presence_update(room_id)
                .await
            {
                warn!("Failed to broadcast presence update: {}", e);
            }
        }
        IncomingWebSocketMessage::StartTyping { room_id } => {
            // Start typing indicator in connection manager
            if let Err(e) = state
                .message_service
                .connection_manager()
                .start_typing(user_id, room_id)
                .await
            {
                warn!("Failed to start typing indicator: {}", e);
            }
            
            // Send typing indicator to room
            let typing_msg = WebSocketMessage::TypingStart {
                user_id,
                room_id,
            };
            
            if let Err(e) = state
                .message_service
                .connection_manager()
                .broadcast_to_room(room_id, typing_msg)
                .await
            {
                warn!("Failed to broadcast typing start: {}", e);
            }
        }
        IncomingWebSocketMessage::StopTyping { room_id } => {
            // Stop typing indicator in connection manager
            if let Err(e) = state
                .message_service
                .connection_manager()
                .stop_typing(user_id, room_id)
                .await
            {
                warn!("Failed to stop typing indicator: {}", e);
            }
            
            // Send typing indicator to room
            let typing_msg = WebSocketMessage::TypingStop {
                user_id,
                room_id,
            };
            
            if let Err(e) = state
                .message_service
                .connection_manager()
                .broadcast_to_room(room_id, typing_msg)
                .await
            {
                warn!("Failed to broadcast typing stop: {}", e);
            }
        }
    }

    Ok(())
}

/// Incoming WebSocket message types (from client)
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
enum IncomingWebSocketMessage {
    CreateMessage {
        room_id: crate::models::RoomId,
        content: String,
        client_message_id: Uuid,
    },
    UpdateLastSeen {
        message_id: MessageId,
    },
    JoinRoom {
        room_id: crate::models::RoomId,
    },
    LeaveRoom {
        room_id: crate::models::RoomId,
    },
    StartTyping {
        room_id: crate::models::RoomId,
    },
    StopTyping {
        room_id: crate::models::RoomId,
    },
}

/// Outgoing WebSocket message types (to client)
#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
enum OutgoingWebSocketMessage {
    Error {
        message: String,
        code: String,
    },
    Pong {
        data: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        database::CampfireDatabase,
        models::RoomId,
        services::{AuthService, RoomService, MessageService},
        ConnectionManagerImpl,
    };
    use std::sync::Arc;
    use tokio::time::{timeout, Duration};

    async fn create_test_state() -> AppState {
        let db = CampfireDatabase::new(":memory:").await.unwrap();
        let db_arc = Arc::new(db.clone());
        
        let connection_manager = Arc::new(ConnectionManagerImpl::new(db_arc.clone()));
        let auth_service = Arc::new(AuthService::new(db_arc.clone()));
        let room_service = Arc::new(RoomService::new(db_arc.clone()));
        let message_service = Arc::new(MessageService::new(
            db_arc.clone(),
            connection_manager,
            room_service.clone(),
        ));
        let search_service = Arc::new(crate::SearchService::new(
            db_arc.clone(),
            room_service.clone(),
        ));
        
        // Create push notification service for testing
        let vapid_config = crate::VapidConfig::default();
        let push_service = Arc::new(crate::PushNotificationServiceImpl::new(
            db.clone(),
            db.writer(),
            vapid_config,
        ));
        
        let bot_service = Arc::new(crate::BotServiceImpl::new(
            db_arc.clone(),
            db.writer(),
            message_service.clone(),
        ));
        
        let setup_service = Arc::new(crate::services::setup::SetupServiceImpl::new(db.clone()));
        let demo_service = Arc::new(crate::services::demo::DemoServiceImpl::new(db_arc.clone()));
        
        AppState {
            db,
            auth_service,
            room_service,
            message_service,
            search_service,
            push_service,
            bot_service,
            setup_service,
            demo_service,
            analytics_store: Arc::new(crate::analytics::AnalyticsStore::new(100)),
        }
    }

    #[tokio::test]
    async fn test_websocket_message_parsing() {
        // Test incoming message parsing
        let create_msg = r#"{"type":"CreateMessage","room_id":"550e8400-e29b-41d4-a716-446655440000","content":"Hello World","client_message_id":"550e8400-e29b-41d4-a716-446655440001"}"#;
        
        let parsed: Result<IncomingWebSocketMessage, _> = serde_json::from_str(create_msg);
        assert!(parsed.is_ok());
        
        if let Ok(IncomingWebSocketMessage::CreateMessage { content, .. }) = parsed {
            assert_eq!(content, "Hello World");
        } else {
            panic!("Failed to parse CreateMessage");
        }
    }

    #[tokio::test]
    async fn test_outgoing_message_serialization() {
        let error_msg = OutgoingWebSocketMessage::Error {
            message: "Test error".to_string(),
            code: "TEST_ERROR".to_string(),
        };
        
        let serialized = serde_json::to_string(&error_msg).unwrap();
        assert!(serialized.contains("Test error"));
        assert!(serialized.contains("TEST_ERROR"));
    }

    #[tokio::test]
    async fn test_handle_incoming_message() {
        let state = create_test_state().await;
        let user_id = UserId::new();
        let connection_id = ConnectionId::new();
        
        // Test UpdateLastSeen message
        let msg = r#"{"type":"UpdateLastSeen","message_id":"550e8400-e29b-41d4-a716-446655440000"}"#;
        
        // This should not panic or error
        let result = timeout(
            Duration::from_secs(1),
            handle_incoming_message(msg, user_id, connection_id, &state)
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_typing_message_handling() {
        let state = create_test_state().await;
        let user_id = UserId::new();
        let connection_id = ConnectionId::new();
        let room_id = RoomId::new();
        
        // Test StartTyping message
        let start_msg = format!(
            r#"{{"type":"StartTyping","room_id":"{}"}}"#,
            room_id.0
        );
        
        let result = timeout(
            Duration::from_secs(1),
            handle_incoming_message(&start_msg, user_id, connection_id, &state)
        ).await;
        
        assert!(result.is_ok());
        
        // Test StopTyping message
        let stop_msg = format!(
            r#"{{"type":"StopTyping","room_id":"{}"}}"#,
            room_id.0
        );
        
        let result = timeout(
            Duration::from_secs(1),
            handle_incoming_message(&stop_msg, user_id, connection_id, &state)
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_join_leave_room_messages() {
        let state = create_test_state().await;
        let user_id = UserId::new();
        let connection_id = ConnectionId::new();
        let room_id = RoomId::new();
        
        // Test JoinRoom message
        let join_msg = format!(
            r#"{{"type":"JoinRoom","room_id":"{}"}}"#,
            room_id.0
        );
        
        let result = timeout(
            Duration::from_secs(1),
            handle_incoming_message(&join_msg, user_id, connection_id, &state)
        ).await;
        
        assert!(result.is_ok());
        
        // Test LeaveRoom message
        let leave_msg = format!(
            r#"{{"type":"LeaveRoom","room_id":"{}"}}"#,
            room_id.0
        );
        
        let result = timeout(
            Duration::from_secs(1),
            handle_incoming_message(&leave_msg, user_id, connection_id, &state)
        ).await;
        
        assert!(result.is_ok());
    }
}