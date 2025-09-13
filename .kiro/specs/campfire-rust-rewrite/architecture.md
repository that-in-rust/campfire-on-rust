# Campfire Rust Rewrite - Architecture Document

## ⚠️ Critical Database Deployment Rule

**NEVER INCLUDE DATABASE FILES IN CONTAINER IMAGES**

### Why This Rule Exists:
- **Data Loss Risk**: Container updates/restarts can wipe database
- **No Persistence**: Accidental container deletion = complete data loss  
- **Backup Impossible**: Can't backup database independently
- **Scaling Issues**: Can't run multiple instances
- **Recovery Problems**: Must restore entire container for data recovery

### Correct Deployment Approach:
```dockerfile
# ✅ CORRECT: No database in image
FROM alpine:latest
COPY campfire-on-rust /usr/local/bin/campfire-on-rust
# Database will be in mounted volume or persistent filesystem
EXPOSE $PORT
CMD ["/usr/local/bin/campfire-on-rust"]
```

### Deployment Strategies by Platform:
- **Docker/VPS**: Use volume mounts (`-v campfire-data:/data`)
- **Railway/Render**: Use persistent filesystem (`/app/data/`)
- **AWS/GCP**: Use managed volumes (EFS/Persistent Disks)
- **Kubernetes**: Use PersistentVolumeClaims

---

## Project Structure Overview

### Simplified Directory Organization (~50 files)

```
campfire-on-rust/
├── 📁 Root Configuration
│   ├── Cargo.toml                    # Rust project configuration
│   ├── README.md                     # Project documentation
│   ├── .env.example                  # Environment variables template
│   └── docker-compose.yml            # Development environment
│
├── 🦀 src/ (Backend - 35 files)      # Rails-inspired Rust server
│   ├── main.rs                       # Application entry point
│   ├── lib.rs                        # Library exports
│   │
│   ├── 📁 models/ (5 files)          # Domain models
│   │   ├── message.rs                # Message with rich content
│   │   ├── room.rs                   # Room types (Open/Closed/Direct)
│   │   ├── user.rs                   # User authentication
│   │   ├── session.rs                # Session management
│   │   └── mod.rs                    # Model exports
│   │
│   ├── 📁 database/ (3 files)        # Direct SQLite operations
│   │   ├── connection.rs             # Connection pool
│   │   ├── migrations.rs             # Schema migrations
│   │   └── mod.rs                    # Database exports
│   │
│   ├── 📁 handlers/ (8 files)        # HTTP API endpoints
│   │   ├── messages.rs               # Message CRUD API
│   │   ├── rooms.rs                  # Room management
│   │   ├── users.rs                  # User management
│   │   ├── auth.rs                   # Authentication
│   │   ├── websocket.rs              # WebSocket upgrade
│   │   ├── health.rs                 # Health checks
│   │   ├── assets.rs                 # Static assets
│   │   └── mod.rs                    # Handler exports
│   │
│   ├── 📁 websocket/ (2 files)       # ActionCable-style broadcasting
│   │   ├── broadcaster.rs            # Room-based broadcasting
│   │   └── mod.rs                    # WebSocket exports
│   │
│   ├── 📁 services/ (6 files)        # Business logic (Rails-style)
│   │   ├── message_service.rs        # Message processing
│   │   ├── room_service.rs           # Room management
│   │   ├── auth_service.rs           # Authentication logic
│   │   ├── notification_service.rs   # Push notifications
│   │   ├── webhook_service.rs        # Bot webhooks
│   │   └── mod.rs                    # Service exports
│   │
│   ├── 📁 middleware/ (5 files)      # HTTP middleware
│   │   ├── auth.rs                   # Authentication
│   │   ├── cors.rs                   # CORS headers
│   │   ├── logging.rs                # Request logging
│   │   ├── rate_limit.rs             # Rate limiting
│   │   └── mod.rs                    # Middleware exports
│   │
│   ├── 📁 assets/ (3 files)          # Asset embedding
│   │   ├── embedded.rs               # Rust-embed integration
│   │   ├── sounds.rs                 # Sound command handling
│   │   └── mod.rs                    # Asset exports
│   │
│   └── 📁 utils/ (3 files)           # Utilities
│       ├── validation.rs             # Input validation
│       ├── config.rs                 # Configuration
│       └── mod.rs                    # Utility exports
│
├── ⚛️ frontend/ (React - 15 files)   # Simple React frontend
│   ├── package.json                  # Dependencies (simplified)
│   ├── vite.config.ts                # Build configuration
│   ├── index.html                    # Entry point
│   │
│   └── 📁 src/
│       ├── main.tsx                  # React entry point
│       ├── App.tsx                   # Root component
│       │
│       ├── 📁 components/ (8 files)  # UI components
│       │   ├── MessageList.tsx       # Message display
│       │   ├── MessageComposer.tsx   # Message input
│       │   ├── RoomList.tsx          # Room navigation
│       │   ├── UserList.tsx          # Member list
│       │   ├── LoginForm.tsx         # Authentication
│       │   ├── Layout.tsx            # App layout
│       │   ├── ErrorBoundary.tsx     # Error handling
│       │   └── LoadingSpinner.tsx    # Loading states
│       │
│       ├── 📁 hooks/ (3 files)       # Custom hooks
│       │   ├── useWebSocket.ts       # WebSocket connection
│       │   ├── useAuth.ts            # Authentication state
│       │   └── useMessages.ts        # Message state
│       │
│       ├── 📁 services/ (2 files)    # API services
│       │   ├── api.ts                # HTTP client
│       │   └── websocket.ts          # WebSocket service
│       │
│       └── 📁 types/ (2 files)       # TypeScript types
│           ├── api.ts                # API types
│           └── models.ts             # Domain types
│
├── 🎨 assets/ (164 files)            # Original Campfire assets
│   ├── 📁 images/ (79 SVG files)     # Complete UI icons
│   ├── 📁 sounds/ (59 MP3 files)     # /play command sounds
│   └── 📁 stylesheets/ (26 CSS)      # Complete styling
│
├── 🗄️ migrations/ (4 files)          # Database schema
│   ├── 001_initial_schema.sql        # Core tables
│   ├── 002_add_fts_search.sql        # Full-text search
│   ├── 003_add_sessions.sql          # Session management
│   └── 004_add_webhooks.sql          # Bot integration
│
├── 🧪 tests/ (10 files)              # Test suite
│   ├── 📁 unit/ (5 files)            # Unit tests
│   ├── 📁 integration/ (3 files)     # Integration tests
│   └── 📁 fixtures/ (2 files)        # Test data
│
└── 🐳 docker/ (2 files)              # Deployment
    ├── Dockerfile                    # Production container
    └── docker-compose.yml            # Development setup
```

