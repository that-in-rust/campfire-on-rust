use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tracing::Level;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Push notification configuration
    pub push: PushConfig,
    
    /// Metrics configuration
    pub metrics: MetricsConfig,
    
    /// Feature flags
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_address: SocketAddr,
    
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    
    /// Maximum request body size in bytes
    pub max_request_size: usize,
    
    /// Graceful shutdown timeout in seconds
    pub shutdown_timeout_secs: u64,
    
    /// Number of worker threads (0 = auto)
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database file path
    pub database_url: String,
    
    /// Connection pool size
    pub max_connections: u32,
    
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    
    /// Enable WAL mode for better concurrency
    pub enable_wal_mode: bool,
    
    /// Database backup directory
    pub backup_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    
    /// Log format (json, pretty, compact)
    pub format: LogFormat,
    
    /// Enable structured logging
    pub structured: bool,
    
    /// Log file path (None = stdout only)
    pub file_path: Option<PathBuf>,
    
    /// Enable request tracing
    pub trace_requests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// CORS allowed origins (empty = allow all)
    pub cors_origins: Vec<String>,
    
    /// Rate limiting: requests per minute
    pub rate_limit_rpm: u32,
    
    /// Session token length in bytes
    pub session_token_length: usize,
    
    /// Session expiry in hours
    pub session_expiry_hours: u64,
    
    /// Enable HTTPS redirect
    pub force_https: bool,
    
    /// Trusted proxy headers
    pub trust_proxy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushConfig {
    /// VAPID private key (base64 encoded)
    pub vapid_private_key: Option<String>,
    
    /// VAPID public key (base64 encoded)
    pub vapid_public_key: Option<String>,
    
    /// VAPID subject (email or URL)
    pub vapid_subject: String,
    
    /// Enable push notifications
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    
    /// Metrics endpoint path
    pub endpoint: String,
    
    /// Enable detailed request metrics
    pub detailed_requests: bool,
    
    /// Histogram buckets for response times
    pub response_time_buckets: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable WebSocket connections
    pub websockets: bool,
    
    /// Enable push notifications
    pub push_notifications: bool,
    
    /// Enable bot API
    pub bot_api: bool,
    
    /// Enable search functionality
    pub search: bool,
    
    /// Enable sound system
    pub sounds: bool,
    
    /// Enable file uploads (future feature)
    pub file_uploads: bool,
    
    /// Enable offline demo mode with sample data
    pub demo_mode: bool,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let config = Config {
            server: ServerConfig::from_env()?,
            database: DatabaseConfig::from_env()?,
            logging: LoggingConfig::from_env()?,
            security: SecurityConfig::from_env()?,
            push: PushConfig::from_env()?,
            metrics: MetricsConfig::from_env()?,
            features: FeatureFlags::from_env()?,
        };
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        // Validate server config
        if self.server.request_timeout_secs == 0 {
            return Err(anyhow::anyhow!("Request timeout must be greater than 0"));
        }
        
        if self.server.max_request_size == 0 {
            return Err(anyhow::anyhow!("Max request size must be greater than 0"));
        }
        
        // Validate database config
        if self.database.max_connections == 0 {
            return Err(anyhow::anyhow!("Database max connections must be greater than 0"));
        }
        
        // Validate security config
        if self.security.session_token_length < 16 {
            return Err(anyhow::anyhow!("Session token length must be at least 16 bytes"));
        }
        
        if self.security.session_expiry_hours == 0 {
            return Err(anyhow::anyhow!("Session expiry must be greater than 0 hours"));
        }
        
        // Validate push config if enabled
        if self.push.enabled {
            if self.push.vapid_private_key.is_none() || self.push.vapid_public_key.is_none() {
                return Err(anyhow::anyhow!("VAPID keys required when push notifications are enabled"));
            }
            
            if self.push.vapid_subject.is_empty() {
                return Err(anyhow::anyhow!("VAPID subject required when push notifications are enabled"));
            }
        }
        
        Ok(())
    }
    
    /// Get tracing level from config
    pub fn tracing_level(&self) -> Level {
        match self.logging.level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    }
    
    /// Get request timeout as Duration
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.server.request_timeout_secs)
    }
    
    /// Get shutdown timeout as Duration
    pub fn shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.server.shutdown_timeout_secs)
    }
    
    /// Get connection timeout as Duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.database.connection_timeout_secs)
    }
    
    /// Get session expiry as Duration
    pub fn session_expiry(&self) -> Duration {
        Duration::from_secs(self.security.session_expiry_hours * 3600)
    }
}

