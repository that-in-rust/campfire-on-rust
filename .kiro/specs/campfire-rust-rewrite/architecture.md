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
COPY campfire-rust /usr/local/bin/campfire-rust
# Database will be in mounted volume or persistent filesystem
EXPOSE $PORT
CMD ["/usr/local/bin/campfire-rust"]
```

### Deployment Strategies by Platform:
- **Docker/VPS**: Use volume mounts (`-v campfire-data:/data`)
- **Railway/Render**: Use persistent filesystem (`/app/data/`)
- **AWS/GCP**: Use managed volumes (EFS/Persistent Disks)
- **Kubernetes**: Use PersistentVolumeClaims

---

## Architecture Overview: "UI-Complete, Files-Disabled MVP" 🎯

### Philosophy
Build the complete user interface and experience while disabling only the heavy file processing backend, achieving ultra-low costs with zero UI redesign needed for future upgrades.

### Core Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                Single Rust Binary (~30MB)                   │
├─────────────────────────────────────────────────────────────┤
│  Complete React UI (Full Rails Parity)                     │
│  ├─── All Components (File Upload, Lightbox, Avatars)      │
│  ├─── Complete CSS/Styling (25+ stylesheets)               │
│  ├─── Sound Assets (Embedded Audio Files)                  │
│  ├─── Graceful Degradation (Disabled Features)             │
│  ├─── Optimistic UI with Client Message IDs                │
│  └─── Service Worker (PWA, Push Notifications)             │
├─────────────────────────────────────────────────────────────┤
│  Axum Web Server (HTTP + WebSocket)                        │
│  ├─── REST API Handlers (Stubbed File Endpoints)          │
│  ├─── WebSocket Connection Manager with State Sync         │
│  ├─── Session-based Authentication with WS Integration     │
│  └─── Rate Limiting & Security Middleware                  │
├─────────────────────────────────────────────────────────────┤
│  Coordinated Real-time Engine                              │
│  ├─── Message Coordinator (Optimistic UI + Persistence)    │
│  ├─── Room State Manager (Distributed Actors)             │
│  ├─── Presence Tracker with Connection Cleanup             │
│  ├─── Event Bus (Message/Presence/Typing Coordination)     │
│  └─── Feature Flag Broadcaster (Real-time Config Updates)  │
├─────────────────────────────────────────────────────────────┤
│  Prioritized Task Queue                                     │
│  ├─── High Priority: Message Processing                    │
│  ├─── Medium Priority: Webhook Delivery                    │
│  ├─── Low Priority: Presence Updates, Cleanup             │
│  └─── Feature Processing (Disabled/Stubbed)               │
├─────────────────────────────────────────────────────────────┤
│  Optimized SQLite Database (10-300MB)                      │
│  ├─── Write-Ahead Logging (WAL) with Checkpointing        │
│  ├─── Dedicated Writer Task (DWT) Pattern                 │
│  ├─── FTS5 Search Index with Async Updates                │
│  ├─── Connection Pool with Priority Queuing               │
│  └─── Migration System with Feature Flag Schema           │
└─────────────────────────────────────────────────────────────┘
```

### Technical Stack
- **Web Framework**: Axum (complete API, stubbed file endpoints)
- **Database**: SQLite (text-only backend, ready for files)
- **Real-time**: Actor pattern (complete implementation)
- **Frontend**: Complete React UI (all components built)
- **Task Queue**: Tokio tasks (feature-flagged file processing)
- **Authentication**: Full session management
- **Deployment**: Complete UI with minimal backend

---

## Critical Coordination Mechanisms

### 1. Optimistic UI Message Flow
```
Client                    Server                     Database
  │                        │                          │
  ├─ Send Message ────────▶│                          │
  │  (client_message_id)   │                          │
  │                        │                          │
  ├─ Optimistic UI ────────┤                          │
  │  (show pending)        │                          │
  │                        │                          │
  │                        ├─ Validate & Store ─────▶│
  │                        │                          │
  │                        ├─ Broadcast Confirmed ───┤
  │                        │  (with client_msg_id)    │
  │                        │                          │
  ├─ Replace Optimistic ◀──┤                          │
  │  (match client_id)     │                          │
```

### 2. Real-time State Synchronization
- **Event Bus Pattern**: Central coordination for all real-time events with sequence numbers
- **State Reconciliation**: WebSocket reconnection triggers state diff sync with missed events
- **Atomic Operations**: Message creation + broadcast as single database transaction
- **Connection Cleanup**: Heartbeat-based zombie connection detection with 60-second timeout
- **Circuit Breakers**: Prevent cascade failures when components are unhealthy
- **Backpressure Management**: Queue limits and load shedding prevent system overload

### 3. Feature Flag Propagation
- **Server-Side Changes**: Broadcast feature flag updates via WebSocket with versioning
- **Client-Side Caching**: Local feature flag cache with TTL and invalidation
- **Graceful Transitions**: UI components react to real-time flag changes with animations
- **Rollback Safety**: Feature flag changes can be reverted instantly with backward compatibility
- **Consistency Guarantees**: All connected clients receive flag updates within 5 seconds