### Key Architectural Decisions

#### **Simplification Strategy**
- **75% File Reduction**: 50 files vs 200+ in coordination approach
- **No Coordination Layer**: Direct operations instead of complex coordination
- **Rails-Inspired Patterns**: Proven ActionCable and ActiveRecord equivalents
- **Linear Dependencies**: Simple dependency chain instead of coordination web

#### **Rails Compatibility Focus**
- **ActionCable Broadcasting**: Room-based WebSocket channels
- **Service Objects**: Rails-style business logic organization
- **Direct Database Operations**: ActiveRecord-equivalent queries
- **Middleware Stack**: Rails-style request processing

---

## Architecture Overview: Rails-Inspired Pragmatic MVP 🎯

### Philosophy
Build a simple, working chat application that replicates Rails ActionCable behavior using idiomatic Rust patterns. Focus on proven Rails patterns rather than theoretical coordination improvements.

### Core Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                Single Rust Binary (~25MB)                   │
├─────────────────────────────────────────────────────────────┤
│  Complete React UI (Rails Parity)                          │
│  ├─── All Components (File Upload, Lightbox, Avatars)      │
│  ├─── Complete CSS/Styling (26 stylesheets)                │
│  ├─── Sound Assets (59 MP3 files)                          │
│  ├─── Graceful Degradation (Disabled Features)             │
│  ├─── Simple State Management (No Cross-Tab Coordination)  │
│  └─── Service Worker (PWA, Push Notifications)             │
├─────────────────────────────────────────────────────────────┤
│  Axum Web Server (HTTP + WebSocket)                        │
│  ├─── REST API Handlers (Rails-style routing)             │
│  ├─── ActionCable-Inspired WebSocket Broadcasting          │
│  ├─── Rails-Style Session Authentication                   │
│  └─── Basic Security Middleware                            │
├─────────────────────────────────────────────────────────────┤
│  Simple Real-time Layer                                    │
│  ├─── Direct Message Broadcasting (No Global Coordination) │
│  ├─── Basic Presence Tracking (Rails-style)               │
│  ├─── Simple Typing Notifications                          │
│  └─── Feature Flag Support (Static Configuration)          │
├─────────────────────────────────────────────────────────────┤
│  Basic Task Processing                                      │
│  ├─── Async Webhook Delivery                               │
│  ├─── Push Notification Sending                            │
│  └─── Simple Background Tasks                              │
├─────────────────────────────────────────────────────────────┤
│  Direct SQLite Operations (10-200MB)                       │
│  ├─── Write-Ahead Logging (WAL) Mode                      │
│  ├─── Direct Database Queries (No Coordination Layer)      │
│  ├─── FTS5 Search Index (Simple Updates)                  │
│  ├─── Basic Connection Pooling                             │
│  └─── Rails-Compatible Schema                              │
└─────────────────────────────────────────────────────────────┘
```

### Technical Stack
- **Web Framework**: Axum (Rails-inspired routing and middleware)
- **Database**: SQLite (direct operations, Rails-compatible schema)
- **Real-time**: ActionCable-inspired WebSocket broadcasting
- **Frontend**: Complete React UI (simple state management)
- **Task Queue**: Basic tokio tasks (webhook delivery, push notifications)
- **Authentication**: Rails-style session management
- **Deployment**: Single binary with embedded assets

---

## Simple Rails-Inspired Patterns

**Based on Strategic Pivot**: The architecture has been simplified to use proven Rails patterns rather than complex coordination mechanisms.

### 1. Simple Message Flow
```
Client                    API Handler              Database                WebSocket
  │                        │                        │                       │
  ├─ Send Message ────────▶│                        │                       │
  │  (basic HTTP POST)     │                        │                       │
  │                        │                        │                       │
  ├─ Optimistic UI ────────┤                        │                       │
  │  (show pending)        │                        │                       │
  │                        │                        │                       │
  │                        ├─ Insert Message ─────▶│                       │
  │                        │  (simple SQL INSERT)   │                       │
  │                        │                        │                       │
  │                        ├─ Broadcast Message ───────────────────────────▶│
  │                        │  (ActionCable-style)   │                       │
  │                        │                        │                       │
  ├─ Receive Broadcast ◀───────────────────────────────────────────────────┤
  │  (WebSocket message)   │                        │                       │
