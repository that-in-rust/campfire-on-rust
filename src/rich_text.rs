use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

use crate::models::UserId;

/// Rich text processing for Campfire messages
/// 
/// Handles:
/// - Enhanced HTML sanitization with rich text features
/// - @mention parsing and user linking
/// - /play command detection and processing
/// - Sound command validation
pub struct RichTextProcessor;

/// Configuration for HTML sanitization with rich text support
/// We'll create the builder fresh each time since it doesn't implement Clone

/// Regex for detecting @mentions
static MENTION_REGEX: OnceLock<Regex> = OnceLock::new();

/// Regex for detecting /play commands
static PLAY_COMMAND_REGEX: OnceLock<Regex> = OnceLock::new();

/// Available sound names (from embedded assets)
static AVAILABLE_SOUNDS: &[&str] = &[
    "56k", "ballmer", "bell", "bezos", "bueller", "butts", "clowntown", 
    "cottoneyejoe", "crickets", "curb", "dadgummit", "dangerzone", "danielsan",
    "deeper", "donotwant", "drama", "flawless", "glados", "gogogo", "greatjob",
    "greyjoy", "guarantee", "heygirl", "honk", "horn", "horror", "incoming",
    "inconceivable", "letitgo", "live", "loggins", "makeitso", "mario_coin",
    "maybe", "noooo", "nyan", "ohmy", "ohyeah", "pushit", "rimshot", "rollout",
    "rumble", "sax", "secret", "sexyback", "story", "tada", "tmyk", "totes",
    "trololo", "trombone", "unix", "vuvuzela", "what", "whoomp", "wups",
    "yay", "yeah", "yodel"
];

/// Result of processing rich text content
#[derive(Debug, Clone)]
pub struct ProcessedContent {
    /// The sanitized HTML content with rich text features
    pub html: String,
    /// Extracted @mentions (username -> user_id mapping)
    pub mentions: Vec<String>,
    /// Extracted /play commands
    pub play_commands: Vec<String>,
    /// Whether the message contains any rich text features
    pub has_rich_features: bool,
}

impl RichTextProcessor {
    /// Process message content for rich text features
    /// 
    /// # Arguments
    /// * `content` - Raw message content
    /// * `user_lookup` - Function to resolve usernames to user IDs
    /// 
    /// # Returns
    /// ProcessedContent with sanitized HTML and extracted features
    pub async fn process_content<F>(
        content: &str,
        user_lookup: F,
    ) -> Result<ProcessedContent, RichTextError>
    where
        F: Fn(&str) -> Option<UserId>,
    {
        // Step 1: Extract @mentions before HTML processing
        let mentions = Self::extract_mentions(content);
        
        // Step 2: Extract /play commands
        let play_commands = Self::extract_play_commands(content);
        
        // Step 3: Process content for rich text HTML
        let processed_html = Self::process_html_content(content, &mentions, &user_lookup)?;
        
        // Step 4: Determine if content has rich features
        let has_rich_features = !mentions.is_empty() 
            || !play_commands.is_empty() 
            || Self::has_html_formatting(&processed_html)
            || processed_html != content; // If HTML was sanitized, it's a rich feature
        
        Ok(ProcessedContent {
            html: processed_html,
            mentions,
            play_commands,
            has_rich_features,
        })
    }
    
    /// Extract @mentions from content
    /// 
    /// Returns list of mentioned usernames (without @ prefix)
    fn extract_mentions(content: &str) -> Vec<String> {
        let regex = MENTION_REGEX.get_or_init(|| {
            Regex::new(r"@([a-zA-Z0-9_-]+)").expect("Invalid mention regex")
        });
        
        regex
            .captures_iter(content)
            .map(|cap| cap[1].to_string())
            .collect()
    }
    
