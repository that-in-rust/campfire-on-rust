use campfire_on_rust::{
    AppState, CampfireDatabase, AuthService, RoomService, MessageService,
    ConnectionManagerImpl, SearchService, PushNotificationServiceImpl,
    VapidConfig, BotServiceImpl, SetupServiceImpl, metrics
};
use campfire_on_rust::models::*;
use campfire_on_rust::services::OptimizedConnectionManager;
use campfire_on_rust::database::{OptimizedConnectionPool, PoolConfig};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_performance_monitoring_integration() {
    // Initialize metrics system
    metrics::init_metrics().expect("Failed to initialize metrics");
    
    // Create optimized database pool
    let pool_config = PoolConfig {
        max_connections: 10,
        min_connections: 2,
        ..Default::default()
    };
    
    let optimized_pool = OptimizedConnectionPool::new(":memory:", Some(pool_config))
        .await
        .expect("Failed to create optimized pool");
    
    // Create database with optimized pool
    let db = CampfireDatabase::new(":memory:")
        .await
        .expect("Failed to create database");
    
    // Create optimized connection manager
    let connection_manager = OptimizedConnectionManager::new(None);
    
    // Create services
    let auth_service = Arc::new(AuthService::new(Arc::new(db.clone())));
    let room_service = Arc::new(RoomService::new(Arc::new(db.clone())));
    let connection_manager_arc = Arc::new(connection_manager);
    let message_service = Arc::new(MessageService::new(
        Arc::new(db.clone()),
        connection_manager_arc.clone(),
        room_service.clone(),
    ));
    let search_service = Arc::new(SearchService::new(
        Arc::new(db.clone()),
        room_service.clone(),
    ));
    
    let vapid_config = VapidConfig {
        public_key: "test_public_key".to_string(),
        private_key: "test_private_key".to_string(),
        subject: "mailto:test@example.com".to_string(),
    };
    let push_service = Arc::new(PushNotificationServiceImpl::new(
        db.clone(),
        db.writer(),
        vapid_config,
    ));
    let bot_service = Arc::new(BotServiceImpl::new(
        Arc::new(db.clone()),
        db.writer(),
        message_service.clone(),
    ));
    let setup_service = Arc::new(SetupServiceImpl::new(db.clone()));
    let demo_service = Arc::new(campfire_on_rust::DemoServiceImpl::new(Arc::new(db.clone())));
    
    // Create app state
    let app_state = AppState {
        db: db.clone(),
        auth_service,
        room_service,
        message_service,
        search_service,
        push_service,
        bot_service,
        setup_service,
        demo_service,
    };
    
    // Test performance monitoring with actual operations
    
    // 1. Test HTTP request monitoring
    let monitor = metrics::get_performance_monitor();
    
    // Simulate HTTP requests
    monitor.record_http_request("GET /api/health", Duration::from_millis(25), false);
    monitor.record_http_request("POST /api/messages", Duration::from_millis(150), false);
    monitor.record_http_request("GET /api/rooms", Duration::from_millis(75), false);
    monitor.record_http_request("GET /api/search", Duration::from_millis(200), true); // Error
    
    // 2. Test database query monitoring
    monitor.record_database_query("SELECT users", Duration::from_millis(15), false);
    monitor.record_database_query("INSERT message", Duration::from_millis(45), false);
    monitor.record_database_query("SELECT messages", Duration::from_millis(120), false);
    monitor.record_database_query("SLOW QUERY", Duration::from_millis(500), true); // Slow + error
    
    // 3. Test WebSocket performance monitoring
    monitor.update_websocket_stats(|stats| {
        stats.active_connections = 50;
        stats.total_messages_sent = 1500;
        stats.total_messages_received = 1450;
        stats.broadcast_latency_ms = 25.5;
        stats.connection_errors = 3;
        stats.reconnection_count = 8;
    }).await;
    
    // 4. Test connection pool monitoring
    monitor.update_connection_pool_stats(|stats| {
        stats.active_connections = 8;
        stats.idle_connections = 2;
        stats.total_connections = 10;
        stats.connection_wait_time_ms = 5.2;
        stats.connection_errors = 1;
    }).await;
    
    // 5. Test caching functionality
    let cache_key = "test_performance_cache";
    let test_data = b"performance test data";
    
    let cached_data = monitor.get_or_cache(cache_key, || async {
        test_data.to_vec()
    }).await;
    
    assert_eq!(&*cached_data, test_data);
    
    // 6. Get comprehensive performance summary
    let summary = monitor.get_performance_summary().await;
    
    // Verify HTTP metrics
    assert_eq!(summary.endpoints.len(), 4);
    
    let health_endpoint = summary.endpoints.iter()
        .find(|e| e.endpoint == "GET /api/health")
        .expect("Health endpoint should be recorded");
    assert_eq!(health_endpoint.total_requests, 1);
    assert_eq!(health_endpoint.avg_response_time_ms, 25.0);
    assert_eq!(health_endpoint.error_rate_percent, 0.0);
    
    let search_endpoint = summary.endpoints.iter()
        .find(|e| e.endpoint == "GET /api/search")
        .expect("Search endpoint should be recorded");
    assert_eq!(search_endpoint.error_rate_percent, 100.0);
    
    // Verify database metrics
    assert_eq!(summary.database_queries.len(), 4);
    
    let users_query = summary.database_queries.iter()
        .find(|q| q.query_type == "SELECT users")
        .expect("Users query should be recorded");
    assert_eq!(users_query.avg_duration_ms, 15.0);
    assert_eq!(users_query.error_count, 0);
    
    let slow_query = summary.database_queries.iter()
        .find(|q| q.query_type == "SLOW QUERY")
        .expect("Slow query should be recorded");
    assert_eq!(slow_query.avg_duration_ms, 500.0);
    assert_eq!(slow_query.error_count, 1);
    assert_eq!(slow_query.slow_query_count, 1); // Should be marked as slow (>100ms threshold)
    
    // Verify WebSocket metrics
    assert_eq!(summary.websocket.active_connections, 50);
    assert_eq!(summary.websocket.total_messages_sent, 1500);
    assert_eq!(summary.websocket.total_messages_received, 1450);
    assert_eq!(summary.websocket.broadcast_latency_ms, 25.5);
    assert_eq!(summary.websocket.connection_errors, 3);
    assert_eq!(summary.websocket.reconnection_count, 8);
    
    // Verify connection pool metrics
    assert_eq!(summary.connection_pool.active_connections, 8);
    assert_eq!(summary.connection_pool.idle_connections, 2);
    assert_eq!(summary.connection_pool.total_connections, 10);
    assert_eq!(summary.connection_pool.connection_wait_time_ms, 5.2);
    assert_eq!(summary.connection_pool.connection_errors, 1);
    
    // Verify timestamp is recent
    let now = chrono::Utc::now();
    let time_diff = now.signed_duration_since(summary.timestamp);
    assert!(time_diff.num_seconds() < 10); // Should be within 10 seconds
}

