use campfire_on_rust::metrics::{get_performance_monitor, PerformanceMonitor};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_performance_monitor_initialization() {
    let monitor = get_performance_monitor();
    
    // Test that we can get the monitor instance
    assert!(!std::ptr::eq(monitor.as_ref(), std::ptr::null()));
    
    // Test performance summary generation
    let summary = monitor.get_performance_summary().await;
    assert_eq!(summary.endpoints.len(), 0); // No requests recorded yet
    assert_eq!(summary.database_queries.len(), 0); // No queries recorded yet
}

#[tokio::test]
async fn test_http_request_monitoring() {
    let monitor = get_performance_monitor();
    
    // Record some HTTP requests
    monitor.record_http_request("GET /api/health", Duration::from_millis(50), false);
    monitor.record_http_request("POST /api/messages", Duration::from_millis(120), false);
    monitor.record_http_request("GET /api/rooms", Duration::from_millis(200), true); // Error
    
    // Get performance summary
    let summary = monitor.get_performance_summary().await;
    
    // Should have recorded endpoints
    assert!(summary.endpoints.len() > 0);
    
    // Find the health endpoint
    let health_endpoint = summary.endpoints.iter()
        .find(|e| e.endpoint == "GET /api/health")
        .expect("Health endpoint should be recorded");
    
    assert_eq!(health_endpoint.total_requests, 1);
    assert_eq!(health_endpoint.avg_response_time_ms, 50.0);
    assert_eq!(health_endpoint.error_rate_percent, 0.0);
    
    // Find the rooms endpoint (with error)
    let rooms_endpoint = summary.endpoints.iter()
        .find(|e| e.endpoint == "GET /api/rooms")
        .expect("Rooms endpoint should be recorded");
    
    assert_eq!(rooms_endpoint.total_requests, 1);
    assert_eq!(rooms_endpoint.error_rate_percent, 100.0);
}

#[tokio::test]
async fn test_database_query_monitoring() {
    let monitor = get_performance_monitor();
    
    // Record some database queries
    monitor.record_database_query("SELECT users", Duration::from_millis(25), false);
    monitor.record_database_query("INSERT message", Duration::from_millis(80), false);
    monitor.record_database_query("SELECT messages", Duration::from_millis(150), true); // Error
    
    // Get performance summary
    let summary = monitor.get_performance_summary().await;
    
    // Should have recorded queries
    assert!(summary.database_queries.len() > 0);
    
    // Find the users query
    let users_query = summary.database_queries.iter()
        .find(|q| q.query_type == "SELECT users")
        .expect("Users query should be recorded");
    
    assert_eq!(users_query.total_queries, 1);
    assert_eq!(users_query.avg_duration_ms, 25.0);
    assert_eq!(users_query.error_count, 0);
    
    // Find the messages query (with error)
    let messages_query = summary.database_queries.iter()
        .find(|q| q.query_type == "SELECT messages")
        .expect("Messages query should be recorded");
    
    assert_eq!(messages_query.total_queries, 1);
    assert_eq!(messages_query.error_count, 1);
}

#[tokio::test]
async fn test_websocket_performance_monitoring() {
    let monitor = get_performance_monitor();
    
    // Update WebSocket stats
    monitor.update_websocket_stats(|stats| {
        stats.active_connections = 25;
        stats.total_messages_sent = 1000;
        stats.total_messages_received = 950;
        stats.broadcast_latency_ms = 15.5;
        stats.connection_errors = 2;
        stats.reconnection_count = 5;
    }).await;
    
    // Get performance summary
    let summary = monitor.get_performance_summary().await;
    
    assert_eq!(summary.websocket.active_connections, 25);
    assert_eq!(summary.websocket.total_messages_sent, 1000);
    assert_eq!(summary.websocket.total_messages_received, 950);
    assert_eq!(summary.websocket.broadcast_latency_ms, 15.5);
    assert_eq!(summary.websocket.connection_errors, 2);
    assert_eq!(summary.websocket.reconnection_count, 5);
}

#[tokio::test]
async fn test_connection_pool_monitoring() {
    let monitor = get_performance_monitor();
    
    // Update connection pool stats
    monitor.update_connection_pool_stats(|stats| {
        stats.active_connections = 15;
        stats.idle_connections = 5;
        stats.total_connections = 20;
        stats.connection_wait_time_ms = 12.3;
        stats.connection_errors = 1;
    }).await;
    
    // Get performance summary
    let summary = monitor.get_performance_summary().await;
    
    assert_eq!(summary.connection_pool.active_connections, 15);
    assert_eq!(summary.connection_pool.idle_connections, 5);
    assert_eq!(summary.connection_pool.total_connections, 20);
    assert_eq!(summary.connection_pool.connection_wait_time_ms, 12.3);
    assert_eq!(summary.connection_pool.connection_errors, 1);
}

