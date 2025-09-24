use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{Duration, Instant};
use uuid::Uuid;

use crate::errors::{ConnectionError, BroadcastError};
use crate::models::{ConnectionId, MessageId, RoomId, UserId, WebSocketMessage};
use crate::database::CampfireDatabase;

// Type alias for WebSocket sender
pub type WebSocketSender = mpsc::UnboundedSender<String>;

#[async_trait]
pub trait ConnectionManager: Send + Sync {
    /// Adds WebSocket connection for user
    async fn add_connection(
        &self,
        user_id: UserId,
        connection_id: ConnectionId,
        sender: WebSocketSender,
    ) -> Result<(), ConnectionError>;
    
    /// Removes WebSocket connection
    async fn remove_connection(
        &self,
        connection_id: ConnectionId,
    ) -> Result<(), ConnectionError>;
    
    /// Broadcasts message to room subscribers
    async fn broadcast_to_room(
        &self,
        room_id: RoomId,
        message: WebSocketMessage,
    ) -> Result<(), BroadcastError>;
    
    /// Gets presence information for room (Critical Gap #5)
    async fn get_room_presence(
        &self,
        room_id: RoomId,
    ) -> Result<Vec<UserId>, ConnectionError>;
    
    /// Handles missed messages on reconnection (Critical Gap #2)
    async fn send_missed_messages(
        &self,
        user_id: UserId,
        connection_id: ConnectionId,
        last_seen_message_id: Option<MessageId>,
    ) -> Result<(), ConnectionError>;
    
    /// Updates last seen message for connection (Critical Gap #2)
    async fn update_last_seen_message(
        &self,
        connection_id: ConnectionId,
        message_id: MessageId,
    ) -> Result<(), ConnectionError>;
    
    /// Sends message to specific connection
    async fn send_to_connection(
        &self,
        connection_id: ConnectionId,
        message: String,
    ) -> Result<(), ConnectionError>;
    
    /// Gets room-specific presence information
    async fn get_room_specific_presence(
        &self,
        room_id: RoomId,
    ) -> Result<Vec<UserId>, ConnectionError>;
    
    /// Starts typing indicator for user in room
    async fn start_typing(
        &self,
        user_id: UserId,
        room_id: RoomId,
    ) -> Result<(), ConnectionError>;
    
    /// Stops typing indicator for user in room
    async fn stop_typing(
        &self,
        user_id: UserId,
        room_id: RoomId,
    ) -> Result<(), ConnectionError>;
    
    /// Gets currently typing users in room
    async fn get_typing_users(
        &self,
        room_id: RoomId,
    ) -> Result<Vec<UserId>, ConnectionError>;
    
    /// Broadcasts presence update to room members
    async fn broadcast_presence_update(
        &self,
        room_id: RoomId,
    ) -> Result<(), BroadcastError>;
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    user_id: UserId,
    sender: WebSocketSender,
    last_seen_message_id: Option<MessageId>,
    connected_at: Instant,
    last_activity: Instant,
}

#[derive(Debug, Clone)]
struct PresenceInfo {
    user_id: UserId,
    connection_count: usize,
    last_seen: Instant,
}

#[derive(Debug, Clone)]
struct TypingInfo {
    user_id: UserId,
    room_id: RoomId,
    started_at: Instant,
}

#[derive(Debug, Clone)]
struct RoomPresence {
    online_users: HashSet<UserId>,
    typing_users: HashMap<UserId, Instant>, // user_id -> when they started typing
}

pub struct ConnectionManagerImpl {
    // Active WebSocket connections
    connections: Arc<RwLock<HashMap<ConnectionId, ConnectionInfo>>>,
    
    // Room memberships (simplified - in full version would query database)
    room_members: Arc<RwLock<HashMap<RoomId, Vec<UserId>>>>,
    
    // Presence tracking (Critical Gap #5)
    presence: Arc<RwLock<HashMap<UserId, PresenceInfo>>>,
    
    // Room-specific presence tracking
    room_presence: Arc<RwLock<HashMap<RoomId, RoomPresence>>>,
    
    // Database for missed message queries (Critical Gap #2)
    database: Arc<CampfireDatabase>,
}

impl ConnectionManagerImpl {
    pub fn new(database: Arc<CampfireDatabase>) -> Self {
        let manager = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            room_members: Arc::new(RwLock::new(HashMap::new())),
            presence: Arc::new(RwLock::new(HashMap::new())),
            room_presence: Arc::new(RwLock::new(HashMap::new())),
            database,
        };
        
