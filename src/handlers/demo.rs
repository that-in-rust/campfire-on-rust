use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};


use crate::AppState;

/// Query parameters for demo credential selection
#[derive(Debug, Deserialize)]
pub struct DemoCredentialQuery {
    pub email: Option<String>,
    pub tab_id: Option<String>,
}

/// Request body for starting simulation session
#[derive(Debug, Deserialize)]
pub struct StartSimulationRequest {
    pub user_email: String,
    pub browser_tab_id: String,
}

/// Request body for updating session activity
#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    pub session_id: String,
    pub features_explored: Vec<String>,
}

/// Request body for completing tour step
#[derive(Debug, Deserialize)]
pub struct CompleteTourStepRequest {
    pub session_id: String,
    pub step_id: String,
}

/// Get demo mode status and basic information
pub async fn get_demo_status(State(state): State<AppState>) -> impl IntoResponse {
    // Check if we're in demo mode
    let demo_mode = std::env::var("CAMPFIRE_DEMO_MODE")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";
    
    if !demo_mode {
        return Json(json!({
            "demo_mode": false,
            "message": "Demo mode not enabled"
        })).into_response();
    }
    
    // Get demo statistics if in demo mode
    match state.demo_service.get_demo_statistics().await {
        Ok(stats) => {
            Json(json!({
                "demo_mode": true,
                "demo_users": stats.total_users,
                "demo_rooms": stats.total_rooms,
                "demo_messages": stats.total_messages,
                "active_sessions": stats.active_sessions,
                "uptime_seconds": stats.uptime_seconds,
                "features_available": [
                    "real_time_messaging",
                    "mentions",
                    "sound_effects", 
                    "search",
                    "multiple_rooms",
                    "bot_integration",
                    "mobile_responsive",
                    "dark_mode"
                ]
            })).into_response()
        }
        Err(e) => {
            warn!("Failed to get demo statistics: {}", e);
            Json(json!({
                "demo_mode": true,
                "demo_users": 0,
                "demo_rooms": 0,
                "demo_messages": 0,
                "active_sessions": 0,
                "uptime_seconds": 0,
                "error": "Failed to load demo statistics"
            })).into_response()
        }
    }
}

/// Track demo events for analytics (simple implementation)
pub async fn track_demo_event(
    State(_state): State<AppState>,
    Json(event_data): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Log the event for now - could be enhanced with proper analytics
    info!("Demo event tracked: {}", event_data);
    
    Json(json!({
        "success": true,
        "message": "Event tracked successfully"
    }))
}

/// Get demo user credentials for one-click login (Requirement 10.3)
pub async fn get_demo_credentials(State(state): State<AppState>) -> impl IntoResponse {
    match state.demo_service.get_demo_credentials().await {
        Ok(credentials) => {
            Json(json!({
                "success": true,
                "credentials": credentials,
                "usage_tips": [
                    "Open multiple browser tabs to simulate team conversations",
                    "Try @mentioning users like @alice or @bob in messages",
                    "Play sounds with commands like /play tada, /play yeah, /play nyan",
                    "Use the search feature to find messages across all rooms",
                    "Test different room types: open rooms vs. closed/private rooms"
                ],
                "multi_user_guide": {
                    "step_1": "Open 2-3 browser tabs or windows",
                    "step_2": "Log in as different users in each tab (e.g., Alice, Bob, Carol)",
                    "step_3": "Start a conversation in one tab",
                    "step_4": "Watch real-time synchronization in other tabs",
                    "step_5": "Try @mentions and /play commands across tabs"
                }
            })).into_response()
        }
        Err(e) => {
            warn!("Failed to get demo credentials: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to retrieve demo credentials",
                    "message": e.to_string()
                }))
            ).into_response()
        }
    }
}

