use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use dashmap::DashMap;
use parking_lot::Mutex;
use arc_swap::ArcSwap;
use uuid::Uuid;
use tracing::{debug, info, warn, error};

use crate::errors::{ConnectionError, BroadcastError};
use crate::models::{ConnectionId, MessageId, RoomId, UserId, WebSocketMessage};
use crate::services::ConnectionManager;
use crate::metrics::get_performance_monitor;

// Type alias for WebSocket sender
pub type WebSocketSender = mpsc::UnboundedSender<String>;

/// High-performance WebSocket connection manager with optimizations
pub struct OptimizedConnectionManager {
    /// Active WebSocket connections using DashMap for concurrent access
    connections: DashMap<ConnectionId, ConnectionInfo>,
    
    /// User to connections mapping for fast user lookups
    user_connections: DashMap<UserId, Vec<ConnectionId>>,
    
    /// Room to users mapping for fast room broadcasts
    room_users: DashMap<RoomId, Arc<Vec<UserId>>>,
    
    /// Presence tracking with atomic operations
    presence: DashMap<UserId, PresenceInfo>,
    
    /// Room-specific presence and typing indicators
    room_presence: DashMap<RoomId, Arc<RoomPresence>>,
    
    /// Message broadcast cache for deduplication
    broadcast_cache: Arc<moka::future::Cache<String, Arc<[u8]>>>,
    
    /// Performance metrics
    metrics: Arc<ConnectionMetrics>,
    
    /// Configuration
    config: ConnectionManagerConfig,
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    user_id: UserId,
    sender: WebSocketSender,
    last_seen_message_id: Option<MessageId>,
    connected_at: Instant,
    last_activity: Instant,
    room_subscriptions: Vec<RoomId>,
}

#[derive(Debug, Clone)]
struct PresenceInfo {
    user_id: UserId,
    connection_count: usize,
    last_seen: Instant,
    status: PresenceStatus,
}

#[derive(Debug, Clone)]
enum PresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
}

#[derive(Debug, Clone)]
struct RoomPresence {
    online_users: HashMap<UserId, Instant>,
    typing_users: HashMap<UserId, Instant>,
    last_updated: Instant,
}

#[derive(Debug, Clone)]
pub struct ConnectionManagerConfig {
    pub presence_timeout: Duration,
    pub typing_timeout: Duration,
    pub cleanup_interval: Duration,
    pub broadcast_cache_size: usize,
    pub broadcast_cache_ttl: Duration,
    pub max_connections_per_user: usize,
    pub connection_timeout: Duration,
}

impl Default for ConnectionManagerConfig {
    fn default() -> Self {
        Self {
            presence_timeout: Duration::from_secs(60),
            typing_timeout: Duration::from_secs(10),
            cleanup_interval: Duration::from_secs(30),
            broadcast_cache_size: 10_000,
            broadcast_cache_ttl: Duration::from_secs(60),
            max_connections_per_user: 10,
            connection_timeout: Duration::from_secs(300),
        }
    }
}

#[derive(Debug, Default)]
struct ConnectionMetrics {
    total_connections: std::sync::atomic::AtomicU64,
    active_connections: std::sync::atomic::AtomicU64,
    total_broadcasts: std::sync::atomic::AtomicU64,
    failed_broadcasts: std::sync::atomic::AtomicU64,
    cache_hits: std::sync::atomic::AtomicU64,
    cache_misses: std::sync::atomic::AtomicU64,
}

