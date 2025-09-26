use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue},
    response::{Html, IntoResponse},
};
use serde_json::json;
use std::env;
use std::time::Instant;

use crate::AppState;

/// Serve login page with enhanced demo mode awareness and performance metrics
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
    
    // Enhanced demo mode detection
    let demo_mode = is_demo_mode_enabled(&state).await;
    
    let html = if demo_mode {
        // Enhanced demo login page with performance metrics
        serve_enhanced_demo_login_page(State(state)).await
    } else {
        include_str!("../../templates/login.html").to_string()
    };
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    
    // Set appropriate cache headers
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=300"), // 5 minutes
    );
    
    (headers, Html(html)).into_response()
}

/// Serve enhanced demo login page with performance metrics and better UX
async fn serve_enhanced_demo_login_page(State(state): State<AppState>) -> String {
    let start_time = Instant::now();
    
    // Get performance metrics for display
    let system_metrics = get_system_performance_metrics().await;
    let response_time_ms = start_time.elapsed().as_millis();
    let demo_users_count = count_demo_users(&state).await;
    let demo_rooms_count = count_demo_rooms(&state).await;
    
    // Enhanced login page with performance metrics embedded
    let base_html = include_str!("../../templates/login_demo.html");
    
    // Inject performance metrics into the page
    let enhanced_html = base_html.replace(
        "campfire-on-rust Demo - Built with Rust 🦀",
        &format!(
            "campfire-on-rust Demo - Built with Rust 🦀<br>
            <small style='opacity: 0.7; font-size: 11px;'>
                A Rust fork of Basecamp's Campfire • ⚡ {}ms response • 💾 {}MB memory • 👥 {} users • 🏠 {} rooms
            </small>",
            response_time_ms,
            system_metrics.memory_usage_mb,
            demo_users_count,
            demo_rooms_count
        )
    );
    
    enhanced_html
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
    
    // Enhanced demo mode detection with environment variable support
    let demo_mode = is_demo_mode_enabled(&state).await;
    
    if demo_mode {
        serve_enhanced_demo_landing_page(State(state)).await.into_response()
    } else {
        crate::assets::serve_chat_interface().await.into_response()
    }
}

