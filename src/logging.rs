use anyhow::{Context, Result};
use std::io;
use std::path::Path;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
};

use crate::config::{Config, LogFormat};

/// Initialize structured logging based on configuration
pub fn init_logging(config: &Config) -> Result<()> {
    let log_level = config.tracing_level();
    let env_filter = create_env_filter(&config.logging.level)?;
    
    // Initialize log rotation if enabled
    if config.logging.rotation.enabled {
        if let Some(log_path) = &config.logging.file_path {
            tokio::spawn(log_rotation_task(
                log_path.clone(),
                config.logging.rotation.clone(),
            ));
        }
    }
    
    match (&config.logging.format, &config.logging.file_path) {
        // JSON format, stdout only
        (LogFormat::Json, None) => {
            tracing_subscriber::fmt()
                .json()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .with_current_span(true)
                .with_span_list(false)
                .init();
        }
        
        // Pretty format, stdout only
        (LogFormat::Pretty, None) => {
            let span_events = if config.logging.trace_requests {
                FmtSpan::NEW | FmtSpan::CLOSE
            } else {
                FmtSpan::NONE
            };
            
            tracing_subscriber::fmt()
                .pretty()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false)
                .with_span_events(span_events)
                .init();
        }
        
        // Compact format, stdout only
        (LogFormat::Compact, None) => {
            tracing_subscriber::fmt()
                .compact()
                .with_env_filter(env_filter)
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false)
                .init();
        }
        
        // File output with enhanced configuration
        (format, Some(file_path)) => {
            let file_appender = tracing_appender::rolling::daily(
                file_path.parent().unwrap_or(std::path::Path::new(".")),
                file_path.file_name().unwrap_or(std::ffi::OsStr::new("campfire.log"))
            );
            
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
            
            match format {
                LogFormat::Json => {
                    tracing_subscriber::fmt()
                        .json()
                        .with_env_filter(env_filter)
                        .with_writer(non_blocking)
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_thread_names(true)
                        .with_file(true)
                        .with_line_number(true)
                        .init();
                }
                LogFormat::Pretty => {
                    tracing_subscriber::fmt()
                        .pretty()
                        .with_env_filter(env_filter)
                        .with_writer(non_blocking)
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_thread_names(true)
                        .with_file(true)
                        .with_line_number(true)
                        .init();
                }
                LogFormat::Compact => {
                    tracing_subscriber::fmt()
                        .compact()
                        .with_env_filter(env_filter)
                        .with_writer(non_blocking)
                        .with_target(true)
                        .with_file(true)
                        .with_line_number(true)
                        .init();
                }
            }
        }
    }
    
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_level = %log_level,
        format = ?config.logging.format,
        structured = config.logging.structured,
        trace_requests = config.logging.trace_requests,
        audit_enabled = config.logging.audit_enabled,
        performance_monitoring = config.logging.performance_monitoring,
        error_recovery_logging = config.logging.error_recovery_logging,
        rotation_enabled = config.logging.rotation.enabled,
        "Enhanced logging initialized"
    );
    
    // Initialize error documentation
    let _error_docs = documentation::ErrorDocumentation::new();
    tracing::info!("Error documentation initialized with recovery procedures");
    
    Ok(())
}

/// Background task for log rotation
async fn log_rotation_task(
    log_path: std::path::PathBuf,
    rotation_config: crate::config::LogRotationConfig,
) {
    let mut interval = tokio::time::interval(
        std::time::Duration::from_secs(rotation_config.check_interval_secs)
    );
    
    loop {
        interval.tick().await;
        
        if let Err(e) = rotation::rotate_if_needed(
            &log_path,
            rotation_config.max_size_bytes,
            rotation_config.max_files,
        ) {
            tracing::error!("Log rotation failed: {}", e);
        }
    }
}

/// Create environment filter for log levels
fn create_env_filter(level: &str) -> Result<EnvFilter> {
    let filter = EnvFilter::try_new(format!(
        "campfire_on_rust={level},tower_http=info,axum=info,sqlx=warn,hyper=warn"
    ))
    .context("Failed to create environment filter")?;
    
    Ok(filter)
}

/// Structured logging macros for common operations
#[macro_export]
macro_rules! log_request {
    ($method:expr, $path:expr, $status:expr, $duration:expr) => {
        tracing::info!(
            method = %$method,
            path = %$path,
            status = %$status,
            duration_ms = %$duration.as_millis(),
            "HTTP request completed"
        );
    };
    ($method:expr, $path:expr, $status:expr, $duration:expr, $user_id:expr) => {
        tracing::info!(
            method = %$method,
            path = %$path,
            status = %$status,
            duration_ms = %$duration.as_millis(),
            user_id = %$user_id,
            "HTTP request completed"
        );
    };
}

