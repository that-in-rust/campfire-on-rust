use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use thiserror::Error;
use tokio::sync::RwLock;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};

use crate::models::{UserId, RoomId, MessageId, Message, User, InvolvementLevel, Session};
use crate::services::search::{SearchResponse, SearchRequest};

/// Cache-specific errors
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache operation failed: {reason}")]
    OperationFailed { reason: String },
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Cache entry expired")]
    Expired,
    
    #[error("Cache entry not found")]
    NotFound,
}

/// Cache entry with TTL support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, ttl: Option<StdDuration>) -> Self {
        let now = Utc::now();
        let expires_at = ttl.map(|duration| {
            now + Duration::from_std(duration).unwrap_or(Duration::seconds(3600))
        });
        
        Self {
            data,
            created_at: now,
            expires_at,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    pub fn into_data(self) -> Result<T, CacheError> {
        if self.is_expired() {
            Err(CacheError::Expired)
        } else {
            Ok(self.data)
        }
    }
}

/// Cache key types for type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
    UserSession(String),
    RoomMembership(RoomId, UserId),
    UserRooms(UserId),
    MessageHistory(RoomId, u32, Option<MessageId>), // room_id, limit, before
    SearchResult(String), // Hashed search query
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheKey::UserSession(token) => write!(f, "session:{}", &token[..8]),
            CacheKey::RoomMembership(room_id, user_id) => write!(f, "membership:{}:{}", room_id, user_id),
            CacheKey::UserRooms(user_id) => write!(f, "user_rooms:{}", user_id),
            CacheKey::MessageHistory(room_id, limit, before) => {
                if let Some(before_id) = before {
                    write!(f, "messages:{}:{}:{}", room_id, limit, before_id)
                } else {
                    write!(f, "messages:{}:{}", room_id, limit)
                }
            }
            CacheKey::SearchResult(query_hash) => write!(f, "search:{}", query_hash),
        }
    }
}

/// Cached data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheValue {
    UserSession(User),
    RoomMembership(Option<InvolvementLevel>),
    UserRooms(Vec<crate::models::Room>),
    MessageHistory(Vec<Message>),
    SearchResult(SearchResponse),
}

/// Cache service trait for frequently accessed data
#[async_trait]
pub trait CacheServiceTrait: Send + Sync {
    /// Session caching
    async fn get_cached_session(&self, token: &str) -> Result<Option<User>, CacheError>;
    async fn cache_session(&self, token: String, user: User, ttl: StdDuration) -> Result<(), CacheError>;
    async fn invalidate_session(&self, token: &str) -> Result<(), CacheError>;
    
    /// Room membership caching
    async fn get_cached_membership(&self, room_id: RoomId, user_id: UserId) -> Result<Option<Option<InvolvementLevel>>, CacheError>;
    async fn cache_membership(&self, room_id: RoomId, user_id: UserId, level: Option<InvolvementLevel>, ttl: StdDuration) -> Result<(), CacheError>;
    async fn invalidate_membership(&self, room_id: RoomId, user_id: UserId) -> Result<(), CacheError>;
    async fn invalidate_room_memberships(&self, room_id: RoomId) -> Result<(), CacheError>;
    
    /// Message history caching for active rooms
    async fn get_cached_messages(&self, room_id: RoomId, limit: u32, before: Option<MessageId>) -> Result<Option<Vec<Message>>, CacheError>;
    async fn cache_messages(&self, room_id: RoomId, limit: u32, before: Option<MessageId>, messages: Vec<Message>, ttl: StdDuration) -> Result<(), CacheError>;
    async fn invalidate_room_messages(&self, room_id: RoomId) -> Result<(), CacheError>;
    
    /// Search result caching with TTL
    async fn get_cached_search(&self, request: &SearchRequest) -> Result<Option<SearchResponse>, CacheError>;
    async fn cache_search(&self, request: &SearchRequest, response: SearchResponse, ttl: StdDuration) -> Result<(), CacheError>;
    async fn invalidate_search_cache(&self) -> Result<(), CacheError>;
    
    /// Cache statistics and management
    async fn get_cache_stats(&self) -> CacheStats;
    async fn clear_expired_entries(&self) -> Result<u64, CacheError>;
    async fn clear_all_cache(&self) -> Result<(), CacheError>;
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub session_cache_size: u64,
    pub membership_cache_size: u64,
    pub message_cache_size: u64,
    pub search_cache_size: u64,
    pub total_entries: u64,
    pub hit_rate: f64,
    pub memory_usage_bytes: u64,
}

/// High-performance in-memory cache implementation
pub struct CacheService {
    // Session cache with automatic TTL
    session_cache: Cache<String, CacheEntry<User>>,
    
