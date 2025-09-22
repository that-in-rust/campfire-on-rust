use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, Level};
use tracing_subscriber;

use campfire_on_rust::{AppState, Database, AuthService};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Campfire Rust server...");

    // Initialize database
    let db = Database::new("campfire.db").await?;
    
    // Initialize auth service
    let auth_service = Arc::new(AuthService::new(Arc::new(db.clone())));
    
    let app_state = AppState { 
        db,
        auth_service,
    };

    // Build application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/login", post(campfire_on_rust::handlers::auth::login))
        .route("/api/auth/logout", post(campfire_on_rust::handlers::auth::logout))
        .route("/api/users/me", get(campfire_on_rust::handlers::users::get_current_user))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}