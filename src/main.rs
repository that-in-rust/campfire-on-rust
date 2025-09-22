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

use campfire_on_rust::{AppState, CampfireDatabase, AuthService, RoomService, MessageService, ConnectionManagerImpl, SearchService};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Campfire Rust server...");

    // Initialize database
    let db = CampfireDatabase::new("campfire.db").await?;
    let db_arc = Arc::new(db.clone());
    
    // Initialize connection manager
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db_arc.clone()));
    
    // Initialize services
    let auth_service = Arc::new(AuthService::new(db_arc.clone()));
    let room_service = Arc::new(RoomService::new(db_arc.clone()));
    let message_service = Arc::new(MessageService::new(
        db_arc.clone(), 
        connection_manager,
        room_service.clone()
    ));
    let search_service = Arc::new(SearchService::new(
        db_arc.clone(),
        room_service.clone()
    ));
    
    let app_state = AppState { 
        db,
        auth_service,
        room_service,
        message_service,
        search_service,
    };

    // Build application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws", get(campfire_on_rust::handlers::websocket::websocket_handler))
        .route("/api/auth/login", post(campfire_on_rust::handlers::auth::login))
        .route("/api/auth/logout", post(campfire_on_rust::handlers::auth::logout))
        .route("/api/users/me", get(campfire_on_rust::handlers::users::get_current_user))
        .route("/api/rooms", get(campfire_on_rust::handlers::rooms::get_rooms))
        .route("/api/rooms", post(campfire_on_rust::handlers::rooms::create_room))
        .route("/api/rooms/:id", get(campfire_on_rust::handlers::rooms::get_room))
        .route("/api/rooms/:id/members", post(campfire_on_rust::handlers::rooms::add_room_member))
        .route("/api/rooms/:id/messages", get(campfire_on_rust::handlers::messages::get_messages))
        .route("/api/rooms/:id/messages", post(campfire_on_rust::handlers::messages::create_message))
        .route("/api/search", get(campfire_on_rust::handlers::search::search_messages))
        .route("/api/sounds", get(campfire_on_rust::handlers::sounds::list_sounds))
        .route("/api/sounds/:sound_name", get(campfire_on_rust::handlers::sounds::get_sound))
        .route("/api/sounds/:sound_name/info", get(campfire_on_rust::handlers::sounds::get_sound_info))
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