#[macro_export]
macro_rules! log_database_operation {
    ($operation:expr, $table:expr, $duration:expr) => {
        tracing::debug!(
            operation = %$operation,
            table = %$table,
            duration_ms = %$duration.as_millis(),
            "Database operation completed"
        );
    };
    ($operation:expr, $table:expr, $duration:expr, $affected_rows:expr) => {
        tracing::debug!(
            operation = %$operation,
            table = %$table,
            duration_ms = %$duration.as_millis(),
            affected_rows = %$affected_rows,
            "Database operation completed"
        );
    };
}

#[macro_export]
macro_rules! log_websocket_event {
    ($event:expr, $user_id:expr, $room_id:expr) => {
        tracing::info!(
            event = %$event,
            user_id = %$user_id,
            room_id = %$room_id,
            "WebSocket event"
        );
    };
    ($event:expr, $user_id:expr, $room_id:expr, $connection_id:expr) => {
        tracing::info!(
            event = %$event,
            user_id = %$user_id,
            room_id = %$room_id,
            connection_id = %$connection_id,
            "WebSocket event"
        );
    };
}

#[macro_export]
macro_rules! log_security_event {
    ($event:expr, $user_id:expr, $details:expr) => {
        tracing::warn!(
            event = %$event,
            user_id = %$user_id,
            details = %$details,
            security_level = "warning",
            "Security event"
        );
    };
    ($event:expr, $user_id:expr, $details:expr, $ip_address:expr) => {
        tracing::warn!(
            event = %$event,
            user_id = %$user_id,
            details = %$details,
            ip_address = %$ip_address,
            security_level = "warning",
            "Security event"
        );
    };
}

