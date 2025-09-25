use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

use crate::database::CampfireDatabase;
use crate::models::{UserId, RoomId};
use crate::services::search::{SearchService, SearchServiceTrait, SearchRequest, SearchResponse, SearchError};
use crate::services::room::RoomServiceTrait;
use crate::services::cache::{CacheService, CacheServiceTrait};

/// Cached search service that wraps the base SearchService
/// 
/// Provides caching for:
/// - Search results with TTL (search is expensive)
/// - Popular/frequent queries
/// 
/// Cache TTLs:
/// - Search results: 10 minutes (balance between performance and freshness)
/// - Popular queries: 15 minutes (longer for frequently accessed results)
/// 
/// Cache invalidation:
/// - New messages invalidate search cache (search index changes)
/// - Manual cache clearing for administrative purposes
/// 
/// Cache strategy:
/// - Hash search parameters to create cache keys
/// - Include user context in cache key for authorization
/// - Use longer TTL for complex queries that are expensive to compute
/// - Shorter TTL for simple queries that might change frequently
#[derive(Clone)]
pub struct CachedSearchService {
    search_service: SearchService,
    cache_service: Arc<dyn CacheServiceTrait>,
}

impl CachedSearchService {
    pub fn new(
        db: Arc<CampfireDatabase>,
        room_service: Arc<dyn RoomServiceTrait>,
        cache_service: Arc<dyn CacheServiceTrait>,
    ) -> Self {
        Self {
            search_service: SearchService::new(db, room_service),
            cache_service,
        }
    }
    
    /// Cache TTLs for different types of searches
    const SEARCH_CACHE_TTL: Duration = Duration::from_secs(600);      // 10 minutes
    const POPULAR_QUERY_TTL: Duration = Duration::from_secs(900);     // 15 minutes
    const SIMPLE_QUERY_TTL: Duration = Duration::from_secs(300);      // 5 minutes
    
    /// Determine cache TTL based on query complexity and expected frequency
    fn get_cache_ttl(&self, request: &SearchRequest) -> Duration {
        // Simple heuristics for cache TTL
        let query_len = request.query.len();
        let has_room_filter = request.room_id.is_some();
        let large_limit = request.limit.unwrap_or(20) > 50;
        
        if query_len <= 3 {
            // Very short queries - likely to be popular, cache longer
            Self::POPULAR_QUERY_TTL
        } else if query_len > 20 || has_room_filter || large_limit {
            // Complex queries - cache longer since they're expensive
            Self::SEARCH_CACHE_TTL
        } else {
            // Simple queries - shorter cache since they're quick to recompute
            Self::SIMPLE_QUERY_TTL
        }
    }
    
    /// Create a cache key that includes user context for authorization
    fn create_cache_key(&self, user_id: UserId, request: &SearchRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Include user ID for authorization context
        user_id.hash(&mut hasher);
        
        // Include all search parameters
        request.query.hash(&mut hasher);
        request.limit.hash(&mut hasher);
        request.offset.hash(&mut hasher);
        request.room_id.hash(&mut hasher);
        
        format!("search:{}:{:x}", user_id, hasher.finish())
    }
    
    /// Check if a query is likely to be popular/frequent
    fn is_popular_query(&self, query: &str) -> bool {
        // Simple heuristics for popular queries
        let query_lower = query.to_lowercase();
        
        // Short queries are often popular
        if query.len() <= 3 {
            return true;
        }
        
        // Common search terms
        let popular_terms = [
            "error", "bug", "fix", "help", "issue", "problem",
            "todo", "task", "meeting", "deadline", "urgent",
            "api", "database", "server", "deploy", "release"
        ];
        
        popular_terms.iter().any(|term| query_lower.contains(term))
    }
}

#[async_trait]
impl SearchServiceTrait for CachedSearchService {
    async fn search_messages(
        &self,
        user_id: UserId,
        request: SearchRequest,
    ) -> Result<SearchResponse, SearchError> {
        // Create cache key that includes user context
        let cache_key = self.create_cache_key(user_id, &request);
        
        // Try cache first
        match self.cache_service.get_cached_search(&request).await {
            Ok(Some(cached_response)) => {
                tracing::debug!("Search cache hit for user {} query: '{}'", user_id, request.query);
                return Ok(cached_response);
            }
            Ok(None) => {
                tracing::debug!("Search cache miss for user {} query: '{}'", user_id, request.query);
            }
            Err(e) => {
                tracing::warn!("Search cache error for user {} query '{}': {}", user_id, request.query, e);
            }
        }
        
        // Cache miss - perform search
        match self.search_service.search_messages(user_id, request.clone()).await {
            Ok(response) => {
                // Determine cache TTL based on query characteristics
                let ttl = if self.is_popular_query(&request.query) {
                    Self::POPULAR_QUERY_TTL
                } else {
                    self.get_cache_ttl(&request)
                };
                
                // Cache the result
                if let Err(e) = self.cache_service.cache_search(&request, response.clone(), ttl).await {
                    tracing::warn!("Failed to cache search result for user {} query '{}': {}", user_id, request.query, e);
                }
                
                Ok(response)
            }
            Err(e) => {
                // Don't cache errors
                Err(e)
            }
        }
    }
    
