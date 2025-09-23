# Campfire Rust API Reference

## Overview

The Campfire Rust API provides comprehensive REST endpoints and WebSocket connections for real-time chat functionality. This documentation covers all available endpoints, request/response formats, authentication requirements, and usage examples.

## Base Information

- **Base URL**: `http://localhost:3000/api`
- **WebSocket URL**: `ws://localhost:3000/ws`
- **Content Type**: `application/json`
- **API Version**: v1 (implicit)

## Authentication

### Authentication Methods

The API supports multiple authentication methods:

1. **Session Cookies** (recommended for web browsers)
2. **Bearer Token** in Authorization header
3. **Bot API Keys** for bot integrations

### Session Authentication

```bash
# Login to get session token
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password123"
  }'
```

**Response:**
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "John Doe",
    "email": "user@example.com",
    "bio": "Software developer",
    "admin": false,
    "created_at": "2023-12-07T10:30:00Z"
  },
  "session_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

### Using Session Token

```bash
# Use session token in subsequent requests
curl -H "Authorization: Bearer <session_token>" \
  http://localhost:3000/api/users/me
```

## Rate Limiting

All API endpoints are subject to rate limiting:

- **Global Rate Limit**: 1000 requests per hour per IP
- **User Rate Limit**: 500 requests per hour per authenticated user
- **Message Creation**: 60 messages per minute per user
- **WebSocket Messages**: 50 messages per minute per connection

### Rate Limit Headers

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1701943800
Retry-After: 3600
```

## Error Handling

### Standard Error Response Format

```json
{
  "error": {
    "message": "Human-readable error message",
    "code": "MACHINE_READABLE_CODE",
    "status": 400
  },
  "success": false
}
```

### HTTP Status Codes

- **200 OK**: Request successful
- **201 Created**: Resource created successfully
- **400 Bad Request**: Invalid request format or parameters
- **401 Unauthorized**: Authentication required or invalid
- **403 Forbidden**: Insufficient permissions
- **404 Not Found**: Resource not found
- **409 Conflict**: Resource already exists or conflict
- **422 Unprocessable Entity**: Validation errors
- **429 Too Many Requests**: Rate limit exceeded
- **500 Internal Server Error**: Server error

## Authentication Endpoints

### POST /api/auth/login

Authenticate user with email and password.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**Validation Rules:**
- `email`: Valid email format, required
- `password`: Non-empty string, required

**Response (200 OK):**
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "John Doe",
    "email": "user@example.com",
    "bio": "Software developer",
    "admin": false,
    "created_at": "2023-12-07T10:30:00Z"
  },
  "session_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

**Error Responses:**
- **400 Bad Request**: Invalid email format or missing password
- **401 Unauthorized**: Invalid credentials

**cURL Example:**
```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password123"
  }'
```

### POST /api/auth/logout

Revoke the current session token.

**Authentication**: Required (session token)

**Request Body**: None

**Response (200 OK):**
```json
{
  "message": "Logged out successfully",
  "success": true
}
```

**cURL Example:**
```bash
curl -X POST http://localhost:3000/api/auth/logout \
  -H "Authorization: Bearer <session_token>"
```

## User Endpoints

### GET /api/users/me

Get current authenticated user information.

**Authentication**: Required (session token)

**Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "John Doe",
  "email": "user@example.com",
  "bio": "Software developer",
  "admin": false,
  "created_at": "2023-12-07T10:30:00Z"
}
```

**cURL Example:**
```bash
curl -H "Authorization: Bearer <session_token>" \
  http://localhost:3000/api/users/me
```

## Room Management Endpoints

### GET /api/rooms

Get list of rooms the current user has access to.

**Authentication**: Required (session token)

**Response (200 OK):**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "General Discussion",
    "topic": "General chat for everyone",
    "room_type": "Open",
    "created_at": "2023-12-07T10:00:00Z",
    "last_message_at": "2023-12-07T10:30:00Z"
  },
  {
    "id": "550e8400-e29b-41d4-a716-446655440002",
    "name": "Development Team",
    "topic": "Private room for developers",
    "room_type": "Closed",
    "created_at": "2023-12-07T09:00:00Z",
    "last_message_at": "2023-12-07T10:25:00Z"
  }
]
```

**cURL Example:**
```bash
curl -H "Authorization: Bearer <session_token>" \
  http://localhost:3000/api/rooms
