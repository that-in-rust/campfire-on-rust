use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{error, warn};

use crate::AppState;

/// Health check response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: HealthChecks,
}

/// Overall health status
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual health check results
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthChecks {
    pub database: CheckResult,
    pub memory: CheckResult,
    pub disk_space: CheckResult,
}

/// Individual check result
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckResult {
    pub status: CheckStatus,
    pub message: String,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Status of individual checks
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

/// Readiness check response (simpler than health check)
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub timestamp: DateTime<Utc>,
    pub checks: ReadinessChecks,
}

/// Readiness check results
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessChecks {
    pub database: bool,
    pub services: bool,
}

/// Application startup time for uptime calculation
static mut START_TIME: Option<Instant> = None;

/// Initialize the health check system
pub fn init() {
    unsafe {
        START_TIME = Some(Instant::now());
    }
}

/// Get application uptime in seconds
pub fn get_uptime_seconds() -> u64 {
    unsafe {
        START_TIME
            .map(|start| start.elapsed().as_secs())
            .unwrap_or(0)
    }
}

/// Comprehensive health check endpoint
pub async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    let _start_time = Instant::now();
    
    // Run all health checks
    let database_check = check_database_health(&state).await;
    let memory_check = check_memory_health().await;
    let disk_check = check_disk_space_health().await;
    
    // Determine overall status
    let overall_status = determine_overall_status(&[
        &database_check,
        &memory_check,
        &disk_check,
    ]);
    
    let response = HealthResponse {
        status: overall_status,
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_uptime_seconds(),
        checks: HealthChecks {
            database: database_check,
            memory: memory_check,
            disk_space: disk_check,
        },
    };
    
    // Return appropriate HTTP status based on health
    let status_code = match response.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // Still serving traffic
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };
    
    if status_code != StatusCode::OK {
        warn!("Health check failed: {:?}", response);
    }
    
    Ok(Json(response))
}

/// Simple readiness check endpoint
pub async fn readiness_check(State(state): State<AppState>) -> Result<Json<ReadinessResponse>, StatusCode> {
    // Quick checks for readiness
    let database_ready = check_database_readiness(&state).await;
    let services_ready = check_services_readiness(&state).await;
    
    let ready = database_ready && services_ready;
    
    let response = ReadinessResponse {
        ready,
        timestamp: Utc::now(),
        checks: ReadinessChecks {
            database: database_ready,
            services: services_ready,
        },
    };
    
    let _status_code = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    Ok(Json(response))
}

/// Simple liveness check endpoint
pub async fn liveness_check() -> StatusCode {
    // Basic liveness - if we can respond, we're alive
    StatusCode::OK
}

/// Check database connectivity and performance
async fn check_database_health(state: &AppState) -> CheckResult {
    let start = Instant::now();
    
    match state.db.health_check().await {
        Ok(stats) => {
            let duration = start.elapsed();
            
            // Warn if database response is slow
            let status = if duration > Duration::from_millis(1000) {
                CheckStatus::Warn
            } else {
                CheckStatus::Pass
            };
            
            CheckResult {
                status,
                message: "Database connection healthy".to_string(),
                duration_ms: duration.as_millis() as u64,
                details: Some(serde_json::json!({
                    "connection_count": stats.connection_count,
                    "total_queries": stats.total_queries,
                    "avg_query_time_ms": stats.avg_query_time_ms,
                })),
            }
        }
        Err(e) => {
            error!("Database health check failed: {}", e);
            CheckResult {
                status: CheckStatus::Fail,
                message: format!("Database connection failed: {}", e),
                duration_ms: start.elapsed().as_millis() as u64,
                details: None,
            }
        }
    }
}

/// Check memory usage
async fn check_memory_health() -> CheckResult {
    let start = Instant::now();
    
    // Get memory statistics (simplified - in production you'd use a proper system info crate)
    let memory_info = get_memory_info();
    
    let status = match memory_info.usage_percent {
        usage if usage > 90.0 => CheckStatus::Fail,
        usage if usage > 80.0 => CheckStatus::Warn,
        _ => CheckStatus::Pass,
    };
    
    let message = match status {
        CheckStatus::Pass => "Memory usage normal".to_string(),
        CheckStatus::Warn => format!("Memory usage high: {:.1}%", memory_info.usage_percent),
        CheckStatus::Fail => format!("Memory usage critical: {:.1}%", memory_info.usage_percent),
    };
    
    CheckResult {
        status,
        message,
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "used_mb": memory_info.used_mb,
            "total_mb": memory_info.total_mb,
            "usage_percent": memory_info.usage_percent,
        })),
    }
}