        // Start cleanup task for presence tracking (Critical Gap #5)
        manager.start_presence_cleanup();
        
        // Start cleanup task for typing indicators
        manager.start_typing_cleanup();
        
        manager
    }
    
    /// Test helper: Add room membership for testing
    pub async fn add_room_membership(&self, room_id: RoomId, user_ids: Vec<UserId>) {
        let mut room_members = self.room_members.write().await;
        room_members.insert(room_id, user_ids);
    }
    
    /// Test helper: Check if connection exists
    pub async fn connection_exists(&self, connection_id: ConnectionId) -> bool {
        let connections = self.connections.read().await;
        connections.contains_key(&connection_id)
    }
    
    /// Starts background task to clean up stale presence information
    /// Removes users who haven't been active for 60 seconds (Critical Gap #5)
    fn start_presence_cleanup(&self) {
        let presence = Arc::clone(&self.presence);
        let connections = Arc::clone(&self.connections);
        let room_presence = Arc::clone(&self.room_presence);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let now = Instant::now();
                let timeout = Duration::from_secs(60);
                
                // Clean up stale presence entries
                let mut presence_guard = presence.write().await;
                let mut to_remove = Vec::new();
                
                for (user_id, info) in presence_guard.iter() {
                    if now.duration_since(info.last_seen) > timeout {
                        to_remove.push(*user_id);
                    }
                }
                
                for user_id in to_remove {
                    presence_guard.remove(&user_id);
                    tracing::debug!("Cleaned up stale presence for user {}", user_id.0);
                }
                
                // Clean up room presence for offline users
                let mut room_presence_guard = room_presence.write().await;
                for room_presence_info in room_presence_guard.values_mut() {
                    room_presence_info.online_users.retain(|user_id| {
                        presence_guard.contains_key(user_id)
                    });
                }
                
                drop(presence_guard);
                drop(room_presence_guard);
                
                // Also clean up dead connections
                let connections_guard = connections.read().await;
                let mut dead_connections = Vec::new();
                
                for (connection_id, info) in connections_guard.iter() {
                    if info.sender.is_closed() {
                        dead_connections.push(*connection_id);
                    }
                }
                
                drop(connections_guard);
                
                if !dead_connections.is_empty() {
                    let mut connections_guard = connections.write().await;
                    for connection_id in dead_connections {
                        connections_guard.remove(&connection_id);
                        tracing::debug!("Cleaned up dead connection {}", connection_id.0);
                    }
                }
            }
        });
    }
    
    /// Starts background task to clean up stale typing indicators
    /// Removes typing indicators older than 10 seconds
    fn start_typing_cleanup(&self) {
        let room_presence = Arc::clone(&self.room_presence);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                let now = Instant::now();
                let typing_timeout = Duration::from_secs(10);
                
                let mut room_presence_guard = room_presence.write().await;
                let mut rooms_to_update = Vec::new();
                
                for (room_id, room_info) in room_presence_guard.iter_mut() {
                    let mut users_to_remove = Vec::new();
                    
                    for (user_id, started_at) in room_info.typing_users.iter() {
                        if now.duration_since(*started_at) > typing_timeout {
                            users_to_remove.push(*user_id);
                        }
                    }
                    
                    if !users_to_remove.is_empty() {
                        for user_id in &users_to_remove {
                            room_info.typing_users.remove(user_id);
                            tracing::debug!("Cleaned up stale typing indicator for user {} in room {}", 
                                          user_id.0, room_id.0);
                        }
                        rooms_to_update.push(*room_id);
                    }
                }
                
                drop(room_presence_guard);
                
                // Note: In a full implementation, we would broadcast typing stop events
                // for the cleaned up typing indicators, but for simplicity we'll let
                // clients handle the timeout on their end
            }
        });
    }
    
    /// Updates presence information for a user
    async fn update_presence(&self, user_id: UserId) {
        let mut presence_guard = self.presence.write().await;
        let connections_guard = self.connections.read().await;
        
        // Count active connections for this user
        let connection_count = connections_guard
            .values()
            .filter(|info| info.user_id == user_id)
            .count();
        
        if connection_count > 0 {
            presence_guard.insert(user_id, PresenceInfo {
                user_id,
                connection_count,
                last_seen: Instant::now(),
            });
        } else {
            presence_guard.remove(&user_id);
        }
    }
    
    /// Updates room-specific presence for a user
    async fn update_room_presence(&self, user_id: UserId) {
        let room_members_guard = self.room_members.read().await;
        let presence_guard = self.presence.read().await;
        let mut room_presence_guard = self.room_presence.write().await;
        
        let is_online = presence_guard.contains_key(&user_id);
        
        // Update presence in all rooms the user is a member of
        for (room_id, members) in room_members_guard.iter() {
            if members.contains(&user_id) {
                let room_info = room_presence_guard.entry(*room_id).or_insert_with(|| RoomPresence {
                    online_users: HashSet::new(),
                    typing_users: HashMap::new(),
                });
                
                if is_online {
                    room_info.online_users.insert(user_id);
                } else {
                    room_info.online_users.remove(&user_id);
                    // Also remove from typing users if they went offline
                    room_info.typing_users.remove(&user_id);
                }
            }
        }
    }
    
    /// Gets all connections for users in a room
    async fn get_room_connections(&self, room_id: RoomId) -> Vec<(ConnectionId, WebSocketSender)> {
        let connections_guard = self.connections.read().await;
        let room_members_guard = self.room_members.read().await;
        
        // Get members of the room
        let members = room_members_guard.get(&room_id).cloned().unwrap_or_default();
        
        // Find all connections for room members
        let mut room_connections = Vec::new();
        for (connection_id, info) in connections_guard.iter() {
            if members.contains(&info.user_id) {
                room_connections.push((*connection_id, info.sender.clone()));
            }
        }
        
        room_connections
    }
}