### 4. Fault Tolerance and Recovery
- **Database Transactions**: All multi-step operations use atomic transactions with rollback
- **Message Retry System**: Exponential backoff retry with persistent queue for failed messages
- **Fallback Storage**: Critical operations use fallback storage when primary database fails
- **Graceful Degradation**: System continues with reduced functionality during outages
- **Error Classification**: Different error types have appropriate recovery strategies

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
COPY campfire-rust /usr/local/bin/campfire-rust
EXPOSE $PORT
CMD ["/usr/local/bin/campfire-rust"]
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

### MVP Phase 1 Targets (With Fault Tolerance)
- **Memory**: 20-50MB total (includes coordination, retry queues, fallback storage)
- **Connections**: 1,000+ concurrent WebSocket (with circuit breaker protection)
- **Startup**: <200ms cold start (includes health checks and state recovery)
- **Throughput**: 3K+ req/sec sustainable (with backpressure and retry overhead)
- **Storage**: 15MB-400MB (text-only + retry queues + fallback storage)
- **Cost Reduction**: 90-95% vs Rails (fault tolerance adds minimal cost)

### Response Time Targets (With Coordination Overhead)
- **API Calls**: <20ms (includes transaction overhead and health checks)
- **Message Operations**: <100ms (optimistic UI masks latency, includes retry logic)
- **Static Assets**: <1ms (unaffected by coordination)
- **WebSocket Messages**: <10ms routing (includes state sync and event ordering)
- **Database Queries**: <10ms (includes transaction coordination and circuit breaker)

### Reliability Targets
- **Availability**: 99.9% uptime (8.76 hours downtime per year)
- **Message Delivery**: 99.99% success rate (with retry mechanisms)
- **Data Consistency**: 100% (atomic transactions prevent corruption)
- **Recovery Time**: <30 seconds for component failures
- **State Sync**: <5 seconds for WebSocket reconnection

### Scalability Limits (Fault-Tolerant)
- **Single Room**: 150 concurrent users (reduced due to coordination overhead)
- **Total Rooms**: 75 active rooms (memory for retry queues and fallback storage)
- **Message Rate**: 75 messages/second system-wide (with retry and coordination)
- **Retry Queue**: 10,000 pending operations maximum
- **Fallback Storage**: 100MB maximum before oldest operations are discarded

---

## Evolution Strategy

### Phase 1: Complete UI, Text-Only Backend (Months 1-2)
```rust
AppConfig { 
    files_enabled: false, 
    avatars_enabled: false, 
    opengraph_enabled: false,
    max_file_size: 0
}
```
**Focus**: Complete professional UI with text-only functionality
**Cost**: 90-95% reduction, $3-5/month hosting
**Memory**: 10-30MB total

### Phase 2: Enable Avatar Uploads (Month 3)
```rust
AppConfig { 
    avatars_enabled: true,
    files_enabled: false,
    opengraph_enabled: false,
    max_file_size: 1_048_576  // 1MB for avatars
}
```
**Added**: Avatar upload, image processing, basic file storage
**Cost**: Still 85-90% reduction
**Memory**: 20-40MB total

### Phase 3: Enable Document Uploads (Month 4)
```rust
AppConfig { 
    avatars_enabled: true,
    files_enabled: true,
    opengraph_enabled: false,
    max_file_size: 10_485_760  // 10MB for documents
}
```
**Added**: Document sharing, file attachments, enhanced processing
**Cost**: 80-85% reduction
**Memory**: 30-50MB total

### Phase 4: Full Feature Parity (Months 5-6)
```rust
AppConfig { 
    files_enabled: true, 
    avatars_enabled: true, 
    opengraph_enabled: true,
    max_file_size: 52_428_800  // 50MB for all files
}
```
**Added**: Image/video processing, OpenGraph previews, complete Rails parity
**Cost**: 75-80% reduction (still significant savings)
**Memory**: 50-100MB total

### Coordination Considerations for Evolution

#### Phase Transition Coordination
- **Database Schema Evolution**: Backward-compatible migrations with feature flags
- **State Migration**: Existing optimistic messages preserved during feature rollout
- **Connection Continuity**: WebSocket connections maintained during feature flag changes
- **Rollback Safety**: Each phase can be rolled back without data loss

#### Scaling and Reliability Preparation
- **Phase 1**: Establish fault tolerance patterns, monitor reliability metrics
- **Phase 2-3**: Monitor room actor performance, prepare sharding, enhance retry mechanisms for file operations
- **Phase 3-4**: Implement horizontal scaling preparation, distributed coordination patterns
- **Phase 4+**: Multi-instance deployment with shared state coordination, distributed fallback storage

#### Reliability Evolution
- **Phase 1**: Single-instance fault tolerance with local fallback storage
- **Phase 2**: Enhanced retry mechanisms for avatar uploads, file processing circuit breakers
- **Phase 3**: Distributed retry queues, cross-instance state synchronization
- **Phase 4**: Full distributed fault tolerance with external message queues and shared storage

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

This architecture provides the optimal balance of **complete user experience**, **ultra-low costs**, and **zero redesign risk**. By building the complete UI while implementing only text-based backend functionality, we achieve:

1. **Professional appearance** that satisfies users and stakeholders
2. **90-95% cost reduction** through minimal resource usage
3. **Clear evolution path** with feature flags for gradual rollout
4. **Risk mitigation** by validating core functionality first
5. **Technical foundation** ready for future feature expansion

The approach eliminates the common MVP problem of "looking unfinished" while maintaining the cost benefits of a minimal backend implementation. Users get a complete, professional chat experience with clear expectations about future enhancements.

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