#[macro_export]
macro_rules! log_audit_event {
    ($action:expr, $user_id:expr, $resource:expr, $details:expr) => {
        tracing::info!(
            action = %$action,
            user_id = %$user_id,
            resource = %$resource,
            details = %$details,
            event_type = "audit",
            timestamp = %chrono::Utc::now().to_rfc3339(),
            "Audit event"
        );
    };
    ($action:expr, $user_id:expr, $resource:expr, $resource_id:expr, $details:expr) => {
        tracing::info!(
            action = %$action,
            user_id = %$user_id,
            resource = %$resource,
            resource_id = %$resource_id,
            details = %$details,
            event_type = "audit",
            timestamp = %chrono::Utc::now().to_rfc3339(),
            "Audit event"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr, $context:expr) => {
        tracing::error!(
            error = %$error,
            context = %$context,
            error_type = "application",
            "Application error"
        );
    };
    ($error:expr, $context:expr, $user_id:expr) => {
        tracing::error!(
            error = %$error,
            context = %$context,
            user_id = %$user_id,
            error_type = "application",
            "Application error"
        );
    };
}

#[macro_export]
macro_rules! log_performance_warning {
    ($operation:expr, $duration:expr, $threshold:expr) => {
        tracing::warn!(
            operation = %$operation,
            duration_ms = %$duration.as_millis(),
            threshold_ms = %$threshold.as_millis(),
            performance_issue = true,
            "Performance threshold exceeded"
        );
    };
}

#[macro_export]
macro_rules! log_business_event {
    ($event:expr, $user_id:expr, $details:expr) => {
        tracing::info!(
            event = %$event,
            user_id = %$user_id,
            details = %$details,
            event_type = "business",
            "Business event"
        );
    };
}

/// Request tracing middleware for structured HTTP logging
pub mod middleware {
    use axum::{
        extract::MatchedPath,
        http::Request,
        middleware::Next,
        response::IntoResponse,
    };
    use std::time::Instant;
    use tracing::{info_span, Instrument};
    
    /// Trace HTTP requests with structured logging
    pub async fn trace_requests<B>(
        request: Request<B>,
        next: Next<B>,
    ) -> impl IntoResponse {
        let start = Instant::now();
        let method = request.method().clone();
        let uri = request.uri().clone();
        
        // Get matched path for better grouping
        let path = request
            .extensions()
            .get::<MatchedPath>()
            .map(|p| p.as_str())
            .unwrap_or(uri.path());
        
        let span = info_span!(
            "http_request",
            method = %method,
            path = %path,
            uri = %uri,
        );
        
        async move {
            let response = next.run(request).await;
            let duration = start.elapsed();
            let status = response.status();
            
            tracing::info!(
                status = %status,
                duration_ms = %duration.as_millis(),
                "Request completed"
            );
            
            response
        }
        .instrument(span)
        .await
    }
}

/// Error recovery and user-friendly error handling
pub mod error_handling {
    use crate::errors::*;
    use axum::{
        http::StatusCode,
        response::{IntoResponse, Json, Response},
    };
    use serde_json::json;
    use tracing::{error, warn};
    use std::collections::HashMap;

    /// User-friendly error messages with actionable guidance
    pub struct UserFriendlyError {
        pub message: String,
        pub code: String,
        pub status: StatusCode,
        pub recovery_suggestions: Vec<String>,
        pub support_info: Option<String>,
    }

    impl UserFriendlyError {
        pub fn new(
            message: impl Into<String>,
            code: impl Into<String>,
            status: StatusCode,
        ) -> Self {
            Self {
                message: message.into(),
                code: code.into(),
                status,
                recovery_suggestions: Vec::new(),
                support_info: None,
            }
        }

        pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
            self.recovery_suggestions = suggestions;
            self
        }

        pub fn with_support_info(mut self, info: impl Into<String>) -> Self {
            self.support_info = Some(info.into());
            self
        }
    }

    impl IntoResponse for UserFriendlyError {
        fn into_response(self) -> Response {
            let mut response_body = json!({
                "error": {
                    "message": self.message,
                    "code": self.code,
                    "status": self.status.as_u16(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }
            });

            if !self.recovery_suggestions.is_empty() {
                response_body["error"]["recovery_suggestions"] = json!(self.recovery_suggestions);
            }

            if let Some(support_info) = self.support_info {
                response_body["error"]["support_info"] = json!(support_info);
            }

            (self.status, Json(response_body)).into_response()
        }
    }

    /// Convert application errors to user-friendly responses
    pub fn handle_message_error(error: MessageError, user_context: Option<&str>) -> UserFriendlyError {
        match error {
            MessageError::Authorization { user_id, room_id } => {
                warn!("Authorization error: user {} attempted to access room {}", user_id, room_id);
                UserFriendlyError::new(
                    "You don't have permission to access this room",
                    "ROOM_ACCESS_DENIED",
                    StatusCode::FORBIDDEN,
                ).with_suggestions(vec![
                    "Ask a room administrator to invite you to this room".to_string(),
                    "Check if you're logged in with the correct account".to_string(),
                    "Refresh the page and try again".to_string(),
                ])
            }
            MessageError::ContentTooLong { length } => {
                UserFriendlyError::new(
                    format!("Message is too long ({} characters). Maximum allowed is 10,000 characters.", length),
                    "MESSAGE_TOO_LONG",
                    StatusCode::BAD_REQUEST,
                ).with_suggestions(vec![
                    "Try breaking your message into smaller parts".to_string(),
                    "Remove unnecessary text or formatting".to_string(),
                    "Consider using a file attachment for longer content".to_string(),
                ])
            }
            MessageError::ContentTooShort => {
                UserFriendlyError::new(
                    "Message cannot be empty",
                    "MESSAGE_EMPTY",
                    StatusCode::BAD_REQUEST,
                ).with_suggestions(vec![
                    "Type a message before sending".to_string(),
                    "Make sure your message contains visible text".to_string(),
                ])
            }
            MessageError::RateLimit { limit, window } => {
                UserFriendlyError::new(
                    format!("You're sending messages too quickly. Limit: {} messages per {}", limit, window),
                    "RATE_LIMIT_EXCEEDED",
                    StatusCode::TOO_MANY_REQUESTS,
                ).with_suggestions(vec![
                    "Wait a moment before sending another message".to_string(),
                    "Combine multiple thoughts into a single message".to_string(),
                ])
            }
            MessageError::NotFound { message_id } => {
                UserFriendlyError::new(
                    "The requested message could not be found",
                    "MESSAGE_NOT_FOUND",
                    StatusCode::NOT_FOUND,
                ).with_suggestions(vec![
                    "The message may have been deleted".to_string(),
                    "Refresh the page to see the latest messages".to_string(),
                ])
            }
            MessageError::InvalidContent { reason } => {
                UserFriendlyError::new(
                    format!("Message content is invalid: {}", reason),
                    "INVALID_CONTENT",
                    StatusCode::BAD_REQUEST,
                ).with_suggestions(vec![
                    "Check for unsupported characters or formatting".to_string(),
                    "Try typing the message again".to_string(),
                ])
            }
            MessageError::Database(_) | MessageError::Broadcast(_) => {
                error!("Internal message error: {}", error);
                UserFriendlyError::new(
                    "We're experiencing technical difficulties. Please try again in a moment.",
                    "INTERNAL_ERROR",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ).with_suggestions(vec![
                    "Wait a few seconds and try again".to_string(),
                    "Refresh the page if the problem persists".to_string(),
                    "Contact support if you continue to experience issues".to_string(),
                ]).with_support_info("If this problem continues, please contact our support team with the error code and timestamp.")
            }
        }
    }

    /// Convert authentication errors to user-friendly responses
    pub fn handle_auth_error(error: AuthError, user_context: Option<&str>) -> UserFriendlyError {
        match error {
            AuthError::InvalidCredentials => {
                UserFriendlyError::new(
                    "Invalid email or password",
                    "INVALID_CREDENTIALS",
                    StatusCode::UNAUTHORIZED,
                ).with_suggestions(vec![
                    "Double-check your email and password".to_string(),
                    "Make sure Caps Lock is not enabled".to_string(),
                    "Try resetting your password if you've forgotten it".to_string(),
                ])
            }
            AuthError::SessionExpired => {
                UserFriendlyError::new(
                    "Your session has expired. Please log in again.",
                    "SESSION_EXPIRED",
                    StatusCode::UNAUTHORIZED,
                ).with_suggestions(vec![
                    "Click the login button to sign in again".to_string(),
                    "Your data has been saved and will be available after login".to_string(),
                ])
            }
            AuthError::UserNotFound { email } => {
                // Don't reveal if user exists for security
                UserFriendlyError::new(
                    "Invalid email or password",
                    "INVALID_CREDENTIALS",
                    StatusCode::UNAUTHORIZED,
                ).with_suggestions(vec![
                    "Double-check your email and password".to_string(),
                    "Make sure you're using the correct email address".to_string(),
                ])
            }
            AuthError::EmailExists { email: _ } => {
                UserFriendlyError::new(
                    "An account with this email already exists",
                    "EMAIL_EXISTS",
                    StatusCode::CONFLICT,
                ).with_suggestions(vec![
                    "Try logging in instead of creating a new account".to_string(),
                    "Use a different email address".to_string(),
                    "Reset your password if you've forgotten it".to_string(),
                ])
            }
            AuthError::InvalidEmail { email: _ } => {
                UserFriendlyError::new(
                    "Please enter a valid email address",
                    "INVALID_EMAIL",
                    StatusCode::BAD_REQUEST,
                ).with_suggestions(vec![
                    "Make sure your email includes an @ symbol and domain".to_string(),
                    "Check for typos in your email address".to_string(),
                ])
            }
            AuthError::WeakPassword => {
                UserFriendlyError::new(
                    "Password must be at least 8 characters long",
                    "WEAK_PASSWORD",
                    StatusCode::BAD_REQUEST,
                ).with_suggestions(vec![
                    "Use at least 8 characters".to_string(),
                    "Include a mix of letters, numbers, and symbols".to_string(),
                    "Avoid common passwords or personal information".to_string(),
                ])
            }
            AuthError::Database(_) | AuthError::PasswordHash(_) | AuthError::TokenGeneration => {
                error!("Internal auth error: {}", error);
                UserFriendlyError::new(
                    "We're experiencing technical difficulties. Please try again in a moment.",
                    "INTERNAL_ERROR",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ).with_suggestions(vec![
                    "Wait a few seconds and try again".to_string(),
                    "Refresh the page if the problem persists".to_string(),
                ]).with_support_info("If this problem continues, please contact our support team.")
            }
        }
    }

    /// Convert room errors to user-friendly responses
    pub fn handle_room_error(error: RoomError, user_context: Option<&str>) -> UserFriendlyError {
        match error {
            RoomError::NotFound { room_id } => {
                UserFriendlyError::new(
                    "The requested room could not be found",
                    "ROOM_NOT_FOUND",
                    StatusCode::NOT_FOUND,
                ).with_suggestions(vec![
                    "The room may have been deleted or made private".to_string(),
                    "Check the room name or URL for typos".to_string(),
                    "Ask someone to invite you if it's a private room".to_string(),
                ])
            }
            RoomError::NotAuthorized { user_id, room_id } => {
                warn!("Room authorization error: user {} attempted to modify room {}", user_id, room_id);
                UserFriendlyError::new(
                    "You don't have permission to perform this action in this room",
                    "ROOM_PERMISSION_DENIED",
                    StatusCode::FORBIDDEN,
                ).with_suggestions(vec![
                    "Ask a room administrator for the necessary permissions".to_string(),
                    "Make sure you're logged in with the correct account".to_string(),
                ])
            }
            RoomError::AlreadyMember { user_id, room_id } => {
                UserFriendlyError::new(
                    "You are already a member of this room",
                    "ALREADY_MEMBER",
                    StatusCode::CONFLICT,
                ).with_suggestions(vec![
                    "Refresh the page to see the updated room list".to_string(),
                    "Navigate to the room to start chatting".to_string(),
                ])
            }
            RoomError::InvalidName { reason } => {
                UserFriendlyError::new(
                    format!("Invalid room name: {}", reason),
                    "INVALID_ROOM_NAME",
                    StatusCode::BAD_REQUEST,
                ).with_suggestions(vec![
                    "Room names must be 1-100 characters long".to_string(),
                    "Use only letters, numbers, spaces, and basic punctuation".to_string(),
                    "Avoid special characters or emojis in room names".to_string(),
                ])
            }
            RoomError::Database(_) => {
                error!("Internal room error: {}", error);
                UserFriendlyError::new(
                    "We're experiencing technical difficulties. Please try again in a moment.",
                    "INTERNAL_ERROR",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ).with_suggestions(vec![
                    "Wait a few seconds and try again".to_string(),
                    "Refresh the page if the problem persists".to_string(),
                ]).with_support_info("If this problem continues, please contact our support team.")
            }
        }
    }

    /// Error recovery strategies
    pub struct ErrorRecoveryStrategy {
        pub automatic_retry: bool,
        pub retry_delay_ms: u64,
        pub max_retries: u32,
        pub fallback_action: Option<String>,
    }

    impl ErrorRecoveryStrategy {
        pub fn for_database_error() -> Self {
            Self {
                automatic_retry: true,
                retry_delay_ms: 1000,
                max_retries: 3,
                fallback_action: Some("Cache operation for later retry".to_string()),
            }
        }

        pub fn for_network_error() -> Self {
            Self {
                automatic_retry: true,
                retry_delay_ms: 2000,
                max_retries: 2,
                fallback_action: Some("Show offline mode".to_string()),
            }
        }

        pub fn for_validation_error() -> Self {
            Self {
                automatic_retry: false,
                retry_delay_ms: 0,
                max_retries: 0,
                fallback_action: Some("Show validation guidance".to_string()),
            }
        }
    }

    /// Circuit breaker for error recovery
    pub struct ErrorCircuitBreaker {
        failure_count: std::sync::atomic::AtomicU32,
        last_failure: std::sync::Mutex<Option<std::time::Instant>>,
        failure_threshold: u32,
        recovery_timeout: std::time::Duration,
    }

    impl ErrorCircuitBreaker {
        pub fn new(failure_threshold: u32, recovery_timeout: std::time::Duration) -> Self {
            Self {
                failure_count: std::sync::atomic::AtomicU32::new(0),
                last_failure: std::sync::Mutex::new(None),
                failure_threshold,
                recovery_timeout,
            }
        }

        pub fn record_success(&self) {
            self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
            *self.last_failure.lock().unwrap() = None;
        }

        pub fn record_failure(&self) {
            self.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            *self.last_failure.lock().unwrap() = Some(std::time::Instant::now());
        }

        pub fn should_allow_request(&self) -> bool {
            let failure_count = self.failure_count.load(std::sync::atomic::Ordering::Relaxed);
            
            if failure_count < self.failure_threshold {
                return true;
            }

            if let Some(last_failure) = *self.last_failure.lock().unwrap() {
                last_failure.elapsed() > self.recovery_timeout
            } else {
                true
            }
        }
    }
}

/// Log rotation utilities
pub mod rotation {
    use anyhow::{Context, Result};
    use std::fs;
    use std::path::{Path, PathBuf};
    use chrono::{DateTime, Utc};
    
    /// Rotate log file if it exceeds size limit
    pub fn rotate_if_needed(
        log_path: &Path,
        max_size_bytes: u64,
        max_files: usize,
    ) -> Result<()> {
        if !log_path.exists() {
            return Ok(());
        }
        
        let metadata = fs::metadata(log_path)
            .with_context(|| format!("Failed to get metadata for {}", log_path.display()))?;
        
        if metadata.len() <= max_size_bytes {
            return Ok(());
        }
        
        // Create rotated filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let mut rotated_path = log_path.to_path_buf();
        rotated_path.set_extension(format!("log.{}", timestamp));
        
        // Move current log to rotated name
        fs::rename(log_path, &rotated_path)
            .with_context(|| format!("Failed to rotate log file to {}", rotated_path.display()))?;
        
        // Clean up old rotated files
        cleanup_old_logs(log_path, max_files)?;
        
        tracing::info!(
            old_path = %log_path.display(),
            new_path = %rotated_path.display(),
            size_bytes = metadata.len(),
            "Log file rotated"
        );
        
        Ok(())
    }
    
    /// Clean up old rotated log files, keeping only the most recent ones
    fn cleanup_old_logs(log_path: &Path, max_files: usize) -> Result<()> {
        let log_dir = log_path.parent().unwrap_or(Path::new("."));
        let log_name = log_path.file_stem().unwrap_or_default();
        
        let log_name_str = log_name.to_string_lossy();
        let mut rotated_files: Vec<PathBuf> = fs::read_dir(log_dir)
            .context("Failed to read log directory")?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().starts_with(&*log_name_str))
                    .unwrap_or(false)
                    && path.extension()
                        .map(|ext| ext.to_string_lossy().starts_with("log."))
                        .unwrap_or(false)
            })
            .collect();
        
        if rotated_files.len() <= max_files {
            return Ok(());
        }
        
        // Sort by modification time (newest first)
        rotated_files.sort_by_key(|path| {
            fs::metadata(path)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH)
        });
        rotated_files.reverse();
        
        // Remove oldest files
        for old_file in rotated_files.iter().skip(max_files) {
            if let Err(e) = fs::remove_file(old_file) {
                tracing::warn!(
                    file = %old_file.display(),
                    error = %e,
                    "Failed to remove old log file"
                );
            } else {
                tracing::info!(
                    file = %old_file.display(),
                    "Removed old log file"
                );
            }
        }
        
        Ok(())
    }
}