impl ServerConfig {
    fn from_env() -> Result<Self> {
        let host = env::var("CAMPFIRE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("CAMPFIRE_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .context("Invalid CAMPFIRE_PORT")?;
        
        Ok(ServerConfig {
            bind_address: SocketAddr::new(host.parse().context("Invalid CAMPFIRE_HOST")?, port),
            request_timeout_secs: env::var("CAMPFIRE_REQUEST_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("Invalid CAMPFIRE_REQUEST_TIMEOUT")?,
            max_request_size: env::var("CAMPFIRE_MAX_REQUEST_SIZE")
                .unwrap_or_else(|_| "16777216".to_string()) // 16MB
                .parse()
                .context("Invalid CAMPFIRE_MAX_REQUEST_SIZE")?,
            shutdown_timeout_secs: env::var("CAMPFIRE_SHUTDOWN_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("Invalid CAMPFIRE_SHUTDOWN_TIMEOUT")?,
            worker_threads: env::var("CAMPFIRE_WORKER_THREADS")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .context("Invalid CAMPFIRE_WORKER_THREADS")?,
        })
    }
}

impl DatabaseConfig {
    fn from_env() -> Result<Self> {
        Ok(DatabaseConfig {
            database_url: env::var("CAMPFIRE_DATABASE_URL")
                .unwrap_or_else(|_| "campfire.db".to_string()),
            max_connections: env::var("CAMPFIRE_DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("Invalid CAMPFIRE_DB_MAX_CONNECTIONS")?,
            connection_timeout_secs: env::var("CAMPFIRE_DB_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("Invalid CAMPFIRE_DB_TIMEOUT")?,
            enable_wal_mode: env::var("CAMPFIRE_DB_WAL_MODE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .context("Invalid CAMPFIRE_DB_WAL_MODE")?,
            backup_dir: env::var("CAMPFIRE_BACKUP_DIR")
                .ok()
                .map(PathBuf::from),
        })
    }
}

impl LoggingConfig {
    fn from_env() -> Result<Self> {
        let format = match env::var("CAMPFIRE_LOG_FORMAT")
            .unwrap_or_else(|_| "pretty".to_string())
            .to_lowercase()
            .as_str()
        {
            "json" => LogFormat::Json,
            "compact" => LogFormat::Compact,
            _ => LogFormat::Pretty,
        };
        
        Ok(LoggingConfig {
            level: env::var("CAMPFIRE_LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
            format,
            structured: env::var("CAMPFIRE_LOG_STRUCTURED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .context("Invalid CAMPFIRE_LOG_STRUCTURED")?,
            file_path: env::var("CAMPFIRE_LOG_FILE")
                .ok()
                .map(PathBuf::from),
            trace_requests: env::var("CAMPFIRE_TRACE_REQUESTS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .context("Invalid CAMPFIRE_TRACE_REQUESTS")?,
        })
    }
}

impl SecurityConfig {
    fn from_env() -> Result<Self> {
        let cors_origins = env::var("CAMPFIRE_CORS_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();
        
        Ok(SecurityConfig {
            cors_origins,
            rate_limit_rpm: env::var("CAMPFIRE_RATE_LIMIT_RPM")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .context("Invalid CAMPFIRE_RATE_LIMIT_RPM")?,
            session_token_length: env::var("CAMPFIRE_SESSION_TOKEN_LENGTH")
                .unwrap_or_else(|_| "32".to_string())
                .parse()
                .context("Invalid CAMPFIRE_SESSION_TOKEN_LENGTH")?,
            session_expiry_hours: env::var("CAMPFIRE_SESSION_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .context("Invalid CAMPFIRE_SESSION_EXPIRY_HOURS")?,
            force_https: env::var("CAMPFIRE_FORCE_HTTPS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .context("Invalid CAMPFIRE_FORCE_HTTPS")?,
            trust_proxy: env::var("CAMPFIRE_TRUST_PROXY")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .context("Invalid CAMPFIRE_TRUST_PROXY")?,
        })
    }
}

impl PushConfig {
    fn from_env() -> Result<Self> {
        Ok(PushConfig {
            vapid_private_key: env::var("CAMPFIRE_VAPID_PRIVATE_KEY").ok(),
            vapid_public_key: env::var("CAMPFIRE_VAPID_PUBLIC_KEY").ok(),
            vapid_subject: env::var("CAMPFIRE_VAPID_SUBJECT")
                .unwrap_or_else(|_| "mailto:admin@campfire.local".to_string()),
            enabled: env::var("CAMPFIRE_PUSH_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .context("Invalid CAMPFIRE_PUSH_ENABLED")?,
        })
    }
}

impl MetricsConfig {
    fn from_env() -> Result<Self> {
        let buckets = env::var("CAMPFIRE_METRICS_BUCKETS")
            .unwrap_or_else(|_| "0.001,0.005,0.01,0.05,0.1,0.5,1.0,5.0,10.0".to_string())
            .split(',')
            .map(|s| s.trim().parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .context("Invalid CAMPFIRE_METRICS_BUCKETS")?;
        
        Ok(MetricsConfig {
            enabled: env::var("CAMPFIRE_METRICS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .context("Invalid CAMPFIRE_METRICS_ENABLED")?,
            endpoint: env::var("CAMPFIRE_METRICS_ENDPOINT")
                .unwrap_or_else(|_| "/metrics".to_string()),
            detailed_requests: env::var("CAMPFIRE_METRICS_DETAILED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .context("Invalid CAMPFIRE_METRICS_DETAILED")?,
            response_time_buckets: buckets,
        })
    }
}

impl FeatureFlags {
    fn from_env() -> Result<Self> {
        Ok(FeatureFlags {
            websockets: env::var("CAMPFIRE_FEATURE_WEBSOCKETS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .context("Invalid CAMPFIRE_FEATURE_WEBSOCKETS")?,
            push_notifications: env::var("CAMPFIRE_FEATURE_PUSH")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .context("Invalid CAMPFIRE_FEATURE_PUSH")?,
            bot_api: env::var("CAMPFIRE_FEATURE_BOTS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .context("Invalid CAMPFIRE_FEATURE_BOTS")?,
            search: env::var("CAMPFIRE_FEATURE_SEARCH")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .context("Invalid CAMPFIRE_FEATURE_SEARCH")?,
            sounds: env::var("CAMPFIRE_FEATURE_SOUNDS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .context("Invalid CAMPFIRE_FEATURE_SOUNDS")?,
            file_uploads: env::var("CAMPFIRE_FEATURE_FILES")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .context("Invalid CAMPFIRE_FEATURE_FILES")?,
            demo_mode: env::var("CAMPFIRE_DEMO_MODE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .context("Invalid CAMPFIRE_DEMO_MODE")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_config_defaults() {
        // Clear environment variables
        for key in env::vars().map(|(k, _)| k).collect::<Vec<_>>() {
            if key.starts_with("CAMPFIRE_") {
                env::remove_var(key);
            }
        }
        
        // Set required VAPID keys for test
        env::set_var("CAMPFIRE_VAPID_PRIVATE_KEY", "test_private_key");
        env::set_var("CAMPFIRE_VAPID_PUBLIC_KEY", "test_public_key");
        
        let config = Config::from_env().unwrap();
        
        assert_eq!(config.server.bind_address.port(), 3000);
        assert_eq!(config.database.database_url, "campfire.db");
        assert_eq!(config.logging.level, "info");
        assert!(config.features.websockets);
    }
    
    #[test]
    fn test_config_validation() {
        // Test invalid session token length
        env::set_var("CAMPFIRE_SESSION_TOKEN_LENGTH", "8");
        let result = Config::from_env();
        assert!(result.is_err());
        
        env::remove_var("CAMPFIRE_SESSION_TOKEN_LENGTH");
    }
}