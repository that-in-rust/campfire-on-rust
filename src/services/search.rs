use async_trait::async_trait;
use std::sync::Arc;
use crate::database::CampfireDatabase;
use crate::models::{Message, UserId, RoomId, MessageId};
use crate::errors::{DatabaseError, RoomError};
use crate::services::room::{RoomServiceTrait};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use sqlx::Row;

/// Search-specific errors
#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Invalid search query: {reason}")]
    InvalidQuery { reason: String },
    
    #[error("Search query too short: minimum 2 characters")]
    QueryTooShort,
    
    #[error("Search query too long: maximum 100 characters")]
    QueryTooLong,
    
    #[error("Database operation failed: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("Room access error: {0}")]
    RoomAccess(#[from] RoomError),
}

/// Search result with ranking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub message: Message,
    pub rank: f64,
    pub snippet: String,
}

/// Search request parameters
#[derive(Debug, Clone, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub room_id: Option<RoomId>,
}

/// Search response with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: u32,
    pub query: String,
    pub limit: u32,
    pub offset: u32,
    pub has_more: bool,
}

/// Search service trait for full-text search functionality
#[async_trait]
pub trait SearchServiceTrait: Send + Sync {
    /// Search messages with full-text search
    /// 
    /// # Preconditions
    /// - User authenticated
    /// - Query: 2-100 chars, non-empty after trimming
    /// - User has access to rooms containing results
    /// 
    /// # Postconditions  
    /// - Returns Ok(SearchResponse) with authorized results only
    /// - Results ranked by FTS5 relevance score
    /// - Includes snippet with highlighted matches
    /// - Pagination support with limit/offset
    /// 
    /// # Error Conditions
    /// - SearchError::InvalidQuery if query format invalid
    /// - SearchError::QueryTooShort if query < 2 chars
    /// - SearchError::QueryTooLong if query > 100 chars
    /// - SearchError::Database on FTS5 operation failure
    async fn search_messages(
        &self,
        user_id: UserId,
        request: SearchRequest,
    ) -> Result<SearchResponse, SearchError>;
    
    /// Search messages within a specific room
    async fn search_room_messages(
        &self,
        user_id: UserId,
        room_id: RoomId,
        query: String,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResponse, SearchError>;
}

/// Implementation of SearchService using SQLite FTS5
#[derive(Clone)]
pub struct SearchService {
    db: Arc<CampfireDatabase>,
    room_service: Arc<dyn RoomServiceTrait>,
}

impl SearchService {
    pub fn new(
        db: Arc<CampfireDatabase>,
        room_service: Arc<dyn RoomServiceTrait>,
    ) -> Self {
        Self { db, room_service }
    }
    
    /// Get reference to the database for testing purposes
    pub fn database(&self) -> &Arc<CampfireDatabase> {
        &self.db
    }
    
    /// Validate search query
    fn validate_query(&self, query: &str) -> Result<String, SearchError> {
        let trimmed = query.trim();
        
        if trimmed.is_empty() {
            return Err(SearchError::InvalidQuery {
                reason: "Query cannot be empty".to_string(),
            });
        }
        
        if trimmed.len() < 2 {
            return Err(SearchError::QueryTooShort);
        }
        
        if trimmed.len() > 100 {
            return Err(SearchError::QueryTooLong);
        }
        
        // Escape FTS5 special characters to prevent injection
        let escaped = trimmed
            .replace('"', "\"\"")  // Escape quotes
            .replace('*', "")    // Remove wildcards for safety
            .replace(':', "");   // Remove column specifiers
        
        Ok(escaped)
    }
    
    /// Generate snippet with highlighted matches
    fn generate_snippet(&self, content: &str, query: &str) -> String {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        
        // Find the first occurrence of the query
        if let Some(pos) = content_lower.find(&query_lower) {
            let start = pos.saturating_sub(50);
            let end = (pos + query.len() + 50).min(content.len());
            
            let mut snippet = content[start..end].to_string();
            
            // Add ellipsis if truncated
            if start > 0 {
                snippet = format!("...{}", snippet);
            }
            if end < content.len() {
                snippet = format!("{}...", snippet);
            }
            
            snippet
        } else {
            // Fallback to first 100 characters
            if content.len() > 100 {
                format!("{}...", &content[..100])
            } else {
                content.to_string()
            }
        }
    }
    