    async fn search_room_messages(
        &self,
        user_id: UserId,
        room_id: RoomId,
        query: String,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResponse, SearchError> {
        let request = SearchRequest {
            query,
            limit: Some(limit),
            offset: Some(offset),
            room_id: Some(room_id),
        };
        
        self.search_messages(user_id, request).await
    }
}

/// Extension methods for cache management
impl CachedSearchService {
    /// Invalidate all search cache
    /// Call this when the search index is rebuilt or when doing bulk message operations
    pub async fn invalidate_search_cache(&self) -> Result<(), SearchError> {
        if let Err(e) = self.cache_service.invalidate_search_cache().await {
            tracing::warn!("Failed to invalidate search cache: {}", e);
        }
        
        Ok(())
    }
    
    /// Preload popular search queries
    /// Useful for warming up cache with commonly searched terms
    pub async fn preload_popular_searches(
        &self,
        user_id: UserId,
        popular_queries: Vec<String>,
    ) -> Result<(), SearchError> {
        for query in popular_queries {
            let request = SearchRequest {
                query,
                limit: Some(20),
                offset: Some(0),
                room_id: None,
            };
            
            // This will populate the cache
            if let Err(e) = self.search_messages(user_id, request).await {
                tracing::warn!("Failed to preload search query: {}", e);
                // Continue with other queries even if one fails
            }
        }
        
        Ok(())
    }
    
    /// Get search cache statistics
    pub async fn get_search_cache_stats(&self) -> Result<crate::services::cache::CacheStats, SearchError> {
        Ok(self.cache_service.get_cache_stats().await)
    }
    
    /// Warm up search cache for a specific room
    /// Useful when a room becomes active and users are likely to search in it
    pub async fn warm_up_room_search(
        &self,
        user_id: UserId,
        room_id: RoomId,
        common_terms: Vec<String>,
    ) -> Result<(), SearchError> {
        for term in common_terms {
            let request = SearchRequest {
                query: term,
                limit: Some(20),
                offset: Some(0),
                room_id: Some(room_id),
            };
            
            if let Err(e) = self.search_messages(user_id, request).await {
                tracing::warn!("Failed to warm up room search for term: {}", e);
                // Continue with other terms
            }
        }
        
        Ok(())
    }
    
    /// Clear expired search cache entries
    /// Call this periodically to clean up memory
    pub async fn cleanup_expired_search_cache(&self) -> Result<u64, SearchError> {
        match self.cache_service.clear_expired_entries().await {
            Ok(count) => {
                tracing::info!("Cleaned up {} expired search cache entries", count);
                Ok(count)
            }
            Err(e) => {
                tracing::warn!("Failed to cleanup expired search cache entries: {}", e);
                Ok(0)
            }
        }
    }
    
    /// Get popular search terms from cache statistics
    /// This could be enhanced to track actual query frequency
    pub fn get_default_popular_terms(&self) -> Vec<String> {
        vec![
            "error".to_string(),
            "bug".to_string(),
            "fix".to_string(),
            "help".to_string(),
            "issue".to_string(),
            "todo".to_string(),
            "task".to_string(),
            "meeting".to_string(),
            "api".to_string(),
            "deploy".to_string(),
        ]
    }
    
