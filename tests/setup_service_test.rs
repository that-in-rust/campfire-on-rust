use campfire_on_rust::CampfireDatabase;
use campfire_on_rust::errors::SetupError;
use campfire_on_rust::models::CreateAdminRequest;
use campfire_on_rust::services::{SetupService, SetupServiceImpl};
use std::env;

/// Test suite for SetupService implementation
/// Validates Requirements 11.1, 11.2, 11.3, 11.4

async fn create_test_setup_service() -> SetupServiceImpl {
    let database = CampfireDatabase::new("sqlite::memory:").await.unwrap();
    SetupServiceImpl::new(database)
}

#[tokio::test]
async fn test_requirement_11_1_first_run_detection() {
    // Requirement 11.1: Detects if this is a first-run scenario
    
    // Ensure clean environment
    env::remove_var("CAMPFIRE_DEMO_MODE");
    
    let service = create_test_setup_service().await;
    
    // Empty database should be first run
    assert!(service.is_first_run().await.unwrap());
    
    // Create admin account
    let request = CreateAdminRequest {
        email: "admin@example.com".to_string(),
        password: "securepass123".to_string(),
        name: "System Admin".to_string(),
    };
    
    service.create_admin_account(request).await.unwrap();
    
    // Should not be first run anymore
    assert!(!service.is_first_run().await.unwrap());
}

#[tokio::test]
async fn test_requirement_11_1_demo_mode_detection() {
    // Requirement 11.1: Not first run when demo mode is enabled
    
    // Ensure clean start
    env::remove_var("CAMPFIRE_DEMO_MODE");
    
    // Test normal first run
    let service = create_test_setup_service().await;
    assert!(service.is_first_run().await.unwrap());
    
    // Now test with demo mode enabled
    env::set_var("CAMPFIRE_DEMO_MODE", "true");
    
    // Should not be first run when demo mode is enabled (same service instance)
    assert!(!service.is_first_run().await.unwrap());
    
    // Clean up
    env::remove_var("CAMPFIRE_DEMO_MODE");
}

#[tokio::test]
async fn test_requirement_11_2_admin_account_creation() {
    // Requirement 11.2: Creates initial admin account with validation
    
    // Ensure clean environment
    env::remove_var("CAMPFIRE_DEMO_MODE");
    
    let service = create_test_setup_service().await;
    
    let request = CreateAdminRequest {
        email: "admin@campfire.local".to_string(),
        password: "MySecurePassword123".to_string(),
        name: "Primary Administrator".to_string(),
    };
    
    let response = service.create_admin_account(request).await.unwrap();
    
    // Verify admin user properties
    assert_eq!(response.user.email, "admin@campfire.local");
    assert_eq!(response.user.name, "Primary Administrator");
    assert!(response.user.admin);
    assert_eq!(response.user.bio, Some("System Administrator".to_string()));
    
    // Verify session token is provided
    assert!(!response.session_token.is_empty());
    assert_eq!(response.session_token.len(), 64); // 32 bytes hex encoded
    
    // Verify deployment config is included
    assert!(!response.deployment_config.database_url.is_empty());
}

#[tokio::test]
async fn test_requirement_11_2_email_validation() {
    // Requirement 11.2: Email format validation
    
    // Test invalid emails
    let invalid_emails = vec![
        "",
        "invalid",
        "@domain.com",
        "user@",
        "user@domain",
        "user.domain.com",
    ];
    
    for email in invalid_emails {
        // Ensure clean environment for each test
        env::remove_var("CAMPFIRE_DEMO_MODE");
        let service = create_test_setup_service().await;
        
        let request = CreateAdminRequest {
            email: email.to_string(),
            password: "validpass123".to_string(),
            name: "Admin".to_string(),
        };
        
        let result = service.create_admin_account(request).await;
        assert!(matches!(result, Err(SetupError::InvalidEmail { .. })), 
                "Email '{}' should be invalid", email);
    }
    
    // Test valid emails
    let valid_emails = vec![
        "admin@example.com",
        "user.name@domain.co.uk",
        "test+tag@subdomain.example.org",
    ];
    
    for (i, email) in valid_emails.iter().enumerate() {
        // Ensure clean environment for each test
        env::remove_var("CAMPFIRE_DEMO_MODE");
        let service = create_test_setup_service().await; // Fresh service for each test
        
        let request = CreateAdminRequest {
            email: email.to_string(),
            password: "validpass123".to_string(),
            name: format!("Admin {}", i),
        };
        
        let result = service.create_admin_account(request).await;
        assert!(result.is_ok(), "Email {} should be valid", email);
    }
}