```

### POST /api/rooms

Create a new room with the authenticated user as admin.

**Authentication**: Required (session token)

**Request Body:**
```json
{
  "name": "New Room",
  "topic": "Optional room description",
  "room_type": "Open"
}
```

**Validation Rules:**
- `name`: 1-100 characters, required
- `topic`: Max 500 characters, optional
- `room_type`: Must be "Open", "Closed", or "Direct"

**Response (201 Created):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440003",
  "name": "New Room",
  "topic": "Optional room description",
  "room_type": "Open",
  "created_at": "2023-12-07T10:35:00Z",
  "last_message_at": null
}
```

**cURL Example:**
```bash
curl -X POST http://localhost:3000/api/rooms \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New Room",
    "topic": "Optional room description",
    "room_type": "Open"
  }'
```

### GET /api/rooms/:id

Get details for a specific room.

**Authentication**: Required (session token)
**Authorization**: User must have access to the room

**Path Parameters:**
- `id`: UUID of the room

**Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "name": "General Discussion",
  "topic": "General chat for everyone",
  "room_type": "Open",
  "created_at": "2023-12-07T10:00:00Z",
  "last_message_at": "2023-12-07T10:30:00Z"
}
```

**Error Responses:**
- **400 Bad Request**: Invalid room ID format
- **403 Forbidden**: User does not have access to room
- **404 Not Found**: Room not found

**cURL Example:**
```bash
curl -H "Authorization: Bearer <session_token>" \
  http://localhost:3000/api/rooms/550e8400-e29b-41d4-a716-446655440001
```

### POST /api/rooms/:id/members

Add a member to a room.

**Authentication**: Required (session token)
**Authorization**: User must be an admin of the room

**Path Parameters:**
- `id`: UUID of the room

**Request Body:**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440004",
  "involvement_level": "Member"
}
```

**Validation Rules:**
- `user_id`: Valid UUID, required
- `involvement_level`: Must be "Member" or "Admin"

**Response (201 Created):**
```json
{
  "message": "Member added successfully",
  "success": true
}
```

**Error Responses:**
- **403 Forbidden**: User not authorized to add members
- **404 Not Found**: Room or user not found
- **409 Conflict**: User is already a member

**cURL Example:**
```bash
curl -X POST http://localhost:3000/api/rooms/550e8400-e29b-41d4-a716-446655440001/members \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440004",
    "involvement_level": "Member"
  }'
```

## Message Endpoints

### POST /api/rooms/:room_id/messages

Create a new message in the specified room with deduplication.

**Authentication**: Required (session token)
**Authorization**: User must have access to the room

**Path Parameters:**
- `room_id`: UUID of the room

**Request Body:**
```json
{
  "content": "Hello, world! This is my message.",
  "client_message_id": "550e8400-e29b-41d4-a716-446655440005"
}
```

**Validation Rules:**
- `content`: 1-10,000 characters, required, HTML sanitized
- `client_message_id`: Valid UUID for deduplication, required

**Response (201 Created):**
```json
{
  "message": {
    "id": "550e8400-e29b-41d4-a716-446655440006",
    "room_id": "550e8400-e29b-41d4-a716-446655440001",
    "creator_id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "Hello, world! This is my message.",
    "client_message_id": "550e8400-e29b-41d4-a716-446655440005",
    "created_at": "2023-12-07T10:35:00Z",
    "html_content": null,
    "mentions": [],
    "sound_commands": []
  }
}
```

**Deduplication**: If a message with the same `client_message_id` already exists in the room, the existing message is returned instead of creating a duplicate.

**Rich Text Features:**
- **@mentions**: `@username` automatically creates mentions
- **Sound commands**: `/play sound_name` triggers sound playback
- **HTML formatting**: Basic HTML tags are preserved

**Error Responses:**
- **400 Bad Request**: Invalid content or client_message_id
- **403 Forbidden**: User not authorized for room
- **429 Too Many Requests**: Rate limit exceeded (60 messages/minute)

