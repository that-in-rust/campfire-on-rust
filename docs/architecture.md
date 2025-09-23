# Architecture Documentation

## System Architecture Overview

This document provides a comprehensive view of the Campfire Rust rewrite architecture, following Rails-compatible patterns with Rust performance benefits.

## High-Level Architecture

```mermaid
graph TD
    subgraph "Client Layer"
        direction LR
        WEB[Web Browser<br/>HTML + HTMX]
        API[API Clients<br/>REST + JSON]
        WS_CLIENT[WebSocket<br/>Real-time]
    end
    
    subgraph "Single Rust Binary"
        subgraph "Web Layer"
            direction LR
            HTTP[HTTP Server<br/>Axum Framework]
            WS[WebSocket Handler<br/>tokio-tungstenite]
            STATIC[Static Assets<br/>rust-embed]
        end
        
        subgraph "Middleware Layer"
            direction LR
            AUTH_MW[Authentication<br/>Session Validation]
            RATE[Rate Limiting<br/>Governor]
            CORS[CORS Protection<br/>tower-http]
            LOG[Logging<br/>tracing]
        end
        
        subgraph "Service Layer"
            direction TB
            AUTH_SVC[AuthService<br/>Login/Sessions]
            MSG_SVC[MessageService<br/>CRUD + Broadcasting]
            ROOM_SVC[RoomService<br/>Membership + Access]
            USER_SVC[UserService<br/>Profile Management]
            SEARCH_SVC[SearchService<br/>FTS5 Queries]
            PUSH_SVC[PushService<br/>Web Push + VAPID]
        end
        
        subgraph "Data Layer"
            direction LR
            DB[(SQLite Database<br/>WAL Mode)]
            FTS[(FTS5 Virtual Table<br/>Message Search)]
            CACHE[In-Memory Cache<br/>Sessions + Presence]
        end
        
        subgraph "Background Tasks"
            direction LR
            WEBHOOK[Webhook Delivery<br/>Bot Integration]
            CLEANUP[Connection Cleanup<br/>Presence TTL]
            METRICS[Metrics Collection<br/>Prometheus]
        end
    end
    
    %% Client connections
    WEB --> HTTP
    API --> HTTP
    WS_CLIENT --> WS
    
    %% Web layer to middleware
    HTTP --> AUTH_MW
    HTTP --> RATE
    HTTP --> CORS
    HTTP --> LOG
    WS --> AUTH_MW
    
    %% Middleware to services
    AUTH_MW --> AUTH_SVC
    AUTH_MW --> MSG_SVC
    AUTH_MW --> ROOM_SVC
    AUTH_MW --> USER_SVC
    AUTH_MW --> SEARCH_SVC
    
    %% Services to data
    AUTH_SVC --> DB
    AUTH_SVC --> CACHE
    MSG_SVC --> DB
    MSG_SVC --> FTS
    ROOM_SVC --> DB
    USER_SVC --> DB
    SEARCH_SVC --> FTS
    PUSH_SVC --> CACHE
    
    %% WebSocket to services
    WS --> MSG_SVC
    WS --> ROOM_SVC
    WS --> CACHE
    
    %% Background tasks
    MSG_SVC --> WEBHOOK
    WS --> CLEANUP
    HTTP --> METRICS
    
    classDef client fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef web fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef middleware fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    classDef service fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef data fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    classDef background fill:#e0f2f1,stroke:#00695c,stroke-width:2px
    
    class WEB,API,WS_CLIENT client
    class HTTP,WS,STATIC web
    class AUTH_MW,RATE,CORS,LOG middleware
    class AUTH_SVC,MSG_SVC,ROOM_SVC,USER_SVC,SEARCH_SVC,PUSH_SVC service
    class DB,FTS,CACHE data
    class WEBHOOK,CLEANUP,METRICS background
```

## Core Components Deep Dive

### Web Layer Architecture