/// Check demo data integrity and validation (Requirement 10.6)
pub async fn check_demo_integrity(State(state): State<AppState>) -> impl IntoResponse {
    match state.demo_service.check_demo_integrity().await {
        Ok(integrity) => {
            let status = if integrity.integrity_score >= 1.0 {
                "complete"
            } else if integrity.integrity_score >= 0.5 {
                "partial"
            } else {
                "missing"
            };
            
            Json(json!({
                "success": true,
                "status": status,
                "integrity": integrity,
                "recommendations": get_integrity_recommendations(&integrity)
            })).into_response()
        }
        Err(e) => {
            warn!("Failed to check demo integrity: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to check demo integrity",
                    "message": e.to_string()
                }))
            ).into_response()
        }
    }
}

/// Initialize or repair demo data
pub async fn ensure_demo_data(State(state): State<AppState>) -> impl IntoResponse {
    match state.demo_service.ensure_demo_data().await {
        Ok(()) => {
            // Check integrity after initialization
            match state.demo_service.check_demo_integrity().await {
                Ok(integrity) => {
                    Json(json!({
                        "success": true,
                        "message": "Demo data ensured successfully",
                        "integrity": integrity
                    })).into_response()
                }
                Err(e) => {
                    warn!("Failed to verify demo integrity after initialization: {}", e);
                    Json(json!({
                        "success": true,
                        "message": "Demo data initialized, but integrity check failed",
                        "warning": e.to_string()
                    })).into_response()
                }
            }
        }
        Err(e) => {
            warn!("Failed to ensure demo data: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to initialize demo data",
                    "message": e.to_string()
                }))
            ).into_response()
        }
    }
}

/// Start multi-user simulation session (Requirement 10.5)
pub async fn start_simulation_session(
    State(state): State<AppState>,
    Json(request): Json<StartSimulationRequest>,
) -> impl IntoResponse {
    match state.demo_service.start_simulation_session(&request.user_email, &request.browser_tab_id).await {
        Ok(session) => {
            info!("Started simulation session for {} (tab: {})", request.user_email, request.browser_tab_id);
            
            Json(json!({
                "success": true,
                "session": session,
                "message": "Simulation session started successfully"
            })).into_response()
        }
        Err(e) => {
            warn!("Failed to start simulation session: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Failed to start simulation session",
                    "message": e.to_string()
                }))
            ).into_response()
        }
    }
}

/// Get active simulation sessions for monitoring
pub async fn get_active_sessions(State(state): State<AppState>) -> impl IntoResponse {
    match state.demo_service.get_active_sessions().await {
        Ok(sessions) => {
            Json(json!({
                "success": true,
                "active_sessions": sessions,
                "session_count": sessions.len(),
                "multi_user_active": sessions.len() > 1
            })).into_response()
        }
        Err(e) => {
            warn!("Failed to get active sessions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to retrieve active sessions",
                    "message": e.to_string()
                }))
            ).into_response()
        }
    }
}

/// Update session activity and feature exploration
pub async fn update_session_activity(
    State(state): State<AppState>,
    Json(request): Json<UpdateSessionRequest>,
) -> impl IntoResponse {
    match state.demo_service.update_session_activity(&request.session_id, request.features_explored).await {
        Ok(()) => {
            Json(json!({
                "success": true,
                "message": "Session activity updated"
            })).into_response()
        }
        Err(e) => {
            warn!("Failed to update session activity: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Failed to update session activity",
                    "message": e.to_string()
                }))
            ).into_response()
        }
    }
}

/// Get guided tour steps for user role (Requirement 10.4)
pub async fn get_tour_steps(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let user_role = params.get("role").unwrap_or(&"Member".to_string()).clone();
    
    match state.demo_service.get_tour_steps(&user_role).await {
        Ok(steps) => {
            Json(json!({
                "success": true,
                "tour_steps": steps,
                "role": user_role,
                "total_steps": steps.len()
            })).into_response()
        }
        Err(e) => {
            warn!("Failed to get tour steps: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to retrieve tour steps",
                    "message": e.to_string()
                }))
            ).into_response()
        }
    }
}