    // Room membership cache
    membership_cache: Cache<(RoomId, UserId), CacheEntry<Option<InvolvementLevel>>>,
    
    // Message history cache for active rooms
    message_cache: Cache<(RoomId, u32, Option<MessageId>), CacheEntry<Vec<Message>>>,
    
    // Search result cache with TTL
    search_cache: Cache<String, CacheEntry<SearchResponse>>,
    
    // Cache invalidation tracking
    invalidation_tracker: Arc<RwLock<DashMap<RoomId, DateTime<Utc>>>>,
    
    // Cache hit/miss statistics
    stats: Arc<RwLock<CacheStatsInternal>>,
}

#[derive(Debug, Default)]
struct CacheStatsInternal {
    hits: u64,
    misses: u64,
}

impl CacheService {
    /// Create a new cache service with configurable sizes and TTLs
    pub fn new(
        session_cache_size: u64,
        membership_cache_size: u64,
        message_cache_size: u64,
        search_cache_size: u64,
    ) -> Self {
        Self {
            session_cache: Cache::builder()
                .max_capacity(session_cache_size)
                .time_to_live(StdDuration::from_secs(3600)) // 1 hour default TTL
                .build(),
            
            membership_cache: Cache::builder()
                .max_capacity(membership_cache_size)
                .time_to_live(StdDuration::from_secs(1800)) // 30 minutes default TTL
                .build(),
            
            message_cache: Cache::builder()
                .max_capacity(message_cache_size)
                .time_to_live(StdDuration::from_secs(300)) // 5 minutes default TTL
                .build(),
            
            search_cache: Cache::builder()
                .max_capacity(search_cache_size)
                .time_to_live(StdDuration::from_secs(600)) // 10 minutes default TTL
                .build(),
            
            invalidation_tracker: Arc::new(RwLock::new(DashMap::new())),
            stats: Arc::new(RwLock::new(CacheStatsInternal::default())),
        }
    }
    
    /// Create cache service with default configuration
    pub fn with_defaults() -> Self {
        Self::new(
            10_000,  // 10K sessions
            50_000,  // 50K memberships
            1_000,   // 1K message history entries
            5_000,   // 5K search results
        )
    }
    
    /// Generate hash for search query to use as cache key
    fn hash_search_query(&self, request: &SearchRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        request.query.hash(&mut hasher);
        request.limit.hash(&mut hasher);
        request.offset.hash(&mut hasher);
        request.room_id.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    /// Record cache hit
    async fn record_hit(&self) {
        let mut stats = self.stats.write().await;
        stats.hits += 1;
    }
    
    /// Record cache miss
    async fn record_miss(&self) {
        let mut stats = self.stats.write().await;
        stats.misses += 1;
    }
    
    /// Check if room data should be invalidated based on recent changes
    async fn should_invalidate_room(&self, room_id: RoomId) -> bool {
        let tracker = self.invalidation_tracker.read().await;
        if let Some(last_invalidation) = tracker.get(&room_id) {
            // Invalidate if last change was within the last 5 minutes
            let should_invalidate = Utc::now() - *last_invalidation < Duration::minutes(5);
            should_invalidate
        } else {
            false
        }
    }
    
    /// Mark room for invalidation
    async fn mark_room_invalidated(&self, room_id: RoomId) {
        let tracker = self.invalidation_tracker.write().await;
        tracker.insert(room_id, Utc::now());
    }
}

#[async_trait]
impl CacheServiceTrait for CacheService {
    async fn get_cached_session(&self, token: &str) -> Result<Option<User>, CacheError> {
        if let Some(entry) = self.session_cache.get(token).await {
            if entry.is_expired() {
                self.session_cache.remove(token).await;
                self.record_miss().await;
                Ok(None)
            } else {
                self.record_hit().await;
                Ok(Some(entry.data))
            }
        } else {
            self.record_miss().await;
            Ok(None)
        }
    }
    
    async fn cache_session(&self, token: String, user: User, ttl: StdDuration) -> Result<(), CacheError> {
        let entry = CacheEntry::new(user, Some(ttl));
        self.session_cache.insert(token, entry).await;
        Ok(())
    }
    
    async fn invalidate_session(&self, token: &str) -> Result<(), CacheError> {
        self.session_cache.remove(token).await;
        Ok(())
    }
    
    async fn get_cached_membership(&self, room_id: RoomId, user_id: UserId) -> Result<Option<Option<InvolvementLevel>>, CacheError> {
        // Check if room should be invalidated
        if self.should_invalidate_room(room_id).await {
            self.invalidate_room_memberships(room_id).await?;
            self.record_miss().await;
            return Ok(None);
        }
        
        if let Some(entry) = self.membership_cache.get(&(room_id, user_id)).await {
            if entry.is_expired() {
                self.membership_cache.remove(&(room_id, user_id)).await;
                self.record_miss().await;
                Ok(None)
            } else {
                self.record_hit().await;
                Ok(Some(entry.data))
            }
        } else {
            self.record_miss().await;
            Ok(None)
        }
    }
    
