use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, Interval};
use tracing::{info, warn, error};

use crate::config::CacheConfig;
use crate::services::cache::{CacheService, CacheServiceTrait, CacheStats};
use crate::services::{
    CachedAuthService, CachedRoomService, CachedMessageService, CachedSearchService
};
use crate::database::CampfireDatabase;
use crate::services::connection::ConnectionManager;
use crate::services::room::RoomServiceTrait;
use crate::services::push::PushNotificationService;

/// Cache manager that coordinates all caching services and provides
/// centralized cache management, monitoring, and cleanup
pub struct CacheManager {
    cache_service: Arc<CacheService>,
    config: CacheConfig,
    cleanup_interval: Interval,
}

impl CacheManager {
    /// Create a new cache manager with the given configuration
    pub fn new(config: CacheConfig) -> Self {
        let cache_service = Arc::new(CacheService::new(
            config.session_cache_size,
            config.membership_cache_size,
            config.message_cache_size,
            config.search_cache_size,
        ));
        
        let cleanup_interval = interval(Duration::from_secs(config.cleanup_interval_secs));
        
        Self {
            cache_service,
            config,
            cleanup_interval,
        }
    }
    
    /// Get the underlying cache service
    pub fn cache_service(&self) -> Arc<CacheService> {
        Arc::clone(&self.cache_service)
    }
    
    /// Create cached auth service
    pub fn create_cached_auth_service(&self, db: Arc<CampfireDatabase>) -> CachedAuthService {
        CachedAuthService::new(db, self.cache_service.clone())
    }
    
    /// Create cached room service
    pub fn create_cached_room_service(&self, db: Arc<CampfireDatabase>) -> CachedRoomService {
        CachedRoomService::new(db, self.cache_service.clone())
    }
    
    /// Create cached message service
    pub fn create_cached_message_service(
        &self,
        db: Arc<CampfireDatabase>,
        connection_manager: Arc<dyn ConnectionManager>,
        room_service: Arc<dyn RoomServiceTrait>,
    ) -> CachedMessageService {
        CachedMessageService::new(db, connection_manager, room_service, self.cache_service.clone())
    }
    
    /// Create cached message service with push notifications
    pub fn create_cached_message_service_with_push(
        &self,
        db: Arc<CampfireDatabase>,
        connection_manager: Arc<dyn ConnectionManager>,
        room_service: Arc<dyn RoomServiceTrait>,
        push_service: Arc<dyn PushNotificationService>,
    ) -> CachedMessageService {
        CachedMessageService::with_push_service(
            db,
            connection_manager,
            room_service,
            push_service,
            self.cache_service.clone(),
        )
    }
    
    /// Create cached search service
    pub fn create_cached_search_service(
        &self,
        db: Arc<CampfireDatabase>,
        room_service: Arc<dyn RoomServiceTrait>,
    ) -> CachedSearchService {
        CachedSearchService::new(db, room_service, self.cache_service.clone())
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        self.cache_service.get_cache_stats().await
    }
    
    /// Start the cache cleanup background task
    pub async fn start_cleanup_task(mut self) {
        info!("Starting cache cleanup task with interval: {}s", self.config.cleanup_interval_secs);
        
        loop {
            self.cleanup_interval.tick().await;
            
            match self.perform_cleanup().await {
                Ok(cleaned_count) => {
                    if cleaned_count > 0 {
                        info!("Cache cleanup completed: {} entries removed", cleaned_count);
                    }
                }
                Err(e) => {
                    error!("Cache cleanup failed: {}", e);
                }
            }
        }
    }
    
    /// Perform cache cleanup and return number of entries cleaned
    async fn perform_cleanup(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let stats_before = self.cache_service.get_cache_stats().await;
        
        // Clear expired entries
        let cleaned_count = self.cache_service.clear_expired_entries().await?;
        
        let stats_after = self.cache_service.get_cache_stats().await;
        
        // Log cache statistics periodically
        if cleaned_count > 0 || stats_after.total_entries > 1000 {
            info!(
                "Cache stats - Total: {}, Hit rate: {:.2}%, Memory: {} bytes",
                stats_after.total_entries,
                stats_after.hit_rate * 100.0,
                stats_after.memory_usage_bytes
            );
        }
        
        // Warn if cache hit rate is low
        if stats_after.hit_rate < 0.5 && stats_after.total_entries > 100 {
            warn!(
                "Low cache hit rate: {:.2}% - consider adjusting cache sizes or TTLs",
                stats_after.hit_rate * 100.0
            );
        }
        
        Ok(cleaned_count)
    }
    
    /// Warm up caches with common data
    pub async fn warm_up_caches(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Warming up caches...");
        
        // This would typically involve:
        // 1. Loading popular search terms
        // 2. Preloading active room memberships
        // 3. Caching recent messages for active rooms
        // 4. Loading session data for active users
        
        // For now, just log that warmup is complete
        info!("Cache warmup completed");
        
        Ok(())
    }
    
