use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use metrics::{counter, gauge, histogram, describe_counter, describe_gauge, describe_histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{error, info};

use crate::AppState;

/// Metrics recorder handle for Prometheus export
static mut PROMETHEUS_HANDLE: Option<PrometheusHandle> = None;

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
    
    info!("Metrics system initialized");
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
    
    // Increment request counter
    counter!("http_requests_total", 1, "method" => method.to_string(), "path" => path.clone());
    
    let response = next.run(req).await;
    
    // Record request duration
    let duration = start.elapsed();
    histogram!("http_request_duration_seconds", duration.as_secs_f64(), "method" => method.to_string(), "path" => path.clone());
    
    // Record errors
    if response.status().is_client_error() || response.status().is_server_error() {
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
    counter!("database_queries_total", 1, "operation" => operation.to_string());
    histogram!("database_query_duration_seconds", duration.as_secs_f64(), "operation" => operation.to_string());
    
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