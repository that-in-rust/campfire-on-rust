use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use tracing::{error, info, Level};
use tracing_subscriber;

use campfire_on_rust::{
    AppState, CampfireDatabase, AuthService, RoomService, MessageService, 
    ConnectionManagerImpl, SearchService, PushNotificationServiceImpl, 
    VapidConfig, BotServiceImpl, health, metrics, shutdown
};
use campfire_on_rust::middleware::security;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Campfire Rust server...");

    // Initialize health check system
    health::init();

    // Initialize metrics system
    if let Err(e) = metrics::init_metrics() {
        error!("Failed to initialize metrics: {}", e);
        // Continue without metrics rather than failing
    }

    // Initialize shutdown coordinator
    let mut shutdown_coordinator = shutdown::ShutdownCoordinator::new();
    let shutdown_receiver = shutdown_coordinator.subscribe();

    // Start listening for shutdown signals
    shutdown_coordinator.listen_for_signals().await;

    // Run startup validation
    let mut startup_validator = shutdown::StartupValidator::new();
    startup_validator.add_check(shutdown::DatabaseConnectivityCheck::new("campfire.db".to_string()));
    startup_validator.add_check(shutdown::ConfigurationCheck::new("campfire".to_string()));
    startup_validator.add_check(shutdown::ServicesCheck::new(vec![
        "auth".to_string(),
        "messaging".to_string(),
        "push".to_string(),
    ]));

    if let Err(e) = startup_validator.validate_all().await {
        error!("Startup validation failed: {}", e);
        return Err(anyhow::anyhow!("Startup validation failed: {}", e));
    }

    // Initialize database
    let db = CampfireDatabase::new("campfire.db").await?;
    let db_arc = Arc::new(db.clone());
    
    // Initialize connection manager
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db_arc.clone()));
    
    // Initialize services
    let auth_service = Arc::new(AuthService::new(db_arc.clone()));
    let room_service = Arc::new(RoomService::new(db_arc.clone()));
    
    // Initialize push notification service
    let vapid_config = VapidConfig::default(); // In production, load from environment
    let push_service = Arc::new(PushNotificationServiceImpl::new(
        db.clone(),
        db.writer(),
        vapid_config,
    ));
    
    // Initialize message service with push notifications
    let message_service = Arc::new(MessageService::with_push_service(
        db_arc.clone(), 
        connection_manager,
        room_service.clone(),
        push_service.clone(),
    ));
    
    let search_service = Arc::new(SearchService::new(
        db_arc.clone(),
        room_service.clone()
    ));
    
    // Initialize bot service
    let bot_service = Arc::new(BotServiceImpl::new(
        db_arc.clone(),
        db.writer(),
        message_service.clone(),
    ));
    
    let app_state = AppState { 
        db,
        auth_service,
        room_service,
        message_service,
        search_service,
        push_service,
        bot_service,
    };

    // Setup resource manager for cleanup
    let mut resource_manager = shutdown::ResourceManager::new();
    resource_manager.add_resource(shutdown::DatabaseResource::new("campfire_db".to_string()));
    resource_manager.add_resource(shutdown::WebSocketResource::new("websocket_connections".to_string(), 0));

    // Add shutdown tasks
    let resource_manager_arc = Arc::new(resource_manager);
    let resource_manager_for_shutdown = resource_manager_arc.clone();
    
    shutdown_coordinator.add_task(
        "resource_cleanup".to_string(),
        Duration::from_secs(10),
        move || {
            let rm = resource_manager_for_shutdown.clone();
            tokio::spawn(async move {
                rm.cleanup_all().await;
            })
        }
    );

    // Build application with routes
    let app = Router::new()
        // HTML pages
        .route("/", get(campfire_on_rust::assets::serve_chat_interface))
        .route("/login", get(campfire_on_rust::assets::serve_login_page))
        .route("/manifest.json", get(campfire_on_rust::assets::serve_manifest))
        
        // Static assets
        .route("/static/*path", get(campfire_on_rust::assets::serve_static_asset))
        
        // Health and monitoring endpoints
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
        .route("/metrics", get(metrics::metrics_endpoint))
        .route("/metrics/summary", get(metrics::metrics_summary))
        
        // WebSocket
        .route("/ws", get(campfire_on_rust::handlers::websocket::websocket_handler))
        
        // API routes
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
        .route("/api/push/subscriptions", post(campfire_on_rust::handlers::push::create_push_subscription))
        .route("/api/push/subscriptions/:id", axum::routing::delete(campfire_on_rust::handlers::push::delete_push_subscription))
        .route("/api/push/preferences", get(campfire_on_rust::handlers::push::get_notification_preferences))
        .route("/api/push/preferences", axum::routing::put(campfire_on_rust::handlers::push::update_notification_preferences))
        .route("/api/push/vapid-key", get(campfire_on_rust::handlers::push::get_vapid_public_key))
        // Bot management endpoints (admin only)
        .route("/api/bots", get(campfire_on_rust::handlers::bot::list_bots))
        .route("/api/bots", post(campfire_on_rust::handlers::bot::create_bot))
        .route("/api/bots/:id", get(campfire_on_rust::handlers::bot::get_bot))
        .route("/api/bots/:id", axum::routing::put(campfire_on_rust::handlers::bot::update_bot))
        .route("/api/bots/:id", axum::routing::delete(campfire_on_rust::handlers::bot::delete_bot))
        .route("/api/bots/:id/reset-token", post(campfire_on_rust::handlers::bot::reset_bot_token))
        // Bot API endpoint (no session auth, uses bot key)
        .route("/rooms/:room_id/bot/:bot_key/messages", post(campfire_on_rust::handlers::bot::create_bot_message));
    
    #[cfg(debug_assertions)]
    let app = app.route("/api/push/test", post(campfire_on_rust::handlers::push::send_test_notification));
    
    let app = app
        .layer(middleware::from_fn(metrics::record_http_request))
        .layer(security::create_request_size_limit_layer())
        .layer(security::create_timeout_layer())
        .layer(security::create_production_cors_layer())
        .layer(security::create_security_headers_layer())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server with graceful shutdown
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server listening on {}", addr);
    
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            let mut shutdown_receiver = shutdown_receiver;
            if let Ok(signal) = shutdown_receiver.recv().await {
                info!("Received shutdown signal: {:?}", signal);
            }
        });

    // Run server and wait for shutdown
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                error!("Server error: {}", e);
                return Err(e.into());
            }
        }
        _ = shutdown_coordinator.wait_for_shutdown() => {
            info!("Shutdown signal received, stopping server...");
        }
    }

    // Perform final cleanup
    info!("Performing final cleanup...");
    shutdown_coordinator.shutdown(shutdown::ShutdownSignal::Application).await;

    info!("Campfire server shutdown complete");
    Ok(())
}

