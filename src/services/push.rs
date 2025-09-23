use crate::database::{CampfireDatabase, DatabaseWriter};
use crate::errors::PushNotificationError;
use crate::models::*;
use crate::validation::CreatePushSubscriptionRequest;
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use web_push::{
    WebPushClient, WebPushMessage, WebPushMessageBuilder, VapidSignatureBuilder, SubscriptionInfo,
};
use sqlx::Row;

/// Push notification service trait
#[async_trait]
pub trait PushNotificationService: Send + Sync {
    /// Create a new push subscription for a user
    async fn create_subscription(
        &self,
        user_id: UserId,
        request: CreatePushSubscriptionRequest,
    ) -> Result<PushSubscription, PushNotificationError>;
    
    /// Delete a push subscription
    async fn delete_subscription(
        &self,
        subscription_id: PushSubscriptionId,
    ) -> Result<(), PushNotificationError>;
    
    /// Update notification preferences for a user
    async fn update_preferences(
        &self,
        user_id: UserId,
        request: UpdateNotificationPreferencesRequest,
    ) -> Result<NotificationPreferences, PushNotificationError>;
    
    /// Get notification preferences for a user
    async fn get_preferences(
        &self,
        user_id: UserId,
    ) -> Result<NotificationPreferences, PushNotificationError>;
    
    /// Send push notification for a new message
    async fn send_message_notification(
        &self,
        message: &Message,
        room: &Room,
        sender_name: &str,
    ) -> Result<(), PushNotificationError>;
    
    /// Send push notification for a mention
    async fn send_mention_notification(
        &self,
        message: &Message,
        room: &Room,
        sender_name: &str,
        mentioned_user: UserId,
    ) -> Result<(), PushNotificationError>;
    
    /// Send push notification for sound playback
    async fn send_sound_notification(
        &self,
        sound_name: &str,
        room: &Room,
        triggered_by_name: &str,
    ) -> Result<(), PushNotificationError>;
}

/// VAPID configuration for Web Push
#[derive(Debug, Clone)]
pub struct VapidConfig {
    pub private_key: String,
    pub public_key: String,
    pub subject: String, // Usually "mailto:your-email@example.com"
}

impl Default for VapidConfig {
    fn default() -> Self {
        // In production, these should be loaded from environment variables
        Self {
            private_key: "YOUR_VAPID_PRIVATE_KEY".to_string(),
            public_key: "YOUR_VAPID_PUBLIC_KEY".to_string(),
            subject: "mailto:admin@campfire.example.com".to_string(),
        }
    }
}

/// Push notification service implementation
pub struct PushNotificationServiceImpl {
    database: CampfireDatabase,
    writer: Arc<dyn DatabaseWriter>,
    vapid_config: VapidConfig,
    client: WebPushClient,
}

impl PushNotificationServiceImpl {
    pub fn new(
        database: CampfireDatabase,
        writer: Arc<dyn DatabaseWriter>,
        vapid_config: VapidConfig,
    ) -> Self {
        let client = WebPushClient::new().expect("Failed to create WebPush client");
        
        Self {
            database,
            writer,
            vapid_config,
            client,
        }
    }
    
    /// Send a push notification to a specific subscription
    async fn send_push_notification(
        &self,
        subscription: &PushSubscription,
        payload: &PushNotificationPayload,
    ) -> Result<(), PushNotificationError> {
        // Create subscription info for web-push
        let subscription_info = SubscriptionInfo::new(
            &subscription.endpoint,
            &subscription.p256dh_key,
            &subscription.auth_key,
        );
        
        // Create VAPID signature
        let mut sig_builder = VapidSignatureBuilder::from_pem(
            self.vapid_config.private_key.as_bytes(),
            &subscription_info,
        )?;
        sig_builder.add_claim("sub", self.vapid_config.subject.as_str());
        let signature = sig_builder.build()?;
        
        // Create message
        let payload_json = serde_json::to_string(payload)?;
        let mut message_builder = WebPushMessageBuilder::new(&subscription_info)?;
        message_builder.set_payload(web_push::ContentEncoding::Aes128Gcm, payload_json.as_bytes());
        message_builder.set_vapid_signature(signature);
        let message = message_builder.build()?;
        
        // Send the notification
        match self.client.send(message).await {
            Ok(_) => {
                // Update last_used_at for the subscription
                let mut updated_subscription = subscription.clone();
                updated_subscription.last_used_at = Some(Utc::now());
                let _ = self.writer.create_push_subscription(updated_subscription).await;
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Failed to send push notification: {:?}", e);
                // If the subscription is invalid, we might want to delete it
                // For now, just return the error
                Err(PushNotificationError::SendFailed(e.to_string()))
            }
        }
    }
    
