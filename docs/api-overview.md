# API Overview

## API Architecture

The Campfire Rust API provides both REST endpoints and WebSocket connections for complete chat functionality.

## API Structure Overview

```mermaid
graph TD
    subgraph "API Ecosystem"
        direction TB
        CLIENTS[Client Applications<br/>Web + Mobile + Bots]
        GATEWAY[API Gateway<br/>Axum Router + Middleware]
        ENDPOINTS[API Endpoints<br/>REST + WebSocket]
        SERVICES[Service Layer<br/>Business Logic]
    end
    
    CLIENTS --> GATEWAY
    GATEWAY --> ENDPOINTS
    ENDPOINTS --> SERVICES
    
    classDef ecosystem fill:#e1f5fe,stroke:#01579b,stroke-width:3px
    class CLIENTS,GATEWAY,ENDPOINTS,SERVICES ecosystem
```

### Detailed API Architecture

```mermaid
graph TD
    subgraph "Client Applications"
        direction LR
        WEB[Web Browser<br/>HTML + HTMX]
        MOBILE[Mobile Apps<br/>REST + WebSocket]
        BOTS[Bot Integrations<br/>Webhook + API]
    end
    
    subgraph "API Gateway Layer"
        direction TB
        ROUTER[Axum Router<br/>Route Matching]
        MIDDLEWARE[Middleware Stack<br/>Auth + CORS + Rate Limiting]
        HANDLERS[Request Handlers<br/>HTTP + WebSocket]
    end
    
    subgraph "REST API Endpoints"
        direction TB
        AUTH_API[Authentication<br/>/api/auth/*]
        USERS_API[Users<br/>/api/users/*]
        ROOMS_API[Rooms<br/>/api/rooms/*]
        MESSAGES_API[Messages<br/>/api/rooms/:id/messages]
        SEARCH_API[Search<br/>/api/search]
        HEALTH_API[Health<br/>/health, /metrics]
    end
    
    subgraph "WebSocket API"
        direction TB
        WS_ENDPOINT[WebSocket Endpoint<br/>/ws]
        REALTIME[Real-time Features<br/>Messages + Presence]
        EVENTS[Event Broadcasting<br/>Room-based]
    end
    
    subgraph "Service Layer"
        direction TB
        AUTH_SVC[AuthService]
        MESSAGE_SVC[MessageService]
        ROOM_SVC[RoomService]
        SEARCH_SVC[SearchService]
    end
    
    WEB --> ROUTER
    MOBILE --> ROUTER
    BOTS --> ROUTER
    
    ROUTER --> MIDDLEWARE
    MIDDLEWARE --> HANDLERS
    
    HANDLERS --> AUTH_API
    HANDLERS --> USERS_API
    HANDLERS --> ROOMS_API
    HANDLERS --> MESSAGES_API
    HANDLERS --> SEARCH_API
    HANDLERS --> HEALTH_API
    
    HANDLERS --> WS_ENDPOINT
    WS_ENDPOINT --> REALTIME
    REALTIME --> EVENTS
    
    AUTH_API --> AUTH_SVC
    USERS_API --> AUTH_SVC
    ROOMS_API --> ROOM_SVC
    MESSAGES_API --> MESSAGE_SVC
    SEARCH_API --> SEARCH_SVC
    
    WS_ENDPOINT --> MESSAGE_SVC
    WS_ENDPOINT --> ROOM_SVC
    
    classDef client fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef gateway fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef rest fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef websocket fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    classDef service fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class WEB,MOBILE,BOTS client
    class ROUTER,MIDDLEWARE,HANDLERS gateway
    class AUTH_API,USERS_API,ROOMS_API,MESSAGES_API,SEARCH_API,HEALTH_API rest
    class WS_ENDPOINT,REALTIME,EVENTS websocket
    class AUTH_SVC,MESSAGE_SVC,ROOM_SVC,SEARCH_SVC service
```

## REST API Endpoints

### Authentication Endpoints

