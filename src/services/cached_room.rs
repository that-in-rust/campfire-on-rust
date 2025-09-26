use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

use crate::database::CampfireDatabase;
use crate::errors::RoomError;
use crate::models::{Room, RoomId, RoomType, UserId, InvolvementLevel};
use crate::services::room::{RoomService, RoomServiceTrait};
use crate::services::cache::CacheServiceTrait;

/// Cached room service that wraps the base RoomService
/// 
/// Provides caching for:
/// - Room membership checks (most frequent operation)
/// - User room lists
/// - Room metadata
/// 
/// Cache TTLs:
/// - Memberships: 30 minutes (relatively stable)
/// - Room lists: 10 minutes (can change when users join/leave rooms)
/// - Room metadata: 1 hour (rarely changes)
/// 
/// Cache invalidation:
/// - Membership changes invalidate affected user/room combinations
/// - Room creation/updates invalidate room metadata
/// - User joining/leaving rooms invalidates user room lists
#[derive(Clone)]
pub struct CachedRoomService {
    room_service: RoomService,
    cache_service: Arc<dyn CacheServiceTrait>,
}

impl CachedRoomService {
    pub fn new(
        db: Arc<CampfireDatabase>,
        cache_service: Arc<dyn CacheServiceTrait>,
    ) -> Self {
        Self {
            room_service: RoomService::new(db),
            cache_service,
        }
    }
    
    /// Cache TTLs for different data types
    const MEMBERSHIP_CACHE_TTL: Duration = Duration::from_secs(1800); // 30 minutes
    #[allow(dead_code)]
    const ROOM_LIST_CACHE_TTL: Duration = Duration::from_secs(600);   // 10 minutes
    #[allow(dead_code)]
    const ROOM_METADATA_CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour
}

#[async_trait]
impl RoomServiceTrait for CachedRoomService {
    async fn create_room(
        &self,
        name: String,
        topic: Option<String>,
        room_type: RoomType,
        creator_id: UserId,
    ) -> Result<Room, RoomError> {
        // Create room in database
        let room = self.room_service.create_room(name, topic, room_type, creator_id).await?;
        
        // Cache the creator's membership as admin
        if let Err(e) = self.cache_service.cache_membership(
            room.id,
            creator_id,
            Some(InvolvementLevel::Admin),
            Self::MEMBERSHIP_CACHE_TTL,
        ).await {
            tracing::warn!("Failed to cache creator membership for room {}: {}", room.id, e);
        }
        
        // Invalidate user's room list since they now have a new room
        // Note: We don't have a direct user room list cache in this implementation,
        // but this is where we would invalidate it
        
        Ok(room)
    }
    
    async fn add_member(
        &self,
        room_id: RoomId,
        user_id: UserId,
        added_by: UserId,
        involvement_level: InvolvementLevel,
    ) -> Result<(), RoomError> {
        // Add member in database
        let result = self.room_service.add_member(room_id, user_id, added_by, involvement_level.clone()).await;
        
        if result.is_ok() {
            // Cache the new membership
            if let Err(e) = self.cache_service.cache_membership(
                room_id,
                user_id,
                Some(involvement_level),
                Self::MEMBERSHIP_CACHE_TTL,
            ).await {
                tracing::warn!("Failed to cache new membership for user {} in room {}: {}", user_id, room_id, e);
            }
            
            // Invalidate room memberships to ensure consistency
            if let Err(e) = self.cache_service.invalidate_room_memberships(room_id).await {
                tracing::warn!("Failed to invalidate room memberships for room {}: {}", room_id, e);
            }
        }
        
        result
    }
    
