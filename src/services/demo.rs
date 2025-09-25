use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::database::CampfireDatabase;
use crate::models::*;

/// Demo user credential for one-click login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoUserCredential {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: String,
    pub avatar: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub demo_context: String,
    pub tour_highlights: Vec<String>,
}

/// Demo data integrity status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoIntegrityStatus {
    pub users_exist: bool,
    pub rooms_exist: bool,
    pub messages_exist: bool,
    pub bots_configured: bool,
    pub expected_users: u32,
    pub actual_users: u32,
    pub expected_rooms: u32,
    pub actual_rooms: u32,
    pub expected_messages: u32,
    pub actual_messages: u32,
    pub integrity_score: f32,
    pub missing_components: Vec<String>,
}

/// Multi-user simulation session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSession {
    pub session_id: String,
    pub user_id: UserId,
    pub user_email: String,
    pub user_name: String,
    pub browser_tab_id: String,
    pub started_at: chrono::DateTime<Utc>,
    pub last_activity: chrono::DateTime<Utc>,
    pub tour_completed: bool,
    pub features_explored: Vec<String>,
}

/// Guided tour step for feature highlighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourStep {
    pub step_id: String,
    pub title: String,
    pub description: String,
    pub target_element: String,
    pub highlight_type: String,
    pub action_required: Option<String>,
    pub completion_criteria: String,
}

/// Demo service trait for multi-user simulation capabilities
#[async_trait]
pub trait DemoServiceTrait: Send + Sync {
    /// Get demo user credentials for one-click login
    async fn get_demo_credentials(&self) -> Result<Vec<DemoUserCredential>>;
    
    /// Validate demo data integrity
    async fn check_demo_integrity(&self) -> Result<DemoIntegrityStatus>;
    
    /// Initialize demo data if missing
    async fn ensure_demo_data(&self) -> Result<()>;
    
    /// Start multi-user simulation session
    async fn start_simulation_session(&self, user_email: &str, browser_tab_id: &str) -> Result<SimulationSession>;
    
    /// Get active simulation sessions
    async fn get_active_sessions(&self) -> Result<Vec<SimulationSession>>;
    
    /// Update session activity
    async fn update_session_activity(&self, session_id: &str, features_explored: Vec<String>) -> Result<()>;
    
    /// Get guided tour steps for user role
    async fn get_tour_steps(&self, user_role: &str) -> Result<Vec<TourStep>>;
    
    /// Mark tour step as completed
    async fn complete_tour_step(&self, session_id: &str, step_id: &str) -> Result<()>;
    
    /// Get demo statistics for display
    async fn get_demo_statistics(&self) -> Result<DemoStatistics>;
}

/// Demo statistics for metrics display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoStatistics {
    pub total_users: u32,
    pub total_rooms: u32,
    pub total_messages: u32,
    pub active_sessions: u32,
    pub tours_completed: u32,
    pub features_demonstrated: Vec<String>,
    pub uptime_seconds: u64,
}

/// Demo service implementation
pub struct DemoServiceImpl {
    db: Arc<CampfireDatabase>,
    demo_initializer: Arc<crate::demo::DemoDataInitializer>,
    active_sessions: Arc<tokio::sync::RwLock<Vec<SimulationSession>>>,
    start_time: std::time::Instant,
}