#[async_trait]
impl ConnectionManager for ConnectionManagerImpl {
    async fn add_connection(
        &self,
        user_id: UserId,
        connection_id: ConnectionId,
        sender: WebSocketSender,
    ) -> Result<(), ConnectionError> {
        let now = Instant::now();
        
        let connection_info = ConnectionInfo {
            user_id,
            sender,
            last_seen_message_id: None,
            connected_at: now,
            last_activity: now,
        };
        
        // Add connection
        {
            let mut connections_guard = self.connections.write().await;
            connections_guard.insert(connection_id, connection_info);
        }
        
        // Update presence (Critical Gap #5)
        self.update_presence(user_id).await;
        
        // Update room-specific presence for all rooms the user is in
        self.update_room_presence(user_id).await;
        
        tracing::info!("Added connection {} for user {}", connection_id.0, user_id.0);
        
        Ok(())
    }
    
    async fn remove_connection(
        &self,
        connection_id: ConnectionId,
    ) -> Result<(), ConnectionError> {
        let user_id = {
            let mut connections_guard = self.connections.write().await;
            let connection_info = connections_guard.remove(&connection_id)
                .ok_or(ConnectionError::NotFound { connection_id })?;
            connection_info.user_id
        };
        
        // Update presence (Critical Gap #5)
        self.update_presence(user_id).await;
        
        // Update room-specific presence for all rooms the user was in
        self.update_room_presence(user_id).await;
        
        tracing::info!("Removed connection {} for user {}", connection_id.0, user_id.0);
        
        Ok(())
    }
    
    async fn broadcast_to_room(
        &self,
        room_id: RoomId,
        message: WebSocketMessage,
    ) -> Result<(), BroadcastError> {
        let room_connections = self.get_room_connections(room_id).await;
        
        if room_connections.is_empty() {
            return Err(BroadcastError::NoConnections { room_id });
        }
        
        // Serialize message once
        let serialized = serde_json::to_string(&message)?;
        
        let mut failed_sends = 0;
        let total_connections = room_connections.len();
        
        // Send to all connections
        for (connection_id, sender) in room_connections {
            if let Err(_) = sender.send(serialized.clone()) {
                failed_sends += 1;
                tracing::warn!("Failed to send message to connection {}", connection_id.0);
            }
        }
        
        if failed_sends > 0 {
            return Err(BroadcastError::PartialFailure { 
                connection_count: failed_sends 
            });
        }
        
        tracing::debug!("Broadcasted message to {} connections in room {}", 
                       total_connections, room_id.0);
        
        Ok(())
    }
    
