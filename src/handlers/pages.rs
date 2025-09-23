use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue},
    response::{Html, IntoResponse},
};

use crate::AppState;

/// Serve login page with demo mode awareness
pub async fn serve_login_page(State(state): State<AppState>) -> impl IntoResponse {
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
    
    (headers, Html(html))
}

/// Serve root page based on demo mode
pub async fn serve_root_page(State(state): State<AppState>) -> impl IntoResponse {
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