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
│  └─── Service Worker (PWA, Push Notifications)             │
├─────────────────────────────────────────────────────────────┤
│  Axum Web Server (HTTP + WebSocket)                        │
│  ├─── REST API Handlers (Stubbed File Endpoints)          │
│  ├─── WebSocket Connection Manager                         │
│  ├─── Session-based Authentication                         │
│  └─── Rate Limiting & Security Middleware                  │
├─────────────────────────────────────────────────────────────┤
│  Complete Real-time Engine                                  │
│  ├─── Room Actors (State Management)                       │
│  ├─── Presence Tracking                                    │
│  ├─── Message Broadcasting (Rich Text)                     │
│  ├─── Typing Notifications                                 │
│  └─── Sound Command Processing                             │
├─────────────────────────────────────────────────────────────┤
│  Feature-Flagged Task Queue                                │
│  ├─── Webhook Delivery (Text Responses)                   │
│  ├─── Push Notification Sending                           │
│  ├─── Background Cleanup                                   │
│  └─── File Processing (Disabled/Stubbed)                  │
├─────────────────────────────────────────────────────────────┤
│  Text-Only SQLite Database (10-300MB)                      │
│  ├─── Connection Pool                                      │
│  ├─── FTS5 Search Index                                   │
│  ├─── Prepared Statements                                 │
│  ├─── No Blob Storage (Feature Flagged)                   │
│  └─── Migration System                                     │
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

### MVP Phase 1 Targets
- **Memory**: 10-30MB total (same as text-only)
- **Connections**: 10,000+ concurrent WebSocket
- **Startup**: <50ms cold start
- **Throughput**: 15K+ req/sec
- **Storage**: 12.5MB-314MB (text-only)
- **Cost Reduction**: 90-95% vs Rails

### Response Time Targets
- **API Calls**: <2ms
- **Message Operations**: <5ms
- **Static Assets**: <1ms
- **WebSocket Messages**: <1ms routing
- **Database Queries**: <2ms

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