```

### 2. Rails-Style State Management
- **Direct Database Operations**: Simple SQL queries, no coordination layer
- **ActionCable-Style Broadcasting**: Room-based WebSocket channels like Rails
- **Basic Presence Tracking**: Simple connection counting without complex coordination
- **Simple Session Management**: Rails-style session cookies and authentication
- **Straightforward Error Handling**: Basic error responses, no complex recovery

### 3. Database Patterns
- **Direct SQLite Operations**: No coordination layer, direct SQL queries
- **WAL Mode**: Simple write-ahead logging for basic concurrency
- **FTS5 Search**: Direct search queries, no async coordination
- **Connection Pooling**: Basic SQLite connection pool

### 4. Real-time Architecture
- **Room Channels**: ActionCable-inspired room-based broadcasting
- **Simple Presence**: Basic online/offline tracking
- **Typing Notifications**: Simple start/stop notifications
- **Message Broadcasting**: Direct WebSocket sends to room subscribers

### 5. Basic Reliability Patterns
- **Simple Retry**: Basic retry logic for failed operations
- **Error Logging**: Log errors for debugging, no complex recovery
- **Health Checks**: Basic /health endpoint
- **Graceful Shutdown**: Clean server shutdown handling

---

## Feature Scope

### ✅ **Fully Implemented (Complete UX):**
- Complete React UI with all components
- Rich text messaging with Trix editor
- Real-time chat with full presence system
- @mentions with autocomplete
- Sound commands with embedded audio
- Unicode emoji support
- Complete room management UI
- User presence and typing indicators
- Full search functionality
- Bot integration (text responses)
- PWA support and push notifications
- Multi-device session management with QR codes

### 🚧 **Gracefully Disabled (UI Present, Backend Stubbed):**
- File upload zones (show "Coming in v2.0" message)
- Avatar upload areas (text initials with placeholder)
- Image lightbox (ready for images, shows upgrade prompt)
- Document sharing (upload UI present but disabled)
- OpenGraph previews (links shown as text with "Preview coming soon")

---

## Data Volume Analysis

### Small Team (25 users)
```
Users: 25 × 0.5KB = 12.5KB
Messages: 10,000 × 0.8KB = 8MB
Rich Text: 2,000 × 1KB = 2MB
FTS5 Index: ~2.5MB
Total Database: ~12.5MB
Total Storage: ~12.5MB (no files!)
```

### Large Team (500 users)
```
Users: 500 × 0.5KB = 250KB
Messages: 250,000 × 0.8KB = 200MB
Rich Text: 50,000 × 1KB = 50MB
FTS5 Index: ~62.5MB
Total Database: ~314MB
Total Storage: ~314MB
```

---

## Deployment Architecture

### Container Image
```dockerfile
# Complete UI Container (No Database!)
FROM alpine:latest
RUN apk add --no-cache ca-certificates curl
COPY campfire-on-rust /usr/local/bin/campfire-on-rust
EXPOSE $PORT
CMD ["/usr/local/bin/campfire-on-rust"]
```

### Feature Flag Configuration
```rust
// Configuration with feature flags
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub files_enabled: bool,        // v1.0: false
    pub avatars_enabled: bool,      // v1.0: false
    pub opengraph_enabled: bool,    // v1.0: false
    pub max_file_size: usize,       // v1.0: 0
}
```

### Environment Variables
```bash
# Core Configuration
DATABASE_PATH=/app/data/campfire.db
SECRET_KEY_BASE=your-secret-key
VAPID_PUBLIC_KEY=your-vapid-public-key
VAPID_PRIVATE_KEY=your-vapid-private-key