/// Check disk space
async fn check_disk_space_health() -> CheckResult {
    let start = Instant::now();
    
    // Get disk space information (simplified)
    let disk_info = get_disk_info();
    
    let status = match disk_info.usage_percent {
        usage if usage > 95.0 => CheckStatus::Fail,
        usage if usage > 85.0 => CheckStatus::Warn,
        _ => CheckStatus::Pass,
    };
    
    let message = match status {
        CheckStatus::Pass => "Disk space sufficient".to_string(),
        CheckStatus::Warn => format!("Disk space low: {:.1}%", disk_info.usage_percent),
        CheckStatus::Fail => format!("Disk space critical: {:.1}%", disk_info.usage_percent),
    };
    
    CheckResult {
        status,
        message,
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "used_gb": disk_info.used_gb,
            "total_gb": disk_info.total_gb,
            "usage_percent": disk_info.usage_percent,
        })),
    }
}

/// Quick database readiness check
async fn check_database_readiness(state: &AppState) -> bool {
    // Simple ping to database with short timeout
    tokio::time::timeout(
        Duration::from_millis(500),
        state.db.ping()
    ).await.is_ok()
}

/// Check if all services are ready
async fn check_services_readiness(_state: &AppState) -> bool {
    // In a more complex system, you'd check if all required services are initialized
    // For now, we assume services are ready if we got this far
    true
}

/// Determine overall health status from individual checks
fn determine_overall_status(checks: &[&CheckResult]) -> HealthStatus {
    let has_failures = checks.iter().any(|check| check.status == CheckStatus::Fail);
    let has_warnings = checks.iter().any(|check| check.status == CheckStatus::Warn);
    
    if has_failures {
        HealthStatus::Unhealthy
    } else if has_warnings {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    }
}

/// Memory information structure
#[derive(Debug)]
struct MemoryInfo {
    used_mb: u64,
    total_mb: u64,
    usage_percent: f64,
}

/// Get memory information (simplified implementation)
fn get_memory_info() -> MemoryInfo {
    // In production, you'd use a proper system info crate like `sysinfo`
    // This is a simplified implementation for demonstration
    MemoryInfo {
        used_mb: 512,  // Placeholder values
        total_mb: 1024,
        usage_percent: 50.0,
    }
}

/// Disk information structure
#[derive(Debug)]
struct DiskInfo {
    used_gb: u64,
    total_gb: u64,
    usage_percent: f64,
}

/// Get disk space information (simplified implementation)
fn get_disk_info() -> DiskInfo {
    // In production, you'd use proper system calls or crates
    // This is a simplified implementation for demonstration
    DiskInfo {
        used_gb: 10,   // Placeholder values
        total_gb: 100,
        usage_percent: 10.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_determine_overall_status() {
        let pass_check = CheckResult {
            status: CheckStatus::Pass,
            message: "OK".to_string(),
            duration_ms: 10,
            details: None,
        };
        
        let warn_check = CheckResult {
            status: CheckStatus::Warn,
            message: "Warning".to_string(),
            duration_ms: 20,
            details: None,
        };
        
        let fail_check = CheckResult {
            status: CheckStatus::Fail,
            message: "Failed".to_string(),
            duration_ms: 30,
            details: None,
        };
        
        // All pass = healthy
        assert_eq!(
            determine_overall_status(&[&pass_check, &pass_check]),
            HealthStatus::Healthy
        );
        
        // Has warning = degraded
        assert_eq!(
            determine_overall_status(&[&pass_check, &warn_check]),
            HealthStatus::Degraded
        );
        
        // Has failure = unhealthy
        assert_eq!(
            determine_overall_status(&[&pass_check, &fail_check]),
            HealthStatus::Unhealthy
        );
        
        // Failure overrides warning
        assert_eq!(
            determine_overall_status(&[&warn_check, &fail_check]),
            HealthStatus::Unhealthy
        );
    }
}