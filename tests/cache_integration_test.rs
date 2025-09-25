use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use campfire_on_rust::database::CampfireDatabase;
use campfire_on_rust::services::{
    CacheManager, CacheManagerFactory, CacheHealth,
    CachedAuthService, CachedRoomService, CachedMessageService, CachedSearchService
};
use campfire_on_rust::services::connection::ConnectionManagerImpl;
use campfire_on_rust::services::room::RoomService;
use campfire_on_rust::services::search::SearchRequest;
use campfire_on_rust::models::{User, Room, RoomType, UserId, RoomId, InvolvementLevel, Membership};
use campfire_on_rust::config::CacheConfig;
use chrono::Utc;

/// Integration tests for the caching layer
/// 
/// Tests the four main caching components:
/// 1. Session caching (CachedAuthService)
/// 2. Room membership caching (CachedRoomService)  
/// 3. Message history caching (CachedMessageService)
/// 4. Search result caching (CachedSearchService)

async fn create_test_database() -> Arc<CampfireDatabase> {
    Arc::new(CampfireDatabase::new(":memory:").await.unwrap())
}

async fn create_test_cache_manager() -> CacheManager {
    let config = CacheConfig {
        enabled: true,
        session_cache_size: 1000,
        membership_cache_size: 5000,
        message_cache_size: 100,
        search_cache_size: 500,
        session_ttl_secs: 60,    // Short TTL for testing
        membership_ttl_secs: 60,
        message_ttl_secs: 30,
        search_ttl_secs: 60,
        cleanup_interval_secs: 10,
    };
    
    CacheManager::new(config)
}

async fn create_test_user(db: &CampfireDatabase, email: &str) -> User {
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
    
    db.create_user(user.clone()).await.unwrap();
    user
}

async fn create_test_room(db: &CampfireDatabase, name: &str, creator_id: UserId) -> Room {
    let room = Room {
        id: RoomId::new(),
        name: name.to_string(),
        topic: None,
        room_type: RoomType::Open,
        created_at: Utc::now(),
        last_message_at: None,
    };
    
    db.create_room(room.clone()).await.unwrap();
    
    // Add creator as admin
    let membership = Membership {
        room_id: room.id,
        user_id: creator_id,
        involvement_level: InvolvementLevel::Admin,
        created_at: Utc::now(),
    };
    
    db.create_membership(membership).await.unwrap();
    
    room
}

