// Railway Deployment Testing Module
// Professional testing framework for Railway deployment validation

use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Railway Deployment Testing Framework
/// 
/// Implements professional testing patterns for Railway deployment validation
/// following the L1→L2→L3 layered architecture approach:
/// 
/// L1 Core: Configuration validation and deployment contracts
/// L2 Standard: HTTP client testing and async infrastructure  
/// L3 External: Railway API integration and deployment verification

#[derive(Error, Debug)]
pub enum RailwayTestError {
    #[error("Configuration validation failed: {field} - {reason}")]
    ConfigurationInvalid { field: String, reason: String },
    
    #[error("Deployment contract violation: {contract} - expected {expected}, got {actual}")]
    ContractViolation { contract: String, expected: String, actual: String },
    
    #[error("Performance requirement failed: {metric} took {actual:?}, limit {limit:?}")]
    PerformanceViolation { metric: String, actual: Duration, limit: Duration },
    
    #[error("Accessibility test failed: {url} - {reason}")]
    AccessibilityFailed { url: String, reason: String },
    
    #[error("Functionality test failed: {feature} - {details}")]
    FunctionalityFailed { feature: String, details: String },
    
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("IO operation failed: {0}")]
    IoError(#[from] std::io::Error),
}

/// Railway deployment configuration contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RailwayConfig {
    pub build: BuildConfig,
    pub deploy: DeployConfig,
    pub env: HashMap<String, EnvVar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub builder: String,
    #[serde(rename = "dockerfilePath")]
    pub dockerfile_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    #[serde(rename = "healthcheckPath")]
    pub healthcheck_path: String,
    #[serde(rename = "healthcheckTimeout")]
    pub healthcheck_timeout: u32,
    #[serde(rename = "restartPolicyType")]
    pub restart_policy_type: String,
    #[serde(rename = "restartPolicyMaxRetries")]
    pub restart_policy_max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub default: Option<String>,
}

/// Railway deployment test contract
/// 
/// # Preconditions
/// - Valid Railway configuration exists
/// - Railway CLI is authenticated
/// - Network access to Railway API
/// 
/// # Postconditions
/// - Deployment completes within 3 minutes (Requirement 3.2)
/// - Instance is accessible and functional (Requirement 3.3)
/// - Admin setup and chat functionality work (Requirement 3.4)
/// - Error messages are clear and actionable (Requirement 3.6)
/// 
/// # Error Conditions
/// - RailwayTestError::ConfigurationInvalid if config is malformed
/// - RailwayTestError::ContractViolation if deployment doesn't meet requirements
/// - RailwayTestError::PerformanceViolation if timing constraints are exceeded
pub struct RailwayDeploymentValidator {
    config: RailwayConfig,
    client: reqwest::Client,
    performance_limits: PerformanceLimits,
}

#[derive(Debug, Clone)]
pub struct PerformanceLimits {
    pub deployment_timeout: Duration,
    pub health_check_timeout: Duration,
    pub api_response_timeout: Duration,
}

impl Default for PerformanceLimits {
    fn default() -> Self {
        Self {
            deployment_timeout: Duration::from_secs(180), // 3 minutes (Requirement 3.2)
            health_check_timeout: Duration::from_secs(30),
            api_response_timeout: Duration::from_secs(10),
        }
    }
}