**cURL Example:**
```bash
curl -X POST http://localhost:3000/api/rooms/550e8400-e29b-41d4-a716-446655440001/messages \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello, world! @john check this out /play tada",
    "client_message_id": "550e8400-e29b-41d4-a716-446655440005"
  }'
```

### GET /api/rooms/:room_id/messages

Retrieve message history for the specified room with pagination.

**Authentication**: Required (session token)
**Authorization**: User must have access to the room

**Path Parameters:**
- `room_id`: UUID of the room

**Query Parameters:**
- `limit`: Number of messages to retrieve (default: 50, max: 100)
- `before`: MessageId to paginate before (optional)

**Response (200 OK):**
```json
{
  "messages": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440006",
      "room_id": "550e8400-e29b-41d4-a716-446655440001",
      "creator_id": "550e8400-e29b-41d4-a716-446655440000",
      "content": "Hello, world! This is my message.",
      "client_message_id": "550e8400-e29b-41d4-a716-446655440005",
      "created_at": "2023-12-07T10:35:00Z",
      "html_content": null,
      "mentions": [],
      "sound_commands": []
    }
  ],
  "has_more": false
}
```

**Pagination**: Use the `before` parameter with the oldest message ID to get earlier messages.

**cURL Example:**
```bash
# Get latest 20 messages
curl -H "Authorization: Bearer <session_token>" \
  "http://localhost:3000/api/rooms/550e8400-e29b-41d4-a716-446655440001/messages?limit=20"

# Get messages before a specific message ID
curl -H "Authorization: Bearer <session_token>" \
  "http://localhost:3000/api/rooms/550e8400-e29b-41d4-a716-446655440001/messages?limit=20&before=550e8400-e29b-41d4-a716-446655440006"
```

## Search Endpoints

### GET /api/search

Search messages with full-text search across user's accessible rooms.

**Authentication**: Required (session token)

**Query Parameters:**
- `q`: Search query (1-100 characters, required)
- `limit`: Number of results (1-100, default: 20)
- `room_id`: Limit search to specific room (optional)

**Response (200 OK):**
```json
{
  "results": [
    {
      "message": {
        "id": "550e8400-e29b-41d4-a716-446655440006",
        "room_id": "550e8400-e29b-41d4-a716-446655440001",
        "creator_id": "550e8400-e29b-41d4-a716-446655440000",
        "content": "Hello, world! This is my message.",
        "created_at": "2023-12-07T10:35:00Z"
      },
      "room_name": "General Discussion",
      "creator_name": "John Doe",
      "relevance_score": 0.95
    }
  ],
  "total_count": 1,
  "query": "hello world"
}
```

**Search Features:**
- Full-text search using SQLite FTS5
- Searches message content only
- Results ranked by relevance
- Only returns messages from accessible rooms

**Error Responses:**
- **400 Bad Request**: Invalid query (too short/long)
- **403 Forbidden**: Access denied to specified room

**cURL Example:**
```bash
# Search all accessible rooms
curl -H "Authorization: Bearer <session_token>" \
  "http://localhost:3000/api/search?q=hello%20world&limit=10"

# Search specific room
curl -H "Authorization: Bearer <session_token>" \
  "http://localhost:3000/api/search?q=hello&room_id=550e8400-e29b-41d4-a716-446655440001"
```

## Sound System Endpoints

### GET /api/sounds

Get list of all available sounds with metadata.

**Authentication**: None required

**Response (200 OK):**
```json
{
  "sounds": [
    {
      "name": "tada",
      "filename": "tada.mp3",
      "size_bytes": 15420,
      "duration_ms": 1200,
      "description": "Celebration sound"
    },
    {
      "name": "bell",
      "filename": "bell.mp3", 
      "size_bytes": 8934,
      "duration_ms": 800,
      "description": "Notification bell"
    }
  ]
}
```

**cURL Example:**
```bash
curl http://localhost:3000/api/sounds
```

### GET /api/sounds/:sound_name

Get MP3 data for a specific sound.

**Authentication**: None required

**Path Parameters:**
- `sound_name`: Name of the sound (without .mp3 extension)

**Response (200 OK):**
- **Content-Type**: `audio/mpeg`
- **Cache-Control**: `public, max-age=86400`
- **Body**: Binary MP3 data

