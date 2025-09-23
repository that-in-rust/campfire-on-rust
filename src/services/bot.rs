use async_trait::async_trait;
use chrono::Utc;
use rand::Rng;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info};

use crate::database::{Database, DatabaseWriter};
use crate::errors::BotError;
use crate::models::*;
use crate::services::MessageServiceTrait;

/// Bot service trait for bot management and webhook delivery
#[async_trait]
pub trait BotService: Send + Sync {
    /// Create a new bot user
    async fn create_bot(
        &self,
        name: String,
        webhook_url: Option<String>,
    ) -> Result<Bot, BotError>;
    
    /// Update an existing bot
    async fn update_bot(
        &self,
        bot_id: UserId,
        name: Option<String>,
        webhook_url: Option<String>,
    ) -> Result<Bot, BotError>;
    
    /// Delete a bot (deactivate)
    async fn delete_bot(&self, bot_id: UserId) -> Result<(), BotError>;
    
    /// Get bot by ID
    async fn get_bot(&self, bot_id: UserId) -> Result<Option<Bot>, BotError>;
    
    /// List all active bots
    async fn list_bots(&self) -> Result<Vec<Bot>, BotError>;
    
    /// Authenticate bot using bot key (user_id-bot_token format)
    async fn authenticate_bot(&self, bot_key: &str) -> Result<User, BotError>;
    
    /// Reset bot token (generate new one)
    async fn reset_bot_token(&self, bot_id: UserId) -> Result<String, BotError>;
    
    /// Deliver webhook notification for a message
    async fn deliver_webhook(&self, bot: &Bot, message: &Message, room: &Room) -> Result<(), BotError>;
    
    /// Create a message from bot
    async fn create_bot_message(
        &self,
        bot_id: UserId,
        room_id: RoomId,
        content: String,
    ) -> Result<Message, BotError>;
}

/// Bot service implementation
pub struct BotServiceImpl {
    database: Arc<crate::CampfireDatabase>,
    database_writer: Arc<dyn DatabaseWriter>,
    http_client: Client,
    message_service: Arc<dyn MessageServiceTrait>,
}

impl BotServiceImpl {
    pub fn new(
        database: Arc<crate::CampfireDatabase>,
        database_writer: Arc<dyn DatabaseWriter>,
        message_service: Arc<dyn MessageServiceTrait>,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(7)) // Match Rails ENDPOINT_TIMEOUT
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            database,
            database_writer,
            http_client,
            message_service,
        }
    }
    
    /// Generate a secure bot token (12 alphanumeric characters like Rails)
    fn generate_bot_token() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        
        (0..12)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
    
    /// Parse bot key into user_id and bot_token
    fn parse_bot_key(bot_key: &str) -> Result<(UserId, String), BotError> {
        // Find the last hyphen to separate UUID from token
        // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx (36 chars)
        // So we expect the UUID part to be 36 characters
        if bot_key.len() < 37 { // 36 for UUID + 1 for hyphen + at least 1 for token
            return Err(BotError::InvalidToken);
        }
        
        let uuid_part = &bot_key[0..36];
        let remaining = &bot_key[36..];
        
        if !remaining.starts_with('-') {
            return Err(BotError::InvalidToken);
        }
        
        let bot_token = &remaining[1..]; // Skip the hyphen
        
        let user_id = uuid::Uuid::parse_str(uuid_part)
            .map_err(|_| BotError::InvalidToken)?;
        
        Ok((UserId(user_id), bot_token.to_string()))
    }
    
    /// Validate webhook URL
    fn validate_webhook_url(url: &str) -> Result<(), BotError> {
        if url.is_empty() {
            return Ok(());
        }
        
        let parsed = url::Url::parse(url)
            .map_err(|_| BotError::InvalidWebhookUrl { url: url.to_string() })?;
            
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(BotError::InvalidWebhookUrl { url: url.to_string() });
        }
        
        Ok(())
    }
    
    /// Create webhook payload for message
    fn create_webhook_payload(
        &self,
        message: &Message,
        room: &Room,
        user: &User,
        bot: &Bot,
    ) -> WebhookPayload {
        // Create paths (simplified for MVP - no full URL generation)
        let message_path = format!("/rooms/{}/messages/{}", room.id.0, message.id.0);
        let room_bot_path = format!("/rooms/{}/bot/{}/messages", room.id.0, bot.bot_key());
        
        // Remove bot mentions from plain text (simplified)
        let plain_text = message.content.clone(); // TODO: Implement mention removal
        
        WebhookPayload {
            user: WebhookUser {
                id: user.id,
                name: user.name.clone(),
            },
            room: WebhookRoom {
                id: room.id,
                name: room.name.clone(),
                path: room_bot_path,
            },
            message: WebhookMessage {
                id: message.id,
                body: WebhookMessageBody {
                    html: message.display_content().to_string(),
                    plain: plain_text,
                },
                path: message_path,
            },
        }
    }
}

