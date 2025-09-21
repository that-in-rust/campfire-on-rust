# Requirements Document - MVP Phase 1

## Introduction

This document outlines the requirements for the **MVP Phase 1** of rewriting the existing Ruby on Rails Campfire chat application using **simple, proven patterns** that prioritize rapid delivery and Rails compatibility over architectural innovation.

**Strategic Focus: Rails-Compatible Simplicity**
- **Direct Implementation**: Build working chat features using proven Rails patterns
- **Rust Performance**: Leverage Rust's natural speed and safety benefits
- **Single Binary**: Zero coordination overhead with embedded assets
- **Rails Parity**: Replicate ActionCable and Rails patterns exactly

**Core Philosophy**: Build a working chat application first, optimize and innovate later based on real usage data.
**Primary Goals:**
- **Working Chat Application**: Complete feature parity with Rails Campfire
- **Rust Performance Benefits**: Natural speed improvements from Rust implementation
- **Simple Deployment**: Single binary with zero coordination overhead
- **Rails Compatibility**: Familiar patterns for easy maintenance and extension
- **Evidence-Based Evolution**: Add complexity only when proven necessary

## MVP 1.0 Strategic Focus

### Complete Native Rust Web UI (Included in MVP 1.0)
- ✅ **Complete Rust Web UI** - All components using Askama templates and HTMX (26 template files)
- ✅ **Rich text messaging** - Server-side HTML rendering with formatting, sounds, boosts
- ✅ **Real-time features** - WebSocket broadcasting, presence, typing notifications
- ✅ **Room management** - Open/Closed/Direct rooms with membership controls
- ✅ **Authentication** - Session management, bot integration, role-based access
- ✅ **Search functionality** - FTS5-powered message search
- ✅ **Push notifications** - Web Push with VAPID keys
- ✅ **Sound system** - 59 embedded MP3 files with /play commands

### Gracefully Deferred to v2.0 (Future Enhancement)
- 🚫 **File attachments** - Complete UI shown with "Coming in v2.0" messaging
- 🚫 **Avatar uploads** - Text-based initials with upload UI for future
- 🚫 **OpenGraph previews** - Link detection with placeholder for future unfurling

**MVP Scope:** Campfire is a web-based chat application that supports multiple rooms with access controls, direct messages, rich text messaging, search, notifications via Web Push, @mentions, and API support for bot integrations. File attachments, avatars, and OpenGraph previews are **gracefully disabled** with clear upgrade messaging.

**Architecture Approach:** Simple 3-layer monolith (Database → API → WebSocket) with server-rendered HTML using Askama templates, direct SQLite operations, and Rails-inspired patterns for real-time functionality.

## Technical Architecture Context

The simplified MVP implementation includes these core components:
- **Database**: Direct SQLite operations with FTS5 virtual table for message search
- **Real-time**: ActionCable-inspired WebSocket broadcasting (simple, proven patterns)
- **Authentication**: Rails-style session management with secure tokens and bot API keys
- **Push Notifications**: Web Push with VAPID keys for message notifications
- **Frontend**: Complete server-rendered HTML with Askama templates and HTMX for interactivity
- **Background Jobs**: Simple async tasks for webhook delivery and push notifications
- **Security**: Basic rate limiting, input validation, content sanitization

**Architecture Philosophy:**
- **Rails Compatibility**: Replicate Rails ActionCable behavior, not improve it
- **Simple Patterns**: Use proven Rails patterns implemented in idiomatic Rust
- **No Coordination Complexity**: Direct operations, no global event sequencing
- **Evidence-Based**: Add complexity only when Rails proves it's necessary

**Deferred to Future Phases:**
- File Storage: Active Storage with blob storage, image/video processing
- Avatar uploads and image processing
- OpenGraph link unfurling and preview generation
- Advanced coordination (only if Rails analysis proves necessary)

## Hard Constraints - Anti-Coordination Mandates

**CRITICAL: These constraints are MANDATORY for MVP Phase 1 to prevent coordination complexity madness:**

### 🚫 **FORBIDDEN PATTERNS** (Will cause immediate spec rejection)
- **NO coordination layers, coordinators, or event buses**
- **NO distributed transactions, sagas, or event sourcing**
- **NO circuit breakers, retry queues, or complex error recovery**
- **NO cross-tab coordination or global state synchronization**
- **NO microservices, service mesh, or distributed architecture**
- **NO message queues, event streams, or async coordination**
- **NO complex state machines or coordination protocols**

### ✅ **MANDATORY SIMPLICITY PATTERNS**
- **Direct SQLite operations** - Simple INSERT/UPDATE/SELECT queries
- **Basic WebSocket broadcasting** - Direct room-based message sending
- **Rails-style session management** - Simple cookie-based authentication
- **Simple error handling** - Basic Result<T, E> with user-friendly messages
- **Direct function calls** - No async coordination between components
- **Single binary deployment** - No orchestration or service discovery

