use campfire_on_rust::*;
use std::sync::Arc;

#[tokio::test]
async fn test_enhanced_demo_mode_detection() {
    // Test environment variable detection
    std::env::set_var("CAMPFIRE_DEMO_MODE", "true");
    
    // Create test database
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let db_arc = Arc::new(db.clone());
    
    // Create services
    let auth_service = Arc::new(AuthService::new(db_arc.clone()));
    let room_service = Arc::new(RoomService::new(db_arc.clone()));
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db_arc.clone()));
    let push_service = Arc::new(PushNotificationServiceImpl::new(
        db.clone(),
        db.writer(),
        VapidConfig::default(),
    ));
    let message_service = Arc::new(MessageService::with_push_service(
        db_arc.clone(),
        connection_manager,
        room_service.clone(),
        push_service.clone(),
    ));
    let search_service = Arc::new(SearchService::new(db_arc.clone(), room_service.clone()));
    let bot_service = Arc::new(BotServiceImpl::new(db_arc.clone(), db.writer(), message_service.clone()));
    let setup_service = Arc::new(SetupServiceImpl::new(db.clone()));
    
    let demo_service = Arc::new(services::DemoServiceImpl::new(db_arc.clone()));
    
    let analytics_store = Arc::new(campfire_on_rust::analytics::AnalyticsStore::new());
    
    let app_state = AppState {
        db,
        auth_service,
        room_service,
        message_service,
        search_service,
        push_service,
        bot_service,
        setup_service,
        demo_service,
        analytics_store,
    };
    
    // Initialize demo data
    let demo_initializer = demo::DemoDataInitializer::new(Arc::new(app_state.db.clone()));
    demo_initializer.initialize_if_needed().await.unwrap();
    
    // Test demo mode detection
    let demo_mode = app_state.db.get_user_by_email("admin@campfire.demo").await.unwrap().is_some();
    assert!(demo_mode, "Demo mode should be detected after initialization");
    
    // Test demo user count
    let admin_user = app_state.db.get_user_by_email("admin@campfire.demo").await.unwrap();
    assert!(admin_user.is_some(), "Admin demo user should exist");
    assert!(admin_user.unwrap().admin, "Admin user should have admin privileges");
    
    // Test demo room creation
    let alice_user = app_state.db.get_user_by_email("alice@campfire.demo").await.unwrap();
    assert!(alice_user.is_some(), "Alice demo user should exist");
    
    // Clean up environment
    std::env::remove_var("CAMPFIRE_DEMO_MODE");
}

#[tokio::test]
async fn test_demo_credentials_api_structure() {
    // Test that demo credentials have the expected structure
    let demo_credentials = demo::DemoDataInitializer::get_demo_credentials();
    
    assert_eq!(demo_credentials.len(), 8, "Should have 8 demo users");
    
    // Test admin user
    let admin_cred = demo_credentials.iter().find(|(email, _, _)| email.contains("admin")).unwrap();
    assert_eq!(admin_cred.0, "admin@campfire.demo");
    assert_eq!(admin_cred.1, "password");
    assert_eq!(admin_cred.2, "System Administrator");
    
    // Test other users exist
    let user_emails: Vec<&str> = demo_credentials.iter().map(|(email, _, _)| *email).collect();
    assert!(user_emails.contains(&"alice@campfire.demo"));
    assert!(user_emails.contains(&"bob@campfire.demo"));
    assert!(user_emails.contains(&"carol@campfire.demo"));
}