    /// Determine notification type based on message and room context
    fn determine_notification_type(
        &self,
        message: &Message,
        room: &Room,
        user_id: UserId,
    ) -> NotificationType {
        if room.room_type == RoomType::Direct {
            NotificationType::DirectMessage
        } else if message.mentions.iter().any(|mention| {
            // This is a simplified check - in practice you'd resolve mentions to user IDs
            mention.contains(&user_id.to_string())
        }) {
            NotificationType::Mention
        } else {
            NotificationType::NewMessage
        }
    }
    
    /// Create notification payload based on type and context
    fn create_notification_payload(
        &self,
        notification_type: NotificationType,
        message: &Message,
        room: &Room,
        sender_name: &str,
    ) -> PushNotificationPayload {
        let (title, body) = match notification_type {
            NotificationType::DirectMessage => (
                format!("Direct message from {}", sender_name),
                message.content.clone(),
            ),
            NotificationType::Mention => (
                format!("{} mentioned you in {}", sender_name, room.name),
                message.content.clone(),
            ),
            NotificationType::NewMessage => (
                format!("New message in {}", room.name),
                format!("{}: {}", sender_name, message.content),
            ),
            NotificationType::SoundPlayback => (
                format!("Sound played in {}", room.name),
                format!("{} played a sound", sender_name),
            ),
        };
        
        let data = serde_json::json!({
            "messageId": message.id,
            "roomId": message.room_id,
            "senderId": message.creator_id,
            "type": match notification_type {
                NotificationType::DirectMessage => "direct_message",
                NotificationType::Mention => "mention",
                NotificationType::NewMessage => "new_message",
                NotificationType::SoundPlayback => "sound_playback",
            },
            "timestamp": message.created_at,
        });
        
        PushNotificationPayload {
            title,
            body: if body.len() > 100 {
                format!("{}...", &body[..97])
            } else {
                body
            },
            icon: Some("/icon-192x192.png".to_string()),
            badge: Some("/badge-72x72.png".to_string()),
            tag: Some(format!("room-{}", message.room_id)),
            data,
        }
    }
}

#[async_trait]
impl PushNotificationService for PushNotificationServiceImpl {
    async fn create_subscription(
        &self,
        user_id: UserId,
        request: CreatePushSubscriptionRequest,
    ) -> Result<PushSubscription, PushNotificationError> {
        let subscription = PushSubscription {
            id: PushSubscriptionId::new(),
            user_id,
            endpoint: request.endpoint,
            p256dh_key: request.keys.p256dh,
            auth_key: request.keys.auth,
            created_at: Utc::now(),
            last_used_at: None,
        };
        
        self.writer.create_push_subscription(subscription.clone()).await?;
        
        Ok(subscription)
    }
    
    async fn delete_subscription(
        &self,
        subscription_id: PushSubscriptionId,
    ) -> Result<(), PushNotificationError> {
        self.database.delete_push_subscription(subscription_id).await?;
        Ok(())
    }
    
    async fn update_preferences(
        &self,
        user_id: UserId,
        request: UpdateNotificationPreferencesRequest,
    ) -> Result<NotificationPreferences, PushNotificationError> {
        // Get current preferences
        let mut preferences = self.database.get_notification_preferences(user_id).await?;
        
        // Update with provided values
        if let Some(mentions_enabled) = request.mentions_enabled {
            preferences.mentions_enabled = mentions_enabled;
        }
        if let Some(direct_messages_enabled) = request.direct_messages_enabled {
            preferences.direct_messages_enabled = direct_messages_enabled;
        }
        if let Some(all_messages_enabled) = request.all_messages_enabled {
            preferences.all_messages_enabled = all_messages_enabled;
        }
        if let Some(sounds_enabled) = request.sounds_enabled {
            preferences.sounds_enabled = sounds_enabled;
        }
        
        preferences.updated_at = Utc::now();
        
        self.writer.update_notification_preferences(preferences.clone()).await?;
        
        Ok(preferences)
    }
    
    async fn get_preferences(
        &self,
        user_id: UserId,
    ) -> Result<NotificationPreferences, PushNotificationError> {
        Ok(self.database.get_notification_preferences(user_id).await?)
    }
    