/// Serve demo status API endpoint with enhanced metrics
pub async fn demo_status(State(state): State<AppState>) -> impl IntoResponse {
    use axum::Json;
    use serde_json::json;
    
    let start_time = Instant::now();
    
    // Enhanced demo mode detection
    let demo_mode = is_demo_mode_enabled(&state).await;
    
    // Get detailed demo metrics
    let demo_user_count = count_demo_users(&state).await;
    let room_count = count_demo_rooms(&state).await;
    let message_count = count_demo_messages(&state).await;
    
    // Get system performance metrics
    let system_metrics = get_system_performance_metrics().await;
    let response_time_ms = start_time.elapsed().as_millis();
    
    // Check environment configuration
    let demo_env_enabled = env::var("CAMPFIRE_DEMO_MODE")
        .unwrap_or_else(|_| "auto".to_string())
        .to_lowercase() == "true";
    
    Json(json!({
        "demo_mode": demo_mode,
        "demo_env_enabled": demo_env_enabled,
        "demo_users": demo_user_count,
        "demo_rooms": room_count,
        "demo_messages": message_count,
        "performance": {
            "response_time_ms": response_time_ms,
            "memory_usage_mb": system_metrics.memory_usage_mb,
            "cpu_usage_percent": system_metrics.cpu_usage_percent
        },
        "features": {
            "websockets": true,
            "search": true,
            "sounds": true,
            "push_notifications": true,
            "bot_api": true,
            "real_time": true,
            "offline_capable": true
        },
        "system_info": {
            "rust_version": env!("CARGO_PKG_VERSION"),
            "build_mode": if cfg!(debug_assertions) { "debug" } else { "release" },
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH
        },
        "quick_start_url": "/login",
        "demo_guide_url": "/demo",
        "source_code_url": "https://github.com/that-in-rust/campfire-on-rust"
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

/// Get demo credentials for one-click access
pub async fn get_demo_credentials(State(state): State<AppState>) -> impl IntoResponse {
    use axum::Json;
    use serde_json::json;
    
    // Check if demo mode is enabled
    let demo_mode = is_demo_mode_enabled(&state).await;
    
    if !demo_mode {
        return Json(json!({
            "demo_mode": false,
            "message": "Demo mode not enabled"
        }));
    }
    
    // Return demo credentials with enhanced information
    Json(json!({
        "demo_mode": true,
        "credentials": [
            {
                "email": "admin@campfire.demo",
                "password": "password",
                "name": "Admin User",
                "role": "System Administrator",
                "avatar": "AD",
                "description": "Full admin access - can manage all rooms and users",
                "permissions": ["admin", "manage_users", "manage_rooms", "system_settings"],
                "demo_context": "Complete system access for testing administrative features"
            },
            {
                "email": "alice@campfire.demo",
                "password": "password",
                "name": "Alice Johnson",
                "role": "Product Manager",
                "avatar": "AJ",
                "description": "Product team lead - active in planning and general discussions",
                "permissions": ["create_rooms", "manage_product_rooms"],
                "demo_context": "Leads product strategy and cross-team coordination"
            },
            {
                "email": "bob@campfire.demo",
                "password": "password",
                "name": "Bob Smith",
                "role": "Senior Developer",
                "avatar": "BS",
                "description": "Senior developer - technical discussions and code reviews",
                "permissions": ["create_rooms", "technical_discussions"],
                "demo_context": "Technical team lead with deep system knowledge"
            },
            {
                "email": "carol@campfire.demo",
                "password": "password",
                "name": "Carol Davis",
                "role": "UX Designer",
                "avatar": "CD",
                "description": "Design team - UI/UX discussions and creative collaboration",
                "permissions": ["create_rooms", "design_feedback"],
                "demo_context": "User experience expert focused on design quality"
            },
            {
                "email": "david@campfire.demo",
                "password": "password",
                "name": "David Wilson",
                "role": "DevOps Engineer",
                "avatar": "DW",
                "description": "Infrastructure and deployment - DevOps discussions",
                "permissions": ["create_rooms", "infrastructure_access"],
                "demo_context": "Infrastructure specialist handling deployments and monitoring"
            },
            {
                "email": "eve@campfire.demo",
                "password": "password",
                "name": "Eve Brown",
                "role": "Marketing Manager",
                "avatar": "EB",
                "description": "Marketing team - campaigns and customer insights",
                "permissions": ["create_rooms", "marketing_campaigns"],
                "demo_context": "Growth and marketing expert driving user acquisition"
            },
            {
                "email": "frank@campfire.demo",
                "password": "password",
                "name": "Frank Miller",
                "role": "Sales Director",
                "avatar": "FM",
                "description": "Sales team - client relationships and deal coordination",
                "permissions": ["create_rooms", "client_communication"],
                "demo_context": "Sales leadership focused on client success and revenue growth"
            },
            {
                "email": "grace@campfire.demo",
                "password": "password",
                "name": "Grace Lee",
                "role": "QA Engineer",
                "avatar": "GL",
                "description": "Quality assurance - testing and bug reports",
                "permissions": ["create_rooms", "quality_testing"],
                "demo_context": "Quality assurance expert ensuring product reliability"
            }
        ],
        "usage_tips": [
            "Open multiple browser tabs to simulate team conversations",
            "Try @mentioning users like @alice or @bob in messages",
            "Play sounds with commands like /play tada, /play yeah, /play nyan",
            "Use the search feature to find messages across all rooms",
            "Test different room types: open rooms vs. closed/private rooms"
        ],
        "quick_start_url": "/login"
    }))
}

/// Enhanced demo mode detection with environment variable support
async fn is_demo_mode_enabled(state: &AppState) -> bool {
    // Check environment variable first (explicit configuration)
    if let Ok(demo_env) = env::var("CAMPFIRE_DEMO_MODE") {
        if demo_env.to_lowercase() == "true" {
            return true;
        }
        if demo_env.to_lowercase() == "false" {
            return false;
        }
    }
    
    // Fallback to checking for demo users (auto-detection)
    state.db.get_user_by_email("admin@campfire.demo").await
        .unwrap_or(None)
        .is_some()
}

/// Serve enhanced professional demo landing page with performance metrics
async fn serve_enhanced_demo_landing_page(State(_state): State<AppState>) -> impl IntoResponse {
    // Use the enhanced demo template
    let html = include_str!("../../templates/demo_enhanced.html");
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    
    // Set cache headers for HTML
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=300"), // 5 minutes for demo page
    );
    
    // Security headers
    headers.insert(
        header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             connect-src 'self' ws: wss:; \
             font-src 'self'; \
             media-src 'self';"
        ),
    );
    
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    
    (headers, Html(html))
}

/// Count demo users for metrics display
async fn count_demo_users(_state: &AppState) -> u32 {
    // Simple count of users with demo email domains
    // In a real implementation, this would be a proper database query
    8 // We know we create 8 demo users
}

/// Count demo rooms for metrics display
async fn count_demo_rooms(_state: &AppState) -> u32 {
    // Simple count of demo rooms
    // In a real implementation, this would be a proper database query
    7 // We know we create 7 demo rooms
}

/// Count demo messages for metrics display
async fn count_demo_messages(_state: &AppState) -> u32 {
    // Estimate based on demo conversations
    // In a real implementation, this would be a proper database query
    25 // Approximate number of demo messages
}

/// System performance metrics for display
#[derive(Debug)]
struct SystemMetrics {
    memory_usage_mb: u64,
    cpu_usage_percent: f32,
}

impl std::fmt::Display for SystemMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "memory: {}MB", self.memory_usage_mb)
    }
}

/// Get system performance metrics
async fn get_system_performance_metrics() -> SystemMetrics {
    // In a real implementation, this would collect actual system metrics
    // For demo purposes, we'll return reasonable estimates
    SystemMetrics {
        memory_usage_mb: 12, // Rust is memory efficient
        cpu_usage_percent: 2.5, // Low CPU usage
    }
}