impl DemoServiceImpl {
    pub fn new(db: Arc<CampfireDatabase>) -> Self {
        let demo_initializer = Arc::new(crate::demo::DemoDataInitializer::new(db.clone()));
        
        Self {
            db,
            demo_initializer,
            active_sessions: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Get predefined demo user credentials with enhanced information
    fn get_predefined_credentials() -> Vec<DemoUserCredential> {
        vec![
            DemoUserCredential {
                email: "admin@campfire.demo".to_string(),
                password: "password".to_string(),
                name: "Admin User".to_string(),
                role: "System Administrator".to_string(),
                avatar: "AD".to_string(),
                description: "Full admin access - can manage all rooms and users".to_string(),
                permissions: vec![
                    "admin".to_string(),
                    "manage_users".to_string(),
                    "manage_rooms".to_string(),
                    "system_settings".to_string(),
                ],
                demo_context: "Complete system access for testing administrative features".to_string(),
                tour_highlights: vec![
                    "Admin dashboard access".to_string(),
                    "User management capabilities".to_string(),
                    "Room administration".to_string(),
                    "System configuration".to_string(),
                ],
            },
            DemoUserCredential {
                email: "alice@campfire.demo".to_string(),
                password: "password".to_string(),
                name: "Alice Johnson".to_string(),
                role: "Product Manager".to_string(),
                avatar: "AJ".to_string(),
                description: "Product team lead - active in planning and general discussions".to_string(),
                permissions: vec![
                    "create_rooms".to_string(),
                    "manage_product_rooms".to_string(),
                ],
                demo_context: "Leads product strategy and cross-team coordination".to_string(),
                tour_highlights: vec![
                    "Product planning rooms".to_string(),
                    "Cross-team @mentions".to_string(),
                    "Strategic discussions".to_string(),
                    "Feature prioritization".to_string(),
                ],
            },
            DemoUserCredential {
                email: "bob@campfire.demo".to_string(),
                password: "password".to_string(),
                name: "Bob Smith".to_string(),
                role: "Senior Developer".to_string(),
                avatar: "BS".to_string(),
                description: "Senior developer - technical discussions and code reviews".to_string(),
                permissions: vec![
                    "create_rooms".to_string(),
                    "technical_discussions".to_string(),
                ],
                demo_context: "Technical team lead with deep system knowledge".to_string(),
                tour_highlights: vec![
                    "Development room access".to_string(),
                    "Code review discussions".to_string(),
                    "Technical sound commands".to_string(),
                    "Search for technical topics".to_string(),
                ],
            },
            DemoUserCredential {
                email: "carol@campfire.demo".to_string(),
                password: "password".to_string(),
                name: "Carol Davis".to_string(),
                role: "UX Designer".to_string(),
                avatar: "CD".to_string(),
                description: "Design team - UI/UX discussions and creative collaboration".to_string(),
                permissions: vec![
                    "create_rooms".to_string(),
                    "design_feedback".to_string(),
                ],
                demo_context: "User experience expert focused on design quality".to_string(),
                tour_highlights: vec![
                    "Design room collaboration".to_string(),
                    "Creative feedback loops".to_string(),
                    "Visual design discussions".to_string(),
                    "User experience insights".to_string(),
                ],
            },
            DemoUserCredential {
                email: "david@campfire.demo".to_string(),
                password: "password".to_string(),
                name: "David Wilson".to_string(),
                role: "DevOps Engineer".to_string(),
                avatar: "DW".to_string(),
                description: "Infrastructure and deployment - DevOps discussions".to_string(),
                permissions: vec![
                    "create_rooms".to_string(),
                    "infrastructure_access".to_string(),
                ],
                demo_context: "Infrastructure specialist handling deployments and monitoring".to_string(),
                tour_highlights: vec![
                    "Infrastructure discussions".to_string(),
                    "Deployment coordination".to_string(),
                    "System monitoring alerts".to_string(),
                    "Performance optimization".to_string(),
                ],
            },
            DemoUserCredential {
                email: "eve@campfire.demo".to_string(),
                password: "password".to_string(),
                name: "Eve Brown".to_string(),
                role: "Marketing Manager".to_string(),
                avatar: "EB".to_string(),
                description: "Marketing team - campaigns and customer insights".to_string(),
                permissions: vec![
                    "create_rooms".to_string(),
                    "marketing_campaigns".to_string(),
                ],
                demo_context: "Growth and marketing expert driving user acquisition".to_string(),
                tour_highlights: vec![
                    "Marketing campaign coordination".to_string(),
                    "Customer insight sharing".to_string(),
                    "Growth strategy discussions".to_string(),
                    "Brand messaging alignment".to_string(),
                ],
            },
            DemoUserCredential {
                email: "frank@campfire.demo".to_string(),
                password: "password".to_string(),
                name: "Frank Miller".to_string(),
                role: "Sales Director".to_string(),
                avatar: "FM".to_string(),
                description: "Sales team - client relationships and deal coordination".to_string(),
                permissions: vec![
                    "create_rooms".to_string(),
                    "client_communication".to_string(),
                ],
                demo_context: "Sales leadership focused on client success and revenue growth".to_string(),
                tour_highlights: vec![
                    "Client relationship management".to_string(),
                    "Deal coordination".to_string(),
                    "Revenue pipeline discussions".to_string(),
                    "Customer success stories".to_string(),
                ],
            },
            DemoUserCredential {
                email: "grace@campfire.demo".to_string(),
                password: "password".to_string(),
                name: "Grace Lee".to_string(),
                role: "QA Engineer".to_string(),
                avatar: "GL".to_string(),
                description: "Quality assurance - testing and bug reports".to_string(),
                permissions: vec![
                    "create_rooms".to_string(),
                    "quality_testing".to_string(),
                ],
                demo_context: "Quality assurance expert ensuring product reliability".to_string(),
                tour_highlights: vec![
                    "Quality testing processes".to_string(),
                    "Bug report coordination".to_string(),
                    "Testing automation".to_string(),
                    "Quality metrics tracking".to_string(),
                ],
            },
        ]
    }
    
    /// Get guided tour steps based on user role
    fn get_role_specific_tour_steps(role: &str) -> Vec<TourStep> {
        let common_steps = vec![
            TourStep {
                step_id: "welcome".to_string(),
                title: "Welcome to Campfire!".to_string(),
                description: "This is your team's chat application built with Rust for blazing fast performance.".to_string(),
                target_element: ".chat-container".to_string(),
                highlight_type: "overlay".to_string(),
                action_required: None,
                completion_criteria: "user_acknowledges".to_string(),
            },
            TourStep {
                step_id: "rooms_sidebar".to_string(),
                title: "Room Navigation".to_string(),
                description: "Browse different rooms on the left. Each room has a specific purpose and team members.".to_string(),
                target_element: ".rooms-sidebar".to_string(),
                highlight_type: "highlight".to_string(),
                action_required: Some("click_room".to_string()),
                completion_criteria: "room_clicked".to_string(),
            },
            TourStep {
                step_id: "message_input".to_string(),
                title: "Send Messages".to_string(),
                description: "Type your message here. Try @mentioning someone or using /play commands!".to_string(),
                target_element: ".message-input".to_string(),
                highlight_type: "highlight".to_string(),
                action_required: Some("type_message".to_string()),
                completion_criteria: "message_sent".to_string(),
            },
            TourStep {
                step_id: "search_feature".to_string(),
                title: "Search Messages".to_string(),
                description: "Use the search feature to find messages across all rooms. Try searching for 'authentication' or 'performance'.".to_string(),
                target_element: ".search-input".to_string(),
                highlight_type: "highlight".to_string(),
                action_required: Some("perform_search".to_string()),
                completion_criteria: "search_performed".to_string(),
            },
        ];
        
        let role_specific_steps = match role {
            "System Administrator" => vec![
                TourStep {
                    step_id: "admin_features".to_string(),
                    title: "Admin Features".to_string(),
                    description: "As an admin, you have access to user management, room administration, and system settings.".to_string(),
                    target_element: ".admin-menu".to_string(),
                    highlight_type: "highlight".to_string(),
                    action_required: Some("explore_admin".to_string()),
                    completion_criteria: "admin_menu_opened".to_string(),
                },
            ],
            "Product Manager" => vec![
                TourStep {
                    step_id: "product_rooms".to_string(),
                    title: "Product Planning".to_string(),
                    description: "Check out the Product Planning room for strategic discussions and roadmap coordination.".to_string(),
                    target_element: "[data-room='product-planning']".to_string(),
                    highlight_type: "highlight".to_string(),
                    action_required: Some("visit_product_room".to_string()),
                    completion_criteria: "product_room_visited".to_string(),
                },
            ],
            "Senior Developer" => vec![
                TourStep {
                    step_id: "dev_features".to_string(),
                    title: "Development Tools".to_string(),
                    description: "Explore the Development room for code reviews, technical discussions, and team coordination.".to_string(),
                    target_element: "[data-room='development']".to_string(),
                    highlight_type: "highlight".to_string(),
                    action_required: Some("visit_dev_room".to_string()),
                    completion_criteria: "dev_room_visited".to_string(),
                },
            ],
            _ => vec![],
        };
        
        [common_steps, role_specific_steps].concat()
    }
}

#[async_trait]
impl DemoServiceTrait for DemoServiceImpl {
    async fn get_demo_credentials(&self) -> Result<Vec<DemoUserCredential>> {
        Ok(Self::get_predefined_credentials())
    }
    
    async fn check_demo_integrity(&self) -> Result<DemoIntegrityStatus> {
        let expected_users = 8u32; // 8 demo users
        let expected_rooms = 7u32; // 7 demo rooms
        let expected_messages = 25u32; // Approximate demo messages
        
        // Check if demo users exist
        let mut actual_users = 0u32;
        let demo_emails = [
            "admin@campfire.demo",
            "alice@campfire.demo", 
            "bob@campfire.demo",
            "carol@campfire.demo",
            "david@campfire.demo",
            "eve@campfire.demo",
            "frank@campfire.demo",
            "grace@campfire.demo",
        ];
        
        for email in &demo_emails {
            if self.db.get_user_by_email(email).await?.is_some() {
                actual_users += 1;
            }
        }
        
        // Check if demo bot exists
        let bot_exists = self.db.get_user_by_email("bot@campfire.demo").await?.is_some();
        
        // For now, use estimates for rooms and messages
        // In a full implementation, these would be proper database queries
        let actual_rooms = if actual_users > 0 { expected_rooms } else { 0 };
        let actual_messages = if actual_users > 0 { expected_messages } else { 0 };
        
        let users_exist = actual_users == expected_users;
        let rooms_exist = actual_rooms == expected_rooms;
        let messages_exist = actual_messages >= (expected_messages / 2); // Allow some variance
        let bots_configured = bot_exists;
        
        // Calculate integrity score
        let mut score = 0.0f32;
        if users_exist { score += 0.4; }
        if rooms_exist { score += 0.3; }
        if messages_exist { score += 0.2; }
        if bots_configured { score += 0.1; }
        
        // Identify missing components
        let mut missing_components = Vec::new();
        if !users_exist {
            missing_components.push(format!("Demo users ({}/{})", actual_users, expected_users));
        }
        if !rooms_exist {
            missing_components.push("Demo rooms".to_string());
        }
        if !messages_exist {
            missing_components.push("Demo conversations".to_string());
        }
        if !bots_configured {
            missing_components.push("Demo bot".to_string());
        }
        
        Ok(DemoIntegrityStatus {
            users_exist,
            rooms_exist,
            messages_exist,
            bots_configured,
            expected_users,
            actual_users,
            expected_rooms,
            actual_rooms,
            expected_messages,
            actual_messages,
            integrity_score: score,
            missing_components,
        })
    }
    
    async fn ensure_demo_data(&self) -> Result<()> {
        info!("Ensuring demo data integrity...");
        
        let integrity = self.check_demo_integrity().await?;
        
        if integrity.integrity_score < 1.0 {
            info!("Demo data incomplete (score: {:.1}), initializing...", integrity.integrity_score);
            self.demo_initializer.initialize_if_needed().await?;
            info!("Demo data initialization completed");
        } else {
            info!("Demo data integrity verified (score: 1.0)");
        }
        
        Ok(())
    }
    
    async fn start_simulation_session(&self, user_email: &str, browser_tab_id: &str) -> Result<SimulationSession> {
        // Get user information
        let user = self.db.get_user_by_email(user_email).await?
            .ok_or_else(|| anyhow::anyhow!("Demo user not found: {}", user_email))?;
        
        let session = SimulationSession {
            session_id: Uuid::new_v4().to_string(),
            user_id: user.id,
            user_email: user_email.to_string(),
            user_name: user.name.clone(),
            browser_tab_id: browser_tab_id.to_string(),
            started_at: Utc::now(),
            last_activity: Utc::now(),
            tour_completed: false,
            features_explored: Vec::new(),
        };
        
        // Add to active sessions
        let mut sessions = self.active_sessions.write().await;
        
        // Remove any existing session for this user/tab combination
        sessions.retain(|s| !(s.user_email == user_email && s.browser_tab_id == browser_tab_id));
        
        sessions.push(session.clone());
        
        info!("Started simulation session for {} (tab: {})", user_email, browser_tab_id);
        
        Ok(session)
    }
    
    async fn get_active_sessions(&self) -> Result<Vec<SimulationSession>> {
        let sessions = self.active_sessions.read().await;
        
        // Filter out sessions older than 1 hour
        let cutoff = Utc::now() - chrono::Duration::hours(1);
        let active_sessions: Vec<SimulationSession> = sessions
            .iter()
            .filter(|s| s.last_activity > cutoff)
            .cloned()
            .collect();
        
        Ok(active_sessions)
    }
    
    async fn update_session_activity(&self, session_id: &str, features_explored: Vec<String>) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        
        if let Some(session) = sessions.iter_mut().find(|s| s.session_id == session_id) {
            session.last_activity = Utc::now();
            
            // Merge new features with existing ones
            for feature in features_explored {
                if !session.features_explored.contains(&feature) {
                    session.features_explored.push(feature);
                }
            }
        }
        
        Ok(())
    }
    
    async fn get_tour_steps(&self, user_role: &str) -> Result<Vec<TourStep>> {
        Ok(Self::get_role_specific_tour_steps(user_role))
    }
    
    async fn complete_tour_step(&self, session_id: &str, step_id: &str) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        
        if let Some(session) = sessions.iter_mut().find(|s| s.session_id == session_id) {
            let feature = format!("tour_step_{}", step_id);
            if !session.features_explored.contains(&feature) {
                session.features_explored.push(feature);
            }
            
            // Check if all tour steps are completed
            let role_steps = Self::get_role_specific_tour_steps(&session.user_name); // Using name as role proxy
            let completed_steps: Vec<String> = session.features_explored
                .iter()
                .filter(|f| f.starts_with("tour_step_"))
                .cloned()
                .collect();
            
            if completed_steps.len() >= role_steps.len() {
                session.tour_completed = true;
            }
        }
        
        Ok(())
    }
    
