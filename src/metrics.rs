use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use metrics::{counter, gauge, histogram, describe_counter, describe_gauge, describe_histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tracing::{error, info, warn};
use tokio::sync::RwLock;
use dashmap::DashMap;
use parking_lot::Mutex;
use once_cell::sync::Lazy;

#[cfg(feature = "performance-monitoring")]
use sysinfo::{System, SystemExt, CpuExt, ProcessExt};

use crate::AppState;

/// Metrics recorder handle for Prometheus export
static mut PROMETHEUS_HANDLE: Option<PrometheusHandle> = None;

/// Global performance monitor instance
static PERFORMANCE_MONITOR: Lazy<Arc<PerformanceMonitor>> = Lazy::new(|| {
    Arc::new(PerformanceMonitor::new())
});

/// Performance monitoring and optimization system
pub struct PerformanceMonitor {
    /// System information collector
    #[cfg(feature = "performance-monitoring")]
    system: Arc<Mutex<System>>,
    
    /// Request timing cache for performance analysis
    request_timings: Arc<DashMap<String, RequestTimingStats>>,
    
    /// Database query performance tracking
    db_query_stats: Arc<DashMap<String, QueryPerformanceStats>>,
    
    /// WebSocket connection performance tracking
    websocket_stats: Arc<RwLock<WebSocketPerformanceStats>>,
    
    /// Memory usage tracking
    memory_stats: Arc<RwLock<MemoryStats>>,
    
    /// Performance alert thresholds
    alert_thresholds: Arc<RwLock<AlertThresholds>>,
    
    /// Performance optimization cache
    optimization_cache: Arc<moka::future::Cache<String, Arc<[u8]>>>,
    
    /// Connection pool metrics
    connection_pool_stats: Arc<RwLock<ConnectionPoolStats>>,
}

#[derive(Debug)]
pub struct RequestTimingStats {
    pub total_requests: AtomicU64,
    pub total_duration_ms: AtomicU64,
    pub min_duration_ms: AtomicU64,
    pub max_duration_ms: AtomicU64,
    pub error_count: AtomicU64,
}

#[derive(Debug)]
pub struct QueryPerformanceStats {
    pub query_count: AtomicU64,
    pub total_duration_ms: AtomicU64,
    pub slow_query_count: AtomicU64,
    pub error_count: AtomicU64,
    pub last_execution: AtomicU64, // timestamp
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebSocketPerformanceStats {
    pub active_connections: usize,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub broadcast_latency_ms: f64,
    pub connection_errors: u64,
    pub reconnection_count: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    pub heap_allocated_bytes: u64,
    pub heap_deallocated_bytes: u64,
    pub resident_memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub cache_hit_ratio: f64,
    pub gc_collections: u64,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_response_time_ms: u64,
    pub max_memory_usage_mb: u64,
    pub max_cpu_usage_percent: f64,
    pub max_error_rate_percent: f64,
    pub max_db_query_time_ms: u64,
    pub max_websocket_latency_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionPoolStats {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_connections: u32,
    pub connection_wait_time_ms: f64,
    pub connection_errors: u64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_response_time_ms: 1000,
            max_memory_usage_mb: 1024,
            max_cpu_usage_percent: 80.0,
            max_error_rate_percent: 5.0,
            max_db_query_time_ms: 500,
            max_websocket_latency_ms: 100,
        }
    }
}

impl RequestTimingStats {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
            min_duration_ms: AtomicU64::new(u64::MAX),
            max_duration_ms: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }
    
    pub fn record_request(&self, duration_ms: u64, is_error: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        
        // Update min duration
        let mut current_min = self.min_duration_ms.load(Ordering::Relaxed);
        while duration_ms < current_min {
            match self.min_duration_ms.compare_exchange_weak(
                current_min,
                duration_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_min) => current_min = new_min,
            }
        }
        
        // Update max duration
        let mut current_max = self.max_duration_ms.load(Ordering::Relaxed);
        while duration_ms > current_max {
            match self.max_duration_ms.compare_exchange_weak(
                current_max,
                duration_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_max) => current_max = new_max,
            }
        }
        
        if is_error {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    pub fn avg_duration_ms(&self) -> f64 {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        if total_requests == 0 {
            0.0
        } else {
            self.total_duration_ms.load(Ordering::Relaxed) as f64 / total_requests as f64
        }
    }
    
    pub fn error_rate(&self) -> f64 {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        if total_requests == 0 {
            0.0
        } else {
            (self.error_count.load(Ordering::Relaxed) as f64 / total_requests as f64) * 100.0
        }
    }
}

