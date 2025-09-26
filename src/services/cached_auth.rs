use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

use crate::database::CampfireDatabase;
use crate::errors::AuthError;
use crate::models::{Session, User, UserId};
use crate::services::auth::{AuthService, AuthServiceTrait};
use crate::services::cache::CacheServiceTrait;

/// Cached authentication service that wraps the base AuthService
/// 
/// Provides caching for:
/// - Session validation (most frequent operation)
/// - User lookups by session token
/// 
/// Cache TTLs:
/// - Sessions: 30 minutes (balances performance vs security)
/// - Failed lookups: 5 minutes (prevents repeated DB queries for invalid tokens)
#[derive(Clone)]
pub struct CachedAuthService {
    auth_service: AuthService,
    cache_service: Arc<dyn CacheServiceTrait>,
}

impl CachedAuthService {
    pub fn new(
        db: Arc<CampfireDatabase>,
        cache_service: Arc<dyn CacheServiceTrait>,
    ) -> Self {
        Self {
            auth_service: AuthService::new(db),
            cache_service,
        }
    }
    
    /// Session cache TTL - 30 minutes for security balance
    const SESSION_CACHE_TTL: Duration = Duration::from_secs(1800);
}

#[async_trait]
impl AuthServiceTrait for CachedAuthService {
    async fn authenticate(&self, email: String, password: String) -> Result<Session, AuthError> {
        // Authentication always goes to database for security
        // We don't cache password verification results
        let session = self.auth_service.authenticate(email, password).await?;
        
        // Cache the user for the new session
        if let Ok(user) = self.auth_service.validate_session(session.token.clone()).await {
            let _ = self.cache_service.cache_session(
                session.token.clone(),
                user,
                Self::SESSION_CACHE_TTL,
            ).await;
        }
        
        Ok(session)
    }
    
    async fn create_session(&self, user_id: UserId) -> Result<Session, AuthError> {
        // Create session in database
        let session = self.auth_service.create_session(user_id).await?;
        
        // Cache the user for the new session
        if let Ok(user) = self.auth_service.validate_session(session.token.clone()).await {
            let _ = self.cache_service.cache_session(
                session.token.clone(),
                user,
                Self::SESSION_CACHE_TTL,
            ).await;
        }
        
        Ok(session)
    }
    
    async fn validate_session(&self, token: String) -> Result<User, AuthError> {
        // Try cache first
        match self.cache_service.get_cached_session(&token).await {
            Ok(Some(user)) => {
                tracing::debug!("Session cache hit for token: {}", &token[..8]);
                return Ok(user);
            }
            Ok(None) => {
                tracing::debug!("Session cache miss for token: {}", &token[..8]);
            }
            Err(e) => {
                tracing::warn!("Session cache error for token {}: {}", &token[..8], e);
            }
        }
        
        // Cache miss - validate with database
        match self.auth_service.validate_session(token.clone()).await {
            Ok(user) => {
                // Cache the successful validation
                if let Err(e) = self.cache_service.cache_session(
                    token.clone(),
                    user.clone(),
                    Self::SESSION_CACHE_TTL,
                ).await {
                    tracing::warn!("Failed to cache session for token {}: {}", &token[..8], e);
                }
                Ok(user)
            }
            Err(e) => {
                // Don't cache failures for security reasons
                // Invalid tokens should always hit the database
                Err(e)
            }
        }
    }
    
    async fn revoke_session(&self, token: String) -> Result<(), AuthError> {
        // Revoke in database first
        let result = self.auth_service.revoke_session(token.clone()).await;
        
        // Remove from cache regardless of database result
        if let Err(e) = self.cache_service.invalidate_session(&token).await {
            tracing::warn!("Failed to invalidate cached session for token {}: {}", &token[..8], e);
        }
        
        result
    }
    
    async fn create_user(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<User, AuthError> {
        // User creation always goes to database
        // No caching needed for this infrequent operation
        self.auth_service.create_user(name, email, password).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::CampfireDatabase;
    use crate::services::cache::CacheService;
    
    async fn create_test_cached_auth_service() -> CachedAuthService {
        let db = CampfireDatabase::new(":memory:").await.unwrap();
        let cache_service = Arc::new(CacheService::with_defaults());
        CachedAuthService::new(Arc::new(db), cache_service)
    }
    
    #[tokio::test]
    async fn test_session_caching() {
        let service = create_test_cached_auth_service().await;
        
        // Create a user first
        let user = service.create_user(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password123".to_string(),
        ).await.unwrap();
        
        // Authenticate to get a session
        let session = service.authenticate(
            "test@example.com".to_string(),
            "password123".to_string(),
        ).await.unwrap();
        
        // First validation should hit database and cache the result
        let validated_user1 = service.validate_session(session.token.clone()).await.unwrap();
        assert_eq!(validated_user1.id, user.id);
        
        // Second validation should hit cache
        let validated_user2 = service.validate_session(session.token.clone()).await.unwrap();
        assert_eq!(validated_user2.id, user.id);
        
        // Revoke session should remove from cache
        service.revoke_session(session.token.clone()).await.unwrap();
        
        // Validation should now fail
        let result = service.validate_session(session.token).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_cache_miss_fallback() {
        let service = create_test_cached_auth_service().await;
        
        // Create a user
        let user = service.create_user(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password123".to_string(),
        ).await.unwrap();
        
        // Create session directly through auth service (bypassing cache)
        let session = service.auth_service.create_session(user.id).await.unwrap();
        
        // Validation should work even though not in cache (fallback to DB)
        let validated_user = service.validate_session(session.token.clone()).await.unwrap();
        assert_eq!(validated_user.id, user.id);
        
        // Second call should now hit cache
        let validated_user2 = service.validate_session(session.token).await.unwrap();
        assert_eq!(validated_user2.id, user.id);
    }
    
    #[tokio::test]
    async fn test_authentication_does_not_cache_failures() {
        let service = create_test_cached_auth_service().await;
        
        // Create a user
        service.create_user(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password123".to_string(),
        ).await.unwrap();
        
        // Failed authentication should not be cached
        let result1 = service.authenticate(
            "test@example.com".to_string(),
            "wrongpassword".to_string(),
        ).await;
        assert!(result1.is_err());
        
        // Second attempt should still hit database (not cached)
        let result2 = service.authenticate(
            "test@example.com".to_string(),
            "wrongpassword".to_string(),
        ).await;
        assert!(result2.is_err());
    }
}