    /// Extract /play commands from content
    /// 
    /// Returns list of valid sound names
    fn extract_play_commands(content: &str) -> Vec<String> {
        let regex = PLAY_COMMAND_REGEX.get_or_init(|| {
            Regex::new(r"/play\s+([a-zA-Z0-9_-]+)").expect("Invalid play command regex")
        });
        
        regex
            .captures_iter(content)
            .map(|cap| cap[1].to_string())
            .filter(|sound_name| AVAILABLE_SOUNDS.contains(&sound_name.as_str()))
            .collect()
    }
    
    /// Process HTML content with rich text features
    /// 
    /// Sanitizes HTML while preserving rich text formatting and converting
    /// @mentions to proper links
    fn process_html_content<F>(
        content: &str,
        mentions: &[String],
        user_lookup: &F,
    ) -> Result<String, RichTextError>
    where
        F: Fn(&str) -> Option<UserId>,
    {
        // Create rich text sanitizer configuration
        let mut schemes = HashSet::new();
        schemes.insert("http");
        schemes.insert("https");
        schemes.insert("mailto");
        
        let mut builder = ammonia::Builder::default();
        builder
            // Allow basic formatting tags
            .add_tags(&["b", "strong", "i", "em", "u", "s", "strike", "del"])
            // Allow links with specific attributes
            .add_tags(&["a"])
            .add_tag_attributes("a", &["href", "data-mention-id", "class"])
            // Allow line breaks
            .add_tags(&["br"])
            // Allow code formatting
            .add_tags(&["code", "pre"])
            // Allow lists
            .add_tags(&["ul", "ol", "li"])
            // Allow blockquotes
            .add_tags(&["blockquote"])
            // Set URL schemes for links
            .url_schemes(schemes);
        
        // First pass: convert @mentions to proper HTML links
        let mut processed_content = content.to_string();
        
        for mention in mentions {
            if let Some(user_id) = user_lookup(mention) {
                let mention_pattern = format!("@{}", mention);
                let mention_link = format!(
                    r#"<a href="/users/{}" data-mention-id="{}" class="mention">@{}</a>"#,
                    user_id, user_id, mention
                );
                processed_content = processed_content.replace(&mention_pattern, &mention_link);
            }
        }
        
        // Second pass: sanitize HTML while preserving our rich text features
        let sanitized = builder.clean(&processed_content).to_string();
        
        // Validate that sanitization didn't remove everything important
        if sanitized.trim().is_empty() && !content.trim().is_empty() {
            return Err(RichTextError::SanitizationRemoved);
        }
        
        Ok(sanitized)
    }
    
    /// Check if content has HTML formatting beyond plain text
    fn has_html_formatting(content: &str) -> bool {
        // Simple check for HTML tags (excluding our mention links and line breaks)
        let tag_regex = Regex::new(r"<[^>]+>").unwrap();
        let matches: Vec<&str> = tag_regex.find_iter(content)
            .map(|m| m.as_str())
            .collect();
        
        // Filter out allowed tags (a, br)
        for tag in matches {
            if !tag.starts_with("<a ") && !tag.starts_with("</a>") && 
               !tag.starts_with("<br") && !tag.starts_with("</br>") {
                return true;
            }
        }
        false
    }
    
    /// Validate that a sound name is available
    pub fn is_valid_sound(sound_name: &str) -> bool {
        AVAILABLE_SOUNDS.contains(&sound_name)
    }
    
    /// Get list of all available sounds
    pub fn available_sounds() -> &'static [&'static str] {
        AVAILABLE_SOUNDS
    }
    
    /// Process /play commands in content and return clean content + commands
    /// 
    /// This removes /play commands from the visible content while preserving
    /// them for sound playback
    pub fn extract_and_clean_play_commands(content: &str) -> (String, Vec<String>) {
        let regex = PLAY_COMMAND_REGEX.get_or_init(|| {
            Regex::new(r"/play\s+([a-zA-Z0-9_-]+)").expect("Invalid play command regex")
        });
        
        let play_commands: Vec<String> = regex
            .captures_iter(content)
            .map(|cap| cap[1].to_string())
            .filter(|sound_name| AVAILABLE_SOUNDS.contains(&sound_name.as_str()))
            .collect();
        
        // Remove /play commands from content for display
        let cleaned_content = regex.replace_all(content, "").to_string();
        
        // Clean up extra whitespace
        let cleaned_content = cleaned_content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        
        (cleaned_content, play_commands)
    }
}