**Error Responses:**
- **400 Bad Request**: Invalid sound name format
- **404 Not Found**: Sound not found

**cURL Example:**
```bash
# Download sound file
curl -o tada.mp3 http://localhost:3000/api/sounds/tada

# Play sound in browser
curl http://localhost:3000/api/sounds/tada | mpg123 -
```

### GET /api/sounds/:sound_name/info

Get metadata for a specific sound.

**Authentication**: None required

**Path Parameters:**
- `sound_name`: Name of the sound (without .mp3 extension)

**Response (200 OK):**
```json
{
  "name": "tada",
  "filename": "tada.mp3",
  "size_bytes": 15420,
  "duration_ms": 1200,
  "description": "Celebration sound"
}
```

**cURL Example:**
```bash
curl http://localhost:3000/api/sounds/tada/info
```

## Push Notification Endpoints

### POST /api/push/subscriptions

Create a new push notification subscription.

**Authentication**: Required (session token)

**Request Body:**
```json
{
  "endpoint": "https://fcm.googleapis.com/fcm/send/...",
  "keys": {
    "p256dh": "BNcRdreALRFXTkOOUHK1EtK2wtaz5Ry4YfYCA_0QTpQtUbVlUK...",
    "auth": "tBHItJI5svbpez7KI4CCXg"
  }
}
```

**Validation Rules:**
- `endpoint`: Valid URL, required
- `keys.p256dh`: Non-empty string, required
- `keys.auth`: Non-empty string, required

**Response (201 Created):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440007",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "endpoint": "https://fcm.googleapis.com/fcm/send/...",
  "created_at": "2023-12-07T10:40:00Z"
}
```

**cURL Example:**
```bash
curl -X POST http://localhost:3000/api/push/subscriptions \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "endpoint": "https://fcm.googleapis.com/fcm/send/...",
    "keys": {
      "p256dh": "BNcRdreALRFXTkOOUHK1EtK2wtaz5Ry4YfYCA_0QTpQtUbVlUK...",
      "auth": "tBHItJI5svbpez7KI4CCXg"
    }
  }'
```

### DELETE /api/push/subscriptions/:id

Delete a push notification subscription.

**Authentication**: Required (session token)

**Path Parameters:**
- `id`: UUID of the subscription

**Response (204 No Content):**
No response body.

**cURL Example:**
```bash
curl -X DELETE http://localhost:3000/api/push/subscriptions/550e8400-e29b-41d4-a716-446655440007 \
  -H "Authorization: Bearer <session_token>"
```

### GET /api/push/preferences

Get notification preferences for the current user.

**Authentication**: Required (session token)

**Response (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "mentions_enabled": true,
  "direct_messages_enabled": true,
  "all_messages_enabled": false,
  "sounds_enabled": true,
  "updated_at": "2023-12-07T10:40:00Z"
}
```

**cURL Example:**
```bash
curl -H "Authorization: Bearer <session_token>" \
  http://localhost:3000/api/push/preferences
```

### PUT /api/push/preferences

Update notification preferences for the current user.

**Authentication**: Required (session token)

**Request Body:**
```json
{
  "mentions_enabled": true,
  "direct_messages_enabled": true,
  "all_messages_enabled": false,
  "sounds_enabled": false
}
```

**Response (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "mentions_enabled": true,
  "direct_messages_enabled": true,
  "all_messages_enabled": false,
  "sounds_enabled": false,
  "updated_at": "2023-12-07T10:45:00Z"
}
```

**cURL Example:**
```bash
curl -X PUT http://localhost:3000/api/push/preferences \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "mentions_enabled": true,
    "sounds_enabled": false
  }'
```

### GET /api/push/vapid-key

Get VAPID public key for client-side subscription setup.

**Authentication**: None required

**Response (200 OK):**
```json
{
  "publicKey": "BNcRdreALRFXTkOOUHK1EtK2wtaz5Ry4YfYCA_0QTpQtUbVlUK..."
}
```

**cURL Example:**
```bash
curl http://localhost:3000/api/push/vapid-key
```

## Bot API Endpoints

### GET /api/bots

List all active bots (admin only).

**Authentication**: Required (session token)
**Authorization**: Admin privileges required

**Response (200 OK):**
```json
{
  "bots": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440008",
      "name": "Weather Bot",
      "bot_token": "bot_abc123...",
      "webhook_url": "https://example.com/webhook",
      "created_at": "2023-12-07T09:00:00Z"
    }
  ],
  "success": true
}
```

**Error Responses:**
- **403 Forbidden**: Admin privileges required

**cURL Example:**
```bash
curl -H "Authorization: Bearer <admin_session_token>" \
  http://localhost:3000/api/bots
