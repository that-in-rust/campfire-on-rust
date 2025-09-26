// Analytics Handlers for Simple Success Tracking
// Focuses on deployment success metrics with privacy-friendly approach

use axum::{
    extract::{State, Query},
    response::{IntoResponse, Json},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    AppState,
    analytics::{EventType, extract_user_agent, extract_ip_address},
};

/// Track deployment button clicks from README
#[derive(Debug, Deserialize)]
pub struct TrackDeployClickRequest {
    pub source: Option<String>, // "readme", "demo", etc.
    pub deployment_type: Option<String>, // "railway", "self-hosted", etc.
}

pub async fn track_deploy_click(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<TrackDeployClickRequest>,
) -> impl IntoResponse {
    let mut properties = HashMap::new();
    
    if let Some(source) = params.source {
        properties.insert("source".to_string(), source);
    }
    
    if let Some(deployment_type) = params.deployment_type {
        properties.insert("deployment_type".to_string(), deployment_type);
    }
    
    let user_agent = extract_user_agent(&headers);
    let ip_address = extract_ip_address(&headers, None);
    
    state.analytics_store.track_event(
        EventType::DeployButtonClick,
        properties,
        user_agent,
        ip_address,
    ).await;
    
    // Return 1x1 transparent pixel for tracking
    let pixel_data = base64::decode("R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7")
        .unwrap_or_else(|_| vec![0]);
    
    (
        StatusCode::OK,
        [("Content-Type", "image/gif"), ("Cache-Control", "no-cache")],
        pixel_data,
    )
}

/// Track install script downloads
pub async fn track_install_download(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user_agent = extract_user_agent(&headers);
    let ip_address = extract_ip_address(&headers, None);
    
    state.analytics_store.track_event(
        EventType::InstallScriptDownload,
        HashMap::new(),
        user_agent,
        ip_address,
    ).await;
    
    // Return success response
    Json(serde_json::json!({
        "status": "tracked",
        "event": "install_script_download"
    }))
}

/// Track install script execution results
#[derive(Debug, Deserialize)]
pub struct TrackInstallResultRequest {
    pub success: bool,
    pub error_message: Option<String>,
    pub platform: Option<String>,
    pub version: Option<String>,
}

pub async fn track_install_result(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TrackInstallResultRequest>,
) -> impl IntoResponse {
    let mut properties = HashMap::new();
    
    if let Some(platform) = payload.platform {
        properties.insert("platform".to_string(), platform);
    }
    
    if let Some(version) = payload.version {
        properties.insert("version".to_string(), version);
    }
    
    if let Some(error) = payload.error_message {
        properties.insert("error_message".to_string(), error);
    }
    
    let event_type = if payload.success {
        EventType::InstallScriptSuccess
    } else {
        EventType::InstallScriptFailure
    };
    
    let user_agent = extract_user_agent(&headers);
    let ip_address = extract_ip_address(&headers, None);
    
    state.analytics_store.track_event(
        event_type,
        properties,
        user_agent,
        ip_address,
    ).await;
    
    Json(serde_json::json!({
        "status": "tracked",
        "event": if payload.success { "install_success" } else { "install_failure" }
    }))
}

/// Track local application startup
#[derive(Debug, Deserialize)]
pub struct TrackStartupRequest {
    pub success: bool,
    pub startup_time_ms: Option<u64>,
    pub demo_mode: Option<bool>,
    pub error_message: Option<String>,
}