#[tokio::test]
async fn test_requirement_11_2_password_strength() {
    // Requirement 11.2: Password strength requirements
    let service = create_test_setup_service().await;
    
    // Test weak passwords
    let long_password = "x".repeat(129);
    let weak_passwords = vec![
        "short",           // Too short
        "onlyletters",     // No numbers
        "12345678",        // No letters
        &long_password,    // Too long
    ];
    
    for password in weak_passwords {
        // Ensure clean environment for each test
        env::remove_var("CAMPFIRE_DEMO_MODE");
        let service = create_test_setup_service().await;
        
        let request = CreateAdminRequest {
            email: "admin@example.com".to_string(),
            password: password.to_string(),
            name: "Admin".to_string(),
        };
        
        let result = service.create_admin_account(request).await;
        assert!(matches!(result, Err(SetupError::WeakPassword { .. })));
    }
    
    // Test strong passwords
    let strong_passwords = vec![
        "password123",
        "MySecure1Pass",
        "Complex!Pass2024",
    ];
    
    for (i, password) in strong_passwords.iter().enumerate() {
        // Ensure clean environment for each test
        env::remove_var("CAMPFIRE_DEMO_MODE");
        let service = create_test_setup_service().await; // Fresh service for each test
        
        let request = CreateAdminRequest {
            email: format!("admin{}@example.com", i),
            password: password.to_string(),
            name: format!("Admin {}", i),
        };
        
        let result = service.create_admin_account(request).await;
        assert!(result.is_ok(), "Password {} should be valid", password);
    }
}

#[tokio::test]
async fn test_requirement_11_2_prevents_duplicate_admin() {
    // Requirement 11.2: Prevents creating admin when not first run
    
    // Ensure clean environment
    env::remove_var("CAMPFIRE_DEMO_MODE");
    
    let service = create_test_setup_service().await;
    
    // Create first admin
    let request1 = CreateAdminRequest {
        email: "admin1@example.com".to_string(),
        password: "securepass123".to_string(),
        name: "First Admin".to_string(),
    };
    
    let result1 = service.create_admin_account(request1).await;
    assert!(result1.is_ok());
    
    // Try to create second admin - should fail
    let request2 = CreateAdminRequest {
        email: "admin2@example.com".to_string(),
        password: "securepass456".to_string(),
        name: "Second Admin".to_string(),
    };
    
    let result2 = service.create_admin_account(request2).await;
    assert!(matches!(result2, Err(SetupError::NotFirstRun)));
}

#[tokio::test]
async fn test_requirement_11_3_deployment_configuration() {
    // Requirement 11.3: Environment-based deployment configuration
    let service = create_test_setup_service().await;
    
    // Set some environment variables
    env::set_var("CAMPFIRE_DATABASE_URL", "test.db");
    env::set_var("CAMPFIRE_VAPID_PUBLIC_KEY", "test_public_key");
    env::set_var("CAMPFIRE_VAPID_PRIVATE_KEY", "test_private_key");
    env::set_var("CAMPFIRE_SSL_DOMAIN", "campfire.example.com");
    env::set_var("CAMPFIRE_SESSION_EXPIRY_HOURS", "48");
    env::set_var("CAMPFIRE_MAX_MESSAGE_LENGTH", "5000");
    env::set_var("CAMPFIRE_ENABLE_REGISTRATION", "true");
    
    let config = service.get_deployment_config().await.unwrap();
    
    assert_eq!(config.database_url, "test.db");
    assert_eq!(config.vapid_public_key, Some("test_public_key".to_string()));
    assert_eq!(config.vapid_private_key, Some("test_private_key".to_string()));
    assert_eq!(config.ssl_domain, Some("campfire.example.com".to_string()));
    assert_eq!(config.session_timeout_hours, 48);
    assert_eq!(config.max_message_length, 5000);
    assert!(config.enable_user_registration);
    
    // Clean up
    env::remove_var("CAMPFIRE_DATABASE_URL");
    env::remove_var("CAMPFIRE_VAPID_PUBLIC_KEY");
    env::remove_var("CAMPFIRE_VAPID_PRIVATE_KEY");
    env::remove_var("CAMPFIRE_SSL_DOMAIN");
    env::remove_var("CAMPFIRE_SESSION_EXPIRY_HOURS");
    env::remove_var("CAMPFIRE_MAX_MESSAGE_LENGTH");
    env::remove_var("CAMPFIRE_ENABLE_REGISTRATION");
}

