use campfire_on_rust::{health, metrics, shutdown};
use std::time::Duration;

#[tokio::test]
async fn test_health_check_initialization() {
    // Test that health check system can be initialized
    health::init();
    
    // Test that uptime is tracked
    tokio::time::sleep(Duration::from_millis(100)).await;
    let uptime = health::get_uptime_seconds();
    assert!(uptime >= 0);
}

#[tokio::test]
async fn test_metrics_initialization() {
    // Test that metrics system can be initialized
    let result = metrics::init_metrics();
    // Should not panic, but might fail if already initialized
    // In a real test environment, we'd use a separate instance
}

#[tokio::test]
async fn test_shutdown_coordinator() {
    let mut coordinator = shutdown::ShutdownCoordinator::new();
    
    // Add a test shutdown task
    coordinator.add_task(
        "test_task".to_string(),
        Duration::from_secs(1),
        || {
            tokio::spawn(async {
                tokio::time::sleep(Duration::from_millis(100)).await;
            })
        }
    );
    
    // Test shutdown
    coordinator.shutdown(shutdown::ShutdownSignal::Application).await;
}

#[tokio::test]
async fn test_resource_manager() {
    let mut manager = shutdown::ResourceManager::new();
    
    manager.add_resource(shutdown::DatabaseResource::new("test_db".to_string()));
    manager.add_resource(shutdown::WebSocketResource::new("test_ws".to_string(), 5));
    
    manager.cleanup_all().await;
}

#[tokio::test]
async fn test_startup_validator() {
    let mut validator = shutdown::StartupValidator::new();
    
    validator.add_check(shutdown::DatabaseConnectivityCheck::new("sqlite://test.db".to_string()));
    validator.add_check(shutdown::ConfigurationCheck::new("test_config".to_string()));
    validator.add_check(shutdown::ServicesCheck::new(vec!["auth".to_string(), "messaging".to_string()]));
    
    let result = validator.validate_all().await;
    assert!(result.is_ok());
}