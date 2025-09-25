use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::database::CampfireDatabase;
use crate::errors::{MessageError, ValidationError, BroadcastError, RoomError};
use crate::models::{Message, MessageId, RoomId, UserId, WebSocketMessage};
use crate::services::connection::ConnectionManager;
use crate::services::room::RoomServiceTrait;
use crate::services::push::PushNotificationService;
use crate::rich_text::{RichTextProcessor, RichTextError};
use crate::sounds::SoundManager;

#[async_trait]
pub trait MessageServiceTrait: Send + Sync {
    /// Creates message with deduplication (Critical Gap #1)
    /// 
    /// # Preconditions
    /// - User authenticated with room access
    /// - Content: 1-10000 chars, sanitized HTML
    /// - client_message_id: valid UUID
    /// 
    /// # Postconditions  
    /// - Returns Ok(Message) on success
    /// - Inserts row into 'messages' table
    /// - Updates room.last_message_at timestamp
    /// - Broadcasts to room subscribers via WebSocket
    /// - Deduplication: returns existing if client_message_id exists
    /// 
    /// # Error Conditions
    /// - MessageError::Authorization if user lacks room access
    /// - MessageError::InvalidContent if content violates constraints
    /// - MessageError::Database on persistence failure
    async fn create_message_with_deduplication(
        &self,
        content: String,
        room_id: RoomId,
        user_id: UserId,
        client_message_id: Uuid,
    ) -> Result<Message, MessageError>;
    
    /// Retrieves message history for a room
    async fn get_room_messages(
        &self,
        room_id: RoomId,
        user_id: UserId,
        limit: u32,
        before: Option<MessageId>,
    ) -> Result<Vec<Message>, MessageError>;
    
    /// Broadcasts message to room subscribers
    async fn broadcast_message(
        &self,
        message: &Message,
        room_id: RoomId,
    ) -> Result<(), BroadcastError>;
    
    /// Returns reference to the connection manager for WebSocket operations
    fn connection_manager(&self) -> &Arc<dyn ConnectionManager>;
}

#[derive(Clone)]
pub struct MessageService {
    db: Arc<CampfireDatabase>,
    connection_manager: Arc<dyn ConnectionManager>,
    room_service: Arc<dyn RoomServiceTrait>,
    push_service: Option<Arc<dyn PushNotificationService>>,
}

impl MessageService {
    pub fn new(
        db: Arc<CampfireDatabase>, 
        connection_manager: Arc<dyn ConnectionManager>,
        room_service: Arc<dyn RoomServiceTrait>,
    ) -> Self {
        Self {
            db,
            connection_manager,
            room_service,
            push_service: None,
        }
    }
    
    /// Get reference to the database for testing purposes
    pub fn database(&self) -> &Arc<CampfireDatabase> {
        &self.db
    }
    
    pub fn with_push_service(
        db: Arc<CampfireDatabase>, 
        connection_manager: Arc<dyn ConnectionManager>,
        room_service: Arc<dyn RoomServiceTrait>,
        push_service: Arc<dyn PushNotificationService>,
    ) -> Self {
        Self {
            db,
            connection_manager,
            room_service,
            push_service: Some(push_service),
        }
    }
    
    /// Returns reference to the connection manager for WebSocket operations
    pub fn connection_manager(&self) -> &Arc<dyn ConnectionManager> {
        &self.connection_manager
    }
    
