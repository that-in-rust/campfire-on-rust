use async_trait::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{Duration, Utc};
use rand::Rng;
use sqlx::Row;
use std::env;

use crate::database::CampfireDatabase;
use crate::errors::{SetupError, DatabaseError};
use crate::models::{
    User, UserId, Session, DeploymentConfig, SystemHealth,
    CreateAdminRequest, SetupStatusResponse, AdminCreationResponse,
};

/// First-Run Setup Service - Basecamp-style admin setup (Requirement 11)
/// 
/// Provides simple, guided first-run experience that creates the initial admin account
/// and configures the system for production deployment.
#[async_trait]
pub trait SetupService: Send + Sync {
    /// Detects if this is a first-run scenario
    /// 
    /// # Preconditions
    /// - Database is accessible
    /// - Not in demo mode
    /// 
    /// # Postconditions
    /// - Returns true if no users exist in database
    /// - Indicates first-run setup is needed
    async fn is_first_run(&self) -> Result<bool, SetupError>;
    
    /// Creates initial admin account
    /// 
    /// # Preconditions
    /// - First-run condition verified
    /// - Valid email and password provided
    /// - Email format validated
    /// - Password strength requirements met
    /// 
    /// # Postconditions
    /// - Creates admin user with full permissions
    /// - Marks user as primary administrator
    /// - Returns created user and session token
    /// - Enables subsequent normal login flow
    async fn create_admin_account(
        &self,
        request: CreateAdminRequest,
    ) -> Result<AdminCreationResponse, SetupError>;
    
    /// Gets environment-based configuration
    async fn get_deployment_config(&self) -> Result<DeploymentConfig, SetupError>;
    
    /// Validates system readiness for production
    async fn validate_system_health(&self) -> Result<SystemHealth, SetupError>;
    
    /// Gets complete setup status for UI
    async fn get_setup_status(&self) -> Result<SetupStatusResponse, SetupError>;
}

/// Implementation of SetupService following Rails-style patterns
pub struct SetupServiceImpl {
    database: CampfireDatabase,
}

impl SetupServiceImpl {
    pub fn new(database: CampfireDatabase) -> Self {
        Self { database }
    }
    
    /// Validates email format using simple regex
    fn validate_email(&self, email: &str) -> Result<(), SetupError> {
        if email.is_empty() {
            return Err(SetupError::InvalidEmail { 
                email: email.to_string() 
            });
        }
        
        // Simple email validation - contains @ and .
        if !email.contains('@') || !email.contains('.') {
            return Err(SetupError::InvalidEmail { 
                email: email.to_string() 
            });
        }
        
        // Check for basic format: something@something.something
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(SetupError::InvalidEmail { 
                email: email.to_string() 
            });
        }
        
        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        if domain_parts.len() < 2 || domain_parts.iter().any(|part| part.is_empty()) {
            return Err(SetupError::InvalidEmail { 
                email: email.to_string() 
            });
        }
        
        Ok(())
    }
    
    /// Validates password strength
    fn validate_password(&self, password: &str) -> Result<(), SetupError> {
        if password.len() < 8 {
            return Err(SetupError::WeakPassword { 
                reason: "Password must be at least 8 characters long".to_string() 
            });
        }
        
        if password.len() > 128 {
            return Err(SetupError::WeakPassword { 
                reason: "Password must be less than 128 characters".to_string() 
            });
        }
        
        // Check for at least one letter and one number
        let has_letter = password.chars().any(|c| c.is_alphabetic());
        let has_number = password.chars().any(|c| c.is_numeric());
        
        if !has_letter || !has_number {
            return Err(SetupError::WeakPassword { 
                reason: "Password must contain at least one letter and one number".to_string() 
            });
        }
        
        Ok(())
    }
    
    /// Validates user name
    fn validate_name(&self, name: &str) -> Result<(), SetupError> {
        if name.trim().is_empty() {
            return Err(SetupError::AdminCreationFailed(
                "Name cannot be empty".to_string()
            ));
        }
        
        if name.len() > 100 {
            return Err(SetupError::AdminCreationFailed(
                "Name must be less than 100 characters".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Generates secure session token
    fn generate_session_token(&self) -> String {
        let mut rng = rand::thread_rng();
        let token_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(token_bytes)
    }
    
    /// Checks if any users exist in the database
    async fn has_existing_users(&self) -> Result<bool, DatabaseError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(self.database.pool())
            .await?;
        
        let count: i64 = row.get("count");
        Ok(count > 0)
    }
    
    /// Checks if admin user exists
    async fn has_admin_user(&self) -> Result<bool, DatabaseError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM users WHERE admin = TRUE")
            .fetch_one(self.database.pool())
            .await?;
        
        let count: i64 = row.get("count");
        Ok(count > 0)
    }
    
    /// Creates user directly in database (bypasses writer for setup)
    async fn create_user_direct(&self, user: &User) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, name, email, password_hash, bio, admin, bot_token, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(user.id.0.to_string())
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.bio)
        .bind(user.admin)
        .bind(&user.bot_token)
        .bind(user.created_at)
        .execute(self.database.pool())
        .await?;
        
        Ok(())
    }
    
    /// Creates session directly in database (bypasses writer for setup)
    async fn create_session_direct(&self, session: &Session) -> Result<(), DatabaseError> {
        sqlx::query(
            "INSERT INTO sessions (token, user_id, created_at, expires_at) VALUES (?, ?, ?, ?)"
        )
        .bind(&session.token)
        .bind(session.user_id.0.to_string())
        .bind(session.created_at)
        .bind(session.expires_at)
        .execute(self.database.pool())
        .await?;
        
        Ok(())
    }
}

