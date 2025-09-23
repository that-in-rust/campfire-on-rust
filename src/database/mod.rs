use sqlx::{sqlite::SqlitePool, Row, Sqlite, Transaction};
use anyhow::Result;
use crate::errors::DatabaseError;
use crate::models::*;
use tokio::sync::{mpsc, oneshot};
use std::sync::Arc;
use async_trait::async_trait;

/// Database Writer Pattern (Critical Gap #3)
/// 
/// All write operations are serialized through a single writer task
/// to prevent SQLite write conflicts and ensure data consistency.
#[async_trait]
pub trait DatabaseWriter: Send + Sync {
    /// Create a new user
    async fn create_user(&self, user: User) -> Result<(), DatabaseError>;
    
    /// Create a new session
    async fn create_session(&self, session: Session) -> Result<(), DatabaseError>;
    
    /// Delete a session
    async fn delete_session(&self, token: String) -> Result<(), DatabaseError>;
    
    /// Create a message with deduplication
    async fn create_message_with_deduplication(&self, message: Message) -> Result<Message, DatabaseError>;
    
    /// Create a new room
    async fn create_room(&self, room: Room) -> Result<(), DatabaseError>;
    
    /// Create a room membership
    async fn create_membership(&self, membership: Membership) -> Result<(), DatabaseError>;
    
    /// Create a push subscription
    async fn create_push_subscription(&self, subscription: PushSubscription) -> Result<(), DatabaseError>;
    
    /// Update notification preferences
    async fn update_notification_preferences(&self, preferences: NotificationPreferences) -> Result<(), DatabaseError>;
}

/// Write operations that can be sent to the writer task
#[derive(Debug)]
pub enum WriteOperation {
    CreateUser {
        user: User,
        respond_to: oneshot::Sender<Result<(), DatabaseError>>,
    },
    CreateSession {
        session: Session,
        respond_to: oneshot::Sender<Result<(), DatabaseError>>,
    },
    DeleteSession {
        token: String,
        respond_to: oneshot::Sender<Result<(), DatabaseError>>,
    },
    CreateMessageWithDeduplication {
        message: Message,
        respond_to: oneshot::Sender<Result<Message, DatabaseError>>,
    },
    CreateRoom {
        room: Room,
        respond_to: oneshot::Sender<Result<(), DatabaseError>>,
    },
    CreateMembership {
        membership: Membership,
        respond_to: oneshot::Sender<Result<(), DatabaseError>>,
    },
    CreatePushSubscription {
        subscription: PushSubscription,
        respond_to: oneshot::Sender<Result<(), DatabaseError>>,
    },
    UpdateNotificationPreferences {
        preferences: NotificationPreferences,
        respond_to: oneshot::Sender<Result<(), DatabaseError>>,
    },
}

/// Database writer implementation that serializes all writes
pub struct SerializedDatabaseWriter {
    write_sender: mpsc::Sender<WriteOperation>,
}

impl SerializedDatabaseWriter {
    /// Create a new serialized database writer with background task
    pub fn new(database: Database) -> Self {
        let (write_sender, write_receiver) = mpsc::channel::<WriteOperation>(1000);
        
        // Spawn the writer task
        tokio::spawn(Self::writer_task(database, write_receiver));
        
        Self { write_sender }
    }
    
    /// Background task that processes all write operations serially
    async fn writer_task(
        database: Database,
        mut write_receiver: mpsc::Receiver<WriteOperation>,
    ) {
        while let Some(operation) = write_receiver.recv().await {
            match operation {
                WriteOperation::CreateUser { user, respond_to } => {
                    let result = database.create_user_internal(&user).await;
                    let _ = respond_to.send(result);
                }
                WriteOperation::CreateSession { session, respond_to } => {
                    let result = database.create_session_internal(&session).await;
                    let _ = respond_to.send(result);
                }
                WriteOperation::DeleteSession { token, respond_to } => {
                    let result = database.delete_session_internal(&token).await;
                    let _ = respond_to.send(result);
                }
                WriteOperation::CreateMessageWithDeduplication { message, respond_to } => {
                    let result = database.create_message_with_deduplication_internal(&message).await;
                    let _ = respond_to.send(result);
                }
                WriteOperation::CreateRoom { room, respond_to } => {
                    let result = database.create_room_internal(&room).await;
                    let _ = respond_to.send(result);
                }
                WriteOperation::CreateMembership { membership, respond_to } => {
                    let result = database.create_membership_internal(&membership).await;
                    let _ = respond_to.send(result);
                }
                WriteOperation::CreatePushSubscription { subscription, respond_to } => {
                    let result = database.create_push_subscription_internal(&subscription).await;
                    let _ = respond_to.send(result);
                }
                WriteOperation::UpdateNotificationPreferences { preferences, respond_to } => {
                    let result = database.update_notification_preferences_internal(&preferences).await;
                    let _ = respond_to.send(result);
                }
            }
        }
    }
}

