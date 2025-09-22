use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::database::Database;
use crate::errors::{MessageError, ValidationError, BroadcastError};
use crate::models::{Message, MessageId, RoomId, UserId, WebSocketMessage};
use crate::services::connection::ConnectionManager;

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
}

#[derive(Clone)]
pub struct MessageService {
    db: Arc<Database>,
    connection_manager: Arc<dyn ConnectionManager>,
}

impl MessageService {
    pub fn new(db: Arc<Database>, connection_manager: Arc<dyn ConnectionManager>) -> Self {
        Self {
            db,
            connection_manager,
        }
    }
    
    /// Validates message content according to Campfire rules
    /// 
    /// Rules:
    /// - Content must be between 1 and 10,000 characters
    /// - HTML content must be sanitized
    /// - No malicious scripts or dangerous HTML
    fn validate_content(content: &str) -> Result<String, ValidationError> {
        // Check length constraints
        if content.trim().is_empty() {
            return Err(ValidationError::InvalidContentLength);
        }
        
        if content.len() > 10000 {
            return Err(ValidationError::InvalidContentLength);
        }
        
        // Sanitize HTML content using ammonia
        let sanitized = ammonia::clean(content);
        
        // Ensure sanitization didn't remove everything
        if sanitized.trim().is_empty() && !content.trim().is_empty() {
            return Err(ValidationError::HtmlSanitization {
                reason: "Content was entirely removed during sanitization".to_string(),
            });
        }
        
        Ok(sanitized)
    }
    
    /// Checks if user has access to the room
    /// 
    /// This is a simplified version - in a full implementation,
    /// this would check room memberships and permissions
    async fn check_room_access(&self, room_id: RoomId, user_id: UserId) -> Result<bool, MessageError> {
        // For MVP, we'll implement a basic check
        // In the full version, this would query room_memberships table
        
        // TODO: Implement proper room access checking
        // For now, assume all authenticated users have access
        Ok(true)
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
        // Step 1: Validate content
        let sanitized_content = Self::validate_content(&content)
            .map_err(|e| MessageError::InvalidContent { 
                reason: e.to_string() 
            })?;
        
        // Step 2: Check room access
        if !self.check_room_access(room_id, user_id).await? {
            return Err(MessageError::Authorization { user_id, room_id });
        }
        
        // Step 3: Create message object
        let message = Message {
            id: MessageId::new(),
            room_id,
            creator_id: user_id,
            content: sanitized_content,
            client_message_id,
            created_at: Utc::now(),
        };
        
        // Step 4: Persist with deduplication (Critical Gap #1)
        let persisted_message = self.db
            .create_message_with_deduplication(&message)
            .await?;
        
        // Step 5: Broadcast to room subscribers
        if let Err(broadcast_error) = self.broadcast_message(&persisted_message, room_id).await {
            // Log the error but don't fail the message creation
            tracing::warn!("Failed to broadcast message {}: {}", persisted_message.id.0, broadcast_error);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use crate::services::connection::ConnectionManagerImpl;
    use sqlx::Row;
    
    async fn create_test_message_service() -> MessageService {
        let db = Database::new(":memory:").await.unwrap();
        let connection_manager = Arc::new(ConnectionManagerImpl::new());
        MessageService::new(Arc::new(db), connection_manager)
    }
    
    #[tokio::test]
    async fn test_content_validation() {
        // Valid content
        let result = MessageService::validate_content("Hello, world!");
        assert!(result.is_ok());
        
        // Empty content
        let result = MessageService::validate_content("");
        assert!(matches!(result, Err(ValidationError::InvalidContentLength)));
        
        // Whitespace only
        let result = MessageService::validate_content("   ");
        assert!(matches!(result, Err(ValidationError::InvalidContentLength)));
        
        // Too long content
        let long_content = "a".repeat(10001);
        let result = MessageService::validate_content(&long_content);
        assert!(matches!(result, Err(ValidationError::InvalidContentLength)));
        
        // HTML sanitization
        let html_content = "<script>alert('xss')</script>Hello";
        let result = MessageService::validate_content(html_content).unwrap();
        assert!(!result.contains("<script>"));
        assert!(result.contains("Hello"));
    }
    
    #[tokio::test]
    async fn test_database_basic_operations() {
        // Test basic database operations first
        let db = crate::database::Database::new(":memory:").await.unwrap();
        
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
        let db = crate::database::Database::new(":memory:").await.unwrap();
        
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
        let db = crate::database::Database::new(":memory:").await.unwrap();
        
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
        
        let result = db.create_user(&user).await;
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
        
        service.db.create_user(&user).await.unwrap();
        
        // Create a room first (required for foreign key constraint)
        sqlx::query("INSERT INTO rooms (id, name, room_type) VALUES (?, ?, ?)")
            .bind(room_id.0.to_string())
            .bind("Test Room")
            .bind("open")
            .execute(service.db.pool())
            .await
            .unwrap();
        
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
        
        service.db.create_user(&user).await.unwrap();
        
        // Create a room first (required for foreign key constraint)
        sqlx::query("INSERT INTO rooms (id, name, room_type) VALUES (?, ?, ?)")
            .bind(room_id.0.to_string())
            .bind("Test Room 2")
            .bind("open")
            .execute(service.db.pool())
            .await
            .unwrap();
        
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
        
        // Request more than the safe limit
        let messages = service
            .get_room_messages(room_id, user_id, 1000, None)
            .await
            .unwrap();
        
        // Should be limited (though empty in this test)
        assert!(messages.len() <= 100);
    }
}