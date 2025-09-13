# Campfire Rust Rewrite - Architecture Options Analysis

## Overview

This document presents three distinct high-level architecture approaches for the Campfire Rust rewrite, each balancing different priorities while meeting the core requirements for 87% cost reduction, <2MB memory usage, and 100% feature parity with the Rails implementation.

## Requirements Context

Based on the comprehensive requirements analysis, the key architectural drivers are:

- **Performance**: <2MB memory, 10K+ WebSocket connections, <100ms startup
- **Cost Efficiency**: 87% cost reduction (2 vCPU/4GB → 0.25 vCPU/0.5GB)
- **Feature Parity**: 28 detailed requirements covering all Rails functionality
- **Deployment**: Single-binary with embedded assets
- **Database**: SQLite with FTS5, 12 tables, complex relationships
- **Real-time**: WebSocket-based with presence, typing, broadcasting
- **Security**: Session auth, rate limiting, content sanitization

---

## Architecture Option 1: "Monolithic Efficiency" ⭐ RECOMMENDED

### Philosophy
Single-binary deployment with embedded components, optimized for the 87% cost reduction goal and <2MB memory usage.

### Core Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                    Single Rust Binary                       │
├─────────────────────────────────────────────────────────────┤
│  Embedded React SPA (include_bytes!)                       │
│  ├─── Static Assets (CSS, JS, Images, Sounds)              │
│  └─── Service Worker (PWA, Push Notifications)             │
├─────────────────────────────────────────────────────────────┤
│  Axum Web Server (HTTP + WebSocket)                        │
│  ├─── REST API Handlers                                    │
│  ├─── WebSocket Connection Manager                         │
│  ├─── Session-based Authentication                         │
│  └─── Rate Limiting & Security Middleware                  │
├─────────────────────────────────────────────────────────────┤
│  Actor-based Real-time Engine                              │
│  ├─── Room Actors (State Management)                       │
│  ├─── Presence Tracking                                    │
│  ├─── Message Broadcasting                                 │
│  └─── Typing Notifications                                 │
├─────────────────────────────────────────────────────────────┤
│  Embedded Task Queue (Tokio Tasks)                         │
│  ├─── Webhook Delivery                                     │
│  ├─── Push Notification Sending                           │
│  ├─── File Processing (VIPS)                              │
│  └─── Background Cleanup                                   │
├─────────────────────────────────────────────────────────────┤
│  SQLite Database (WAL Mode)                                │
│  ├─── Connection Pool                                      │
│  ├─── FTS5 Search Index                                   │
│  ├─── Prepared Statements                                 │
│  └─── Migration System                                     │
└─────────────────────────────────────────────────────────────┘
```

### Technical Stack
- **Web Framework**: Axum (hyper-based, async)
- **Database**: SQLite with sqlx, WAL mode, connection pooling
- **Real-time**: Actor pattern with tokio channels
- **Frontend**: Embedded React SPA with Vite build
- **Task Queue**: Tokio spawn tasks (no Redis dependency)
- **File Processing**: libvips-rs with async spawn_blocking
- **Authentication**: Session-based with secure tokens
- **Deployment**: Single binary with embedded assets

### Key Benefits
- **Ultra-low resource usage**: <2MB memory, single process
- **Fastest startup**: <100ms cold start with embedded assets
- **Simplest deployment**: Single binary + SQLite file + volume mount
- **Maximum performance**: 10K+ WebSocket connections, 10-12K req/sec
- **Cost optimization**: Directly achieves 87% cost reduction goal
- **Rails parity**: Closest architectural match to current monolith
- **Zero external dependencies**: No Redis, no separate services

### Trade-offs
- **Horizontal scaling limitations**: Single SQLite instance constraint
- **Component coupling**: All components in single process
- **All-or-nothing deployment**: Cannot deploy components independently
- **Memory sharing**: All features share same memory space

### Performance Targets
- Memory: <2MB baseline (vs Rails 50-100MB)
- Connections: 10,000+ concurrent WebSocket (vs Rails ~1,000)
- Startup: <100ms cold start (vs Rails several seconds)
- Throughput: 10-12K req/sec (vs Rails few hundred per core)
- Response times: <5ms API, <10ms messages, <1ms static assets

---

## Architecture Option 2: "Microservices Scalability"

### Philosophy
Distributed architecture with separate services for different concerns, optimized for horizontal scaling and team development.

### Core Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                    Load Balancer                            │
└─────────────────────┬───────────────────────────────────────┘
                      │
    ┌─────────────────┼─────────────────┐
    │                 │                 │
┌───▼────┐    ┌──────▼──────┐    ┌─────▼─────┐
│Frontend│    │   API       │    │WebSocket  │
│Service │    │ Gateway     │    │Service    │
│(Axum)  │    │  (Axum)     │    │ (Axum)    │
└────────┘    └─────────────┘    └───────────┘
                      │
        ┌─────────────┼─────────────┐
        │             │             │
   ┌────▼───┐   ┌────▼────┐   ┌───▼────┐
   │Message │   │  User   │   │  Room  │
   │Service │   │ Service │   │Service │
   │        │   │         │   │        │
   └────────┘   └─────────┘   └────────┘
        │             │             │
        └─────────────┼─────────────┘
                      │
        ┌─────────────▼─────────────┐
        │     Message Queue         │
        │      (Redis/NATS)         │
        └───────────────────────────┘
                      │
              ┌───────▼────────┐
              │  Shared SQLite │
              │   (Network)    │
              └────────────────┘
```