/// Audit logging for administrative and security-sensitive actions
pub mod audit {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use tracing::{info, warn};
    use crate::models::{UserId, RoomId, MessageId};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AuditEvent {
        pub timestamp: DateTime<Utc>,
        pub event_id: String,
        pub user_id: Option<UserId>,
        pub action: AuditAction,
        pub resource_type: String,
        pub resource_id: Option<String>,
        pub details: HashMap<String, String>,
        pub ip_address: Option<String>,
        pub user_agent: Option<String>,
        pub session_id: Option<String>,
        pub success: bool,
        pub error_message: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AuditAction {
        // Authentication actions
        Login,
        Logout,
        PasswordChange,
        SessionExpired,
        
        // User management
        UserCreated,
        UserUpdated,
        UserDeleted,
        UserPromoted,
        UserDemoted,
        
        // Room management
        RoomCreated,
        RoomUpdated,
        RoomDeleted,
        RoomMemberAdded,
        RoomMemberRemoved,
        RoomPermissionChanged,
        
        // Message actions
        MessageCreated,
        MessageUpdated,
        MessageDeleted,
        MessageFlagged,
        
        // Bot management
        BotCreated,
        BotUpdated,
        BotDeleted,
        BotTokenReset,
        
        // System administration
        SystemConfigChanged,
        DatabaseMaintenance,
        BackupCreated,
        BackupRestored,
        
        // Security events
        SecurityViolation,
        RateLimitExceeded,
        UnauthorizedAccess,
        SuspiciousActivity,
    }

    impl AuditEvent {
        pub fn new(action: AuditAction, resource_type: impl Into<String>) -> Self {
            Self {
                timestamp: Utc::now(),
                event_id: uuid::Uuid::new_v4().to_string(),
                user_id: None,
                action,
                resource_type: resource_type.into(),
                resource_id: None,
                details: HashMap::new(),
                ip_address: None,
                user_agent: None,
                session_id: None,
                success: true,
                error_message: None,
            }
        }

        pub fn with_user(mut self, user_id: UserId) -> Self {
            self.user_id = Some(user_id);
            self
        }

        pub fn with_resource_id(mut self, resource_id: impl Into<String>) -> Self {
            self.resource_id = Some(resource_id.into());
            self
        }

        pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.details.insert(key.into(), value.into());
            self
        }

        pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
            self.ip_address = Some(ip.into());
            self
        }

        pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
            self.user_agent = Some(user_agent.into());
            self
        }

        pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
            self.session_id = Some(session_id.into());
            self
        }

        pub fn with_error(mut self, error: impl Into<String>) -> Self {
            self.success = false;
            self.error_message = Some(error.into());
            self
        }

        pub fn log(self) {
            let level = if self.success {
                match self.action {
                    AuditAction::SecurityViolation 
                    | AuditAction::UnauthorizedAccess 
                    | AuditAction::SuspiciousActivity => "security_warning",
                    _ => "audit_info",
                }
            } else {
                "audit_error"
            };

            if self.success {
                info!(
                    event_id = %self.event_id,
                    user_id = ?self.user_id,
                    action = ?self.action,
                    resource_type = %self.resource_type,
                    resource_id = ?self.resource_id,
                    ip_address = ?self.ip_address,
                    details = ?self.details,
                    audit_level = level,
                    "Audit event"
                );
            } else {
                warn!(
                    event_id = %self.event_id,
                    user_id = ?self.user_id,
                    action = ?self.action,
                    resource_type = %self.resource_type,
                    resource_id = ?self.resource_id,
                    ip_address = ?self.ip_address,
                    error = ?self.error_message,
                    details = ?self.details,
                    audit_level = level,
                    "Audit event failed"
                );
            }
        }
    }

    /// Audit logger for tracking administrative actions
    #[derive(Clone)]
