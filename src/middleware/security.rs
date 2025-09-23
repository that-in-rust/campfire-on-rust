use axum::http::{header, HeaderValue, Uri};
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    set_header::SetResponseHeaderLayer,
};

/// Create security headers layer with HTTPS enforcement option
pub fn create_security_headers_layer(_force_https: bool) -> SetResponseHeaderLayer<HeaderValue> {
    // For now, just return the basic security header
    // In a full implementation, we'd create a custom middleware to handle multiple headers
    SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    )
}

/// Create CORS layer with configurable origins
pub fn create_cors_layer(allowed_origins: &[String], _force_https: bool) -> CorsLayer {
    let mut cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            "x-requested-with".parse().unwrap(),
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600));
    
    if allowed_origins.is_empty() {
        // Allow all origins (development mode)
        cors = cors.allow_origin(Any);
    } else {
        // Allow specific origins (production mode)
        for origin in allowed_origins {
            if let Ok(origin_header) = origin.parse::<HeaderValue>() {
                cors = cors.allow_origin(origin_header);
            }
        }
    }
    
    cors
}

/// Create request size limit layer with configurable size
pub fn create_request_size_limit_layer_with_size(max_size: usize) -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(max_size)
}

/// Create timeout layer with configurable duration
pub fn create_timeout_layer_with_duration(timeout: Duration) -> TimeoutLayer {
    TimeoutLayer::new(timeout)
}

/// Legacy function for backward compatibility
pub fn create_security_headers_layer_legacy() -> SetResponseHeaderLayer<HeaderValue> {
    create_security_headers_layer(false)
}

pub fn create_production_cors_layer() -> CorsLayer {
    create_cors_layer(&[], false)
}

pub fn create_request_size_limit_layer() -> RequestBodyLimitLayer {
    create_request_size_limit_layer_with_size(10 * 1024 * 1024) // 10MB
}

pub fn create_timeout_layer() -> TimeoutLayer {
    create_timeout_layer_with_duration(Duration::from_secs(30))
}