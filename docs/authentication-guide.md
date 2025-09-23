# Authentication System Guide

## Overview

Campfire implements a Rails-inspired session-based authentication system using secure tokens and bcrypt password hashing. The system provides both user authentication for web clients and bot authentication for API integrations.

**Key Features:**
- Session-based authentication with secure token generation
- bcrypt password hashing with configurable cost
- Bot API authentication with unique tokens
- Flexible session extraction from headers or cookies
- Comprehensive security validation and error handling

## Architecture

```mermaid
graph TD
    subgraph "Authentication System"
        direction TB
        LOGIN[User Login<br/>Email + Password]
        SESSION[Session Management<br/>Secure Tokens]
        VALIDATION[Token Validation<br/>Request Authentication]
        BOTS[Bot Authentication<br/>API Keys]
    end
    
    LOGIN --> SESSION
    SESSION --> VALIDATION
    VALIDATION --> BOTS
    
    classDef auth fill:#e3f2fd,stroke:#1976d2,stroke-width:3px
    class LOGIN,SESSION,VALIDATION,BOTS auth
```

### Detailed Authentication Flow

```mermaid
graph TD
    subgraph "Authentication Flow"
        A[Client Request] --> B{Has Auth Token?}
        B -->|No| C[Return 401 Unauthorized]
        B -->|Yes| D[Extract Token]
        D --> E[Validate Session]
        E -->|Invalid| F[Return 401 Unauthorized]
        E -->|Valid| G[Load User]
        G --> H[Process Request]
    end
    
    subgraph "Session Management"
        I[Login Request] --> J[Validate Credentials]
        J -->|Valid| K[Generate Secure Token]
        K --> L[Store Session in DB]
        L --> M[Return Token to Client]
        J -->|Invalid| N[Return 401 Error]
    end
    
    subgraph "Bot Authentication"
        O[Bot API Request] --> P[Extract Bot Key]
        P --> Q[Parse user_id-bot_token]
        Q --> R[Validate Bot Token]
        R -->|Valid| S[Load Bot User]
        R -->|Invalid| T[Return 401 Error]
    end
    
    classDef flow fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef session fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef bot fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class A,B,C,D,E,F,G,H flow
    class I,J,K,L,M,N session
    class O,P,Q,R,S,T bot
```

## Session Management

### Session Token Generation

Campfire uses cryptographically secure session tokens following Rails security patterns:

```rust
/// Generates cryptographically secure session token (Critical Gap #4)
/// 
/// Uses Rails-equivalent secure token generation with:
/// - 32 bytes of cryptographically secure random data
/// - Base64 URL-safe encoding
/// - No predictable patterns
fn generate_secure_token() -> Result<String, AuthError> {
    let mut token_bytes = [0u8; 32];
    thread_rng().fill(&mut token_bytes);
    
    // Use base64 URL-safe encoding (no padding)
    let token = base64::encode_config(&token_bytes, base64::URL_SAFE_NO_PAD);
    
    if token.len() < 32 {
        return Err(AuthError::TokenGeneration);
    }
    
    Ok(token)
}
```

**Security Properties:**
- **Entropy**: 256 bits of cryptographically secure randomness
- **Encoding**: Base64 URL-safe (no `+`, `/`, or `=` characters)
- **Length**: Minimum 32 characters after encoding
- **Uniqueness**: Collision probability is negligible (2^-256)

### Session Storage

Sessions are stored in SQLite with automatic expiration:

```sql
CREATE TABLE sessions (
    token TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL
);

-- Index for efficient cleanup of expired sessions
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
```

**Session Properties:**
- **Default Expiry**: 30 days (configurable via `CAMPFIRE_SESSION_EXPIRY_HOURS`)
- **Automatic Cleanup**: Expired sessions are removed during validation
- **Single Sign-Out**: Revoking a session immediately invalidates it

### Session Extraction

The authentication middleware extracts session tokens from multiple sources with priority order:

