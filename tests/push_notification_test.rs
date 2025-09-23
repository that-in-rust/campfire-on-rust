use campfire_on_rust::{
    CampfireDatabase, PushNotificationServiceImpl, VapidConfig,
    models::*,
};
use std::sync::Arc;

#[tokio::test]
async fn test_push_notification_service_creation() {
    // Create test database
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let writer = db.writer();
    
    // Create VAPID config
    let vapid_config = VapidConfig::default();
    
    // Create push notification service
    let push_service = PushNotificationServiceImpl::new(
        db.clone(),
        writer,
        vapid_config,
    );
    
    // Test creating notification preferences
    let user_id = UserId::new();
    let preferences = NotificationPreferences {
        user_id,
        mentions_enabled: true,
        direct_messages_enabled: true,
        all_messages_enabled: false,
        sounds_enabled: true,
        updated_at: chrono::Utc::now(),
    };
    
    // This should work without errors
    let result = push_service.update_preferences(
        user_id,
        UpdateNotificationPreferencesRequest {
            mentions_enabled: Some(true),
            direct_messages_enabled: Some(true),
            all_messages_enabled: Some(false),
            sounds_enabled: Some(true),
        },
    ).await;
    
    assert!(result.is_ok());
    let updated_preferences = result.unwrap();
    assert_eq!(updated_preferences.user_id, user_id);
    assert!(updated_preferences.mentions_enabled);
    assert!(updated_preferences.direct_messages_enabled);
    assert!(!updated_preferences.all_messages_enabled);
    assert!(updated_preferences.sounds_enabled);
}

#[tokio::test]
async fn test_push_subscription_creation() {
    // Create test database
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let writer = db.writer();
    
    // Create VAPID config
    let vapid_config = VapidConfig::default();
    
    // Create push notification service
    let push_service = PushNotificationServiceImpl::new(
        db.clone(),
        writer,
        vapid_config,
    );
    
    let user_id = UserId::new();
    let request = CreatePushSubscriptionRequest {
        endpoint: "https://fcm.googleapis.com/fcm/send/test".to_string(),
        keys: PushSubscriptionKeys {
            p256dh: "test_p256dh_key".to_string(),
            auth: "test_auth_key".to_string(),
        },
    };
    
    let result = push_service.create_subscription(user_id, request).await;
    assert!(result.is_ok());
    
    let subscription = result.unwrap();
    assert_eq!(subscription.user_id, user_id);
    assert_eq!(subscription.endpoint, "https://fcm.googleapis.com/fcm/send/test");
    assert_eq!(subscription.p256dh_key, "test_p256dh_key");
    assert_eq!(subscription.auth_key, "test_auth_key");
}

#[tokio::test]
async fn test_notification_preferences_defaults() {
    // Create test database
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let writer = db.writer();
    
    // Create VAPID config
    let vapid_config = VapidConfig::default();
    
    // Create push notification service
    let push_service = PushNotificationServiceImpl::new(
        db.clone(),
        writer,
        vapid_config,
    );
    
    let user_id = UserId::new();
    
    // Get preferences for a user that doesn't exist yet - should return defaults
    let result = push_service.get_preferences(user_id).await;
    assert!(result.is_ok());
    
    let preferences = result.unwrap();
    assert_eq!(preferences.user_id, user_id);
    // Check default values
    assert!(preferences.mentions_enabled);
    assert!(preferences.direct_messages_enabled);
    assert!(!preferences.all_messages_enabled);
    assert!(preferences.sounds_enabled);
}