### Service Breakdown
- **Frontend Service**: Static asset serving, React SPA
- **API Gateway**: Request routing, authentication, rate limiting
- **WebSocket Service**: Real-time connections, presence, broadcasting
- **Message Service**: Message CRUD, rich content, search
- **User Service**: Authentication, sessions, bot management
- **Room Service**: Room management, memberships, access control

### Technical Stack
- **Service Framework**: Axum for each service
- **Service Discovery**: Consul or embedded DNS
- **Inter-service Communication**: gRPC or HTTP/JSON
- **Message Queue**: Redis Streams or NATS
- **Database**: Shared SQLite or per-service databases
- **Load Balancing**: HAProxy or cloud load balancer
- **Orchestration**: Docker Compose or Kubernetes

### Key Benefits
- **Independent scaling**: Scale services based on specific load patterns
- **Team autonomy**: Different teams can own and deploy services independently
- **Technology flexibility**: Could use different databases per service
- **Fault isolation**: Service failures don't bring down entire system
- **Development parallelization**: Teams can work on services simultaneously
- **Deployment flexibility**: Rolling updates, canary deployments per service

### Trade-offs
- **Higher complexity**: Service discovery, inter-service communication overhead
- **More resource usage**: Multiple processes, network latency, serialization
- **Deployment complexity**: Orchestration, service mesh, monitoring required
- **Cost implications**: May not meet 87% cost reduction goal due to overhead
- **Data consistency**: Distributed transactions, eventual consistency challenges
- **Debugging difficulty**: Distributed tracing, log aggregation required

### Performance Implications
- **Memory**: 5-10MB per service (30-60MB total)
- **Network overhead**: Inter-service communication latency
- **Startup time**: Service dependency chains increase startup time
- **Operational complexity**: Multiple deployment units to manage

---

## Architecture Option 3: "Hybrid Modular Monolith"

### Philosophy
Modular monolith with clear internal boundaries and optional service extraction, balancing simplicity with scalability.

