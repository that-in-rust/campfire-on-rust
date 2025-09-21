# Requirements Document - MVP Phase 1

## Introduction

This document outlines the requirements for the **MVP Phase 1** of rewriting the existing Ruby on Rails Campfire chat application using **simple, proven patterns** that prioritize rapid delivery and Rails compatibility over architectural innovation.

**Strategic Focus: Rails-Compatible Simplicity**
- **Direct Implementation**: Build working chat features using proven Rails patterns adapted to Rust
- **Rust Performance**: Leverage Rust's natural speed, memory safety, and zero-cost abstractions
- **Single Binary**: Zero coordination overhead with embedded assets using Rust's compile-time inclusion
- **Rails Parity**: Replicate ActionCable and Rails patterns using idiomatic Rust equivalents

**Core Philosophy**: Build a working chat application first, optimize and innovate later based on real usage data.

**Primary Goals:**
- **Working Chat Application**: Complete feature parity with Rails Campfire using Rust web frameworks
- **Rust Performance Benefits**: Natural speed improvements from Rust's zero-cost abstractions and memory safety
- **Simple Deployment**: Single binary with zero coordination overhead leveraging Rust's static compilation
- **Rails Compatibility**: Familiar patterns adapted to Rust's ownership model and type system
- **Evidence-Based Evolution**: Add complexity only when proven necessary

## MVP 1.0 Strategic Focus

### Complete Native Rust Web UI (Included in MVP 1.0)
- ✅ **Complete Rust Web UI** - All components using Askama templates (Rust-native templating) and HTMX (26 template files)
- ✅ **Rich text messaging** - Server-side HTML rendering with formatting, sounds, boosts using Rust string processing
- ✅ **Real-time features** - WebSocket broadcasting using tokio-tungstenite, presence, typing notifications
- ✅ **Room management** - Open/Closed/Direct rooms with membership controls using Rust's type safety
- ✅ **Authentication** - Session management using Rust crypto libraries, bot integration, role-based access
- ✅ **Search functionality** - SQLite FTS5-powered message search with rusqlite bindings
- ✅ **Push notifications** - Web Push with VAPID keys using Rust web-push crates
- ✅ **Sound system** - 59 embedded MP3 files with /play commands using Rust's include_bytes! macro

### Gracefully Deferred to v2.0 (Future Enhancement)
- 🚫 **File attachments** - Complete UI shown with "Coming in v2.0" messaging
- 🚫 **Avatar uploads** - Text-based initials with upload UI for future
- 🚫 **OpenGraph previews** - Link detection with placeholder for future unfurling

**MVP Scope:** Campfire is a web-based chat application that supports multiple rooms with access controls, direct messages, rich text messaging, search, notifications via Web Push, @mentions, and API support for bot integrations. File attachments, avatars, and OpenGraph previews are **gracefully disabled** with clear upgrade messaging.

**Architecture Approach:** Simple 3-layer monolith (Database → API → WebSocket) with server-rendered HTML using Askama templates, direct SQLite operations via rusqlite, and Rails-inspired patterns adapted to Rust's async/await model for real-time functionality.

## Technical Architecture Context

The simplified MVP implementation includes these core components:
- **Database**: Direct SQLite operations using rusqlite with FTS5 virtual table for message search
- **Real-time**: ActionCable-inspired WebSocket broadcasting using tokio-tungstenite (simple, proven patterns)
- **Authentication**: Rails-style session management using Rust crypto crates with secure tokens and bot API keys
- **Push Notifications**: Web Push with VAPID keys using rust web-push libraries for message notifications
- **Frontend**: Complete server-rendered HTML with Askama templates and HTMX for interactivity
- **Background Jobs**: Simple tokio async tasks for webhook delivery and push notifications
- **Security**: Basic rate limiting using Rust's type system, input validation, content sanitization

**Architecture Philosophy:**
- **Rails Compatibility**: Replicate Rails ActionCable behavior using Rust's async ecosystem, not improve it
- **Simple Patterns**: Use proven Rails patterns implemented in idiomatic Rust with ownership guarantees
- **No Coordination Complexity**: Direct operations leveraging Rust's fearless concurrency, no global event sequencing
- **Evidence-Based**: Add complexity only when Rails proves it's necessary

