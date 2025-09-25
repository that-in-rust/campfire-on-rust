use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::database::CampfireDatabase;
use crate::errors::{MessageError, BroadcastError};
use crate::models::{Message, MessageId, RoomId, UserId};
use crate::services::message::{MessageService, MessageServiceTrait};
use crate::services::room::RoomServiceTrait;
use crate::services::connection::ConnectionManager;
use crate::services::push::PushNotificationService;
use crate::services::cache::{CacheService, CacheServiceTrait};

/// Cached message service that wraps the base MessageService
/// 
/// Provides caching for:
/// - Message history for active rooms (most frequent read operation)
/// - Recent messages for real-time updates
/// 
/// Cache TTLs:
/// - Message history: 5 minutes (messages are relatively static once created)
/// - Recent messages: 2 minutes (for real-time scenarios)
/// 
/// Cache invalidation:
/// - New messages invalidate room message cache
/// - Message updates/deletions invalidate affected caches
/// 
/// Cache strategy:
/// - Cache message history by (room_id, limit, before) key
/// - Invalidate entire room cache when new messages arrive
/// - Use shorter TTL for active rooms vs inactive rooms
#[derive(Clone)]
pub struct CachedMessageService {
    message_service: MessageService,
    cache_service: Arc<dyn CacheServiceTrait>,
}

impl CachedMessageService {
    pub fn new(
        db: Arc<CampfireDatabase>,
        connection_manager: Arc<dyn ConnectionManager>,
        room_service: Arc<dyn RoomServiceTrait>,
        cache_service: Arc<dyn CacheServiceTrait>,
    ) -> Self {
        Self {
            message_service: MessageService::new(db, connection_manager, room_service),
            cache_service,
        }
    }
    
    pub fn with_push_service(
        db: Arc<CampfireDatabase>,
        connection_manager: Arc<dyn ConnectionManager>,
        room_service: Arc<dyn RoomServiceTrait>,
        push_service: Arc<dyn PushNotificationService>,
        cache_service: Arc<dyn CacheServiceTrait>,
    ) -> Self {
        Self {
            message_service: MessageService::with_push_service(db, connection_manager, room_service, push_service),
            cache_service,
        }
    }
    
    /// Cache TTLs for different scenarios
    const MESSAGE_HISTORY_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes
    const RECENT_MESSAGES_CACHE_TTL: Duration = Duration::from_secs(120); // 2 minutes
    
    /// Determine cache TTL based on message recency and room activity
    fn get_cache_ttl(&self, messages: &[Message]) -> Duration {
        if messages.is_empty() {
            return Self::MESSAGE_HISTORY_CACHE_TTL;
        }
        
        // Use shorter TTL for recent messages (more likely to change)
        let now = chrono::Utc::now();
        let most_recent = messages.iter()
            .map(|m| m.created_at)
            .max()
            .unwrap_or(now);
        
        let age = now - most_recent;
        
        if age.num_minutes() < 10 {
            // Very recent messages - shorter cache
            Self::RECENT_MESSAGES_CACHE_TTL
        } else {
            // Older messages - longer cache
            Self::MESSAGE_HISTORY_CACHE_TTL
        }
    }
}

#[async_trait]
impl MessageServiceTrait for CachedMessageService {
    async fn create_message_with_deduplication(
        &self,
        content: String,
        room_id: RoomId,
        user_id: UserId,
        client_message_id: Uuid,
    ) -> Result<Message, MessageError> {
        // Create message in database (no caching for writes)
        let message = self.message_service.create_message_with_deduplication(
            content,
            room_id,
            user_id,
            client_message_id,
        ).await?;
        
        // Invalidate message cache for this room since we added a new message
        if let Err(e) = self.cache_service.invalidate_room_messages(room_id).await {
            tracing::warn!("Failed to invalidate message cache for room {}: {}", room_id, e);
        }
        
        Ok(message)
    }
    
    async fn get_room_messages(
        &self,
        room_id: RoomId,
        user_id: UserId,
        limit: u32,
        before: Option<MessageId>,
    ) -> Result<Vec<Message>, MessageError> {
        // Try cache first
        match self.cache_service.get_cached_messages(room_id, limit, before).await {
            Ok(Some(cached_messages)) => {
                tracing::debug!("Message cache hit for room {} (limit: {}, before: {:?})", room_id, limit, before);
                return Ok(cached_messages);
            }
            Ok(None) => {
                tracing::debug!("Message cache miss for room {} (limit: {}, before: {:?})", room_id, limit, before);
            }
            Err(e) => {
                tracing::warn!("Message cache error for room {}: {}", room_id, e);
            }
        }
        
        // Cache miss - get from database
        match self.message_service.get_room_messages(room_id, user_id, limit, before).await {
            Ok(messages) => {
                // Cache the result with appropriate TTL
                let ttl = self.get_cache_ttl(&messages);
                
                if let Err(e) = self.cache_service.cache_messages(
                    room_id,
                    limit,
                    before,
                    messages.clone(),
                    ttl,
                ).await {
                    tracing::warn!("Failed to cache messages for room {}: {}", room_id, e);
                }
                
                Ok(messages)
            }
            Err(e) => {
                // Don't cache errors
                Err(e)
            }
        }
    }
    
