use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use crate::errors::PushNotificationError;
use crate::models::*;
use crate::services::PushNotificationService;
use std::sync::Arc;

/// Create a new push subscription
pub async fn create_push_subscription(
    State(app_state): State<crate::AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<CreatePushSubscriptionRequest>,
) -> Result<Json<PushSubscription>, (StatusCode, Json<serde_json::Value>)> {
    let push_service = &app_state.push_service;
    match push_service.create_subscription(user.id, request).await {
        Ok(subscription) => Ok(Json(subscription)),
        Err(e) => {
            tracing::error!("Failed to create push subscription: {:?}", e);
            let error_msg = e.to_string();
            let status = StatusCode::from(e);
            Err((
                status,
                Json(serde_json::json!({
                    "error": "Failed to create push subscription",
                    "details": error_msg
                })),
            ))
        }
    }
}

/// Delete a push subscription
pub async fn delete_push_subscription(
    State(app_state): State<crate::AppState>,
    Extension(_user): Extension<User>,
    Path(subscription_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let push_service = &app_state.push_service;
    let subscription_id = match uuid::Uuid::parse_str(&subscription_id) {
        Ok(id) => PushSubscriptionId(id),
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid subscription ID format"
                })),
            ));
        }
    };
    
    match push_service.delete_subscription(subscription_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to delete push subscription: {:?}", e);
            let error_msg = e.to_string();
            let status = StatusCode::from(e);
            Err((
                status,
                Json(serde_json::json!({
                    "error": "Failed to delete push subscription",
                    "details": error_msg
                })),
            ))
        }
    }
}

/// Get notification preferences for the current user
pub async fn get_notification_preferences(
    State(app_state): State<crate::AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<NotificationPreferences>, (StatusCode, Json<serde_json::Value>)> {
    let push_service = &app_state.push_service;
    match push_service.get_preferences(user.id).await {
        Ok(preferences) => Ok(Json(preferences)),
        Err(e) => {
            tracing::error!("Failed to get notification preferences: {:?}", e);
            let error_msg = e.to_string();
            let status = StatusCode::from(e);
            Err((
                status,
                Json(serde_json::json!({
                    "error": "Failed to get notification preferences",
                    "details": error_msg
                })),
            ))
        }
    }
}

/// Update notification preferences for the current user
pub async fn update_notification_preferences(
    State(app_state): State<crate::AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<UpdateNotificationPreferencesRequest>,
) -> Result<Json<NotificationPreferences>, (StatusCode, Json<serde_json::Value>)> {
    let push_service = &app_state.push_service;
    match push_service.update_preferences(user.id, request).await {
        Ok(preferences) => Ok(Json(preferences)),
        Err(e) => {
            tracing::error!("Failed to update notification preferences: {:?}", e);
            let error_msg = e.to_string();
            let status = StatusCode::from(e);
            Err((
                status,
                Json(serde_json::json!({
                    "error": "Failed to update notification preferences",
                    "details": error_msg
                })),
            ))
        }
    }
}

/// Get VAPID public key for client-side subscription
pub async fn get_vapid_public_key() -> Json<serde_json::Value> {
    // In production, this should come from the actual VAPID configuration
    Json(serde_json::json!({
        "publicKey": "YOUR_VAPID_PUBLIC_KEY"
    }))
}

/// Test endpoint to send a test notification
#[cfg(debug_assertions)]
pub async fn send_test_notification(
    State(app_state): State<crate::AppState>,
    Extension(user): Extension<User>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let push_service = &app_state.push_service;
    // Create a test message and room for demonstration
    let test_message = Message::new(
        RoomId::new(),
        user.id,
        "This is a test notification".to_string(),
        uuid::Uuid::new_v4(),
    );
    
    let test_room = Room {
        id: RoomId::new(),
        name: "Test Room".to_string(),
        topic: None,
        room_type: RoomType::Open,
        created_at: chrono::Utc::now(),
        last_message_at: None,
    };
    
    match push_service.send_message_notification(&test_message, &test_room, &user.name).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("Failed to send test notification: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to send test notification",
                    "details": e.to_string()
                })),
            ))
        }
    }
}