```mermaid
graph TD
    subgraph "HTTP Server (Axum)"
        direction TB
        ROUTER[Router<br/>Route Matching]
        EXTRACT[Extractors<br/>Request Parsing]
        HANDLER[Handlers<br/>Business Logic]
        RESPONSE[Response<br/>JSON/HTML]
    end
    
    subgraph "WebSocket Server"
        direction TB
        UPGRADE[WebSocket Upgrade<br/>HTTP → WS]
        AUTH_WS[Authentication<br/>Token Validation]
        MSG_PARSE[Message Parsing<br/>JSON → Struct]
        BROADCAST[Broadcasting<br/>Room-based]
    end
    
    subgraph "Static Assets"
        direction TB
        EMBED[rust-embed<br/>Compile-time Inclusion]
        SERVE[Asset Serving<br/>MIME Types + Caching]
        COMPRESS[Compression<br/>gzip + brotli]
    end
    
    ROUTER --> EXTRACT
    EXTRACT --> HANDLER
    HANDLER --> RESPONSE
    
    UPGRADE --> AUTH_WS
    AUTH_WS --> MSG_PARSE
    MSG_PARSE --> BROADCAST
    
    EMBED --> SERVE
    SERVE --> COMPRESS
    
    classDef http fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef websocket fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef assets fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class ROUTER,EXTRACT,HANDLER,RESPONSE http
    class UPGRADE,AUTH_WS,MSG_PARSE,BROADCAST websocket
    class EMBED,SERVE,COMPRESS assets
```

### Service Layer Architecture

```mermaid
graph TD
    subgraph "Authentication Service"
        direction TB
        LOGIN[Login<br/>Email + Password]
        SESSION[Session Management<br/>Token Generation]
        VALIDATE[Token Validation<br/>Middleware Integration]
        LOGOUT[Logout<br/>Session Cleanup]
    end
    
    subgraph "Message Service"
        direction TB
        CREATE[Message Creation<br/>Deduplication Logic]
        VALIDATE_MSG[Content Validation<br/>HTML Sanitization]
        PERSIST[Database Persistence<br/>Atomic Operations]
        BROADCAST_MSG[WebSocket Broadcasting<br/>Room Subscribers]
    end
    
    subgraph "Room Service"
        direction TB
        ROOM_CREATE[Room Creation<br/>Access Control Setup]
        MEMBERSHIP[Membership Management<br/>Join/Leave/Invite]
        ACCESS[Access Control<br/>Permission Checking]
        PRESENCE[Presence Tracking<br/>Online Users]
    end
    
    subgraph "Search Service"
        direction TB
        QUERY[Query Processing<br/>FTS5 Syntax]
        AUTH_SEARCH[Authorization<br/>Room Access Filter]
        RANK[Result Ranking<br/>Relevance Scoring]
        PAGINATE[Pagination<br/>Offset + Limit]
    end
    
    LOGIN --> SESSION
    SESSION --> VALIDATE
    VALIDATE --> LOGOUT
    
    CREATE --> VALIDATE_MSG
    VALIDATE_MSG --> PERSIST
    PERSIST --> BROADCAST_MSG
    
    ROOM_CREATE --> MEMBERSHIP
    MEMBERSHIP --> ACCESS
    ACCESS --> PRESENCE
    
    QUERY --> AUTH_SEARCH
    AUTH_SEARCH --> RANK
    RANK --> PAGINATE
    
    classDef auth fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef message fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef room fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef search fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class LOGIN,SESSION,VALIDATE,LOGOUT auth
    class CREATE,VALIDATE_MSG,PERSIST,BROADCAST_MSG message
    class ROOM_CREATE,MEMBERSHIP,ACCESS,PRESENCE room
    class QUERY,AUTH_SEARCH,RANK,PAGINATE search
```

### Data Layer Architecture

```mermaid
graph TD
    subgraph "SQLite Database"
        direction TB
        TABLES[Core Tables<br/>users, rooms, messages]
        INDEXES[Indexes<br/>Performance Optimization]
        WAL[WAL Mode<br/>Concurrent Reads]
        BACKUP[Backup Strategy<br/>Point-in-time Recovery]
    end
    
    subgraph "FTS5 Search Engine"
        direction TB
        VIRTUAL[Virtual Table<br/>messages_fts]
        TRIGGERS[Sync Triggers<br/>Auto-update Index]
        TOKENIZER[Tokenization<br/>Unicode + Stemming]
        RANKING[BM25 Ranking<br/>Relevance Scoring]
    end
    
    subgraph "In-Memory Cache"
        direction TB
        SESSIONS[Session Cache<br/>HashMap<Token, User>]
        PRESENCE_CACHE[Presence Cache<br/>HashMap<Room, Users>]
        CONNECTIONS[Connection Pool<br/>WebSocket Management]
        TTL[TTL Cleanup<br/>Background Task]
    end
    
    TABLES --> INDEXES
    INDEXES --> WAL
    WAL --> BACKUP
    
    VIRTUAL --> TRIGGERS
    TRIGGERS --> TOKENIZER
    TOKENIZER --> RANKING
    
    SESSIONS --> PRESENCE_CACHE
    PRESENCE_CACHE --> CONNECTIONS
    CONNECTIONS --> TTL
    
    %% Cross-connections
    TABLES -.-> VIRTUAL
    SESSIONS -.-> TABLES
    
    classDef database fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef search fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef cache fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class TABLES,INDEXES,WAL,BACKUP database
    class VIRTUAL,TRIGGERS,TOKENIZER,RANKING search
    class SESSIONS,PRESENCE_CACHE,CONNECTIONS,TTL cache
```