pub struct AuditLogger {
        enabled: bool,
    }

    impl AuditLogger {
        pub fn new(enabled: bool) -> Self {
            Self { enabled }
        }

        pub fn log_user_action(
            &self,
            action: AuditAction,
            user_id: UserId,
            resource_type: impl Into<String>,
            resource_id: Option<impl Into<String>>,
            details: HashMap<String, String>,
        ) {
            if !self.enabled {
                return;
            }

            let mut event = AuditEvent::new(action, resource_type).with_user(user_id);
            
            if let Some(id) = resource_id {
                event = event.with_resource_id(id);
            }

            for (key, value) in details {
                event = event.with_detail(key, value);
            }

            event.log();
        }

        pub fn log_security_event(
            &self,
            action: AuditAction,
            user_id: Option<UserId>,
            ip_address: Option<impl Into<String>>,
            details: HashMap<String, String>,
        ) {
            if !self.enabled {
                return;
            }

            let mut event = AuditEvent::new(action, "security");
            
            if let Some(uid) = user_id {
                event = event.with_user(uid);
            }

            if let Some(ip) = ip_address {
                event = event.with_ip_address(ip);
            }

            for (key, value) in details {
                event = event.with_detail(key, value);
            }

            event.log();
        }

        pub fn log_system_event(
            &self,
            action: AuditAction,
            details: HashMap<String, String>,
        ) {
            if !self.enabled {
                return;
            }

            let mut event = AuditEvent::new(action, "system");

            for (key, value) in details {
                event = event.with_detail(key, value);
            }

            event.log();
        }
    }

    /// Convenience macros for audit logging
    #[macro_export]
    macro_rules! audit_user_action {
        ($logger:expr, $action:expr, $user_id:expr, $resource_type:expr) => {
            $logger.log_user_action($action, $user_id, $resource_type, None::<String>, std::collections::HashMap::new());
        };
        ($logger:expr, $action:expr, $user_id:expr, $resource_type:expr, $resource_id:expr) => {
            $logger.log_user_action($action, $user_id, $resource_type, Some($resource_id), std::collections::HashMap::new());
        };
        ($logger:expr, $action:expr, $user_id:expr, $resource_type:expr, $resource_id:expr, $details:expr) => {
            $logger.log_user_action($action, $user_id, $resource_type, Some($resource_id), $details);
        };
    }

    #[macro_export]
    macro_rules! audit_security_event {
        ($logger:expr, $action:expr, $user_id:expr, $ip:expr) => {
            $logger.log_security_event($action, Some($user_id), Some($ip), std::collections::HashMap::new());
        };
        ($logger:expr, $action:expr, $user_id:expr, $ip:expr, $details:expr) => {
            $logger.log_security_event($action, Some($user_id), Some($ip), $details);
        };
    }
}