```mermaid
graph TD
    subgraph "Authentication Flow"
        direction TB
        LOGIN[POST /api/auth/login<br/>Email + Password]
        VALIDATE[Credential Validation<br/>bcrypt Hash Check]
        SESSION[Session Creation<br/>Secure Token]
        RESPONSE[Login Response<br/>User + Token]
    end
    
    subgraph "Session Management"
        direction TB
        ME[GET /api/users/me<br/>Current User Info]
        LOGOUT[POST /api/auth/logout<br/>Session Termination]
        REFRESH[Token Refresh<br/>Automatic Extension]
    end
    
    subgraph "User Registration"
        direction TB
        REGISTER[POST /api/users<br/>Create Account]
        VALIDATION[Input Validation<br/>Email + Password Rules]
        CREATION[User Creation<br/>Hash Password]
    end
    
    LOGIN --> VALIDATE
    VALIDATE --> SESSION
    SESSION --> RESPONSE
    
    RESPONSE --> ME
    ME --> LOGOUT
    LOGOUT --> REFRESH
    
    REGISTER --> VALIDATION
    VALIDATION --> CREATION
    
    classDef auth fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef session fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef register fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class LOGIN,VALIDATE,SESSION,RESPONSE auth
    class ME,LOGOUT,REFRESH session
    class REGISTER,VALIDATION,CREATION register
```

### Room Management Endpoints

```mermaid
graph TD
    subgraph "Room Operations"
        direction TB
        LIST[GET /api/rooms<br/>User's Rooms]
        CREATE[POST /api/rooms<br/>Create Room]
        DETAILS[GET /api/rooms/:id<br/>Room Details]
        UPDATE[PUT /api/rooms/:id<br/>Update Room]
    end
    
    subgraph "Membership Management"
        direction TB
        JOIN[POST /api/rooms/:id/join<br/>Join Room]
        LEAVE[POST /api/rooms/:id/leave<br/>Leave Room]
        INVITE[POST /api/rooms/:id/invite<br/>Invite User]
        MEMBERS[GET /api/rooms/:id/members<br/>List Members]
    end
    
    subgraph "Access Control"
        direction TB
        PERMISSIONS[Permission Check<br/>Room Access]
        ROLES[Role Validation<br/>Admin vs Member]
        VISIBILITY[Visibility Rules<br/>Open vs Closed]
    end
    
    LIST --> JOIN
    CREATE --> LEAVE
    DETAILS --> INVITE
    UPDATE --> MEMBERS
    
    JOIN --> PERMISSIONS
    LEAVE --> ROLES
    INVITE --> VISIBILITY
    
    classDef operations fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef membership fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef access fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class LIST,CREATE,DETAILS,UPDATE operations
    class JOIN,LEAVE,INVITE,MEMBERS membership
    class PERMISSIONS,ROLES,VISIBILITY access
```

### Message Endpoints

```mermaid
graph TD
    subgraph "Message Operations"
        direction TB
        SEND[POST /api/rooms/:id/messages<br/>Send Message]
        HISTORY[GET /api/rooms/:id/messages<br/>Message History]
        EDIT[PUT /api/messages/:id<br/>Edit Message]
        DELETE[DELETE /api/messages/:id<br/>Delete Message]
    end
    
    subgraph "Message Processing"
        direction TB
        VALIDATE[Content Validation<br/>Length + HTML Sanitization]
        DEDUP[Deduplication Check<br/>client_message_id]
        PERSIST[Database Persistence<br/>Atomic Transaction]
        BROADCAST[WebSocket Broadcast<br/>Room Subscribers]
    end
    
    subgraph "Message Features"
        direction TB
        MENTIONS[@ Mentions<br/>User Notifications]
        SOUNDS[Sound Commands<br/>/play integration]
        FORMATTING[Rich Text<br/>HTML Support]
        PAGINATION[History Pagination<br/>Cursor-based]
    end
    
    SEND --> VALIDATE
    HISTORY --> DEDUP
    EDIT --> PERSIST
    DELETE --> BROADCAST
    
    VALIDATE --> MENTIONS
    DEDUP --> SOUNDS
    PERSIST --> FORMATTING
    BROADCAST --> PAGINATION
    
    classDef operations fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef processing fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef features fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class SEND,HISTORY,EDIT,DELETE operations
    class VALIDATE,DEDUP,PERSIST,BROADCAST processing
    class MENTIONS,SOUNDS,FORMATTING,PAGINATION features
```