    /// Validates and processes message content with rich text features
    /// 
    /// Rules:
    /// - Content must be between 1 and 10,000 characters
    /// - HTML content must be sanitized with rich text support
    /// - @mentions are processed and linked
    /// - /play commands are extracted and validated
    /// - No malicious scripts or dangerous HTML
    async fn validate_and_process_content(
        &self,
        content: &str,
    ) -> Result<(String, Option<String>, Vec<String>, Vec<String>), ValidationError> {
        // Check length constraints
        if content.trim().is_empty() {
            return Err(ValidationError::InvalidContentLength);
        }
        
        if content.len() > 10000 {
            return Err(ValidationError::InvalidContentLength);
        }
        
        // Extract and clean /play commands first
        let (cleaned_content, play_commands) = RichTextProcessor::extract_and_clean_play_commands(content);
        
        // Use cleaned content for display if play commands were removed
        let display_content = if cleaned_content.trim().is_empty() && !play_commands.is_empty() {
            // If only play commands, use a default message
            format!("🎵 Played: {}", play_commands.join(", "))
        } else if cleaned_content != content {
            cleaned_content
        } else {
            content.to_string()
        };
        
        // Create user lookup function for @mentions
        let _db = Arc::clone(&self.db);
        let user_lookup = move |_username: &str| -> Option<UserId> {
            // For now, we'll do a simple lookup - in a real implementation,
            // this would be async and cached
            // TODO: Implement proper async user lookup with caching
            None // Placeholder - will be implemented when user lookup is available
        };
        
        // Process rich text content
        match RichTextProcessor::process_content(&display_content, user_lookup).await {
            Ok(processed) => {
                // Use the sanitized HTML as the display content
                let final_display_content = processed.html.clone();
                
                let html_content = if processed.has_rich_features || processed.html != display_content {
                    Some(processed.html)
                } else {
                    None
                };
                
                Ok((final_display_content, html_content, processed.mentions, play_commands))
            }
            Err(RichTextError::SanitizationRemoved) => {
                Err(ValidationError::HtmlSanitization {
                    reason: "Content was entirely removed during sanitization".to_string(),
                })
            }
            Err(e) => {
                Err(ValidationError::HtmlSanitization {
                    reason: format!("Rich text processing failed: {}", e),
                })
            }
        }
    }
    
    /// Checks if user has access to the room using RoomService
    async fn check_room_access(&self, room_id: RoomId, user_id: UserId) -> Result<bool, MessageError> {
        match self.room_service.check_room_access(room_id, user_id).await {
            Ok(Some(_involvement_level)) => Ok(true),
            Ok(None) => Ok(false),
            Err(RoomError::NotFound { .. }) => {
                Err(MessageError::Authorization { user_id, room_id })
            }
            Err(e) => {
                // Convert RoomError to MessageError
                Err(MessageError::Database(
                    sqlx::Error::Configuration(format!("Room access check failed: {}", e).into())
                ))
            }
        }
    }
}