impl QueryPerformanceStats {
    pub fn new() -> Self {
        Self {
            query_count: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
            slow_query_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            last_execution: AtomicU64::new(0),
        }
    }
    
    pub fn record_query(&self, duration_ms: u64, is_error: bool, slow_threshold_ms: u64) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        self.last_execution.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            Ordering::Relaxed,
        );
        
        if duration_ms > slow_threshold_ms {
            self.slow_query_count.fetch_add(1, Ordering::Relaxed);
        }
        
        if is_error {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    pub fn avg_duration_ms(&self) -> f64 {
        let query_count = self.query_count.load(Ordering::Relaxed);
        if query_count == 0 {
            0.0
        } else {
            self.total_duration_ms.load(Ordering::Relaxed) as f64 / query_count as f64
        }
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "performance-monitoring")]
            system: Arc::new(Mutex::new(System::new_all())),
            request_timings: Arc::new(DashMap::new()),
            db_query_stats: Arc::new(DashMap::new()),
            websocket_stats: Arc::new(RwLock::new(WebSocketPerformanceStats::default())),
            memory_stats: Arc::new(RwLock::new(MemoryStats::default())),
            alert_thresholds: Arc::new(RwLock::new(AlertThresholds::default())),
            optimization_cache: Arc::new(
                moka::future::Cache::builder()
                    .max_capacity(10_000)
                    .time_to_live(Duration::from_secs(300))
                    .time_to_idle(Duration::from_secs(60))
                    .build()
            ),
            connection_pool_stats: Arc::new(RwLock::new(ConnectionPoolStats::default())),
        }
    }
    
    /// Start the performance monitoring background task
    pub fn start_monitoring(&self) {
        let monitor = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = monitor.collect_system_metrics().await {
                    warn!("Failed to collect system metrics: {}", e);
                }
                
                if let Err(e) = monitor.check_performance_alerts().await {
                    warn!("Failed to check performance alerts: {}", e);
                }
                
                monitor.update_prometheus_metrics().await;
            }
        });
    }
    
    /// Collect system-level performance metrics
    #[cfg(feature = "performance-monitoring")]
    async fn collect_system_metrics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Collect metrics in a blocking task to avoid Send issues
        let system_metrics = {
            let mut system = self.system.lock();
            system.refresh_all();
            
            // CPU usage
            let cpu_usage = system.global_cpu_info().cpu_usage();
            
            // Memory usage
            let total_memory = system.total_memory();
            let used_memory = system.used_memory();
            let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;
            
            // Process-specific metrics
            let process_metrics = system.processes().values().find(|p| {
                p.name().contains("campfire") || p.name().contains("rust")
            }).map(|process| {
                (
                    process.memory() as f64 * 1024.0,
                    process.virtual_memory() as f64 * 1024.0,
                    process.cpu_usage() as f64,
                )
            });
            
            (cpu_usage, total_memory, used_memory, memory_usage_percent, process_metrics)
        };
        
        let (cpu_usage, total_memory, used_memory, memory_usage_percent, process_metrics) = system_metrics;
        
        gauge!("system_cpu_usage_percent", cpu_usage as f64);
        gauge!("system_memory_total_bytes", total_memory as f64);
        gauge!("system_memory_used_bytes", used_memory as f64);
        gauge!("system_memory_usage_percent", memory_usage_percent);
        
        if let Some((process_memory, process_virtual_memory, process_cpu)) = process_metrics {
            gauge!("process_memory_bytes", process_memory);
            gauge!("process_virtual_memory_bytes", process_virtual_memory);
            gauge!("process_cpu_usage_percent", process_cpu);
        }
        
        // Update internal memory stats
        {
            let mut memory_stats = self.memory_stats.write().await;
            memory_stats.resident_memory_bytes = used_memory;
            memory_stats.virtual_memory_bytes = total_memory;
        }
        
        Ok(())
    }
    
    #[cfg(not(feature = "performance-monitoring"))]
    async fn collect_system_metrics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder metrics when sysinfo is not available
        gauge!("system_cpu_usage_percent", 0.0);
        gauge!("system_memory_usage_percent", 0.0);
        Ok(())
    }
    
    /// Check performance thresholds and generate alerts
    async fn check_performance_alerts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let thresholds = self.alert_thresholds.read().await;
        
        // Check request performance
        for entry in self.request_timings.iter() {
            let (endpoint, stats) = entry.pair();
            let avg_duration = stats.avg_duration_ms();
            let error_rate = stats.error_rate();
            
            if avg_duration > thresholds.max_response_time_ms as f64 {
                warn!(
                    "High response time alert: {} endpoint averaging {:.2}ms (threshold: {}ms)",
                    endpoint, avg_duration, thresholds.max_response_time_ms
                );
                counter!("performance_alerts_total", 1, "type" => "high_response_time", "endpoint" => endpoint.clone());
            }
            
            if error_rate > thresholds.max_error_rate_percent {
                warn!(
                    "High error rate alert: {} endpoint at {:.2}% errors (threshold: {:.2}%)",
                    endpoint, error_rate, thresholds.max_error_rate_percent
                );
                counter!("performance_alerts_total", 1, "type" => "high_error_rate", "endpoint" => endpoint.clone());
            }
        }
        
        // Check database query performance
        for entry in self.db_query_stats.iter() {
            let (query_type, stats) = entry.pair();
            let avg_duration = stats.avg_duration_ms();
            
            if avg_duration > thresholds.max_db_query_time_ms as f64 {
                warn!(
                    "Slow database query alert: {} averaging {:.2}ms (threshold: {}ms)",
                    query_type, avg_duration, thresholds.max_db_query_time_ms
                );
                counter!("performance_alerts_total", 1, "type" => "slow_database_query", "query_type" => query_type.clone());
            }
        }
        
        // Check WebSocket performance
        {
            let ws_stats = self.websocket_stats.read().await;
            if ws_stats.broadcast_latency_ms > thresholds.max_websocket_latency_ms as f64 {
                warn!(
                    "High WebSocket latency alert: {:.2}ms (threshold: {}ms)",
                    ws_stats.broadcast_latency_ms, thresholds.max_websocket_latency_ms
                );
                counter!("performance_alerts_total", 1, "type" => "high_websocket_latency");
            }
        }
        
        Ok(())
    }
    
    /// Update Prometheus metrics with current performance data
    async fn update_prometheus_metrics(&self) {
        // Update request timing metrics
        for entry in self.request_timings.iter() {
            let (endpoint, stats) = entry.pair();
            let total_requests = stats.total_requests.load(Ordering::Relaxed);
            let avg_duration = stats.avg_duration_ms();
            let error_rate = stats.error_rate();
            
            gauge!("http_request_avg_duration_ms", avg_duration, "endpoint" => endpoint.clone());
            gauge!("http_request_error_rate_percent", error_rate, "endpoint" => endpoint.clone());
            counter!("http_requests_processed_total", total_requests, "endpoint" => endpoint.clone());
        }
        
        // Update database metrics
        for entry in self.db_query_stats.iter() {
            let (query_type, stats) = entry.pair();
            let avg_duration = stats.avg_duration_ms();
            let query_count = stats.query_count.load(Ordering::Relaxed);
            let slow_queries = stats.slow_query_count.load(Ordering::Relaxed);
            
            gauge!("database_query_avg_duration_ms", avg_duration, "query_type" => query_type.clone());
            counter!("database_queries_processed_total", query_count, "query_type" => query_type.clone());
            counter!("database_slow_queries_total", slow_queries, "query_type" => query_type.clone());
        }
        
        // Update WebSocket metrics
        {
            let ws_stats = self.websocket_stats.read().await;
            gauge!("websocket_active_connections", ws_stats.active_connections as f64);
            gauge!("websocket_broadcast_latency_ms", ws_stats.broadcast_latency_ms);
            counter!("websocket_messages_sent_total", ws_stats.total_messages_sent);
            counter!("websocket_messages_received_total", ws_stats.total_messages_received);
            counter!("websocket_connection_errors_total", ws_stats.connection_errors);
            counter!("websocket_reconnections_total", ws_stats.reconnection_count);
        }
        
        // Update connection pool metrics
        {
            let pool_stats = self.connection_pool_stats.read().await;
            gauge!("connection_pool_active", pool_stats.active_connections as f64);
            gauge!("connection_pool_idle", pool_stats.idle_connections as f64);
            gauge!("connection_pool_total", pool_stats.total_connections as f64);
            gauge!("connection_pool_wait_time_ms", pool_stats.connection_wait_time_ms);
            counter!("connection_pool_errors_total", pool_stats.connection_errors);
        }
        
        // Update cache metrics
        let cache_stats = self.optimization_cache.weighted_size();
        gauge!("optimization_cache_size", cache_stats as f64);
    }
    
    /// Record HTTP request performance
    pub fn record_http_request(&self, endpoint: &str, duration: Duration, is_error: bool) {
        let duration_ms = duration.as_millis() as u64;
        
        let stats = self.request_timings
            .entry(endpoint.to_string())
            .or_insert_with(RequestTimingStats::new);
        
        stats.record_request(duration_ms, is_error);
        
        // Also record in Prometheus
        histogram!("http_request_duration_ms", duration_ms as f64, "endpoint" => endpoint.to_string());
        if is_error {
            counter!("http_request_errors_total", 1, "endpoint" => endpoint.to_string());
        }
    }
    
    /// Record database query performance
    pub fn record_database_query(&self, query_type: &str, duration: Duration, is_error: bool) {
        let duration_ms = duration.as_millis() as u64;
        
        let stats = self.db_query_stats
            .entry(query_type.to_string())
            .or_insert_with(QueryPerformanceStats::new);
        
        stats.record_query(duration_ms, is_error, 100); // 100ms slow query threshold
        
        // Also record in Prometheus
        histogram!("database_query_duration_ms", duration_ms as f64, "query_type" => query_type.to_string());
        if is_error {
            counter!("database_query_errors_total", 1, "query_type" => query_type.to_string());
        }
    }
    
    /// Update WebSocket performance stats
    pub async fn update_websocket_stats<F>(&self, updater: F)
    where
        F: FnOnce(&mut WebSocketPerformanceStats),
    {
        let mut stats = self.websocket_stats.write().await;
        updater(&mut *stats);
    }
    
    /// Update connection pool stats
    pub async fn update_connection_pool_stats<F>(&self, updater: F)
    where
        F: FnOnce(&mut ConnectionPoolStats),
    {
        let mut stats = self.connection_pool_stats.write().await;
        updater(&mut *stats);
    }
    
    /// Get cached data or compute and cache it
    pub async fn get_or_cache<F, Fut>(&self, key: &str, compute: F) -> Arc<[u8]>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Vec<u8>>,
    {
        if let Some(cached) = self.optimization_cache.get(key).await {
            return cached;
        }
        
        let computed = compute().await;
        let arc_data: Arc<[u8]> = computed.into();
        
        self.optimization_cache.insert(key.to_string(), arc_data.clone()).await;
        arc_data
    }
    
    /// Get performance summary for monitoring endpoints
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let mut endpoint_stats = Vec::new();
        for entry in self.request_timings.iter() {
            let (endpoint, stats) = entry.pair();
            endpoint_stats.push(EndpointPerformance {
                endpoint: endpoint.clone(),
                avg_response_time_ms: stats.avg_duration_ms(),
                total_requests: stats.total_requests.load(Ordering::Relaxed),
                error_rate_percent: stats.error_rate(),
                min_response_time_ms: stats.min_duration_ms.load(Ordering::Relaxed),
                max_response_time_ms: stats.max_duration_ms.load(Ordering::Relaxed),
            });
        }
        
        let mut query_stats = Vec::new();
        for entry in self.db_query_stats.iter() {
            let (query_type, stats) = entry.pair();
            query_stats.push(QueryPerformance {
                query_type: query_type.clone(),
                avg_duration_ms: stats.avg_duration_ms(),
                total_queries: stats.query_count.load(Ordering::Relaxed),
                slow_query_count: stats.slow_query_count.load(Ordering::Relaxed),
                error_count: stats.error_count.load(Ordering::Relaxed),
            });
        }
        
        let websocket_stats = self.websocket_stats.read().await.clone();
        let memory_stats = self.memory_stats.read().await.clone();
        let connection_pool_stats = self.connection_pool_stats.read().await.clone();
        
        PerformanceSummary {
            timestamp: chrono::Utc::now(),
            endpoints: endpoint_stats,
            database_queries: query_stats,
            websocket: websocket_stats,
            memory: memory_stats,
            connection_pool: connection_pool_stats,
        }
    }
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            #[cfg(feature = "performance-monitoring")]
            system: Arc::clone(&self.system),
            request_timings: Arc::clone(&self.request_timings),
            db_query_stats: Arc::clone(&self.db_query_stats),
            websocket_stats: Arc::clone(&self.websocket_stats),
            memory_stats: Arc::clone(&self.memory_stats),
            alert_thresholds: Arc::clone(&self.alert_thresholds),
            optimization_cache: Arc::clone(&self.optimization_cache),
            connection_pool_stats: Arc::clone(&self.connection_pool_stats),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub endpoints: Vec<EndpointPerformance>,
    pub database_queries: Vec<QueryPerformance>,
    pub websocket: WebSocketPerformanceStats,
    pub memory: MemoryStats,
    pub connection_pool: ConnectionPoolStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointPerformance {
    pub endpoint: String,
    pub avg_response_time_ms: f64,
    pub total_requests: u64,
    pub error_rate_percent: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryPerformance {
    pub query_type: String,
    pub avg_duration_ms: f64,
    pub total_queries: u64,
    pub slow_query_count: u64,
    pub error_count: u64,
}

/// Get global performance monitor instance
pub fn get_performance_monitor() -> Arc<PerformanceMonitor> {
    Arc::clone(&PERFORMANCE_MONITOR)
}

/// Initialize metrics system
pub fn init_metrics() -> Result<(), Box<dyn std::error::Error>> {
    // Build Prometheus recorder
    let builder = PrometheusBuilder::new();
    let handle = builder.install_recorder()?;
    
    unsafe {
        PROMETHEUS_HANDLE = Some(handle);
    }
    
    // Describe all metrics
    describe_metrics();
    
    // Start performance monitoring
    let monitor = get_performance_monitor();
    monitor.start_monitoring();
    
    info!("Metrics system and performance monitoring initialized");
    Ok(())
}

/// Describe all metrics for Prometheus
fn describe_metrics() {
    // HTTP metrics
    describe_counter!("http_requests_total", "Total number of HTTP requests");
    describe_histogram!("http_request_duration_seconds", "HTTP request duration in seconds");
    describe_counter!("http_requests_errors_total", "Total number of HTTP request errors");
    
    // WebSocket metrics
    describe_gauge!("websocket_connections_active", "Number of active WebSocket connections");
    describe_counter!("websocket_messages_sent_total", "Total WebSocket messages sent");
    describe_counter!("websocket_messages_received_total", "Total WebSocket messages received");
    
    // Database metrics
    describe_histogram!("database_query_duration_seconds", "Database query duration in seconds");
    describe_counter!("database_queries_total", "Total database queries");
    describe_counter!("database_errors_total", "Total database errors");
    describe_gauge!("database_connections_active", "Number of active database connections");
    
    // Message metrics
    describe_counter!("messages_created_total", "Total messages created");
    describe_counter!("messages_deduplicated_total", "Total messages deduplicated");
    describe_histogram!("message_processing_duration_seconds", "Message processing duration");
    
    // Room metrics
    describe_gauge!("rooms_total", "Total number of rooms");
    describe_gauge!("users_online", "Number of users currently online");
    
    // Push notification metrics
    describe_counter!("push_notifications_sent_total", "Total push notifications sent");
    describe_counter!("push_notifications_failed_total", "Total push notification failures");
    
    // System metrics
    describe_gauge!("memory_usage_bytes", "Memory usage in bytes");
    describe_gauge!("cpu_usage_percent", "CPU usage percentage");
    
    // Performance monitoring metrics
    describe_gauge!("system_cpu_usage_percent", "System CPU usage percentage");
    describe_gauge!("system_memory_usage_percent", "System memory usage percentage");
    describe_gauge!("system_memory_total_bytes", "Total system memory in bytes");
    describe_gauge!("system_memory_used_bytes", "Used system memory in bytes");
    describe_gauge!("process_memory_bytes", "Process memory usage in bytes");
    describe_gauge!("process_virtual_memory_bytes", "Process virtual memory usage in bytes");
    describe_gauge!("process_cpu_usage_percent", "Process CPU usage percentage");
    
    // HTTP performance metrics
    describe_histogram!("http_request_duration_ms", "HTTP request duration in milliseconds");
    describe_gauge!("http_request_avg_duration_ms", "Average HTTP request duration");
    describe_gauge!("http_request_error_rate_percent", "HTTP request error rate percentage");
    describe_counter!("http_requests_processed_total", "Total HTTP requests processed");
    describe_counter!("http_request_errors_total", "Total HTTP request errors");
    
    // Database performance metrics
    describe_histogram!("database_query_duration_ms", "Database query duration in milliseconds");
    describe_gauge!("database_query_avg_duration_ms", "Average database query duration");
    describe_counter!("database_queries_processed_total", "Total database queries processed");
    describe_counter!("database_slow_queries_total", "Total slow database queries");
    describe_counter!("database_query_errors_total", "Total database query errors");
    
    // WebSocket performance metrics
    describe_gauge!("websocket_active_connections", "Number of active WebSocket connections");
    describe_gauge!("websocket_broadcast_latency_ms", "WebSocket broadcast latency in milliseconds");
    describe_counter!("websocket_messages_sent_total", "Total WebSocket messages sent");
    describe_counter!("websocket_messages_received_total", "Total WebSocket messages received");
    describe_counter!("websocket_connection_errors_total", "Total WebSocket connection errors");
    describe_counter!("websocket_reconnections_total", "Total WebSocket reconnections");
    
    // Connection pool metrics
    describe_gauge!("connection_pool_active", "Active database connections");
    describe_gauge!("connection_pool_idle", "Idle database connections");
    describe_gauge!("connection_pool_total", "Total database connections");
    describe_gauge!("connection_pool_wait_time_ms", "Connection pool wait time in milliseconds");
    describe_counter!("connection_pool_errors_total", "Total connection pool errors");
    
    // Cache metrics
    describe_gauge!("optimization_cache_size", "Optimization cache size");
    
    // Performance alerts
    describe_counter!("performance_alerts_total", "Total performance alerts triggered");
}

/// Prometheus metrics endpoint
pub async fn metrics_endpoint() -> Result<Response, StatusCode> {
    unsafe {
        if let Some(handle) = PROMETHEUS_HANDLE.as_ref() {
            let metrics = handle.render();
            Ok(metrics.into_response())
        } else {
            error!("Metrics system not initialized");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Application metrics summary endpoint
pub async fn metrics_summary(State(state): State<AppState>) -> Result<axum::Json<MetricsSummary>, StatusCode> {
    let summary = collect_metrics_summary(&state).await;
    Ok(axum::Json(summary))
}

/// Performance monitoring summary endpoint
pub async fn performance_summary() -> Result<axum::Json<PerformanceSummary>, StatusCode> {
    let monitor = get_performance_monitor();
    let summary = monitor.get_performance_summary().await;
    Ok(axum::Json(summary))
}

/// Performance optimization endpoint for caching frequently accessed data
pub async fn optimize_performance() -> Result<axum::Json<serde_json::Value>, StatusCode> {
    let monitor = get_performance_monitor();
    
    // Trigger optimization tasks
    tokio::spawn(async move {
        // Clear old cache entries
        // In a real implementation, you might want to be more selective
        info!("Performance optimization triggered");
    });
    
    Ok(axum::Json(serde_json::json!({
        "status": "optimization_triggered",
        "timestamp": chrono::Utc::now()
    })))
}

/// Metrics summary structure
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
    pub http: HttpMetrics,
    pub websocket: WebSocketMetrics,
    pub database: DatabaseMetrics,
    pub messages: MessageMetrics,
    pub system: SystemMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpMetrics {
    pub requests_total: u64,
    pub errors_total: u64,
    pub avg_response_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMetrics {
    pub active_connections: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub active_connections: u32,
    pub queries_total: u64,
    pub avg_query_time_ms: f64,
    pub errors_total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageMetrics {
    pub created_total: u64,
    pub deduplicated_total: u64,
    pub avg_processing_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub disk_usage_percent: f64,
}

/// Collect metrics summary from various sources
async fn collect_metrics_summary(state: &AppState) -> MetricsSummary {
    // Get database stats
    let db_stats = state.db.health_check().await.unwrap_or_else(|_| crate::database::DatabaseStats {
        connection_count: 0,
        total_queries: 0,
        avg_query_time_ms: 0,
    });
    
    // In a real implementation, you'd collect these from actual metrics stores
    // For now, we'll use placeholder values
    MetricsSummary {
        timestamp: chrono::Utc::now(),
        uptime_seconds: crate::health::get_uptime_seconds(),
        http: HttpMetrics {
            requests_total: 0, // Would be collected from metrics store
            errors_total: 0,
            avg_response_time_ms: 0.0,
        },
        websocket: WebSocketMetrics {
            active_connections: 0, // Would get from connection manager
            messages_sent: 0,
            messages_received: 0,
        },
        database: DatabaseMetrics {
            active_connections: db_stats.connection_count,
            queries_total: db_stats.total_queries,
            avg_query_time_ms: db_stats.avg_query_time_ms as f64,
            errors_total: 0,
        },
        messages: MessageMetrics {
            created_total: 0,
            deduplicated_total: 0,
            avg_processing_time_ms: 0.0,
        },
        system: SystemMetrics {
            memory_usage_mb: 512, // Placeholder - would use system info crate
            cpu_usage_percent: 25.0,
            disk_usage_percent: 10.0,
        },
    }
}

/// Middleware to record HTTP request metrics
pub async fn record_http_request<B>(
    req: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let endpoint = format!("{} {}", method, path);
    
    // Increment request counter
    counter!("http_requests_total", 1, "method" => method.to_string(), "path" => path.clone());
    
    let response = next.run(req).await;
    
    // Record request duration
    let duration = start.elapsed();
    let is_error = response.status().is_client_error() || response.status().is_server_error();
    
    // Record in Prometheus
    histogram!("http_request_duration_seconds", duration.as_secs_f64(), "method" => method.to_string(), "path" => path.clone());
    
    // Record in performance monitor
    let monitor = get_performance_monitor();
    monitor.record_http_request(&endpoint, duration, is_error);
    
    // Record errors
    if is_error {
        counter!("http_requests_errors_total", 1, 
            "method" => method.to_string(), 
            "path" => path, 
            "status" => response.status().as_u16().to_string()
        );
    }
    
    response
}

/// Record database query metrics
pub fn record_database_query(operation: &str, duration: std::time::Duration, success: bool) {
    // Record in Prometheus
    counter!("database_queries_total", 1, "operation" => operation.to_string());
    histogram!("database_query_duration_seconds", duration.as_secs_f64(), "operation" => operation.to_string());
    
    // Record in performance monitor
    let monitor = get_performance_monitor();
    monitor.record_database_query(operation, duration, !success);
    
    if !success {
        counter!("database_errors_total", 1, "operation" => operation.to_string());
    }
}

/// Record WebSocket connection metrics
pub fn record_websocket_connection(connected: bool) {
    if connected {
        gauge!("websocket_connections_active", 1.0);
    } else {
        gauge!("websocket_connections_active", -1.0);
    }
}

/// Record WebSocket message metrics
pub fn record_websocket_message(direction: &str) {
    match direction {
        "sent" => counter!("websocket_messages_sent_total", 1),
        "received" => counter!("websocket_messages_received_total", 1),
        _ => {}
    }
}

/// Record message processing metrics
pub fn record_message_processing(duration: std::time::Duration, deduplicated: bool) {
    counter!("messages_created_total", 1);
    histogram!("message_processing_duration_seconds", duration.as_secs_f64());
    
    if deduplicated {
        counter!("messages_deduplicated_total", 1);
    }
}

/// Update system metrics
pub fn update_system_metrics() {
    // In production, you'd use a proper system info crate
    // For now, we'll use placeholder values
    gauge!("memory_usage_bytes", 512.0 * 1024.0 * 1024.0); // 512 MB
    gauge!("cpu_usage_percent", 25.0);
}

/// Record push notification metrics
pub fn record_push_notification(success: bool) {
    if success {
        counter!("push_notifications_sent_total", 1);
    } else {
        counter!("push_notifications_failed_total", 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_summary_serialization() {
        let summary = MetricsSummary {
            timestamp: chrono::Utc::now(),
            uptime_seconds: 3600,
            http: HttpMetrics {
                requests_total: 1000,
                errors_total: 10,
                avg_response_time_ms: 50.0,
            },
            websocket: WebSocketMetrics {
                active_connections: 25,
                messages_sent: 5000,
                messages_received: 4800,
            },
            database: DatabaseMetrics {
                active_connections: 5,
                queries_total: 2000,
                avg_query_time_ms: 10.0,
                errors_total: 2,
            },
            messages: MessageMetrics {
                created_total: 1500,
                deduplicated_total: 50,
                avg_processing_time_ms: 5.0,
            },
            system: SystemMetrics {
                memory_usage_mb: 512,
                cpu_usage_percent: 25.0,
                disk_usage_percent: 10.0,
            },
        };
        
        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: MetricsSummary = serde_json::from_str(&json).unwrap();
        
        assert_eq!(summary.uptime_seconds, deserialized.uptime_seconds);
        assert_eq!(summary.http.requests_total, deserialized.http.requests_total);
    }
}