### Core Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                 Campfire Rust Binary                        │
├─────────────────────────────────────────────────────────────┤
│  Frontend Module (Embedded React)                          │
│  └─── Asset Serving + PWA Support                          │
├─────────────────────────────────────────────────────────────┤
│  Web Layer (Axum)                                          │
│  ├─── HTTP Handlers                                        │
│  ├─── WebSocket Manager                                    │
│  └─── Middleware Stack                                     │
├─────────────────────────────────────────────────────────────┤
│  Domain Modules (Clear Boundaries)                         │
│  ├─── Messages Module                                      │
│  │    ├─── Message Service                                │
│  │    ├─── Rich Content Processing                        │
│  │    └─── Search Integration                             │
│  ├─── Rooms Module                                         │
│  │    ├─── Room Service                                   │
│  │    ├─── Membership Management                          │
│  │    └─── Access Control                                 │
│  ├─── Users Module                                         │
│  │    ├─── Authentication Service                         │
│  │    ├─── Session Management                             │
│  │    └─── Bot Integration                                │
│  └─── Real-time Module                                     │
│       ├─── Connection Manager                              │
│       ├─── Presence Tracking                              │
│       └─── Event Broadcasting                             │
├─────────────────────────────────────────────────────────────┤
│  Infrastructure Layer                                       │
│  ├─── Database Access (SQLite + Pool)                     │
│  ├─── Task Queue (Tokio)                                  │
│  ├─── File Storage                                        │
│  └─── External APIs (Webhooks, Push)                      │
└─────────────────────────────────────────────────────────────┘
```

### Module Design Principles
- **Clear interfaces**: Each module exposes well-defined APIs
- **Dependency inversion**: Modules depend on abstractions, not implementations
- **Single responsibility**: Each module has one primary concern
- **Loose coupling**: Minimal dependencies between modules
- **High cohesion**: Related functionality grouped together

### Technical Implementation
- **Module boundaries**: Rust modules with public interfaces
- **Dependency injection**: Service traits and implementations
- **Event system**: Internal event bus for module communication
- **Shared types**: Common domain types across modules
- **Testing isolation**: Each module can be tested independently

### Key Benefits
- **Clear boundaries**: Well-defined modules with explicit interfaces
- **Extraction ready**: Modules can become services later if needed
- **Development efficiency**: Single codebase, shared types, unified testing
- **Performance**: In-process communication, shared memory, no serialization
- **Meets cost goals**: Single deployment with efficiency benefits
- **Refactoring safety**: Module boundaries prevent accidental coupling
- **Team scaling**: Teams can own modules with clear responsibilities

### Trade-offs
- **Module discipline required**: Need to enforce boundaries through code review
- **Potential coupling**: Risk of tight coupling if boundaries not maintained
- **Single point of failure**: Still a monolith at runtime
- **Shared database**: All modules share same database instance
- **Deployment coupling**: All modules deploy together

### Evolution Path
1. **Phase 1**: Start as modular monolith with clear boundaries
2. **Phase 2**: Extract high-load modules (e.g., WebSocket service)
3. **Phase 3**: Extract domain modules as needed for scaling
4. **Phase 4**: Full microservices if business requirements demand

---

## Comparative Analysis

### Performance Requirements Alignment

| Requirement | Option 1 (Monolith) | Option 2 (Microservices) | Option 3 (Modular) |
|-------------|---------------------|---------------------------|---------------------|
| <2MB Memory | ✅ Excellent (1-2MB) | ❌ Poor (30-60MB) | ✅ Good (2-5MB) |
| 10K+ WebSocket | ✅ Excellent | ⚠️ Complex (service mesh) | ✅ Excellent |
| <100ms Startup | ✅ Excellent | ❌ Poor (service deps) | ✅ Good |
| 87% Cost Reduction | ✅ Excellent | ❌ Poor (overhead) | ✅ Good |
| Single Binary Deploy | ✅ Perfect | ❌ N/A | ✅ Perfect |
| 10-12K req/sec | ✅ Excellent | ⚠️ Network overhead | ✅ Excellent |

### Development & Maintenance

| Aspect | Option 1 | Option 2 | Option 3 |
|--------|----------|----------|----------|
| Initial Development Speed | ✅ Fast | ❌ Slow | ✅ Medium |
| Team Scaling | ⚠️ Limited | ✅ Excellent | ✅ Good |
| Debugging Complexity | ✅ Simple | ❌ Complex | ✅ Good |
| Testing Complexity | ✅ Simple | ❌ Complex | ✅ Good |
| Deployment Complexity | ✅ Simple | ❌ Complex | ✅ Simple |
| Operational Overhead | ✅ Minimal | ❌ High | ✅ Low |

### Scalability & Evolution

| Aspect | Option 1 | Option 2 | Option 3 |
|--------|----------|----------|----------|
| Horizontal Scaling | ⚠️ Limited | ✅ Excellent | ⚠️ Limited |
| Component Independence | ❌ Coupled | ✅ Independent | ⚠️ Bounded |
| Technology Diversity | ❌ Single stack | ✅ Per-service | ⚠️ Single stack |
| Future Evolution | ⚠️ Rewrite needed | ✅ Already distributed | ✅ Extract services |
| Resource Efficiency | ✅ Maximum | ❌ Overhead | ✅ Good |

---

## Architecture Option 4: "Ultra-Lightweight Text-Only MVP" 🚀 NEW

### Philosophy
Minimal viable product focused exclusively on text-based chat, eliminating all file storage to achieve maximum deployment simplicity and cost efficiency.

### Core Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                Single Rust Binary (~25MB)                   │
├─────────────────────────────────────────────────────────────┤
│  Embedded React SPA (Text-Only UI)                         │
│  ├─── Static Assets (CSS, JS, Fonts)                       │
│  ├─── Sound Assets (Embedded Audio Files)                  │
│  └─── Service Worker (PWA, Push Notifications)             │
├─────────────────────────────────────────────────────────────┤
│  Axum Web Server (HTTP + WebSocket)                        │
│  ├─── REST API Handlers (No File Upload)                  │
│  ├─── WebSocket Connection Manager                         │
│  ├─── Session-based Authentication                         │
│  └─── Rate Limiting & Security Middleware                  │
├─────────────────────────────────────────────────────────────┤
│  Text-Only Real-time Engine                                │
│  ├─── Room Actors (State Management)                       │
│  ├─── Presence Tracking                                    │
│  ├─── Message Broadcasting (Text Only)                     │
│  ├─── Typing Notifications                                 │
│  └─── Sound Command Processing                             │
├─────────────────────────────────────────────────────────────┤
│  Minimal Task Queue (Tokio Tasks)                          │
│  ├─── Webhook Delivery (Text Responses Only)              │
│  ├─── Push Notification Sending                           │
│  ├─── Background Cleanup                                   │
│  └─── No File Processing                                   │
├─────────────────────────────────────────────────────────────┤
│  Ultra-Compact SQLite Database (10-300MB)                  │
│  ├─── Connection Pool                                      │
│  ├─── FTS5 Search Index (Text Only)                       │
│  ├─── Prepared Statements                                 │
│  ├─── No Blob Storage                                     │
│  └─── Migration System                                     │
└─────────────────────────────────────────────────────────────┘
```