impl OptimizedConnectionManager {
    pub fn new(config: Option<ConnectionManagerConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        let broadcast_cache = Arc::new(
            moka::future::Cache::builder()
                .max_capacity(config.broadcast_cache_size as u64)
                .time_to_live(config.broadcast_cache_ttl)
                .build()
        );
        
        let manager = Self {
            connections: DashMap::new(),
            user_connections: DashMap::new(),
            room_users: DashMap::new(),
            presence: DashMap::new(),
            room_presence: DashMap::new(),
            broadcast_cache,
            metrics: Arc::new(ConnectionMetrics::default()),
            config,
        };
        
        // Start background tasks
        manager.start_cleanup_task();
        manager.start_metrics_task();
        
        manager
    }
    
    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let connections = self.connections.clone();
        let user_connections = self.user_connections.clone();
        let presence = self.presence.clone();
        let room_presence = self.room_presence.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.cleanup_interval);
            
            loop {
                interval.tick().await;
                
                let now = Instant::now();
                let mut dead_connections = Vec::new();
                let mut stale_presence = Vec::new();
                
                // Find dead connections
                for entry in connections.iter() {
                    let (connection_id, info) = entry.pair();
                    
                    if info.sender.is_closed() || 
                       now.duration_since(info.last_activity) > config.connection_timeout {
                        dead_connections.push(*connection_id);
                    }
                }
                
                // Clean up dead connections
                for connection_id in dead_connections {
                    if let Some((_, info)) = connections.remove(&connection_id) {
                        // Remove from user connections
                        if let Some(mut user_conns) = user_connections.get_mut(&info.user_id) {
                            user_conns.retain(|&id| id != connection_id);
                            if user_conns.is_empty() {
                                user_connections.remove(&info.user_id);
                            }
                        }
                        
                        debug!("Cleaned up dead connection: {}", connection_id.0);
                    }
                }
                
                // Find stale presence entries
                for entry in presence.iter() {
                    let (user_id, info) = entry.pair();
                    
                    if now.duration_since(info.last_seen) > config.presence_timeout {
                        stale_presence.push(*user_id);
                    }
                }
                
                // Clean up stale presence
                for user_id in stale_presence {
                    presence.remove(&user_id);
                    debug!("Cleaned up stale presence for user: {}", user_id.0);
                }
                
                // Clean up typing indicators
                let mut rooms_to_update = Vec::new();
                
                for entry in room_presence.iter() {
                    let room_id = *entry.key();
                    let current_presence = entry.value();
                    let mut updated = false;
                    
                    // Check for expired typing indicators
                    let mut new_typing_users = current_presence.typing_users.clone();
                    new_typing_users.retain(|_, &mut started_at| {
                        let keep = now.duration_since(started_at) <= config.typing_timeout;
                        if !keep {
                            updated = true;
                        }
                        keep
                    });
                    
                    if updated {
                        let new_presence = RoomPresence {
                            online_users: current_presence.online_users.clone(),
                            typing_users: new_typing_users,
                            last_updated: now,
                        };
                        rooms_to_update.push((room_id, Arc::new(new_presence)));
                    }
                }
                
                // Update the rooms that need updating
                for (room_id, new_presence) in rooms_to_update {
                    room_presence.insert(room_id, new_presence);
                }
            }
        });
    }
    
    /// Start metrics collection task
    fn start_metrics_task(&self) {
        let connections = self.connections.clone();
        let metrics = Arc::clone(&self.metrics);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                let active_count = connections.len() as u64;
                metrics.active_connections.store(
                    active_count,
                    std::sync::atomic::Ordering::Relaxed
                );
                
                // Update performance monitor
                let monitor = get_performance_monitor();
                monitor.update_websocket_stats(|stats| {
                    stats.active_connections = active_count as usize;
                }).await;
            }
        });
    }
    
    /// Get or create cached broadcast message
    async fn get_cached_broadcast(&self, cache_key: &str, message: &WebSocketMessage) -> Arc<[u8]> {
        if let Some(cached) = self.broadcast_cache.get(cache_key).await {
            self.metrics.cache_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return cached;
        }
        
        self.metrics.cache_misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Serialize message
        let serialized = serde_json::to_vec(message).unwrap_or_default();
        let arc_data: Arc<[u8]> = serialized.into();
        
        // Cache for future use
        self.broadcast_cache.insert(cache_key.to_string(), arc_data.clone()).await;
        
        arc_data
    }
    
    /// Update presence information for a user
    async fn update_presence(&self, user_id: UserId) {
        let connection_count = self.user_connections
            .get(&user_id)
            .map(|conns| conns.len())
            .unwrap_or(0);
        
        if connection_count > 0 {
            self.presence.insert(user_id, PresenceInfo {
                user_id,
                connection_count,
                last_seen: Instant::now(),
                status: PresenceStatus::Online,
            });
        } else {
            self.presence.remove(&user_id);
        }
    }
    
    /// Update room presence for a user
    async fn update_room_presence(&self, user_id: UserId, room_ids: &[RoomId]) {
        let is_online = self.presence.contains_key(&user_id);
        
        for &room_id in room_ids {
            let mut entry = self.room_presence.entry(room_id).or_insert_with(|| {
                Arc::new(RoomPresence {
                    online_users: HashMap::new(),
                    typing_users: HashMap::new(),
                    last_updated: Instant::now(),
                })
            });
            
            // Clone the current state and modify it
            let mut new_presence = (**entry).clone();
            
            if is_online {
                new_presence.online_users.insert(user_id, Instant::now());
            } else {
                new_presence.online_users.remove(&user_id);
                new_presence.typing_users.remove(&user_id);
            }
            
            new_presence.last_updated = Instant::now();
            
            // Update the Arc
            *entry = Arc::new(new_presence);
        }
    }
    
    /// Get connections for users in a room
    fn get_room_connections(&self, room_id: RoomId) -> Vec<(ConnectionId, WebSocketSender)> {
        let mut room_connections = Vec::new();
        
        if let Some(users) = self.room_users.get(&room_id) {
            for &user_id in users.iter() {
                if let Some(connection_ids) = self.user_connections.get(&user_id) {
                    for &connection_id in connection_ids.iter() {
                        if let Some(info) = self.connections.get(&connection_id) {
                            room_connections.push((connection_id, info.sender.clone()));
                        }
                    }
                }
            }
        }
        
        room_connections
    }
    
    /// Broadcast message to room with optimizations
    async fn broadcast_optimized(
        &self,
        room_id: RoomId,
        message: &WebSocketMessage,
    ) -> Result<usize, BroadcastError> {
        let start = Instant::now();
        
        // Create cache key for this message type
        let message_type_id = match message {
            WebSocketMessage::NewMessage { .. } => 0u8,
            WebSocketMessage::UserJoined { .. } => 1u8,
            WebSocketMessage::UserLeft { .. } => 2u8,
            WebSocketMessage::TypingStart { .. } => 3u8,
            WebSocketMessage::TypingStop { .. } => 4u8,
            WebSocketMessage::TypingIndicator { .. } => 5u8,
            WebSocketMessage::PresenceUpdate { .. } => 6u8,
            WebSocketMessage::SoundPlayback { .. } => 7u8,
        };
        
        let cache_key = format!("{}:{}", 
            message_type_id,
            match message {
                WebSocketMessage::NewMessage { message } => message.id.0.to_string(),
                WebSocketMessage::PresenceUpdate { room_id, .. } => room_id.0.to_string(),
                WebSocketMessage::TypingIndicator { room_id, .. } => room_id.0.to_string(),
                _ => "generic".to_string(),
            }
        );
        
        // Get cached serialized message
        let serialized_data = self.get_cached_broadcast(&cache_key, message).await;
        let serialized = String::from_utf8_lossy(&serialized_data);
        
        // Get room connections
        let room_connections = self.get_room_connections(room_id);
        
        if room_connections.is_empty() {
            return Err(BroadcastError::NoConnections { room_id });
        }
        
        let mut successful_sends = 0;
        let mut failed_sends = 0;
        
        // Send to all connections concurrently
        let send_futures: Vec<_> = room_connections
            .into_iter()
            .map(|(connection_id, sender)| {
                let message_clone = serialized.to_string();
                async move {
                    match sender.send(message_clone) {
                        Ok(_) => Ok(connection_id),
                        Err(_) => Err(connection_id),
                    }
                }
            })
            .collect();
        
        let results = futures_util::future::join_all(send_futures).await;
        
        for result in results {
            match result {
                Ok(_) => successful_sends += 1,
                Err(connection_id) => {
                    failed_sends += 1;
                    warn!("Failed to send message to connection {}", connection_id.0);
                }
            }
        }
        
        let duration = start.elapsed();
        
        // Update metrics
        self.metrics.total_broadcasts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if failed_sends > 0 {
            self.metrics.failed_broadcasts.fetch_add(failed_sends, std::sync::atomic::Ordering::Relaxed);
        }
        
        // Update performance monitor
        let monitor = get_performance_monitor();
        monitor.update_websocket_stats(|stats| {
            stats.broadcast_latency_ms = duration.as_millis() as f64;
            stats.total_messages_sent += successful_sends;
        }).await;
        
        debug!(
            "Broadcasted to room {} - successful: {}, failed: {}, duration: {:?}",
            room_id.0, successful_sends, failed_sends, duration
        );
        
        if failed_sends > 0 {
            Err(BroadcastError::PartialFailure { connection_count: failed_sends as usize })
        } else {
            Ok(successful_sends as usize)
        }
    }
}

