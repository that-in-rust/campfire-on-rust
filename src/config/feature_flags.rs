use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub files_enabled: bool,        // MVP: false
    pub avatars_enabled: bool,      // MVP: false  
    pub opengraph_enabled: bool,    // MVP: false
    pub max_file_size: usize,       // MVP: 0
    pub search_enabled: bool,       // MVP: true
    pub push_notifications: bool,   // MVP: true
    pub bot_integrations: bool,     // MVP: true
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            files_enabled: false,
            avatars_enabled: false,
            opengraph_enabled: false,
            max_file_size: 0,
            search_enabled: true,
            push_notifications: true,
            bot_integrations: true,
        }
    }
}