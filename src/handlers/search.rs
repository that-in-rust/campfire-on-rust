use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use validator::Validate;
use crate::{
    AppState,
    services::search::{SearchResponse, SearchError},
    middleware::session::AuthenticatedUser,
    validation::{SearchRequest, sanitization},
};

/// GET /api/search?q=query&limit=20&offset=0&room_id=uuid
/// 
/// Search messages with full-text search across user's accessible rooms
pub async fn search_messages(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(params): Query<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<Value>)> {
    // Validate the search request
    if let Err(validation_errors) = params.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Validation failed",
                "details": validation_errors
            }))
        ));
    }
    
    // Sanitize search query
    let sanitized_query = sanitization::sanitize_user_input(&params.q);
    
    // Create sanitized search request
    let search_request = crate::services::search::SearchRequest {
        query: sanitized_query,
        limit: params.limit,
        offset: None,
        room_id: params.room_id.map(|id| crate::models::RoomId(id)),
    };
    
    match state.search_service.search_messages(auth_user.user.id, search_request).await {
        Ok(response) => Ok(Json(response)),
        Err(err) => {
            let error_message = err.to_string();
            let error_type = match &err {
                SearchError::InvalidQuery { .. } => "invalid_query",
                SearchError::QueryTooShort => "query_too_short",
                SearchError::QueryTooLong => "query_too_long",
                SearchError::Database(_) => "database_error",
                SearchError::RoomAccess(_) => "access_denied",
            };
            let status_code = StatusCode::from(err);
            let error_response = json!({
                "error": error_message,
                "type": error_type
            });
            Err((status_code, Json(error_response)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create test app state would go here
    // This would require mocking the search service for proper testing
    
    #[tokio::test]
    async fn test_search_messages_success() {
        // Test would verify successful search with proper authorization
        // Implementation would require mock services
    }
    
    #[tokio::test]
    async fn test_search_messages_invalid_query() {
        // Test would verify proper error handling for invalid queries
        // Implementation would require mock services
    }
    
    #[tokio::test]
    async fn test_search_messages_unauthorized() {
        // Test would verify proper authorization checking
        // Implementation would require mock services
    }
}