    /// Clear all caches (for administrative purposes)
    pub async fn clear_all_caches(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        warn!("Clearing all caches");
        
        self.cache_service.clear_all_cache().await?;
        
        info!("All caches cleared");
        
        Ok(())
    }
    
    /// Get cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
    
    /// Check if caching is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    /// Get cache health status
    pub async fn get_health_status(&self) -> CacheHealthStatus {
        let stats = self.cache_service.get_cache_stats().await;
        
        let status = if !self.config.enabled {
            CacheHealth::Disabled
        } else if stats.total_entries == 0 {
            CacheHealth::Empty
        } else if stats.hit_rate < 0.3 {
            CacheHealth::Poor
        } else if stats.hit_rate < 0.7 {
            CacheHealth::Good
        } else {
            CacheHealth::Excellent
        };
        
        CacheHealthStatus {
            health: status,
            stats,
            config: self.config.clone(),
        }
    }
}

/// Cache health status for monitoring
#[derive(Debug, Clone)]
pub struct CacheHealthStatus {
    pub health: CacheHealth,
    pub stats: CacheStats,
    pub config: CacheConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CacheHealth {
    Disabled,
    Empty,
    Poor,      // < 30% hit rate
    Good,      // 30-70% hit rate
    Excellent, // > 70% hit rate
}

impl std::fmt::Display for CacheHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheHealth::Disabled => write!(f, "Disabled"),
            CacheHealth::Empty => write!(f, "Empty"),
            CacheHealth::Poor => write!(f, "Poor"),
            CacheHealth::Good => write!(f, "Good"),
            CacheHealth::Excellent => write!(f, "Excellent"),
        }
    }
}

/// Factory for creating cache manager from configuration
pub struct CacheManagerFactory;

impl CacheManagerFactory {
    /// Create cache manager from configuration
    pub fn create(config: CacheConfig) -> CacheManager {
        CacheManager::new(config)
    }
    
    /// Create cache manager with default configuration
    pub fn create_default() -> CacheManager {
        let config = CacheConfig {
            enabled: true,
            session_cache_size: 10_000,
            membership_cache_size: 50_000,
            message_cache_size: 1_000,
            search_cache_size: 5_000,
            session_ttl_secs: 1800,      // 30 minutes
            membership_ttl_secs: 1800,   // 30 minutes
            message_ttl_secs: 300,       // 5 minutes
            search_ttl_secs: 600,        // 10 minutes
            cleanup_interval_secs: 3600, // 1 hour
        };
        
        CacheManager::new(config)
    }
    
    /// Create disabled cache manager (for testing or when caching is disabled)
    pub fn create_disabled() -> CacheManager {
        let config = CacheConfig {
            enabled: false,
            session_cache_size: 0,
            membership_cache_size: 0,
            message_cache_size: 0,
            search_cache_size: 0,
            session_ttl_secs: 0,
            membership_ttl_secs: 0,
            message_ttl_secs: 0,
            search_ttl_secs: 0,
            cleanup_interval_secs: 3600,
        };
        
        CacheManager::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CacheConfig;
    
    #[tokio::test]
    async fn test_cache_manager_creation() {
        let config = CacheConfig {
            enabled: true,
            session_cache_size: 1000,
            membership_cache_size: 5000,
            message_cache_size: 100,
            search_cache_size: 500,
            session_ttl_secs: 1800,
            membership_ttl_secs: 1800,
            message_ttl_secs: 300,
            search_ttl_secs: 600,
            cleanup_interval_secs: 3600,
        };
        
        let manager = CacheManager::new(config.clone());
        
        assert!(manager.is_enabled());
        assert_eq!(manager.config().session_cache_size, 1000);
        assert_eq!(manager.config().membership_cache_size, 5000);
    }
    
    #[tokio::test]
    async fn test_cache_health_status() {
        let manager = CacheManagerFactory::create_default();
        
        let health = manager.get_health_status().await;
        
        // New cache should be empty
        assert_eq!(health.health, CacheHealth::Empty);
        assert_eq!(health.stats.total_entries, 0);
    }
    
    #[tokio::test]
    async fn test_disabled_cache_manager() {
        let manager = CacheManagerFactory::create_disabled();
        
        assert!(!manager.is_enabled());
        
        let health = manager.get_health_status().await;
        assert_eq!(health.health, CacheHealth::Disabled);
    }
    
    #[tokio::test]
    async fn test_cache_stats() {
        let manager = CacheManagerFactory::create_default();
        
        let stats = manager.get_cache_stats().await;
        
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }
    
    #[tokio::test]
    async fn test_clear_all_caches() {
        let manager = CacheManagerFactory::create_default();
        
        // Clear should succeed even with empty cache
        let result = manager.clear_all_caches().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_warm_up_caches() {
        let manager = CacheManagerFactory::create_default();
        
        // Warmup should succeed
        let result = manager.warm_up_caches().await;
        assert!(result.is_ok());
    }
}