```rust
/// Extracts session token from Authorization header or cookies
/// 
/// Priority order:
/// 1. Authorization header: "Bearer <token>"
/// 2. Cookie: "session_token=<token>"
pub fn extract_session_token(parts: &Parts) -> Result<String, SessionExtractionError> {
    // Try Authorization header first
    if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
        let auth_str = auth_header
            .to_str()
            .map_err(|_| SessionExtractionError::InvalidToken)?;
        
        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            return Ok(token.to_string());
        }
    }
    
    // Try session cookie
    if let Some(cookie_header) = parts.headers.get(COOKIE) {
        let cookie_str = cookie_header
            .to_str()
            .map_err(|_| SessionExtractionError::InvalidToken)?;
        
        // Parse cookies and look for session_token
        for cookie in cookie_str.split(';') {
            let cookie = cookie.trim();
            if let Some(value) = cookie.strip_prefix("session_token=") {
                if !value.is_empty() {
                    return Ok(value.to_string());
                }
            }
        }
    }
    
    Err(SessionExtractionError::MissingToken)
}
```

**Supported Authentication Methods:**
1. **Authorization Header**: `Authorization: Bearer <session_token>`
2. **Session Cookie**: `Cookie: session_token=<session_token>`

## Password Security

### Password Requirements

Campfire enforces minimum password security standards:

```rust
/// Validates password strength
fn validate_password(password: &str) -> Result<(), AuthError> {
    if password.len() < 8 {
        return Err(AuthError::WeakPassword);
    }
    Ok(())
}
```

**Current Requirements:**
- **Minimum Length**: 8 characters
- **No Maximum Length**: Passwords are hashed, so length doesn't affect storage
- **Character Requirements**: None (allows flexibility for users)

**Recommended Practices:**
- Use a mix of uppercase, lowercase, numbers, and symbols
- Avoid common passwords and dictionary words
- Consider using a password manager
- Enable two-factor authentication when available (future feature)

### Password Hashing

Passwords are hashed using bcrypt with Rails-compatible settings:

```rust
use bcrypt::{hash, verify, DEFAULT_COST};

// Hash password during user creation
let password_hash = hash(&password, DEFAULT_COST)?;

// Verify password during authentication
if !verify(&password, &user.password_hash)? {
    return Err(AuthError::InvalidCredentials);
}
```

**bcrypt Configuration:**
- **Cost Factor**: 12 (DEFAULT_COST) - provides strong security with reasonable performance
- **Salt**: Automatically generated per password (built into bcrypt)
- **Algorithm**: bcrypt 2b variant (most secure)

**Security Benefits:**
- **Adaptive**: Cost can be increased as hardware improves
- **Salt Protection**: Each password has a unique salt
- **Time Constant**: Verification time is consistent regardless of password

## Authentication API Endpoints

### POST /api/auth/login

Authenticates a user with email and password, returning a session token.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securepassword123"
}
```

**Response (200 OK):**
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "John Doe",
    "email": "user@example.com",
    "bio": "Software Developer",
    "admin": false,
    "created_at": "2024-01-15T10:30:00Z"
  },
  "session_token": "AbCdEf123456789..."
}
```

**Error Responses:**
- `400 Bad Request`: Invalid request format or email format
- `401 Unauthorized`: Invalid credentials
- `500 Internal Server Error`: Server error

### POST /api/auth/logout

Revokes the current session token.

**Authentication Required:** Yes (session token)

**Response (200 OK):**
```json
{
  "message": "Logged out successfully",
  "success": true
}
```

**Error Responses:**
- `401 Unauthorized`: Invalid or missing session token

### GET /api/users/me

Returns information about the currently authenticated user.

**Authentication Required:** Yes (session token)

**Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "John Doe",
  "email": "user@example.com",
  "bio": "Software Developer",
  "admin": false,
  "created_at": "2024-01-15T10:30:00Z"
}
```

**Error Responses:**
- `401 Unauthorized`: Invalid or missing session token

## Bot Authentication

### Bot Token System

Bots use a different authentication mechanism based on API keys:

```rust
/// Generate a secure bot token (12 alphanumeric characters like Rails)
fn generate_bot_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    (0..12)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
```

**Bot Key Format:**
- **Structure**: `{user_id}-{bot_token}`
- **Example**: `550e8400-e29b-41d4-a716-446655440000-AbC123XyZ789`
- **Token Length**: 12 alphanumeric characters
- **Character Set**: A-Z, a-z, 0-9 (62 possible characters)

### Bot Authentication Flow

```mermaid
graph TD
    A[Bot API Request] --> B[Extract Bot Key from Header]
    B --> C{Valid Format?}
    C -->|No| D[Return 401 Invalid Token]
    C -->|Yes| E[Parse user_id and bot_token]
    E --> F[Query Database for User]
    F --> G{User Exists?}
    G -->|No| H[Return 401 User Not Found]
    G -->|Yes| I{Is Bot User?}
    I -->|No| J[Return 403 Not a Bot]
    I -->|Yes| K{Token Matches?}
    K -->|No| L[Return 401 Invalid Token]
    K -->|Yes| M[Allow Request]
```

### Bot API Usage

**Authentication Header:**
```
Authorization: Bearer {user_id}-{bot_token}
```

**Example Request:**
```bash
curl -X POST https://campfire.example.com/api/rooms/123/messages \
  -H "Authorization: Bearer 550e8400-e29b-41d4-a716-446655440000-AbC123XyZ789" \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello from bot!"}'
```

## Security Best Practices

### Session Security

1. **Token Storage**
   - Store session tokens securely (httpOnly cookies recommended for web)
   - Never log session tokens
   - Use HTTPS in production to protect tokens in transit

2. **Session Management**
   - Implement session timeout (default: 30 days)
   - Provide logout functionality to revoke sessions
   - Consider implementing "remember me" with longer expiry

3. **Token Rotation**
   - Consider rotating session tokens periodically
   - Invalidate sessions on password change
   - Implement concurrent session limits if needed

### Password Security

1. **Password Policies**
   - Enforce minimum 8-character length
   - Consider implementing complexity requirements
   - Prevent common passwords (future enhancement)

2. **Account Protection**
   - Implement rate limiting on login attempts
   - Consider account lockout after failed attempts
   - Log authentication events for monitoring

3. **Password Management**
   - Encourage strong, unique passwords
   - Support password managers
   - Provide secure password reset functionality

### Bot Security

1. **Token Management**
   - Treat bot tokens like passwords
   - Rotate bot tokens regularly
   - Revoke compromised tokens immediately

2. **Access Control**
   - Limit bot permissions to necessary rooms
   - Monitor bot activity for unusual patterns
   - Implement rate limiting for bot requests

3. **Webhook Security**
   - Validate webhook URLs
   - Use HTTPS for webhook endpoints
   - Implement webhook signature verification (future enhancement)

## Configuration

### Environment Variables

Authentication behavior can be configured via environment variables:

```bash
# Session Configuration
CAMPFIRE_SESSION_TOKEN_LENGTH=32          # Token length in bytes (min: 16)
CAMPFIRE_SESSION_EXPIRY_HOURS=720         # Session expiry (default: 30 days)

# Security Configuration
CAMPFIRE_RATE_LIMIT_RPM=60               # Rate limit: requests per minute
CAMPFIRE_FORCE_HTTPS=true                # Redirect HTTP to HTTPS
CAMPFIRE_TRUST_PROXY=false               # Trust proxy headers