#[tokio::test]
async fn test_optimized_database_pool_performance() {
    let pool_config = PoolConfig {
        max_connections: 20,
        min_connections: 5,
        acquire_timeout: Duration::from_secs(10),
        idle_timeout: Duration::from_secs(300),
        max_lifetime: Duration::from_secs(900),
        test_before_acquire: true,
        ..Default::default()
    };
    
    let pool = OptimizedConnectionPool::new(":memory:", Some(pool_config))
        .await
        .expect("Failed to create optimized pool");
    
    // Test basic operations
    pool.health_check().await.expect("Health check should pass");
    
    // Test monitored query execution
    let query = sqlx::query("CREATE TABLE test_performance (id INTEGER PRIMARY KEY, data TEXT)");
    pool.execute_monitored(query, "create_test_table")
        .await
        .expect("Should create table");
    
    // Test batch operations for performance
    let start = std::time::Instant::now();
    
    for i in 0..100 {
        let query = sqlx::query("INSERT INTO test_performance (data) VALUES (?)")
            .bind(format!("test_data_{}", i));
        
        pool.execute_monitored(query, "insert_test_data")
            .await
            .expect("Should insert data");
    }
    
    let insert_duration = start.elapsed();
    println!("100 inserts took: {:?}", insert_duration);
    
    // Test query performance
    let start = std::time::Instant::now();
    
    let query = sqlx::query_as::<_, (i64, String)>("SELECT id, data FROM test_performance ORDER BY id");
    let results = pool.fetch_all_monitored(query, "select_all_test_data")
        .await
        .expect("Should fetch all data");
    
    let select_duration = start.elapsed();
    println!("Select all took: {:?}", select_duration);
    
    assert_eq!(results.len(), 100);
    
    // Test database optimization
    pool.optimize().await.expect("Should optimize database");
    
    // Test statistics collection
    let stats = pool.get_stats().await.expect("Should get stats");
    println!("Database stats: {:?}", stats);
    
    assert!(stats.database_size_bytes > 0);
    assert_eq!(stats.connection_count, pool.config().max_connections);
}