#[test]
fn test_environment_variable_demo_detection() {
    // Test explicit true
    std::env::set_var("CAMPFIRE_DEMO_MODE", "true");
    let demo_env = std::env::var("CAMPFIRE_DEMO_MODE").unwrap_or_else(|_| "auto".to_string());
    assert_eq!(demo_env.to_lowercase(), "true");
    
    // Test explicit false
    std::env::set_var("CAMPFIRE_DEMO_MODE", "false");
    let demo_env = std::env::var("CAMPFIRE_DEMO_MODE").unwrap_or_else(|_| "auto".to_string());
    assert_eq!(demo_env.to_lowercase(), "false");
    
    // Test auto-detection (no env var)
    std::env::remove_var("CAMPFIRE_DEMO_MODE");
    let demo_env = std::env::var("CAMPFIRE_DEMO_MODE").unwrap_or_else(|_| "auto".to_string());
    assert_eq!(demo_env, "auto");
    
    // Clean up
    std::env::remove_var("CAMPFIRE_DEMO_MODE");
}

#[tokio::test]
async fn test_multi_user_demo_simulation() {
    // Test Requirement 10.5: Multi-user simulation capability
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let demo_service = services::DemoServiceImpl::new(Arc::new(db.clone()));
    
    // Initialize demo data
    demo_service.ensure_demo_data().await.unwrap();
    
    // Test demo credentials retrieval
    let credentials = demo_service.get_demo_credentials().await.unwrap();
    assert_eq!(credentials.len(), 8);
    
    // Verify each credential has required fields
    for credential in &credentials {
        assert!(!credential.email.is_empty());
        assert_eq!(credential.password, "password");
        assert!(!credential.name.is_empty());
        assert!(!credential.role.is_empty());
        assert!(!credential.avatar.is_empty());
        assert!(!credential.demo_context.is_empty());
        assert!(!credential.tour_highlights.is_empty());
    }
    
    // Test multi-user simulation sessions
    let session1 = demo_service
        .start_simulation_session("alice@campfire.demo", "tab-1")
        .await
        .unwrap();
    
    let session2 = demo_service
        .start_simulation_session("bob@campfire.demo", "tab-2")
        .await
        .unwrap();
    
    let session3 = demo_service
        .start_simulation_session("carol@campfire.demo", "tab-3")
        .await
        .unwrap();
    
    // Verify sessions are tracked
    let active_sessions = demo_service.get_active_sessions().await.unwrap();
    assert_eq!(active_sessions.len(), 3);
    
    // Verify session details
    assert_eq!(session1.user_email, "alice@campfire.demo");
    assert_eq!(session1.browser_tab_id, "tab-1");
    assert_eq!(session2.user_email, "bob@campfire.demo");
    assert_eq!(session2.browser_tab_id, "tab-2");
    assert_eq!(session3.user_email, "carol@campfire.demo");
    assert_eq!(session3.browser_tab_id, "tab-3");
}

#[tokio::test]
async fn test_guided_tour_functionality() {
    // Test Requirement 10.4: Guided tour and feature highlighting
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let demo_service = services::DemoServiceImpl::new(Arc::new(db.clone()));
    
    // Test tour steps for different roles
    let admin_steps = demo_service.get_tour_steps("System Administrator").await.unwrap();
    let pm_steps = demo_service.get_tour_steps("Product Manager").await.unwrap();
    let dev_steps = demo_service.get_tour_steps("Senior Developer").await.unwrap();
    
    // Verify common steps exist for all roles
    assert!(admin_steps.iter().any(|s| s.step_id == "welcome"));
    assert!(admin_steps.iter().any(|s| s.step_id == "rooms_sidebar"));
    assert!(admin_steps.iter().any(|s| s.step_id == "message_input"));
    assert!(admin_steps.iter().any(|s| s.step_id == "search_feature"));
    
    // Verify role-specific steps
    assert!(admin_steps.iter().any(|s| s.step_id == "admin_features"));
    assert!(pm_steps.iter().any(|s| s.step_id == "product_rooms"));
    assert!(dev_steps.iter().any(|s| s.step_id == "dev_features"));
    
    // Test tour step completion
    demo_service.ensure_demo_data().await.unwrap();
    let session = demo_service
        .start_simulation_session("alice@campfire.demo", "tour-tab")
        .await
        .unwrap();
    
    // Complete a tour step
    demo_service
        .complete_tour_step(&session.session_id, "welcome")
        .await
        .unwrap();
    
    // Update session with feature exploration
    demo_service
        .update_session_activity(&session.session_id, vec!["tour_step_welcome".to_string()])
        .await
        .unwrap();
    
    // Verify session was updated
    let updated_sessions = demo_service.get_active_sessions().await.unwrap();
    let updated_session = updated_sessions.iter().find(|s| s.session_id == session.session_id).unwrap();
    assert!(updated_session.features_explored.contains(&"tour_step_welcome".to_string()));
}