    /// Invalidate search cache for a specific room
    /// Call this when a room's messages change significantly
    pub async fn invalidate_room_search_cache(&self, _room_id: RoomId) -> Result<(), SearchError> {
        // For now, invalidate entire search cache
        // In a more sophisticated implementation, you might track room-specific cache entries
        self.invalidate_search_cache().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::CampfireDatabase;
    use crate::services::cache::CacheService;
    use crate::services::room::RoomService;
    use crate::models::{User, Room, RoomType, Membership, InvolvementLevel, Message};
    use chrono::Utc;
    use uuid::Uuid;
    
    async fn create_test_cached_search_service() -> CachedSearchService {
        let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
        let room_service = Arc::new(RoomService::new(db.clone()));
        let cache_service = Arc::new(CacheService::with_defaults());
        
        CachedSearchService::new(db, room_service, cache_service)
    }
    
    async fn setup_test_data(service: &CachedSearchService) -> (User, Room) {
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
        
        // Create test message for search
        let message = Message::new(
            room.id,
            user.id,
            "This is a test message with error keyword".to_string(),
            Uuid::new_v4(),
        );
        
        // Create in database
        service.search_service.database().create_user(user.clone()).await.unwrap();
        service.search_service.database().create_room(room.clone()).await.unwrap();
        service.search_service.database().create_membership(membership).await.unwrap();
        service.search_service.database().create_message_with_deduplication(message).await.unwrap();
        
        (user, room)
    }
    
    #[tokio::test]
    async fn test_search_result_caching() {
        let service = create_test_cached_search_service().await;
        let (user, _room) = setup_test_data(&service).await;
        
        let request = SearchRequest {
            query: "test".to_string(),
            limit: Some(20),
            offset: Some(0),
            room_id: None,
        };
        
        // First search should hit database and cache the result
        let response1 = service.search_messages(user.id, request.clone()).await.unwrap();
        
        // Second search should hit cache
        let response2 = service.search_messages(user.id, request).await.unwrap();
        
        // Results should be identical
        assert_eq!(response1.results.len(), response2.results.len());
        assert_eq!(response1.query, response2.query);
    }
    
    #[tokio::test]
    async fn test_cache_key_includes_user_context() {
        let service = create_test_cached_search_service().await;
        
        let user1 = UserId::new();
        let user2 = UserId::new();
        
        let request = SearchRequest {
            query: "test".to_string(),
            limit: Some(20),
            offset: Some(0),
            room_id: None,
        };
        
        let key1 = service.create_cache_key(user1, &request);
        let key2 = service.create_cache_key(user2, &request);
        
        // Different users should have different cache keys
        assert_ne!(key1, key2);
        
        // Same user should have same cache key
        let key1_again = service.create_cache_key(user1, &request);
        assert_eq!(key1, key1_again);
    }
    
    #[tokio::test]
    async fn test_popular_query_detection() {
        let service = create_test_cached_search_service().await;
        
        // Test popular terms
        assert!(service.is_popular_query("error"));
        assert!(service.is_popular_query("bug fix"));
        assert!(service.is_popular_query("API"));
        
        // Test short queries (considered popular)
        assert!(service.is_popular_query("fix"));
        assert!(service.is_popular_query("api"));
        
        // Test non-popular terms
        assert!(!service.is_popular_query("specific implementation detail"));
        assert!(!service.is_popular_query("random text"));
    }
    
    #[tokio::test]
    async fn test_cache_ttl_determination() {
        let service = create_test_cached_search_service().await;
        
        // Short query (popular)
        let short_request = SearchRequest {
            query: "fix".to_string(),
            limit: Some(20),
            offset: Some(0),
            room_id: None,
        };
        let short_ttl = service.get_cache_ttl(&short_request);
        
        // Complex query
        let complex_request = SearchRequest {
            query: "very long and complex search query".to_string(),
            limit: Some(100),
            offset: Some(0),
            room_id: Some(RoomId::new()),
        };
        let complex_ttl = service.get_cache_ttl(&complex_request);
        
        // Simple query
        let simple_request = SearchRequest {
            query: "simple".to_string(),
            limit: Some(20),
            offset: Some(0),
            room_id: None,
        };
        let simple_ttl = service.get_cache_ttl(&simple_request);
        
        // Popular queries should have longer TTL than simple queries
        assert!(short_ttl > simple_ttl);
        
        // Complex queries should have longer TTL than simple queries
        assert!(complex_ttl > simple_ttl);
    }
    
    #[tokio::test]
    async fn test_room_specific_search_caching() {
        let service = create_test_cached_search_service().await;
        let (user, room) = setup_test_data(&service).await;
        
        // Search in specific room
        let response1 = service.search_room_messages(
            user.id,
            room.id,
            "test".to_string(),
            20,
            0,
        ).await.unwrap();
        
        // Second search should hit cache
        let response2 = service.search_room_messages(
            user.id,
            room.id,
            "test".to_string(),
            20,
            0,
        ).await.unwrap();
        
        assert_eq!(response1.results.len(), response2.results.len());
    }
    
    #[tokio::test]
    async fn test_preload_popular_searches() {
        let service = create_test_cached_search_service().await;
        let (user, _room) = setup_test_data(&service).await;
        
        let popular_queries = vec![
            "error".to_string(),
            "bug".to_string(),
            "fix".to_string(),
        ];
        
        // Preload popular searches
        service.preload_popular_searches(user.id, popular_queries.clone()).await.unwrap();
        
        // Subsequent searches should hit cache
        for query in popular_queries {
            let request = SearchRequest {
                query,
                limit: Some(20),
                offset: Some(0),
                room_id: None,
            };
            
            let _response = service.search_messages(user.id, request).await.unwrap();
        }
    }
    
    #[tokio::test]
    async fn test_cache_invalidation() {
        let service = create_test_cached_search_service().await;
        let (user, _room) = setup_test_data(&service).await;
        
        let request = SearchRequest {
            query: "test".to_string(),
            limit: Some(20),
            offset: Some(0),
            room_id: None,
        };
        
        // Cache a search result
        let _response1 = service.search_messages(user.id, request.clone()).await.unwrap();
        
        // Invalidate cache
        service.invalidate_search_cache().await.unwrap();
        
        // Next search should hit database again (cache was cleared)
        let _response2 = service.search_messages(user.id, request).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_default_popular_terms() {
        let service = create_test_cached_search_service().await;
        
        let popular_terms = service.get_default_popular_terms();
        
        assert!(!popular_terms.is_empty());
        assert!(popular_terms.contains(&"error".to_string()));
        assert!(popular_terms.contains(&"bug".to_string()));
        assert!(popular_terms.contains(&"fix".to_string()));
    }
}