```

### POST /api/bots

Create a new bot (admin only).

**Authentication**: Required (session token)
**Authorization**: Admin privileges required

**Request Body:**
```json
{
  "name": "My Bot",
  "description": "A helpful bot",
  "webhook_url": "https://example.com/webhook"
}
```

**Validation Rules:**
- `name`: 1-50 characters, required
- `description`: Max 200 characters, optional
- `webhook_url`: Valid URL, optional

**Response (201 Created):**
```json
{
  "bot": {
    "id": "550e8400-e29b-41d4-a716-446655440009",
    "name": "My Bot",
    "bot_token": "bot_xyz789...",
    "webhook_url": "https://example.com/webhook",
    "created_at": "2023-12-07T10:50:00Z"
  },
  "success": true
}
```

**cURL Example:**
```bash
curl -X POST http://localhost:3000/api/bots \
  -H "Authorization: Bearer <admin_session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Weather Bot",
    "webhook_url": "https://example.com/webhook"
  }'
```

### GET /api/bots/:id

Get bot details (admin only).

**Authentication**: Required (session token)
**Authorization**: Admin privileges required

**Path Parameters:**
- `id`: UUID of the bot

**Response (200 OK):**
```json
{
  "bot": {
    "id": "550e8400-e29b-41d4-a716-446655440008",
    "name": "Weather Bot",
    "bot_token": "bot_abc123...",
    "webhook_url": "https://example.com/webhook",
    "created_at": "2023-12-07T09:00:00Z"
  },
  "success": true
}
```

**Error Responses:**
- **403 Forbidden**: Admin privileges required
- **404 Not Found**: Bot not found

**cURL Example:**
```bash
curl -H "Authorization: Bearer <admin_session_token>" \
  http://localhost:3000/api/bots/550e8400-e29b-41d4-a716-446655440008
```

### PUT /api/bots/:id

Update bot details (admin only).

**Authentication**: Required (session token)
**Authorization**: Admin privileges required

**Path Parameters:**
- `id`: UUID of the bot

**Request Body:**
```json
{
  "name": "Updated Bot Name",
  "webhook_url": "https://example.com/new-webhook"
}
```

**Response (200 OK):**
```json
{
  "bot": {
    "id": "550e8400-e29b-41d4-a716-446655440008",
    "name": "Updated Bot Name",
    "bot_token": "bot_abc123...",
    "webhook_url": "https://example.com/new-webhook",
    "created_at": "2023-12-07T09:00:00Z"
  },
  "success": true
}
```

**cURL Example:**
```bash
curl -X PUT http://localhost:3000/api/bots/550e8400-e29b-41d4-a716-446655440008 \
  -H "Authorization: Bearer <admin_session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Updated Weather Bot"
  }'
```

### DELETE /api/bots/:id

Delete (deactivate) a bot (admin only).

**Authentication**: Required (session token)
**Authorization**: Admin privileges required

**Path Parameters:**
- `id`: UUID of the bot

**Response (200 OK):**
```json
{
  "message": "Bot deleted successfully",
  "success": true
}
```

**cURL Example:**
```bash
curl -X DELETE http://localhost:3000/api/bots/550e8400-e29b-41d4-a716-446655440008 \
  -H "Authorization: Bearer <admin_session_token>"
```

### POST /api/bots/:id/reset-token

Reset bot API token (admin only).

**Authentication**: Required (session token)
**Authorization**: Admin privileges required

**Path Parameters:**
- `id`: UUID of the bot

**Response (200 OK):**
```json
{
  "bot_key": "550e8400-e29b-41d4-a716-446655440008-new_token_abc123",
  "message": "Bot token reset successfully",
  "success": true
}
```

**cURL Example:**
```bash
curl -X POST http://localhost:3000/api/bots/550e8400-e29b-41d4-a716-446655440008/reset-token \
  -H "Authorization: Bearer <admin_session_token>"