# CORS Configuration
CAMPFIRE_CORS_ORIGINS=https://app.example.com,https://admin.example.com
```

### Security Headers

Campfire automatically sets security headers:

```rust
// Security headers set by middleware
"X-Content-Type-Options": "nosniff"
"X-Frame-Options": "DENY"
"X-XSS-Protection": "1; mode=block"
"Referrer-Policy": "strict-origin-when-cross-origin"
"Content-Security-Policy": "default-src 'self'"
```

## Error Handling

### Authentication Errors

The system provides comprehensive error handling with structured error types:

```rust
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Session not found or expired")]
    SessionExpired,
    
    #[error("User not found: {email}")]
    UserNotFound { email: String },
    
    #[error("Email already exists: {email}")]
    EmailExists { email: String },
    
    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },
    
    #[error("Password too weak")]
    WeakPassword,
    
    #[error("Token generation failed")]
    TokenGeneration,
}
```

### HTTP Status Codes

Authentication errors map to appropriate HTTP status codes:

- **400 Bad Request**: Invalid email format, weak password
- **401 Unauthorized**: Invalid credentials, expired session
- **403 Forbidden**: Insufficient permissions
- **404 Not Found**: User not found
- **409 Conflict**: Email already exists
- **500 Internal Server Error**: Server errors, token generation failure

### Error Response Format

All authentication errors return a consistent JSON format:

```json
{
  "error": {
    "message": "Invalid email or password",
    "code": "INVALID_CREDENTIALS",
    "status": 401
  },
  "success": false
}
```

## Session Cleanup

### Automatic Cleanup

Expired sessions are automatically cleaned up during validation:

```rust
async fn validate_session(&self, token: String) -> Result<User, AuthError> {
    // Get session from database
    let session = self.db.get_session(&token)
        .await?
        .ok_or(AuthError::SessionExpired)?;
    
    // Check if session is expired
    if session.expires_at < Utc::now() {
        // Clean up expired session
        let _ = self.db.delete_session(token.clone()).await;
        return Err(AuthError::SessionExpired);
    }
    
    // Session is valid, return user
    // ...
}
```

### Batch Cleanup

For maintenance, implement a background task to clean up expired sessions:

```rust
// Example background cleanup task
async fn cleanup_expired_sessions(db: &Database) -> Result<usize, DatabaseError> {
    let deleted_count = sqlx::query!(
        "DELETE FROM sessions WHERE expires_at < ?",
        Utc::now()
    )
    .execute(&db.pool)
    .await?
    .rows_affected();
    
    Ok(deleted_count as usize)
}
```

## Testing

### Unit Tests

The authentication system includes comprehensive unit tests:

```rust
#[tokio::test]
async fn test_secure_token_generation() {
    // Test Critical Gap #4: Session Token Security
    let token1 = AuthService::generate_secure_token().unwrap();
    let token2 = AuthService::generate_secure_token().unwrap();
    
    // Tokens should be unique
    assert_ne!(token1, token2);
    
    // Tokens should have sufficient entropy (at least 32 chars)
    assert!(token1.len() >= 32);
    assert!(token2.len() >= 32);
    
    // Tokens should be URL-safe (no special characters that need encoding)
    assert!(!token1.contains('+'));
    assert!(!token1.contains('/'));
    assert!(!token1.contains('='));
}