#[async_trait]
impl SetupService for SetupServiceImpl {
    async fn is_first_run(&self) -> Result<bool, SetupError> {
        // Check if demo mode is enabled - if so, not first run
        if env::var("CAMPFIRE_DEMO_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false)
        {
            return Ok(false);
        }
        
        // Check if any users exist
        let has_users = self.has_existing_users().await?;
        Ok(!has_users)
    }
    
    async fn create_admin_account(
        &self,
        request: CreateAdminRequest,
    ) -> Result<AdminCreationResponse, SetupError> {
        // Verify this is still a first-run scenario
        if !self.is_first_run().await? {
            return Err(SetupError::NotFirstRun);
        }
        
        // Validate input
        self.validate_email(&request.email)?;
        self.validate_password(&request.password)?;
        self.validate_name(&request.name)?;
        
        // Hash password
        let password_hash = hash(&request.password, DEFAULT_COST)?;
        
        // Create admin user
        let user = User {
            id: UserId::new(),
            name: request.name.trim().to_string(),
            email: request.email.trim().to_lowercase(),
            password_hash,
            bio: Some("System Administrator".to_string()),
            admin: true,
            bot_token: None,
            created_at: Utc::now(),
        };
        
        // Create user in database
        self.create_user_direct(&user).await
            .map_err(|e| SetupError::AdminCreationFailed(e.to_string()))?;
        
        // Create session token
        let session_token = self.generate_session_token();
        let session = Session {
            token: session_token.clone(),
            user_id: user.id,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24), // 24 hour session
        };
        
        // Create session in database
        self.create_session_direct(&session).await
            .map_err(|e| SetupError::SessionCreation(e.into()))?;
        
        // Get deployment config
        let deployment_config = self.get_deployment_config().await?;
        
        Ok(AdminCreationResponse {
            user,
            session_token,
            deployment_config,
        })
    }
    
