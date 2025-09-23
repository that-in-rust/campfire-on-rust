use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::collections::HashMap;
use ammonia::Builder;

/// Validated JSON extractor that automatically validates input
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: for<'de> Deserialize<'de> + Validate,
    S: Send + Sync,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|_| ValidationError::new("invalid_json"))?;
        
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

/// Custom validation error response
#[derive(Debug, Serialize)]
pub struct ValidationErrorResponse {
    pub error: String,
    pub details: HashMap<String, Vec<String>>,
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        let mut details = HashMap::new();
        
        // Convert validation errors to a more user-friendly format
        for (field, errors) in self.field_errors() {
            let error_messages: Vec<String> = errors
                .iter()
                .map(|e| e.message.as_ref().unwrap_or(&"Invalid value".into()).to_string())
                .collect();
            details.insert(field.to_string(), error_messages);
        }
        
        let response = ValidationErrorResponse {
            error: "Validation failed".to_string(),
            details,
        };
        
        (StatusCode::BAD_REQUEST, Json(response)).into_response()
    }
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
}

/// Bot message request validation
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBotMessageRequest {
    #[validate(length(min = 1, max = 10000, message = "Message content must be 1-10000 characters"))]
    pub content: String,
}

/// Content sanitization utilities
pub mod sanitization {
    use ammonia::Builder;
    
    /// Sanitize HTML content for messages
    pub fn sanitize_message_content(content: &str) -> String {
        Builder::default()
            .tags(hashset![
                "b", "i", "u", "strong", "em", "br", "p", "a", "code", "pre"
            ])
            .link_rel(Some("nofollow noopener noreferrer"))
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
        // Remove HTML and trim whitespace
        let sanitized = sanitize_plain_text(name);
        sanitized.trim().to_string()
    }
    
    /// Validate and sanitize user input
    pub fn sanitize_user_input(input: &str) -> String {
        // Remove HTML, normalize whitespace
        let sanitized = sanitize_plain_text(input);
        sanitized.trim().chars().take(1000).collect()
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