#[async_trait]
impl ConnectionManager for OptimizedConnectionManager {
    async fn add_connection(
        &self,
        user_id: UserId,
        connection_id: ConnectionId,
        sender: WebSocketSender,
    ) -> Result<(), ConnectionError> {
        let now = Instant::now();
        
        // Check connection limit per user
        if let Some(existing_connections) = self.user_connections.get(&user_id) {
            if existing_connections.len() >= self.config.max_connections_per_user {
                return Err(ConnectionError::TooManyConnections { 
                    current_count: existing_connections.len(),
                    max_count: self.config.max_connections_per_user 
                });
            }
        }
        
        // Add connection info
        let connection_info = ConnectionInfo {
            user_id,
            sender,
            last_seen_message_id: None,
            connected_at: now,
            last_activity: now,
            room_subscriptions: Vec::new(),
        };
        
        self.connections.insert(connection_id, connection_info);
        
        // Add to user connections
        self.user_connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(connection_id);
        
        // Update presence
        self.update_presence(user_id).await;
        
        // Update metrics
        self.metrics.total_connections.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        info!("Added optimized connection {} for user {}", connection_id.0, user_id.0);
        
        Ok(())
    }
    
    async fn remove_connection(
        &self,
        connection_id: ConnectionId,
    ) -> Result<(), ConnectionError> {
        let user_id = if let Some((_, info)) = self.connections.remove(&connection_id) {
            // Remove from user connections
            if let Some(mut user_conns) = self.user_connections.get_mut(&info.user_id) {
                user_conns.retain(|&id| id != connection_id);
                if user_conns.is_empty() {
                    self.user_connections.remove(&info.user_id);
                }
            }
            
            // Update room presence for all subscribed rooms
            self.update_room_presence(info.user_id, &info.room_subscriptions).await;
            
            info.user_id
        } else {
            return Err(ConnectionError::NotFound { connection_id });
        };
        
        // Update presence
        self.update_presence(user_id).await;
        
        info!("Removed optimized connection {} for user {}", connection_id.0, user_id.0);
        
        Ok(())
    }
    