# Feature Flags (MVP Phase 1)
FILES_ENABLED=false
AVATARS_ENABLED=false
OPENGRAPH_ENABLED=false

# Optional Configuration
SSL_DOMAIN=your-domain.com
DISABLE_SSL=false
SENTRY_DSN=your-sentry-dsn
```

---

## Performance Targets

### Simplified MVP Targets (Rails-Inspired)
- **Memory**: 20-40MB total (simple operations, no coordination overhead)
- **Connections**: 200+ concurrent WebSocket (realistic for simple broadcasting)
- **Startup**: <100ms cold start (simple initialization, embedded assets)
- **Throughput**: 2K+ req/sec sustainable (direct operations, no coordination bottleneck)
- **Storage**: 10MB-300MB (text-only messages, simple schema)
- **Cost Reduction**: 85-90% vs Rails (Rust efficiency without coordination complexity)

### Response Time Targets (Simple Operations)
- **API Calls**: <10ms (direct database operations, simple handlers)
- **Message Operations**: <50ms (direct insert + broadcast, optimistic UI)
- **Static Assets**: <1ms (embedded assets, efficient serving)
- **WebSocket Messages**: <5ms routing (direct broadcasting to room subscribers)
- **Database Queries**: <5ms (direct SQLite operations, no coordination)

### Reliability Targets (Pragmatic)
- **Availability**: 99% uptime (87.6 hours downtime per year, realistic for simple system)
- **Message Delivery**: 99% success rate (simple retry logic, basic error handling)
- **Data Consistency**: 95% (eventual consistency, Rails-level reliability)
- **Recovery Time**: <10 seconds for simple reconnection
- **State Sync**: <2 seconds for WebSocket reconnection

### Scalability Limits (Simple Architecture)
- **Single Room**: 50 concurrent users (realistic for simple broadcasting)
- **Total Rooms**: 25 active rooms (memory and processing realistic limits)
- **Message Rate**: 100 messages/second system-wide (direct operations)
- **Database Size**: 500MB maximum for MVP (text-only content)
- **Asset Memory**: 50MB for embedded assets (all sounds, images, CSS)

**Note**: These targets reflect Rails-equivalent performance with Rust efficiency gains. Focus on "good enough" reliability rather than theoretical perfection.

---

## Feature Flag Architecture

### Configuration-Driven Feature Control
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub files_enabled: bool,        // MVP: false
    pub avatars_enabled: bool,      // MVP: false  
    pub opengraph_enabled: bool,    // MVP: false
    pub max_file_size: usize,       // MVP: 0
    pub search_enabled: bool,       // MVP: true
    pub push_notifications: bool,   // MVP: true
    pub bot_integrations: bool,     // MVP: true
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            files_enabled: false,
            avatars_enabled: false,
            opengraph_enabled: false,
            max_file_size: 0,
            search_enabled: true,
            push_notifications: true,
            bot_integrations: true,
        }
    }
}
```