#[async_trait]
impl DatabaseWriter for SerializedDatabaseWriter {
    async fn create_user(&self, user: User) -> Result<(), DatabaseError> {
        let (tx, rx) = oneshot::channel();
        
        self.write_sender
            .send(WriteOperation::CreateUser {
                user,
                respond_to: tx,
            })
            .await
            .map_err(|_| DatabaseError::WriterChannelClosed)?;
        
        rx.await
            .map_err(|_| DatabaseError::WriterChannelClosed)?
    }
    
    async fn create_session(&self, session: Session) -> Result<(), DatabaseError> {
        let (tx, rx) = oneshot::channel();
        
        self.write_sender
            .send(WriteOperation::CreateSession {
                session,
                respond_to: tx,
            })
            .await
            .map_err(|_| DatabaseError::WriterChannelClosed)?;
        
        rx.await
            .map_err(|_| DatabaseError::WriterChannelClosed)?
    }
    
    async fn delete_session(&self, token: String) -> Result<(), DatabaseError> {
        let (tx, rx) = oneshot::channel();
        
        self.write_sender
            .send(WriteOperation::DeleteSession {
                token,
                respond_to: tx,
            })
            .await
            .map_err(|_| DatabaseError::WriterChannelClosed)?;
        
        rx.await
            .map_err(|_| DatabaseError::WriterChannelClosed)?
    }
    
    async fn create_message_with_deduplication(&self, message: Message) -> Result<Message, DatabaseError> {
        let (tx, rx) = oneshot::channel();
        
        self.write_sender
            .send(WriteOperation::CreateMessageWithDeduplication {
                message,
                respond_to: tx,
            })
            .await
            .map_err(|_| DatabaseError::WriterChannelClosed)?;
        
        rx.await
            .map_err(|_| DatabaseError::WriterChannelClosed)?
    }
    
    async fn create_room(&self, room: Room) -> Result<(), DatabaseError> {
        let (tx, rx) = oneshot::channel();
        
        self.write_sender
            .send(WriteOperation::CreateRoom {
                room,
                respond_to: tx,
            })
            .await
            .map_err(|_| DatabaseError::WriterChannelClosed)?;
        
        rx.await
            .map_err(|_| DatabaseError::WriterChannelClosed)?
    }
    
    async fn create_membership(&self, membership: Membership) -> Result<(), DatabaseError> {
        let (tx, rx) = oneshot::channel();
        
        self.write_sender
            .send(WriteOperation::CreateMembership {
                membership,
                respond_to: tx,
            })
            .await
            .map_err(|_| DatabaseError::WriterChannelClosed)?;
        
        rx.await
            .map_err(|_| DatabaseError::WriterChannelClosed)?
    }
    
    async fn create_push_subscription(&self, subscription: PushSubscription) -> Result<(), DatabaseError> {
        let (tx, rx) = oneshot::channel();
        
        self.write_sender
            .send(WriteOperation::CreatePushSubscription {
                subscription,
                respond_to: tx,
            })
            .await
            .map_err(|_| DatabaseError::WriterChannelClosed)?;
        
        rx.await
            .map_err(|_| DatabaseError::WriterChannelClosed)?
    }
    
