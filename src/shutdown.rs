use signal_hook::consts::SIGTERM;
use signal_hook_tokio::Signals;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Notify};
use tokio::time::timeout;
use tracing::{error, info, warn};
use futures_util::stream::StreamExt;

/// Shutdown coordinator that manages graceful shutdown of all components
pub struct ShutdownCoordinator {
    /// Broadcast sender for shutdown signals
    shutdown_sender: broadcast::Sender<ShutdownSignal>,
    /// Notification for when shutdown is complete
    shutdown_complete: Arc<Notify>,
    /// List of shutdown tasks to wait for
    shutdown_tasks: Vec<ShutdownTask>,
}

/// Types of shutdown signals
#[derive(Debug, Clone, Copy)]
pub enum ShutdownSignal {
    /// Graceful shutdown requested (SIGTERM)
    Graceful,
    /// Immediate shutdown requested (SIGINT/Ctrl+C)
    Immediate,
    /// Application-initiated shutdown
    Application,
}

/// A task that needs to be shut down gracefully
pub struct ShutdownTask {
    pub name: String,
    pub timeout: Duration,
    pub shutdown_fn: Box<dyn Fn() -> tokio::task::JoinHandle<()> + Send + Sync>,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new() -> Self {
        let (shutdown_sender, _) = broadcast::channel(16);
        let shutdown_complete = Arc::new(Notify::new());
        
        Self {
            shutdown_sender,
            shutdown_complete,
            shutdown_tasks: Vec::new(),
        }
    }
    
    /// Get a receiver for shutdown signals
    pub fn subscribe(&self) -> broadcast::Receiver<ShutdownSignal> {
        self.shutdown_sender.subscribe()
    }
    
    /// Add a shutdown task
    pub fn add_task<F>(&mut self, name: String, timeout: Duration, shutdown_fn: F)
    where
        F: Fn() -> tokio::task::JoinHandle<()> + Send + Sync + 'static,
    {
        self.shutdown_tasks.push(ShutdownTask {
            name,
            timeout,
            shutdown_fn: Box::new(shutdown_fn),
        });
    }
    
    /// Start listening for shutdown signals
    pub async fn listen_for_signals(&self) {
        let shutdown_sender = self.shutdown_sender.clone();
        let shutdown_complete = self.shutdown_complete.clone();
        
        tokio::spawn(async move {
            // Set up signal handlers
            let mut signals = match Signals::new(&[SIGTERM, signal_hook::consts::SIGINT]) {
                Ok(signals) => signals,
                Err(e) => {
                    error!("Failed to set up signal handlers: {}", e);
                    return;
                }
            };
            
            // Wait for shutdown signal
            while let Some(signal) = signals.next().await {
                let shutdown_type = match signal {
                    signal_hook::consts::SIGTERM => {
                        info!("Received SIGTERM, initiating graceful shutdown...");
                        ShutdownSignal::Graceful
                    }
                    signal_hook::consts::SIGINT => {
                        info!("Received SIGINT (Ctrl+C), initiating immediate shutdown...");
                        ShutdownSignal::Immediate
                    }
                    _ => {
                        warn!("Received unknown signal: {}", signal);
                        continue;
                    }
                };
                
                // Send shutdown signal
                if let Err(e) = shutdown_sender.send(shutdown_type) {
                    error!("Failed to send shutdown signal: {}", e);
                }
                
                // For immediate shutdown, don't wait for graceful cleanup
                if matches!(shutdown_type, ShutdownSignal::Immediate) {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    std::process::exit(1);
                }
                
                break;
            }
            
            // Notify that signal handling is complete
            shutdown_complete.notify_waiters();
        });
    }
    
    /// Initiate application shutdown
    pub async fn shutdown(&self, signal: ShutdownSignal) {
        info!("Initiating shutdown: {:?}", signal);
        
        // Send shutdown signal to all subscribers
        if let Err(e) = self.shutdown_sender.send(signal) {
            error!("Failed to send shutdown signal: {}", e);
        }
        
        // Execute shutdown tasks
        self.execute_shutdown_tasks(signal).await;
        
        info!("Shutdown complete");
    }
    
    /// Execute all shutdown tasks
    async fn execute_shutdown_tasks(&self, signal: ShutdownSignal) {
        let timeout_duration = match signal {
            ShutdownSignal::Graceful => Duration::from_secs(30),
            ShutdownSignal::Immediate => Duration::from_secs(5),
            ShutdownSignal::Application => Duration::from_secs(15),
        };
        
        info!("Executing {} shutdown tasks with timeout {:?}", 
              self.shutdown_tasks.len(), timeout_duration);
        
        let mut handles = Vec::new();
        
        // Start all shutdown tasks
        for task in &self.shutdown_tasks {
            info!("Starting shutdown task: {}", task.name);
            let handle = (task.shutdown_fn)();
            handles.push((task.name.clone(), task.timeout, handle));
        }
        
        // Wait for all tasks to complete or timeout
        for (name, task_timeout, handle) in handles {
            let effective_timeout = std::cmp::min(task_timeout, timeout_duration);
            
            match timeout(effective_timeout, handle).await {
                Ok(Ok(())) => {
                    info!("Shutdown task '{}' completed successfully", name);
                }
                Ok(Err(e)) => {
                    error!("Shutdown task '{}' failed: {}", name, e);
                }
                Err(_) => {
                    warn!("Shutdown task '{}' timed out after {:?}", name, effective_timeout);
                }
            }
        }
    }
    
    /// Wait for shutdown to complete
    pub async fn wait_for_shutdown(&self) {
        self.shutdown_complete.notified().await;
    }
}