### Feature Scope (Text-Only MVP)

#### ✅ Included Features
- **Rich text messaging**: Bold, italic, links, code blocks
- **Real-time chat**: WebSocket-based instant messaging
- **@mentions**: User notifications and highlighting
- **Sound commands**: `/play` commands with embedded audio
- **Emoji support**: Unicode emoji (no custom images)
- **Room management**: Open, closed, and direct message rooms
- **User presence**: Online/offline status and typing indicators
- **Search functionality**: Full-text search across all messages
- **Bot integration**: Text-based webhook responses
- **PWA support**: Offline-capable progressive web app
- **Push notifications**: Web push for mentions and messages
- **Session management**: Multi-device login support

#### ❌ Excluded Features (For Later Phases)
- **File uploads**: No images, documents, or videos
- **Avatar images**: Text initials or default icons only
- **OpenGraph previews**: Links shown as plain text
- **Thumbnail generation**: No image processing
- **File attachments**: External link sharing only

### Data Volume Analysis (Text-Only)

#### Small Team (25 users)
```
Users: 25 × 0.5KB = 12.5KB
Rooms: 10 × 0.3KB = 3KB
Memberships: 250 × 0.2KB = 50KB
Messages: 10,000 × 0.8KB = 8MB
Rich Text: 2,000 × 1KB = 2MB
FTS5 Index: ~2.5MB
Sessions: 50 × 0.3KB = 15KB

Total Database: ~12.5MB
Total Storage: ~12.5MB (no files!)
```

#### Medium Team (100 users)
```
Users: 100 × 0.5KB = 50KB
Rooms: 25 × 0.3KB = 7.5KB
Memberships: 1,000 × 0.2KB = 200KB
Messages: 50,000 × 0.8KB = 40MB
Rich Text: 10,000 × 1KB = 10MB
FTS5 Index: ~12.5MB
Sessions: 200 × 0.3KB = 60KB

Total Database: ~62.5MB
Total Storage: ~62.5MB
```

