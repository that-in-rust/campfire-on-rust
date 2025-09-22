use sqlx::{sqlite::SqlitePool, Row, Sqlite, Transaction};
use anyhow::Result;
use crate::errors::DatabaseError;
use crate::models::*;

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
                UNIQUE(client_message_id, room_id)
            )
            "#
        )
        .execute(&self.pool)
        .await?;

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
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                content,
                content=messages,
                content_rowid=id
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create triggers to keep FTS5 in sync
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS messages_fts_insert AFTER INSERT ON messages BEGIN
                INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
            END
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS messages_fts_delete AFTER DELETE ON messages BEGIN
                DELETE FROM messages_fts WHERE rowid = old.id;
            END
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS messages_fts_update AFTER UPDATE ON messages BEGIN
                DELETE FROM messages_fts WHERE rowid = old.id;
                INSERT INTO messages_fts(rowid, content) VALUES (new.id, new.content);
            END
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

// Database operations for users
impl Database {
    pub async fn create_user(&self, user: &User) -> Result<(), DatabaseError> {
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
    pub async fn create_session(&self, session: &Session) -> Result<(), DatabaseError> {
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
    
    pub async fn delete_session(&self, token: &str) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
}

// Database operations for messages (Critical Gap #1 - Deduplication)
impl Database {
    pub async fn create_message_with_deduplication(
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
        
        // Insert new message
        sqlx::query(
            r#"
            INSERT INTO messages (id, room_id, creator_id, content, client_message_id, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(message.id.0.to_string())
        .bind(message.room_id.0.to_string())
        .bind(message.creator_id.0.to_string())
        .bind(&message.content)
        .bind(message.client_message_id.to_string())
        .bind(message.created_at)
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
            SELECT id, room_id, creator_id, content, client_message_id, created_at
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
            
            Ok(Some(Message {
                id: MessageId(uuid::Uuid::parse_str(id_str)?),
                room_id: RoomId(uuid::Uuid::parse_str(room_id_str)?),
                creator_id: UserId(uuid::Uuid::parse_str(creator_id_str)?),
                content: row.get("content"),
                client_message_id: uuid::Uuid::parse_str(client_message_id_str)?,
                created_at: row.get("created_at"),
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
                SELECT id, room_id, creator_id, content, client_message_id, created_at
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
                SELECT id, room_id, creator_id, content, client_message_id, created_at
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
            
            messages.push(Message {
                id: MessageId(uuid::Uuid::parse_str(id_str)?),
                room_id: RoomId(uuid::Uuid::parse_str(room_id_str)?),
                creator_id: UserId(uuid::Uuid::parse_str(creator_id_str)?),
                content: row.get("content"),
                client_message_id: uuid::Uuid::parse_str(client_message_id_str)?,
                created_at: row.get("created_at"),
            });
        }
        
        Ok(messages)
    }
}