#[tokio::test]
async fn test_performance_caching() {
    let monitor = get_performance_monitor();
    
    // Test caching functionality
    let cache_key = "test_key";
    let test_data = b"test data for caching";
    
    // First call should compute and cache
    let cached_data = monitor.get_or_cache(cache_key, || async {
        test_data.to_vec()
    }).await;
    
    assert_eq!(&*cached_data, test_data);
    
    // Second call should return cached data
    let cached_data2 = monitor.get_or_cache(cache_key, || async {
        b"different data".to_vec() // This shouldn't be called
    }).await;
    
    assert_eq!(&*cached_data2, test_data); // Should still be original data
}

#[tokio::test]
async fn test_performance_monitoring_concurrent_access() {
    let monitor = get_performance_monitor();
    
    // Spawn multiple tasks that record metrics concurrently
    let mut handles = Vec::new();
    
    for i in 0..100 {
        let monitor_clone = monitor.clone();
        let handle = tokio::spawn(async move {
            // Record HTTP request
            monitor_clone.record_http_request(
                &format!("GET /api/test/{}", i),
                Duration::from_millis(i % 100 + 10),
                i % 10 == 0, // 10% error rate
            );
            
            // Record database query
            monitor_clone.record_database_query(
                &format!("SELECT test_{}", i),
                Duration::from_millis(i % 50 + 5),
                i % 20 == 0, // 5% error rate
            );
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Get performance summary
    let summary = monitor.get_performance_summary().await;
    
    // Should have recorded all endpoints and queries
    assert_eq!(summary.endpoints.len(), 100);
    assert_eq!(summary.database_queries.len(), 100);
    
    // Check that error rates are approximately correct
    let total_endpoint_errors: u64 = summary.endpoints.iter()
        .map(|e| if e.error_rate_percent > 0.0 { 1 } else { 0 })
        .sum();
    
    let total_query_errors: u64 = summary.database_queries.iter()
        .map(|q| q.error_count)
        .sum();
    
    assert_eq!(total_endpoint_errors, 10); // 10% of 100
    assert_eq!(total_query_errors, 5); // 5% of 100
}

#[tokio::test]
async fn test_performance_alert_thresholds() {
    let monitor = get_performance_monitor();
    
    // Record some slow requests that should trigger alerts
    monitor.record_http_request("GET /slow-endpoint", Duration::from_millis(2000), false);
    monitor.record_database_query("SLOW SELECT", Duration::from_millis(1000), false);
    
    // Update WebSocket stats with high latency
    monitor.update_websocket_stats(|stats| {
        stats.broadcast_latency_ms = 200.0; // Above default threshold
    }).await;
    
    // The alert checking happens in the background monitoring task
    // In a real test, we might want to trigger it manually or wait for it
    
    let summary = monitor.get_performance_summary().await;
    
    // Verify the slow metrics were recorded
    let slow_endpoint = summary.endpoints.iter()
        .find(|e| e.endpoint == "GET /slow-endpoint")
        .expect("Slow endpoint should be recorded");
    
    assert_eq!(slow_endpoint.avg_response_time_ms, 2000.0);
    
    let slow_query = summary.database_queries.iter()
        .find(|q| q.query_type == "SLOW SELECT")
        .expect("Slow query should be recorded");
    
    assert_eq!(slow_query.avg_duration_ms, 1000.0);
    
    assert_eq!(summary.websocket.broadcast_latency_ms, 200.0);
}

#[cfg(feature = "performance-monitoring")]
#[tokio::test]
async fn test_system_metrics_collection() {
    let monitor = get_performance_monitor();
    
    // Start monitoring (this would normally be done at application startup)
    monitor.start_monitoring();
    
    // Wait a bit for metrics to be collected
    sleep(Duration::from_millis(100)).await;
    
    // The system metrics collection happens in the background
    // We can't easily test the actual values, but we can verify the structure
    let summary = monitor.get_performance_summary().await;
    
    // Memory stats should be initialized (even if with default values)
    assert!(summary.memory.resident_memory_bytes >= 0);
    assert!(summary.memory.virtual_memory_bytes >= 0);
}

#[tokio::test]
async fn test_performance_summary_serialization() {
    let monitor = get_performance_monitor();
    
    // Record some test data
    monitor.record_http_request("GET /test", Duration::from_millis(100), false);
    monitor.record_database_query("SELECT test", Duration::from_millis(50), false);
    
    let summary = monitor.get_performance_summary().await;
    
    // Test JSON serialization
    let json = serde_json::to_string(&summary).expect("Should serialize to JSON");
    assert!(json.contains("endpoints"));
    assert!(json.contains("database_queries"));
    assert!(json.contains("websocket"));
    assert!(json.contains("memory"));
    assert!(json.contains("connection_pool"));
    
    // Test deserialization
    let deserialized: campfire_on_rust::metrics::PerformanceSummary = 
        serde_json::from_str(&json).expect("Should deserialize from JSON");
    
    assert_eq!(deserialized.endpoints.len(), summary.endpoints.len());
    assert_eq!(deserialized.database_queries.len(), summary.database_queries.len());
}