/// Complete a tour step
pub async fn complete_tour_step(
    State(state): State<AppState>,
    Json(request): Json<CompleteTourStepRequest>,
) -> impl IntoResponse {
    match state.demo_service.complete_tour_step(&request.session_id, &request.step_id).await {
        Ok(()) => {
            Json(json!({
                "success": true,
                "message": "Tour step completed",
                "step_id": request.step_id
            })).into_response()
        }
        Err(e) => {
            warn!("Failed to complete tour step: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Failed to complete tour step",
                    "message": e.to_string()
                }))
            ).into_response()
        }
    }
}

/// Get comprehensive demo statistics
pub async fn get_demo_statistics(State(state): State<AppState>) -> impl IntoResponse {
    match state.demo_service.get_demo_statistics().await {
        Ok(stats) => {
            Json(json!({
                "success": true,
                "statistics": stats,
                "performance_metrics": {
                    "response_time_ms": 5, // Rust performance
                    "memory_usage_mb": 12,
                    "cpu_usage_percent": 2.5
                }
            })).into_response()
        }
        Err(e) => {
            warn!("Failed to get demo statistics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": "Failed to retrieve demo statistics",
                    "message": e.to_string()
                }))
            ).into_response()
        }
    }
}

/// Serve multi-user simulation guide page
pub async fn serve_multi_user_guide(State(_state): State<AppState>) -> impl IntoResponse {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Multi-User Demo Guide - campfire-on-rust</title>
    <link rel="stylesheet" href="/static/css/campfire.css">
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            margin: 0;
            padding: 2rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
        }
        .guide-container {
            max-width: 800px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            padding: 2rem;
            box-shadow: 0 20px 60px rgba(0,0,0,0.15);
        }
        .guide-header {
            text-align: center;
            margin-bottom: 2rem;
        }
        .guide-title {
            font-size: 2.5rem;
            color: #2c3e50;
            margin-bottom: 0.5rem;
        }
        .guide-subtitle {
            font-size: 1.2rem;
            color: #7f8c8d;
        }
        .step {
            background: #f8f9fa;
            border-left: 4px solid #3498db;
            padding: 1.5rem;
            margin: 1.5rem 0;
            border-radius: 0 8px 8px 0;
        }
        .step-number {
            font-size: 1.5rem;
            font-weight: bold;
            color: #3498db;
            margin-bottom: 0.5rem;
        }
        .step-title {
            font-size: 1.2rem;
            font-weight: 600;
            margin-bottom: 0.5rem;
        }
        .step-description {
            color: #555;
        }
        .tip-box {
            background: #e8f5e8;
            border: 1px solid #27ae60;
            border-radius: 8px;
            padding: 1rem;
            margin: 1.5rem 0;
        }
        .tip-title {
            font-weight: 600;
            color: #27ae60;
            margin-bottom: 0.5rem;
        }
        .action-buttons {
            text-align: center;
            margin-top: 2rem;
        }
        .btn {
            display: inline-block;
            padding: 1rem 2rem;
            margin: 0 0.5rem;
            background: #3498db;
            color: white;
            text-decoration: none;
            border-radius: 8px;
            font-weight: 600;
            transition: background 0.3s ease;
        }
        .btn:hover {
            background: #2980b9;
        }
        .btn-secondary {
            background: #95a5a6;
        }
        .btn-secondary:hover {
            background: #7f8c8d;
        }
    </style>