## WebSocket API

### Connection Management

```mermaid
graph TD
    subgraph "Connection Lifecycle"
        direction TB
        UPGRADE[HTTP → WebSocket<br/>Protocol Upgrade]
        AUTH_WS[Authentication<br/>Token Validation]
        REGISTER[Connection Registration<br/>ConnectionManager]
        HEARTBEAT[Heartbeat/Ping<br/>Connection Health]
        DISCONNECT[Graceful Disconnect<br/>Cleanup]
    end
    
    subgraph "Message Types"
        direction TB
        CREATE_MSG[CreateMessage<br/>Send New Message]
        JOIN_ROOM[JoinRoom<br/>Subscribe to Room]
        LEAVE_ROOM[LeaveRoom<br/>Unsubscribe]
        TYPING[StartTyping/StopTyping<br/>Typing Indicators]
        LAST_SEEN[UpdateLastSeen<br/>Read Receipts]
    end
    
    subgraph "Event Broadcasting"
        direction TB
        NEW_MSG[NewMessage<br/>Message Broadcast]
        USER_JOIN[UserJoined<br/>Presence Update]
        USER_LEAVE[UserLeft<br/>Presence Update]
        TYPING_EVENT[TypingStart/Stop<br/>Typing Broadcast]
        PRESENCE[PresenceUpdate<br/>Online Users]
    end
    
    UPGRADE --> AUTH_WS
    AUTH_WS --> REGISTER
    REGISTER --> HEARTBEAT
    HEARTBEAT --> DISCONNECT
    
    REGISTER --> CREATE_MSG
    CREATE_MSG --> JOIN_ROOM
    JOIN_ROOM --> LEAVE_ROOM
    LEAVE_ROOM --> TYPING
    TYPING --> LAST_SEEN
    
    CREATE_MSG --> NEW_MSG
    JOIN_ROOM --> USER_JOIN
    LEAVE_ROOM --> USER_LEAVE
    TYPING --> TYPING_EVENT
    LAST_SEEN --> PRESENCE
    
    classDef lifecycle fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef messages fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef events fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class UPGRADE,AUTH_WS,REGISTER,HEARTBEAT,DISCONNECT lifecycle
    class CREATE_MSG,JOIN_ROOM,LEAVE_ROOM,TYPING,LAST_SEEN messages
    class NEW_MSG,USER_JOIN,USER_LEAVE,TYPING_EVENT,PRESENCE events
```

## Authentication & Authorization

### Security Model

```mermaid
graph TD
    subgraph "Authentication Methods"
        direction TB
        SESSION[Session Tokens<br/>HTTP-only Cookies]
        BEARER[Bearer Tokens<br/>Authorization Header]
        BOT_TOKEN[Bot Tokens<br/>API Integration]
        WEBSOCKET_AUTH[WebSocket Auth<br/>Query Parameter]
    end
    
    subgraph "Authorization Levels"
        direction TB
        ADMIN[Admin Users<br/>Full Access]
        MEMBER[Room Members<br/>Room Access]
        BOT[Bot Users<br/>API Access]
        GUEST[Guest Users<br/>Limited Access]
    end
    
    subgraph "Permission Model"
        direction TB
        ROOM_ACCESS[Room Access<br/>Membership Check]
        MESSAGE_PERM[Message Permissions<br/>Create/Edit/Delete]
        ADMIN_PERM[Admin Permissions<br/>User/Room Management]
        API_PERM[API Permissions<br/>Rate Limiting]
    end
    
    SESSION --> ADMIN
    BEARER --> MEMBER
    BOT_TOKEN --> BOT
    WEBSOCKET_AUTH --> GUEST
    
    ADMIN --> ROOM_ACCESS
    MEMBER --> MESSAGE_PERM
    BOT --> ADMIN_PERM
    GUEST --> API_PERM
    
    classDef auth fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef authz fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef perms fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class SESSION,BEARER,BOT_TOKEN,WEBSOCKET_AUTH auth
    class ADMIN,MEMBER,BOT,GUEST authz
    class ROOM_ACCESS,MESSAGE_PERM,ADMIN_PERM,API_PERM perms
```