    async fn broadcast_to_room(
        &self,
        room_id: RoomId,
        message: WebSocketMessage,
    ) -> Result<(), BroadcastError> {
        match self.broadcast_optimized(room_id, &message).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    async fn get_room_presence(
        &self,
        room_id: RoomId,
    ) -> Result<Vec<UserId>, ConnectionError> {
        if let Some(room_presence) = self.room_presence.get(&room_id) {
            Ok(room_presence.online_users.keys().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn send_missed_messages(
        &self,
        user_id: UserId,
        connection_id: ConnectionId,
        last_seen_message_id: Option<MessageId>,
    ) -> Result<(), ConnectionError> {
        // This would integrate with the database to fetch missed messages
        // For now, we'll just update the connection's last seen message
        if let Some(mut connection_info) = self.connections.get_mut(&connection_id) {
            connection_info.last_seen_message_id = last_seen_message_id;
            connection_info.last_activity = Instant::now();
        }
        
        info!("Processed missed messages for user {} connection {}", user_id.0, connection_id.0);
        Ok(())
    }
    
    async fn update_last_seen_message(
        &self,
        connection_id: ConnectionId,
        message_id: MessageId,
    ) -> Result<(), ConnectionError> {
        if let Some(mut connection_info) = self.connections.get_mut(&connection_id) {
            connection_info.last_seen_message_id = Some(message_id);
            connection_info.last_activity = Instant::now();
            Ok(())
        } else {
            Err(ConnectionError::NotFound { connection_id })
        }
    }
    
    async fn send_to_connection(
        &self,
        connection_id: ConnectionId,
        message: String,
    ) -> Result<(), ConnectionError> {
        if let Some(connection_info) = self.connections.get(&connection_id) {
            connection_info.sender.send(message)
                .map_err(|_| ConnectionError::SendFailed { 
                    reason: "Connection closed".to_string() 
                })?;
            Ok(())
        } else {
            Err(ConnectionError::NotFound { connection_id })
        }
    }
    
    async fn get_room_specific_presence(
        &self,
        room_id: RoomId,
    ) -> Result<Vec<UserId>, ConnectionError> {
        self.get_room_presence(room_id).await
    }
    
    async fn start_typing(
        &self,
        user_id: UserId,
        room_id: RoomId,
    ) -> Result<(), ConnectionError> {
        let mut entry = self.room_presence.entry(room_id).or_insert_with(|| {
            Arc::new(RoomPresence {
                online_users: HashMap::new(),
                typing_users: HashMap::new(),
                last_updated: Instant::now(),
            })
        });
        
        // Clone and modify
        let mut new_presence = (**entry).clone();
        new_presence.typing_users.insert(user_id, Instant::now());
        new_presence.last_updated = Instant::now();
        
        // Update the Arc
        *entry = Arc::new(new_presence);
        
        debug!("User {} started typing in room {}", user_id.0, room_id.0);
        Ok(())
    }
    
    async fn stop_typing(
        &self,
        user_id: UserId,
        room_id: RoomId,
    ) -> Result<(), ConnectionError> {
        if let Some(mut entry) = self.room_presence.get_mut(&room_id) {
            // Clone and modify
            let mut new_presence = (**entry).clone();
            new_presence.typing_users.remove(&user_id);
            new_presence.last_updated = Instant::now();
            
            // Update the Arc
            *entry = Arc::new(new_presence);
            
            debug!("User {} stopped typing in room {}", user_id.0, room_id.0);
        }
        
        Ok(())
    }
    
    async fn get_typing_users(
        &self,
        room_id: RoomId,
    ) -> Result<Vec<UserId>, ConnectionError> {
        if let Some(room_presence) = self.room_presence.get(&room_id) {
            Ok(room_presence.typing_users.keys().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn broadcast_presence_update(
        &self,
        room_id: RoomId,
    ) -> Result<(), BroadcastError> {
        let online_users = self.get_room_presence(room_id).await
            .map_err(|_| BroadcastError::PartialFailure { connection_count: 1 })?;
        
        let presence_msg = WebSocketMessage::PresenceUpdate {
            room_id,
            online_users,
        };
        
        self.broadcast_to_room(room_id, presence_msg).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OptimizedConnectionError {
    #[error("Too many connections for user {user_id}: limit is {limit}")]
    TooManyConnections { user_id: UserId, limit: usize },
    
    #[error("Connection {connection_id} not found")]
    NotFound { connection_id: ConnectionId },
    
    #[error("Send failed: {reason}")]
    SendFailed { reason: String },
}

impl From<OptimizedConnectionError> for ConnectionError {
    fn from(err: OptimizedConnectionError) -> Self {
        match err {
            OptimizedConnectionError::TooManyConnections { user_id, limit } => {
                ConnectionError::Protocol(format!("Too many connections for user {}: limit is {}", user_id.0, limit))
            }
            OptimizedConnectionError::NotFound { connection_id } => {
                ConnectionError::NotFound { connection_id }
            }
            OptimizedConnectionError::SendFailed { reason } => {
                ConnectionError::SendFailed { reason }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    
    #[tokio::test]
    async fn test_optimized_connection_management() {
        let manager = OptimizedConnectionManager::new(None);
        let user_id = UserId::new();
        let connection_id = ConnectionId::new();
        
        let (sender, _receiver) = mpsc::unbounded_channel();
        
        // Add connection
        manager.add_connection(user_id, connection_id, sender).await.unwrap();
        
        // Check presence
        let room_id = RoomId::new();
        let presence = manager.get_room_presence(room_id).await.unwrap();
        assert!(presence.is_empty()); // No room membership yet
        
        // Remove connection
        manager.remove_connection(connection_id).await.unwrap();
        
        // Should fail to remove again
        assert!(manager.remove_connection(connection_id).await.is_err());
    }
    
    #[tokio::test]
    async fn test_broadcast_caching() {
        let manager = OptimizedConnectionManager::new(None);
        let room_id = RoomId::new();
        
        let message = WebSocketMessage::PresenceUpdate {
            room_id,
            online_users: vec![UserId::new()],
        };
        
        // First broadcast should miss cache
        let result = manager.broadcast_to_room(room_id, message.clone()).await;
        // Should fail with no connections, but cache should be populated
        assert!(result.is_err());
        
        // Cache should now contain the serialized message
        assert!(manager.broadcast_cache.get(&format!("{}:{}", 2u8, room_id.0)).await.is_some());
    }
    
    #[tokio::test]
    async fn test_connection_limits() {
        let config = ConnectionManagerConfig {
            max_connections_per_user: 2,
            ..Default::default()
        };
        
        let manager = OptimizedConnectionManager::new(Some(config));
        let user_id = UserId::new();
        
        // Add first connection
        let (sender1, _) = mpsc::unbounded_channel();
        manager.add_connection(user_id, ConnectionId::new(), sender1).await.unwrap();
        
        // Add second connection
        let (sender2, _) = mpsc::unbounded_channel();
        manager.add_connection(user_id, ConnectionId::new(), sender2).await.unwrap();
        
        // Third connection should fail
        let (sender3, _) = mpsc::unbounded_channel();
        let result = manager.add_connection(user_id, ConnectionId::new(), sender3).await;
        assert!(result.is_err());
    }
}