## Critical Design Patterns

### Message Deduplication Pattern (Critical Gap #1)

```mermaid
graph TD
    subgraph "Message Creation Flow"
        direction TB
        CLIENT[Client Sends Message<br/>with client_message_id]
        VALIDATE[Validate Content<br/>Length + HTML Sanitization]
        CHECK[Check Deduplication<br/>UNIQUE(client_message_id, room_id)]
        EXISTS{Message Exists?}
        RETURN_EXISTING[Return Existing Message<br/>No Database Write]
        CREATE_NEW[Create New Message<br/>Insert + Broadcast]
        BROADCAST[Broadcast to Room<br/>WebSocket Subscribers]
    end
    
    CLIENT --> VALIDATE
    VALIDATE --> CHECK
    CHECK --> EXISTS
    EXISTS -->|Yes| RETURN_EXISTING
    EXISTS -->|No| CREATE_NEW
    CREATE_NEW --> BROADCAST
    
    classDef process fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef decision fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef action fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class CLIENT,VALIDATE,CHECK,BROADCAST process
    class EXISTS decision
    class RETURN_EXISTING,CREATE_NEW action
```

### WebSocket Reconnection Pattern (Critical Gap #2)

```mermaid
graph TD
    subgraph "Reconnection Flow"
        direction TB
        CONNECT[WebSocket Connect<br/>with Authentication]
        REGISTER[Register Connection<br/>ConnectionManager]
        CHECK_LAST[Check last_seen_message_id<br/>from Client State]
        QUERY_MISSED[Query Missed Messages<br/>WHERE id > last_seen]
        SEND_MISSED[Send Missed Messages<br/>Batch Delivery]
        RESUME[Resume Normal Operation<br/>Real-time Messages]
    end
    
    subgraph "Presence Tracking"
        direction TB
        ADD_PRESENCE[Add to Presence<br/>HashMap<UserId, Count>]
        BROADCAST_JOIN[Broadcast User Joined<br/>to Room Members]
        TTL_CLEANUP[TTL Cleanup Task<br/>60-second Intervals]
    end
    
    CONNECT --> REGISTER
    REGISTER --> CHECK_LAST
    CHECK_LAST --> QUERY_MISSED
    QUERY_MISSED --> SEND_MISSED
    SEND_MISSED --> RESUME
    
    REGISTER --> ADD_PRESENCE
    ADD_PRESENCE --> BROADCAST_JOIN
    BROADCAST_JOIN --> TTL_CLEANUP
    
    classDef reconnect fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef presence fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    
    class CONNECT,REGISTER,CHECK_LAST,QUERY_MISSED,SEND_MISSED,RESUME reconnect
    class ADD_PRESENCE,BROADCAST_JOIN,TTL_CLEANUP presence
```

### SQLite Write Serialization Pattern (Critical Gap #3)

```mermaid
graph TD
    subgraph "Write Serialization"
        direction TB
        WRITE_REQ[Write Request<br/>from Service]
        CHANNEL[mpsc Channel<br/>Bounded Queue]
        WRITER_TASK[Single Writer Task<br/>Dedicated Thread]
        EXECUTE[Execute Write<br/>Atomic Transaction]
        RESPONSE[Response Channel<br/>oneshot Result]
    end
    
    subgraph "Read Operations"
        direction TB
        READ_REQ[Read Request<br/>from Service]
        POOL[Connection Pool<br/>Multiple Readers]
        CONCURRENT[Concurrent Reads<br/>WAL Mode]
    end
    
    WRITE_REQ --> CHANNEL
    CHANNEL --> WRITER_TASK
    WRITER_TASK --> EXECUTE
    EXECUTE --> RESPONSE
    
    READ_REQ --> POOL
    POOL --> CONCURRENT
    
    %% Show separation
    WRITER_TASK -.-> POOL
    
    classDef write fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    classDef read fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    
    class WRITE_REQ,CHANNEL,WRITER_TASK,EXECUTE,RESPONSE write
    class READ_REQ,POOL,CONCURRENT read
```