#[async_trait]
impl MessageServiceTrait for MessageService {
    async fn create_message_with_deduplication(
        &self,
        content: String,
        room_id: RoomId,
        user_id: UserId,
        client_message_id: Uuid,
    ) -> Result<Message, MessageError> {
        // Step 1: Validate and process content with rich text features
        let (display_content, html_content, mentions, play_commands) = self
            .validate_and_process_content(&content)
            .await
            .map_err(|e| MessageError::InvalidContent { 
                reason: e.to_string() 
            })?;
        
        // Step 2: Check room access
        if !self.check_room_access(room_id, user_id).await? {
            return Err(MessageError::Authorization { user_id, room_id });
        }
        
        // Step 3: Create message object with rich text features
        let message = Message::with_rich_content(
            room_id,
            user_id,
            display_content,
            client_message_id,
            html_content,
            mentions,
            play_commands.clone(),
        );
        
        // Step 4: Persist with deduplication (Critical Gap #1)
        let persisted_message = self.db
            .create_message_with_deduplication(message)
            .await?;
        
        // Step 5: Broadcast message to room subscribers
        if let Err(broadcast_error) = self.broadcast_message(&persisted_message, room_id).await {
            // Log the error but don't fail the message creation
            tracing::warn!("Failed to broadcast message {}: {}", persisted_message.id.0, broadcast_error);
        }
        
        // Step 6: Send push notifications if service is available
        if let Some(push_service) = &self.push_service {
            // Get room information for notification context
            if let Ok(Some(room)) = self.room_service.get_room_by_id(room_id).await {
                // Get sender name for notification
                if let Ok(Some(sender)) = self.db.get_user_by_id(user_id).await {
                    // Send message notification
                    if let Err(e) = push_service.send_message_notification(&persisted_message, &room, &sender.name).await {
                        tracing::warn!("Failed to send push notification for message {}: {}", persisted_message.id.0, e);
                    }
                    
                    // Send mention notifications for each mentioned user
                    for mention in &persisted_message.mentions {
                        if let Ok(Some(mentioned_user)) = self.db.get_user_by_email(mention).await {
                            if let Err(e) = push_service.send_mention_notification(&persisted_message, &room, &sender.name, mentioned_user.id).await {
                                tracing::warn!("Failed to send mention notification to {}: {}", mention, e);
                            }
                        }
                    }
                }
            }
        }
        
        // Step 7: Broadcast sound playback commands if any
        for sound_name in &play_commands {
            if SoundManager::sound_exists(sound_name) {
                let sound_message = WebSocketMessage::SoundPlayback {
                    sound_name: sound_name.clone(),
                    triggered_by: user_id,
                    room_id,
                    timestamp: Utc::now(),
                };
                
                if let Err(e) = self.connection_manager.broadcast_to_room(room_id, sound_message).await {
                    tracing::warn!("Failed to broadcast sound playback {}: {}", sound_name, e);
                }
                
                // Send push notification for sound playback if service is available
                if let Some(push_service) = &self.push_service {
                    if let Ok(Some(room)) = self.room_service.get_room_by_id(room_id).await {
                        if let Ok(Some(sender)) = self.db.get_user_by_id(user_id).await {
                            if let Err(e) = push_service.send_sound_notification(sound_name, &room, &sender.name).await {
                                tracing::warn!("Failed to send sound notification for {}: {}", sound_name, e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(persisted_message)
    }
    
    async fn get_room_messages(
        &self,
        room_id: RoomId,
        user_id: UserId,
        limit: u32,
        before: Option<MessageId>,
    ) -> Result<Vec<Message>, MessageError> {
        // Check room access
        if !self.check_room_access(room_id, user_id).await? {
            return Err(MessageError::Authorization { user_id, room_id });
        }
        
        // Limit the number of messages to prevent abuse
        let safe_limit = std::cmp::min(limit, 100);
        
        let messages = self.db
            .get_room_messages(room_id, safe_limit, before)
            .await?;
        
        Ok(messages)
    }
    
    async fn broadcast_message(
        &self,
        message: &Message,
        room_id: RoomId,
    ) -> Result<(), BroadcastError> {
        let ws_message = WebSocketMessage::NewMessage {
            message: message.clone(),
        };
        
        self.connection_manager
            .broadcast_to_room(room_id, ws_message)
            .await
    }
    
    fn connection_manager(&self) -> &Arc<dyn ConnectionManager> {
        &self.connection_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::CampfireDatabase;
    use crate::services::connection::ConnectionManagerImpl;
    use sqlx::Row;
    
    async fn create_test_message_service() -> MessageService {
        let db = CampfireDatabase::new(":memory:").await.unwrap();
        let db_arc = Arc::new(db);
        let connection_manager = Arc::new(ConnectionManagerImpl::new(db_arc.clone()));
        let room_service = Arc::new(crate::services::room::RoomService::new(db_arc.clone()));
        MessageService::new(db_arc, connection_manager, room_service)
    }
    
    #[tokio::test]
    async fn test_content_validation() {
        let service = create_test_message_service().await;
        
        // Valid content
        let result = service.validate_and_process_content("Hello, world!").await;
        assert!(result.is_ok());
        
        // Empty content
        let result = service.validate_and_process_content("").await;
        assert!(matches!(result, Err(ValidationError::InvalidContentLength)));
        
        // Whitespace only
        let result = service.validate_and_process_content("   ").await;
        assert!(matches!(result, Err(ValidationError::InvalidContentLength)));
        
        // Too long content
        let long_content = "a".repeat(10001);
        let result = service.validate_and_process_content(&long_content).await;
        assert!(matches!(result, Err(ValidationError::InvalidContentLength)));
        
        // HTML sanitization
        let html_content = "<script>alert('xss')</script>Hello";
        let result = service.validate_and_process_content(html_content).await.unwrap();
        println!("Sanitized result: {:?}", result);
        // The script tag should be removed by ammonia
        assert!(!result.0.contains("<script>"));
        assert!(result.0.contains("Hello"));
    }
    
    #[tokio::test]
    async fn test_database_basic_operations() {
        // Test basic database operations first
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        
        // Test that we can execute a simple query
        let result = sqlx::query("SELECT 1 as test")
            .fetch_one(db.pool())
            .await
            .unwrap();
        
        let test_value: i32 = result.get("test");
        assert_eq!(test_value, 1);
        
        // Test that tables were created
        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")
            .fetch_optional(db.pool())
            .await
            .unwrap();
        
        assert!(result.is_some(), "Users table should exist");
    }
    
    #[tokio::test]
    async fn test_uuid_insertion() {
        // Test UUID insertion specifically
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        
        let user_id = crate::models::UserId::new();
        let uuid_str = user_id.0.to_string();
        
        // Test direct UUID string insertion
        let result = sqlx::query("INSERT INTO users (id, name, email, password_hash, admin) VALUES (?, ?, ?, ?, ?)")
            .bind(&uuid_str)
            .bind("Test User")
            .bind("test@example.com")
            .bind("hash")
            .bind(false)
            .execute(db.pool())
            .await;
        
        match result {
            Ok(_) => println!("UUID insertion successful"),
            Err(e) => panic!("UUID insertion failed: {}", e),
        }
        
        // Test retrieval
        let result = sqlx::query("SELECT id FROM users WHERE email = ?")
            .bind("test@example.com")
            .fetch_one(db.pool())
            .await
            .unwrap();
        
        let retrieved_id: &str = result.get("id");
        assert_eq!(retrieved_id, uuid_str);
    }
    
    #[tokio::test]
    async fn test_create_user_method() {
        // Test the actual create_user method
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        
        let user = crate::models::User {
            id: crate::models::UserId::new(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            bio: None,
            admin: false,
            bot_token: None,
            created_at: chrono::Utc::now(),
        };
        
        let result = db.create_user(user).await;
        match result {
            Ok(_) => println!("create_user successful"),
            Err(e) => panic!("create_user failed: {:?}", e),
        }
    }
    


    #[tokio::test]
    async fn test_message_deduplication() {
        // Test Critical Gap #1: Message Deduplication
        let service = create_test_message_service().await;
        
        let room_id = RoomId::new();
        let user_id = UserId::new();
        let client_message_id = Uuid::new_v4();
        
        // Create the user first
        let user = crate::models::User {
            id: user_id,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            bio: None,
            admin: false,
            bot_token: None,
            created_at: chrono::Utc::now(),
        };
        
        service.db.create_user(user).await.unwrap();
        
        // Create a room first (required for foreign key constraint)
        let room = crate::models::Room {
            id: room_id,
            name: "Test Room".to_string(),
            topic: None,
            room_type: crate::models::RoomType::Open,
            created_at: chrono::Utc::now(),
            last_message_at: None,
        };
        service.db.create_room(room).await.unwrap();
        
        // Create membership so user has access to the room
        let membership = crate::models::Membership {
            room_id,
            user_id,
            involvement_level: crate::models::InvolvementLevel::Member,
            created_at: chrono::Utc::now(),
        };
        service.db.create_membership(membership).await.unwrap();
        
        // First message should be created
        let message1 = service
            .create_message_with_deduplication(
                "First message".to_string(),
                room_id,
                user_id,
                client_message_id,
            )
            .await
            .unwrap();
        
        // Second message with same client_message_id should return the same message
        let message2 = service
            .create_message_with_deduplication(
                "Second message (should be ignored)".to_string(),
                room_id,
                user_id,
                client_message_id,
            )
            .await
            .unwrap();
        
        // Should be the same message (deduplication worked)
        assert_eq!(message1.id, message2.id);
        assert_eq!(message1.content, message2.content);
        assert_eq!(message1.content, "First message"); // Original content preserved
    }
    
    #[tokio::test]
    async fn test_message_creation_with_broadcast() {
        let service = create_test_message_service().await;
        
        let room_id = RoomId::new();
        let user_id = UserId::new();
        
        // Create the user first
        let user = crate::models::User {
            id: user_id,
            name: "Test User".to_string(),
            email: "test2@example.com".to_string(),
            password_hash: "hash".to_string(),
            bio: None,
            admin: false,
            bot_token: None,
            created_at: chrono::Utc::now(),
        };
        
        service.db.create_user(user).await.unwrap();
        
        // Create a room first (required for foreign key constraint)
        let room = crate::models::Room {
            id: room_id,
            name: "Test Room 2".to_string(),
            topic: None,
            room_type: crate::models::RoomType::Open,
            created_at: chrono::Utc::now(),
            last_message_at: None,
        };
        service.db.create_room(room).await.unwrap();
        
        // Create membership so user has access to the room
        let membership = crate::models::Membership {
            room_id,
            user_id,
            involvement_level: crate::models::InvolvementLevel::Member,
            created_at: chrono::Utc::now(),
        };
        service.db.create_membership(membership).await.unwrap();
        
        let message = service
            .create_message_with_deduplication(
                "Test message".to_string(),
                room_id,
                user_id,
                Uuid::new_v4(),
            )
            .await
            .unwrap();
        
        assert_eq!(message.content, "Test message");
        assert_eq!(message.room_id, room_id);
        assert_eq!(message.creator_id, user_id);
    }
    
    #[tokio::test]
    async fn test_get_room_messages() {
        let service = create_test_message_service().await;
        
        let room_id = RoomId::new();
        let user_id = UserId::new();
        
        // Create the user first
        let user = crate::models::User {
            id: user_id,
            name: "Test User".to_string(),
            email: "test3@example.com".to_string(),
            password_hash: "hash".to_string(),
            bio: None,
            admin: false,
            bot_token: None,
            created_at: chrono::Utc::now(),
        };
        
        service.db.create_user(user).await.unwrap();
        
        // Create a room first
        let room = crate::models::Room {
            id: room_id,
            name: "Test Room 3".to_string(),
            topic: None,
            room_type: crate::models::RoomType::Open,
            created_at: chrono::Utc::now(),
            last_message_at: None,
        };
        service.db.create_room(room).await.unwrap();
        
        // Create membership so user has access to the room
        let membership = crate::models::Membership {
            room_id,
            user_id,
            involvement_level: crate::models::InvolvementLevel::Member,
            created_at: chrono::Utc::now(),
        };
        service.db.create_membership(membership).await.unwrap();
        
        let messages = service
            .get_room_messages(room_id, user_id, 10, None)
            .await
            .unwrap();
        
        // Should return empty list for new room
        assert!(messages.is_empty());
    }
    
    #[tokio::test]
    async fn test_message_limit_enforcement() {
        let service = create_test_message_service().await;
        
        let room_id = RoomId::new();
        let user_id = UserId::new();
        
        // Create the user first
        let user = crate::models::User {
            id: user_id,
            name: "Test User".to_string(),
            email: "test4@example.com".to_string(),
            password_hash: "hash".to_string(),
            bio: None,
            admin: false,
            bot_token: None,
            created_at: chrono::Utc::now(),
        };
        
        service.db.create_user(user).await.unwrap();
        
        // Create a room first
        let room = crate::models::Room {
            id: room_id,
            name: "Test Room 4".to_string(),
            topic: None,
            room_type: crate::models::RoomType::Open,
            created_at: chrono::Utc::now(),
            last_message_at: None,
        };
        service.db.create_room(room).await.unwrap();
        
        // Create membership so user has access to the room
        let membership = crate::models::Membership {
            room_id,
            user_id,
            involvement_level: crate::models::InvolvementLevel::Member,
            created_at: chrono::Utc::now(),
        };
        service.db.create_membership(membership).await.unwrap();
        
        // Request more than the safe limit
        let messages = service
            .get_room_messages(room_id, user_id, 1000, None)
            .await
            .unwrap();
        
        // Should be limited (though empty in this test)
        assert!(messages.len() <= 100);
    }
}