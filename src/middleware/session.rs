use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::errors::AuthError;
use crate::models::User;
use crate::AppState;

/// Authenticated user extractor that validates session tokens
/// 
/// This extractor can be used in handler functions to automatically
/// validate session tokens and extract the authenticated user.
/// 
/// # Usage
/// ```rust
/// async fn protected_handler(
///     auth_user: AuthenticatedUser,
/// ) -> impl IntoResponse {
///     // auth_user.user contains the authenticated User
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user: User,
}

/// Session extraction middleware implementation
/// 
/// Extracts session tokens from:
/// 1. Authorization header: "Bearer <token>"
/// 2. Cookie: "session_token=<token>"
/// 
/// Validates the session using AuthService and returns the authenticated user.
#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = SessionExtractionError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // Extract session token from headers or cookies
        let token = extract_session_token(parts)?;

        // Validate session and get user using auth service from AppState
        let user = state
            .auth_service
            .validate_session(token)
            .await
            .map_err(SessionExtractionError::from)?;

        Ok(AuthenticatedUser { user })
    }
}

/// Extracts session token from Authorization header or cookies
/// 
/// Priority order:
/// 1. Authorization header: "Bearer <token>"
/// 2. Cookie: "session_token=<token>"
pub fn extract_session_token(parts: &Parts) -> Result<String, SessionExtractionError> {
    // Try Authorization header first
    if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
        let auth_str = auth_header
            .to_str()
            .map_err(|_| SessionExtractionError::InvalidToken)?;

        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            if !token.is_empty() {
                return Ok(token.to_string());
            }
        }
    }

    // Try cookies as fallback
    if let Some(cookie_header) = parts.headers.get("cookie") {
        let cookie_str = cookie_header
            .to_str()
            .map_err(|_| SessionExtractionError::InvalidToken)?;

        // Parse cookies and look for session_token
        for cookie in cookie_str.split(';') {
            let cookie = cookie.trim();
            if let Some(value) = cookie.strip_prefix("session_token=") {
                if !value.is_empty() {
                    return Ok(value.to_string());
                }
            }
        }
    }

    Err(SessionExtractionError::MissingToken)
}

/// Session extraction errors with proper HTTP status codes
#[derive(Debug)]
pub enum SessionExtractionError {
    MissingToken,
    InvalidToken,
    SessionExpired,
    UserNotFound,
    InternalError,
}

impl From<AuthError> for SessionExtractionError {
    fn from(auth_error: AuthError) -> Self {
        match auth_error {
            AuthError::SessionExpired => SessionExtractionError::SessionExpired,
            AuthError::UserNotFound { .. } => SessionExtractionError::UserNotFound,
            AuthError::InvalidCredentials => SessionExtractionError::InvalidToken,
            _ => SessionExtractionError::InternalError,
        }
    }
}

impl IntoResponse for SessionExtractionError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            SessionExtractionError::MissingToken => {
                (StatusCode::UNAUTHORIZED, "Missing authentication token")
            }
            SessionExtractionError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Invalid authentication token")
            }
            SessionExtractionError::SessionExpired => {
                (StatusCode::UNAUTHORIZED, "Session expired")
            }
            SessionExtractionError::UserNotFound => {
                (StatusCode::UNAUTHORIZED, "User not found")
            }
            SessionExtractionError::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16()
        }));

        (status, body).into_response()
    }
}

/// Optional authenticated user extractor
/// 
/// Similar to AuthenticatedUser but returns None instead of an error
/// when no valid session is found. Useful for endpoints that work
/// differently for authenticated vs anonymous users.
#[derive(Debug, Clone)]
pub struct OptionalAuthenticatedUser {
    pub user: Option<User>,
}

#[async_trait]
impl FromRequestParts<AppState> for OptionalAuthenticatedUser {
    type Rejection = SessionExtractionError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // Try to extract authenticated user, but don't fail if not found
        match AuthenticatedUser::from_request_parts(parts, state).await {
            Ok(auth_user) => Ok(OptionalAuthenticatedUser {
                user: Some(auth_user.user),
            }),
            Err(SessionExtractionError::MissingToken) 
            | Err(SessionExtractionError::InvalidToken)
            | Err(SessionExtractionError::SessionExpired)
            | Err(SessionExtractionError::UserNotFound) => {
                Ok(OptionalAuthenticatedUser { user: None })
            }
            Err(e) => Err(e), // Only fail on internal errors
        }
    }
}