### Graceful Feature Degradation
- **File Upload Areas**: Complete UI with "Available in v2.0" messaging
- **Avatar Upload**: Text initials with professional styling + upgrade prompt
- **Image Lightbox**: Full component with "Image viewing coming soon" message
- **Document Sharing**: Upload zones present but gracefully disabled
- **OpenGraph Previews**: Links shown as text with "Preview coming soon"

---

## Key Benefits

### Immediate Benefits (Phase 1)
- **Complete user experience**: Full Rails UI from day one
- **Ultra-low costs**: 90-95% cost reduction (same as text-only)
- **Zero redesign risk**: Complete interface built once
- **Professional appearance**: Looks like finished product
- **Perfect user expectation management**: Clear messaging about features
- **Stakeholder ready**: Demo complete vision while controlling costs

### Long-term Benefits
- **Optimal evolution path**: Feature flags enable gradual rollout
- **Risk mitigation**: Validate core functionality before adding complexity
- **Cost control**: Add features only when needed and budget allows
- **User feedback**: Collect feature requests and prioritize development
- **Technical validation**: Prove architecture before scaling complexity

---

## Trade-offs

### Acceptable Trade-offs
- **Slightly larger binary**: 30MB vs 25MB (includes complete UI)
- **User expectation management**: Need clear messaging about disabled features
- **Support questions**: Users will ask about disabled features
- **Temporary workarounds**: External file sharing needed initially

### Mitigated Risks
- **UI Redesign Risk**: ✅ Eliminated (complete UI built)
- **User Experience Risk**: ✅ Mitigated (professional appearance)
- **Cost Risk**: ✅ Minimized (90-95% reduction achieved)
- **Technical Risk**: ✅ Reduced (gradual complexity increase)
- **Business Risk**: ✅ Controlled (validate before investing)

---

## Implementation Priorities

### High Priority (Phase 1)
1. **Complete React UI**: All components with graceful degradation
2. **Rich Text Messaging**: Full Trix editor integration
3. **Real-time Features**: WebSocket, presence, typing indicators
4. **Authentication**: Session management, security
5. **Search**: FTS5 full-text search implementation
6. **PWA Support**: Service worker, push notifications
7. **Bot Integration**: Text-only webhook system

### Medium Priority (Phase 2-3)
1. **Avatar System**: Image upload and processing
2. **File Storage**: Basic blob storage implementation
3. **Document Sharing**: File attachment system
4. **Enhanced Security**: File validation and scanning

### Lower Priority (Phase 4+)
1. **Image Processing**: VIPS integration, thumbnails
2. **Video Support**: Video processing and streaming
3. **OpenGraph**: Link preview system with SSRF protection
4. **Advanced Features**: Lightbox, advanced file management

---

## Success Metrics

### Phase 1 Success Criteria
- **Cost Reduction**: 90-95% achieved
- **Memory Usage**: 10-30MB sustained
- **User Satisfaction**: >90% positive feedback on UI
- **Performance**: All response time targets met
- **Reliability**: >99.9% uptime
- **Feature Messaging**: Clear understanding of roadmap

### Technical Success Metrics
- **Startup Time**: <50ms consistently
- **WebSocket Connections**: 10,000+ concurrent
- **Message Throughput**: 15K+ req/sec
- **Database Performance**: <2ms query times
- **Search Performance**: Sub-millisecond FTS5 queries

### Business Success Metrics
- **Hosting Costs**: $3-5/month for small teams
- **User Adoption**: Smooth transition from Rails
- **Feature Requests**: Clear prioritization data
- **Stakeholder Satisfaction**: Professional demo capability
- **Development Velocity**: Fast iteration on core features

---

## Conclusion

This **coordination-first architecture** provides the optimal balance of **reliability**, **complete user experience**, and **significant cost reduction**. By addressing the 47 critical coordination gaps identified in the cynical analysis, we achieve:

1. **Production-ready reliability** through comprehensive coordination mechanisms
2. **Professional appearance** with complete UI and graceful feature degradation
3. **85-90% cost reduction** (realistic with coordination overhead)
4. **Proven coordination patterns** that work under real-world failure conditions
5. **Clear evolution path** with battle-tested coordination for future features

**Key Insight**: The original analysis revealed that the challenge is not implementing individual features, but ensuring they work together reliably. This architecture prioritizes **coordination over raw performance**, resulting in a system that actually works in production rather than just in demos.