    async fn send_message_notification(
        &self,
        message: &Message,
        room: &Room,
        sender_name: &str,
    ) -> Result<(), PushNotificationError> {
        // Get users who should receive notifications
        let recipients = self.database.get_notification_recipients(message, room).await?;
        
        for (user_id, preferences) in recipients {
            // Skip if user has disabled relevant notifications
            let notification_type = self.determine_notification_type(message, room, user_id);
            let should_notify = match notification_type {
                NotificationType::DirectMessage => preferences.direct_messages_enabled,
                NotificationType::Mention => preferences.mentions_enabled,
                NotificationType::NewMessage => preferences.all_messages_enabled,
                NotificationType::SoundPlayback => preferences.sounds_enabled,
            };
            
            if !should_notify {
                continue;
            }
            
            // Get push subscriptions for this user
            let subscriptions = self.database.get_push_subscriptions_for_user(user_id).await?;
            
            if subscriptions.is_empty() {
                continue;
            }
            
            // Create notification payload
            let payload = self.create_notification_payload(
                notification_type,
                message,
                room,
                sender_name,
            );
            
            // Send to all subscriptions for this user
            for subscription in subscriptions {
                if let Err(e) = self.send_push_notification(&subscription, &payload).await {
                    tracing::warn!(
                        "Failed to send push notification to subscription {}: {:?}",
                        subscription.id,
                        e
                    );
                    // Continue with other subscriptions
                }
            }
        }
        
        Ok(())
    }
    
    async fn send_mention_notification(
        &self,
        message: &Message,
        room: &Room,
        sender_name: &str,
        mentioned_user: UserId,
    ) -> Result<(), PushNotificationError> {
        // Get preferences for the mentioned user
        let preferences = self.database.get_notification_preferences(mentioned_user).await?;
        
        if !preferences.mentions_enabled {
            return Ok(());
        }
        
        // Get push subscriptions for the mentioned user
        let subscriptions = self.database.get_push_subscriptions_for_user(mentioned_user).await?;
        
        if subscriptions.is_empty() {
            return Ok(());
        }
        
        // Create mention notification payload
        let payload = self.create_notification_payload(
            NotificationType::Mention,
            message,
            room,
            sender_name,
        );
        
        // Send to all subscriptions for this user
        for subscription in subscriptions {
            if let Err(e) = self.send_push_notification(&subscription, &payload).await {
                tracing::warn!(
                    "Failed to send mention notification to subscription {}: {:?}",
                    subscription.id,
                    e
                );
            }
        }
        
        Ok(())
    }
    
    async fn send_sound_notification(
        &self,
        sound_name: &str,
        room: &Room,
        triggered_by_name: &str,
    ) -> Result<(), PushNotificationError> {
        // Get all room members with sound notifications enabled
        let rows = sqlx::query(
            r#"
            SELECT rm.user_id,
                   COALESCE(np.sounds_enabled, 1) as sounds_enabled
            FROM room_memberships rm
            LEFT JOIN notification_preferences np ON rm.user_id = np.user_id
            WHERE rm.room_id = ? AND np.sounds_enabled != 0
            "#
        )
        .bind(room.id.0.to_string())
        .fetch_all(self.database.pool())
        .await?;
        
        for row in rows {
            let user_id_str: &str = row.get("user_id");
            let user_id = UserId(uuid::Uuid::parse_str(user_id_str)?);
            
            // Get push subscriptions for this user
            let subscriptions = self.database.get_push_subscriptions_for_user(user_id).await?;
            
            if subscriptions.is_empty() {
                continue;
            }
            
            // Create sound notification payload
            let payload = PushNotificationPayload {
                title: format!("Sound played in {}", room.name),
                body: format!("{} played '{}'", triggered_by_name, sound_name),
                icon: Some("/icon-192x192.png".to_string()),
                badge: Some("/badge-72x72.png".to_string()),
                tag: Some(format!("sound-{}", room.id)),
                data: serde_json::json!({
                    "roomId": room.id,
                    "type": "sound_playback",
                    "soundName": sound_name,
                    "triggeredBy": triggered_by_name,
                    "timestamp": Utc::now(),
                }),
            };
            
            // Send to all subscriptions for this user
            for subscription in subscriptions {
                if let Err(e) = self.send_push_notification(&subscription, &payload).await {
                    tracing::warn!(
                        "Failed to send sound notification to subscription {}: {:?}",
                        subscription.id,
                        e
                    );
                }
            }
        }
        
        Ok(())
    }
}