#[async_trait]
impl BotService for BotServiceImpl {
    async fn create_bot(
        &self,
        name: String,
        webhook_url: Option<String>,
    ) -> Result<Bot, BotError> {
        // Validate inputs
        if name.trim().is_empty() || name.len() > 50 {
            return Err(BotError::InvalidName { 
                reason: "Name must be between 1 and 50 characters".to_string() 
            });
        }
        
        if let Some(ref url) = webhook_url {
            Self::validate_webhook_url(url)?;
        }
        
        // Generate bot token
        let bot_token = Self::generate_bot_token();
        
        // Create bot user
        let bot_user = User {
            id: UserId::new(),
            name: name.clone(),
            email: format!("bot-{}@campfire.local", bot_token), // Unique email for bot
            password_hash: String::new(), // Bots don't have passwords
            bio: Some("Bot user".to_string()),
            admin: false,
            bot_token: Some(bot_token.clone()),
            created_at: Utc::now(),
        };
        
        // Save to database
        self.database_writer.create_user(bot_user.clone()).await?;
        
        // Create webhook if URL provided
        if let Some(webhook_url) = webhook_url.as_ref() {
            self.create_webhook_internal(bot_user.id, webhook_url).await?;
        }
        
        info!("Created bot: {} ({})", name, bot_user.id);
        
        Ok(Bot {
            id: bot_user.id,
            name,
            bot_token,
            webhook_url,
            created_at: bot_user.created_at,
        })
    }
    
    async fn update_bot(
        &self,
        bot_id: UserId,
        name: Option<String>,
        webhook_url: Option<String>,
    ) -> Result<Bot, BotError> {
        // Get existing bot
        let mut bot = self.get_bot(bot_id).await?
            .ok_or(BotError::NotFound { bot_id })?;
        
        // Validate inputs
        if let Some(ref new_name) = name {
            if new_name.trim().is_empty() || new_name.len() > 50 {
                return Err(BotError::InvalidName { 
                    reason: "Name must be between 1 and 50 characters".to_string() 
                });
            }
        }
        
        if let Some(ref url) = webhook_url {
            Self::validate_webhook_url(url)?;
        }
        
        // Update fields
        if let Some(new_name) = name {
            bot.name = new_name;
            // TODO: Update user name in database
        }
        
        if let Some(new_webhook_url) = webhook_url {
            bot.webhook_url = if new_webhook_url.is_empty() {
                None
            } else {
                Some(new_webhook_url)
            };
            // TODO: Update webhook in database
        }
        
        info!("Updated bot: {} ({})", bot.name, bot.id);
        Ok(bot)
    }
    
    async fn delete_bot(&self, bot_id: UserId) -> Result<(), BotError> {
        // Verify bot exists
        let _bot = self.get_bot(bot_id).await?
            .ok_or(BotError::NotFound { bot_id })?;
        
        // TODO: Implement bot deactivation (set bot_token to NULL)
        // For now, we'll just log the action
        info!("Deactivated bot: {}", bot_id);
        Ok(())
    }
    
    async fn get_bot(&self, bot_id: UserId) -> Result<Option<Bot>, BotError> {
        let user = self.database.get_user_by_id(bot_id).await?;
        
        if let Some(user) = user {
            if let Some(bot) = user.to_bot() {
                // Get webhook URL
                let webhook_url = self.get_webhook_url_internal(bot_id).await?;
                
                Ok(Some(Bot {
                    webhook_url,
                    ..bot
                }))
            } else {
                Err(BotError::NotABot { user_id: bot_id })
            }
        } else {
            Ok(None)
        }
    }
    
