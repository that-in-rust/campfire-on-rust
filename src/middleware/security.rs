use axum::{
    extract::Request,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
};

/// Security headers middleware
pub async fn security_headers_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Content Security Policy - restrictive for security
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             connect-src 'self' ws: wss:; \
             font-src 'self'; \
             object-src 'none'; \
             media-src 'self'; \
             frame-src 'none'; \
             base-uri 'self'; \
             form-action 'self'"
        ),
    );
    
    // HTTP Strict Transport Security - enforce HTTPS
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );
    
    // X-Frame-Options - prevent clickjacking
    headers.insert(
        "x-frame-options".parse().unwrap(),
        HeaderValue::from_static("DENY"),
    );
    
    // X-Content-Type-Options - prevent MIME sniffing
    headers.insert(
        "x-content-type-options".parse().unwrap(),
        HeaderValue::from_static("nosniff"),
    );
    
    // X-XSS-Protection - enable XSS filtering
    headers.insert(
        "x-xss-protection".parse().unwrap(),
        HeaderValue::from_static("1; mode=block"),
    );
    
    // Referrer Policy - control referrer information
    headers.insert(
        "referrer-policy".parse().unwrap(),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    
    // Permissions Policy - restrict browser features
    headers.insert(
        "permissions-policy".parse().unwrap(),
        HeaderValue::from_static(
            "camera=(), microphone=(), geolocation=(), payment=(), usb=(), \
             magnetometer=(), gyroscope=(), accelerometer=()"
        ),
    );
    
    Ok(response)
}

/// Create rate limiting layer
pub fn create_rate_limiting_layer() -> GovernorLayer<SmartIpKeyExtractor> {
    // Configure rate limiting: 100 requests per minute per IP
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_minute(100)
            .burst_size(20)
            .finish()
            .unwrap(),
    );
    
    GovernorLayer {
        config: governor_conf,
        key_extractor: SmartIpKeyExtractor,
    }
}

/// Create production CORS layer with restricted origins
pub fn create_production_cors_layer() -> CorsLayer {
    CorsLayer::new()
        // In production, replace with actual allowed origins
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

/// Create complete security middleware stack
pub fn create_security_middleware_stack() -> ServiceBuilder<
    tower::layer::util::Stack<
        tower::layer::util::Stack<
            tower::layer::util::Stack<
                tower::layer::util::Stack<
                    GovernorLayer<SmartIpKeyExtractor>,
                    RequestBodyLimitLayer,
                >,
                TimeoutLayer,
            >,
            CorsLayer,
        >,
        axum::middleware::FromFnLayer<
            fn(Request, Next) -> impl std::future::Future<Output = Result<Response, StatusCode>>,
            Request,
            Response,
        >,
    >,
> {
    ServiceBuilder::new()
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(create_production_cors_layer())
        .layer(create_timeout_layer())
        .layer(create_request_size_limit_layer())
        .layer(create_rate_limiting_layer())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, response::Response};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_security_headers_middleware() {
        let app = axum::Router::new()
            .route("/test", axum::routing::get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(security_headers_middleware));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let headers = response.headers();

        // Check that security headers are present
        assert!(headers.contains_key(header::CONTENT_SECURITY_POLICY));
        assert!(headers.contains_key(header::STRICT_TRANSPORT_SECURITY));
        assert!(headers.contains_key("x-frame-options"));
        assert!(headers.contains_key("x-content-type-options"));
        assert!(headers.contains_key("x-xss-protection"));
        assert!(headers.contains_key("referrer-policy"));
        assert!(headers.contains_key("permissions-policy"));
    }
}