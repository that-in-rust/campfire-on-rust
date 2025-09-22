use async_trait::async_trait;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, Rng};
use std::sync::Arc;

use crate::database::Database;
use crate::errors::{AuthError, DatabaseError};
use crate::models::{Session, User, UserId};

#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    /// Authenticates user with email/password
    async fn authenticate(&self, email: String, password: String) -> Result<Session, AuthError>;
    
    /// Creates secure session token (Critical Gap #4)
    async fn create_session(&self, user_id: UserId) -> Result<Session, AuthError>;
    
    /// Validates session token
    async fn validate_session(&self, token: String) -> Result<User, AuthError>;
    
    /// Revokes session
    async fn revoke_session(&self, token: String) -> Result<(), AuthError>;
    
    /// Creates a new user
    async fn create_user(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<User, AuthError>;
}

#[derive(Clone)]
pub struct AuthService {
    db: Arc<Database>,
}

impl AuthService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
    
    /// Generates cryptographically secure session token (Critical Gap #4)
    /// 
    /// Uses Rails-equivalent secure token generation with:
    /// - 32 bytes of cryptographically secure random data
    /// - Base64 URL-safe encoding
    /// - No predictable patterns
    fn generate_secure_token() -> Result<String, AuthError> {
        let mut token_bytes = [0u8; 32];
        thread_rng().fill(&mut token_bytes);
        
        // Use base64 URL-safe encoding (no padding)
        let token = base64::encode_config(&token_bytes, base64::URL_SAFE_NO_PAD);
        
        if token.len() < 32 {
            return Err(AuthError::TokenGeneration);
        }
        
        Ok(token)
    }
    
    /// Validates password strength
    fn validate_password(password: &str) -> Result<(), AuthError> {
        if password.len() < 8 {
            return Err(AuthError::WeakPassword);
        }
        Ok(())
    }
    
    /// Validates email format
    fn validate_email(email: &str) -> Result<(), AuthError> {
        if !email.contains('@') || !email.contains('.') {
            return Err(AuthError::InvalidEmail { 
                email: email.to_string() 
            });
        }
        Ok(())
    }
}

#[async_trait]
impl AuthServiceTrait for AuthService {
    async fn authenticate(&self, email: String, password: String) -> Result<Session, AuthError> {
        // Get user by email
        let user = self.db.get_user_by_email(&email)
            .await?
            .ok_or_else(|| AuthError::UserNotFound { email: email.clone() })?;
        
        // Verify password
        if !verify(&password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }
        
        // Create session
        self.create_session(user.id).await
    }
    
    async fn create_session(&self, user_id: UserId) -> Result<Session, AuthError> {
        let token = Self::generate_secure_token()?;
        let now = Utc::now();
        let expires_at = now + Duration::days(30); // 30-day session
        
        let session = Session {
            token: token.clone(),
            user_id,
            created_at: now,
            expires_at,
        };
        
        self.db.create_session(&session)
            .await?;
        
        Ok(session)
    }
    
    async fn validate_session(&self, token: String) -> Result<User, AuthError> {
        // Get session from database
        let session = self.db.get_session(&token)
            .await?
            .ok_or(AuthError::SessionExpired)?;
        
        // Check if session is expired (additional check beyond database query)
        if session.expires_at < Utc::now() {
            // Clean up expired session
            let _ = self.db.delete_session(&token).await;
            return Err(AuthError::SessionExpired);
        }
        
        // Get user
        let user = self.db.get_user_by_id(session.user_id)
            .await?
            .ok_or_else(|| AuthError::UserNotFound { 
                email: "unknown".to_string() 
            })?;
        
        Ok(user)
    }
    
    async fn revoke_session(&self, token: String) -> Result<(), AuthError> {
        self.db.delete_session(&token)
            .await?;
        
        Ok(())
    }
    