    async fn cache_membership(&self, room_id: RoomId, user_id: UserId, level: Option<InvolvementLevel>, ttl: StdDuration) -> Result<(), CacheError> {
        let entry = CacheEntry::new(level, Some(ttl));
        self.membership_cache.insert((room_id, user_id), entry).await;
        Ok(())
    }
    
    async fn invalidate_membership(&self, room_id: RoomId, user_id: UserId) -> Result<(), CacheError> {
        self.membership_cache.remove(&(room_id, user_id)).await;
        Ok(())
    }
    
    async fn invalidate_room_memberships(&self, room_id: RoomId) -> Result<(), CacheError> {
        // Mark room as invalidated
        self.mark_room_invalidated(room_id).await;
        
        // Remove all membership entries for this room
        // Note: moka doesn't have a prefix-based removal, so we'll rely on TTL and invalidation tracking
        // In a production system, you might want to use a different cache structure for this
        
        Ok(())
    }
    
    async fn get_cached_messages(&self, room_id: RoomId, limit: u32, before: Option<MessageId>) -> Result<Option<Vec<Message>>, CacheError> {
        // Check if room should be invalidated
        if self.should_invalidate_room(room_id).await {
            self.invalidate_room_messages(room_id).await?;
            self.record_miss().await;
            return Ok(None);
        }
        
        if let Some(entry) = self.message_cache.get(&(room_id, limit, before)).await {
            if entry.is_expired() {
                self.message_cache.remove(&(room_id, limit, before)).await;
                self.record_miss().await;
                Ok(None)
            } else {
                self.record_hit().await;
                Ok(Some(entry.data))
            }
        } else {
            self.record_miss().await;
            Ok(None)
        }
    }
    
    async fn cache_messages(&self, room_id: RoomId, limit: u32, before: Option<MessageId>, messages: Vec<Message>, ttl: StdDuration) -> Result<(), CacheError> {
        let entry = CacheEntry::new(messages, Some(ttl));
        self.message_cache.insert((room_id, limit, before), entry).await;
        Ok(())
    }
    
    async fn invalidate_room_messages(&self, room_id: RoomId) -> Result<(), CacheError> {
        // Mark room as invalidated for message cache
        self.mark_room_invalidated(room_id).await;
        
        // Note: Similar to membership cache, we rely on invalidation tracking
        // rather than prefix-based removal
        
        Ok(())
    }
    
    async fn get_cached_search(&self, request: &SearchRequest) -> Result<Option<SearchResponse>, CacheError> {
        let query_hash = self.hash_search_query(request);
        
        if let Some(entry) = self.search_cache.get(&query_hash).await {
            if entry.is_expired() {
                self.search_cache.remove(&query_hash).await;
                self.record_miss().await;
                Ok(None)
            } else {
                self.record_hit().await;
                Ok(Some(entry.data))
            }
        } else {
            self.record_miss().await;
            Ok(None)
        }
    }
    
    async fn cache_search(&self, request: &SearchRequest, response: SearchResponse, ttl: StdDuration) -> Result<(), CacheError> {
        let query_hash = self.hash_search_query(request);
        let entry = CacheEntry::new(response, Some(ttl));
        self.search_cache.insert(query_hash, entry).await;
        Ok(())
    }
    
    async fn invalidate_search_cache(&self) -> Result<(), CacheError> {
        self.search_cache.invalidate_all();
        Ok(())
    }
    
    async fn get_cache_stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        let total_requests = stats.hits + stats.misses;
        let hit_rate = if total_requests > 0 {
            stats.hits as f64 / total_requests as f64
        } else {
            0.0
        };
        
        CacheStats {
            session_cache_size: self.session_cache.entry_count(),
            membership_cache_size: self.membership_cache.entry_count(),
            message_cache_size: self.message_cache.entry_count(),
            search_cache_size: self.search_cache.entry_count(),
            total_entries: self.session_cache.entry_count() 
                + self.membership_cache.entry_count()
                + self.message_cache.entry_count()
                + self.search_cache.entry_count(),
            hit_rate,
            memory_usage_bytes: 0, // Would need more sophisticated tracking
        }
    }
    