/// Session extractor that only extracts the token without validation
/// 
/// Useful for logout endpoints where you need the token but don't
/// need to validate the user exists.
#[derive(Debug, Clone)]
pub struct SessionToken {
    pub token: String,
}

#[async_trait]
impl FromRequestParts<AppState> for SessionToken {
    type Rejection = SessionExtractionError;

    async fn from_request_parts(parts: &mut Parts, _state: &AppState) -> Result<Self, Self::Rejection> {
        let token = extract_session_token(parts)?;
        Ok(SessionToken { token })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue, Request};

    fn create_test_parts_with_auth_header(token: &str) -> Parts {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        let request = Request::builder()
            .method("GET")
            .uri("/test")
            .body(())
            .unwrap();
        
        let (mut parts, _) = request.into_parts();
        parts.headers = headers;
        parts
    }

    fn create_test_parts_with_cookie(token: &str) -> Parts {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            HeaderValue::from_str(&format!("session_token={}", token)).unwrap(),
        );

        let request = Request::builder()
            .method("GET")
            .uri("/test")
            .body(())
            .unwrap();
        
        let (mut parts, _) = request.into_parts();
        parts.headers = headers;
        parts
    }

    fn create_test_parts_empty() -> Parts {
        let request = Request::builder()
            .method("GET")
            .uri("/test")
            .body(())
            .unwrap();
        
        let (parts, _) = request.into_parts();
        parts
    }

    #[test]
    fn test_extract_session_token_from_auth_header() {
        let parts = create_test_parts_with_auth_header("test_token_123");
        let token = extract_session_token(&parts).unwrap();
        assert_eq!(token, "test_token_123");
    }

    #[test]
    fn test_extract_session_token_from_cookie() {
        let parts = create_test_parts_with_cookie("cookie_token_456");
        let token = extract_session_token(&parts).unwrap();
        assert_eq!(token, "cookie_token_456");
    }

    #[test]
    fn test_extract_session_token_auth_header_priority() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str("Bearer auth_token").unwrap(),
        );
        headers.insert(
            "cookie",
            HeaderValue::from_str("session_token=cookie_token").unwrap(),
        );

        let request = Request::builder()
            .method("GET")
            .uri("/test")
            .body(())
            .unwrap();
        
        let (mut parts, _) = request.into_parts();
        parts.headers = headers;

        let token = extract_session_token(&parts).unwrap();
        assert_eq!(token, "auth_token"); // Auth header takes priority
    }

    #[test]
    fn test_extract_session_token_missing() {
        let parts = create_test_parts_empty();
        let result = extract_session_token(&parts);
        assert!(matches!(result, Err(SessionExtractionError::MissingToken)));
    }

    #[test]
    fn test_extract_session_token_empty_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str("Bearer ").unwrap());

        let request = Request::builder()
            .method("GET")
            .uri("/test")
            .body(())
            .unwrap();
        
        let (mut parts, _) = request.into_parts();
        parts.headers = headers;

        let result = extract_session_token(&parts);
        assert!(matches!(result, Err(SessionExtractionError::MissingToken)));
    }

    #[test]
    fn test_extract_session_token_invalid_auth_format() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str("Basic dGVzdA==").unwrap());

        let request = Request::builder()
            .method("GET")
            .uri("/test")
            .body(())
            .unwrap();
        
        let (mut parts, _) = request.into_parts();
        parts.headers = headers;

        let result = extract_session_token(&parts);
        assert!(matches!(result, Err(SessionExtractionError::MissingToken)));
    }

    #[test]
    fn test_extract_session_token_multiple_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            HeaderValue::from_str("other_cookie=value; session_token=my_token; another=value").unwrap(),
        );

        let request = Request::builder()
            .method("GET")
            .uri("/test")
            .body(())
            .unwrap();
        
        let (mut parts, _) = request.into_parts();
        parts.headers = headers;

        let token = extract_session_token(&parts).unwrap();
        assert_eq!(token, "my_token");
    }
}