#[tokio::test]
async fn test_optimized_connection_manager_performance() {
    use campfire_on_rust::services::optimized_connection::{ConnectionManagerConfig, OptimizedConnectionManager};
    use campfire_on_rust::services::ConnectionManager;
    use tokio::sync::mpsc;
    
    let config = ConnectionManagerConfig {
        max_connections_per_user: 5,
        presence_timeout: Duration::from_secs(30),
        typing_timeout: Duration::from_secs(5),
        cleanup_interval: Duration::from_secs(10),
        broadcast_cache_size: 1000,
        broadcast_cache_ttl: Duration::from_secs(30),
        connection_timeout: Duration::from_secs(120),
    };
    
    let manager = OptimizedConnectionManager::new(Some(config));
    
    // Test adding multiple connections
    let mut connections = Vec::new();
    let user_id = UserId::new();
    
    for i in 0..5 {
        let connection_id = ConnectionId::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        
        manager.add_connection(user_id, connection_id, sender)
            .await
            .expect("Should add connection");
        
        connections.push(connection_id);
    }
    
    // Test connection limit
    let (sender, _receiver) = mpsc::unbounded_channel();
    let result = manager.add_connection(user_id, ConnectionId::new(), sender).await;
    assert!(result.is_err()); // Should fail due to connection limit
    
    // Test presence tracking
    let room_id = RoomId::new();
    let presence = manager.get_room_presence(room_id).await.expect("Should get presence");
    assert!(presence.is_empty()); // No room membership yet
    
    // Test typing indicators
    manager.start_typing(user_id, room_id).await.expect("Should start typing");
    let typing_users = manager.get_typing_users(room_id).await.expect("Should get typing users");
    assert_eq!(typing_users.len(), 1);
    assert_eq!(typing_users[0], user_id);
    
    manager.stop_typing(user_id, room_id).await.expect("Should stop typing");
    let typing_users = manager.get_typing_users(room_id).await.expect("Should get typing users");
    assert!(typing_users.is_empty());
    
    // Test broadcast caching
    let message = WebSocketMessage::PresenceUpdate {
        room_id,
        online_users: vec![user_id],
    };
    
    // First broadcast should populate cache
    let result = manager.broadcast_to_room(room_id, message.clone()).await;
    // Should fail with no connections, but cache should be populated
    assert!(result.is_err());
    
    // Test connection removal
    for connection_id in connections {
        manager.remove_connection(connection_id)
            .await
            .expect("Should remove connection");
    }
    
    // Verify all connections removed
    let presence = manager.get_room_presence(room_id).await.expect("Should get presence");
    assert!(presence.is_empty());
}