## Error Handling

### Error Response Format

```mermaid
graph TD
    subgraph "Error Categories"
        direction TB
        CLIENT[4xx Client Errors<br/>Bad Request, Unauthorized]
        SERVER[5xx Server Errors<br/>Internal Error, Unavailable]
        VALIDATION[Validation Errors<br/>Input Format Issues]
        BUSINESS[Business Logic Errors<br/>Rule Violations]
    end
    
    subgraph "Error Response Structure"
        direction TB
        ERROR_CODE[Error Code<br/>Machine-readable]
        ERROR_MSG[Error Message<br/>Human-readable]
        ERROR_DETAILS[Error Details<br/>Field-specific Info]
        ERROR_CONTEXT[Error Context<br/>Request Information]
    end
    
    subgraph "Error Handling Flow"
        direction TB
        CATCH[Exception Catching<br/>Service Layer]
        MAP[Error Mapping<br/>HTTP Status Codes]
        LOG[Error Logging<br/>Structured Logs]
        RESPOND[Error Response<br/>JSON Format]
    end
    
    CLIENT --> ERROR_CODE
    SERVER --> ERROR_MSG
    VALIDATION --> ERROR_DETAILS
    BUSINESS --> ERROR_CONTEXT
    
    ERROR_CODE --> CATCH
    ERROR_MSG --> MAP
    ERROR_DETAILS --> LOG
    ERROR_CONTEXT --> RESPOND
    
    classDef categories fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef structure fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef handling fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class CLIENT,SERVER,VALIDATION,BUSINESS categories
    class ERROR_CODE,ERROR_MSG,ERROR_DETAILS,ERROR_CONTEXT structure
    class CATCH,MAP,LOG,RESPOND handling
```

## Rate Limiting

### Rate Limiting Strategy

```mermaid
graph TD
    subgraph "Rate Limiting Tiers"
        direction TB
        GLOBAL[Global Rate Limit<br/>Per IP Address]
        USER[User Rate Limit<br/>Per Authenticated User]
        ENDPOINT[Endpoint Rate Limit<br/>Per API Endpoint]
        WEBSOCKET[WebSocket Rate Limit<br/>Per Connection]
    end
    
    subgraph "Limiting Algorithms"
        direction TB
        TOKEN_BUCKET[Token Bucket<br/>Burst Allowance]
        SLIDING_WINDOW[Sliding Window<br/>Time-based Limits]
        FIXED_WINDOW[Fixed Window<br/>Reset Intervals]
    end
    
    subgraph "Rate Limit Response"
        direction TB
        HEADERS[Rate Limit Headers<br/>X-RateLimit-*]
        STATUS[429 Too Many Requests<br/>HTTP Status]
        RETRY[Retry-After Header<br/>Backoff Guidance]
        WEBSOCKET_CLOSE[WebSocket Close<br/>Rate Limit Exceeded]
    end
    
    GLOBAL --> TOKEN_BUCKET
    USER --> SLIDING_WINDOW
    ENDPOINT --> FIXED_WINDOW
    WEBSOCKET --> TOKEN_BUCKET
    
    TOKEN_BUCKET --> HEADERS
    SLIDING_WINDOW --> STATUS
    FIXED_WINDOW --> RETRY
    TOKEN_BUCKET --> WEBSOCKET_CLOSE
    
    classDef tiers fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef algorithms fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef response fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class GLOBAL,USER,ENDPOINT,WEBSOCKET tiers
    class TOKEN_BUCKET,SLIDING_WINDOW,FIXED_WINDOW algorithms
    class HEADERS,STATUS,RETRY,WEBSOCKET_CLOSE response
```

## API Versioning

### Versioning Strategy