    /// Get user's accessible room IDs for authorization
    async fn get_user_room_ids(&self, user_id: UserId) -> Result<Vec<RoomId>, SearchError> {
        let rooms = self.room_service.get_user_rooms(user_id).await?;
        Ok(rooms.into_iter().map(|room| room.id).collect())
    }
}

#[async_trait]
impl SearchServiceTrait for SearchService {
    async fn search_messages(
        &self,
        user_id: UserId,
        request: SearchRequest,
    ) -> Result<SearchResponse, SearchError> {
        // Validate query
        let validated_query = self.validate_query(&request.query)?;
        
        // Get pagination parameters
        let limit = request.limit.unwrap_or(20).min(100); // Max 100 results per page
        let offset = request.offset.unwrap_or(0);
        
        // Get user's accessible rooms for authorization
        let accessible_room_ids = self.get_user_room_ids(user_id).await?;
        
        if accessible_room_ids.is_empty() {
            return Ok(SearchResponse {
                results: vec![],
                total_count: 0,
                query: request.query,
                limit,
                offset,
                has_more: false,
            });
        }
        
        // Build room filter for SQL
        let room_placeholders = accessible_room_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        
        // Search with FTS5 and join with messages table for full data
        let search_query = if let Some(room_id) = request.room_id {
            // Search within specific room (if user has access)
            if !accessible_room_ids.contains(&room_id) {
                return Ok(SearchResponse {
                    results: vec![],
                    total_count: 0,
                    query: request.query,
                    limit,
                    offset,
                    has_more: false,
                });
            }
            
            format!(
                r#"
                SELECT m.id, m.room_id, m.creator_id, m.content, m.client_message_id, m.created_at,
                       rank
                FROM messages_fts fts
                INNER JOIN messages m ON fts.message_id = m.id
                WHERE messages_fts MATCH ? AND m.room_id = ?
                ORDER BY rank, m.created_at DESC
                LIMIT ? OFFSET ?
                "#
            )
        } else {
            // Search across all accessible rooms
            format!(
                r#"
                SELECT m.id, m.room_id, m.creator_id, m.content, m.client_message_id, m.created_at,
                       rank
                FROM messages_fts fts
                INNER JOIN messages m ON fts.message_id = m.id
                WHERE messages_fts MATCH ? AND m.room_id IN ({})
                ORDER BY rank, m.created_at DESC
                LIMIT ? OFFSET ?
                "#,
                room_placeholders
            )
        };
        
        // Execute search query
        let mut query_builder = sqlx::query(&search_query)
            .bind(&validated_query);
        
        if let Some(room_id) = request.room_id {
            query_builder = query_builder.bind(room_id.0.to_string());
        } else {
            for room_id in &accessible_room_ids {
                query_builder = query_builder.bind(room_id.0.to_string());
            }
        }
        
        query_builder = query_builder
            .bind(limit as i64)
            .bind(offset as i64);
        
        let rows = query_builder
            .fetch_all(self.db.pool())
            .await
            .map_err(|e| DatabaseError::Connection(e))?;
        
        // Convert rows to search results
        let mut results = Vec::new();
        for row in rows {
            let id_str: &str = row.get("id");
            let room_id_str: &str = row.get("room_id");
            let creator_id_str: &str = row.get("creator_id");
            let client_message_id_str: &str = row.get("client_message_id");
            let content: String = row.get("content");
            let rank: f64 = row.get("rank");
            
            let message = Message {
                id: MessageId(uuid::Uuid::parse_str(id_str)
                    .map_err(|e| DatabaseError::UuidParse(e))?),
                room_id: RoomId(uuid::Uuid::parse_str(room_id_str)
                    .map_err(|e| DatabaseError::UuidParse(e))?),
                creator_id: UserId(uuid::Uuid::parse_str(creator_id_str)
                    .map_err(|e| DatabaseError::UuidParse(e))?),
                content: content.clone(),
                client_message_id: uuid::Uuid::parse_str(client_message_id_str)
                    .map_err(|e| DatabaseError::UuidParse(e))?,
                created_at: row.get("created_at"),
                html_content: None,
                mentions: Vec::new(),
                sound_commands: Vec::new(),
            };
            
            let snippet = self.generate_snippet(&content, &validated_query);
            
            results.push(SearchResult {
                message,
                rank,
                snippet,
            });
        }
        
        // Get total count for pagination
        let count_query = if let Some(_room_id) = request.room_id {
            format!(
                r#"
                SELECT COUNT(*) as total
                FROM messages_fts fts
                INNER JOIN messages m ON fts.message_id = m.id
                WHERE messages_fts MATCH ? AND m.room_id = ?
                "#
            )
        } else {
            format!(
                r#"
                SELECT COUNT(*) as total
                FROM messages_fts fts
                INNER JOIN messages m ON fts.message_id = m.id
                WHERE messages_fts MATCH ? AND m.room_id IN ({})
                "#,
                room_placeholders
            )
        };
        
        let mut count_query_builder = sqlx::query(&count_query)
            .bind(&validated_query);
        
        if let Some(room_id) = request.room_id {
            count_query_builder = count_query_builder.bind(room_id.0.to_string());
        } else {
            for room_id in &accessible_room_ids {
                count_query_builder = count_query_builder.bind(room_id.0.to_string());
            }
        }
        
        let count_row = count_query_builder
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| DatabaseError::Connection(e))?;
        
        let total_count: i64 = count_row.get("total");
        let has_more = (offset + limit) < total_count as u32;
        
        Ok(SearchResponse {
            results,
            total_count: total_count as u32,
            query: request.query,
            limit,
            offset,
            has_more,
        })
    }
    
    async fn search_room_messages(
        &self,
        user_id: UserId,
        room_id: RoomId,
        query: String,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResponse, SearchError> {
        let request = SearchRequest {
            query,
            limit: Some(limit),
            offset: Some(offset),
            room_id: Some(room_id),
        };
        
        self.search_messages(user_id, request).await
    }
}

// HTTP status code conversions
impl From<SearchError> for axum::http::StatusCode {
    fn from(err: SearchError) -> Self {
        match err {
            SearchError::InvalidQuery { .. }
            | SearchError::QueryTooShort
            | SearchError::QueryTooLong => axum::http::StatusCode::BAD_REQUEST,
            SearchError::Database(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            SearchError::RoomAccess(_) => axum::http::StatusCode::FORBIDDEN,
        }
    }
}