pub async fn track_startup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TrackStartupRequest>,
) -> impl IntoResponse {
    let mut properties = HashMap::new();
    
    if let Some(startup_time) = payload.startup_time_ms {
        properties.insert("startup_time_ms".to_string(), startup_time.to_string());
    }
    
    if let Some(demo_mode) = payload.demo_mode {
        properties.insert("demo_mode".to_string(), demo_mode.to_string());
    }
    
    if let Some(error) = payload.error_message {
        properties.insert("error_message".to_string(), error);
    }
    
    let event_type = if payload.success {
        EventType::LocalStartupSuccess
    } else {
        EventType::LocalStartupFailure
    };
    
    let user_agent = extract_user_agent(&headers);
    let ip_address = extract_ip_address(&headers, None);
    
    state.analytics_store.track_event(
        event_type,
        properties,
        user_agent,
        ip_address,
    ).await;
    
    Json(serde_json::json!({
        "status": "tracked",
        "event": if payload.success { "startup_success" } else { "startup_failure" }
    }))
}

/// Track demo mode access
pub async fn track_demo_access(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user_agent = extract_user_agent(&headers);
    let ip_address = extract_ip_address(&headers, None);
    
    state.analytics_store.track_event(
        EventType::DemoModeAccessed,
        HashMap::new(),
        user_agent,
        ip_address,
    ).await;
    
    Json(serde_json::json!({
        "status": "tracked",
        "event": "demo_access"
    }))
}

/// Track Railway deployment events (if we can detect them)
#[derive(Debug, Deserialize)]
pub struct TrackRailwayDeploymentRequest {
    pub success: bool,
    pub deployment_id: Option<String>,
    pub error_message: Option<String>,
    pub deploy_time_seconds: Option<u64>,
}

pub async fn track_railway_deployment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TrackRailwayDeploymentRequest>,
) -> impl IntoResponse {
    let mut properties = HashMap::new();
    
    if let Some(deployment_id) = payload.deployment_id {
        properties.insert("deployment_id".to_string(), deployment_id);
    }
    
    if let Some(deploy_time) = payload.deploy_time_seconds {
        properties.insert("deploy_time_seconds".to_string(), deploy_time.to_string());
    }
    
    if let Some(error) = payload.error_message {
        properties.insert("error_message".to_string(), error);
    }
    
    let event_type = if payload.success {
        EventType::RailwayDeploymentSuccess
    } else {
        EventType::RailwayDeploymentFailure
    };
    
    let user_agent = extract_user_agent(&headers);
    let ip_address = extract_ip_address(&headers, None);
    
    state.analytics_store.track_event(
        event_type,
        properties,
        user_agent,
        ip_address,
    ).await;
    
    Json(serde_json::json!({
        "status": "tracked",
        "event": if payload.success { "railway_deploy_success" } else { "railway_deploy_failure" }
    }))
}

/// Get deployment metrics (for internal monitoring)
#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub metrics: crate::analytics::DeploymentMetrics,
    pub success_rates: SuccessRates,
}

#[derive(Debug, Serialize)]
pub struct SuccessRates {
    pub install_success_rate: f64,
    pub local_startup_success_rate: f64,
    pub railway_deploy_success_rate: f64,
}

pub async fn get_deployment_metrics(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let metrics = state.analytics_store.get_deployment_metrics().await;
    
    let success_rates = SuccessRates {
        install_success_rate: metrics.install_success_rate(),
        local_startup_success_rate: metrics.local_startup_success_rate(),
        railway_deploy_success_rate: metrics.railway_deploy_success_rate(),
    };
    
    Json(MetricsResponse {
        metrics,
        success_rates,
    })
}

/// Health check for analytics system
pub async fn analytics_health(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let recent_events = state.analytics_store.get_recent_events(5).await;
    
    Json(serde_json::json!({
        "status": "healthy",
        "recent_events_count": recent_events.len(),
        "last_event_time": recent_events.first().map(|e| e.timestamp)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::AnalyticsStore;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_analytics_store_integration() {
        let analytics_store = Arc::new(AnalyticsStore::new(100));
        
        // Test tracking an event
        analytics_store.track_event(
            EventType::DeployButtonClick,
            HashMap::new(),
            None,
            None,
        ).await;
        
        let metrics = analytics_store.get_deployment_metrics().await;
        assert_eq!(metrics.deploy_button_clicks, 1);
    }
}