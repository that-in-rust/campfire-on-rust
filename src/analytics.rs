// Simple Analytics Module for Campfire GTM Success Tracking
// Focuses on deployment success metrics, not complex user behavior

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

/// Simple event tracking for GTM success metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_id: Uuid,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub properties: HashMap<String, String>,
    pub user_agent: Option<String>,
    pub ip_hash: Option<String>, // Hashed for privacy
}

/// Event types focused on deployment success
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // README interactions
    DeployButtonClick,
    InstallScriptDownload,
    
    // Local installation tracking
    InstallScriptStart,
    InstallScriptSuccess,
    InstallScriptFailure,
    
    // Application startup tracking
    LocalStartupSuccess,
    LocalStartupFailure,
    
    // Demo mode interactions
    DemoModeAccessed,
    DemoDeployButtonClick,
    
    // Railway deployment tracking (if we can detect it)
    RailwayDeploymentStart,
    RailwayDeploymentSuccess,
    RailwayDeploymentFailure,
}

/// Simple in-memory analytics store (privacy-friendly)
#[derive(Debug)]
pub struct AnalyticsStore {
    events: RwLock<Vec<AnalyticsEvent>>,
    max_events: usize,
}

impl AnalyticsStore {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            max_events,
        }
    }
    
    /// Track an event with privacy-friendly approach
    pub async fn track_event(
        &self,
        event_type: EventType,
        properties: HashMap<String, String>,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) {
        let event = AnalyticsEvent {
            event_id: Uuid::new_v4(),
            event_type: event_type.clone(),
            timestamp: Utc::now(),
            properties,
            user_agent,
            ip_hash: ip_address.map(|ip| hash_ip(&ip)),
        };
        
        // Log the event for monitoring
        info!(
            event_type = ?event_type,
            event_id = %event.event_id,
            "Analytics event tracked"
        );
        
        let mut events = self.events.write().await;
        events.push(event);
        
        // Keep only recent events for privacy
        if events.len() > self.max_events {
            let excess = events.len() - self.max_events;
            events.drain(0..excess);
        }
    }
    
    /// Get deployment success metrics
    pub async fn get_deployment_metrics(&self) -> DeploymentMetrics {
        let events = self.events.read().await;
        let mut metrics = DeploymentMetrics::default();
        
        for event in events.iter() {
            match &event.event_type {
                EventType::DeployButtonClick => metrics.deploy_button_clicks += 1,
                EventType::InstallScriptDownload => metrics.install_script_downloads += 1,
                EventType::InstallScriptSuccess => metrics.install_successes += 1,
                EventType::InstallScriptFailure => metrics.install_failures += 1,
                EventType::LocalStartupSuccess => metrics.local_startup_successes += 1,
                EventType::LocalStartupFailure => metrics.local_startup_failures += 1,
                EventType::DemoModeAccessed => metrics.demo_mode_accesses += 1,
                EventType::DemoDeployButtonClick => metrics.demo_deploy_clicks += 1,
                EventType::RailwayDeploymentSuccess => metrics.railway_deploy_successes += 1,
                EventType::RailwayDeploymentFailure => metrics.railway_deploy_failures += 1,
                _ => {}
            }
        }
        
        metrics
    }
    
    /// Get recent events for debugging (limited for privacy)
    pub async fn get_recent_events(&self, limit: usize) -> Vec<AnalyticsEvent> {
        let events = self.events.read().await;
        events.iter()
            .rev()
            .take(limit.min(50)) // Max 50 events for privacy
            .cloned()
            .collect()
    }
}

/// Deployment success metrics
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeploymentMetrics {
    pub deploy_button_clicks: u64,
    pub install_script_downloads: u64,
    pub install_successes: u64,
    pub install_failures: u64,
    pub local_startup_successes: u64,
    pub local_startup_failures: u64,
    pub demo_mode_accesses: u64,
    pub demo_deploy_clicks: u64,
    pub railway_deploy_successes: u64,
    pub railway_deploy_failures: u64,
}

impl DeploymentMetrics {
    /// Calculate success rates
    pub fn install_success_rate(&self) -> f64 {
        if self.install_successes + self.install_failures == 0 {
            return 0.0;
        }
        self.install_successes as f64 / (self.install_successes + self.install_failures) as f64
    }
    
    pub fn local_startup_success_rate(&self) -> f64 {
        if self.local_startup_successes + self.local_startup_failures == 0 {
            return 0.0;
        }
        self.local_startup_successes as f64 / (self.local_startup_successes + self.local_startup_failures) as f64
    }
    
    pub fn railway_deploy_success_rate(&self) -> f64 {
        if self.railway_deploy_successes + self.railway_deploy_failures == 0 {
            return 0.0;
        }
        self.railway_deploy_successes as f64 / (self.railway_deploy_successes + self.railway_deploy_failures) as f64
    }
}

/// Hash IP address for privacy (one-way hash)
fn hash_ip(ip: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    ip.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Helper function to extract user agent from headers
pub fn extract_user_agent(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

/// Helper function to extract IP address (with privacy considerations)
pub fn extract_ip_address(headers: &axum::http::HeaderMap, remote_addr: Option<std::net::SocketAddr>) -> Option<String> {
    // Check for forwarded headers first (for reverse proxies)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }
    }
    
    // Check real IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }
    
    // Fall back to remote address
    remote_addr.map(|addr| addr.ip().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_analytics_store() {
        let store = AnalyticsStore::new(100);
        
        // Track some events
        store.track_event(
            EventType::DeployButtonClick,
            HashMap::new(),
            Some("test-agent".to_string()),
            Some("127.0.0.1".to_string()),
        ).await;
        
        store.track_event(
            EventType::InstallScriptSuccess,
            HashMap::new(),
            None,
            None,
        ).await;
        
        // Check metrics
        let metrics = store.get_deployment_metrics().await;
        assert_eq!(metrics.deploy_button_clicks, 1);
        assert_eq!(metrics.install_successes, 1);
        
        // Check recent events
        let events = store.get_recent_events(10).await;
        assert_eq!(events.len(), 2);
    }
    
    #[test]
    fn test_deployment_metrics() {
        let mut metrics = DeploymentMetrics::default();
        metrics.install_successes = 8;
        metrics.install_failures = 2;
        
        assert_eq!(metrics.install_success_rate(), 0.8);
    }
    
    #[test]
    fn test_ip_hashing() {
        let ip1 = "192.168.1.1";
        let ip2 = "192.168.1.2";
        
        let hash1 = hash_ip(ip1);
        let hash2 = hash_ip(ip2);
        
        // Hashes should be different
        assert_ne!(hash1, hash2);
        
        // Same IP should produce same hash
        assert_eq!(hash_ip(ip1), hash1);
    }
}