#[tokio::test]
async fn test_user_creation_and_authentication() {
    let auth_service = create_test_auth_service().await;
    
    // Create user
    let user = auth_service.create_user(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    // Authenticate user
    let session = auth_service.authenticate(
        "test@example.com".to_string(),
        "password123".to_string(),
    ).await.unwrap();
    
    // Validate session
    let validated_user = auth_service.validate_session(session.token.clone()).await.unwrap();
    assert_eq!(validated_user.id, user.id);
    
    // Revoke session
    auth_service.revoke_session(session.token.clone()).await.unwrap();
    
    // Session should no longer be valid
    assert!(auth_service.validate_session(session.token).await.is_err());
}
```

### Integration Tests

Test the complete authentication flow with real HTTP requests:

```rust
#[tokio::test]
async fn test_login_logout_flow() {
    let app = create_test_app().await;
    
    // Test login
    let login_response = app
        .post("/api/auth/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await;
    
    assert_eq!(login_response.status(), 200);
    let login_data: LoginResponse = login_response.json().await;
    
    // Test authenticated request
    let user_response = app
        .get("/api/users/me")
        .header("Authorization", format!("Bearer {}", login_data.session_token))
        .send()
        .await;
    
    assert_eq!(user_response.status(), 200);
    
    // Test logout
    let logout_response = app
        .post("/api/auth/logout")
        .header("Authorization", format!("Bearer {}", login_data.session_token))
        .send()
        .await;
    
    assert_eq!(logout_response.status(), 200);
    
    // Session should be invalid after logout
    let invalid_response = app
        .get("/api/users/me")
        .header("Authorization", format!("Bearer {}", login_data.session_token))
        .send()
        .await;
    
    assert_eq!(invalid_response.status(), 401);
}
```

## Troubleshooting

### Common Issues

1. **"Session expired" errors**
   - Check system clock synchronization
   - Verify session expiry configuration
   - Ensure database timezone is UTC

2. **"Invalid credentials" for correct password**
   - Verify bcrypt cost factor compatibility
   - Check password encoding (UTF-8)
   - Ensure password wasn't truncated during storage

3. **Bot authentication failures**
   - Verify bot key format: `{user_id}-{bot_token}`
   - Check that user exists and is marked as bot
   - Ensure bot_token matches database value

4. **CORS errors in browser**
   - Configure `CAMPFIRE_CORS_ORIGINS` environment variable
   - Ensure frontend origin is included in allowed origins
   - Check that preflight requests are handled correctly

### Debug Logging

Enable debug logging to troubleshoot authentication issues:

```bash
CAMPFIRE_LOG_LEVEL=debug ./campfire
```

Look for log entries like:
```
DEBUG campfire::services::auth: Validating session token for user
DEBUG campfire::middleware::session: Extracting session token from Authorization header
WARN campfire::handlers::auth: Authentication failed for user@example.com: Invalid credentials
```

### Database Queries

Check session and user data directly:

```sql
-- Check active sessions
SELECT token, user_id, created_at, expires_at 
FROM sessions 
WHERE expires_at > datetime('now');

-- Check user authentication data
SELECT id, name, email, admin, bot_token IS NOT NULL as is_bot, created_at 
FROM users 
WHERE email = 'user@example.com';

-- Clean up expired sessions manually
DELETE FROM sessions WHERE expires_at < datetime('now');
```

## Migration from Other Systems

### From Rails Campfire

If migrating from the original Rails Campfire:

1. **Password Hashes**: bcrypt hashes are compatible between Rails and Rust
2. **Session Tokens**: Generate new tokens (Rails tokens won't work)
3. **User Data**: Export users table and import with ID mapping
4. **Bot Tokens**: Regenerate bot tokens using new format

### From Other Chat Systems

1. **User Import**: Create users with temporary passwords, require password reset
2. **Authentication**: Implement SSO integration if needed
3. **Bot Migration**: Recreate bots with new API keys
4. **Session Management**: All users will need to log in again

## Future Enhancements

### Planned Features

1. **Two-Factor Authentication (2FA)**
   - TOTP support with QR codes
   - Backup codes for recovery
   - SMS/email fallback options

2. **OAuth Integration**
   - Google, GitHub, Microsoft providers
   - SAML support for enterprise
   - Custom OAuth providers

3. **Advanced Security**
   - Device fingerprinting
   - Suspicious activity detection
   - Geographic login restrictions

4. **Session Management**
   - Multiple device sessions
   - Session activity monitoring
   - Remote session termination

### API Versioning

Future authentication API changes will be versioned:

```
/api/v1/auth/login  (current)
/api/v2/auth/login  (future with 2FA)
```

Backward compatibility will be maintained for at least one major version.

---

**Security Note**: This authentication system is designed for typical team chat usage. For high-security environments, consider additional measures like hardware security keys, network restrictions, and audit logging.