    async fn get_deployment_config(&self) -> Result<DeploymentConfig, SetupError> {
        Ok(DeploymentConfig {
            database_url: env::var("CAMPFIRE_DATABASE_URL")
                .unwrap_or_else(|_| "campfire.db".to_string()),
            vapid_public_key: env::var("CAMPFIRE_VAPID_PUBLIC_KEY").ok(),
            vapid_private_key: env::var("CAMPFIRE_VAPID_PRIVATE_KEY").ok(),
            ssl_domain: env::var("CAMPFIRE_SSL_DOMAIN").ok(),
            session_timeout_hours: env::var("CAMPFIRE_SESSION_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .map_err(|_| SetupError::InvalidConfiguration { 
                    field: "CAMPFIRE_SESSION_EXPIRY_HOURS".to_string() 
                })?,
            max_message_length: env::var("CAMPFIRE_MAX_MESSAGE_LENGTH")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .map_err(|_| SetupError::InvalidConfiguration { 
                    field: "CAMPFIRE_MAX_MESSAGE_LENGTH".to_string() 
                })?,
            enable_user_registration: env::var("CAMPFIRE_ENABLE_REGISTRATION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .map_err(|_| SetupError::InvalidConfiguration { 
                    field: "CAMPFIRE_ENABLE_REGISTRATION".to_string() 
                })?,
            demo_mode: env::var("CAMPFIRE_DEMO_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .map_err(|_| SetupError::InvalidConfiguration { 
                    field: "CAMPFIRE_DEMO_MODE".to_string() 
                })?,
        })
    }
    
    async fn validate_system_health(&self) -> Result<SystemHealth, SetupError> {
        // Check database connectivity
        let database_connected = self.database.ping().await.is_ok();
        
        // Check FTS search availability
        let fts_search_available = sqlx::query("SELECT * FROM messages_fts LIMIT 1")
            .fetch_optional(self.database.pool())
            .await
            .is_ok();
        
        // WebSocket readiness (always true for this implementation)
        let websocket_ready = true;
        
        // Check push notification configuration
        let push_notifications_configured = env::var("CAMPFIRE_VAPID_PUBLIC_KEY").is_ok() 
            && env::var("CAMPFIRE_VAPID_PRIVATE_KEY").is_ok();
        
        // Static assets are always embedded in this implementation
        let static_assets_embedded = true;
        
        // Check if admin account exists
        let admin_account_exists = self.has_admin_user().await
            .map_err(|e| SetupError::HealthCheckFailed { 
                component: format!("admin_check: {}", e) 
            })?;
        
        Ok(SystemHealth {
            database_connected,
            fts_search_available,
            websocket_ready,
            push_notifications_configured,
            static_assets_embedded,
            admin_account_exists,
        })
    }
    
    async fn get_setup_status(&self) -> Result<SetupStatusResponse, SetupError> {
        let is_first_run = self.is_first_run().await?;
        let admin_exists = self.has_admin_user().await?;
        let system_health = self.validate_system_health().await?;
        
        Ok(SetupStatusResponse {
            is_first_run,
            admin_exists,
            system_health,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    
    async fn create_test_setup_service() -> SetupServiceImpl {
        // Use in-memory SQLite database for tests
        let database = CampfireDatabase::new("sqlite::memory:").await.unwrap();
        SetupServiceImpl::new(database)
    }
    
    #[tokio::test]
    async fn test_is_first_run_empty_database() {
        let service = create_test_setup_service().await;
        
        // Empty database should be first run
        assert!(service.is_first_run().await.unwrap());
    }
    
    #[tokio::test]
    async fn test_email_validation() {
        let service = create_test_setup_service().await;
        
        // Valid emails
        assert!(service.validate_email("admin@example.com").is_ok());
        assert!(service.validate_email("user.name@domain.co.uk").is_ok());
        
        // Invalid emails
        assert!(service.validate_email("").is_err());
        assert!(service.validate_email("invalid").is_err());
        assert!(service.validate_email("@domain.com").is_err());
        assert!(service.validate_email("user@").is_err());
        assert!(service.validate_email("user@domain").is_err());
    }
    
    #[tokio::test]
    async fn test_password_validation() {
        let service = create_test_setup_service().await;
        
        // Valid passwords
        assert!(service.validate_password("password123").is_ok());
        assert!(service.validate_password("MySecure1Pass").is_ok());
        
        // Invalid passwords
        assert!(service.validate_password("short").is_err()); // Too short
        assert!(service.validate_password("onlyletters").is_err()); // No numbers
        assert!(service.validate_password("12345678").is_err()); // No letters
        assert!(service.validate_password(&"x".repeat(129)).is_err()); // Too long
    }
    
    #[tokio::test]
    async fn test_create_admin_account() {
        let service = create_test_setup_service().await;
        
        let request = CreateAdminRequest {
            email: "admin@example.com".to_string(),
            password: "securepass123".to_string(),
            name: "System Admin".to_string(),
        };
        
        let response = service.create_admin_account(request).await.unwrap();
        
        assert_eq!(response.user.email, "admin@example.com");
        assert_eq!(response.user.name, "System Admin");
        assert!(response.user.admin);
        assert!(!response.session_token.is_empty());
        
        // Should not be first run anymore
        assert!(!service.is_first_run().await.unwrap());
    }
    
    #[tokio::test]
    async fn test_create_admin_account_not_first_run() {
        let service = create_test_setup_service().await;
        
        // Create first admin
        let request1 = CreateAdminRequest {
            email: "admin1@example.com".to_string(),
            password: "securepass123".to_string(),
            name: "First Admin".to_string(),
        };
        service.create_admin_account(request1).await.unwrap();
        
        // Try to create second admin - should fail
        let request2 = CreateAdminRequest {
            email: "admin2@example.com".to_string(),
            password: "securepass456".to_string(),
            name: "Second Admin".to_string(),
        };
        
        let result = service.create_admin_account(request2).await;
        assert!(matches!(result, Err(SetupError::NotFirstRun)));
    }
    
    #[tokio::test]
    async fn test_system_health_check() {
        let service = create_test_setup_service().await;
        
        let health = service.validate_system_health().await.unwrap();
        
        assert!(health.database_connected);
        assert!(health.fts_search_available);
        assert!(health.websocket_ready);
        assert!(health.static_assets_embedded);
        assert!(!health.admin_account_exists); // No admin created yet
    }
    
    #[tokio::test]
    async fn test_deployment_config() {
        let service = create_test_setup_service().await;
        
        let config = service.get_deployment_config().await.unwrap();
        
        assert!(!config.database_url.is_empty());
        assert_eq!(config.session_timeout_hours, 24);
        assert_eq!(config.max_message_length, 10000);
        assert!(!config.enable_user_registration);
        assert!(!config.demo_mode);
    }
}