use axum::{
    extract::Path,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

/// Embedded static assets using rust-embed
#[derive(RustEmbed)]
#[folder = "assets/static/"]
pub struct Assets;

/// Serve static assets with proper MIME types and caching headers
pub async fn serve_static_asset(Path(path): Path<String>) -> Response {
    let path = path.trim_start_matches('/');
    
    match Assets::get(path) {
        Some(content) => {
            let mime_type = get_mime_type(path);
            let mut headers = HeaderMap::new();
            
            // Set content type
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type).unwrap(),
            );
            
            // Set caching headers
            set_cache_headers(&mut headers, path);
            
            // Set security headers for certain file types
            set_security_headers(&mut headers, path);
            
            (headers, content.data).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Asset not found").into_response(),
    }
}

/// Get MIME type based on file extension
fn get_mime_type(path: &str) -> &'static str {
    let extension = path.split('.').last().unwrap_or("");
    
    match extension.to_lowercase().as_str() {
        // Text files
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        
        // Images
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        
        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        
        // Video
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogv" => "video/ogg",
        
        // Fonts
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "eot" => "application/vnd.ms-fontobject",
        
        // Archives
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        
        // Documents
        "pdf" => "application/pdf",
        
        // Default
        _ => "application/octet-stream",
    }
}

/// Set appropriate cache headers based on file type
fn set_cache_headers(headers: &mut HeaderMap, path: &str) {
    let extension = path.split('.').last().unwrap_or("").to_lowercase();
    
    match extension.as_str() {
        // Long cache for static assets with hashes
        "css" | "js" | "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" |
        "woff" | "woff2" | "ttf" | "otf" | "eot" | "mp3" | "wav" | "ogg" => {
            // Cache for 1 year for static assets
            headers.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, immutable"),
            );
        }
        
        // Shorter cache for HTML files
        "html" | "htm" => {
            headers.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=3600"),
            );
        }
        
        // No cache for service worker and manifest
        _ if path.ends_with("sw.js") || path.ends_with("manifest.json") => {
            headers.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-cache, no-store, must-revalidate"),
            );
        }
        
        // Default cache
        _ => {
            headers.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=86400"),
            );
        }
    }
    
    // Add ETag for better caching
    let etag = format!("\"{}\"", calculate_etag(path));
    headers.insert(
        header::ETAG,
        HeaderValue::from_str(&etag).unwrap(),
    );
}

/// Set security headers for certain file types
fn set_security_headers(headers: &mut HeaderMap, path: &str) {
    let extension = path.split('.').last().unwrap_or("").to_lowercase();
    
    match extension.as_str() {
        "html" | "htm" => {
            // Content Security Policy for HTML files
            headers.insert(
                header::HeaderName::from_static("content-security-policy"),
                HeaderValue::from_static(
                    "default-src 'self'; \
                     script-src 'self' 'unsafe-inline'; \
                     style-src 'self' 'unsafe-inline'; \
                     img-src 'self' data: https:; \
                     connect-src 'self' ws: wss:; \
                     font-src 'self'; \
                     media-src 'self';"
                ),
            );
            
            // X-Frame-Options
            headers.insert(
                header::HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            );
            
            // X-Content-Type-Options
            headers.insert(
                header::HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );
        }
        
        "js" | "mjs" => {
            // X-Content-Type-Options for JavaScript
            headers.insert(
                header::HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );
        }
        
        _ => {}
    }
}

/// Calculate a simple ETag based on file path (in production, use file hash)
fn calculate_etag(path: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Serve the main chat interface HTML
pub async fn serve_chat_interface() -> impl IntoResponse {
    let html = include_str!("../templates/chat.html");
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    
    // Set cache headers for HTML
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    
    // Security headers
    headers.insert(
        header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             connect-src 'self' ws: wss:; \
             font-src 'self'; \
             media-src 'self';"
        ),
    );
    
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    
    (headers, html)
}

/// Serve login page
pub async fn serve_login_page() -> impl IntoResponse {
    let html = include_str!("../templates/login.html");
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    
    (headers, html)
}

/// Serve demo setup page
pub async fn serve_demo_page() -> impl IntoResponse {
    let html = include_str!("../templates/demo.html");
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    
    // Set cache headers for HTML
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    
    // Security headers
    headers.insert(
        header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             connect-src 'self' ws: wss:; \
             font-src 'self'; \
             media-src 'self';"
        ),
    );
    
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    
    (headers, html)
}

/// Serve PWA manifest
pub async fn serve_manifest() -> Response {
    serve_static_asset(Path("manifest.json".to_string())).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mime_type_detection() {
        assert_eq!(get_mime_type("style.css"), "text/css; charset=utf-8");
        assert_eq!(get_mime_type("script.js"), "application/javascript; charset=utf-8");
        assert_eq!(get_mime_type("image.png"), "image/png");
        assert_eq!(get_mime_type("sound.mp3"), "audio/mpeg");
        assert_eq!(get_mime_type("unknown.xyz"), "application/octet-stream");
    }
    
    #[test]
    fn test_etag_calculation() {
        let etag1 = calculate_etag("test.css");
        let etag2 = calculate_etag("test.css");
        let etag3 = calculate_etag("other.css");
        
        assert_eq!(etag1, etag2);
        assert_ne!(etag1, etag3);
    }
}