    async fn broadcast_message(
        &self,
        message: &Message,
        room_id: RoomId,
    ) -> Result<(), BroadcastError> {
        // Broadcasting doesn't involve caching
        self.message_service.broadcast_message(message, room_id).await
    }
    
    fn connection_manager(&self) -> &Arc<dyn ConnectionManager> {
        self.message_service.connection_manager()
    }
}

/// Extension methods for cache management
impl CachedMessageService {
    /// Invalidate all cached messages for a room
    /// Call this when doing bulk message operations or room cleanup
    pub async fn invalidate_room_message_cache(&self, room_id: RoomId) -> Result<(), MessageError> {
        if let Err(e) = self.cache_service.invalidate_room_messages(room_id).await {
            tracing::warn!("Failed to invalidate message cache for room {}: {}", room_id, e);
        }
        
        Ok(())
    }
    
    /// Preload message cache for active rooms
    /// Useful for warming up cache for rooms that will be accessed frequently
    pub async fn preload_room_messages(
        &self,
        room_id: RoomId,
        user_id: UserId,
        limit: u32,
    ) -> Result<(), MessageError> {
        // This will populate the cache
        let _ = self.get_room_messages(room_id, user_id, limit, None).await?;
        
        Ok(())
    }
    
    /// Get cache statistics for message caching
    pub async fn get_message_cache_stats(&self) -> Result<crate::services::cache::CacheStats, MessageError> {
        Ok(self.cache_service.get_cache_stats().await)
    }
    
    /// Warm up cache for multiple rooms
    /// Useful during application startup or when preparing for high traffic
    pub async fn warm_up_cache(&self, room_user_pairs: Vec<(RoomId, UserId)>) -> Result<(), MessageError> {
        const DEFAULT_LIMIT: u32 = 50;
        
        for (room_id, user_id) in room_user_pairs {
            if let Err(e) = self.preload_room_messages(room_id, user_id, DEFAULT_LIMIT).await {
                tracing::warn!("Failed to preload messages for room {} and user {}: {}", room_id, user_id, e);
                // Continue with other rooms even if one fails
            }
        }
        
        Ok(())
    }
    
