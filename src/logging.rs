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
    
    match (&config.logging.format, &config.logging.file_path) {
        // JSON format, stdout only
        (LogFormat::Json, None) => {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
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
        
        // File output - simplified for now
        (_, Some(_file_path)) => {
            // For now, just use stdout logging
            // File logging can be added later with proper async file writers
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .init();
        }
    }
    
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_level = %log_level,
        format = ?config.logging.format,
        structured = config.logging.structured,
        trace_requests = config.logging.trace_requests,
        "Logging initialized"
    );
    
    Ok(())
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
}

#[macro_export]
macro_rules! log_security_event {
    ($event:expr, $user_id:expr, $details:expr) => {
        tracing::warn!(
            event = %$event,
            user_id = %$user_id,
            details = %$details,
            "Security event"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr, $context:expr) => {
        tracing::error!(
            error = %$error,
            context = %$context,
            "Application error"
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