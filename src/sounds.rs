use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Embedded sound assets using rust-embed
/// 
/// All MP3 files are embedded at compile time for single binary deployment
#[derive(RustEmbed)]
#[folder = "assets/sounds/"]
pub struct SoundAssets;

/// Sound metadata for each embedded sound file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SoundInfo {
    pub name: String,
    pub filename: String,
    pub description: Option<String>,
}

/// Cache of sound metadata
static SOUND_INFO_CACHE: OnceLock<HashMap<String, SoundInfo>> = OnceLock::new();

/// Sound asset manager for Campfire
pub struct SoundManager;

impl SoundManager {
    /// Get embedded sound file data by name
    /// 
    /// # Arguments
    /// * `sound_name` - Name of the sound (without .mp3 extension)
    /// 
    /// # Returns
    /// Option containing the MP3 file data as bytes
    pub fn get_sound_data(sound_name: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
        let filename = format!("{}.mp3", sound_name);
        SoundAssets::get(&filename).map(|file| file.data)
    }
    
    /// Check if a sound exists in the embedded assets
    pub fn sound_exists(sound_name: &str) -> bool {
        let filename = format!("{}.mp3", sound_name);
        SoundAssets::get(&filename).is_some()
    }
    
    /// Get list of all available sound names
    pub fn available_sounds() -> Vec<String> {
        SoundAssets::iter()
            .filter_map(|filename| {
                if filename.ends_with(".mp3") {
                    Some(filename.trim_end_matches(".mp3").to_string())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get sound metadata
    pub fn get_sound_info(sound_name: &str) -> Option<&SoundInfo> {
        let cache = SOUND_INFO_CACHE.get_or_init(|| {
            Self::build_sound_info_cache()
        });
        
        cache.get(sound_name)
    }
    
    /// Get all sound metadata
    pub fn get_all_sound_info() -> &'static HashMap<String, SoundInfo> {
        SOUND_INFO_CACHE.get_or_init(|| {
            Self::build_sound_info_cache()
        })
    }
    
    /// Build the sound information cache with descriptions
    fn build_sound_info_cache() -> HashMap<String, SoundInfo> {
        let mut cache = HashMap::new();
        
        // Define sound descriptions (based on original Campfire)
        let descriptions = [
            ("56k", "Dial-up modem connection sound"),
            ("ballmer", "Steve Ballmer enthusiasm"),
            ("bell", "Simple notification bell"),
            ("bezos", "Jeff Bezos laugh"),
            ("bueller", "Bueller... Bueller..."),
            ("butts", "Beavis and Butt-Head laugh"),
            ("clowntown", "Circus/clown music"),
            ("cottoneyejoe", "Cotton Eye Joe snippet"),
            ("crickets", "Awkward silence crickets"),
            ("curb", "Curb Your Enthusiasm theme"),
            ("dadgummit", "Frustrated exclamation"),
            ("dangerzone", "Top Gun danger zone"),
            ("danielsan", "Karate Kid reference"),
            ("deeper", "We need to go deeper"),
            ("donotwant", "Do not want!"),
            ("drama", "Dramatic music sting"),
            ("flawless", "Flawless victory"),
            ("glados", "Portal GLaDOS voice"),
            ("gogogo", "Urgent go command"),
            ("greatjob", "Great job praise"),
            ("greyjoy", "Game of Thrones reference"),
            ("guarantee", "Men's Wearhouse guarantee"),
            ("heygirl", "Ryan Gosling hey girl"),
            ("honk", "Car horn honk"),
            ("horn", "Air horn blast"),
            ("horror", "Horror movie sting"),
            ("incoming", "Incoming alert"),
            ("inconceivable", "Princess Bride inconceivable"),
            ("letitgo", "Frozen let it go"),
            ("live", "Saturday Night Live"),
            ("loggins", "Kenny Loggins reference"),
            ("makeitso", "Star Trek make it so"),
            ("mario_coin", "Super Mario coin sound"),
            ("maybe", "Maybe response"),
            ("noooo", "Dramatic no scream"),
            ("nyan", "Nyan Cat music"),
            ("ohmy", "Oh my reaction"),
            ("ohyeah", "Oh yeah excitement"),
            ("pushit", "Push it real good"),
            ("rimshot", "Drum rimshot for jokes"),
            ("rollout", "Transformers roll out"),
            ("rumble", "Rumbling sound effect"),
            ("sax", "Sexy saxophone"),
            ("secret", "Secret sound"),
            ("sexyback", "Bringing sexy back"),
            ("story", "Cool story bro"),
            ("tada", "Ta-da celebration"),
            ("tmyk", "The More You Know"),
            ("totes", "Totally agreement"),
            ("trololo", "Russian trololo song"),
            ("trombone", "Sad trombone"),
            ("unix", "Unix system reference"),
            ("vuvuzela", "World Cup vuvuzela"),
            ("what", "Confused what reaction"),
            ("whoomp", "Whoomp there it is"),
            ("wups", "Oops mistake sound"),
            ("yay", "Celebration yay"),
            ("yeah", "Yeah agreement"),
            ("yodel", "Yodeling sound"),
        ];
        
        for (name, description) in descriptions {
            if Self::sound_exists(name) {
                cache.insert(name.to_string(), SoundInfo {
                    name: name.to_string(),
                    filename: format!("{}.mp3", name),
                    description: Some(description.to_string()),
                });
            }
        }
        
        // Add any sounds that don't have descriptions
        for sound_name in Self::available_sounds() {
            if !cache.contains_key(&sound_name) {
                cache.insert(sound_name.clone(), SoundInfo {
                    name: sound_name.clone(),
                    filename: format!("{}.mp3", sound_name),
                    description: None,
                });
            }
        }
        
        cache
    }
    
    /// Get the MIME type for sound files
    pub fn get_mime_type() -> &'static str {
        "audio/mpeg"
    }
    
    /// Get file size for a sound
    pub fn get_sound_size(sound_name: &str) -> Option<usize> {
        Self::get_sound_data(sound_name).map(|data| data.as_ref().len())
    }
    
    /// Validate sound name format
    pub fn is_valid_sound_name(name: &str) -> bool {
        // Sound names should be alphanumeric with underscores and hyphens
        name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            && !name.is_empty()
            && name.len() <= 50
    }
}

/// Sound playback request for WebSocket messages
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SoundPlayback {
    pub sound_name: String,
    pub triggered_by: crate::models::UserId,
    pub room_id: crate::models::RoomId,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl SoundPlayback {
    pub fn new(
        sound_name: String,
        triggered_by: crate::models::UserId,
        room_id: crate::models::RoomId,
    ) -> Self {
        Self {
            sound_name,
            triggered_by,
            room_id,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sound_assets_embedded() {
        // Test that we can access embedded sound files
        let sounds = SoundManager::available_sounds();
        assert!(!sounds.is_empty(), "Should have embedded sound files");
        
        // Test specific sounds that should exist
        assert!(sounds.contains(&"tada".to_string()));
        assert!(sounds.contains(&"bell".to_string()));
    }
    
    #[test]
    fn test_get_sound_data() {
        // Test getting sound data
        if let Some(data) = SoundManager::get_sound_data("tada") {
            let data_ref = data.as_ref();
            assert!(!data_ref.is_empty(), "Sound data should not be empty");
            
            // MP3 files should start with ID3 tag or MP3 frame sync
            assert!(
                data_ref.starts_with(b"ID3") || data_ref[0] == 0xFF,
                "Should be valid MP3 data"
            );
        }
    }
    
    #[test]
    fn test_sound_exists() {
        assert!(SoundManager::sound_exists("tada"));
        assert!(!SoundManager::sound_exists("nonexistent"));
    }
    
    #[test]
    fn test_get_sound_info() {
        if let Some(info) = SoundManager::get_sound_info("tada") {
            assert_eq!(info.name, "tada");
            assert_eq!(info.filename, "tada.mp3");
            assert!(info.description.is_some());
        }
    }
    
    #[test]
    fn test_is_valid_sound_name() {
        assert!(SoundManager::is_valid_sound_name("tada"));
        assert!(SoundManager::is_valid_sound_name("mario_coin"));
        assert!(SoundManager::is_valid_sound_name("test-sound"));
        
        assert!(!SoundManager::is_valid_sound_name(""));
        assert!(!SoundManager::is_valid_sound_name("invalid sound"));
        assert!(!SoundManager::is_valid_sound_name("sound@name"));
    }
    
    #[test]
    fn test_get_mime_type() {
        assert_eq!(SoundManager::get_mime_type(), "audio/mpeg");
    }
    
    #[test]
    fn test_sound_playback_creation() {
        use crate::models::{UserId, RoomId};
        use uuid::Uuid;
        
        let user_id = UserId(Uuid::new_v4());
        let room_id = RoomId(Uuid::new_v4());
        
        let playback = SoundPlayback::new("tada".to_string(), user_id, room_id);
        
        assert_eq!(playback.sound_name, "tada");
        assert_eq!(playback.triggered_by, user_id);
        assert_eq!(playback.room_id, room_id);
    }
}