#[tokio::test]
async fn test_demo_data_integrity_validation() {
    // Test Requirement 10.6: Demo data integrity checking and validation
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let demo_service = services::DemoServiceImpl::new(Arc::new(db.clone()));
    
    // Initially should have no demo data
    let initial_integrity = demo_service.check_demo_integrity().await.unwrap();
    assert_eq!(initial_integrity.actual_users, 0);
    assert_eq!(initial_integrity.integrity_score, 0.0);
    assert!(!initial_integrity.users_exist);
    assert!(!initial_integrity.rooms_exist);
    assert!(!initial_integrity.messages_exist);
    assert!(!initial_integrity.bots_configured);
    assert!(!initial_integrity.missing_components.is_empty());
    
    // Initialize demo data
    demo_service.ensure_demo_data().await.unwrap();
    
    // Check integrity after initialization
    let post_init_integrity = demo_service.check_demo_integrity().await.unwrap();
    assert_eq!(post_init_integrity.actual_users, 8); // 8 demo users
    assert_eq!(post_init_integrity.integrity_score, 1.0); // Perfect score
    assert!(post_init_integrity.users_exist);
    assert!(post_init_integrity.rooms_exist);
    assert!(post_init_integrity.messages_exist);
    assert!(post_init_integrity.bots_configured);
    assert!(post_init_integrity.missing_components.is_empty());
    
    // Test statistics
    let stats = demo_service.get_demo_statistics().await.unwrap();
    assert_eq!(stats.total_users, 8);
    assert_eq!(stats.total_rooms, 7);
    assert!(stats.total_messages > 0);
    assert_eq!(stats.active_sessions, 0); // No active sessions yet
    assert_eq!(stats.tours_completed, 0);
}

#[tokio::test]
async fn test_demo_credential_management() {
    // Test Requirement 10.3: Demo user credential management for one-click login
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let demo_service = services::DemoServiceImpl::new(Arc::new(db.clone()));
    
    let credentials = demo_service.get_demo_credentials().await.unwrap();
    
    // Verify we have all expected demo users
    let expected_emails = [
        "admin@campfire.demo",
        "alice@campfire.demo",
        "bob@campfire.demo",
        "carol@campfire.demo",
        "david@campfire.demo",
        "eve@campfire.demo",
        "frank@campfire.demo",
        "grace@campfire.demo",
    ];
    
    for email in &expected_emails {
        assert!(credentials.iter().any(|c| c.email == *email));
    }
    
    // Verify admin user has admin permissions
    let admin_cred = credentials.iter().find(|c| c.email == "admin@campfire.demo").unwrap();
    assert!(admin_cred.permissions.contains(&"admin".to_string()));
    assert!(admin_cred.permissions.contains(&"manage_users".to_string()));
    assert!(admin_cred.permissions.contains(&"manage_rooms".to_string()));
    
    // Verify role-specific contexts
    let alice_cred = credentials.iter().find(|c| c.email == "alice@campfire.demo").unwrap();
    assert_eq!(alice_cred.role, "Product Manager");
    assert!(alice_cred.demo_context.contains("product strategy"));
    
    let bob_cred = credentials.iter().find(|c| c.email == "bob@campfire.demo").unwrap();
    assert_eq!(bob_cred.role, "Senior Developer");
    assert!(bob_cred.demo_context.contains("Technical team lead"));
    
    // Verify tour highlights are provided
    for credential in &credentials {
        assert!(!credential.tour_highlights.is_empty());
    }
}