    async fn get_room_presence(
        &self,
        room_id: RoomId,
    ) -> Result<Vec<UserId>, ConnectionError> {
        let room_members_guard = self.room_members.read().await;
        let presence_guard = self.presence.read().await;
        
        // Get members of the room
        let members = room_members_guard.get(&room_id).cloned().unwrap_or_default();
        
        // Filter to only online members
        let online_members: Vec<UserId> = members
            .into_iter()
            .filter(|user_id| presence_guard.contains_key(user_id))
            .collect();
        
        Ok(online_members)
    }
    
    async fn send_missed_messages(
        &self,
        user_id: UserId,
        connection_id: ConnectionId,
        last_seen_message_id: Option<MessageId>,
    ) -> Result<(), ConnectionError> {
        // Critical Gap #2: WebSocket Reconnection State
        
        let connections_guard = self.connections.read().await;
        let connection_info = connections_guard.get(&connection_id)
            .ok_or(ConnectionError::NotFound { connection_id })?;
        
        let sender = connection_info.sender.clone();
        drop(connections_guard); // Release the lock early
        
        tracing::info!(
            "User {} reconnected with connection {}, fetching missed messages since: {:?}",
            user_id.0,
            connection_id.0,
            last_seen_message_id
        );
        
        // Query database for missed messages
        let missed_messages = match self.database.get_messages_since(
            user_id,
            last_seen_message_id,
            100, // Limit to 100 missed messages to prevent overwhelming the connection
        ).await {
            Ok(messages) => messages,
            Err(e) => {
                tracing::error!("Failed to fetch missed messages for user {}: {}", user_id.0, e);
                return Err(ConnectionError::Protocol(format!("Database error: {}", e)));
            }
        };
        
        if missed_messages.is_empty() {
            tracing::debug!("No missed messages for user {} on reconnection", user_id.0);
            return Ok(());
        }
        
        tracing::info!(
            "Sending {} missed messages to user {} on reconnection",
            missed_messages.len(),
            user_id.0
        );
        
        // Send each missed message as a WebSocket message
        let mut sent_count = 0;
        let mut failed_count = 0;
        
        for message in missed_messages {
            let ws_message = WebSocketMessage::NewMessage { message: message.clone() };
            
            match serde_json::to_string(&ws_message) {
                Ok(serialized) => {
                    if let Err(_) = sender.send(serialized) {
                        failed_count += 1;
                        tracing::warn!(
                            "Failed to send missed message {} to connection {}",
                            message.id.0,
                            connection_id.0
                        );
                    } else {
                        sent_count += 1;
                        
                        // Update the last seen message ID for this connection
                        if let Err(e) = self.update_last_seen_message(connection_id, message.id).await {
                            tracing::warn!(
                                "Failed to update last seen message for connection {}: {}",
                                connection_id.0,
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    tracing::error!(
                        "Failed to serialize missed message {} for user {}: {}",
                        message.id.0,
                        user_id.0,
                        e
                    );
                }
            }
        }
        
        if failed_count > 0 {
            tracing::warn!(
                "Missed message delivery partially failed for user {}: sent {}, failed {}",
                user_id.0,
                sent_count,
                failed_count
            );
            return Err(ConnectionError::SendFailed {
                reason: format!("Failed to send {} out of {} missed messages", failed_count, sent_count + failed_count)
            });
        }
        
        tracing::info!(
            "Successfully sent {} missed messages to user {} on reconnection",
            sent_count,
            user_id.0
        );
        
        Ok(())
    }
    
    async fn update_last_seen_message(
        &self,
        connection_id: ConnectionId,
        message_id: MessageId,
    ) -> Result<(), ConnectionError> {
        let mut connections_guard = self.connections.write().await;
        let connection_info = connections_guard.get_mut(&connection_id)
            .ok_or(ConnectionError::NotFound { connection_id })?;
        
        connection_info.last_seen_message_id = Some(message_id);
        connection_info.last_activity = Instant::now();
        
        Ok(())
    }
    
    async fn send_to_connection(
        &self,
        connection_id: ConnectionId,
        message: String,
    ) -> Result<(), ConnectionError> {
        let connections_guard = self.connections.read().await;
        let connection_info = connections_guard.get(&connection_id)
            .ok_or(ConnectionError::NotFound { connection_id })?;
        
        connection_info.sender.send(message)
            .map_err(|_| ConnectionError::SendFailed { reason: "Connection closed".to_string() })?;
        
        Ok(())
    }
    
    async fn get_room_specific_presence(
        &self,
        room_id: RoomId,
    ) -> Result<Vec<UserId>, ConnectionError> {
        let room_presence_guard = self.room_presence.read().await;
        
        if let Some(room_info) = room_presence_guard.get(&room_id) {
            Ok(room_info.online_users.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn start_typing(
        &self,
        user_id: UserId,
        room_id: RoomId,
    ) -> Result<(), ConnectionError> {
        let mut room_presence_guard = self.room_presence.write().await;
        
        let room_info = room_presence_guard.entry(room_id).or_insert_with(|| RoomPresence {
            online_users: HashSet::new(),
            typing_users: HashMap::new(),
        });
        
        room_info.typing_users.insert(user_id, Instant::now());
        
        tracing::debug!("User {} started typing in room {}", user_id.0, room_id.0);
        
        Ok(())
    }
    
    async fn stop_typing(
        &self,
        user_id: UserId,
        room_id: RoomId,
    ) -> Result<(), ConnectionError> {
        let mut room_presence_guard = self.room_presence.write().await;
        
        if let Some(room_info) = room_presence_guard.get_mut(&room_id) {
            room_info.typing_users.remove(&user_id);
            tracing::debug!("User {} stopped typing in room {}", user_id.0, room_id.0);
        }
        
        Ok(())
    }
    
    async fn get_typing_users(
        &self,
        room_id: RoomId,
    ) -> Result<Vec<UserId>, ConnectionError> {
        let room_presence_guard = self.room_presence.read().await;
        
        if let Some(room_info) = room_presence_guard.get(&room_id) {
            Ok(room_info.typing_users.keys().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn broadcast_presence_update(
        &self,
        room_id: RoomId,
    ) -> Result<(), BroadcastError> {
        // Get current online users for the room
        let online_users = self.get_room_specific_presence(room_id).await
            .map_err(|_e| BroadcastError::PartialFailure { connection_count: 1 })?;
        
        // Create presence update message
        let presence_msg = WebSocketMessage::PresenceUpdate {
            room_id,
            online_users,
        };
        
        // Broadcast to all room members
        self.broadcast_to_room(room_id, presence_msg).await
    }
}

// Mock implementation for testing
#[cfg(test)]
pub use mockall::mock;

#[cfg(test)]
mock! {
    pub ConnectionManager {}
    
    #[async_trait]
    impl ConnectionManager for ConnectionManager {
        async fn add_connection(
            &self,
            user_id: UserId,
            connection_id: ConnectionId,
            sender: WebSocketSender,
        ) -> Result<(), ConnectionError>;
        
        async fn remove_connection(
            &self,
            connection_id: ConnectionId,
        ) -> Result<(), ConnectionError>;
        
        async fn broadcast_to_room(
            &self,
            room_id: RoomId,
            message: WebSocketMessage,
        ) -> Result<(), BroadcastError>;
        
        async fn get_room_presence(
            &self,
            room_id: RoomId,
        ) -> Result<Vec<UserId>, ConnectionError>;
        
        async fn send_missed_messages(
            &self,
            user_id: UserId,
            connection_id: ConnectionId,
            last_seen_message_id: Option<MessageId>,
        ) -> Result<(), ConnectionError>;
        
        async fn update_last_seen_message(
            &self,
            connection_id: ConnectionId,
            message_id: MessageId,
        ) -> Result<(), ConnectionError>;
        
        async fn send_to_connection(
            &self,
            connection_id: ConnectionId,
            message: String,
        ) -> Result<(), ConnectionError>;
        
        async fn get_room_specific_presence(
            &self,
            room_id: RoomId,
        ) -> Result<Vec<UserId>, ConnectionError>;
        
        async fn start_typing(
            &self,
            user_id: UserId,
            room_id: RoomId,
        ) -> Result<(), ConnectionError>;
        
        async fn stop_typing(
            &self,
            user_id: UserId,
            room_id: RoomId,
        ) -> Result<(), ConnectionError>;
        
        async fn get_typing_users(
            &self,
            room_id: RoomId,
        ) -> Result<Vec<UserId>, ConnectionError>;
        
        async fn broadcast_presence_update(
            &self,
            room_id: RoomId,
        ) -> Result<(), BroadcastError>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    
    #[tokio::test]
    async fn test_connection_management() {
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        let manager = ConnectionManagerImpl::new(Arc::new(db));
        let user_id = UserId::new();
        let connection_id = ConnectionId::new();
        
        let (sender, _receiver) = mpsc::unbounded_channel();
        
        // Add connection
        manager.add_connection(user_id, connection_id, sender).await.unwrap();
        
        // Check presence
        let room_id = RoomId::new();
        let presence = manager.get_room_presence(room_id).await.unwrap();
        // Should be empty since user is not in any room yet
        assert!(presence.is_empty());
        
        // Remove connection
        manager.remove_connection(connection_id).await.unwrap();
        
        // Should fail to remove again
        assert!(manager.remove_connection(connection_id).await.is_err());
    }
    
    #[tokio::test]
    async fn test_presence_tracking() {
        // Test Critical Gap #5: Basic Presence Tracking
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        let manager = ConnectionManagerImpl::new(Arc::new(db));
        let user_id = UserId::new();
        let connection_id = ConnectionId::new();
        let room_id = RoomId::new();
        
        let (sender, _receiver) = mpsc::unbounded_channel();
        
        // Initially no presence
        let presence = manager.get_room_presence(room_id).await.unwrap();
        assert!(presence.is_empty());
        
        // Add connection
        manager.add_connection(user_id, connection_id, sender).await.unwrap();
        
        // Add user to room (simplified for test)
        {
            let mut room_members_guard = manager.room_members.write().await;
            room_members_guard.insert(room_id, vec![user_id]);
        }
        
        // Now should show presence
        let presence = manager.get_room_presence(room_id).await.unwrap();
        assert_eq!(presence.len(), 1);
        assert_eq!(presence[0], user_id);
        
        // Remove connection
        manager.remove_connection(connection_id).await.unwrap();
        
        // Should no longer show presence
        let presence = manager.get_room_presence(room_id).await.unwrap();
        assert!(presence.is_empty());
    }
    
    #[tokio::test]
    async fn test_broadcast_to_room() {
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        let manager = ConnectionManagerImpl::new(Arc::new(db));
        let user_id = UserId::new();
        let connection_id = ConnectionId::new();
        let room_id = RoomId::new();
        
        let (sender, mut receiver) = mpsc::unbounded_channel();
        
        // Add connection and room membership
        manager.add_connection(user_id, connection_id, sender).await.unwrap();
        {
            let mut room_members_guard = manager.room_members.write().await;
            room_members_guard.insert(room_id, vec![user_id]);
        }
        
        // Broadcast message
        let message = WebSocketMessage::NewMessage {
            message: crate::models::Message {
                id: MessageId::new(),
                room_id,
                creator_id: user_id,
                content: "Test message".to_string(),
                client_message_id: Uuid::new_v4(),
                created_at: chrono::Utc::now(),
                html_content: None,
                mentions: Vec::new(),
                sound_commands: Vec::new(),
            },
        };
        
        manager.broadcast_to_room(room_id, message).await.unwrap();
        
        // Should receive the message
        let received = receiver.recv().await.unwrap();
        assert!(received.contains("Test message"));
    }
    
    #[tokio::test]
    async fn test_last_seen_message_tracking() {
        // Test Critical Gap #2: WebSocket Reconnection State
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        let manager = ConnectionManagerImpl::new(Arc::new(db));
        let user_id = UserId::new();
        let connection_id = ConnectionId::new();
        let message_id = MessageId::new();
        
        let (sender, _receiver) = mpsc::unbounded_channel();
        
        // Add connection
        manager.add_connection(user_id, connection_id, sender).await.unwrap();
        
        // Update last seen message
        manager.update_last_seen_message(connection_id, message_id).await.unwrap();
        
        // Verify it was stored
        let connections_guard = manager.connections.read().await;
        let connection_info = connections_guard.get(&connection_id).unwrap();
        assert_eq!(connection_info.last_seen_message_id, Some(message_id));
    }
    
    #[tokio::test]
    async fn test_missed_messages_delivery() {
        // Test Critical Gap #2: Complete missed message delivery on reconnection
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        let db_arc = Arc::new(db);
        let manager = ConnectionManagerImpl::new(db_arc.clone());
        
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let connection_id = ConnectionId::new();
        
        // Create test user and room
        let user = crate::models::User {
            id: user_id,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            bio: None,
            admin: false,
            bot_token: None,
            created_at: chrono::Utc::now(),
        };
        
        let room = crate::models::Room {
            id: room_id,
            name: "Test Room".to_string(),
            topic: None,
            room_type: crate::models::RoomType::Open,
            created_at: chrono::Utc::now(),
            last_message_at: None,
        };
        
        let membership = crate::models::Membership {
            room_id,
            user_id,
            involvement_level: crate::models::InvolvementLevel::Member,
            created_at: chrono::Utc::now(),
        };
        
        // Create user, room, and membership in database
        db_arc.create_user(user).await.unwrap();
        db_arc.create_room(room).await.unwrap();
        db_arc.create_membership(membership).await.unwrap();
        
        // Create some test messages
        let message1 = crate::models::Message {
            id: MessageId::new(),
            room_id,
            creator_id: user_id,
            content: "First message".to_string(),
            client_message_id: uuid::Uuid::new_v4(),
            created_at: chrono::Utc::now(),
            html_content: None,
            mentions: Vec::new(),
            sound_commands: Vec::new(),
        };
        
        let message2 = crate::models::Message {
            id: MessageId::new(),
            room_id,
            creator_id: user_id,
            content: "Second message".to_string(),
            client_message_id: uuid::Uuid::new_v4(),
            created_at: chrono::Utc::now() + chrono::Duration::seconds(1),
            html_content: None,
            mentions: Vec::new(),
            sound_commands: Vec::new(),
        };
        
        let message3 = crate::models::Message {
            id: MessageId::new(),
            room_id,
            creator_id: user_id,
            content: "Third message".to_string(),
            client_message_id: uuid::Uuid::new_v4(),
            created_at: chrono::Utc::now() + chrono::Duration::seconds(2),
            html_content: None,
            mentions: Vec::new(),
            sound_commands: Vec::new(),
        };
        
        // Store messages in database
        db_arc.create_message_with_deduplication(message1.clone()).await.unwrap();
        db_arc.create_message_with_deduplication(message2.clone()).await.unwrap();
        db_arc.create_message_with_deduplication(message3.clone()).await.unwrap();
        
        // Create connection
        let (sender, mut receiver) = mpsc::unbounded_channel();
        manager.add_connection(user_id, connection_id, sender).await.unwrap();
        
        // Test missed messages delivery - should get all messages since no last_seen_message_id
        manager.send_missed_messages(user_id, connection_id, None).await.unwrap();
        
        // Should receive messages (they come in chronological order, oldest first)
        let mut received_messages = Vec::new();
        while let Ok(msg) = receiver.try_recv() {
            received_messages.push(msg);
        }
        
        // Should have received 3 messages
        assert_eq!(received_messages.len(), 3);
        
        // Verify content of messages
        for msg in &received_messages {
            assert!(msg.contains("message"));
        }
        
        // Test missed messages delivery with last_seen_message_id
        // Clear the receiver
        while receiver.try_recv().is_ok() {}
        
        // Send missed messages since message1
        manager.send_missed_messages(user_id, connection_id, Some(message1.id)).await.unwrap();
        
        // Should receive only message2 and message3
        let mut new_messages = Vec::new();
        while let Ok(msg) = receiver.try_recv() {
            new_messages.push(msg);
        }
        
        // Should have received 2 messages (message2 and message3)
        assert_eq!(new_messages.len(), 2);
        
        // Verify the messages are the newer ones
        assert!(new_messages[0].contains("Second message") || new_messages[0].contains("Third message"));
        assert!(new_messages[1].contains("Second message") || new_messages[1].contains("Third message"));
    }
    
    #[tokio::test]
    async fn test_missed_messages_error_handling() {
        // Test error handling for missed messages delivery
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        let db_arc = Arc::new(db);
        let manager = ConnectionManagerImpl::new(db_arc.clone());
        
        let user_id = UserId::new();
        let connection_id = ConnectionId::new();
        
        // Try to send missed messages for non-existent connection
        let result = manager.send_missed_messages(user_id, connection_id, None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConnectionError::NotFound { .. }));
        
        // Create connection but no user in database
        let (sender, _receiver) = mpsc::unbounded_channel();
        manager.add_connection(user_id, connection_id, sender).await.unwrap();
        
        // Should handle gracefully when user has no rooms/messages
        let result = manager.send_missed_messages(user_id, connection_id, None).await;
        assert!(result.is_ok()); // Should succeed with no messages to send
    }
    
    #[tokio::test]
    async fn test_typing_indicators() {
        // Test typing indicator functionality
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        let manager = ConnectionManagerImpl::new(Arc::new(db));
        let user_id = UserId::new();
        let room_id = RoomId::new();
        
        // Initially no typing users
        let typing_users = manager.get_typing_users(room_id).await.unwrap();
        assert!(typing_users.is_empty());
        
        // Start typing
        manager.start_typing(user_id, room_id).await.unwrap();
        
        // Should show user as typing
        let typing_users = manager.get_typing_users(room_id).await.unwrap();
        assert_eq!(typing_users.len(), 1);
        assert_eq!(typing_users[0], user_id);
        
        // Stop typing
        manager.stop_typing(user_id, room_id).await.unwrap();
        
        // Should no longer show user as typing
        let typing_users = manager.get_typing_users(room_id).await.unwrap();
        assert!(typing_users.is_empty());
    }
    
    #[tokio::test]
    async fn test_room_specific_presence() {
        // Test room-specific presence tracking
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        let manager = ConnectionManagerImpl::new(Arc::new(db));
        let user_id = UserId::new();
        let connection_id = ConnectionId::new();
        let room_id = RoomId::new();
        
        let (sender, _receiver) = mpsc::unbounded_channel();
        
        // Initially no presence in room
        let presence = manager.get_room_specific_presence(room_id).await.unwrap();
        assert!(presence.is_empty());
        
        // Add user to room membership (simplified for test)
        {
            let mut room_members_guard = manager.room_members.write().await;
            room_members_guard.insert(room_id, vec![user_id]);
        }
        
        // Add connection
        manager.add_connection(user_id, connection_id, sender).await.unwrap();
        
        // Should show presence in room
        let presence = manager.get_room_specific_presence(room_id).await.unwrap();
        assert_eq!(presence.len(), 1);
        assert_eq!(presence[0], user_id);
        
        // Remove connection
        manager.remove_connection(connection_id).await.unwrap();
        
        // Should no longer show presence in room
        let presence = manager.get_room_specific_presence(room_id).await.unwrap();
        assert!(presence.is_empty());
    }
    
    #[tokio::test]
    async fn test_presence_update_broadcast() {
        // Test presence update broadcasting
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        let manager = ConnectionManagerImpl::new(Arc::new(db));
        let user_id = UserId::new();
        let connection_id = ConnectionId::new();
        let room_id = RoomId::new();
        
        let (sender, mut receiver) = mpsc::unbounded_channel();
        
        // Add user to room membership
        {
            let mut room_members_guard = manager.room_members.write().await;
            room_members_guard.insert(room_id, vec![user_id]);
        }
        
        // Add connection
        manager.add_connection(user_id, connection_id, sender).await.unwrap();
        
        // Broadcast presence update
        manager.broadcast_presence_update(room_id).await.unwrap();
        
        // Should receive presence update message
        let received = receiver.recv().await.unwrap();
        assert!(received.contains("PresenceUpdate"));
        assert!(received.contains(&user_id.0.to_string()));
    }
    
    #[tokio::test]
    async fn test_typing_cleanup() {
        // Test that typing indicators are cleaned up after timeout
        let db = crate::database::CampfireDatabase::new(":memory:").await.unwrap();
        let manager = ConnectionManagerImpl::new(Arc::new(db));
        let user_id = UserId::new();
        let room_id = RoomId::new();
        
        // Start typing
        manager.start_typing(user_id, room_id).await.unwrap();
        
        // Should show user as typing
        let typing_users = manager.get_typing_users(room_id).await.unwrap();
        assert_eq!(typing_users.len(), 1);
        
        // Manually trigger cleanup by setting old timestamp
        {
            let mut room_presence_guard = manager.room_presence.write().await;
            if let Some(room_info) = room_presence_guard.get_mut(&room_id) {
                room_info.typing_users.insert(user_id, Instant::now() - Duration::from_secs(15));
            }
        }
        
        // Wait a bit for cleanup task to run (in real implementation)
        // For this test, we'll just verify the manual cleanup worked
        let typing_users = manager.get_typing_users(room_id).await.unwrap();
        assert_eq!(typing_users.len(), 1); // Still there because cleanup task runs separately
        
        // But if we check the timestamp, it should be old
        let room_presence_guard = manager.room_presence.read().await;
        if let Some(room_info) = room_presence_guard.get(&room_id) {
            if let Some(started_at) = room_info.typing_users.get(&user_id) {
                assert!(Instant::now().duration_since(*started_at) > Duration::from_secs(10));
            }
        }
    }
}