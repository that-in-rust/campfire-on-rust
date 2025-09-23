use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Type-safe ID wrappers using newtype pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PushSubscriptionId(pub Uuid);

// ID implementations
impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl RoomId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl PushSubscriptionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

// From/Into implementations for ergonomic conversions
impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<UserId> for Uuid {
    fn from(user_id: UserId) -> Self {
        user_id.0
    }
}

impl From<Uuid> for RoomId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<RoomId> for Uuid {
    fn from(room_id: RoomId) -> Self {
        room_id.0
    }
}

impl From<Uuid> for MessageId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<MessageId> for Uuid {
    fn from(message_id: MessageId) -> Self {
        message_id.0
    }
}

impl From<Uuid> for ConnectionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<ConnectionId> for Uuid {
    fn from(connection_id: ConnectionId) -> Self {
        connection_id.0
    }
}

impl From<Uuid> for PushSubscriptionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<PushSubscriptionId> for Uuid {
    fn from(subscription_id: PushSubscriptionId) -> Self {
        subscription_id.0
    }
}

// Display implementations for error messages
impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for RoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for PushSubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Core domain models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub bio: Option<String>,
    pub admin: bool,
    pub bot_token: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl User {
    /// Check if this user is a bot (has a bot_token)
    pub fn is_bot(&self) -> bool {
        self.bot_token.is_some()
    }
    
    /// Generate bot key for API authentication (user_id-bot_token)
    pub fn bot_key(&self) -> Option<String> {
        self.bot_token.as_ref().map(|token| format!("{}-{}", self.id.0, token))
    }
    
    /// Convert User to Bot if it's a bot user
    pub fn to_bot(&self) -> Option<Bot> {
        self.bot_token.as_ref().map(|token| Bot {
            id: self.id,
            name: self.name.clone(),
            bot_token: token.clone(),
            webhook_url: None, // Will be populated from webhook table
            created_at: self.created_at,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub topic: Option<String>,
    pub room_type: RoomType,
    pub created_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomType {
    Open,    // Anyone can join
    Closed,  // Invitation only
    Direct,  // Two-person direct message
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub room_id: RoomId,
    pub creator_id: UserId,
    pub content: String,
    pub client_message_id: Uuid,
    pub created_at: DateTime<Utc>,
    /// Rich text HTML content (if different from content)
    pub html_content: Option<String>,
    /// Extracted @mentions from the message
    pub mentions: Vec<String>,
    /// Sound commands triggered by this message
    pub sound_commands: Vec<String>,
}

impl Message {
    /// Create a new message with basic content
    pub fn new(
        room_id: RoomId,
        creator_id: UserId,
        content: String,
        client_message_id: Uuid,
    ) -> Self {
        Self {
            id: MessageId::new(),
            room_id,
            creator_id,
            content,
            client_message_id,
            created_at: Utc::now(),
            html_content: None,
            mentions: Vec::new(),
            sound_commands: Vec::new(),
        }
    }
    
    /// Create a message with rich text features
    pub fn with_rich_content(
        room_id: RoomId,
        creator_id: UserId,
        content: String,
        client_message_id: Uuid,
        html_content: Option<String>,
        mentions: Vec<String>,
        sound_commands: Vec<String>,
    ) -> Self {
        Self {
            id: MessageId::new(),
            room_id,
            creator_id,
            content,
            client_message_id,
            created_at: Utc::now(),
            html_content,
            mentions,
            sound_commands,
        }
    }
    
    /// Get the display content (HTML if available, otherwise plain text)
    pub fn display_content(&self) -> &str {
        self.html_content.as_deref().unwrap_or(&self.content)
    }
    
    /// Check if message has rich text features
    pub fn has_rich_features(&self) -> bool {
        self.html_content.is_some() 
            || !self.mentions.is_empty() 
            || !self.sound_commands.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Membership {
    pub room_id: RoomId,
    pub user_id: UserId,
    pub involvement_level: InvolvementLevel,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvolvementLevel {
    Member,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: User,
    pub session_token: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub content: String,
    pub client_message_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub topic: Option<String>,
    pub room_type: RoomType,
}

// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    NewMessage {
        message: Message,
    },
    UserJoined {
        user_id: UserId,
        room_id: RoomId,
    },
    UserLeft {
        user_id: UserId,
        room_id: RoomId,
    },
    TypingStart {
        user_id: UserId,
        room_id: RoomId,
    },
    TypingStop {
        user_id: UserId,
        room_id: RoomId,
    },
    PresenceUpdate {
        room_id: RoomId,
        online_users: Vec<UserId>,
    },
    SoundPlayback {
        sound_name: String,
        triggered_by: UserId,
        room_id: RoomId,
        timestamp: DateTime<Utc>,
    },
}

// Push notification models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub id: PushSubscriptionId,
    pub user_id: UserId,
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_key: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: UserId,
    pub mentions_enabled: bool,
    pub direct_messages_enabled: bool,
    pub all_messages_enabled: bool,
    pub sounds_enabled: bool,
    pub updated_at: DateTime<Utc>,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            user_id: UserId::new(), // This will be overridden
            mentions_enabled: true,
            direct_messages_enabled: true,
            all_messages_enabled: false,
            sounds_enabled: true,
            updated_at: Utc::now(),
        }
    }
}

// Push notification request/response DTOs
#[derive(Debug, Deserialize)]
pub struct CreatePushSubscriptionRequest {
    pub endpoint: String,
    pub keys: PushSubscriptionKeys,
}

#[derive(Debug, Deserialize)]
pub struct PushSubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotificationPreferencesRequest {
    pub mentions_enabled: Option<bool>,
    pub direct_messages_enabled: Option<bool>,
    pub all_messages_enabled: Option<bool>,
    pub sounds_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    NewMessage,
    Mention,
    DirectMessage,
    SoundPlayback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotificationPayload {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub tag: Option<String>,
    pub data: serde_json::Value,
}

// Bot-related models and DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bot {
    pub id: UserId,
    pub name: String,
    pub bot_token: String,
    pub webhook_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Bot {
    /// Generate bot key in format "user_id-bot_token" for API authentication
    pub fn bot_key(&self) -> String {
        format!("{}-{}", self.id.0, self.bot_token)
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBotRequest {
    pub name: Option<String>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BotMessageRequest {
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub user: WebhookUser,
    pub room: WebhookRoom,
    pub message: WebhookMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookUser {
    pub id: UserId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRoom {
    pub id: RoomId,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMessage {
    pub id: MessageId,
    pub body: WebhookMessageBody,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMessageBody {
    pub html: String,
    pub plain: String,
}