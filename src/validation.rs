use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::collections::{HashMap, HashSet};
use ammonia::Builder;

/// Custom validation error response
#[derive(Debug, Serialize)]
pub struct ValidationErrorResponse {
    pub error: String,
    pub details: HashMap<String, Vec<String>>,
}

impl IntoResponse for ValidationErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

/// Helper function to validate and return error response if validation fails
pub fn validate_request<T: Validate>(request: &T) -> Result<(), ValidationErrorResponse> {
    request.validate().map_err(|e| {
        let mut details = HashMap::new();
        for (field, errors) in e.field_errors() {
            let error_messages: Vec<String> = errors
                .iter()
                .map(|e| e.message.as_ref().unwrap_or(&"Invalid value".into()).to_string())
                .collect();
            details.insert(field.to_string(), error_messages);
        }
        ValidationErrorResponse {
            error: "Validation failed".to_string(),
            details,
        }
    })
}

/// Login request validation
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// Create room request validation
#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoomRequest {
    #[validate(length(min = 1, max = 100, message = "Room name must be 1-100 characters"))]
    pub name: String,
    
    #[validate(length(max = 500, message = "Topic must be less than 500 characters"))]
    pub topic: Option<String>,
    
    #[validate(custom = "validate_room_type")]
    pub room_type: String,
}

fn validate_room_type(room_type: &str) -> Result<(), ValidationError> {
    match room_type {
        "open" | "closed" | "direct" => Ok(()),
        _ => Err(ValidationError::new("invalid_room_type")),
    }
}

/// Create message request validation
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMessageRequest {
    #[validate(length(min = 1, max = 10000, message = "Message content must be 1-10000 characters"))]
    pub content: String,
    
    pub client_message_id: uuid::Uuid,
}

/// Add room member request validation
#[derive(Debug, Deserialize, Validate)]
pub struct AddRoomMemberRequest {
    pub user_id: uuid::Uuid,
    
    #[validate(custom = "validate_involvement_level")]
    pub involvement_level: String,
}

fn validate_involvement_level(level: &str) -> Result<(), ValidationError> {
    match level {
        "member" | "admin" => Ok(()),
        _ => Err(ValidationError::new("invalid_involvement_level")),
    }
}

/// Search request validation
#[derive(Debug, Deserialize, Validate)]
pub struct SearchRequest {
    #[validate(length(min = 1, max = 100, message = "Search query must be 1-100 characters"))]
    pub q: String,
    
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    pub limit: Option<u32>,
    
    pub room_id: Option<uuid::Uuid>,
}