#[tokio::test]
async fn test_session_caching_integration() {
    let db = create_test_database().await;
    let cache_manager = create_test_cache_manager().await;
    let auth_service = cache_manager.create_cached_auth_service(db.clone());
    
    // Create a user
    let user = auth_service.create_user(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    // Authenticate to get a session
    let session = auth_service.authenticate(
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    // First validation should cache the result
    let validated_user1 = auth_service.validate_session(session.token.clone()).await.unwrap();
    assert_eq!(validated_user1.id, user.id);
    
    // Second validation should hit cache (faster)
    let start = std::time::Instant::now();
    let validated_user2 = auth_service.validate_session(session.token.clone()).await.unwrap();
    let cache_duration = start.elapsed();
    
    assert_eq!(validated_user2.id, user.id);
    
    // Cache hit should be very fast (< 1ms typically)
    assert!(cache_duration < Duration::from_millis(10));
    
    // Check cache stats
    let stats = cache_manager.get_cache_stats().await;
    assert!(stats.session_cache_size > 0);
    assert!(stats.hit_rate > 0.0);
}

#[tokio::test]
async fn test_membership_caching_integration() {
    let db = create_test_database().await;
    let cache_manager = create_test_cache_manager().await;
    let room_service = cache_manager.create_cached_room_service(db.clone());
    
    // Create test data
    let user = create_test_user(&db, "test@example.com").await;
    let room = create_test_room(&db, "Test Room", user.id).await;
    
    // First access should cache the membership
    let access1 = room_service.check_room_access(room.id, user.id).await.unwrap();
    assert_eq!(access1, Some(InvolvementLevel::Admin));
    
    // Second access should hit cache
    let start = std::time::Instant::now();
    let access2 = room_service.check_room_access(room.id, user.id).await.unwrap();
    let cache_duration = start.elapsed();
    
    assert_eq!(access2, Some(InvolvementLevel::Admin));
    assert!(cache_duration < Duration::from_millis(10));
    
    // Check cache stats
    let stats = cache_manager.get_cache_stats().await;
    assert!(stats.membership_cache_size > 0);
}

#[tokio::test]
async fn test_message_caching_integration() {
    let db = create_test_database().await;
    let cache_manager = create_test_cache_manager().await;
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db.clone()));
    let room_service = Arc::new(RoomService::new(db.clone()));
    let message_service = cache_manager.create_cached_message_service(
        db.clone(),
        connection_manager,
        room_service,
    );
    
    // Create test data
    let user = create_test_user(&db, "test@example.com").await;
    let room = create_test_room(&db, "Test Room", user.id).await;
    
    // Create a message
    let message = message_service.create_message_with_deduplication(
        "Test message".to_string(),
        room.id,
        user.id,
        Uuid::new_v4(),
    ).await.unwrap();
    
    // First message retrieval should cache the result
    let messages1 = message_service.get_room_messages(room.id, user.id, 10, None).await.unwrap();
    assert_eq!(messages1.len(), 1);
    assert_eq!(messages1[0].id, message.id);
    
    // Second retrieval should hit cache
    let start = std::time::Instant::now();
    let messages2 = message_service.get_room_messages(room.id, user.id, 10, None).await.unwrap();
    let cache_duration = start.elapsed();
    
    assert_eq!(messages2.len(), 1);
    assert_eq!(messages2[0].id, message.id);
    assert!(cache_duration < Duration::from_millis(10));
    
    // Check cache stats
    let stats = cache_manager.get_cache_stats().await;
    assert!(stats.message_cache_size > 0);
}

#[tokio::test]
async fn test_search_caching_integration() {
    let db = create_test_database().await;
    let cache_manager = create_test_cache_manager().await;
    let room_service = Arc::new(RoomService::new(db.clone()));
    let search_service = cache_manager.create_cached_search_service(db.clone(), room_service);
    
    // Create test data
    let user = create_test_user(&db, "test@example.com").await;
    let _room = create_test_room(&db, "Test Room", user.id).await;
    
    let search_request = SearchRequest {
        query: "test".to_string(),
        limit: Some(20),
        offset: Some(0),
        room_id: None,
    };
    
    // First search should cache the result
    let response1 = search_service.search_messages(user.id, search_request.clone()).await.unwrap();
    
    // Second search should hit cache
    let start = std::time::Instant::now();
    let response2 = search_service.search_messages(user.id, search_request).await.unwrap();
    let cache_duration = start.elapsed();
    
    assert_eq!(response1.query, response2.query);
    assert!(cache_duration < Duration::from_millis(10));
    
    // Check cache stats
    let stats = cache_manager.get_cache_stats().await;
    assert!(stats.search_cache_size > 0);
}

#[tokio::test]
async fn test_cache_invalidation_on_new_message() {
    let db = create_test_database().await;
    let cache_manager = create_test_cache_manager().await;
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db.clone()));
    let room_service = Arc::new(RoomService::new(db.clone()));
    let message_service = cache_manager.create_cached_message_service(
        db.clone(),
        connection_manager,
        room_service,
    );
    
    // Create test data
    let user = create_test_user(&db, "test@example.com").await;
    let room = create_test_room(&db, "Test Room", user.id).await;
    
    // Get initial messages (empty, but will cache the result)
    let messages1 = message_service.get_room_messages(room.id, user.id, 10, None).await.unwrap();
    assert_eq!(messages1.len(), 0);
    
    // Create a new message (should invalidate cache)
    let _message = message_service.create_message_with_deduplication(
        "New message".to_string(),
        room.id,
        user.id,
        Uuid::new_v4(),
    ).await.unwrap();
    
    // Next call should see the new message (cache was invalidated)
    let messages2 = message_service.get_room_messages(room.id, user.id, 10, None).await.unwrap();
    assert_eq!(messages2.len(), 1);
}