#### Large Team (500 users)
```
Users: 500 × 0.5KB = 250KB
Rooms: 50 × 0.3KB = 15KB
Memberships: 5,000 × 0.2KB = 1MB
Messages: 250,000 × 0.8KB = 200MB
Rich Text: 50,000 × 1KB = 50MB
FTS5 Index: ~62.5MB
Sessions: 1,000 × 0.3KB = 300KB

Total Database: ~314MB
Total Storage: ~314MB
```

### Deployment Characteristics

#### Docker Image Size
```dockerfile
FROM scratch
COPY campfire-rust /campfire-rust
EXPOSE 80 443
CMD ["/campfire-rust"]

# Image sizes:
# Fresh deployment: ~25MB
# With 1 year data (100 users): ~90MB total
```

#### Memory Usage
```
Base Application: 1-2MB
Message Cache: 2-5MB (text only)
WebSocket Connections: 8KB × users
Search Cache: 1-5MB
Session Cache: 100KB-1MB

Total for 100 users: ~10-15MB
Total for 500 users: ~20-30MB
```

#### Resource Requirements
```
CPU: 0.1 vCPU (burst to 0.25)
Memory: 256MB (vs 4GB Rails)
Storage: 1GB (vs 50GB+ Rails)
Bandwidth: Minimal (text-only)
```

### Key Benefits

#### **Ultra-Minimal Resource Usage**
- **Memory**: 10-30MB total (vs 50-100MB Rails baseline)
- **Storage**: 10-300MB database (vs 1-3GB with files)
- **CPU**: Minimal processing (no image/video handling)
- **Network**: Text-only payloads (1-5KB vs 100KB+ with media)

#### **Deployment Simplicity**
- **Single file**: Binary + SQLite database
- **No dependencies**: No Redis, no file storage service
- **Instant backup**: Copy SQLite file = complete backup
- **Zero configuration**: Works out of the box

#### **Development Velocity**
- **No file handling complexity**: Skip upload/processing logic
- **Faster testing**: No mock file services needed
- **Simpler debugging**: Text-only data flows
- **Rapid iteration**: Deploy in seconds

#### **Cost Optimization**
- **90-95% cost reduction**: Even better than 87% target
- **Micro instances**: AWS t4g.nano ($3.50/month)
- **Edge deployment**: Raspberry Pi capable
- **Bandwidth savings**: Minimal data transfer

#### **GitHub/Distribution Friendly**
- **Small repository**: No large binary assets
- **Fast clones**: Minimal download size
- **Easy distribution**: Single binary deployment
- **Version control**: Text-only changes

### Trade-offs

#### **Feature Limitations**
- **No native file sharing**: Users must use external services (Imgur, etc.)
- **No avatar images**: Text initials only
- **No link previews**: Plain text links
- **Limited rich media**: Text and emoji only

#### **User Experience Impact**
- **Modern chat feel**: Still rich text, real-time, sounds
- **Professional usage**: Code sharing, @mentions work perfectly
- **Mobile friendly**: Fast loading, PWA support
- **Workarounds needed**: External image sharing

#### **Future Migration Complexity**
- **File system addition**: Requires architecture changes
- **Data migration**: Moving from text-only to media support
- **API changes**: Adding file upload endpoints later

### Evolution Strategy

#### Phase 1: Text-Only MVP (Months 1-3)
- Deploy ultra-lightweight version
- Validate core chat functionality
- Build user base and feedback

#### Phase 2: External File Integration (Months 4-5)
- Add support for external image links
- Implement link preview for known services
- Maintain text-only storage

#### Phase 3: Native File Support (Months 6-9)
- Add file upload API
- Implement cloud storage (S3/R2)
- Keep SQLite for metadata, files external

#### Phase 4: Full Feature Parity (Months 10-12)
- Complete Rails feature set
- Advanced file processing
- Video/document support

### Use Cases Perfect for Option 4

#### **Developer Teams**
- Code-focused discussions
- Technical documentation sharing
- Minimal distraction environment
- Fast, lightweight communication

#### **Startup MVPs**
- Rapid deployment and testing
- Minimal infrastructure costs
- Focus on core chat functionality
- Easy scaling and iteration

#### **Edge/Embedded Deployments**
- IoT device communication
- Offline-first environments
- Resource-constrained systems
- Distributed team coordination