/// Resource cleanup manager
pub struct ResourceManager {
    resources: Vec<Box<dyn Resource + Send + Sync>>,
}

/// Trait for resources that need cleanup
#[async_trait::async_trait]
pub trait Resource {
    fn name(&self) -> &str;
    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }
    
    pub fn add_resource<R: Resource + Send + Sync + 'static>(&mut self, resource: R) {
        self.resources.push(Box::new(resource));
    }
    
    pub async fn cleanup_all(&self) {
        info!("Cleaning up {} resources", self.resources.len());
        
        for resource in &self.resources {
            match resource.cleanup().await {
                Ok(()) => {
                    info!("Successfully cleaned up resource: {}", resource.name());
                }
                Err(e) => {
                    error!("Failed to cleanup resource '{}': {}", resource.name(), e);
                }
            }
        }
        
        info!("Resource cleanup complete");
    }
}

/// Database connection resource
pub struct DatabaseResource {
    name: String,
    // In a real implementation, you'd hold a reference to the database connection
}

impl DatabaseResource {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait::async_trait]
impl Resource for DatabaseResource {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Closing database connections for: {}", self.name);
        // In a real implementation, you'd close database connections here
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }
}

/// WebSocket connection resource
pub struct WebSocketResource {
    name: String,
    connection_count: usize,
}

impl WebSocketResource {
    pub fn new(name: String, connection_count: usize) -> Self {
        Self { name, connection_count }
    }
}

#[async_trait::async_trait]
impl Resource for WebSocketResource {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Closing {} WebSocket connections for: {}", self.connection_count, self.name);
        // In a real implementation, you'd close WebSocket connections here
        tokio::time::sleep(Duration::from_millis(50 * self.connection_count as u64)).await;
        Ok(())
    }
}

/// Background task resource
#[allow(dead_code)]
pub struct BackgroundTaskResource {
    name: String,
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl BackgroundTaskResource {
    pub fn new(name: String, task_handle: tokio::task::JoinHandle<()>) -> Self {
        Self { 
            name, 
            task_handle: Some(task_handle),
        }
    }
}

#[async_trait::async_trait]
impl Resource for BackgroundTaskResource {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stopping background task: {}", self.name);
        
        // In a real implementation, you'd signal the task to stop and wait for it
        // For now, we'll just simulate cleanup
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        Ok(())
    }
}

/// Startup validation checks
pub struct StartupValidator {
    checks: Vec<Box<dyn StartupCheck + Send + Sync>>,
}

/// Trait for startup validation checks
#[async_trait::async_trait]
pub trait StartupCheck {
    fn name(&self) -> &str;
    async fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

impl StartupValidator {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }
    
    pub fn add_check<C: StartupCheck + Send + Sync + 'static>(&mut self, check: C) {
        self.checks.push(Box::new(check));
    }
    
    pub async fn validate_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Running {} startup validation checks", self.checks.len());
        
        for check in &self.checks {
            info!("Running startup check: {}", check.name());
            
            match check.validate().await {
                Ok(()) => {
                    info!("Startup check '{}' passed", check.name());
                }
                Err(e) => {
                    error!("Startup check '{}' failed: {}", check.name(), e);
                    return Err(e);
                }
            }
        }
        
        info!("All startup validation checks passed");
        Ok(())
    }
}

/// Database connectivity check
pub struct DatabaseConnectivityCheck {
    database_url: String,
}

impl DatabaseConnectivityCheck {
    pub fn new(database_url: String) -> Self {
        Self { database_url }
    }
}

#[async_trait::async_trait]
impl StartupCheck for DatabaseConnectivityCheck {
    fn name(&self) -> &str {
        "Database Connectivity"
    }
    
    async fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, you'd test database connectivity
        info!("Testing database connectivity to: {}", self.database_url);
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }
}

/// Configuration validation check
pub struct ConfigurationCheck {
    config_name: String,
}

impl ConfigurationCheck {
    pub fn new(config_name: String) -> Self {
        Self { config_name }
    }
}

#[async_trait::async_trait]
impl StartupCheck for ConfigurationCheck {
    fn name(&self) -> &str {
        "Configuration Validation"
    }
    
    async fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Validating configuration: {}", self.config_name);
        // In a real implementation, you'd validate configuration
        Ok(())
    }
}

/// Required services check
pub struct ServicesCheck {
    service_names: Vec<String>,
}

impl ServicesCheck {
    pub fn new(service_names: Vec<String>) -> Self {
        Self { service_names }
    }
}

#[async_trait::async_trait]
impl StartupCheck for ServicesCheck {
    fn name(&self) -> &str {
        "Required Services"
    }
    
    async fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for service in &self.service_names {
            info!("Checking service availability: {}", service);
            // In a real implementation, you'd check if services are available
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_shutdown_coordinator() {
        let mut coordinator = ShutdownCoordinator::new();
        
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
        coordinator.shutdown(ShutdownSignal::Application).await;
    }
    
    #[tokio::test]
    async fn test_resource_manager() {
        let mut manager = ResourceManager::new();
        
        manager.add_resource(DatabaseResource::new("test_db".to_string()));
        manager.add_resource(WebSocketResource::new("test_ws".to_string(), 5));
        
        manager.cleanup_all().await;
    }
    
    #[tokio::test]
    async fn test_startup_validator() {
        let mut validator = StartupValidator::new();
        
        validator.add_check(DatabaseConnectivityCheck::new("sqlite://test.db".to_string()));
        validator.add_check(ConfigurationCheck::new("test_config".to_string()));
        validator.add_check(ServicesCheck::new(vec!["auth".to_string(), "messaging".to_string()]));
        
        let result = validator.validate_all().await;
        assert!(result.is_ok());
    }
}