**Trade-offs Accepted**:
- Lower raw performance (1K vs 15K req/sec) for higher reliability
- Higher memory usage (30-60MB vs 10-30MB) for coordination overhead
- More complex implementation for production-grade fault tolerance

The approach eliminates the common MVP problem of "works in demo but fails in production" while maintaining significant cost benefits over the Rails implementation. Users get a reliable, professional chat experience that continues working under real-world conditions including network issues, concurrent usage, and partial failures.

---

## Implementation Phases

### Phase 1: Simple Monolith (Weeks 1-4)
**Goal**: Working chat app with basic features

**Key Files to Implement**:
- `src/models/{message,room,user}.rs` - Basic domain models
- `src/database/connection.rs` - Direct SQLite operations
- `src/handlers/{messages,rooms,auth}.rs` - Basic API endpoints
- `src/websocket/broadcaster.rs` - Simple room broadcasting
- `frontend/src/components/MessageList.tsx` - Basic message display

**Success Criteria**: 5 users can chat in real-time without complex coordination

### Phase 2: Rails Pattern Study (Weeks 5-6)
**Goal**: Understand what coordination Rails actually uses

**Method**: Deep dive into ActionCable implementation, identify minimal necessary patterns
**Output**: Evidence-based list of required coordination patterns

### Phase 3: Targeted Rails Compatibility (Weeks 7-10)
**Goal**: Add only coordination patterns Rails proves necessary

**Key Files to Enhance**:
- `src/services/` - Add Rails-style service objects
- `src/middleware/` - Add Rails-equivalent middleware
- Enhanced WebSocket broadcasting to match ActionCable behavior
- Simple presence tracking and typing notifications

**Success Criteria**: Behavior matches Rails ActionCable in real-world scenarios

### Phase 4: Production Polish (Weeks 11-12)
**Goal**: Production-ready deployment with monitoring

**Key Additions**:
- Health checks and monitoring
- Error logging and debugging
- Performance optimization
- Docker deployment configuration

**Success Criteria**: Stable deployment handling real user load

### Asset Integration Strategy

**Complete Compatibility**: All 164 original Campfire assets preserved:

- **Sound System**: 59 MP3 files enable complete `/play` command functionality
- **Icon System**: 79 SVG icons provide complete UI compatibility  
- **Style System**: 26 CSS files maintain exact visual appearance
- **Embedded Serving**: All assets embedded in binary for single-file deployment

### Testing Strategy

**Simple, Effective Testing**: Focus on practical testing that ensures reliability:

- **Unit Tests**: Test individual components and services
- **Integration Tests**: Test API endpoints and WebSocket functionality
- **End-to-End Tests**: Test complete user workflows
- **Rails Compatibility Tests**: Verify behavior matches Rails ActionCable

This structure prioritizes practical success over theoretical perfection, using Rails as the proven blueprint for what coordination is actually necessary.

---

## Operational Monitoring and Observability

### Health Check Endpoints
- **`/health`**: Basic service health (database, WebSocket, memory usage)
- **`/health/detailed`**: Comprehensive health including circuit breaker states, queue sizes, retry counts
- **`/metrics`**: Prometheus metrics for monitoring and alerting

### Key Metrics to Monitor
- **Message Processing**: Success rate, retry count, queue depth, processing latency
- **WebSocket Connections**: Active connections, reconnection rate, heartbeat failures
- **Database Performance**: Query latency, transaction rollback rate, connection pool usage
- **Circuit Breaker States**: Open/closed status, failure rates, recovery attempts
- **Memory Usage**: Total memory, retry queue size, fallback storage usage

### Alerting Thresholds
- **Message Failure Rate**: >1% (indicates system issues)
- **WebSocket Reconnection Rate**: >10% (network or server issues)
- **Database Query Latency**: >50ms average (performance degradation)
- **Circuit Breaker Open**: Any circuit open for >5 minutes
- **Memory Usage**: >80% of allocated memory

### Fault Tolerance Validation
- **Recovery Time**: <30 seconds for component failures
- **Data Consistency**: 100% (atomic transactions prevent corruption)
- **Message Delivery**: 99.99% success rate (with retry mechanisms)
- **State Synchronization**: <5 seconds for WebSocket reconnection
- **Availability**: 99.9% uptime target

**This fault-tolerant architecture can now confidently deliver the professional chat experience specified in the requirements while maintaining the 90-95% cost reduction goal and providing production-grade reliability.**