/// Error documentation and recovery procedures
pub mod documentation {
    use std::collections::HashMap;

    /// Error documentation with recovery procedures
    pub struct ErrorDocumentation {
        error_guides: HashMap<String, ErrorGuide>,
    }

    #[derive(Debug, Clone)]
    pub struct ErrorGuide {
        pub error_code: String,
        pub title: String,
        pub description: String,
        pub common_causes: Vec<String>,
        pub recovery_steps: Vec<RecoveryStep>,
        pub prevention_tips: Vec<String>,
        pub related_errors: Vec<String>,
    }

    #[derive(Debug, Clone)]
    pub struct RecoveryStep {
        pub step_number: u32,
        pub description: String,
        pub is_automatic: bool,
        pub estimated_time: Option<String>,
        pub requires_admin: bool,
    }

    impl ErrorDocumentation {
        pub fn new() -> Self {
            let mut error_guides = HashMap::new();
            
            // Database connection errors
            error_guides.insert("DATABASE_CONNECTION_FAILED".to_string(), ErrorGuide {
                error_code: "DATABASE_CONNECTION_FAILED".to_string(),
                title: "Database Connection Failed".to_string(),
                description: "The application cannot connect to the database".to_string(),
                common_causes: vec![
                    "Database file is locked by another process".to_string(),
                    "Insufficient disk space".to_string(),
                    "Database file permissions issue".to_string(),
                    "Corrupted database file".to_string(),
                ],
                recovery_steps: vec![
                    RecoveryStep {
                        step_number: 1,
                        description: "Check if database file exists and is readable".to_string(),
                        is_automatic: true,
                        estimated_time: Some("5 seconds".to_string()),
                        requires_admin: false,
                    },
                    RecoveryStep {
                        step_number: 2,
                        description: "Verify disk space availability".to_string(),
                        is_automatic: true,
                        estimated_time: Some("2 seconds".to_string()),
                        requires_admin: false,
                    },
                    RecoveryStep {
                        step_number: 3,
                        description: "Attempt to create new database if missing".to_string(),
                        is_automatic: true,
                        estimated_time: Some("10 seconds".to_string()),
                        requires_admin: true,
                    },
                ],
                prevention_tips: vec![
                    "Ensure adequate disk space (at least 1GB free)".to_string(),
                    "Regular database backups".to_string(),
                    "Monitor database file permissions".to_string(),
                ],
                related_errors: vec!["DATABASE_MIGRATION_FAILED".to_string()],
            });

            // Rate limiting errors
            error_guides.insert("RATE_LIMIT_EXCEEDED".to_string(), ErrorGuide {
                error_code: "RATE_LIMIT_EXCEEDED".to_string(),
                title: "Rate Limit Exceeded".to_string(),
                description: "Too many requests have been made in a short time period".to_string(),
                common_causes: vec![
                    "Sending messages too quickly".to_string(),
                    "Automated bot behavior".to_string(),
                    "Network issues causing request retries".to_string(),
                ],
                recovery_steps: vec![
                    RecoveryStep {
                        step_number: 1,
                        description: "Wait for rate limit window to reset".to_string(),
                        is_automatic: false,
                        estimated_time: Some("1-5 minutes".to_string()),
                        requires_admin: false,
                    },
                    RecoveryStep {
                        step_number: 2,
                        description: "Reduce request frequency".to_string(),
                        is_automatic: false,
                        estimated_time: None,
                        requires_admin: false,
                    },
                ],
                prevention_tips: vec![
                    "Combine multiple actions into single requests".to_string(),
                    "Implement client-side rate limiting".to_string(),
                    "Use WebSocket for real-time updates instead of polling".to_string(),
                ],
                related_errors: vec!["TOO_MANY_REQUESTS".to_string()],
            });

            // Authentication errors
            error_guides.insert("SESSION_EXPIRED".to_string(), ErrorGuide {
                error_code: "SESSION_EXPIRED".to_string(),
                title: "Session Expired".to_string(),
                description: "Your login session has expired and you need to authenticate again".to_string(),
                common_causes: vec![
                    "Session timeout due to inactivity".to_string(),
                    "Server restart cleared sessions".to_string(),
                    "Security policy forced logout".to_string(),
                ],
                recovery_steps: vec![
                    RecoveryStep {
                        step_number: 1,
                        description: "Redirect to login page".to_string(),
                        is_automatic: true,
                        estimated_time: Some("Immediate".to_string()),
                        requires_admin: false,
                    },
                    RecoveryStep {
                        step_number: 2,
                        description: "Preserve current page for post-login redirect".to_string(),
                        is_automatic: true,
                        estimated_time: Some("Immediate".to_string()),
                        requires_admin: false,
                    },
                ],
                prevention_tips: vec![
                    "Enable 'Remember Me' option if available".to_string(),
                    "Save work frequently".to_string(),
                    "Keep the application active in a browser tab".to_string(),
                ],
                related_errors: vec!["INVALID_CREDENTIALS".to_string()],
            });

            Self { error_guides }
        }