    /// Clear expired message cache entries
    /// Call this periodically to clean up memory
    pub async fn cleanup_expired_cache(&self) -> Result<u64, MessageError> {
        match self.cache_service.clear_expired_entries().await {
            Ok(count) => {
                tracing::info!("Cleaned up {} expired message cache entries", count);
                Ok(count)
            }
            Err(e) => {
                tracing::warn!("Failed to cleanup expired cache entries: {}", e);
                Ok(0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::CampfireDatabase;
    use crate::services::cache::CacheService;
    use crate::services::connection::ConnectionManagerImpl;
    use crate::services::room::RoomService;
    use crate::models::{User, Room, RoomType, Membership, InvolvementLevel};
    use chrono::Utc;
    
    async fn create_test_cached_message_service() -> CachedMessageService {
        let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
        let connection_manager = Arc::new(ConnectionManagerImpl::new(db.clone()));
        let room_service = Arc::new(RoomService::new(db.clone()));
        let cache_service = Arc::new(CacheService::with_defaults());
        
        CachedMessageService::new(db, connection_manager, room_service, cache_service)
    }
    
    async fn setup_test_data(service: &CachedMessageService) -> (User, Room) {
        let user = User {
            id: UserId::new(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            bio: None,
            admin: false,
            bot_token: None,
            created_at: Utc::now(),
        };
        
        let room = Room {
            id: RoomId::new(),
            name: "Test Room".to_string(),
            topic: None,
            room_type: RoomType::Open,
            created_at: Utc::now(),
            last_message_at: None,
        };
        
        let membership = Membership {
            room_id: room.id,
            user_id: user.id,
            involvement_level: InvolvementLevel::Member,
            created_at: Utc::now(),
        };
        
        // Create in database
        service.message_service.database().create_user(user.clone()).await.unwrap();
        service.message_service.database().create_room(room.clone()).await.unwrap();
        service.message_service.database().create_membership(membership).await.unwrap();
        
        (user, room)
    }
    
    #[tokio::test]
    async fn test_message_history_caching() {
        let service = create_test_cached_message_service().await;
        let (user, room) = setup_test_data(&service).await;
        
        // Create a message
        let message = service.create_message_with_deduplication(
            "Test message".to_string(),
            room.id,
            user.id,
            Uuid::new_v4(),
        ).await.unwrap();
        
        // First call should hit database and cache the result
        let messages1 = service.get_room_messages(room.id, user.id, 10, None).await.unwrap();
        assert_eq!(messages1.len(), 1);
        assert_eq!(messages1[0].id, message.id);
        
        // Second call should hit cache
        let messages2 = service.get_room_messages(room.id, user.id, 10, None).await.unwrap();
        assert_eq!(messages2.len(), 1);
        assert_eq!(messages2[0].id, message.id);
    }
    
    #[tokio::test]
    async fn test_cache_invalidation_on_new_message() {
        let service = create_test_cached_message_service().await;
        let (user, room) = setup_test_data(&service).await;
        
        // Get initial messages (empty, but will cache the result)
        let messages1 = service.get_room_messages(room.id, user.id, 10, None).await.unwrap();
        assert_eq!(messages1.len(), 0);
        
        // Create a new message (should invalidate cache)
        let _message = service.create_message_with_deduplication(
            "New message".to_string(),
            room.id,
            user.id,
            Uuid::new_v4(),
        ).await.unwrap();
        
        // Next call should see the new message (cache was invalidated)
        let messages2 = service.get_room_messages(room.id, user.id, 10, None).await.unwrap();
        assert_eq!(messages2.len(), 1);
    }
    
    #[tokio::test]
    async fn test_different_pagination_parameters() {
        let service = create_test_cached_message_service().await;
        let (user, room) = setup_test_data(&service).await;
        
        // Create multiple messages
        for i in 0..5 {
            service.create_message_with_deduplication(
                format!("Message {}", i),
                room.id,
                user.id,
                Uuid::new_v4(),
            ).await.unwrap();
        }
        
        // Different pagination parameters should be cached separately
        let messages_limit_3 = service.get_room_messages(room.id, user.id, 3, None).await.unwrap();
        let messages_limit_5 = service.get_room_messages(room.id, user.id, 5, None).await.unwrap();
        
        assert_eq!(messages_limit_3.len(), 3);
        assert_eq!(messages_limit_5.len(), 5);
        
        // Both should be cached independently
        let messages_limit_3_cached = service.get_room_messages(room.id, user.id, 3, None).await.unwrap();
        let messages_limit_5_cached = service.get_room_messages(room.id, user.id, 5, None).await.unwrap();
        
        assert_eq!(messages_limit_3_cached.len(), 3);
        assert_eq!(messages_limit_5_cached.len(), 5);
    }
    
    #[tokio::test]
    async fn test_preload_room_messages() {
        let service = create_test_cached_message_service().await;
        let (user, room) = setup_test_data(&service).await;
        
        // Create a message
        service.create_message_with_deduplication(
            "Test message".to_string(),
            room.id,
            user.id,
            Uuid::new_v4(),
        ).await.unwrap();
        
        // Preload messages (should populate cache)
        service.preload_room_messages(room.id, user.id, 10).await.unwrap();
        
        // Subsequent call should hit cache
        let messages = service.get_room_messages(room.id, user.id, 10, None).await.unwrap();
        assert_eq!(messages.len(), 1);
    }
    
    #[tokio::test]
    async fn test_cache_ttl_based_on_message_age() {
        let service = create_test_cached_message_service().await;
        
        // Test with recent messages (should get shorter TTL)
        let recent_messages = vec![
            Message::new(RoomId::new(), UserId::new(), "Recent".to_string(), Uuid::new_v4())
        ];
        let recent_ttl = service.get_cache_ttl(&recent_messages);
        
        // Test with older messages (create message with old timestamp)
        let mut old_message = Message::new(RoomId::new(), UserId::new(), "Old".to_string(), Uuid::new_v4());
        old_message.created_at = Utc::now() - chrono::Duration::hours(1);
        let old_messages = vec![old_message];
        let old_ttl = service.get_cache_ttl(&old_messages);
        
        // Recent messages should have shorter TTL
        assert!(recent_ttl < old_ttl);
    }
    
    #[tokio::test]
    async fn test_warm_up_cache() {
        let service = create_test_cached_message_service().await;
        let (user1, room1) = setup_test_data(&service).await;
        
        // Create another user and room
        let user2 = User {
            id: UserId::new(),
            name: "User 2".to_string(),
            email: "user2@example.com".to_string(),
            password_hash: "hash".to_string(),
            bio: None,
            admin: false,
            bot_token: None,
            created_at: Utc::now(),
        };
        
        let room2 = Room {
            id: RoomId::new(),
            name: "Room 2".to_string(),
            topic: None,
            room_type: RoomType::Open,
            created_at: Utc::now(),
            last_message_at: None,
        };
        
        service.message_service.database().create_user(user2.clone()).await.unwrap();
        service.message_service.database().create_room(room2.clone()).await.unwrap();
        
        // Warm up cache for multiple rooms
        let room_user_pairs = vec![
            (room1.id, user1.id),
            (room2.id, user2.id),
        ];
        
        service.warm_up_cache(room_user_pairs).await.unwrap();
        
        // Both rooms should now have cached message history
        let messages1 = service.get_room_messages(room1.id, user1.id, 50, None).await.unwrap();
        let messages2 = service.get_room_messages(room2.id, user2.id, 50, None).await.unwrap();
        
        // Should work even if users don't have access (empty results cached)
        assert_eq!(messages1.len(), 0);
        assert_eq!(messages2.len(), 0);
    }
}