    async fn update_notification_preferences(&self, preferences: NotificationPreferences) -> Result<(), DatabaseError> {
        let (tx, rx) = oneshot::channel();
        
        self.write_sender
            .send(WriteOperation::UpdateNotificationPreferences {
                preferences,
                respond_to: tx,
            })
            .await
            .map_err(|_| DatabaseError::WriterChannelClosed)?;
        
        rx.await
            .map_err(|_| DatabaseError::WriterChannelClosed)?
    }
}

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Create SQLite connection pool
        let pool = SqlitePool::connect(database_url).await?;
        
        let db = Self { pool };
        
        // Run migrations
        db.migrate().await?;
        
        Ok(db)
    }
    
    pub async fn migrate(&self) -> Result<()> {
        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                bio TEXT,
                admin BOOLEAN NOT NULL DEFAULT FALSE,
                bot_token TEXT UNIQUE,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create rooms table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rooms (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                topic TEXT,
                room_type TEXT NOT NULL CHECK (room_type IN ('open', 'closed', 'direct')),
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_message_at DATETIME
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create messages table with UNIQUE constraint for Critical Gap #1
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                room_id TEXT NOT NULL REFERENCES rooms(id),
                creator_id TEXT NOT NULL REFERENCES users(id),
                content TEXT NOT NULL,
                client_message_id TEXT NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                html_content TEXT,
                mentions TEXT,
                sound_commands TEXT,
                UNIQUE(client_message_id, room_id)
            )
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Add rich text columns to existing messages table if they don't exist
        // This handles the case where the table already exists without rich text fields
        let _ = sqlx::query("ALTER TABLE messages ADD COLUMN html_content TEXT")
            .execute(&self.pool)
            .await; // Ignore error if column already exists
            
        let _ = sqlx::query("ALTER TABLE messages ADD COLUMN mentions TEXT")
            .execute(&self.pool)
            .await; // Ignore error if column already exists
            
        let _ = sqlx::query("ALTER TABLE messages ADD COLUMN sound_commands TEXT")
            .execute(&self.pool)
            .await; // Ignore error if column already exists

        // Create room memberships table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS room_memberships (
                room_id TEXT NOT NULL REFERENCES rooms(id),
                user_id TEXT NOT NULL REFERENCES users(id),
                involvement_level TEXT NOT NULL CHECK (involvement_level IN ('member', 'admin')),
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (room_id, user_id)
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create sessions table for authentication (Critical Gap #4)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                expires_at DATETIME NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create FTS5 virtual table for message search
        // We'll create a standalone FTS5 table since we can't use UUID as content_rowid
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                message_id,
                content
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create triggers to keep FTS5 in sync
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS messages_fts_insert AFTER INSERT ON messages BEGIN
                INSERT INTO messages_fts(message_id, content) VALUES (new.id, new.content);
            END
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS messages_fts_delete AFTER DELETE ON messages BEGIN
                DELETE FROM messages_fts WHERE message_id = old.id;
            END
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS messages_fts_update AFTER UPDATE ON messages BEGIN
                DELETE FROM messages_fts WHERE message_id = old.id;
                INSERT INTO messages_fts(message_id, content) VALUES (new.id, new.content);
            END
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create push subscriptions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS push_subscriptions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                endpoint TEXT NOT NULL,
                p256dh_key TEXT NOT NULL,
                auth_key TEXT NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_used_at DATETIME,
                UNIQUE(user_id, endpoint)
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create notification preferences table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notification_preferences (
                user_id TEXT PRIMARY KEY REFERENCES users(id),
                mentions_enabled BOOLEAN NOT NULL DEFAULT TRUE,
                direct_messages_enabled BOOLEAN NOT NULL DEFAULT TRUE,
                all_messages_enabled BOOLEAN NOT NULL DEFAULT FALSE,
                sounds_enabled BOOLEAN NOT NULL DEFAULT TRUE,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
    
    pub async fn begin(&self) -> Result<Transaction<'_, Sqlite>, sqlx::Error> {
        self.pool.begin().await
    }
}

// Internal database operations (used by the writer task)
impl Database {
    pub(crate) async fn create_user_internal(&self, user: &User) -> Result<(), DatabaseError> {
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
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_user_by_id(&self, user_id: UserId) -> Result<Option<User>, DatabaseError> {
        let row = sqlx::query(
            "SELECT id, name, email, password_hash, bio, admin, bot_token, created_at FROM users WHERE id = ?"
        )
        .bind(user_id.0.to_string())
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let id_str: &str = row.get("id");
            Ok(Some(User {
                id: UserId(uuid::Uuid::parse_str(id_str)?),
                name: row.get("name"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                bio: row.get("bio"),
                admin: row.get("admin"),
                bot_token: row.get("bot_token"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DatabaseError> {
        let row = sqlx::query(
            "SELECT id, name, email, password_hash, bio, admin, bot_token, created_at FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let id_str: &str = row.get("id");
            Ok(Some(User {
                id: UserId(uuid::Uuid::parse_str(id_str)?),
                name: row.get("name"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                bio: row.get("bio"),
                admin: row.get("admin"),
                bot_token: row.get("bot_token"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }
}

// Database operations for sessions (Critical Gap #4)
impl Database {
    pub(crate) async fn create_session_internal(&self, session: &Session) -> Result<(), DatabaseError> {
        sqlx::query(
            "INSERT INTO sessions (token, user_id, created_at, expires_at) VALUES (?, ?, ?, ?)"
        )
        .bind(&session.token)
        .bind(session.user_id.0.to_string())
        .bind(session.created_at)
        .bind(session.expires_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_session(&self, token: &str) -> Result<Option<Session>, DatabaseError> {
        let row = sqlx::query(
            "SELECT token, user_id, created_at, expires_at FROM sessions WHERE token = ? AND expires_at > CURRENT_TIMESTAMP"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let user_id_str: &str = row.get("user_id");
            Ok(Some(Session {
                token: row.get("token"),
                user_id: UserId(uuid::Uuid::parse_str(user_id_str)?),
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub(crate) async fn delete_session_internal(&self, token: &str) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
}

// Database operations for messages (Critical Gap #1 - Deduplication)
impl Database {
    pub(crate) async fn create_message_with_deduplication_internal(
        &self,
        message: &Message,
    ) -> Result<Message, DatabaseError> {
        // First, try to get existing message with same client_message_id and room_id
        if let Some(existing) = self.get_message_by_client_id(
            message.client_message_id,
            message.room_id,
        ).await? {
            return Ok(existing);
        }
        
        // Insert new message with rich text fields
        let mentions_json = if message.mentions.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&message.mentions).unwrap_or_default())
        };
        
        let sound_commands_json = if message.sound_commands.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&message.sound_commands).unwrap_or_default())
        };
        
        sqlx::query(
            r#"
            INSERT INTO messages (id, room_id, creator_id, content, client_message_id, created_at, html_content, mentions, sound_commands)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(message.id.0.to_string())
        .bind(message.room_id.0.to_string())
        .bind(message.creator_id.0.to_string())
        .bind(&message.content)
        .bind(message.client_message_id.to_string())
        .bind(message.created_at)
        .bind(&message.html_content)
        .bind(mentions_json)
        .bind(sound_commands_json)
        .execute(&self.pool)
        .await?;
        
        // Update room's last_message_at
        sqlx::query("UPDATE rooms SET last_message_at = ? WHERE id = ?")
            .bind(message.created_at)
            .bind(message.room_id.0.to_string())
            .execute(&self.pool)
            .await?;
        
        Ok(message.clone())
    }
    
    pub async fn get_message_by_client_id(
        &self,
        client_message_id: uuid::Uuid,
        room_id: RoomId,
    ) -> Result<Option<Message>, DatabaseError> {
        let row = sqlx::query(
            r#"
            SELECT id, room_id, creator_id, content, client_message_id, created_at, html_content, mentions, sound_commands
            FROM messages 
            WHERE client_message_id = ? AND room_id = ?
            "#
        )
        .bind(client_message_id.to_string())
        .bind(room_id.0.to_string())
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let id_str: &str = row.get("id");
            let room_id_str: &str = row.get("room_id");
            let creator_id_str: &str = row.get("creator_id");
            let client_message_id_str: &str = row.get("client_message_id");
            
            // Parse JSON fields
            let mentions: Vec<String> = if let Some(mentions_json) = row.get::<Option<String>, _>("mentions") {
                serde_json::from_str(&mentions_json).unwrap_or_default()
            } else {
                Vec::new()
            };
            
            let sound_commands: Vec<String> = if let Some(commands_json) = row.get::<Option<String>, _>("sound_commands") {
                serde_json::from_str(&commands_json).unwrap_or_default()
            } else {
                Vec::new()
            };
            
            Ok(Some(Message {
                id: MessageId(uuid::Uuid::parse_str(id_str)?),
                room_id: RoomId(uuid::Uuid::parse_str(room_id_str)?),
                creator_id: UserId(uuid::Uuid::parse_str(creator_id_str)?),
                content: row.get("content"),
                client_message_id: uuid::Uuid::parse_str(client_message_id_str)?,
                created_at: row.get("created_at"),
                html_content: row.get("html_content"),
                mentions,
                sound_commands,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn get_room_messages(
        &self,
        room_id: RoomId,
        limit: u32,
        before: Option<MessageId>,
    ) -> Result<Vec<Message>, DatabaseError> {
        let query = if let Some(before_id) = before {
            sqlx::query(
                r#"
                SELECT id, room_id, creator_id, content, client_message_id, created_at, html_content, mentions, sound_commands
                FROM messages 
                WHERE room_id = ? AND created_at < (
                    SELECT created_at FROM messages WHERE id = ?
                )
                ORDER BY created_at DESC 
                LIMIT ?
                "#
            )
            .bind(room_id.0.to_string())
            .bind(before_id.0.to_string())
            .bind(limit as i64)
        } else {
            sqlx::query(
                r#"
                SELECT id, room_id, creator_id, content, client_message_id, created_at, html_content, mentions, sound_commands
                FROM messages 
                WHERE room_id = ?
                ORDER BY created_at DESC 
                LIMIT ?
                "#
            )
            .bind(room_id.0.to_string())
            .bind(limit as i64)
        };
        
        let rows = query.fetch_all(&self.pool).await?;
        
        let mut messages = Vec::new();
        for row in rows {
            let id_str: &str = row.get("id");
            let room_id_str: &str = row.get("room_id");
            let creator_id_str: &str = row.get("creator_id");
            let client_message_id_str: &str = row.get("client_message_id");
            
            // Parse JSON fields
            let mentions: Vec<String> = if let Some(mentions_json) = row.get::<Option<String>, _>("mentions") {
                serde_json::from_str(&mentions_json).unwrap_or_default()
            } else {
                Vec::new()
            };
            
            let sound_commands: Vec<String> = if let Some(commands_json) = row.get::<Option<String>, _>("sound_commands") {
                serde_json::from_str(&commands_json).unwrap_or_default()
            } else {
                Vec::new()
            };
            
            messages.push(Message {
                id: MessageId(uuid::Uuid::parse_str(id_str)?),
                room_id: RoomId(uuid::Uuid::parse_str(room_id_str)?),
                creator_id: UserId(uuid::Uuid::parse_str(creator_id_str)?),
                content: row.get("content"),
                client_message_id: uuid::Uuid::parse_str(client_message_id_str)?,
                created_at: row.get("created_at"),
                html_content: row.get("html_content"),
                mentions,
                sound_commands,
            });
        }
        
        Ok(messages)
    }
    
    /// Get messages since a specific message ID for missed message delivery (Critical Gap #2)
    pub async fn get_messages_since(
        &self,
        user_id: UserId,
        last_seen_message_id: Option<MessageId>,
        limit: u32,
    ) -> Result<Vec<Message>, DatabaseError> {
        let query = if let Some(last_seen_id) = last_seen_message_id {
            // Get messages newer than the last seen message in rooms where user is a member
            sqlx::query(
                r#"
                SELECT m.id, m.room_id, m.creator_id, m.content, m.client_message_id, m.created_at, m.html_content, m.mentions, m.sound_commands
                FROM messages m
                INNER JOIN room_memberships rm ON m.room_id = rm.room_id
                WHERE rm.user_id = ? 
                  AND m.created_at > (
                      SELECT created_at FROM messages WHERE id = ?
                  )
                ORDER BY m.created_at ASC 
                LIMIT ?
                "#
            )
            .bind(user_id.0.to_string())
            .bind(last_seen_id.0.to_string())
            .bind(limit as i64)
        } else {
            // If no last seen message, get recent messages from all user's rooms
            sqlx::query(
                r#"
                SELECT m.id, m.room_id, m.creator_id, m.content, m.client_message_id, m.created_at, m.html_content, m.mentions, m.sound_commands
                FROM messages m
                INNER JOIN room_memberships rm ON m.room_id = rm.room_id
                WHERE rm.user_id = ?
                ORDER BY m.created_at DESC 
                LIMIT ?
                "#
            )
            .bind(user_id.0.to_string())
            .bind(limit as i64)
        };
        
        let rows = query.fetch_all(&self.pool).await?;
        
        let mut messages = Vec::new();
        for row in rows {
            let id_str: &str = row.get("id");
            let room_id_str: &str = row.get("room_id");
            let creator_id_str: &str = row.get("creator_id");
            let client_message_id_str: &str = row.get("client_message_id");
            
            // Parse JSON fields
            let mentions: Vec<String> = if let Some(mentions_json) = row.get::<Option<String>, _>("mentions") {
                serde_json::from_str(&mentions_json).unwrap_or_default()
            } else {
                Vec::new()
            };
            
            let sound_commands: Vec<String> = if let Some(commands_json) = row.get::<Option<String>, _>("sound_commands") {
                serde_json::from_str(&commands_json).unwrap_or_default()
            } else {
                Vec::new()
            };
            
            messages.push(Message {
                id: MessageId(uuid::Uuid::parse_str(id_str)?),
                room_id: RoomId(uuid::Uuid::parse_str(room_id_str)?),
                creator_id: UserId(uuid::Uuid::parse_str(creator_id_str)?),
                content: row.get("content"),
                client_message_id: uuid::Uuid::parse_str(client_message_id_str)?,
                created_at: row.get("created_at"),
                html_content: row.get("html_content"),
                mentions,
                sound_commands,
            });
        }
        
        Ok(messages)
    }
}

// Database operations for rooms and memberships
impl Database {
    pub(crate) async fn create_room_internal(&self, room: &Room) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO rooms (id, name, topic, room_type, created_at, last_message_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(room.id.0.to_string())
        .bind(&room.name)
        .bind(&room.topic)
        .bind(match room.room_type {
            RoomType::Open => "open",
            RoomType::Closed => "closed",
            RoomType::Direct => "direct",
        })
        .bind(room.created_at)
        .bind(room.last_message_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_room_by_id(&self, room_id: RoomId) -> Result<Option<Room>, DatabaseError> {
        let row = sqlx::query(
            "SELECT id, name, topic, room_type, created_at, last_message_at FROM rooms WHERE id = ?"
        )
        .bind(room_id.0.to_string())
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let id_str: &str = row.get("id");
            let room_type_str: &str = row.get("room_type");
            
            let room_type = match room_type_str {
                "open" => RoomType::Open,
                "closed" => RoomType::Closed,
                "direct" => RoomType::Direct,
                _ => return Err(DatabaseError::DataIntegrity { 
                    reason: format!("Invalid room_type: {}", room_type_str) 
                }),
            };
            
            Ok(Some(Room {
                id: RoomId(uuid::Uuid::parse_str(id_str)?),
                name: row.get("name"),
                topic: row.get("topic"),
                room_type,
                created_at: row.get("created_at"),
                last_message_at: row.get("last_message_at"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub(crate) async fn create_membership_internal(
        &self,
        membership: &Membership,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO room_memberships (room_id, user_id, involvement_level, created_at)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(membership.room_id.0.to_string())
        .bind(membership.user_id.0.to_string())
        .bind(match membership.involvement_level {
            InvolvementLevel::Member => "member",
            InvolvementLevel::Admin => "admin",
        })
        .bind(membership.created_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_membership(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<Option<Membership>, DatabaseError> {
        let row = sqlx::query(
            r#"
            SELECT room_id, user_id, involvement_level, created_at 
            FROM room_memberships 
            WHERE room_id = ? AND user_id = ?
            "#
        )
        .bind(room_id.0.to_string())
        .bind(user_id.0.to_string())
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let room_id_str: &str = row.get("room_id");
            let user_id_str: &str = row.get("user_id");
            let involvement_level_str: &str = row.get("involvement_level");
            
            let involvement_level = match involvement_level_str {
                "member" => InvolvementLevel::Member,
                "admin" => InvolvementLevel::Admin,
                _ => return Err(DatabaseError::DataIntegrity { 
                    reason: format!("Invalid involvement_level: {}", involvement_level_str) 
                }),
            };
            
            Ok(Some(Membership {
                room_id: RoomId(uuid::Uuid::parse_str(room_id_str)?),
                user_id: UserId(uuid::Uuid::parse_str(user_id_str)?),
                involvement_level,
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn get_user_rooms(&self, user_id: UserId) -> Result<Vec<Room>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT r.id, r.name, r.topic, r.room_type, r.created_at, r.last_message_at
            FROM rooms r
            INNER JOIN room_memberships rm ON r.id = rm.room_id
            WHERE rm.user_id = ?
            ORDER BY r.last_message_at DESC NULLS LAST, r.created_at DESC
            "#
        )
        .bind(user_id.0.to_string())
        .fetch_all(&self.pool)
        .await?;
        
        let mut rooms = Vec::new();
        for row in rows {
            let id_str: &str = row.get("id");
            let room_type_str: &str = row.get("room_type");
            
            let room_type = match room_type_str {
                "open" => RoomType::Open,
                "closed" => RoomType::Closed,
                "direct" => RoomType::Direct,
                _ => return Err(DatabaseError::DataIntegrity { 
                    reason: format!("Invalid room_type: {}", room_type_str) 
                }),
            };
            
            rooms.push(Room {
                id: RoomId(uuid::Uuid::parse_str(id_str)?),
                name: row.get("name"),
                topic: row.get("topic"),
                room_type,
                created_at: row.get("created_at"),
                last_message_at: row.get("last_message_at"),
            });
        }
        
        Ok(rooms)
    }
    
    pub async fn check_user_can_add_member(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<bool, DatabaseError> {
        // Check if user is admin of the room or if it's an open room
        let room = self.get_room_by_id(room_id).await?;
        if room.is_none() {
            return Ok(false);
        }
        let room = room.unwrap();
        
        // Open rooms allow anyone to join
        if matches!(room.room_type, RoomType::Open) {
            return Ok(true);
        }
        
        // For closed/direct rooms, check if user is admin
        let membership = self.get_membership(room_id, user_id).await?;
        if let Some(membership) = membership {
            Ok(matches!(membership.involvement_level, InvolvementLevel::Admin))
        } else {
            Ok(false)
        }
    }
    
    pub async fn user_exists(&self, user_id: UserId) -> Result<bool, DatabaseError> {
        let row = sqlx::query("SELECT 1 FROM users WHERE id = ?")
            .bind(user_id.0.to_string())
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(row.is_some())
    }
}

/// Combined database interface that uses the writer pattern for writes
/// and direct access for reads (Critical Gap #3 implementation)
#[derive(Clone)]
pub struct CampfireDatabase {
    /// Direct database access for read operations
    read_db: Database,
    /// Serialized writer for all write operations
    writer: Arc<dyn DatabaseWriter>,
}

impl CampfireDatabase {
    /// Create a new database with the writer pattern
    pub async fn new(database_url: &str) -> Result<Self> {
        let read_db = Database::new(database_url).await?;
        let writer_db = read_db.clone();
        let writer = Arc::new(SerializedDatabaseWriter::new(writer_db));
        
        Ok(Self {
            read_db,
            writer,
        })
    }
    
    /// Get the writer interface for write operations
    pub fn writer(&self) -> Arc<dyn DatabaseWriter> {
        Arc::clone(&self.writer)
    }
    
    /// Get the database pool for direct read operations
    pub fn pool(&self) -> &SqlitePool {
        self.read_db.pool()
    }
    
    /// Begin a transaction (for complex operations that need atomicity)
    pub async fn begin(&self) -> Result<Transaction<'_, Sqlite>, sqlx::Error> {
        self.read_db.begin().await
    }
    
    // Read operations - direct access to avoid serialization overhead
    
    pub async fn get_user_by_id(&self, user_id: UserId) -> Result<Option<User>, DatabaseError> {
        self.read_db.get_user_by_id(user_id).await
    }
    
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DatabaseError> {
        self.read_db.get_user_by_email(email).await
    }
    
    pub async fn get_session(&self, token: &str) -> Result<Option<Session>, DatabaseError> {
        self.read_db.get_session(token).await
    }
    
    pub async fn get_message_by_client_id(
        &self,
        client_message_id: uuid::Uuid,
        room_id: RoomId,
    ) -> Result<Option<Message>, DatabaseError> {
        self.read_db.get_message_by_client_id(client_message_id, room_id).await
    }
    
    pub async fn get_room_messages(
        &self,
        room_id: RoomId,
        limit: u32,
        before: Option<MessageId>,
    ) -> Result<Vec<Message>, DatabaseError> {
        self.read_db.get_room_messages(room_id, limit, before).await
    }
    
    pub async fn get_messages_since(
        &self,
        user_id: UserId,
        last_seen_message_id: Option<MessageId>,
        limit: u32,
    ) -> Result<Vec<Message>, DatabaseError> {
        self.read_db.get_messages_since(user_id, last_seen_message_id, limit).await
    }
    
    pub async fn get_room_by_id(&self, room_id: RoomId) -> Result<Option<Room>, DatabaseError> {
        self.read_db.get_room_by_id(room_id).await
    }
    
    pub async fn get_membership(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<Option<Membership>, DatabaseError> {
        self.read_db.get_membership(room_id, user_id).await
    }
    
    pub async fn get_user_rooms(&self, user_id: UserId) -> Result<Vec<Room>, DatabaseError> {
        self.read_db.get_user_rooms(user_id).await
    }
    
    pub async fn check_user_can_add_member(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<bool, DatabaseError> {
        self.read_db.check_user_can_add_member(room_id, user_id).await
    }
    
    pub async fn user_exists(&self, user_id: UserId) -> Result<bool, DatabaseError> {
        self.read_db.user_exists(user_id).await
    }
    
    // Write operations - go through the writer pattern
    
    pub async fn create_user(&self, user: User) -> Result<(), DatabaseError> {
        self.writer.create_user(user).await
    }
    
    pub async fn create_session(&self, session: Session) -> Result<(), DatabaseError> {
        self.writer.create_session(session).await
    }
    
    pub async fn delete_session(&self, token: String) -> Result<(), DatabaseError> {
        self.writer.delete_session(token).await
    }
    
    pub async fn create_message_with_deduplication(&self, message: Message) -> Result<Message, DatabaseError> {
        self.writer.create_message_with_deduplication(message).await
    }
    
    pub async fn create_room(&self, room: Room) -> Result<(), DatabaseError> {
        self.writer.create_room(room).await
    }
    
    pub async fn create_membership(&self, membership: Membership) -> Result<(), DatabaseError> {
        self.writer.create_membership(membership).await
    }
    
    // Push notification operations
    
    pub async fn get_push_subscriptions_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<PushSubscription>, DatabaseError> {
        self.read_db.get_push_subscriptions_for_user(user_id).await
    }
    
    pub async fn delete_push_subscription(
        &self,
        subscription_id: PushSubscriptionId,
    ) -> Result<(), DatabaseError> {
        self.read_db.delete_push_subscription(subscription_id).await
    }
    
    pub async fn get_notification_preferences(
        &self,
        user_id: UserId,
    ) -> Result<NotificationPreferences, DatabaseError> {
        self.read_db.get_notification_preferences(user_id).await
    }
    
    pub async fn get_notification_recipients(
        &self,
        message: &Message,
        room: &Room,
    ) -> Result<Vec<(UserId, NotificationPreferences)>, DatabaseError> {
        self.read_db.get_notification_recipients(message, room).await
    }
    
    pub async fn create_push_subscription(&self, subscription: PushSubscription) -> Result<(), DatabaseError> {
        self.writer.create_push_subscription(subscription).await
    }
    
    pub async fn update_notification_preferences(&self, preferences: NotificationPreferences) -> Result<(), DatabaseError> {
        self.writer.update_notification_preferences(preferences).await
    }
}
// 
// Database operations for push notifications
impl Database {
    pub(crate) async fn create_push_subscription_internal(
        &self,
        subscription: &PushSubscription,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO push_subscriptions 
            (id, user_id, endpoint, p256dh_key, auth_key, created_at, last_used_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(subscription.id.0.to_string())
        .bind(subscription.user_id.0.to_string())
        .bind(&subscription.endpoint)
        .bind(&subscription.p256dh_key)
        .bind(&subscription.auth_key)
        .bind(subscription.created_at)
        .bind(subscription.last_used_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_push_subscriptions_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<PushSubscription>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, endpoint, p256dh_key, auth_key, created_at, last_used_at
            FROM push_subscriptions 
            WHERE user_id = ?
            "#
        )
        .bind(user_id.0.to_string())
        .fetch_all(&self.pool)
        .await?;
        
        let mut subscriptions = Vec::new();
        for row in rows {
            let id_str: &str = row.get("id");
            let user_id_str: &str = row.get("user_id");
            
            subscriptions.push(PushSubscription {
                id: PushSubscriptionId(uuid::Uuid::parse_str(id_str)?),
                user_id: UserId(uuid::Uuid::parse_str(user_id_str)?),
                endpoint: row.get("endpoint"),
                p256dh_key: row.get("p256dh_key"),
                auth_key: row.get("auth_key"),
                created_at: row.get("created_at"),
                last_used_at: row.get("last_used_at"),
            });
        }
        
        Ok(subscriptions)
    }
    
    pub async fn delete_push_subscription(
        &self,
        subscription_id: PushSubscriptionId,
    ) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM push_subscriptions WHERE id = ?")
            .bind(subscription_id.0.to_string())
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    pub(crate) async fn update_notification_preferences_internal(
        &self,
        preferences: &NotificationPreferences,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO notification_preferences 
            (user_id, mentions_enabled, direct_messages_enabled, all_messages_enabled, sounds_enabled, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(preferences.user_id.0.to_string())
        .bind(preferences.mentions_enabled)
        .bind(preferences.direct_messages_enabled)
        .bind(preferences.all_messages_enabled)
        .bind(preferences.sounds_enabled)
        .bind(preferences.updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_notification_preferences(
        &self,
        user_id: UserId,
    ) -> Result<NotificationPreferences, DatabaseError> {
        let row = sqlx::query(
            r#"
            SELECT user_id, mentions_enabled, direct_messages_enabled, all_messages_enabled, sounds_enabled, updated_at
            FROM notification_preferences 
            WHERE user_id = ?
            "#
        )
        .bind(user_id.0.to_string())
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let user_id_str: &str = row.get("user_id");
            Ok(NotificationPreferences {
                user_id: UserId(uuid::Uuid::parse_str(user_id_str)?),
                mentions_enabled: row.get("mentions_enabled"),
                direct_messages_enabled: row.get("direct_messages_enabled"),
                all_messages_enabled: row.get("all_messages_enabled"),
                sounds_enabled: row.get("sounds_enabled"),
                updated_at: row.get("updated_at"),
            })
        } else {
            // Return default preferences if none exist
            Ok(NotificationPreferences {
                user_id,
                ..Default::default()
            })
        }
    }
    
    /// Get users who should receive push notifications for a message
    pub async fn get_notification_recipients(
        &self,
        message: &Message,
        room: &Room,
    ) -> Result<Vec<(UserId, NotificationPreferences)>, DatabaseError> {
        let mut recipients = Vec::new();
        
        // For direct messages, notify the other participant
        if room.room_type == RoomType::Direct {
            let rows = sqlx::query(
                r#"
                SELECT rm.user_id, 
                       COALESCE(np.mentions_enabled, 1) as mentions_enabled,
                       COALESCE(np.direct_messages_enabled, 1) as direct_messages_enabled,
                       COALESCE(np.all_messages_enabled, 0) as all_messages_enabled,
                       COALESCE(np.sounds_enabled, 1) as sounds_enabled,
                       COALESCE(np.updated_at, CURRENT_TIMESTAMP) as updated_at
                FROM room_memberships rm
                LEFT JOIN notification_preferences np ON rm.user_id = np.user_id
                WHERE rm.room_id = ? AND rm.user_id != ? AND np.direct_messages_enabled != 0
                "#
            )
            .bind(message.room_id.0.to_string())
            .bind(message.creator_id.0.to_string())
            .fetch_all(&self.pool)
            .await?;
            
            for row in rows {
                let user_id_str: &str = row.get("user_id");
                recipients.push((
                    UserId(uuid::Uuid::parse_str(user_id_str)?),
                    NotificationPreferences {
                        user_id: UserId(uuid::Uuid::parse_str(user_id_str)?),
                        mentions_enabled: row.get("mentions_enabled"),
                        direct_messages_enabled: row.get("direct_messages_enabled"),
                        all_messages_enabled: row.get("all_messages_enabled"),
                        sounds_enabled: row.get("sounds_enabled"),
                        updated_at: row.get("updated_at"),
                    },
                ));
            }
        } else {
            // For mentions, notify mentioned users
            if !message.mentions.is_empty() {
                for mention in &message.mentions {
                    if let Some(user) = self.get_user_by_email(mention).await? {
                        let preferences = self.get_notification_preferences(user.id).await?;
                        if preferences.mentions_enabled {
                            recipients.push((user.id, preferences));
                        }
                    }
                }
            }
            
            // For all messages (if enabled), notify all room members except sender
            let rows = sqlx::query(
                r#"
                SELECT rm.user_id,
                       COALESCE(np.mentions_enabled, 1) as mentions_enabled,
                       COALESCE(np.direct_messages_enabled, 1) as direct_messages_enabled,
                       COALESCE(np.all_messages_enabled, 0) as all_messages_enabled,
                       COALESCE(np.sounds_enabled, 1) as sounds_enabled,
                       COALESCE(np.updated_at, CURRENT_TIMESTAMP) as updated_at
                FROM room_memberships rm
                LEFT JOIN notification_preferences np ON rm.user_id = np.user_id
                WHERE rm.room_id = ? AND rm.user_id != ? AND np.all_messages_enabled = 1
                "#
            )
            .bind(message.room_id.0.to_string())
            .bind(message.creator_id.0.to_string())
            .fetch_all(&self.pool)
            .await?;
            
            for row in rows {
                let user_id_str: &str = row.get("user_id");
                let user_id = UserId(uuid::Uuid::parse_str(user_id_str)?);
                
                // Skip if already added for mentions
                if !recipients.iter().any(|(id, _)| *id == user_id) {
                    recipients.push((
                        user_id,
                        NotificationPreferences {
                            user_id,
                            mentions_enabled: row.get("mentions_enabled"),
                            direct_messages_enabled: row.get("direct_messages_enabled"),
                            all_messages_enabled: row.get("all_messages_enabled"),
                            sounds_enabled: row.get("sounds_enabled"),
                            updated_at: row.get("updated_at"),
                        },
                    ));
                }
            }
        }
        
        Ok(recipients)
    }
}