```

### POST /rooms/:room_id/bot/:bot_key/messages

Create a message from a bot (bot API endpoint).

**Authentication**: Bot key in URL path

**Path Parameters:**
- `room_id`: UUID of the room
- `bot_key`: Bot authentication key (format: `{bot_id}-{bot_token}`)

**Request Body:**
```json
{
  "content": "Hello from bot! Weather today is sunny."
}
```

**Validation Rules:**
- `content`: 1-10,000 characters, required

**Response (201 Created):**
```json
{
  "message": {
    "id": "550e8400-e29b-41d4-a716-446655440010",
    "room_id": "550e8400-e29b-41d4-a716-446655440001",
    "creator_id": "550e8400-e29b-41d4-a716-446655440008",
    "content": "Hello from bot! Weather today is sunny.",
    "client_message_id": "550e8400-e29b-41d4-a716-446655440011",
    "created_at": "2023-12-07T10:55:00Z"
  },
  "success": true
}
```

**Error Responses:**
- **401 Unauthorized**: Invalid bot key
- **403 Forbidden**: Bot not authorized for room
- **404 Not Found**: Room not found

**cURL Example:**
```bash
curl -X POST http://localhost:3000/rooms/550e8400-e29b-41d4-a716-446655440001/bot/550e8400-e29b-41d4-a716-446655440008-bot_abc123/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello from Weather Bot! Today will be sunny with a high of 75°F."
  }'
```

## Health and Monitoring Endpoints

### GET /health

Basic health check endpoint.

**Authentication**: None required

**Response (200 OK):**
```json
{
  "status": "healthy",
  "timestamp": "2023-12-07T10:55:00Z",
  "version": "1.0.0"
}
```

**cURL Example:**
```bash
curl http://localhost:3000/health
```

### GET /health/ready

Readiness check (includes database connectivity).

**Authentication**: None required

**Response (200 OK):**
```json
{
  "status": "ready",
  "checks": {
    "database": "healthy",
    "services": "healthy"
  },
  "timestamp": "2023-12-07T10:55:00Z"
}
```

**cURL Example:**
```bash
curl http://localhost:3000/health/ready
```

### GET /health/live

Liveness check (basic server responsiveness).

**Authentication**: None required

**Response (200 OK):**
```json
{
  "status": "alive",
  "timestamp": "2023-12-07T10:55:00Z"
}
```

**cURL Example:**
```bash
curl http://localhost:3000/health/live
```

### GET /metrics

Prometheus metrics endpoint (if enabled).

**Authentication**: None required

**Response (200 OK):**
```
# HELP http_requests_total Total number of HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",status="200"} 1234
http_requests_total{method="POST",status="201"} 567

# HELP websocket_connections_active Active WebSocket connections
# TYPE websocket_connections_active gauge
websocket_connections_active 42
```

**cURL Example:**
```bash
curl http://localhost:3000/metrics
```

## WebSocket API

### Connection

Connect to WebSocket endpoint with authentication.

**URL**: `ws://localhost:3000/ws`

**Authentication Methods:**
1. Query parameter: `ws://localhost:3000/ws?token=<session_token>`
2. Authorization header: `Authorization: Bearer <session_token>`
3. Cookie: `session_token=<session_token>`

### Incoming Message Types (Client → Server)

#### Create Message
```json
{
  "type": "CreateMessage",
  "room_id": "550e8400-e29b-41d4-a716-446655440001",
  "content": "Hello, world!",
  "client_message_id": "550e8400-e29b-41d4-a716-446655440005"
}
```

#### Join Room
```json
{
  "type": "JoinRoom",
  "room_id": "550e8400-e29b-41d4-a716-446655440001"
}
```

#### Leave Room
```json
{
  "type": "LeaveRoom",
  "room_id": "550e8400-e29b-41d4-a716-446655440001"
}
```

#### Start Typing
```json
{
  "type": "StartTyping",
  "room_id": "550e8400-e29b-41d4-a716-446655440001"
}
```

