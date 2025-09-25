pub mod session;
pub mod security;
pub mod setup;
pub mod error_handling;
pub mod rate_limiting;

pub use session::{AuthenticatedUser, OptionalAuthenticatedUser, SessionToken};
pub use setup::{setup_detection_middleware, setup_completion_middleware};
pub use error_handling::{
    global_error_handler, 
    error_recovery_middleware, 
    CircuitBreakerMiddleware,
    panic_recovery_middleware,
    timeout_middleware,
};
pub use rate_limiting::{RateLimitingMiddleware, RateLimitConfig, create_rate_limiting_layer};
pub use security::{
    CsrfProtection, BotAbuseProtection, 
    create_csrf_protection_layer, create_bot_abuse_protection_layer,
    security_headers_middleware, input_sanitization_middleware,
};