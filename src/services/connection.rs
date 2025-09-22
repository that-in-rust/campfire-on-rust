use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{Duration, Instant};
use uuid::Uuid;

use crate::errors::{ConnectionError, BroadcastError};
use crate::models::{ConnectionId, MessageId, RoomId, UserId, WebSocketMessage};

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

pub struct ConnectionManagerImpl {
    // Active WebSocket connections
    connections: Arc<RwLock<HashMap<ConnectionId, ConnectionInfo>>>,
    
    // Room memberships (simplified - in full version would query database)
    room_members: Arc<RwLock<HashMap<RoomId, Vec<UserId>>>>,
    
    // Presence tracking (Critical Gap #5)
    presence: Arc<RwLock<HashMap<UserId, PresenceInfo>>>,
}

impl ConnectionManagerImpl {
    pub fn new() -> Self {
        let manager = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            room_members: Arc::new(RwLock::new(HashMap::new())),
            presence: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Start cleanup task for presence tracking (Critical Gap #5)
        manager.start_presence_cleanup();
        
        manager
    }
    
    /// Starts background task to clean up stale presence information
    /// Removes users who haven't been active for 60 seconds (Critical Gap #5)
    fn start_presence_cleanup(&self) {
        let presence = Arc::clone(&self.presence);
        let connections = Arc::clone(&self.connections);
        
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
        
        // TODO: Implement missed message delivery
        // This would query the database for messages newer than last_seen_message_id
        // and send them to the connection
        
        // For now, just log the reconnection
        tracing::info!(
            "User {} reconnected with connection {}, last seen message: {:?}",
            user_id.0,
            connection_id.0,
            last_seen_message_id
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    
    #[tokio::test]
    async fn test_connection_management() {
        let manager = ConnectionManagerImpl::new();
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
        let manager = ConnectionManagerImpl::new();
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
        let manager = ConnectionManagerImpl::new();
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
        let manager = ConnectionManagerImpl::new();
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
}