```mermaid
graph TD
    subgraph "Current Version (v1)"
        direction TB
        V1_REST[REST API v1<br/>/api/v1/*]
        V1_WS[WebSocket API v1<br/>/ws]
        V1_FEATURES[Current Features<br/>Complete MVP]
    end
    
    subgraph "Future Versions"
        direction TB
        V2_PLANNING[v2 Planning<br/>File Attachments]
        V2_FEATURES[Enhanced Features<br/>Avatar Uploads]
        V3_ADVANCED[v3 Advanced<br/>OpenGraph Previews]
    end
    
    subgraph "Compatibility Strategy"
        direction TB
        BACKWARDS[Backwards Compatibility<br/>Maintain v1 Support]
        DEPRECATION[Deprecation Policy<br/>6-month Notice]
        MIGRATION[Migration Guide<br/>Version Upgrade Path]
    end
    
    V1_REST --> V2_PLANNING
    V1_WS --> V2_FEATURES
    V1_FEATURES --> V3_ADVANCED
    
    V2_PLANNING --> BACKWARDS
    V2_FEATURES --> DEPRECATION
    V3_ADVANCED --> MIGRATION
    
    classDef current fill:#e8f5e8,stroke:#2e7d32,stroke-width:3px
    classDef future fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef compatibility fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class V1_REST,V1_WS,V1_FEATURES current
    class V2_PLANNING,V2_FEATURES,V3_ADVANCED future
    class BACKWARDS,DEPRECATION,MIGRATION compatibility
```

## Performance Characteristics

### API Performance Metrics

```mermaid
graph TD
    subgraph "Response Time Targets"
        direction TB
        AUTH_TIME[Authentication<br/>< 100ms]
        MESSAGE_TIME[Message Creation<br/>< 200ms]
        SEARCH_TIME[Search Queries<br/>< 500ms]
        WEBSOCKET_TIME[WebSocket Messages<br/>< 50ms]
    end
    
    subgraph "Throughput Targets"
        direction TB
        HTTP_RPS[HTTP Requests<br/>1K+ req/sec]
        WS_MSG[WebSocket Messages<br/>50 msg/sec system-wide]
        CONCURRENT[Concurrent Connections<br/>500+ WebSocket]
        SEARCH_QPS[Search Queries<br/>100+ queries/sec]
    end
    
    subgraph "Resource Usage"
        direction TB
        MEMORY[Memory Usage<br/>30-60MB total]
        CPU[CPU Usage<br/>< 50% single core]
        DISK[Disk I/O<br/>SQLite WAL mode]
        NETWORK[Network Bandwidth<br/>Minimal overhead]
    end
    
    AUTH_TIME --> HTTP_RPS
    MESSAGE_TIME --> WS_MSG
    SEARCH_TIME --> CONCURRENT
    WEBSOCKET_TIME --> SEARCH_QPS
    
    HTTP_RPS --> MEMORY
    WS_MSG --> CPU
    CONCURRENT --> DISK
    SEARCH_QPS --> NETWORK
    
    classDef response fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef throughput fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef resources fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class AUTH_TIME,MESSAGE_TIME,SEARCH_TIME,WEBSOCKET_TIME response
    class HTTP_RPS,WS_MSG,CONCURRENT,SEARCH_QPS throughput
    class MEMORY,CPU,DISK,NETWORK resources
```

## API Documentation Links

- **[Search API](search-api.md)** - Full-text search functionality
- **[WebSocket API](websocket-api.md)** - Real-time communication
- **[Architecture](architecture.md)** - System architecture overview
- **[Deployment](deployment.md)** - Deployment and operations guide
- **[Development](development.md)** - Development workflow and guidelines

## Quick Reference

### Base URLs
- **REST API**: `http://localhost:3000/api`
- **WebSocket**: `ws://localhost:3000/ws`
- **Health Check**: `http://localhost:3000/health`
- **Metrics**: `http://localhost:3000/metrics`

### Authentication
```bash
# Login and get session token
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password"}'

# Use session token in subsequent requests
curl -H "Authorization: Bearer <session_token>" \
  http://localhost:3000/api/users/me
```

### WebSocket Connection
```javascript
const ws = new WebSocket('ws://localhost:3000/ws?token=<session_token>');
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};
```

This API overview provides a comprehensive understanding of the Campfire Rust API architecture, endpoints, and usage patterns.