/// Push subscription request validation
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePushSubscriptionRequest {
    #[validate(url(message = "Invalid endpoint URL"))]
    pub endpoint: String,
    
    pub keys: PushSubscriptionKeys,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PushSubscriptionKeys {
    #[validate(length(min = 1, message = "p256dh key is required"))]
    pub p256dh: String,
    
    #[validate(length(min = 1, message = "auth key is required"))]
    pub auth: String,
}

/// Bot creation request validation
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBotRequest {
    #[validate(length(min = 1, max = 50, message = "Bot name must be 1-50 characters"))]
    pub name: String,
    
    #[validate(length(max = 200, message = "Description must be less than 200 characters"))]
    pub description: Option<String>,
    
    #[validate(url(message = "Invalid webhook URL"))]
    pub webhook_url: Option<String>,
}

/// Bot message request validation
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBotMessageRequest {
    #[validate(length(min = 1, max = 10000, message = "Message content must be 1-10000 characters"))]
    pub content: String,
}

/// Content sanitization utilities
pub mod sanitization {
    use super::Builder;
    use std::collections::HashSet;
    use regex::Regex;
    
    // Precompiled regex patterns for security
    fn get_sql_injection_pattern() -> &'static Regex {
        static PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute|script|javascript|vbscript|onload|onerror|onclick)").unwrap()
        })
    }
    
    fn get_xss_pattern() -> &'static Regex {
        static PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r"(?i)(<script|javascript:|vbscript:|onload=|onerror=|onclick=|data:text/html)").unwrap()
        })
    }
    
    fn get_path_traversal_pattern() -> &'static Regex {
        static PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r"(\.\./|\.\.\\|%2e%2e%2f|%2e%2e%5c)").unwrap()
        })
    }
    
    /// Comprehensive input validation and sanitization
    pub fn validate_and_sanitize_input(input: &str, max_length: usize) -> Result<String, String> {
        // Check for null bytes
        if input.contains('\0') {
            return Err("Input contains null bytes".to_string());
        }
        
        // Check length
        if input.len() > max_length {
            return Err(format!("Input too long: {} > {}", input.len(), max_length));
        }
        
        // Check for SQL injection patterns
        if get_sql_injection_pattern().is_match(input) {
            return Err("Input contains potentially dangerous SQL patterns".to_string());
        }
        
        // Check for XSS patterns
        if get_xss_pattern().is_match(input) {
            return Err("Input contains potentially dangerous script patterns".to_string());
        }
        
        // Check for path traversal
        if get_path_traversal_pattern().is_match(input) {
            return Err("Input contains path traversal patterns".to_string());
        }
        
        // Sanitize and return
        Ok(sanitize_user_input(input))
    }
    
    /// Sanitize HTML content for messages
    pub fn sanitize_message_content(content: &str) -> String {
        let mut allowed_tags = HashSet::new();
        allowed_tags.insert("b");
        allowed_tags.insert("i");
        allowed_tags.insert("u");
        allowed_tags.insert("strong");
        allowed_tags.insert("em");
        allowed_tags.insert("br");
        allowed_tags.insert("p");
        allowed_tags.insert("a");
        allowed_tags.insert("code");
        allowed_tags.insert("pre");
        
        let mut allowed_attributes = HashSet::new();
        allowed_attributes.insert("href");
        allowed_attributes.insert("title");
        
        Builder::default()
            .tags(allowed_tags)
            .generic_attributes(allowed_attributes)
            .link_rel(Some("nofollow noopener noreferrer"))
            .url_schemes({
                let mut schemes = HashSet::new();
                schemes.insert("http");
                schemes.insert("https");
                schemes.insert("mailto");
                schemes
            })
            .clean(content)
            .to_string()
    }
    
    /// Sanitize plain text (remove any HTML)
    pub fn sanitize_plain_text(content: &str) -> String {
        Builder::empty()
            .clean(content)
            .to_string()
    }
    
    /// Validate and sanitize room name
    pub fn sanitize_room_name(name: &str) -> String {
        // Simple HTML tag removal using regex-like approach
        let mut result = String::new();
        let mut in_tag = false;
        
        for ch in name.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(ch),
                _ => {} // Skip characters inside tags
            }
        }
        
        result.trim().to_string()
    }
    
    /// Validate and sanitize user input
    pub fn sanitize_user_input(input: &str) -> String {
        // Remove HTML, normalize whitespace, remove control characters
        let sanitized = sanitize_plain_text(input);
        
        let normalized: String = sanitized
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        normalized.trim().chars().take(1000).collect()
    }
    
    /// Sanitize email addresses
    pub fn sanitize_email(email: &str) -> String {
        // Remove HTML and normalize
        let sanitized = sanitize_plain_text(email);
        
        // Remove whitespace and convert to lowercase
        sanitized.trim().to_lowercase()
    }
    
    /// Sanitize URLs
    pub fn sanitize_url(url: &str) -> Result<String, String> {
        let sanitized = sanitize_plain_text(url);
        
        // Basic URL validation
        if !sanitized.starts_with("http://") && !sanitized.starts_with("https://") {
            return Err("URL must start with http:// or https://".to_string());
        }
        
        // Check for dangerous protocols
        if sanitized.contains("javascript:") || sanitized.contains("data:") || sanitized.contains("vbscript:") {
            return Err("URL contains dangerous protocol".to_string());
        }
        
        Ok(sanitized.trim().to_string())
    }
    
    /// Validate bot token format
    pub fn validate_bot_token(token: &str) -> Result<String, String> {
        let sanitized = sanitize_plain_text(token);
        
        // Bot tokens should be alphanumeric with hyphens/underscores
        if !sanitized.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err("Bot token contains invalid characters".to_string());
        }
        
        if sanitized.len() < 10 || sanitized.len() > 100 {
            return Err("Bot token length must be between 10 and 100 characters".to_string());
        }
        
        Ok(sanitized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_login_request_validation() {
        let valid_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_email = LoginRequest {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        let empty_password = LoginRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(),
        };
        assert!(empty_password.validate().is_err());
    }

    #[test]
    fn test_create_room_request_validation() {
        let valid_request = CreateRoomRequest {
            name: "Test Room".to_string(),
            topic: Some("Test topic".to_string()),
            room_type: "open".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_room_type = CreateRoomRequest {
            name: "Test Room".to_string(),
            topic: None,
            room_type: "invalid".to_string(),
        };
        assert!(invalid_room_type.validate().is_err());

        let empty_name = CreateRoomRequest {
            name: "".to_string(),
            topic: None,
            room_type: "open".to_string(),
        };
        assert!(empty_name.validate().is_err());
    }

    #[test]
    fn test_create_message_request_validation() {
        let valid_request = CreateMessageRequest {
            content: "Hello, world!".to_string(),
            client_message_id: uuid::Uuid::new_v4(),
        };
        assert!(valid_request.validate().is_ok());

        let empty_content = CreateMessageRequest {
            content: "".to_string(),
            client_message_id: uuid::Uuid::new_v4(),
        };
        assert!(empty_content.validate().is_err());

        let too_long_content = CreateMessageRequest {
            content: "a".repeat(10001),
            client_message_id: uuid::Uuid::new_v4(),
        };
        assert!(too_long_content.validate().is_err());
    }

    #[test]
    fn test_content_sanitization() {
        use sanitization::*;

        let html_content = "<script>alert('xss')</script><b>Bold text</b>";
        let sanitized = sanitize_message_content(html_content);
        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("<b>Bold text</b>"));

        let plain_text = sanitize_plain_text("<b>Bold</b> text");
        assert_eq!(plain_text, "Bold text");

        let room_name = sanitize_room_name("  <script>Room</script>  ");
        assert_eq!(room_name, "Room");
    }
}