#### **Privacy-Focused Organizations**
- No file storage concerns
- Minimal data footprint
- Easy compliance auditing
- Complete data portability

### Performance Targets (Text-Only)

- **Memory**: <30MB total (vs Rails 50-100MB)
- **Connections**: 10,000+ concurrent WebSocket
- **Startup**: <50ms cold start (faster than Option 1)
- **Throughput**: 15K+ req/sec (no file processing overhead)
- **Response times**: <2ms API, <5ms messages, <1ms static
- **Database**: Sub-millisecond queries (smaller indexes)

---

## Recommendation: Option 4 - "Ultra-Lightweight Text-Only MVP" 🚀

### Primary Rationale

**Option 4 (Ultra-Lightweight Text-Only MVP) is now the recommended approach for initial deployment:**

1. **Exceeds Cost Goals**: 90-95% cost reduction (better than 87% target)
2. **Minimal Complexity**: Eliminates file handling complexity entirely
3. **Ultra-Fast Development**: Focus on core chat features only
4. **Maximum Portability**: 25MB binary runs anywhere
5. **Perfect MVP**: Validates core value proposition quickly
6. **Clear Evolution Path**: Can add file support in Phase 2

**Fallback to Option 1 if file support is absolutely required for MVP.**

### Implementation Strategy

#### Phase 1: Core Monolith (Months 1-3)
- Single Rust binary with embedded React
- SQLite database with connection pooling
- Basic HTTP API and WebSocket support
- Essential features: auth, messages, rooms

#### Phase 2: Feature Completion (Months 4-6)
- Real-time features (presence, typing)
- File uploads and processing
- Bot integration and webhooks
- Search functionality

#### Phase 3: Optimization (Months 7-8)
- Performance tuning for 10K+ connections
- Memory optimization for <2MB target
- Security hardening and rate limiting
- Production deployment and monitoring

### Migration Path from Option 1

If scaling demands eventually require distribution:

1. **Extract WebSocket Service**: High-connection load component
2. **Extract File Processing**: CPU-intensive operations
3. **Extract Bot Services**: External integrations
4. **Database Sharding**: If SQLite becomes bottleneck

### Success Metrics

- **Cost Reduction**: Achieve 87% reduction (2 vCPU/4GB → 0.25 vCPU/0.5GB)
- **Performance**: <2MB memory, 10K+ connections, <100ms startup
- **Feature Parity**: 100% Rails functionality replicated
- **Reliability**: 99.9% uptime with graceful degradation
- **Developer Experience**: Faster development cycles than Rails

---

## Alternative Scenarios

### When to Choose Option 2 (Microservices)
- **Large development team** (10+ developers)
- **Different scaling requirements** per component
- **Regulatory requirements** for service isolation
- **Existing microservices infrastructure**
- **Cost is not primary concern**

### When to Choose Option 3 (Modular Monolith)
- **Medium development team** (3-8 developers)
- **Uncertain future scaling requirements**
- **Need for clear module boundaries**
- **Plan to extract services later**
- **Balance between simplicity and flexibility**

### When to Choose Option 4 (Ultra-Lightweight MVP)
- **MVP/Proof of concept** development
- **Extreme cost optimization** required
- **Text-focused use cases** (developer teams, documentation)
- **Edge/embedded deployments** with resource constraints
- **Rapid iteration** and validation needed
- **GitHub/single-binary distribution** preferred

---

## Conclusion

**Option 4 (Ultra-Lightweight Text-Only MVP)** is the recommended architecture for the initial Campfire Rust rewrite based on:

- **Exceeds cost goals**: 90-95% cost reduction vs 87% target
- **Minimal implementation risk**: Text-only eliminates file handling complexity
- **Ultra-fast time to market**: Focus on core chat features only
- **Maximum deployment flexibility**: 25MB binary runs anywhere
- **Perfect validation tool**: Proves core value proposition quickly
- **Clear evolution strategy**: Add file support in Phase 2 if needed

**Fallback Strategy**: If file uploads are absolutely required for MVP, use Option 1 (Monolithic Efficiency) which still achieves the 87% cost reduction goal while providing full Rails feature parity.

The text-only approach provides the fastest path to market with maximum cost savings, allowing rapid validation of the core chat experience before investing in file handling infrastructure.