#### Stop Typing
```json
{
  "type": "StopTyping",
  "room_id": "550e8400-e29b-41d4-a716-446655440001"
}
```

#### Update Last Seen
```json
{
  "type": "UpdateLastSeen",
  "message_id": "550e8400-e29b-41d4-a716-446655440006"
}
```

### Outgoing Message Types (Server → Client)

#### New Message
```json
{
  "type": "NewMessage",
  "message": {
    "id": "550e8400-e29b-41d4-a716-446655440006",
    "room_id": "550e8400-e29b-41d4-a716-446655440001",
    "creator_id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "Hello, world!",
    "created_at": "2023-12-07T10:35:00Z"
  }
}
```

#### User Joined
```json
{
  "type": "UserJoined",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "room_id": "550e8400-e29b-41d4-a716-446655440001"
}
```

#### User Left
```json
{
  "type": "UserLeft",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "room_id": "550e8400-e29b-41d4-a716-446655440001"
}
```

#### Typing Start
```json
{
  "type": "TypingStart",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "room_id": "550e8400-e29b-41d4-a716-446655440001"
}
```

#### Typing Stop
```json
{
  "type": "TypingStop",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "room_id": "550e8400-e29b-41d4-a716-446655440001"
}
```

#### Presence Update
```json
{
  "type": "PresenceUpdate",
  "room_id": "550e8400-e29b-41d4-a716-446655440001",
  "online_users": [
    "550e8400-e29b-41d4-a716-446655440000",
    "550e8400-e29b-41d4-a716-446655440002"
  ]
}
```

#### Sound Playback
```json
{
  "type": "SoundPlayback",
  "sound_name": "tada",
  "triggered_by": "550e8400-e29b-41d4-a716-446655440000",
  "room_id": "550e8400-e29b-41d4-a716-446655440001",
  "timestamp": "2023-12-07T10:35:00Z"
}
```

#### Error
```json
{
  "type": "Error",
  "message": "Failed to create message: Invalid content",
  "code": "MESSAGE_CREATION_FAILED"
}
```

### JavaScript WebSocket Example

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:3000/ws?token=your_session_token');

// Handle connection open
ws.onopen = function() {
    console.log('WebSocket connected');
    
    // Join a room
    ws.send(JSON.stringify({
        type: 'JoinRoom',
        room_id: '550e8400-e29b-41d4-a716-446655440001'
    }));
};

// Handle incoming messages
ws.onmessage = function(event) {
    const message = JSON.parse(event.data);
    console.log('Received:', message);
    
    switch(message.type) {
        case 'NewMessage':
            displayMessage(message.message);
            break;
        case 'UserJoined':
            showUserJoined(message.user_id);
            break;
        case 'TypingStart':
            showTypingIndicator(message.user_id);
            break;
        case 'SoundPlayback':
            playSound(message.sound_name);
            break;
    }
};

// Send a message
function sendMessage(roomId, content) {
    ws.send(JSON.stringify({
        type: 'CreateMessage',
        room_id: roomId,
        content: content,
        client_message_id: generateUUID()
    }));
}

// Start typing indicator
function startTyping(roomId) {
    ws.send(JSON.stringify({
        type: 'StartTyping',
        room_id: roomId
    }));
}

// Handle connection close
ws.onclose = function() {
    console.log('WebSocket disconnected');
    // Implement reconnection logic here
};
```

## Common Usage Patterns

### Complete Chat Flow

```bash
# 1. Login
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password123"}')

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.session_token')

# 2. Get user's rooms
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/rooms

# 3. Get messages from a room
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/api/rooms/550e8400-e29b-41d4-a716-446655440001/messages?limit=10"

# 4. Send a message
curl -X POST http://localhost:3000/api/rooms/550e8400-e29b-41d4-a716-446655440001/messages \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello everyone! /play tada",
    "client_message_id": "'$(uuidgen)'"
  }'

# 5. Search messages
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/api/search?q=hello&limit=5"

# 6. Logout
curl -X POST http://localhost:3000/api/auth/logout \
  -H "Authorization: Bearer $TOKEN"
