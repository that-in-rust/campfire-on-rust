use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

use crate::database::CampfireDatabase;
use crate::errors::RoomError;
use crate::models::{Room, RoomId, RoomType, UserId, InvolvementLevel, Membership};

/// Room Service trait defining the contract for room management operations
/// 
/// # Preconditions
/// - All user IDs must reference existing users in the database
/// - Room names must be 1-100 characters long
/// - Authorization checks must be performed before operations
/// 
/// # Postconditions  
/// - Room creation adds creator as admin member
/// - Member addition creates membership record
/// - Access checks return current involvement level or None
/// - User rooms are ordered by last activity
/// 
/// # Error Conditions
/// - RoomError::NotFound if room doesn't exist
/// - RoomError::NotAuthorized if user lacks permissions
/// - RoomError::AlreadyMember if user is already a member
/// - RoomError::InvalidName if room name is invalid
/// - RoomError::Database on persistence failure
#[async_trait]
pub trait RoomServiceTrait: Send + Sync {
    /// Creates a new room with the creator as admin
    async fn create_room(
        &self,
        name: String,
        topic: Option<String>,
        room_type: RoomType,
        creator_id: UserId,
    ) -> Result<Room, RoomError>;
    
    /// Adds user to room with proper authorization checks
    async fn add_member(
        &self,
        room_id: RoomId,
        user_id: UserId,
        added_by: UserId,
        involvement_level: InvolvementLevel,
    ) -> Result<(), RoomError>;
    
    /// Checks if user has access to room and returns involvement level
    async fn check_room_access(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<Option<InvolvementLevel>, RoomError>;
    
    /// Gets all rooms for a user, ordered by activity
    async fn get_user_rooms(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Room>, RoomError>;
    
    /// Gets a room by ID
    async fn get_room_by_id(
        &self,
        room_id: RoomId,
    ) -> Result<Option<Room>, RoomError>;
}

#[derive(Clone)]
pub struct RoomService {
    db: Arc<CampfireDatabase>,
}

impl RoomService {
    pub fn new(db: Arc<CampfireDatabase>) -> Self {
        Self { db }
    }
    
    /// Validates room name according to business rules
    fn validate_room_name(name: &str) -> Result<(), RoomError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(RoomError::InvalidName {
                reason: "Room name cannot be empty".to_string(),
            });
        }
        if trimmed.len() > 100 {
            return Err(RoomError::InvalidName {
                reason: format!("Room name too long: {} chars (max: 100)", trimmed.len()),
            });
        }
        Ok(())
    }
}

#[async_trait]
impl RoomServiceTrait for RoomService {
    async fn create_room(
        &self,
        name: String,
        topic: Option<String>,
        room_type: RoomType,
        creator_id: UserId,
    ) -> Result<Room, RoomError> {
        // Validate room name
        Self::validate_room_name(&name)?;
        
        // Validate topic length if provided
        if let Some(ref topic) = topic {
            if topic.len() > 500 {
                return Err(RoomError::InvalidName {
                    reason: format!("Topic too long: {} chars (max: 500)", topic.len()),
                });
            }
        }
        
        // Check if creator exists
        if !self.db.user_exists(creator_id).await? {
            return Err(RoomError::Database(
                sqlx::Error::RowNotFound
            ));
        }
        
        let now = Utc::now();
        let room = Room {
            id: RoomId::new(),
            name: name.trim().to_string(),
            topic: topic.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()),
            room_type,
            created_at: now,
            last_message_at: None,
        };
        
        // Create room in database
        self.db.create_room(room.clone()).await?;
        
        // Add creator as admin member
        let membership = Membership {
            room_id: room.id,
            user_id: creator_id,
            involvement_level: InvolvementLevel::Admin,
            created_at: now,
        };
        
        self.db.create_membership(membership).await?;
        
        Ok(room)
    }
    
    async fn add_member(
        &self,
        room_id: RoomId,
        user_id: UserId,
        added_by: UserId,
        involvement_level: InvolvementLevel,
    ) -> Result<(), RoomError> {
        // Check if room exists
        let room = self.db.get_room_by_id(room_id).await?;
        if room.is_none() {
            return Err(RoomError::NotFound { room_id });
        }
        
        // Check if user to be added exists
        if !self.db.user_exists(user_id).await? {
            return Err(RoomError::Database(
                sqlx::Error::RowNotFound
            ));
        }
        
        // Check if user is already a member
        if let Some(_) = self.db.get_membership(room_id, user_id).await? {
            return Err(RoomError::AlreadyMember { user_id, room_id });
        }
        
        // Check authorization - user must be able to add members
        if !self.db.check_user_can_add_member(room_id, added_by).await? {
            return Err(RoomError::NotAuthorized { 
                user_id: added_by, 
                room_id 
            });
        }
        
        // Create membership
        let membership = Membership {
            room_id,
            user_id,
            involvement_level,
            created_at: Utc::now(),
        };
        
        self.db.create_membership(membership).await?;
        
        Ok(())
    }
    
    async fn check_room_access(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<Option<InvolvementLevel>, RoomError> {
        // Check if room exists
        let room = self.db.get_room_by_id(room_id).await?;
        if room.is_none() {
            return Err(RoomError::NotFound { room_id });
        }
        let room = room.unwrap();
        
        // For open rooms, anyone can access as member if they're not already a member
        if matches!(room.room_type, RoomType::Open) {
            // Check if user has explicit membership
            if let Some(membership) = self.db.get_membership(room_id, user_id).await? {
                return Ok(Some(membership.involvement_level));
            }
            // For open rooms, non-members can access as implicit members
            return Ok(Some(InvolvementLevel::Member));
        }
        
        // For closed and direct rooms, check explicit membership
        if let Some(membership) = self.db.get_membership(room_id, user_id).await? {
            Ok(Some(membership.involvement_level))
        } else {
            Ok(None)
        }
    }
    
    async fn get_user_rooms(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Room>, RoomError> {
        // Check if user exists
        if !self.db.user_exists(user_id).await? {
            return Err(RoomError::Database(
                sqlx::Error::RowNotFound
            ));
        }
        
        // Get rooms where user is a member
        let rooms = self.db.get_user_rooms(user_id).await?;
        
        Ok(rooms)
    }
    
    async fn get_room_by_id(
        &self,
        room_id: RoomId,
    ) -> Result<Option<Room>, RoomError> {
        Ok(self.db.get_room_by_id(room_id).await?)
    }
}