/// Errors that can occur during rich text processing
#[derive(Debug, thiserror::Error)]
pub enum RichTextError {
    #[error("HTML sanitization removed all content")]
    SanitizationRemoved,
    
    #[error("Invalid mention format: {mention}")]
    InvalidMention { mention: String },
    
    #[error("Invalid sound name: {sound}")]
    InvalidSound { sound: String },
    
    #[error("Content processing failed: {reason}")]
    ProcessingFailed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserId;
    use uuid::Uuid;
    
    fn mock_user_lookup(username: &str) -> Option<UserId> {
        match username {
            "alice" => Some(UserId(Uuid::new_v4())),
            "bob" => Some(UserId(Uuid::new_v4())),
            _ => None,
        }
    }
    
    #[tokio::test]
    async fn test_extract_mentions() {
        let content = "Hello @alice and @bob, how are you?";
        let mentions = RichTextProcessor::extract_mentions(content);
        
        assert_eq!(mentions, vec!["alice", "bob"]);
    }
    
    #[tokio::test]
    async fn test_extract_play_commands() {
        let content = "Check this out! /play tada and /play bell";
        let commands = RichTextProcessor::extract_play_commands(content);
        
        assert_eq!(commands, vec!["tada", "bell"]);
    }
    
    #[tokio::test]
    async fn test_extract_invalid_play_commands() {
        let content = "/play invalidsound and /play bell";
        let commands = RichTextProcessor::extract_play_commands(content);
        
        // Only valid sounds should be returned
        assert_eq!(commands, vec!["bell"]);
    }
    
    #[tokio::test]
    async fn test_process_content_with_mentions() {
        let content = "Hello @alice, this is <b>bold</b> text!";
        let result = RichTextProcessor::process_content(content, mock_user_lookup).await.unwrap();
        
        assert_eq!(result.mentions, vec!["alice"]);
        assert!(result.html.contains("data-mention-id"));
        assert!(result.html.contains("<b>bold</b>"));
        assert!(result.has_rich_features);
    }
    
    #[tokio::test]
    async fn test_extract_and_clean_play_commands() {
        let content = "Hello everyone! /play tada\n\nThis is a message /play bell";
        let (cleaned, commands) = RichTextProcessor::extract_and_clean_play_commands(content);
        
        assert_eq!(commands, vec!["tada", "bell"]);
        // The cleaning process removes /play commands and cleans up whitespace
        assert_eq!(cleaned, "Hello everyone!\nThis is a message");
    }
    
    #[test]
    fn test_is_valid_sound() {
        assert!(RichTextProcessor::is_valid_sound("tada"));
        assert!(RichTextProcessor::is_valid_sound("bell"));
        assert!(!RichTextProcessor::is_valid_sound("invalid"));
    }
    
    #[test]
    fn test_available_sounds() {
        let sounds = RichTextProcessor::available_sounds();
        assert!(sounds.contains(&"tada"));
        assert!(sounds.contains(&"bell"));
        assert_eq!(sounds.len(), 59); // Total number of sound files
    }
    
    #[tokio::test]
    async fn test_html_sanitization() {
        let content = r#"<script>alert('xss')</script><b>Safe content</b>"#;
        let result = RichTextProcessor::process_content(content, mock_user_lookup).await.unwrap();
        
        // Script should be removed, but bold should remain
        assert!(!result.html.contains("<script>"));
        assert!(result.html.contains("<b>Safe content</b>"));
    }
}