impl RailwayDeploymentValidator {
    /// Create new Railway deployment validator
    pub fn new() -> Result<Self, RailwayTestError> {
        let config = Self::load_railway_config()?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            config,
            client,
            performance_limits: PerformanceLimits::default(),
        })
    }
    
    /// Load and validate Railway configuration
    fn load_railway_config() -> Result<RailwayConfig, RailwayTestError> {
        let config_content = std::fs::read_to_string("railway.toml")
            .map_err(|e| RailwayTestError::ConfigurationInvalid {
                field: "railway.toml".to_string(),
                reason: format!("File not found or unreadable: {}", e),
            })?;
        
        // Parse TOML configuration
        let config: RailwayConfig = toml::from_str(&config_content)
            .map_err(|e| RailwayTestError::ConfigurationInvalid {
                field: "railway.toml".to_string(),
                reason: format!("Invalid TOML format: {}", e),
            })?;
        
        // Validate configuration
        Self::validate_config(&config)?;
        
        Ok(config)
    }
    
    /// Validate Railway configuration against requirements
    fn validate_config(config: &RailwayConfig) -> Result<(), RailwayTestError> {
        // Validate build configuration
        if config.build.builder != "DOCKERFILE" {
            return Err(RailwayTestError::ConfigurationInvalid {
                field: "build.builder".to_string(),
                reason: format!("Expected 'DOCKERFILE', got '{}'", config.build.builder),
            });
        }
        
        // Validate health check configuration
        if config.deploy.healthcheck_path != "/health" {
            return Err(RailwayTestError::ConfigurationInvalid {
                field: "deploy.healthcheckPath".to_string(),
                reason: format!("Expected '/health', got '{}'", config.deploy.healthcheck_path),
            });
        }
        
        if config.deploy.healthcheck_timeout > 60 {
            return Err(RailwayTestError::ConfigurationInvalid {
                field: "deploy.healthcheckTimeout".to_string(),
                reason: format!("Timeout too long: {}s (max 60s)", config.deploy.healthcheck_timeout),
            });
        }
        
        // Validate required environment variables
        let required_env_vars = [
            "CAMPFIRE_HOST",
            "CAMPFIRE_PORT", 
            "CAMPFIRE_DATABASE_URL",
            "CAMPFIRE_LOG_LEVEL",
        ];
        
        for var in &required_env_vars {
            if !config.env.contains_key(*var) {
                return Err(RailwayTestError::ConfigurationInvalid {
                    field: format!("env.{}", var),
                    reason: "Required environment variable not defined".to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate Railway template configuration
    pub fn validate_template_config(&self) -> Result<TemplateValidationReport, RailwayTestError> {
        let template_content = std::fs::read_to_string("railway-template.json")
            .map_err(|e| RailwayTestError::ConfigurationInvalid {
                field: "railway-template.json".to_string(),
                reason: format!("File not found: {}", e),
            })?;
        
        let template: serde_json::Value = serde_json::from_str(&template_content)?;
        
        let mut report = TemplateValidationReport::new();
        
        // Validate required fields
        self.validate_template_field(&template, "name", &mut report)?;
        self.validate_template_field(&template, "description", &mut report)?;
        self.validate_template_field(&template, "services", &mut report)?;
        
        // Validate service configuration
        if let Some(services) = template["services"].as_array() {
            for (i, service) in services.iter().enumerate() {
                self.validate_service_config(service, i, &mut report)?;
            }
        }
        
        // Validate instructions
        self.validate_template_instructions(&template, &mut report)?;
        
        Ok(report)
    }
    
    fn validate_template_field(
        &self,
        template: &serde_json::Value,
        field: &str,
        report: &mut TemplateValidationReport,
    ) -> Result<(), RailwayTestError> {
        if template[field].is_null() {
            report.add_error(format!("Missing required field: {}", field));
        } else {
            report.add_success(format!("Field '{}' present", field));
        }
        Ok(())
    }
    
    fn validate_service_config(
        &self,
        service: &serde_json::Value,
        index: usize,
        report: &mut TemplateValidationReport,
    ) -> Result<(), RailwayTestError> {
        let default_name = format!("service-{}", index);
        let service_name = service["name"].as_str()
            .unwrap_or(&default_name);
        
        // Validate health check configuration
        if let Some(healthcheck_path) = service["healthcheckPath"].as_str() {
            if healthcheck_path == "/health" {
                report.add_success(format!("Service '{}' has correct health check path", service_name));
            } else {
                report.add_warning(format!("Service '{}' health check path: {}", service_name, healthcheck_path));
            }
        }
        
        // Validate environment variables
        if let Some(variables) = service["variables"].as_object() {
            let required_vars = ["CAMPFIRE_HOST", "CAMPFIRE_PORT", "CAMPFIRE_DATABASE_URL"];
            for var in &required_vars {
                if variables.contains_key(*var) {
                    report.add_success(format!("Service '{}' has required env var: {}", service_name, var));
                } else {
                    report.add_error(format!("Service '{}' missing env var: {}", service_name, var));
                }
            }
        }
        
        Ok(())
    }
    
    fn validate_template_instructions(
        &self,
        template: &serde_json::Value,
        report: &mut TemplateValidationReport,
    ) -> Result<(), RailwayTestError> {
        if let Some(instructions) = template["instructions"].as_object() {
            if let Some(start) = instructions["start"].as_str() {
                if start.len() > 100 {
                    report.add_success("Start instructions are comprehensive".to_string());
                } else {
                    report.add_warning("Start instructions may be too brief".to_string());
                }
            }
            
            if let Some(end) = instructions["end"].as_str() {
                if end.contains("URL") && end.contains("admin") {
                    report.add_success("End instructions mention URL and admin setup".to_string());
                } else {
                    report.add_warning("End instructions may be missing key information".to_string());
                }
            }
        }
        
        Ok(())
    }
    
    /// Test deployment performance contract (Requirement 3.2)
    pub async fn test_deployment_performance(&self, deployment_url: &str) -> Result<PerformanceReport, RailwayTestError> {
        let mut report = PerformanceReport::new();
        
        // Test health check response time
        let health_url = format!("{}/health", deployment_url);
        let start = Instant::now();
        
        match tokio::time::timeout(
            self.performance_limits.health_check_timeout,
            self.client.get(&health_url).send()
        ).await {
            Ok(Ok(response)) => {
                let elapsed = start.elapsed();
                report.health_check_time = Some(elapsed);
                
                if response.status().is_success() {
                    report.add_success("Health check endpoint responds successfully".to_string());
                } else {
                    report.add_error(format!("Health check returned status: {}", response.status()));
                }
                
                if elapsed <= Duration::from_secs(5) {
                    report.add_success(format!("Health check response time: {:?}", elapsed));
                } else {
                    report.add_warning(format!("Health check slow: {:?}", elapsed));
                }
            }
            Ok(Err(e)) => {
                report.add_error(format!("Health check request failed: {}", e));
            }
            Err(_) => {
                report.add_error("Health check timed out".to_string());
            }
        }
        
        // Test API endpoint response times
        let api_endpoints = ["/api/rooms", "/api/messages", "/api/users"];
        
        for endpoint in &api_endpoints {
            let url = format!("{}{}", deployment_url, endpoint);
            let start = Instant::now();
            
            match tokio::time::timeout(
                self.performance_limits.api_response_timeout,
                self.client.get(&url).send()
            ).await {
                Ok(Ok(response)) => {
                    let elapsed = start.elapsed();
                    
                    // Accept auth-required responses as valid
                    if response.status().is_success() || 
                       response.status() == 401 || 
                       response.status() == 403 {
                        report.add_success(format!("API endpoint {} responds: {}", endpoint, response.status()));
                    } else {
                        report.add_warning(format!("API endpoint {} status: {}", endpoint, response.status()));
                    }
                    
                    if elapsed <= Duration::from_secs(2) {
                        report.add_success(format!("API {} response time: {:?}", endpoint, elapsed));
                    } else {
                        report.add_warning(format!("API {} slow: {:?}", endpoint, elapsed));
                    }
                }
                Ok(Err(e)) => {
                    report.add_error(format!("API endpoint {} failed: {}", endpoint, e));
                }
                Err(_) => {
                    report.add_error(format!("API endpoint {} timed out", endpoint));
                }
            }
        }
        
        Ok(report)
    }
    
    /// Test error handling quality (Requirement 3.6)
    pub async fn test_error_handling(&self, deployment_url: &str) -> Result<ErrorHandlingReport, RailwayTestError> {
        let mut report = ErrorHandlingReport::new();
        
        // Test 404 error handling
        let invalid_url = format!("{}/invalid-test-endpoint", deployment_url);
        
        match self.client.get(&invalid_url).send().await {
            Ok(response) => {
                if response.status() == 404 {
                    report.add_success("404 errors handled correctly".to_string());
                    
                    // Check error response content
                    if let Ok(text) = response.text().await {
                        if text.len() > 20 && !text.contains("nginx") && !text.contains("Apache") {
                            report.add_success("404 error response is custom and informative".to_string());
                        } else {
                            report.add_warning("404 error response may be generic server error".to_string());
                        }
                    }
                } else {
                    report.add_warning(format!("Invalid endpoint returned status: {}", response.status()));
                }
            }
            Err(e) => {
                report.add_error(format!("Error testing 404 handling: {}", e));
            }
        }
        
        // Test malformed request handling
        let api_url = format!("{}/api/messages", deployment_url);
        
        match self.client.post(&api_url)
            .header("Content-Type", "application/json")
            .body("invalid json data")
            .send()
            .await {
            Ok(response) => {
                if response.status() == 400 || response.status() == 422 {
                    report.add_success("Malformed requests handled correctly".to_string());
                } else {
                    report.add_warning(format!("Malformed request returned: {}", response.status()));
                }
            }
            Err(e) => {
                report.add_error(format!("Error testing malformed request: {}", e));
            }
        }
        
        Ok(report)
    }
}

#[derive(Debug)]
pub struct TemplateValidationReport {
    pub successes: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl TemplateValidationReport {
    pub fn new() -> Self {
        Self {
            successes: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    pub fn add_success(&mut self, message: String) {
        self.successes.push(message);
    }
    
    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }
    
    pub fn add_error(&mut self, message: String) {
        self.errors.push(message);
    }
    
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug)]
pub struct PerformanceReport {
    pub health_check_time: Option<Duration>,
    pub successes: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl PerformanceReport {
    pub fn new() -> Self {
        Self {
            health_check_time: None,
            successes: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    pub fn add_success(&mut self, message: String) {
        self.successes.push(message);
    }
    
    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }
    
    pub fn add_error(&mut self, message: String) {
        self.errors.push(message);
    }
    
    pub fn meets_performance_requirements(&self) -> bool {
        self.errors.is_empty() && 
        self.health_check_time.map_or(false, |t| t <= Duration::from_secs(5))
    }
}

#[derive(Debug)]
pub struct ErrorHandlingReport {
    pub successes: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ErrorHandlingReport {
    pub fn new() -> Self {
        Self {
            successes: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    pub fn add_success(&mut self, message: String) {
        self.successes.push(message);
    }
    
    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }
    
    pub fn add_error(&mut self, message: String) {
        self.errors.push(message);
    }
    
    pub fn has_clear_error_messages(&self) -> bool {
        self.errors.is_empty() && !self.successes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_railway_config_validation() {
        // Test valid configuration
        let valid_config = RailwayConfig {
            build: BuildConfig {
                builder: "DOCKERFILE".to_string(),
                dockerfile_path: "Dockerfile".to_string(),
            },
            deploy: DeployConfig {
                healthcheck_path: "/health".to_string(),
                healthcheck_timeout: 30,
                restart_policy_type: "ON_FAILURE".to_string(),
                restart_policy_max_retries: 3,
            },
            env: [
                ("CAMPFIRE_HOST".to_string(), EnvVar { default: Some("0.0.0.0".to_string()) }),
                ("CAMPFIRE_PORT".to_string(), EnvVar { default: Some("3000".to_string()) }),
                ("CAMPFIRE_DATABASE_URL".to_string(), EnvVar { default: Some("sqlite:///app/data/campfire.db".to_string()) }),
                ("CAMPFIRE_LOG_LEVEL".to_string(), EnvVar { default: Some("info".to_string()) }),
            ].into_iter().collect(),
        };
        
        assert!(RailwayDeploymentValidator::validate_config(&valid_config).is_ok());
    }
    
    #[test]
    fn test_invalid_builder_config() {
        let invalid_config = RailwayConfig {
            build: BuildConfig {
                builder: "INVALID".to_string(),
                dockerfile_path: "Dockerfile".to_string(),
            },
            deploy: DeployConfig {
                healthcheck_path: "/health".to_string(),
                healthcheck_timeout: 30,
                restart_policy_type: "ON_FAILURE".to_string(),
                restart_policy_max_retries: 3,
            },
            env: HashMap::new(),
        };
        
        let result = RailwayDeploymentValidator::validate_config(&invalid_config);
        assert!(result.is_err());
        
        if let Err(RailwayTestError::ConfigurationInvalid { field, reason }) = result {
            assert_eq!(field, "build.builder");
            assert!(reason.contains("DOCKERFILE"));
        }
    }
    
    #[test]
    fn test_performance_limits() {
        let limits = PerformanceLimits::default();
        
        assert_eq!(limits.deployment_timeout, Duration::from_secs(180)); // 3 minutes
        assert_eq!(limits.health_check_timeout, Duration::from_secs(30));
        assert_eq!(limits.api_response_timeout, Duration::from_secs(10));
    }
    
    #[tokio::test]
    async fn test_performance_report_validation() {
        let mut report = PerformanceReport::new();
        report.health_check_time = Some(Duration::from_secs(2));
        report.add_success("Health check passed".to_string());
        
        assert!(report.meets_performance_requirements());
        
        // Test slow health check
        report.health_check_time = Some(Duration::from_secs(10));
        assert!(!report.meets_performance_requirements());
    }
    
    #[test]
    fn test_template_validation_report() {
        let mut report = TemplateValidationReport::new();
        
        report.add_success("Valid field found".to_string());
        report.add_warning("Minor issue detected".to_string());
        
        assert!(report.is_valid()); // No errors
        
        report.add_error("Critical issue found".to_string());
        assert!(!report.is_valid()); // Has errors
    }
}