### 📏 **COMPLEXITY LIMITS**
- **Maximum 50 total files** in entire codebase (backend + frontend templates)
- **No file over 500 lines** - Split large files into smaller modules
- **Maximum 3 async operations per request** - Keep request handling simple
- **No more than 2 levels of error handling** - Avoid nested Result chains
- **Single database connection pool** - No distributed data management

### 🎯 **RAILS PARITY RULE**
- **If Rails doesn't do it, we don't do it** - Use Rails as the complexity ceiling
- **Replicate Rails patterns exactly** - Don't "improve" on proven Rails behavior
- **Evidence-based additions only** - New patterns require Rails precedent
- **Simple beats clever** - Choose obvious solutions over optimized ones

## Requirements

### Requirement 0: Anti-Coordination Architecture Enforcement (MVP Phase 1)

**User Story:** As a project stakeholder, I want absolute assurance that the MVP implementation remains simple and Rails-equivalent, so that we avoid coordination complexity that increases costs, development time, and system fragility.

#### Acceptance Criteria

1. WHEN any component is implemented THEN it SHALL use direct function calls instead of async coordination, implement single-threaded logic where possible, avoid global state management, and replicate Rails patterns exactly
2. WHEN database operations occur THEN they SHALL use direct SQLite queries with basic connection pooling, avoid distributed transactions or coordination, implement simple INSERT/UPDATE/SELECT patterns, and maintain Rails-equivalent data access patterns
3. WHEN WebSocket functionality is implemented THEN it SHALL use basic room-based broadcasting like Rails ActionCable, avoid complex message ordering or delivery guarantees, implement simple connection management, and provide basic presence tracking without coordination
4. WHEN error handling is implemented THEN it SHALL use simple Result<T, E> patterns, provide user-friendly error messages, avoid complex retry logic or circuit breakers, and implement basic logging without coordination overhead
5. WHEN real-time features are added THEN they SHALL replicate Rails ActionCable behavior exactly, avoid event sourcing or complex state management, use direct WebSocket sends to room subscribers, and maintain simple connection state
6. WHEN background tasks are needed THEN they SHALL use basic tokio::spawn for simple async tasks, avoid message queues or complex job systems, implement direct webhook delivery, and maintain Rails-equivalent background job simplicity
7. WHEN authentication is implemented THEN it SHALL use Rails-style session cookies, avoid complex OAuth flows or token management, implement basic bcrypt password hashing, and maintain simple session state management
8. WHEN the codebase grows THEN it SHALL maintain maximum 50 total files, keep individual files under 500 lines, avoid deep module hierarchies, and prioritize readability over optimization
9. WHEN performance optimization is considered THEN it SHALL use Rust's natural performance benefits, avoid premature optimization or complex caching, maintain simple database queries, and focus on Rails-equivalent functionality first
10. WHEN any "improvement" over Rails is proposed THEN it SHALL be rejected unless it provides direct cost reduction, maintains identical user experience, requires no additional complexity, and has clear evidence of necessity
11. WHEN code review occurs THEN it SHALL verify compliance with anti-coordination constraints, check for forbidden patterns, ensure Rails parity, and reject any coordination complexity regardless of perceived benefits
12. WHEN deployment is implemented THEN it SHALL use single binary with embedded assets, avoid orchestration or service discovery, maintain simple environment configuration, and provide basic health checks without coordination overhead

## Strategic Prioritization: Feature-First Implementation

**Governing Principle**: Build a working chat application using proven Rails patterns, focusing on user value over architectural innovation.

### Phase 1: Core Chat Functionality
**Primary Focus**: Essential chat features that users need immediately
- **Message deduplication** (Gap #1)
- **WebSocket reconnection** (Gap #2)
- **Write serialization** (Gap #3)
- **Session security** (Gap #4)
- **Presence tracking** (Gap #5)

### Phase 2: Enhanced Features
**Secondary Priority**: Features that improve user experience
- **Sound system** (56 effects)
- **Advanced UI components**
- **Search functionality**
- **Push notifications**

### Phase 3: Future Enhancements
**Tertiary Priority**: Features for v2.0 and beyond
- **File attachments**
- **Avatar uploads**
- **OpenGraph previews**
- **Advanced integrations**

**Key Insight**: Focus on delivering working chat functionality first, then iterate based on real user feedback and usage patterns.
## 5 Critical Gaps That Must Be Solved

**Governing Thought (Minto Apex)**: These gaps represent the only coordination complexity we accept - each has proven Rails solutions that we replicate exactly, avoiding over-engineering while ensuring reliability.
