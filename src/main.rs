use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, Level};
use tracing_subscriber;

mod models;
mod services;
mod errors;
mod handlers;
mod database;

use crate::database::Database;
use crate::models::*;

#[derive(Clone)]
pub struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Campfire Rust server...");

    // Initialize database
    let db = Database::new("campfire.db").await?;
    
    let app_state = AppState { db };

    // Build application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/logout", post(handlers::auth::logout))
        .route("/api/users/me", get(handlers::users::get_current_user))
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