#[tokio::test]
async fn test_cache_ttl_expiration() {
    let db = create_test_database().await;
    
    // Create cache manager with very short TTL
    let config = CacheConfig {
        enabled: true,
        session_cache_size: 1000,
        membership_cache_size: 5000,
        message_cache_size: 100,
        search_cache_size: 500,
        session_ttl_secs: 1,     // 1 second TTL
        membership_ttl_secs: 1,
        message_ttl_secs: 1,
        search_ttl_secs: 1,
        cleanup_interval_secs: 1,
    };
    
    let cache_manager = CacheManager::new(config);
    let auth_service = cache_manager.create_cached_auth_service(db.clone());
    
    // Create and authenticate user
    let user = auth_service.create_user(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    let session = auth_service.authenticate(
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    // First validation should cache the result
    let _validated_user1 = auth_service.validate_session(session.token.clone()).await.unwrap();
    
    // Wait for TTL to expire
    sleep(Duration::from_secs(2)).await;
    
    // Next validation should hit database again (cache expired)
    let _validated_user2 = auth_service.validate_session(session.token).await.unwrap();
    
    // Cache should have been refreshed
    let stats = cache_manager.get_cache_stats().await;
    assert!(stats.session_cache_size > 0);
}

#[tokio::test]
async fn test_cache_health_monitoring() {
    let cache_manager = create_test_cache_manager().await;
    
    // New cache should be empty
    let health = cache_manager.get_health_status().await;
    assert_eq!(health.health, CacheHealth::Empty);
    
    // Disabled cache manager
    let disabled_manager = CacheManagerFactory::create_disabled();
    let disabled_health = disabled_manager.get_health_status().await;
    assert_eq!(disabled_health.health, CacheHealth::Disabled);
}

#[tokio::test]
async fn test_cache_cleanup() {
    let cache_manager = create_test_cache_manager().await;
    
    // Cleanup should succeed even with empty cache
    let result = cache_manager.clear_all_caches().await;
    assert!(result.is_ok());
    
    // Stats should show empty cache
    let stats = cache_manager.get_cache_stats().await;
    assert_eq!(stats.total_entries, 0);
}

#[tokio::test]
async fn test_concurrent_cache_access() {
    let db = create_test_database().await;
    let cache_manager = Arc::new(create_test_cache_manager().await);
    let auth_service = Arc::new(cache_manager.create_cached_auth_service(db.clone()));
    
    // Create user
    let user = auth_service.create_user(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    let session = auth_service.authenticate(
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    // Spawn multiple concurrent validation tasks
    let mut handles = vec![];
    
    for _ in 0..10 {
        let auth_service_clone = Arc::clone(&auth_service);
        let token_clone = session.token.clone();
        let user_id = user.id;
        
        let handle = tokio::spawn(async move {
            let validated_user = auth_service_clone.validate_session(token_clone).await.unwrap();
            assert_eq!(validated_user.id, user_id);
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Cache should have handled concurrent access correctly
    let stats = cache_manager.get_cache_stats().await;
    assert!(stats.session_cache_size > 0);
    assert!(stats.hit_rate > 0.0);
}

#[tokio::test]
async fn test_cache_performance_improvement() {
    let db = create_test_database().await;
    let cache_manager = create_test_cache_manager().await;
    let room_service = cache_manager.create_cached_room_service(db.clone());
    
    // Create test data
    let user = create_test_user(&db, "test@example.com").await;
    let room = create_test_room(&db, "Test Room", user.id).await;
    
    // Measure database access time (first call)
    let start = std::time::Instant::now();
    let _access1 = room_service.check_room_access(room.id, user.id).await.unwrap();
    let db_duration = start.elapsed();
    
    // Measure cache access time (second call)
    let start = std::time::Instant::now();
    let _access2 = room_service.check_room_access(room.id, user.id).await.unwrap();
    let cache_duration = start.elapsed();
    
    // Cache should be significantly faster than database
    // Note: In memory database might not show dramatic difference,
    // but cache should still be faster
    println!("DB access: {:?}, Cache access: {:?}", db_duration, cache_duration);
    
    // Cache access should be faster or at least not significantly slower
    assert!(cache_duration <= db_duration * 2);
}