**Deferred to Future Phases:**
- File Storage: Active Storage equivalent with blob storage, image/video processing using Rust image crates
- Avatar uploads and image processing using Rust's image manipulation libraries
- OpenGraph link unfurling and preview generation using Rust HTTP clients
- Advanced coordination (only if Rails analysis proves necessary)

## Hard Constraints - Anti-Coordination Mandates

**CRITICAL: These constraints are MANDATORY for MVP Phase 1 to prevent coordination complexity madness:**

### 🚫 **FORBIDDEN PATTERNS** (Will cause immediate spec rejection)
- **NO coordination layers, coordinators, or event buses** (even though Rust's channels make these easy)
- **NO distributed transactions, sagas, or event sourcing** (despite Rust's excellent async support)
- **NO circuit breakers, retry queues, or complex error recovery** (beyond Rust's Result<T, E>)
- **NO cross-tab coordination or global state synchronization** (even with Rust's Arc/Mutex)
- **NO microservices, service mesh, or distributed architecture** (single Rust binary only)
- **NO message queues, event streams, or async coordination** (beyond basic tokio tasks)
- **NO complex state machines or coordination protocols** (keep Rust's type system simple)

### ✅ **MANDATORY SIMPLICITY PATTERNS**
- **Direct SQLite operations** - Simple INSERT/UPDATE/SELECT queries using rusqlite
- **Basic WebSocket broadcasting** - Direct room-based message sending using tokio-tungstenite
- **Rails-style session management** - Simple cookie-based authentication using Rust crypto crates
- **Simple error handling** - Basic Result<T, E> with user-friendly messages leveraging Rust's error handling
- **Direct function calls** - No async coordination between components beyond basic tokio::spawn
- **Single binary deployment** - Leveraging Rust's static compilation with embedded assets

### 📏 **COMPLEXITY LIMITS**
- **Maximum 30 total files** in entire codebase (backend + frontend templates)
- **No file over 300 lines** - Split large files into smaller Rust modules
- **Maximum 2 async operations per request** - Keep request handling simple using tokio
- **Single level of error handling** - Avoid nested Result chains, use Rust's ? operator judiciously
- **Single database connection pool** - No distributed data management, use r2d2 or similar simple pool

### 🎯 **RAILS PARITY RULE**
- **If Rails doesn't do it, we don't do it** - Use Rails as the complexity ceiling, adapt to Rust idioms
- **Replicate Rails patterns exactly** - Don't "improve" on proven Rails behavior, just make it memory-safe
- **Evidence-based additions only** - New patterns require Rails precedent, implemented in idiomatic Rust
- **Simple beats clever** - Choose obvious Rust solutions over optimized ones

## Requirements

### Requirement 0: Anti-Coordination Architecture Enforcement (MVP Phase 1)

**User Story:** As a project stakeholder, I want absolute assurance that the MVP implementation remains simple and Rails-equivalent, so that we avoid coordination complexity that increases costs, development time, and system fragility.

#### Acceptance Criteria

1. WHEN any component is implemented THEN it SHALL use direct function calls instead of async coordination, implement single-threaded logic where possible using Rust's ownership model, avoid global state management beyond simple Arc<Mutex<T>>, and replicate Rails patterns exactly using idiomatic Rust
2. WHEN database operations occur THEN they SHALL use direct SQLite queries with rusqlite and basic connection pooling (r2d2), avoid distributed transactions or coordination, implement simple INSERT/UPDATE/SELECT patterns, and maintain Rails-equivalent data access patterns
3. WHEN WebSocket functionality is implemented THEN it SHALL use basic room-based broadcasting like Rails ActionCable using tokio-tungstenite, avoid complex message ordering or delivery guarantees, implement simple connection management with Rust's type safety, and provide basic presence tracking without coordination
4. WHEN error handling is implemented THEN it SHALL use simple Result<T, E> patterns leveraging Rust's error handling, provide user-friendly error messages, avoid complex retry logic or circuit breakers, and implement basic logging using Rust logging crates without coordination overhead
5. WHEN real-time features are added THEN they SHALL replicate Rails ActionCable behavior exactly using tokio WebSockets, avoid event sourcing or complex state management, use direct WebSocket sends to room subscribers, and maintain simple connection state with Rust's memory safety
6. WHEN background tasks are needed THEN they SHALL use basic tokio::spawn for simple async tasks, avoid message queues or complex job systems, implement direct webhook delivery using Rust HTTP clients, and maintain Rails-equivalent background job simplicity
7. WHEN authentication is implemented THEN it SHALL use Rails-style session cookies with Rust crypto libraries, avoid complex OAuth flows or token management, implement basic bcrypt password hashing using Rust bcrypt crates, and maintain simple session state management
8. WHEN the codebase grows THEN it SHALL maintain maximum 30 total files, keep individual files under 300 lines, avoid deep module hierarchies, and prioritize readability over Rust's zero-cost abstraction capabilities
9. WHEN performance optimization is considered THEN it SHALL use Rust's natural performance benefits and memory safety, avoid premature optimization or complex caching, maintain simple database queries, and focus on Rails-equivalent functionality first
10. WHEN any "improvement" over Rails is proposed THEN it SHALL be rejected unless it provides direct cost reduction, maintains identical user experience, requires no additional complexity beyond Rust's safety guarantees, and has clear evidence of necessity
11. WHEN code review occurs THEN it SHALL verify compliance with anti-coordination constraints, check for forbidden patterns, ensure Rails parity adapted to Rust idioms, and reject any coordination complexity regardless of perceived benefits
12. WHEN deployment is implemented THEN it SHALL use single binary with embedded assets using Rust's compile-time inclusion, avoid orchestration or service discovery, maintain simple environment configuration, and provide basic health checks without coordination overhead

## Strategic Prioritization: Feature-First Implementation

**Governing Principle**: Build a working chat application using proven Rails patterns adapted to Rust, focusing on user value over architectural innovation.

### Phase 1: Core Chat Functionality
**Primary Focus**: Essential chat features that users need immediately
- **Basic message sending/receiving** - Direct SQLite insert/broadcast using rusqlite and tokio
- **Room-based chat** - Simple room membership checks leveraging Rust's type system
- **User authentication** - Rails-style sessions using Rust crypto libraries only
- **WebSocket connections** - Basic connection management using tokio-tungstenite
- **Message history** - Simple database queries with rusqlite

**Governing Thought**: These represent the absolute minimum viable chat application - anything beyond this scope is deferred to v2.0 unless it's a direct Rails equivalent pattern implemented in idiomatic Rust.
## MVP Sufficiency Assessment

**YES - This MVP is sufficient for a text-only Campfire replacement with exact same UI.**

The MVP includes all essential Campfire features for text-based chat:
- ✅ **Complete UI Parity** - All 26 Askama templates replicate Campfire's exact interface
- ✅ **Core Chat Features** - Message sending, room management, direct messages, @mentions
- ✅ **Real-time Experience** - WebSocket broadcasting, presence, typing indicators
- ✅ **Rich Text Support** - Formatting, sounds (/play commands), message boosts
- ✅ **Search Functionality** - Full-text search across message history
- ✅ **Authentication & Security** - Session management, bot API, role-based access
- ✅ **Push Notifications** - Web Push for message alerts
- ✅ **Sound System** - All 59 embedded MP3 files for /play commands

**Graceful Feature Deferrals:**
- File attachments show "Coming in v2.0" messaging
- Avatar uploads display text initials with future upload UI
- OpenGraph previews show link detection with placeholder

**User Experience**: Identical to original Campfire for text chat with clear upgrade path messaging for deferred features. Users get full chat functionality immediately while understanding what's coming next.

**Technical Completeness**: Single Rust binary with embedded assets, SQLite database, WebSocket real-time features, and Rails-equivalent patterns - everything needed for production text chat deployment.


I hope I get the requirements right