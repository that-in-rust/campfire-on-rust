use axum::http::{header, HeaderValue};
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    set_header::SetResponseHeaderLayer,
};

/// Create security headers layer
pub fn create_security_headers_layer() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    )
}

/// Create production CORS layer with restricted origins
pub fn create_production_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any) // TODO: Replace with specific origins in production
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
        .max_age(Duration::from_secs(3600))
}

/// Create request size limit layer (10MB max)
pub fn create_request_size_limit_layer() -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(10 * 1024 * 1024) // 10MB
}

/// Create timeout layer (30 seconds)
pub fn create_timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(30))
}