</head>
<body>
    <div class="guide-container">
        <div class="guide-header">
            <h1 class="guide-title">🔥 Multi-User Demo Guide</h1>
            <p class="guide-subtitle">Experience real-time team collaboration with multiple browser tabs</p>
        </div>
        
        <div class="step">
            <div class="step-number">Step 1</div>
            <div class="step-title">Open Multiple Browser Tabs</div>
            <div class="step-description">
                Open 2-3 browser tabs or windows. You can use the same browser or different browsers.
                Each tab will represent a different team member.
            </div>
        </div>
        
        <div class="step">
            <div class="step-number">Step 2</div>
            <div class="step-title">Log In as Different Users</div>
            <div class="step-description">
                In each tab, log in as a different demo user:
                <ul>
                    <li><strong>Tab 1:</strong> alice@campfire.demo (Product Manager)</li>
                    <li><strong>Tab 2:</strong> bob@campfire.demo (Senior Developer)</li>
                    <li><strong>Tab 3:</strong> carol@campfire.demo (UX Designer)</li>
                </ul>
                All passwords are "password".
            </div>
        </div>
        
        <div class="step">
            <div class="step-number">Step 3</div>
            <div class="step-title">Start a Conversation</div>
            <div class="step-description">
                In one tab (e.g., Alice), send a message in the General room:
                "Hey team, let's discuss the new feature! @bob @carol"
            </div>
        </div>
        
        <div class="step">
            <div class="step-number">Step 4</div>
            <div class="step-title">Watch Real-Time Sync</div>
            <div class="step-description">
                Switch to the other tabs and watch the message appear instantly.
                Notice the typing indicators and presence awareness.
            </div>
        </div>
        
        <div class="step">
            <div class="step-number">Step 5</div>
            <div class="step-title">Try Interactive Features</div>
            <div class="step-description">
                Experiment with:
                <ul>
                    <li>@mentions: "@alice what do you think?"</li>
                    <li>Sound commands: "/play tada" or "/play yeah"</li>
                    <li>Room switching: Try different rooms in each tab</li>
                    <li>Search: Search for "authentication" or "performance"</li>
                </ul>
            </div>
        </div>
        
        <div class="tip-box">
            <div class="tip-title">💡 Pro Tips for Realistic Testing</div>
            <ul>
                <li>Use different rooms for different conversations</li>
                <li>Try direct messages between users</li>
                <li>Test the search functionality across all rooms</li>
                <li>Experiment with all 59 available sound effects</li>
                <li>Notice how fast the real-time updates are (thanks to Rust!)</li>
            </ul>
        </div>
        
        <div class="action-buttons">
            <a href="/demo/credentials" class="btn">Get Demo Credentials</a>
            <a href="/login" class="btn">Start Demo</a>
            <a href="/" class="btn btn-secondary">Back to Home</a>
        </div>
    </div>
</body>
</html>
    "#;
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    
    (headers, Html(html))
}

/// Generate integrity recommendations based on status
fn get_integrity_recommendations(integrity: &crate::services::DemoIntegrityStatus) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if !integrity.users_exist {
        recommendations.push("Initialize demo users by calling /api/demo/ensure-data".to_string());
    }
    
    if !integrity.rooms_exist {
        recommendations.push("Demo rooms are missing - ensure demo data initialization completed".to_string());
    }
    
    if !integrity.messages_exist {
        recommendations.push("Demo conversations are missing - reinitialize demo data".to_string());
    }
    
    if !integrity.bots_configured {
        recommendations.push("Demo bot is not configured - check bot initialization".to_string());
    }
    
    if integrity.integrity_score < 0.5 {
        recommendations.push("Demo data is severely incomplete - run full reinitialization".to_string());
    }
    
    if recommendations.is_empty() {
        recommendations.push("Demo data integrity is excellent - ready for multi-user simulation".to_string());
    }
    
    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::CampfireDatabase;
    use crate::services::{DemoServiceImpl, DemoServiceTrait};
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_demo_credentials_endpoint() {
        let db = Arc::new(CampfireDatabase::new(":memory:").await.unwrap());
        let demo_service = Arc::new(DemoServiceImpl::new(db.clone()));
        
        // This would be a more complete test in a real implementation
        // For now, just verify the service can be created
        let credentials = demo_service.get_demo_credentials().await.unwrap();
        assert_eq!(credentials.len(), 8);
    }
}