#[tokio::test]
async fn test_performance_monitoring_concurrent_load() {
    let monitor = metrics::get_performance_monitor();
    
    // Spawn multiple tasks that simulate concurrent load
    let mut handles = Vec::new();
    
    for task_id in 0..50 {
        let monitor_clone = monitor.clone();
        let handle = tokio::spawn(async move {
            for i in 0..20 {
                // Simulate HTTP requests
                monitor_clone.record_http_request(
                    &format!("GET /api/endpoint/{}", task_id),
                    Duration::from_millis((i % 100) + 10),
                    i % 10 == 0, // 10% error rate
                );
                
                // Simulate database queries
                monitor_clone.record_database_query(
                    &format!("SELECT data_{}", task_id),
                    Duration::from_millis((i % 50) + 5),
                    i % 20 == 0, // 5% error rate
                );
                
                // Small delay to simulate real work
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task should complete successfully");
    }
    
    // Verify metrics were recorded correctly
    let summary = monitor.get_performance_summary().await;
    
    // Should have recorded all endpoints (50 tasks)
    assert_eq!(summary.endpoints.len(), 50);
    
    // Should have recorded all query types (50 tasks)
    assert_eq!(summary.database_queries.len(), 50);
    
    // Verify total request counts
    let total_requests: u64 = summary.endpoints.iter()
        .map(|e| e.total_requests)
        .sum();
    assert_eq!(total_requests, 50 * 20); // 50 tasks * 20 requests each
    
    // Verify total query counts
    let total_queries: u64 = summary.database_queries.iter()
        .map(|q| q.total_queries)
        .sum();
    assert_eq!(total_queries, 50 * 20); // 50 tasks * 20 queries each
    
    // Verify error rates are approximately correct
    let total_endpoint_errors: u64 = summary.endpoints.iter()
        .map(|e| if e.error_rate_percent > 0.0 { 
            (e.total_requests as f64 * e.error_rate_percent / 100.0) as u64 
        } else { 
            0 
        })
        .sum();
    
    let total_query_errors: u64 = summary.database_queries.iter()
        .map(|q| q.error_count)
        .sum();
    
    // Should be approximately 10% and 5% respectively
    assert!(total_endpoint_errors >= 90 && total_endpoint_errors <= 110); // ~100 errors (10% of 1000)
    assert!(total_query_errors >= 45 && total_query_errors <= 55); // ~50 errors (5% of 1000)
}

#[tokio::test]
async fn test_performance_optimization_cache() {
    let monitor = metrics::get_performance_monitor();
    
    // Test cache performance with concurrent access
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let monitor_clone = monitor.clone();
        let handle = tokio::spawn(async move {
            let cache_key = format!("test_key_{}", i % 3); // Only 3 unique keys
            let data = format!("test_data_{}", i);
            
            let cached_data = monitor_clone.get_or_cache(&cache_key, || async {
                data.into_bytes()
            }).await;
            
            // Verify data is cached
            assert!(!cached_data.is_empty());
        });
        handles.push(handle);
    }
    
    // Wait for all cache operations to complete
    for handle in handles {
        handle.await.expect("Cache operation should complete");
    }
    
    // Test cache hit/miss behavior
    let cache_key = "consistent_key";
    let original_data = b"original_data";
    
    // First access should cache the data
    let cached_data1 = monitor.get_or_cache(cache_key, || async {
        original_data.to_vec()
    }).await;
    
    assert_eq!(&*cached_data1, original_data);
    
    // Second access should return cached data (not compute new data)
    let cached_data2 = monitor.get_or_cache(cache_key, || async {
        b"different_data".to_vec() // This shouldn't be called
    }).await;
    
    assert_eq!(&*cached_data2, original_data); // Should still be original data
}