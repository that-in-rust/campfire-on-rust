use anyhow::Result;
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::database::CampfireDatabase;
use crate::models::*;

/// Demo data initialization for offline demo mode
/// 
/// Creates a realistic chat environment with:
/// - Multiple users with different roles
/// - Various room types (open, closed, direct)
/// - Sample conversations that demonstrate features
/// - Bot integrations and sound commands
pub struct DemoDataInitializer {
    db: Arc<CampfireDatabase>,
}

impl DemoDataInitializer {
    pub fn new(db: Arc<CampfireDatabase>) -> Self {
        Self { db }
    }
    
    /// Initialize all demo data if not already present
    pub async fn initialize_if_needed(&self) -> Result<()> {
        // Check if demo data already exists
        if self.demo_data_exists().await? {
            info!("Demo data already exists, skipping initialization");
            return Ok(());
        }
        
        info!("Initializing demo data for offline experience...");
        
        // Create demo users
        let users = self.create_demo_users().await?;
        info!("Created {} demo users", users.len());
        
        // Create demo rooms
        let rooms = self.create_demo_rooms(&users).await?;
        info!("Created {} demo rooms", rooms.len());
        
        // Create sample conversations
        self.create_sample_conversations(&users, &rooms).await?;
        info!("Created sample conversations");
        
        info!("Demo data initialization complete!");
        Ok(())
    }
    
    /// Check if demo data already exists
    async fn demo_data_exists(&self) -> Result<bool> {
        // Check if the admin user exists
        let admin_exists = self.db.get_user_by_email("admin@campfire.demo").await?.is_some();
        Ok(admin_exists)
    }
    
    /// Create demo users with realistic profiles
    async fn create_demo_users(&self) -> Result<Vec<User>> {
        let mut users = Vec::new();
        
        // Demo users with realistic profiles
        let demo_users = vec![
            ("admin@campfire.demo", "Admin User", "System Administrator", true, "password"),
            ("alice@campfire.demo", "Alice Johnson", "Product Manager", false, "password"),
            ("bob@campfire.demo", "Bob Smith", "Senior Developer", false, "password"),
            ("carol@campfire.demo", "Carol Davis", "UX Designer", false, "password"),
            ("david@campfire.demo", "David Wilson", "DevOps Engineer", false, "password"),
            ("eve@campfire.demo", "Eve Brown", "Marketing Manager", false, "password"),
            ("frank@campfire.demo", "Frank Miller", "Sales Director", false, "password"),
            ("grace@campfire.demo", "Grace Lee", "QA Engineer", false, "password"),
        ];
        
        for (email, name, bio, is_admin, password) in demo_users {
            let password_hash = hash(password, DEFAULT_COST)?;
            
            let user = User {
                id: UserId::new(),
                name: name.to_string(),
                email: email.to_string(),
                password_hash,
                bio: Some(bio.to_string()),
                admin: is_admin,
                bot_token: None,
                created_at: Utc::now(),
            };
            
            self.db.create_user(user.clone()).await?;
            users.push(user);
        }
        
        // Create a demo bot user
        let bot_user = User {
            id: UserId::new(),
            name: "Demo Bot".to_string(),
            email: "bot@campfire.demo".to_string(),
            password_hash: hash("bot_password", DEFAULT_COST)?,
            bio: Some("Automated assistant for demo purposes".to_string()),
            admin: false,
            bot_token: Some("demo_bot_token_12345".to_string()),
            created_at: Utc::now(),
        };
        
        self.db.create_user(bot_user.clone()).await?;
        users.push(bot_user);
        
        Ok(users)
    }
    