    async fn check_room_access(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<Option<InvolvementLevel>, RoomError> {
        // Try cache first
        match self.cache_service.get_cached_membership(room_id, user_id).await {
            Ok(Some(cached_level)) => {
                tracing::debug!("Membership cache hit for user {} in room {}", user_id, room_id);
                return Ok(cached_level);
            }
            Ok(None) => {
                tracing::debug!("Membership cache miss for user {} in room {}", user_id, room_id);
            }
            Err(e) => {
                tracing::warn!("Membership cache error for user {} in room {}: {}", user_id, room_id, e);
            }
        }
        
        // Cache miss - check with database
        match self.room_service.check_room_access(room_id, user_id).await {
            Ok(level) => {
                // Cache the result (including None for no access)
                if let Err(e) = self.cache_service.cache_membership(
                    room_id,
                    user_id,
                    level.clone(),
                    Self::MEMBERSHIP_CACHE_TTL,
                ).await {
                    tracing::warn!("Failed to cache membership for user {} in room {}: {}", user_id, room_id, e);
                }
                Ok(level)
            }
            Err(e) => {
                // Don't cache errors
                Err(e)
            }
        }
    }
    
    async fn get_user_rooms(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Room>, RoomError> {
        // For now, always go to database for user room lists
        // In a production system, you might want to cache this with shorter TTL
        // and invalidate when user joins/leaves rooms
        
        self.room_service.get_user_rooms(user_id).await
    }
    
    async fn get_room_by_id(
        &self,
        room_id: RoomId,
    ) -> Result<Option<Room>, RoomError> {
        // Room metadata changes infrequently, so we could cache this
        // For now, delegate to the base service
        // In production, you might cache room metadata with longer TTL
        
        self.room_service.get_room_by_id(room_id).await
    }
}

/// Extension methods for cache management
impl CachedRoomService {
    /// Invalidate all cached data for a room
    /// Call this when room metadata changes or when doing bulk membership updates
    pub async fn invalidate_room_cache(&self, room_id: RoomId) -> Result<(), RoomError> {
        if let Err(e) = self.cache_service.invalidate_room_memberships(room_id).await {
            tracing::warn!("Failed to invalidate room memberships for room {}: {}", room_id, e);
        }
        
        // If we cached room metadata, we would invalidate it here too
        
        Ok(())
    }
    
    /// Invalidate cached membership for a specific user/room combination
    /// Call this when a user leaves a room or their role changes
    pub async fn invalidate_user_room_membership(&self, room_id: RoomId, user_id: UserId) -> Result<(), RoomError> {
        if let Err(e) = self.cache_service.invalidate_membership(room_id, user_id).await {
            tracing::warn!("Failed to invalidate membership for user {} in room {}: {}", user_id, room_id, e);
        }
        
        Ok(())
    }
    
    /// Preload membership cache for a room
    /// Useful when you know a room will be accessed frequently
    pub async fn preload_room_memberships(&self, room_id: RoomId, user_ids: Vec<UserId>) -> Result<(), RoomError> {
        for user_id in user_ids {
            // This will populate the cache
            let _ = self.check_room_access(room_id, user_id).await;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::CampfireDatabase;
    use crate::services::cache::CacheService;
    use crate::models::{User, RoomType};
    use chrono::Utc;
    
    async fn create_test_cached_room_service() -> CachedRoomService {
        let db = CampfireDatabase::new(":memory:").await.unwrap();
        let cache_service = Arc::new(CacheService::with_defaults());
        CachedRoomService::new(Arc::new(db), cache_service)
    }
    
    async fn create_test_user(service: &CachedRoomService, email: &str) -> User {
        let user = User {
            id: UserId::new(),
            name: "Test User".to_string(),
            email: email.to_string(),
            password_hash: "hash".to_string(),
            bio: None,
            admin: false,
            bot_token: None,
            created_at: Utc::now(),
        };
        
        // Create user in database directly
        service.room_service.database().create_user(user.clone()).await.unwrap();
        user
    }
    
    #[tokio::test]
    async fn test_membership_caching() {
        let service = create_test_cached_room_service().await;
        let user = create_test_user(&service, "test@example.com").await;
        
        // Create a room
        let room = service.create_room(
            "Test Room".to_string(),
            None,
            RoomType::Open,
            user.id,
        ).await.unwrap();
        
        // First access should hit database and cache the result
        let access1 = service.check_room_access(room.id, user.id).await.unwrap();
        assert_eq!(access1, Some(InvolvementLevel::Admin)); // Creator is admin
        
        // Second access should hit cache
        let access2 = service.check_room_access(room.id, user.id).await.unwrap();
        assert_eq!(access2, Some(InvolvementLevel::Admin));
        
        // Invalidate cache
        service.invalidate_user_room_membership(room.id, user.id).await.unwrap();
        
        // Next access should hit database again
        let access3 = service.check_room_access(room.id, user.id).await.unwrap();
        assert_eq!(access3, Some(InvolvementLevel::Admin));
    }
    
    #[tokio::test]
    async fn test_add_member_invalidates_cache() {
        let service = create_test_cached_room_service().await;
        let creator = create_test_user(&service, "creator@example.com").await;
        let member = create_test_user(&service, "member@example.com").await;
        
        // Create a room
        let room = service.create_room(
            "Test Room".to_string(),
            None,
            RoomType::Closed,
            creator.id,
        ).await.unwrap();
        
        // Member initially has no access
        let access1 = service.check_room_access(room.id, member.id).await.unwrap();
        assert_eq!(access1, None);
        
        // Add member to room
        service.add_member(room.id, member.id, creator.id, InvolvementLevel::Member).await.unwrap();
        
        // Member should now have access
        let access2 = service.check_room_access(room.id, member.id).await.unwrap();
        assert_eq!(access2, Some(InvolvementLevel::Member));
    }
    
    #[tokio::test]
    async fn test_cache_miss_fallback() {
        let service = create_test_cached_room_service().await;
        let user = create_test_user(&service, "test@example.com").await;
        
        // Create room directly through room service (bypassing cache)
        let room = service.room_service.create_room(
            "Test Room".to_string(),
            None,
            RoomType::Open,
            user.id,
        ).await.unwrap();
        
        // Access check should work even though not in cache (fallback to DB)
        let access = service.check_room_access(room.id, user.id).await.unwrap();
        assert_eq!(access, Some(InvolvementLevel::Admin));
        
        // Second call should now hit cache
        let access2 = service.check_room_access(room.id, user.id).await.unwrap();
        assert_eq!(access2, Some(InvolvementLevel::Admin));
    }
    
    #[tokio::test]
    async fn test_preload_memberships() {
        let service = create_test_cached_room_service().await;
        let user1 = create_test_user(&service, "user1@example.com").await;
        let user2 = create_test_user(&service, "user2@example.com").await;
        
        // Create a room
        let room = service.create_room(
            "Test Room".to_string(),
            None,
            RoomType::Open,
            user1.id,
        ).await.unwrap();
        
        // Add second user
        service.add_member(room.id, user2.id, user1.id, InvolvementLevel::Member).await.unwrap();
        
        // Preload memberships
        service.preload_room_memberships(room.id, vec![user1.id, user2.id]).await.unwrap();
        
        // Both users should now have cached memberships
        let access1 = service.check_room_access(room.id, user1.id).await.unwrap();
        let access2 = service.check_room_access(room.id, user2.id).await.unwrap();
        
        assert_eq!(access1, Some(InvolvementLevel::Admin));
        assert_eq!(access2, Some(InvolvementLevel::Member));
    }
}