    async fn list_bots(&self) -> Result<Vec<Bot>, BotError> {
        // TODO: Implement database query for all bot users
        // For now, return empty list
        Ok(Vec::new())
    }
    
    async fn authenticate_bot(&self, bot_key: &str) -> Result<User, BotError> {
        let (user_id, bot_token) = Self::parse_bot_key(bot_key)?;
        
        let user = self.database.get_user_by_id(user_id).await?
            .ok_or(BotError::InvalidToken)?;
        
        // Verify bot token matches
        match &user.bot_token {
            Some(stored_token) if stored_token == &bot_token => Ok(user),
            _ => Err(BotError::InvalidToken),
        }
    }
    
    async fn reset_bot_token(&self, bot_id: UserId) -> Result<String, BotError> {
        // Verify bot exists
        let _bot = self.get_bot(bot_id).await?
            .ok_or(BotError::NotFound { bot_id })?;
        
        // Generate new token
        let new_token = Self::generate_bot_token();
        
        // TODO: Update bot_token in database
        info!("Reset bot token for: {}", bot_id);
        
        Ok(new_token)
    }
    
    async fn deliver_webhook(
        &self,
        bot: &Bot,
        message: &Message,
        room: &Room,
    ) -> Result<(), BotError> {
        let webhook_url = match &bot.webhook_url {
            Some(url) => url,
            None => {
                // No webhook configured, skip delivery
                return Ok(());
            }
        };
        
        // Get message creator
        let creator = self.database.get_user_by_id(message.creator_id).await?
            .ok_or(BotError::Database(crate::errors::DatabaseError::DataIntegrity { 
                reason: "Message creator not found".to_string() 
            }))?;
        
        // Create webhook payload
        let payload = self.create_webhook_payload(message, room, &creator, bot);
        
        // Send webhook with timeout
        let webhook_future = self.http_client
            .post(webhook_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send();
        
        match timeout(Duration::from_secs(7), webhook_future).await {
            Ok(Ok(response)) => {
                info!("Webhook delivered to bot {} ({}): {}", bot.name, bot.id, response.status());
                
                // Handle webhook response (simplified - no reply processing for MVP)
                if response.status().is_success() {
                    if let Ok(response_text) = response.text().await {
                        if !response_text.trim().is_empty() {
                            // TODO: Create reply message from bot
                            info!("Bot {} replied: {}", bot.name, response_text.chars().take(100).collect::<String>());
                        }
                    }
                }
                
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Webhook delivery failed for bot {} ({}): {}", bot.name, bot.id, e);
                Err(BotError::WebhookDeliveryFailed { 
                    reason: e.to_string() 
                })
            }
            Err(_) => {
                error!("Webhook timeout for bot {} ({})", bot.name, bot.id);
                Err(BotError::WebhookTimeout { timeout_seconds: 7 })
            }
        }
    }
    
    async fn create_bot_message(
        &self,
        bot_id: UserId,
        room_id: RoomId,
        content: String,
    ) -> Result<Message, BotError> {
        // Verify bot exists and is actually a bot
        let bot_user = self.database.get_user_by_id(bot_id).await?
            .ok_or(BotError::NotFound { bot_id })?;
        
        if !bot_user.is_bot() {
            return Err(BotError::NotABot { user_id: bot_id });
        }
        
        // Create message using message service
        let client_message_id = uuid::Uuid::new_v4();
        
        match self.message_service.create_message_with_deduplication(
            content,
            room_id,
            bot_id,
            client_message_id,
        ).await {
            Ok(message) => {
                info!("Bot {} created message in room {}", bot_id, room_id);
                Ok(message)
            }
            Err(e) => {
                error!("Failed to create bot message: {}", e);
                Err(BotError::Database(crate::errors::DatabaseError::DataIntegrity { 
                    reason: e.to_string() 
                }))
            }
        }
    }
}

// Internal helper methods
impl BotServiceImpl {
    async fn create_webhook_internal(&self, _bot_id: UserId, _webhook_url: &str) -> Result<(), BotError> {
        // TODO: Implement webhook table creation
        Ok(())
    }
    
    async fn get_webhook_url_internal(&self, _bot_id: UserId) -> Result<Option<String>, BotError> {
        // TODO: Implement webhook URL retrieval from database
        Ok(None)
    }
}