    /// Create demo rooms with different types and purposes
    async fn create_demo_rooms(&self, users: &[User]) -> Result<Vec<Room>> {
        let mut rooms = Vec::new();
        
        // Find key users
        let admin = users.iter().find(|u| u.admin).unwrap();
        let alice = users.iter().find(|u| u.name == "Alice Johnson").unwrap();
        let bob = users.iter().find(|u| u.name == "Bob Smith").unwrap();
        let carol = users.iter().find(|u| u.name == "Carol Davis").unwrap();
        
        // Demo rooms with realistic purposes
        let demo_rooms = vec![
            ("General", "General discussion for the whole team", RoomType::Open),
            ("Development", "Development team coordination", RoomType::Open),
            ("Design", "Design team collaboration", RoomType::Open),
            ("Product Planning", "Product roadmap and planning", RoomType::Closed),
            ("Random", "Random chatter and fun stuff", RoomType::Open),
            ("Support", "Customer support coordination", RoomType::Closed),
            ("Marketing", "Marketing campaigns and ideas", RoomType::Open),
        ];
        
        for (name, topic, room_type) in demo_rooms {
            let room = Room {
                id: RoomId::new(),
                name: name.to_string(),
                topic: Some(topic.to_string()),
                room_type,
                created_at: Utc::now(),
                last_message_at: None,
            };
            
            self.db.create_room(room.clone()).await?;
            
            // Add memberships based on room type
            match room.room_type {
                RoomType::Open => {
                    // Add all users to open rooms
                    for user in users {
                        let membership = Membership {
                            room_id: room.id,
                            user_id: user.id,
                            involvement_level: if user.admin { 
                                InvolvementLevel::Admin 
                            } else { 
                                InvolvementLevel::Member 
                            },
                            created_at: Utc::now(),
                        };
                        self.db.create_membership(membership).await?;
                    }
                }
                RoomType::Closed => {
                    // Add specific users to closed rooms
                    let members = match name {
                        "Product Planning" => vec![admin, alice, bob],
                        "Support" => vec![admin, alice, carol],
                        _ => vec![admin, alice, bob, carol],
                    };
                    
                    for user in members {
                        let membership = Membership {
                            room_id: room.id,
                            user_id: user.id,
                            involvement_level: if user.admin { 
                                InvolvementLevel::Admin 
                            } else { 
                                InvolvementLevel::Member 
                            },
                            created_at: Utc::now(),
                        };
                        self.db.create_membership(membership).await?;
                    }
                }
                RoomType::Direct => {
                    // Direct rooms will be created separately
                }
            }
            
            rooms.push(room);
        }
        
        // Create a few direct message rooms
        let direct_pairs = vec![
            (alice, bob),
            (alice, carol),
            (bob, carol),
        ];
        
        for (user1, user2) in direct_pairs {
            let room = Room {
                id: RoomId::new(),
                name: format!("{} & {}", user1.name, user2.name),
                topic: None,
                room_type: RoomType::Direct,
                created_at: Utc::now(),
                last_message_at: None,
            };
            
            self.db.create_room(room.clone()).await?;
            
            // Add both users to the direct room
            for user in [user1, user2] {
                let membership = Membership {
                    room_id: room.id,
                    user_id: user.id,
                    involvement_level: InvolvementLevel::Member,
                    created_at: Utc::now(),
                };
                self.db.create_membership(membership).await?;
            }
            
            rooms.push(room);
        }
        
        Ok(rooms)
    }
    
    /// Create sample conversations that demonstrate features
    async fn create_sample_conversations(&self, users: &[User], rooms: &[Room]) -> Result<()> {
        // Find key users and rooms
        let admin = users.iter().find(|u| u.admin).unwrap();
        let alice = users.iter().find(|u| u.name == "Alice Johnson").unwrap();
        let bob = users.iter().find(|u| u.name == "Bob Smith").unwrap();
        let carol = users.iter().find(|u| u.name == "Carol Davis").unwrap();
        let bot = users.iter().find(|u| u.name == "Demo Bot").unwrap();
        
        let general_room = rooms.iter().find(|r| r.name == "General").unwrap();
        let dev_room = rooms.iter().find(|r| r.name == "Development").unwrap();
        let design_room = rooms.iter().find(|r| r.name == "Design").unwrap();
        let random_room = rooms.iter().find(|r| r.name == "Random").unwrap();
        
        // Welcome messages in General
        self.create_message(
            admin,
            general_room,
            "Welcome to Campfire! 🔥 This is our team chat where we collaborate and stay connected.",
        ).await?;
        
        self.create_message(
            admin,
            general_room,
            "Feel free to explore the different rooms and try out features like @mentions, /play sounds, and search!",
        ).await?;
        
        self.create_message(
            alice,
            general_room,
            "Thanks for setting this up! Looking forward to better team communication.",
        ).await?;
        
        // Development discussion
        self.create_message(
            bob,
            dev_room,
            "Just pushed the new authentication system. Ready for code review!",
        ).await?;
        
        self.create_message(
            alice,
            dev_room,
            "@bob Great work! I'll review it this afternoon. How's the performance looking?",
        ).await?;
        
        self.create_message(
            bob,
            dev_room,
            "@alice Performance is solid - response times under 100ms for login. Added comprehensive tests too.",
        ).await?;
        
        self.create_message(
            admin,
            dev_room,
            "Excellent! Security review passed as well. Let's deploy to staging. /play tada",
        ).await?;
        
        // Design collaboration
        self.create_message(
            carol,
            design_room,
            "New mockups for the dashboard are ready! The user flow is much cleaner now.",
        ).await?;
        
        self.create_message(
            alice,
            design_room,
            "@carol Love the new layout! The navigation feels much more intuitive.",
        ).await?;
        
        self.create_message(
            carol,
            design_room,
            "Thanks! I focused on reducing cognitive load. Users can now find what they need in 2 clicks max.",
        ).await?;
        
        // Fun conversation in Random
        self.create_message(
            bob,
            random_room,
            "Anyone else excited about the new Rust features in 1.75? /play yeah",
        ).await?;
        
        self.create_message(
            carol,
            random_room,
            "The async improvements look amazing! Our WebSocket performance should get even better.",
        ).await?;
        
        self.create_message(
            alice,
            random_room,
            "Speaking of performance, our chat app is blazing fast compared to Slack! 🚀",
        ).await?;
        
        self.create_message(
            admin,
            random_room,
            "That's the power of Rust! Memory safety AND performance. /play greatjob",
        ).await?;
        
        // Bot demonstration
        self.create_message(
            bot,
            general_room,
            "🤖 Demo Bot here! I can help with automated tasks and notifications. Try mentioning me with @bot!",
        ).await?;
        
        self.create_message(
            alice,
            general_room,
            "@bot What can you help us with?",
        ).await?;
        
        self.create_message(
            bot,
            general_room,
            "@alice I can send notifications, run automated reports, and integrate with external services. This is just a demo, but imagine the possibilities!",
        ).await?;
        
        // Sound system demonstration
        self.create_message(
            bob,
            random_room,
            "Let's test the sound system! /play horn",
        ).await?;
        
        self.create_message(
            carol,
            random_room,
            "Haha, that's fun! /play rimshot",
        ).await?;
        
        self.create_message(
            alice,
            random_room,
            "We have 59 different sounds! Try /play nyan for some nostalgia 😸",
        ).await?;
        
        // Search demonstration
        self.create_message(
            admin,
            general_room,
            "Pro tip: Use the search feature to find old conversations. Try searching for 'authentication' or 'performance'!",
        ).await?;
        
        self.create_message(
            bob,
            dev_room,
            "The full-text search is powered by SQLite FTS5 - super fast and accurate!",
        ).await?;
        
        Ok(())
    }
    