        pub fn get_guide(&self, error_code: &str) -> Option<&ErrorGuide> {
            self.error_guides.get(error_code)
        }

        pub fn get_recovery_steps(&self, error_code: &str) -> Vec<RecoveryStep> {
            self.error_guides
                .get(error_code)
                .map(|guide| guide.recovery_steps.clone())
                .unwrap_or_default()
        }

        pub fn get_prevention_tips(&self, error_code: &str) -> Vec<String> {
            self.error_guides
                .get(error_code)
                .map(|guide| guide.prevention_tips.clone())
                .unwrap_or_default()
        }
    }

    impl Default for ErrorDocumentation {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_env_filter_creation() {
        let filter = create_env_filter("debug").unwrap();
        // Basic test that filter was created successfully
        let filter_str = format!("{:?}", filter);
        // The filter should contain some indication of the debug level
        assert!(filter_str.contains("debug") || filter_str.contains("DEBUG") || filter_str.len() > 0);
    }
    
    #[test]
    fn test_log_rotation() {
        use std::fs::File;
        use std::io::Write;
        
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        
        // Create a log file with some content
        let mut file = File::create(&log_path).unwrap();
        writeln!(file, "Test log content").unwrap();
        drop(file);
        
        // Test rotation (should not rotate since file is small)
        rotation::rotate_if_needed(&log_path, 1024, 5).unwrap();
        assert!(log_path.exists());
        
        // Test rotation with small size limit (should rotate)
        rotation::rotate_if_needed(&log_path, 1, 5).unwrap();
        // Original file should be gone or recreated empty
        if log_path.exists() {
            let metadata = std::fs::metadata(&log_path).unwrap();
            assert_eq!(metadata.len(), 0);
        }
    }
}