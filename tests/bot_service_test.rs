use campfire_on_rust::{
    BotServiceImpl, BotService, CampfireDatabase, MessageService, 
    models::*,
    errors::BotError,
};
use std::sync::Arc;

async fn create_test_bot_service() -> BotServiceImpl {
    let db = CampfireDatabase::new(":memory:").await.unwrap();
    let db_arc = Arc::new(db.clone());
    
    // Create required services
    let connection_manager = Arc::new(campfire_on_rust::ConnectionManagerImpl::new(db_arc.clone()));
    let room_service = Arc::new(campfire_on_rust::RoomService::new(db_arc.clone()));
    let message_service = Arc::new(MessageService::new(
        db_arc.clone(),
        connection_manager,
        room_service,
    ));
    
    BotServiceImpl::new(
        db_arc.clone(),
        db.writer(),
        message_service,
    )
}

#[tokio::test]
async fn test_create_bot() {
    let bot_service = create_test_bot_service().await;
    
    // Test creating a bot
    let bot = bot_service.create_bot(
        "Test Bot".to_string(),
        Some("https://example.com/webhook".to_string()),
    ).await.unwrap();
    
    assert_eq!(bot.name, "Test Bot");
    assert_eq!(bot.webhook_url, Some("https://example.com/webhook".to_string()));
    assert!(!bot.bot_token.is_empty());
    assert_eq!(bot.bot_token.len(), 12); // Should be 12 characters
}

#[tokio::test]
async fn test_create_bot_invalid_name() {
    let bot_service = create_test_bot_service().await;
    
    // Test creating a bot with empty name
    let result = bot_service.create_bot(
        "".to_string(),
        None,
    ).await;
    
    assert!(matches!(result, Err(BotError::InvalidName { .. })));
}

#[tokio::test]
async fn test_create_bot_invalid_webhook_url() {
    let bot_service = create_test_bot_service().await;
    
    // Test creating a bot with invalid webhook URL
    let result = bot_service.create_bot(
        "Test Bot".to_string(),
        Some("not-a-url".to_string()),
    ).await;
    
    assert!(matches!(result, Err(BotError::InvalidWebhookUrl { .. })));
}

#[tokio::test]
async fn test_authenticate_bot() {
    let bot_service = create_test_bot_service().await;
    
    // Create a bot
    let bot = bot_service.create_bot(
        "Test Bot".to_string(),
        None,
    ).await.unwrap();
    
    // Test authentication with correct bot key
    let bot_key = bot.bot_key();
    
    let authenticated_user = bot_service.authenticate_bot(&bot_key).await.unwrap();
    
    assert_eq!(authenticated_user.id, bot.id);
    assert_eq!(authenticated_user.name, bot.name);
    assert!(authenticated_user.is_bot());
}

#[tokio::test]
async fn test_authenticate_bot_invalid_key() {
    let bot_service = create_test_bot_service().await;
    
    // Test authentication with invalid bot key format
    let result = bot_service.authenticate_bot("invalid-key").await;
    assert!(matches!(result, Err(BotError::InvalidToken)));
    
    // Test authentication with non-existent bot
    let fake_uuid = uuid::Uuid::new_v4();
    let fake_key = format!("{}-faketoken123", fake_uuid);
    let result = bot_service.authenticate_bot(&fake_key).await;
    assert!(matches!(result, Err(BotError::InvalidToken)));
}