## Performance Characteristics

### Memory Usage Profile

```mermaid
graph TD
    subgraph "Memory Allocation (30-60MB Total)"
        direction TB
        BINARY[Binary Code<br/>~10MB]
        ASSETS[Embedded Assets<br/>~5MB]
        RUNTIME[Runtime Heap<br/>~15-45MB]
    end
    
    subgraph "Runtime Breakdown"
        direction TB
        CONNECTIONS[WebSocket Connections<br/>~100KB per 100 users]
        CACHE[In-Memory Cache<br/>~5-10MB]
        BUFFERS[I/O Buffers<br/>~2-5MB]
        STACK[Thread Stacks<br/>~8MB per thread]
    end
    
    BINARY --> RUNTIME
    ASSETS --> RUNTIME
    
    RUNTIME --> CONNECTIONS
    RUNTIME --> CACHE
    RUNTIME --> BUFFERS
    RUNTIME --> STACK
    
    classDef memory fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef breakdown fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    
    class BINARY,ASSETS,RUNTIME memory
    class CONNECTIONS,CACHE,BUFFERS,STACK breakdown
```

### Throughput Characteristics

```mermaid
graph TD
    subgraph "Request Processing"
        direction LR
        HTTP_REQ[HTTP Requests<br/>1K+ req/sec]
        WS_MSG[WebSocket Messages<br/>50 msg/sec system-wide]
        SEARCH_QRY[Search Queries<br/>Sub-millisecond FTS5]
    end
    
    subgraph "Database Operations"
        direction LR
        READS[Concurrent Reads<br/>No Limit (WAL)]
        WRITES[Serialized Writes<br/>Single Writer Queue]
        FTS_SEARCH[FTS5 Search<br/>~500μs per query]
    end
    
    subgraph "WebSocket Broadcasting"
        direction LR
        ROOM_CAST[Room Broadcasting<br/>100 users per room]
        PRESENCE[Presence Updates<br/>Real-time sync]
        TYPING[Typing Indicators<br/>10s timeout]
    end
    
    HTTP_REQ --> READS
    HTTP_REQ --> WRITES
    WS_MSG --> ROOM_CAST
    SEARCH_QRY --> FTS_SEARCH
    
    ROOM_CAST --> PRESENCE
    PRESENCE --> TYPING
    
    classDef request fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef database fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef websocket fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class HTTP_REQ,WS_MSG,SEARCH_QRY request
    class READS,WRITES,FTS_SEARCH database
    class ROOM_CAST,PRESENCE,TYPING websocket
```

## Security Architecture

### Authentication & Authorization Flow

```mermaid
graph TD
    subgraph "Authentication Flow"
        direction TB
        LOGIN_REQ[Login Request<br/>Email + Password]
        VALIDATE_CREDS[Validate Credentials<br/>bcrypt Hash Check]
        GEN_TOKEN[Generate Session Token<br/>Cryptographically Secure]
        STORE_SESSION[Store Session<br/>Database + Cache]
        RETURN_TOKEN[Return Token<br/>HTTP-only Cookie]
    end
    
    subgraph "Authorization Flow"
        direction TB
        REQUEST[Incoming Request<br/>with Session Token]
        EXTRACT_TOKEN[Extract Token<br/>Cookie or Header]
        VALIDATE_SESSION[Validate Session<br/>Cache → Database]
        CHECK_PERMS[Check Permissions<br/>Room Access Control]
        ALLOW_DENY{Allow or Deny}
    end
    
    LOGIN_REQ --> VALIDATE_CREDS
    VALIDATE_CREDS --> GEN_TOKEN
    GEN_TOKEN --> STORE_SESSION
    STORE_SESSION --> RETURN_TOKEN
    
    REQUEST --> EXTRACT_TOKEN
    EXTRACT_TOKEN --> VALIDATE_SESSION
    VALIDATE_SESSION --> CHECK_PERMS
    CHECK_PERMS --> ALLOW_DENY
    
    classDef auth fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef authz fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef decision fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    
    class LOGIN_REQ,VALIDATE_CREDS,GEN_TOKEN,STORE_SESSION,RETURN_TOKEN auth
    class REQUEST,EXTRACT_TOKEN,VALIDATE_SESSION,CHECK_PERMS authz
    class ALLOW_DENY decision
```