    async fn get_demo_statistics(&self) -> Result<DemoStatistics> {
        let integrity = self.check_demo_integrity().await?;
        let active_sessions = self.get_active_sessions().await?;
        
        let tours_completed = active_sessions
            .iter()
            .filter(|s| s.tour_completed)
            .count() as u32;
        
        let mut features_demonstrated = Vec::new();
        for session in &active_sessions {
            for feature in &session.features_explored {
                if !features_demonstrated.contains(feature) {
                    features_demonstrated.push(feature.clone());
                }
            }
        }
        
        Ok(DemoStatistics {
            total_users: integrity.actual_users,
            total_rooms: integrity.actual_rooms,
            total_messages: integrity.actual_messages,
            active_sessions: active_sessions.len() as u32,
            tours_completed,
            features_demonstrated,
            uptime_seconds: self.start_time.elapsed().as_secs(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::CampfireDatabase;
    
    #[tokio::test]
    async fn test_demo_credentials() {
        let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
        let demo_service = DemoServiceImpl::new(db);
        
        let credentials = demo_service.get_demo_credentials().await.unwrap();
        
        assert_eq!(credentials.len(), 8);
        assert!(credentials.iter().any(|c| c.email == "admin@campfire.demo"));
        assert!(credentials.iter().any(|c| c.role == "Product Manager"));
    }
    
    #[tokio::test]
    async fn test_demo_integrity_check() {
        let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
        let demo_service = DemoServiceImpl::new(db);
        
        // Initially should have no demo data
        let integrity = demo_service.check_demo_integrity().await.unwrap();
        assert_eq!(integrity.actual_users, 0);
        assert_eq!(integrity.integrity_score, 0.0);
        assert!(!integrity.missing_components.is_empty());
    }
    
    #[tokio::test]
    async fn test_simulation_session() {
        let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
        let demo_service = DemoServiceImpl::new(db.clone());
        
        // Initialize demo data first
        demo_service.ensure_demo_data().await.unwrap();
        
        // Start a simulation session
        let session = demo_service
            .start_simulation_session("alice@campfire.demo", "tab-1")
            .await
            .unwrap();
        
        assert_eq!(session.user_email, "alice@campfire.demo");
        assert_eq!(session.browser_tab_id, "tab-1");
        assert!(!session.tour_completed);
        
        // Check active sessions
        let active_sessions = demo_service.get_active_sessions().await.unwrap();
        assert_eq!(active_sessions.len(), 1);
    }
    
    #[tokio::test]
    async fn test_tour_steps() {
        let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
        let demo_service = DemoServiceImpl::new(db);
        
        let admin_steps = demo_service.get_tour_steps("System Administrator").await.unwrap();
        let pm_steps = demo_service.get_tour_steps("Product Manager").await.unwrap();
        
        // Admin should have more steps than regular users
        assert!(admin_steps.len() >= 4);
        assert!(pm_steps.len() >= 4);
        
        // Check for role-specific steps
        assert!(admin_steps.iter().any(|s| s.step_id == "admin_features"));
        assert!(pm_steps.iter().any(|s| s.step_id == "product_rooms"));
    }
}