use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue},
    response::{Html, IntoResponse},
};

use crate::AppState;

/// Serve login page with setup status and demo mode awareness
pub async fn serve_login_page(State(state): State<AppState>) -> impl IntoResponse {
    use axum::http::StatusCode;
    
    // First check if this is a first-run scenario
    match state.setup_service.is_first_run().await {
        Ok(true) => {
            // First run - redirect to setup page
            let mut headers = HeaderMap::new();
            headers.insert(
                header::LOCATION,
                HeaderValue::from_static("/setup"),
            );
            return (StatusCode::FOUND, headers, Html("")).into_response();
        }
        Ok(false) => {
            // Not first run - continue with normal logic
        }
        Err(e) => {
            // Error checking first-run status - log and continue
            tracing::warn!("Failed to check first-run status: {}", e);
        }
    }
    
    // Check if demo mode is enabled by looking for demo users
    let demo_mode = state.db.get_user_by_email("admin@campfire.demo").await
        .unwrap_or(None)
        .is_some();
    
    let html = if demo_mode {
        include_str!("../../templates/login_demo.html")
    } else {
        include_str!("../../templates/login.html")
    };
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    
    (headers, Html(html)).into_response()
}

/// Serve root page based on setup status and demo mode
pub async fn serve_root_page(State(state): State<AppState>) -> impl IntoResponse {
    use axum::http::StatusCode;
    
    // First check if this is a first-run scenario
    match state.setup_service.is_first_run().await {
        Ok(true) => {
            // First run - redirect to setup page
            let mut headers = HeaderMap::new();
            headers.insert(
                header::LOCATION,
                HeaderValue::from_static("/setup"),
            );
            return (StatusCode::FOUND, headers, Html("")).into_response();
        }
        Ok(false) => {
            // Not first run - continue with normal logic
        }
        Err(e) => {
            // Error checking first-run status - log and continue
            tracing::warn!("Failed to check first-run status: {}", e);
        }
    }
    
    // Check if demo mode is enabled by looking for demo users
    let demo_mode = state.db.get_user_by_email("admin@campfire.demo").await
        .unwrap_or(None)
        .is_some();
    
    if demo_mode {
        crate::assets::serve_demo_page().await.into_response()
    } else {
        crate::assets::serve_chat_interface().await.into_response()
    }
}

/// Serve demo status API endpoint
pub async fn demo_status(State(state): State<AppState>) -> impl IntoResponse {
    use axum::Json;
    use serde_json::json;
    
    // Check if demo mode is enabled by looking for demo users
    let demo_mode = state.db.get_user_by_email("admin@campfire.demo").await
        .unwrap_or(None)
        .is_some();
    
    // Get demo user count (simplified approach)
    let demo_user_count = if demo_mode {
        8 // We know we create 8 demo users
    } else {
        0
    };
    
    // Get room count (simplified approach)
    let room_count = if demo_mode {
        7 // We know we create 7 demo rooms
    } else {
        0
    };
    
    Json(json!({
        "demo_mode": demo_mode,
        "demo_users": demo_user_count,
        "demo_rooms": room_count,
        "features": {
            "websockets": true,
            "search": true,
            "sounds": true,
            "push_notifications": true,
            "bot_api": true
        },
        "quick_start_url": "/login",
        "demo_guide_url": "/demo"
    }))
}

/// Initialize demo data endpoint
pub async fn initialize_demo(State(state): State<AppState>) -> impl IntoResponse {
    use axum::{Json, http::StatusCode};
    use serde_json::json;
    use std::sync::Arc;
    
    // Check if demo data already exists
    let demo_exists = state.db.get_user_by_email("admin@campfire.demo").await
        .unwrap_or(None)
        .is_some();
    
    if demo_exists {
        return (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": "Demo data already exists",
                "demo_users": 8,
                "demo_rooms": 7,
                "login_url": "/login"
            }))
        );
    }
    
    // Initialize demo data
    let demo_initializer = crate::demo::DemoDataInitializer::new(Arc::new(state.db.clone()));
    
    match demo_initializer.initialize_if_needed().await {
        Ok(()) => {
            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "message": "Demo data initialized successfully",
                    "demo_users": 8,
                    "demo_rooms": 7,
                    "login_url": "/login",
                    "demo_credentials": crate::demo::DemoDataInitializer::get_demo_credentials()
                }))
            )
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": format!("Failed to initialize demo data: {}", e),
                    "error": "DEMO_INIT_FAILED"
                }))
            )
        }
    }
}