## Deployment Architecture

### Single Binary Deployment

```mermaid
graph TD
    subgraph "Build Process"
        direction TB
        SOURCE[Source Code<br/>Rust + Assets]
        COMPILE[Cargo Build<br/>--release]
        EMBED[Asset Embedding<br/>rust-embed]
        BINARY[Single Binary<br/>~15MB]
    end
    
    subgraph "Runtime Environment"
        direction TB
        CONTAINER[Docker Container<br/>Alpine Linux]
        PROCESS[Single Process<br/>Multi-threaded]
        DATABASE[SQLite File<br/>Local Storage]
        LOGS[Structured Logs<br/>JSON Output]
    end
    
    subgraph "External Dependencies"
        direction TB
        NONE[No External Services<br/>Self-contained]
        OPTIONAL[Optional Integrations<br/>Webhooks, Push]
    end
    
    SOURCE --> COMPILE
    COMPILE --> EMBED
    EMBED --> BINARY
    
    BINARY --> CONTAINER
    CONTAINER --> PROCESS
    PROCESS --> DATABASE
    PROCESS --> LOGS
    
    PROCESS -.-> OPTIONAL
    
    classDef build fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef runtime fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef external fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class SOURCE,COMPILE,EMBED,BINARY build
    class CONTAINER,PROCESS,DATABASE,LOGS runtime
    class NONE,OPTIONAL external
```

## Monitoring & Observability

### Health Check & Metrics

```mermaid
graph TD
    subgraph "Health Monitoring"
        direction TB
        HEALTH[/health Endpoint<br/>Basic Liveness]
        DB_CHECK[Database Connectivity<br/>Simple Query]
        READY[/ready Endpoint<br/>Readiness Check]
        STATUS[Service Status<br/>Component Health]
    end
    
    subgraph "Metrics Collection"
        direction TB
        PROMETHEUS[/metrics Endpoint<br/>Prometheus Format]
        REQUEST_METRICS[Request Metrics<br/>Count, Duration, Status]
        WS_METRICS[WebSocket Metrics<br/>Connections, Messages]
        SYSTEM_METRICS[System Metrics<br/>Memory, CPU, Disk]
    end
    
    subgraph "Logging"
        direction TB
        STRUCTURED[Structured Logging<br/>JSON Format]
        LEVELS[Log Levels<br/>ERROR, WARN, INFO, DEBUG]
        CONTEXT[Request Context<br/>Trace IDs]
        ROTATION[Log Rotation<br/>Size-based]
    end
    
    HEALTH --> DB_CHECK
    DB_CHECK --> READY
    READY --> STATUS
    
    PROMETHEUS --> REQUEST_METRICS
    REQUEST_METRICS --> WS_METRICS
    WS_METRICS --> SYSTEM_METRICS
    
    STRUCTURED --> LEVELS
    LEVELS --> CONTEXT
    CONTEXT --> ROTATION
    
    classDef health fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef metrics fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef logging fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class HEALTH,DB_CHECK,READY,STATUS health
    class PROMETHEUS,REQUEST_METRICS,WS_METRICS,SYSTEM_METRICS metrics
    class STRUCTURED,LEVELS,CONTEXT,ROTATION logging
```

## Technology Stack Summary

**Core Framework**: Axum (HTTP) + tokio-tungstenite (WebSocket) + SQLite (Database)
**Templates**: Askama (compile-time HTML)
**Authentication**: bcrypt + secure sessions
**Search**: SQLite FTS5
**Push Notifications**: web-push + VAPID
**Assets**: rust-embed (compile-time)
**Testing**: tokio-test + proptest
**Monitoring**: tracing + prometheus

This architecture provides a solid foundation for a production-ready chat application with Rails-equivalent functionality and significant performance improvements through Rust's type safety and zero-cost abstractions.