    async fn create_user(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<User, AuthError> {
        // Validate inputs
        Self::validate_email(&email)?;
        Self::validate_password(&password)?;
        
        if name.trim().is_empty() || name.len() > 50 {
            return Err(AuthError::InvalidEmail { 
                email: "Invalid name length".to_string() 
            });
        }
        
        // Check if email already exists
        if self.db.get_user_by_email(&email).await?.is_some() {
            return Err(AuthError::EmailExists { email });
        }
        
        // Hash password
        let password_hash = hash(&password, DEFAULT_COST)?;
        
        // Create user
        let user = User {
            id: UserId::new(),
            name,
            email,
            password_hash,
            bio: None,
            admin: false,
            bot_token: None,
            created_at: Utc::now(),
        };
        
        self.db.create_user(&user)
            .await?;
        
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    
    async fn create_test_auth_service() -> AuthService {
        let db = Database::new(":memory:").await.unwrap();
        AuthService::new(Arc::new(db))
    }
    
    #[tokio::test]
    async fn test_secure_token_generation() {
        // Test Critical Gap #4: Session Token Security
        let token1 = AuthService::generate_secure_token().unwrap();
        let token2 = AuthService::generate_secure_token().unwrap();
        
        // Tokens should be unique
        assert_ne!(token1, token2);
        
        // Tokens should have sufficient entropy (at least 32 chars)
        assert!(token1.len() >= 32);
        assert!(token2.len() >= 32);
        
        // Tokens should be URL-safe (no special characters that need encoding)
        assert!(!token1.contains('+'));
        assert!(!token1.contains('/'));
        assert!(!token1.contains('='));
    }
    
    #[tokio::test]
    async fn test_user_creation_and_authentication() {
        let auth_service = create_test_auth_service().await;
        
        // Create user
        let user = auth_service.create_user(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password123".to_string(),
        ).await.unwrap();
        
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
        assert!(!user.admin);
        
        // Authenticate user
        let session = auth_service.authenticate(
            "test@example.com".to_string(),
            "password123".to_string(),
        ).await.unwrap();
        
        assert_eq!(session.user_id, user.id);
        assert!(session.expires_at > Utc::now());
        
        // Validate session
        let validated_user = auth_service.validate_session(session.token.clone()).await.unwrap();
        assert_eq!(validated_user.id, user.id);
        
        // Revoke session
        auth_service.revoke_session(session.token.clone()).await.unwrap();
        
        // Session should no longer be valid
        assert!(auth_service.validate_session(session.token).await.is_err());
    }
    
    #[tokio::test]
    async fn test_invalid_credentials() {
        let auth_service = create_test_auth_service().await;
        
        // Create user
        auth_service.create_user(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password123".to_string(),
        ).await.unwrap();
        
        // Wrong password
        let result = auth_service.authenticate(
            "test@example.com".to_string(),
            "wrongpassword".to_string(),
        ).await;
        
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
        
        // Wrong email
        let result = auth_service.authenticate(
            "wrong@example.com".to_string(),
            "password123".to_string(),
        ).await;
        
        assert!(matches!(result, Err(AuthError::UserNotFound { .. })));
    }
    
    #[tokio::test]
    async fn test_password_validation() {
        let auth_service = create_test_auth_service().await;
        
        // Too short password
        let result = auth_service.create_user(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "short".to_string(),
        ).await;
        
        assert!(matches!(result, Err(AuthError::WeakPassword)));
    }
    
    #[tokio::test]
    async fn test_email_validation() {
        let auth_service = create_test_auth_service().await;
        
        // Invalid email
        let result = auth_service.create_user(
            "Test User".to_string(),
            "invalid-email".to_string(),
            "password123".to_string(),
        ).await;
        
        assert!(matches!(result, Err(AuthError::InvalidEmail { .. })));
    }
    
    #[tokio::test]
    async fn test_duplicate_email() {
        let auth_service = create_test_auth_service().await;
        
        // Create first user
        auth_service.create_user(
            "User One".to_string(),
            "test@example.com".to_string(),
            "password123".to_string(),
        ).await.unwrap();
        
        // Try to create second user with same email
        let result = auth_service.create_user(
            "User Two".to_string(),
            "test@example.com".to_string(),
            "password456".to_string(),
        ).await;
        
        assert!(matches!(result, Err(AuthError::EmailExists { .. })));
    }
}