    async fn clear_expired_entries(&self) -> Result<u64, CacheError> {
        // moka automatically handles TTL-based expiration
        // This method could trigger manual cleanup if needed
        let mut cleared = 0;
        
        // Trigger cleanup on all caches
        self.session_cache.run_pending_tasks().await;
        self.membership_cache.run_pending_tasks().await;
        self.message_cache.run_pending_tasks().await;
        self.search_cache.run_pending_tasks().await;
        
        Ok(cleared)
    }
    
    async fn clear_all_cache(&self) -> Result<(), CacheError> {
        self.session_cache.invalidate_all();
        self.membership_cache.invalidate_all();
        self.message_cache.invalidate_all();
        self.search_cache.invalidate_all();
        
        // Clear invalidation tracker
        let tracker = self.invalidation_tracker.write().await;
        tracker.clear();
        
        // Reset stats
        let mut stats = self.stats.write().await;
        *stats = CacheStatsInternal::default();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{User, RoomId, UserId, InvolvementLevel};
    use chrono::Utc;
    use std::time::Duration as StdDuration;
    
    fn create_test_user() -> User {
        User {
            id: UserId::new(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            bio: None,
            admin: false,
            bot_token: None,
            created_at: Utc::now(),
        }
    }
    
    #[tokio::test]
    async fn test_session_caching() {
        let cache = CacheService::with_defaults();
        let user = create_test_user();
        let token = "test_token".to_string();
        
        // Cache miss initially
        let result = cache.get_cached_session(&token).await.unwrap();
        assert!(result.is_none());
        
        // Cache the session
        cache.cache_session(token.clone(), user.clone(), StdDuration::from_secs(3600)).await.unwrap();
        
        // Cache hit
        let result = cache.get_cached_session(&token).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, user.id);
        
        // Invalidate session
        cache.invalidate_session(&token).await.unwrap();
        
        // Cache miss after invalidation
        let result = cache.get_cached_session(&token).await.unwrap();
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_membership_caching() {
        let cache = CacheService::with_defaults();
        let room_id = RoomId::new();
        let user_id = UserId::new();
        let level = Some(InvolvementLevel::Member);
        
        // Cache miss initially
        let result = cache.get_cached_membership(room_id, user_id).await.unwrap();
        assert!(result.is_none());
        
        // Cache the membership
        cache.cache_membership(room_id, user_id, level.clone(), StdDuration::from_secs(1800)).await.unwrap();
        
        // Cache hit
        let result = cache.get_cached_membership(room_id, user_id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), level);
        
        // Invalidate membership
        cache.invalidate_membership(room_id, user_id).await.unwrap();
        
        // Cache miss after invalidation
        let result = cache.get_cached_membership(room_id, user_id).await.unwrap();
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_cache_entry_expiration() {
        // Test with very short TTL
        let entry = CacheEntry::new("test_data".to_string(), Some(StdDuration::from_millis(1)));
        
        // Should not be expired immediately
        assert!(!entry.is_expired());
        
        // Wait for expiration
        tokio::time::sleep(StdDuration::from_millis(10)).await;
        
        // Should be expired now
        assert!(entry.is_expired());
        
        // Should return error when accessing expired data
        let result = entry.into_data();
        assert!(matches!(result, Err(CacheError::Expired)));
    }
    
    #[tokio::test]
    async fn test_cache_stats() {
        let cache = CacheService::with_defaults();
        let user = create_test_user();
        let token = "test_token".to_string();
        
        // Initial stats
        let stats = cache.get_cache_stats().await;
        assert_eq!(stats.total_entries, 0);
        
        // Cache miss (should increment miss counter)
        let _ = cache.get_cached_session(&token).await.unwrap();
        
        // Cache session
        cache.cache_session(token.clone(), user, StdDuration::from_secs(3600)).await.unwrap();
        
        // Cache hit (should increment hit counter)
        let _ = cache.get_cached_session(&token).await.unwrap();
        
        let stats = cache.get_cache_stats().await;
        assert_eq!(stats.session_cache_size, 1);
        assert!(stats.hit_rate > 0.0);
    }
    
    #[tokio::test]
    async fn test_search_query_hashing() {
        let cache = CacheService::with_defaults();
        
        let request1 = SearchRequest {
            query: "test query".to_string(),
            limit: Some(20),
            offset: Some(0),
            room_id: None,
        };
        
        let request2 = SearchRequest {
            query: "test query".to_string(),
            limit: Some(20),
            offset: Some(0),
            room_id: None,
        };
        
        let request3 = SearchRequest {
            query: "different query".to_string(),
            limit: Some(20),
            offset: Some(0),
            room_id: None,
        };
        
        let hash1 = cache.hash_search_query(&request1);
        let hash2 = cache.hash_search_query(&request2);
        let hash3 = cache.hash_search_query(&request3);
        
        // Same requests should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different requests should produce different hashes
        assert_ne!(hash1, hash3);
    }
}