```

### Bot Integration Example

```bash
# 1. Admin creates bot
BOT_RESPONSE=$(curl -s -X POST http://localhost:3000/api/bots \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Weather Bot",
    "webhook_url": "https://example.com/webhook"
  }')

BOT_ID=$(echo $BOT_RESPONSE | jq -r '.bot.id')
BOT_TOKEN=$(echo $BOT_RESPONSE | jq -r '.bot.bot_token')
BOT_KEY="$BOT_ID-$BOT_TOKEN"

# 2. Bot sends message
curl -X POST http://localhost:3000/rooms/550e8400-e29b-41d4-a716-446655440001/bot/$BOT_KEY/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "🌤️ Weather update: Sunny, 75°F. Have a great day!"
  }'
```

### Push Notification Setup

```bash
# 1. Get VAPID public key
VAPID_KEY=$(curl -s http://localhost:3000/api/push/vapid-key | jq -r '.publicKey')

# 2. Create subscription (after getting keys from browser)
curl -X POST http://localhost:3000/api/push/subscriptions \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "endpoint": "https://fcm.googleapis.com/fcm/send/...",
    "keys": {
      "p256dh": "BNcRdreALRFXTkOOUHK1EtK2wtaz5Ry4YfYCA_0QTpQtUbVlUK...",
      "auth": "tBHItJI5svbpez7KI4CCXg"
    }
  }'

# 3. Update notification preferences
curl -X PUT http://localhost:3000/api/push/preferences \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "mentions_enabled": true,
    "direct_messages_enabled": true,
    "all_messages_enabled": false
  }'
```

## Performance Characteristics

### Response Time Targets

- **Authentication**: < 100ms
- **Message Creation**: < 200ms
- **Message Retrieval**: < 150ms
- **Search Queries**: < 500ms
- **WebSocket Messages**: < 50ms

### Throughput Targets

- **HTTP Requests**: 1,000+ req/sec
- **WebSocket Messages**: 50 msg/sec system-wide
- **Concurrent Connections**: 500+ WebSocket connections
- **Search Queries**: 100+ queries/sec

### Resource Usage

- **Memory Usage**: 30-60MB total
- **CPU Usage**: < 50% single core
- **Disk I/O**: SQLite WAL mode for optimal performance
- **Network Bandwidth**: Minimal overhead with efficient JSON

## Security Considerations

### Input Validation

- All user input is validated and sanitized
- HTML content is sanitized using ammonia crate
- SQL injection prevention through parameterized queries
- XSS prevention through content sanitization

### Authentication Security

- Secure session token generation using cryptographically secure random
- Session tokens have configurable expiration
- Bot tokens use secure random generation
- Password hashing with bcrypt

### Rate Limiting

- Global rate limiting per IP address
- User-specific rate limiting
- Endpoint-specific rate limiting
- WebSocket message rate limiting

### CORS and Headers

- Configurable CORS origins
- Security headers (CSP, HSTS, X-Frame-Options)
- Request size limits
- Timeout protection

## Troubleshooting

### Common Error Codes

- **INVALID_CREDENTIALS**: Check email/password combination
- **SESSION_EXPIRED**: Re-authenticate to get new session token
- **ACCESS_DENIED**: User lacks permission for requested resource
- **RATE_LIMIT_EXCEEDED**: Reduce request frequency
- **VALIDATION_FAILED**: Check request format and required fields
- **ROOM_NOT_FOUND**: Verify room ID exists and user has access
- **INVALID_BOT_KEY**: Check bot key format and validity

### Debugging Tips

1. **Check Authentication**: Ensure session token is valid and not expired
2. **Verify Permissions**: Confirm user has access to requested rooms/resources
3. **Validate Input**: Check request format matches API specification
4. **Monitor Rate Limits**: Check rate limit headers in responses
5. **WebSocket Issues**: Verify authentication and message format
6. **Search Problems**: Ensure query length and room access permissions

### Health Check Endpoints

Use health check endpoints to verify system status:

- `/health` - Basic health check
- `/health/ready` - Readiness check with dependencies
- `/health/live` - Liveness check for load balancers
- `/metrics` - Detailed metrics (if enabled)

This comprehensive API reference provides all the information needed to integrate with the Campfire Rust API, including authentication, all endpoints, request/response formats, error handling, and practical examples.