#[tokio::test]
async fn test_requirement_11_3_invalid_configuration() {
    // Requirement 11.3: Handles invalid configuration gracefully
    let service = create_test_setup_service().await;
    
    // Set invalid environment variable
    env::set_var("CAMPFIRE_SESSION_EXPIRY_HOURS", "invalid_number");
    
    let result = service.get_deployment_config().await;
    assert!(matches!(result, Err(SetupError::InvalidConfiguration { .. })));
    
    // Clean up
    env::remove_var("CAMPFIRE_SESSION_EXPIRY_HOURS");
}

#[tokio::test]
async fn test_requirement_11_4_system_health_validation() {
    // Requirement 11.4: Validates system readiness for production
    let service = create_test_setup_service().await;
    
    let health = service.validate_system_health().await.unwrap();
    
    // Verify all health checks
    assert!(health.database_connected);
    assert!(health.fts_search_available);
    assert!(health.websocket_ready);
    assert!(health.static_assets_embedded);
    
    // Admin account should not exist initially
    assert!(!health.admin_account_exists);
    
    // Create admin account
    let request = CreateAdminRequest {
        email: "admin@example.com".to_string(),
        password: "securepass123".to_string(),
        name: "System Admin".to_string(),
    };
    
    service.create_admin_account(request).await.unwrap();
    
    // Health check should now show admin exists
    let health_after = service.validate_system_health().await.unwrap();
    assert!(health_after.admin_account_exists);
}

#[tokio::test]
async fn test_requirement_11_4_setup_status_response() {
    // Requirement 11.4: Provides complete setup status for UI
    let service = create_test_setup_service().await;
    
    // Initial status
    let status = service.get_setup_status().await.unwrap();
    assert!(status.is_first_run);
    assert!(!status.admin_exists);
    assert!(status.system_health.database_connected);
    
    // Create admin account
    let request = CreateAdminRequest {
        email: "admin@example.com".to_string(),
        password: "securepass123".to_string(),
        name: "System Admin".to_string(),
    };
    
    service.create_admin_account(request).await.unwrap();
    
    // Status after admin creation
    let status_after = service.get_setup_status().await.unwrap();
    assert!(!status_after.is_first_run);
    assert!(status_after.admin_exists);
    assert!(status_after.system_health.admin_account_exists);
}

#[tokio::test]
async fn test_session_token_security() {
    // Verify session tokens are cryptographically secure
    let service = create_test_setup_service().await;
    
    let request = CreateAdminRequest {
        email: "admin@example.com".to_string(),
        password: "securepass123".to_string(),
        name: "System Admin".to_string(),
    };
    
    let response = service.create_admin_account(request).await.unwrap();
    
    // Token should be 64 characters (32 bytes hex encoded)
    assert_eq!(response.session_token.len(), 64);
    
    // Token should only contain hex characters
    assert!(response.session_token.chars().all(|c| c.is_ascii_hexdigit()));
    
    // Token should not be predictable (create another and compare)
    let service2 = create_test_setup_service().await;
    let request2 = CreateAdminRequest {
        email: "admin2@example.com".to_string(),
        password: "securepass123".to_string(),
        name: "System Admin 2".to_string(),
    };
    
    let response2 = service2.create_admin_account(request2).await.unwrap();
    
    // Tokens should be different
    assert_ne!(response.session_token, response2.session_token);
}

#[tokio::test]
async fn test_password_hashing_security() {
    // Verify passwords are properly hashed
    
    // Ensure clean environment
    env::remove_var("CAMPFIRE_DEMO_MODE");
    
    let service = create_test_setup_service().await;
    
    let request = CreateAdminRequest {
        email: "admin@example.com".to_string(),
        password: "plaintextpassword123".to_string(),
        name: "System Admin".to_string(),
    };
    
    let response = service.create_admin_account(request).await.unwrap();
    
    // Password hash should not contain the original password
    assert!(!response.user.password_hash.contains("plaintextpassword123"));
    
    // Password hash should be bcrypt format (starts with $2b$)
    assert!(response.user.password_hash.starts_with("$2b$"));
    
    // Password hash should be reasonable length (bcrypt hashes are ~60 chars)
    assert!(response.user.password_hash.len() >= 50);
}