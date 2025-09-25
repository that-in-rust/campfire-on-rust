use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

use campfire_on_rust::{
    AppState, CampfireDatabase, AuthService, RoomService, MessageService, 
    ConnectionManagerImpl, SearchService, PushNotificationServiceImpl, 
    VapidConfig, BotServiceImpl, SetupServiceImpl, health, metrics, shutdown, config, logging, demo
};
use campfire_on_rust::middleware::security;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if it exists
    if let Err(e) = dotenvy::dotenv() {
        // Only warn if the error is not "file not found"
        if !e.to_string().contains("not found") {
            eprintln!("Warning: Failed to load .env file: {}", e);
        }
    }
    
    // Load configuration from environment
    let config = config::Config::from_env()?;
    
    // Initialize structured logging
    logging::init_logging(&config)?;

    info!(
        version = env!("CARGO_PKG_VERSION"),
        host = %config.server.bind_address.ip(),
        port = config.server.bind_address.port(),
        "Starting Campfire Rust server"
    );

    // Initialize health check system
    health::init();

    // Initialize metrics system if enabled
    if config.metrics.enabled {
        if let Err(e) = metrics::init_metrics() {
            error!("Failed to initialize metrics: {}", e);
            // Continue without metrics rather than failing
        }
    } else {
        info!("Metrics disabled by configuration");
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

    // Initialize database with configuration
    let db = CampfireDatabase::new(&config.database.database_url).await?;
    let db_arc = Arc::new(db.clone());
    
    // Initialize demo data if demo mode is enabled
    if config.features.demo_mode {
        let demo_initializer = demo::DemoDataInitializer::new(db_arc.clone());
        if let Err(e) = demo_initializer.initialize_if_needed().await {
            warn!("Failed to initialize demo data: {}", e);
            // Continue without demo data rather than failing
        }
    }
    
    // Initialize connection manager
    let connection_manager = Arc::new(ConnectionManagerImpl::new(db_arc.clone()));
    
    // Initialize services
    let auth_service = Arc::new(AuthService::new(db_arc.clone()));
    let room_service = Arc::new(RoomService::new(db_arc.clone()));
    
    // Initialize push notification service with configuration
    let vapid_config = if config.push.enabled {
        VapidConfig {
            private_key: config.push.vapid_private_key.clone().unwrap_or_default(),
            public_key: config.push.vapid_public_key.clone().unwrap_or_default(),
            subject: config.push.vapid_subject.clone(),
        }
    } else {
        VapidConfig::default()
    };
    
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
    
    // Initialize setup service
    let setup_service = Arc::new(SetupServiceImpl::new(db.clone()));
    
    let app_state = AppState { 
        db,
        auth_service,
        room_service,
        message_service,
        search_service,
        push_service,
        bot_service,
        setup_service,
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

    // Build application with routes based on feature flags
    let mut app = Router::new()
        // HTML pages with demo mode awareness
        .route("/", get(campfire_on_rust::handlers::pages::serve_root_page))
        .route("/chat", get(campfire_on_rust::assets::serve_chat_interface))
        .route("/login", get(campfire_on_rust::handlers::pages::serve_login_page))
        .route("/demo", get(campfire_on_rust::assets::serve_demo_page))
        .route("/manifest.json", get(campfire_on_rust::assets::serve_manifest))
        
        // Demo API endpoints
        .route("/api/demo/status", get(campfire_on_rust::handlers::pages::demo_status))
        .route("/api/demo/initialize", post(campfire_on_rust::handlers::pages::initialize_demo))
        .route("/api/demo/credentials", get(campfire_on_rust::handlers::pages::get_demo_credentials))
        
        // First-run setup endpoints
        .route("/setup", get(campfire_on_rust::handlers::setup::serve_setup_page))
        .route("/api/setup/status", get(campfire_on_rust::handlers::setup::get_setup_status))
        .route("/api/setup/admin", post(campfire_on_rust::handlers::setup::create_admin_account))
        .route("/api/setup/environment", get(campfire_on_rust::handlers::setup::validate_environment))
        
        // Static assets
        .route("/static/*path", get(campfire_on_rust::assets::serve_static_asset))
        
        // Health and monitoring endpoints
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check));
    
    // Add metrics endpoints if enabled
    if config.metrics.enabled {
        app = app
            .route(&config.metrics.endpoint, get(metrics::metrics_endpoint))
            .route("/metrics/summary", get(metrics::metrics_summary));
    }
    
    // Add WebSocket endpoint if enabled (with setup completion validation)
    if config.features.websockets {
        let websocket_routes = Router::new()
            .route("/ws", get(campfire_on_rust::handlers::websocket::websocket_handler))
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                campfire_on_rust::middleware::setup::setup_completion_middleware
            ));
        app = app.merge(websocket_routes);
    }
    
    // Core API routes with setup completion validation
    let protected_api_routes = Router::new()
        .route("/api/auth/login", post(campfire_on_rust::handlers::auth::login))
        .route("/api/auth/logout", post(campfire_on_rust::handlers::auth::logout))
        .route("/api/users/me", get(campfire_on_rust::handlers::users::get_current_user))
        .route("/api/rooms", get(campfire_on_rust::handlers::rooms::get_rooms))
        .route("/api/rooms", post(campfire_on_rust::handlers::rooms::create_room))
        .route("/api/rooms/:id", get(campfire_on_rust::handlers::rooms::get_room))
        .route("/api/rooms/:id/members", post(campfire_on_rust::handlers::rooms::add_room_member))
        .route("/api/rooms/:id/messages", get(campfire_on_rust::handlers::messages::get_messages))
        .route("/api/rooms/:id/messages", post(campfire_on_rust::handlers::messages::create_message))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            campfire_on_rust::middleware::setup::setup_completion_middleware
        ));
    
    app = app.merge(protected_api_routes);
    
    // Add search endpoints if enabled (with setup completion validation)
    if config.features.search {
        let search_routes = Router::new()
            .route("/api/search", get(campfire_on_rust::handlers::search::search_messages))
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                campfire_on_rust::middleware::setup::setup_completion_middleware
            ));
        app = app.merge(search_routes);
    }
    
    // Add sound endpoints if enabled (with setup completion validation)
    if config.features.sounds {
        let sound_routes = Router::new()
            .route("/api/sounds", get(campfire_on_rust::handlers::sounds::list_sounds))
            .route("/api/sounds/:sound_name", get(campfire_on_rust::handlers::sounds::get_sound))
            .route("/api/sounds/:sound_name/info", get(campfire_on_rust::handlers::sounds::get_sound_info))
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                campfire_on_rust::middleware::setup::setup_completion_middleware
            ));
        app = app.merge(sound_routes);
    }
    
    // Add push notification endpoints if enabled (with setup completion validation)
    if config.features.push_notifications {
        let mut push_routes = Router::new()
            .route("/api/push/subscriptions", post(campfire_on_rust::handlers::push::create_push_subscription))
            .route("/api/push/subscriptions/:id", axum::routing::delete(campfire_on_rust::handlers::push::delete_push_subscription))
            .route("/api/push/preferences", get(campfire_on_rust::handlers::push::get_notification_preferences))
            .route("/api/push/preferences", axum::routing::put(campfire_on_rust::handlers::push::update_notification_preferences))
            .route("/api/push/vapid-key", get(campfire_on_rust::handlers::push::get_vapid_public_key));
        
        #[cfg(debug_assertions)]
        {
            push_routes = push_routes.route("/api/push/test", post(campfire_on_rust::handlers::push::send_test_notification));
        }
        
        let push_routes = push_routes.layer(middleware::from_fn_with_state(
            app_state.clone(),
            campfire_on_rust::middleware::setup::setup_completion_middleware
        ));
        app = app.merge(push_routes);
    }
    
    // Add bot endpoints if enabled (with setup completion validation)
    if config.features.bot_api {
        let bot_routes = Router::new()
            .route("/api/bots", get(campfire_on_rust::handlers::bot::list_bots))
            .route("/api/bots", post(campfire_on_rust::handlers::bot::create_bot))
            .route("/api/bots/:id", get(campfire_on_rust::handlers::bot::get_bot))
            .route("/api/bots/:id", axum::routing::put(campfire_on_rust::handlers::bot::update_bot))
            .route("/api/bots/:id", axum::routing::delete(campfire_on_rust::handlers::bot::delete_bot))
            .route("/api/bots/:id/reset-token", post(campfire_on_rust::handlers::bot::reset_bot_token))
            .route("/rooms/:room_id/bot/:bot_key/messages", post(campfire_on_rust::handlers::bot::create_bot_message))
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                campfire_on_rust::middleware::setup::setup_completion_middleware
            ));
        app = app.merge(bot_routes);
    }
    
    // Apply middleware layers
    if config.metrics.enabled {
        app = app.layer(middleware::from_fn(metrics::record_http_request));
    }
    
    if config.logging.trace_requests {
        app = app.layer(middleware::from_fn(logging::middleware::trace_requests));
    }
    
    // Add setup detection middleware for automatic redirection to setup when needed
    // This middleware runs early to catch first-run scenarios before other processing
    app = app.layer(middleware::from_fn_with_state(
        app_state.clone(),
        campfire_on_rust::middleware::setup::setup_detection_middleware
    ));
    
    let app = app
        .layer(security::create_request_size_limit_layer_with_size(config.server.max_request_size))
        .layer(security::create_timeout_layer_with_duration(config.request_timeout()))
        .layer(security::create_cors_layer(&config.security.cors_origins, config.security.force_https))
        .layer(security::create_security_headers_layer(config.security.force_https))
        .with_state(app_state);

    // Start server with graceful shutdown
    let addr = config.server.bind_address;
    info!(
        address = %addr,
        features = ?config.features,
        "Server starting with configuration"
    );
    
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