    /// Helper to create a message with rich text processing
    async fn create_message(&self, user: &User, room: &Room, content: &str) -> Result<()> {
        let message = Message {
            id: MessageId::new(),
            room_id: room.id,
            creator_id: user.id,
            content: content.to_string(),
            client_message_id: Uuid::new_v4(),
            created_at: Utc::now(),
            html_content: Some(self.process_rich_text(content)),
            mentions: self.extract_mentions(content),
            sound_commands: self.extract_sound_commands(content),
        };
        
        self.db.create_message_with_deduplication(message).await?;
        Ok(())
    }
    
    /// Process rich text formatting (basic implementation)
    fn process_rich_text(&self, content: &str) -> String {
        let mut html = html_escape::encode_text(content).to_string();
        
        // Convert @mentions to links
        html = regex::Regex::new(r"@(\w+)")
            .unwrap()
            .replace_all(&html, r#"<span class="mention">@$1</span>"#)
            .to_string();
        
        // Convert /play commands to sound links
        html = regex::Regex::new(r"/play (\w+)")
            .unwrap()
            .replace_all(&html, r#"<span class="sound-command">/play $1</span>"#)
            .to_string();
        
        html
    }
    
    /// Extract @mentions from message content
    fn extract_mentions(&self, content: &str) -> Vec<String> {
        regex::Regex::new(r"@(\w+)")
            .unwrap()
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect()
    }
    
    /// Extract /play sound commands from message content
    fn extract_sound_commands(&self, content: &str) -> Vec<String> {
        regex::Regex::new(r"/play (\w+)")
            .unwrap()
            .captures_iter(content)
            .map(|cap| cap[1].to_string())
            .collect()
    }
    
    /// Get demo user credentials for display
    pub fn get_demo_credentials() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("admin@campfire.demo", "password", "System Administrator"),
            ("alice@campfire.demo", "password", "Product Manager"),
            ("bob@campfire.demo", "password", "Senior Developer"),
            ("carol@campfire.demo", "password", "UX Designer"),
            ("david@campfire.demo", "password", "DevOps Engineer"),
            ("eve@campfire.demo", "password", "Marketing Manager"),
            ("frank@campfire.demo", "password", "Sales Director"),
            ("grace@campfire.demo", "password", "QA Engineer"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_demo_data_initialization() {
        let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
        let initializer = DemoDataInitializer::new(db.clone());
        
        // Should not exist initially
        assert!(!initializer.demo_data_exists().await.unwrap());
        
        // Initialize demo data
        initializer.initialize_if_needed().await.unwrap();
        
        // Should exist after initialization
        assert!(initializer.demo_data_exists().await.unwrap());
        
        // Should not reinitialize
        initializer.initialize_if_needed().await.unwrap();
        
        // Verify admin user exists
        let admin = db.get_user_by_email("admin@campfire.demo").await.unwrap();
        assert!(admin.is_some());
        assert!(admin.unwrap().admin);
    }
    
    #[tokio::test]
    async fn test_rich_text_processing() {
        let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
        let initializer = DemoDataInitializer::new(db);
        
        let content = "Hey @alice, check this out! /play tada";
        let html = initializer.process_rich_text(content);
        
        assert!(html.contains(r#"<span class="mention">@alice</span>"#));
        assert!(html.contains(r#"<span class="sound-command">/play tada</span>"#));
    }
    
    #[tokio::test]
    async fn test_mention_extraction() {
        let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
        let initializer = DemoDataInitializer::new(db);
        
        let content = "Hey @alice and @bob, what do you think?";
        let mentions = initializer.extract_mentions(content);
        
        assert_eq!(mentions, vec!["@alice", "@bob"]);
    }
    
    #[tokio::test]
    async fn test_sound_command_extraction() {
        let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
        let initializer = DemoDataInitializer::new(db);
        
        let content = "Great work! /play tada Let's celebrate /play yeah";
        let sounds = initializer.extract_sound_commands(content);
        
        assert_eq!(sounds, vec!["tada", "yeah"]);
    }
}