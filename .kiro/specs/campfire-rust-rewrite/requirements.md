# Requirements Document

## Introduction

This document outlines the requirements for rewriting the existing Ruby on Rails Campfire chat application to use a Rust backend with a React frontend. The primary goal is to reduce cloud hosting costs while maintaining 100% feature parity and preserving the existing user interface and user experience.

Campfire is a web-based chat application that supports multiple rooms with access controls, direct messages, file attachments with previews, search, notifications via Web Push, @mentions, and API support for bot integrations. It is single-tenant where public rooms are accessible by all users in the system.

The current Rails implementation uses SQLite with FTS5 full-text search, ActionCable for WebSocket connections, Active Storage for file handling, Turbo Streams for real-time updates, and includes 50+ sound effects, Progressive Web App support, OpenGraph link unfurling, and sophisticated presence tracking.

## Technical Architecture Context

The existing system has these key components that must be replicated:
- **Database**: SQLite with FTS5 virtual table for message search, 12 main tables with complex relationships
- **Real-time**: ActionCable channels (RoomChannel, PresenceChannel, TypingNotificationsChannel, etc.)
- **File Storage**: Active Storage with blob storage, image/video thumbnail generation, VIPS image processing
- **Authentication**: Session-based auth with secure tokens, bot API keys, CSRF protection
- **Push Notifications**: Web Push with VAPID keys, thread pool for delivery (50 threads, 150 HTTP connections)
- **Frontend**: Stimulus controllers with complex JavaScript models for message formatting, autocomplete, presence
- **Background Jobs**: Webhook delivery, push notification sending, file processing
- **Security**: Rate limiting, private network guards for OpenGraph, content sanitization

## Requirements

### Requirement 1: Message System and Rich Content

**User Story:** As a chat user, I want all message functionality to work identically including rich text, attachments, sounds, and boosts, so that I have the complete chat experience.

#### Acceptance Criteria

1. WHEN a user sends a message THEN the system SHALL store it with client_message_id (UUID format), creator_id, room_id, created_at/updated_at timestamps and broadcast via Turbo Streams within 100ms using turbo_stream_from @room, :messages
2. WHEN a message contains rich text THEN the system SHALL store it in action_text_rich_texts table with HTML body, support Trix editor formatting with bold/italics/blockquotes/hyperlinks, and render with proper sanitization
3. WHEN a user uploads a file attachment THEN the system SHALL store it in active_storage_blobs with key, filename, content_type, byte_size, checksum, and service_name fields, supporting drag-and-drop, clipboard paste, and file picker
4. WHEN an image/video is uploaded THEN the system SHALL generate thumbnails with max dimensions 1200x800 using VIPS-compatible processing, create variant records, and display client-side previews during upload
5. WHEN a user plays a sound command (/play soundname) THEN the system SHALL recognize 50+ predefined sounds (56k, bell, bezos, bueller, trombone, etc.) and render special UI element for sound playback instead of plain text
6. WHEN a user boosts a message THEN the system SHALL create boost record with message_id, booster_id, content (max 16 chars emoji), timestamps, and broadcast via Turbo Streams to append/remove boost from message
7. WHEN messages are paginated THEN the system SHALL support before/after parameters, page_around functionality, last_page method, and maintain scroll position with proper threading using intersection observers
8. WHEN a message is edited/deleted THEN the system SHALL broadcast updates via Turbo Streams using replace/remove actions, maintain message integrity, and require creator or administrator permissions
9. WHEN emoji-only messages are detected THEN the system SHALL apply message--emoji CSS class using Unicode emoji detection regex and enlarge display appropriately
10. WHEN code blocks are present THEN the system SHALL apply syntax highlighting using highlight.js for plain text code blocks with language detection and proper formatting
11. WHEN @mentions are processed THEN the system SHALL use <action-text-attachment> tags with Signed Global ID (SGID), trigger rich_autocomplete_controller.js with 300ms debounce, and send notifications to mentioned users
12. WHEN optimistic UI updates occur THEN the system SHALL generate temporary client_message_id (UUID), create pending message UI, show upload progress for files, and replace with confirmed message using same client_message_id

### Requirement 2: Room Types and Membership Management

**User Story:** As a user, I want to create and manage different types of rooms with sophisticated access controls and membership tracking, so that I can organize conversations appropriately.

#### Acceptance Criteria

1. WHEN a user creates an open room (Rooms::Open) THEN the system SHALL automatically grant membership to all active users via after_save_commit callback, auto-grant to new users joining the account, and make rooms public to all account users
2. WHEN a user creates a closed room (Rooms::Closed) THEN the system SHALL restrict access to explicitly invited members only, require manual invitation by admin/creator, and provide UI with user selector for initial members
3. WHEN a user creates a direct message (Rooms::Direct) THEN the system SHALL enforce singleton pattern using Set comparison of user_ids, find existing room or create new one, set default involvement to "everything", and auto-generate name from member list
4. WHEN memberships are managed THEN the system SHALL support involvement levels: invisible (hidden from sidebar), nothing (no notifications), mentions (@mention notifications only), everything (all message notifications) with proper enum scoping
5. WHEN a room receives a message THEN the system SHALL call room.receive(message) to update unread_at for visible disconnected members excluding creator, mark room as unread only when user's Membership is disconnected
6. WHEN a user accesses a room THEN the system SHALL support message pagination with page_before, page_after, last_page, and page_around methods, load last page by default, and establish real-time ActionCable connection
7. WHEN a room is deleted THEN the system SHALL cascade delete messages, memberships, boosts, and broadcast removal via Turbo Streams to all affected users' sidebars
8. WHEN room membership changes THEN the system SHALL support grant_to/revoke_from batch operations, reset remote connections on membership destruction, and broadcast sidebar updates via Turbo Streams
9. WHEN direct rooms are managed THEN the system SHALL use Set comparison of user_ids for singleton behavior, optimize for large room counts (10K+), and handle membership efficiently with proper indexing
10. WHEN room types are processed THEN the system SHALL use Single Table Inheritance (STI) with type column storing class name, support open?, closed?, direct? methods, and enable specialized behavior in subclasses
11. WHEN room involvement is changed THEN the system SHALL cycle through sequence: mentions → everything → nothing → invisible → mentions via Rooms::InvolvementsController, update bell icon UI, and broadcast sidebar changes
12. WHEN room settings are managed THEN the system SHALL allow name changes, member addition/removal for closed rooms, and broadcast all changes via Turbo Streams to affected users

### Requirement 3: User Authentication and Session Management

**User Story:** As a system administrator, I want comprehensive user authentication with session management, role-based access, and security features, so that the system is secure and manageable.

#### Acceptance Criteria

1. WHEN first run setup occurs THEN the system SHALL detect empty database (Account.any? is false), present "Set up Campfire" screen via FirstRunsController, create singleton Account, first User with administrator role, initial "All Talk" Rooms::Open, and auto-login administrator
2. WHEN a user registers THEN the system SHALL verify join_code format "XXXX-XXXX-XXXX" against Current.account.join_code via verify_join_code, create user with has_secure_password and default member role, auto-grant memberships to all existing Rooms::Open, and redirect existing emails to sign-in
3. WHEN a user logs in THEN the system SHALL perform browser compatibility check via AllowBrowser concern, authenticate via User.authenticate_by(email_address, password), create Session record with secure token, set httponly SameSite=Lax session_token cookie, and redirect to last visited room
4. WHEN login attempts exceed limits THEN the system SHALL rate limit to 10 attempts per 3 minutes via SessionsController rate limiting, return :too_many_requests status, and log security events
5. WHEN sessions are managed THEN the system SHALL track token, ip_address, user_agent, last_active_at with 1-hour refresh rate, implement automatic cleanup, and support concurrent sessions per user
6. WHEN user avatars are handled THEN the system SHALL use has_one_attached :avatar with Active Storage, generate signed avatar tokens for security, serve with proper caching headers, and support multiple formats
7. WHEN user roles are managed THEN the system SHALL support role enum (member: 0, administrator: 1, bot: 2), implement can_administer? method checking role OR record ownership, and use ensure_can_administer before_action for protected operations
8. WHEN users are deactivated THEN the system SHALL execute User#deactivate method to close_remote_connections, delete non-direct memberships, anonymize email with UUID suffix, set active=false, and delete sessions
9. WHEN bot authentication occurs THEN the system SHALL parse bot_key format "id-token", authenticate via User.authenticate_bot with bot_token (SecureRandom.alphanumeric(12)), skip CSRF protection via allow_bot_access, and restrict to designated controllers
10. WHEN session transfer occurs THEN the system SHALL generate unique single-use transfer URL and QR code via User::Transferable concern, validate transfer ID via Sessions::TransfersController, create new session on second device, and provide passwordless cross-device login
11. WHEN sign-out occurs THEN the system SHALL remove push notification subscription from database, delete session_token cookie, destroy Session record, and redirect to sign-in page
12. WHEN authorization is enforced THEN the system SHALL use deny_bots filter globally except for controllers with allow_bot_access, implement dual-condition logic (administrator role OR record ownership), and protect all destructive actions

### Requirement 4: Real-time Communication and Presence

**User Story:** As a chat user, I want sophisticated real-time updates for messages, presence tracking, and typing indicators with proper connection management, so that I have an immediate and responsive chat experience.

#### Acceptance Criteria

1. WHEN messages are broadcast THEN the system SHALL use Turbo Streams format with stream_for @room, broadcast_append_to room :messages, and ActionCable.server.broadcast for unread rooms
2. WHEN presence is managed THEN the system SHALL track connections count, connected_at timestamp with 60-second TTL, and support connected/disconnected scopes
3. WHEN users connect to presence THEN the system SHALL call membership.present to increment connections, update connected_at, clear unread_at, and broadcast read status
4. WHEN users disconnect THEN the system SHALL call membership.disconnected to decrement connections, set connected_at to nil when connections < 1
5. WHEN presence refreshes THEN the system SHALL send refresh action every 50 seconds, call membership.refresh_connection to maintain connection state
6. WHEN typing notifications occur THEN the system SHALL broadcast start/stop actions with user attributes (id, name) to TypingNotificationsChannel subscribers
7. WHEN visibility changes THEN the system SHALL delay by 5 seconds using VISIBILITY_CHANGE_DELAY, track wasVisible state, and send present/absent actions
8. WHEN WebSocket connections are managed THEN the system SHALL identify by current_user, authenticate via find_session_by_cookie, and reject unauthorized connections
9. WHEN channels are subscribed THEN the system SHALL verify room access via current_user.rooms.find_by(id: params[:room_id]) and reject if unauthorized
10. WHEN connection state changes THEN the system SHALL broadcast to user-specific channels for read rooms and unread room badge updates

### Requirement 5: Bot Integration and Webhook System

**User Story:** As a developer, I want comprehensive bot integration with webhook delivery, API authentication, and message processing, so that I can extend Campfire's functionality with external services.

#### Acceptance Criteria

1. WHEN bots are created THEN the system SHALL use User.create_bot! with bot_token (SecureRandom.alphanumeric(12)), role: :bot enum value (2), optional webhook_url, and management via Accounts::BotsController for administrators
2. WHEN bot authentication occurs THEN the system SHALL parse bot_key "id-token" format, authenticate via User.authenticate_bot using bot_token, set Current.user with :bot_key authentication method, and restrict access via deny_bots filter except allowed controllers
3. WHEN webhook triggers are detected THEN the system SHALL trigger Bot::WebhookJob when bot is @mentioned in any room OR when any message is posted in Direct Message room with bot membership, check eligibility in MessagesController#create
4. WHEN webhook payloads are built THEN the system SHALL construct detailed JSON payload with user (id, name), room (id, name, path), message (id, body.html, body.plain, path), and remove bot's own mentions using attachable_plain_text_representation
5. WHEN webhook delivery occurs THEN the system SHALL execute Webhook#deliver method, POST to bot's webhook_url with 7-second timeout, Content-Type: application/json, handle Net::OpenTimeout/ReadTimeout, and process asynchronously via Bot::WebhookJob
6. WHEN webhook responses are processed THEN the system SHALL extract text from text/html or text/plain responses, create attachment from binary responses with proper MIME types, validate response content, and handle malformed responses gracefully
7. WHEN webhook replies are sent THEN the system SHALL create messages with extracted content via Messages::ByBotsController, set creator to bot user, broadcast_create to room via Turbo Streams, and maintain same message flow as user messages
8. WHEN bot tokens are reset THEN the system SHALL generate new SecureRandom.alphanumeric(12) token via Accounts::Bots::KeysController, invalidate old token immediately, display new token to administrator, and update bot_token field
9. WHEN bot API requests are made THEN the system SHALL bypass CSRF protection for Messages::ByBotsController, authenticate via bot_key parameter, process messages identically to user messages, and maintain API compatibility
10. WHEN bot management occurs THEN the system SHALL provide administrator interface to create/edit/deactivate bots, modify name/avatar/webhook_url, reset API keys, and list all active bots with proper permissions
11. WHEN bot permissions are enforced THEN the system SHALL restrict bots to programmatic functions only, prevent access to standard user interface, allow only designated API endpoints, and maintain strict separation from user workflows
12. WHEN webhook failures occur THEN the system SHALL log delivery failures, implement retry logic for transient errors, handle bot service unavailability gracefully, and provide administrator visibility into webhook status

### Requirement 6: Performance Optimization and Resource Efficiency

**User Story:** As a system operator, I want the new implementation to dramatically reduce resource usage while maintaining or improving performance, so that I can reduce hosting costs by 75-90% while improving user experience.

#### Acceptance Criteria

1. WHEN the system handles memory usage THEN it SHALL use <2MB baseline vs Rails 50-100MB, with efficient Rust structs and optimized WebSocket connection handling achieving 5-10x memory reduction
2. WHEN concurrent connections are managed THEN it SHALL support 10,000+ concurrent WebSocket connections vs Rails ~1,000 using async/await and tokio runtime with proper backpressure
3. WHEN the system starts up THEN it SHALL be ready in <100ms cold start vs Rails several seconds, including SQLite database opening, FTS5 index verification, and embedded asset loading
4. WHEN HTTP requests are processed THEN it SHALL achieve 10-12k requests/second vs Rails few hundred per core, with <5ms response times for API calls and <10ms for message operations
5. WHEN database operations occur THEN it SHALL use SQLite connection pooling with prepared statements, maintain <2ms query times with compile-time SQL validation via Diesel
6. WHEN file processing happens THEN it SHALL use async image processing with tokio::spawn_blocking for CPU-bound tasks, generate thumbnails without blocking message delivery
7. WHEN static assets are served THEN it SHALL embed React build artifacts in binary creating <50MB Docker images vs Rails several hundred MB, with zero-copy serving
8. WHEN WebSocket broadcasting occurs THEN it SHALL use efficient message serialization, batch broadcasts to multiple connections, and minimize memory allocations per connection
9. WHEN search operations are performed THEN it SHALL leverage SQLite FTS5 with optimized queries, proper indexing, and result caching achieving sub-millisecond search times
10. WHEN measuring cost efficiency THEN it SHALL demonstrate 87% cost reduction example (2 vCPU/4GB → 0.25 vCPU/0.5GB) enabling single instance to replace multiple Rails servers

### Requirement 7: Data Migration and Schema Compatibility

**User Story:** As a system administrator, I want comprehensive data migration from the Rails SQLite database with full schema compatibility, so that no data is lost and all existing functionality continues working.

#### Acceptance Criteria

1. WHEN the migration runs THEN it SHALL transfer all 12 tables: accounts, users, rooms, messages, memberships, boosts, sessions, webhooks, push_subscriptions, searches, action_text_rich_texts, active_storage_blobs/attachments
2. WHEN schema is migrated THEN it SHALL preserve exact column types, constraints, and indexes including unique indexes on sessions.token, users.email_address, users.bot_token
3. WHEN Active Storage is migrated THEN it SHALL transfer blobs with key, filename, content_type, metadata, service_name, byte_size, checksum and maintain attachment relationships
4. WHEN ActionText content is migrated THEN it SHALL preserve rich_texts records with name, body, record_type, record_id relationships and HTML formatting
5. WHEN FTS5 search index is migrated THEN it SHALL rebuild message_search_index virtual table with identical tokenization (Porter stemming) and search capabilities
6. WHEN password hashes are migrated THEN it SHALL maintain bcrypt compatibility for existing password_digest values and session authentication
7. WHEN foreign key relationships are preserved THEN it SHALL maintain all associations: room->messages, user->memberships, message->boosts, user->sessions, user->webhooks
8. WHEN enumerated values are migrated THEN it SHALL preserve user.role (member/administrator/bot), membership.involvement (invisible/nothing/mentions/everything) mappings
9. WHEN timestamps are migrated THEN it SHALL preserve created_at, updated_at, last_active_at, connected_at, unread_at with proper timezone handling
10. WHEN data integrity is validated THEN it SHALL verify foreign key constraints, check for orphaned records, validate enum values, and report migration statistics

### Requirement 8: Frontend React Implementation with Stimulus Parity

**User Story:** As an existing user, I want the new React frontend to replicate every Stimulus controller behavior and CSS styling exactly, so that the interface is indistinguishable from the current implementation.

#### Acceptance Criteria

1. WHEN CSS is implemented THEN it SHALL include all 25+ stylesheets: base, messages, composer, avatars, buttons, code, lightbox, nav, panels, sidebar, signup, etc. with identical styling
2. WHEN message formatting occurs THEN it SHALL replicate MessageFormatter with threading (5-minute window), first-of-day detection, emoji-only detection, mention highlighting, and code syntax highlighting
3. WHEN the composer is used THEN it SHALL replicate ComposerController with toolbar toggle, file upload (drag/drop/paste), keyboard shortcuts (Enter/Cmd+Enter), and client message rendering
4. WHEN presence is tracked THEN it SHALL replicate PresenceController with 50-second refresh timer, 5-second visibility delay, WebSocket connection management, and proper state tracking
5. WHEN autocomplete is used THEN it SHALL replicate AutocompleteController with 300ms debounce, fuzzy user search, keyboard navigation, and selection management
6. WHEN typing notifications occur THEN it SHALL replicate TypingNotificationsController with throttled sending, user tracking, and proper display/hiding logic
7. WHEN scroll management happens THEN it SHALL replicate ScrollManager with auto-scroll threshold (100px), keep-scroll positioning, and pending operation queuing
8. WHEN notifications are handled THEN it SHALL replicate NotificationsController with service worker registration, VAPID subscription, permission handling, and bell pulsing
9. WHEN lightbox is used THEN it SHALL replicate LightboxController with image/video display, keyboard navigation, and proper modal behavior
10. WHEN client messages are rendered THEN it SHALL replicate ClientMessage with emoji detection regex, sound command parsing, rich text handling, and template substitution
11. WHEN message pagination occurs THEN it SHALL replicate MessagePaginator with intersection observer, excess message trimming, and proper loading states
12. WHEN routes are handled THEN it SHALL support all Rails routes including /rooms/:id, /rooms/:id/@:message_id, /users/:id, /join/:join_code, and API endpoints

### Requirement 9: Deployment Architecture and Operations

**User Story:** As a system operator, I want a single-binary deployment with embedded assets and identical operational characteristics to the current Docker setup, so that deployment and maintenance are simplified.

#### Acceptance Criteria

1. WHEN the binary is built THEN it SHALL embed all React build artifacts, CSS files, JavaScript bundles, images, and sound files using include_bytes! or similar
2. WHEN environment configuration is used THEN it SHALL support identical variables: SSL_DOMAIN, DISABLE_SSL, VAPID_PUBLIC_KEY, VAPID_PRIVATE_KEY, SENTRY_DSN, SECRET_KEY_BASE, HTTP_*_TIMEOUT
3. WHEN SSL is configured THEN it SHALL use ACME client for Let's Encrypt certificates, automatic renewal, HTTP->HTTPS redirects, and HSTS headers
4. WHEN the system starts THEN it SHALL run SQLite migrations automatically, verify FTS5 support, create storage directories, and bind to ports 80/443
5. WHEN health checks are needed THEN it SHALL provide /up endpoint returning 200 OK, include basic system stats, and support Docker health check format
6. WHEN graceful shutdown occurs THEN it SHALL drain WebSocket connections, complete in-flight requests, close database connections, and exit cleanly within 30 seconds
7. WHEN storage is managed THEN it SHALL use /rails/storage volume mount with subdirectories for database (db/), blobs (blobs/), and maintain file structure compatibility
8. WHEN background jobs are processed THEN it SHALL use embedded task queue (tokio tasks) instead of Redis, handle webhook delivery and push notifications internally
9. WHEN logging occurs THEN it SHALL use structured JSON logging compatible with existing log aggregation, include request IDs, and support log level configuration
10. WHEN metrics are exposed THEN it SHALL provide Prometheus-compatible /metrics endpoint with HTTP request metrics, WebSocket connection counts, and system resource usage

### Requirement 10: Security Implementation and Content Protection

**User Story:** As a security-conscious user, I want comprehensive security measures including authentication, content sanitization, and network protection, so that my data remains safe and the system is protected from attacks.

#### Acceptance Criteria

1. WHEN password authentication occurs THEN it SHALL use bcrypt with proper cost factor, secure session tokens via has_secure_token equivalent, and httponly SameSite=Lax cookies
2. WHEN rate limiting is applied THEN it SHALL limit login attempts to 10 per 3 minutes per IP, implement exponential backoff, and log security events
3. WHEN content is sanitized THEN it SHALL use HTML sanitization equivalent to Rails sanitize helper, strip dangerous tags, and prevent XSS in rich text content
4. WHEN file uploads are processed THEN it SHALL validate MIME types, enforce size limits (5MB for OpenGraph), scan for malicious content, and use secure blob storage
5. WHEN OpenGraph fetching occurs THEN it SHALL use RestrictedHTTP::PrivateNetworkGuard to prevent SSRF attacks, limit redirects to 10, and validate URLs
6. WHEN CSRF protection is implemented THEN it SHALL generate and validate CSRF tokens for web forms, skip for bot API requests, and use secure token generation
7. WHEN database queries are executed THEN it SHALL use parameterized queries/prepared statements, validate all inputs, and prevent SQL injection attacks
8. WHEN WebSocket connections are managed THEN it SHALL authenticate via session cookies, validate room access permissions, and prevent unauthorized subscriptions
9. WHEN sensitive data is handled THEN it SHALL log security events, mask sensitive data in logs, and implement proper access controls for user data
10. WHEN network requests are made THEN it SHALL validate SSL certificates, use proper timeouts (7 seconds for webhooks), and handle network errors securely

### Requirement 11: Advanced Features and Progressive Web App

**User Story:** As a user, I want all advanced features including PWA support, push notifications, QR codes, search functionality, and OpenGraph link unfurling to work identically, so that I have the complete Campfire experience.

#### Acceptance Criteria

1. WHEN PWA functionality is accessed THEN it SHALL serve /webmanifest with app metadata and /service-worker.js for offline support and push notification handling
2. WHEN push notifications are managed THEN it SHALL use WebPush gem equivalent with VAPID keys, thread pool (50 threads, 150 HTTP connections), and proper subscription lifecycle
3. WHEN push payloads are delivered THEN it SHALL include title, body, path, badge (unread count), icon (account logo), and handle subscription expiration/invalidation
4. WHEN QR codes are generated THEN it SHALL create cacheable SVG QR codes for room sharing and session transfer with proper Base64 URL encoding
5. WHEN search functionality is used THEN it SHALL maintain user search history (limit 10), provide FTS5 full-text search with Porter stemming, and highlight results
6. WHEN OpenGraph unfurling occurs THEN it SHALL fetch metadata with security restrictions, validate image URLs, sanitize content, and cache results appropriately
7. WHEN sound effects are played THEN it SHALL support all 50+ sounds with proper asset serving, recognize /play commands, and display appropriate text/image responses
8. WHEN session transfers occur THEN it SHALL generate secure transfer tokens, support cross-device authentication, and maintain security during transfers
9. WHEN custom account styles are applied THEN it SHALL sanitize CSS content, apply custom styles safely, and maintain interface integrity
10. WHEN notification permissions are managed THEN it SHALL handle browser permission states (granted/denied/default), show appropriate UI feedback, and manage subscription lifecycle

### Requirement 12: Account Management and Administrative Features

**User Story:** As an account administrator, I want comprehensive account management with branding, user administration, and system configuration, so that I can fully customize and control the Campfire instance.

#### Acceptance Criteria

1. WHEN accounts are managed THEN it SHALL support single-tenant architecture with account name, join_code (regeneratable), custom_styles text field, and logo attachment
2. WHEN first run occurs THEN it SHALL create default account "Campfire" with first room "All Talk", set first user as administrator, and establish proper memberships
3. WHEN join codes are used THEN it SHALL validate against Current.account.join_code, allow regeneration by administrators, and control new user registration access
4. WHEN account logos are handled THEN it SHALL use has_one_attached :logo, process with VIPS-compatible image handling, serve with caching headers and fresh_account_logo helper
5. WHEN custom styles are applied THEN it SHALL store CSS in custom_styles field, sanitize for security, apply to interface, and allow administrator-only editing
6. WHEN user administration occurs THEN it SHALL support user listing with pagination (500 per page), role management, activation/deactivation, and profile editing by administrators
7. WHEN account settings change THEN it SHALL broadcast updates via Turbo Streams, update cached account data, and reflect changes across all connected clients
8. WHEN account branding is displayed THEN it SHALL show account name in interface, use custom logo in notifications and PWA manifest, and apply custom styles globally
9. WHEN administrative permissions are checked THEN it SHALL use ensure_can_administer before_action, verify Current.user.can_administer?, and restrict sensitive operations
10. WHEN account data is accessed THEN it SHALL use Current.account pattern, maintain account context throughout requests, and ensure proper tenant isolation

### Requirement 13: File Upload and Attachment Processing

**User Story:** As a user, I want comprehensive file upload capabilities with drag-and-drop, progress tracking, and thumbnail generation, so that I can share files seamlessly in conversations.

#### Acceptance Criteria

1. WHEN files are uploaded THEN the system SHALL support drag-and-drop via DropTargetController, paste from clipboard, and file picker with progress tracking
2. WHEN file uploads are processed THEN it SHALL use FileUploader with XMLHttpRequest, track upload progress, and provide visual feedback with percentage completion
3. WHEN attachments are stored THEN it SHALL use Active Storage compatible blob storage with key, filename, content_type, metadata, service_name, byte_size, and checksum fields
4. WHEN image/video thumbnails are generated THEN it SHALL create variants with resize_to_limit [1200, 800], process video previews as WebP, and handle representable files
5. WHEN file attachments are displayed THEN it SHALL show appropriate previews, support lightbox viewing, and provide download/share functionality
6. WHEN file validation occurs THEN it SHALL check MIME types, enforce size limits, validate file extensions, and prevent malicious uploads
7. WHEN attachment URLs are served THEN it SHALL use signed URLs for security, proper caching headers, and direct blob serving
8. WHEN file processing fails THEN it SHALL handle errors gracefully, show appropriate error messages, and allow retry functionality

### Requirement 14: Advanced UI Components and Interactions

**User Story:** As a user, I want sophisticated UI components including autocomplete, lightbox, modals, and keyboard navigation, so that I have a polished and efficient user experience.

#### Acceptance Criteria

1. WHEN autocomplete is used THEN it SHALL implement AutocompleteHandler with 300ms debounce, fuzzy search, keyboard navigation (up/down/enter/escape), and selection management
2. WHEN lightbox is displayed THEN it SHALL support image/video viewing, keyboard navigation (left/right/escape), zoom functionality, and proper modal behavior
3. WHEN modals are shown THEN it SHALL implement proper focus management, escape key handling, click-outside-to-close, and accessibility features
4. WHEN keyboard shortcuts are used THEN it SHALL support Enter/Cmd+Enter for sending, up arrow for editing last message, escape for closing, and tab navigation
5. WHEN responsive design is applied THEN it SHALL adapt to mobile devices, handle soft keyboard resizing, support touch gestures, and maintain usability
6. WHEN animations are used THEN it SHALL implement smooth transitions, loading states, hover effects, and proper performance optimization
7. WHEN accessibility is implemented THEN it SHALL support screen readers, keyboard navigation, proper ARIA labels, and semantic HTML structure
8. WHEN error states are handled THEN it SHALL show appropriate error messages, loading states, retry functionality, and graceful degradation

### Requirement 15: Background Processing and Job Queue

**User Story:** As a system operator, I want efficient background processing for webhooks, push notifications, and file processing without external dependencies, so that the system remains self-contained and performant.

#### Acceptance Criteria

1. WHEN background jobs are processed THEN it SHALL use embedded task queue with tokio tasks instead of Redis, handle job failures, and provide retry logic
2. WHEN webhook delivery occurs THEN it SHALL use Bot::WebhookJob equivalent with 7-second timeout, proper error handling, and response processing
3. WHEN push notifications are sent THEN it SHALL use Room::PushMessageJob equivalent with thread pool (50 threads), batch processing, and subscription management
4. WHEN file processing happens THEN it SHALL generate thumbnails asynchronously, process video previews, and handle large file uploads without blocking
5. WHEN job scheduling is needed THEN it SHALL support delayed execution, periodic tasks, and proper job prioritization
6. WHEN job monitoring is required THEN it SHALL provide job status tracking, failure logging, and performance metrics
7. WHEN system shutdown occurs THEN it SHALL gracefully complete running jobs, drain job queues, and handle cleanup properly
8. WHEN job persistence is needed THEN it SHALL store job state in SQLite, handle job recovery after restart, and maintain job history

### Requirement 16: Frontend JavaScript Architecture and Complex UI Models

**User Story:** As a user, I want sophisticated client-side functionality including message formatting, scroll management, and real-time interactions that replicate the existing Stimulus controller behavior exactly, so that the interface feels identical to the current implementation.

#### Acceptance Criteria

1. WHEN message formatting occurs THEN it SHALL implement MessageFormatter with 5-minute threading windows, emoji detection using Unicode regex, mention highlighting with @username parsing, and first-of-day detection with separators
2. WHEN message pagination happens THEN it SHALL implement MessagePaginator with intersection observers for auto-loading, excess message trimming (performance optimization), scroll position management, and loading state indicators
3. WHEN scroll management is active THEN it SHALL implement ScrollManager with auto-scroll threshold (100px from bottom), keep-scroll positioning during updates, pending operation queuing, and smooth scrolling behavior
4. WHEN client messages are rendered THEN it SHALL implement ClientMessage with template rendering, state management (pending/sent/failed), UUID generation for client_message_id, and optimistic updates
5. WHEN file uploads occur THEN it SHALL implement FileUploader with XMLHttpRequest progress tracking, visual feedback with percentage completion, drag-and-drop support, and error recovery mechanisms
6. WHEN message threading is displayed THEN it SHALL group messages within 5-minute windows, show thread style with reduced avatars, calculate proper sort values, and handle thread breaks with separators
7. WHEN emoji-only messages are detected THEN it SHALL apply message--emoji CSS class using Unicode emoji detection regex and enlarge display appropriately
8. WHEN code blocks are processed THEN it SHALL apply syntax highlighting using highlight.js, detect language from content, and maintain proper formatting
9. WHEN mentions are processed THEN it SHALL highlight @username patterns, link to user profiles, and trigger notification logic for mentioned users
10. WHEN message positioning occurs THEN it SHALL calculate proper sort values, handle message insertion at correct positions, and maintain chronological order with threading

### Requirement 17: Content Processing and Security Pipeline

**User Story:** As a security-conscious administrator, I want comprehensive content filtering and sanitization that prevents XSS attacks and malicious content while preserving rich text functionality, so that the system remains secure against content-based attacks.

#### Acceptance Criteria

1. WHEN ActionText content is processed THEN it SHALL implement ContentFilters::RemoveSoloUnfurledLinkText to clean up link previews, maintain content integrity, and preserve user-intended formatting
2. WHEN social media content is processed THEN it SHALL implement ContentFilters::StyleUnfurledTwitterAvatars for proper avatar styling, handle Twitter/X embed formatting, and maintain visual consistency
3. WHEN HTML content is sanitized THEN it SHALL implement ContentFilters::SanitizeTags to strip dangerous HTML tags, preserve safe formatting elements, and prevent XSS attacks
4. WHEN rich text is processed THEN it SHALL use ActionText-compatible filtering pipeline, maintain HTML body integrity, and support Trix editor formatting requirements
5. WHEN user input is validated THEN it SHALL sanitize all HTML input using allowlist-based filtering, escape dangerous characters, and validate against known attack patterns
6. WHEN content is stored THEN it SHALL validate content length limits, check for malicious patterns, and maintain referential integrity with attachments
7. WHEN content is displayed THEN it SHALL apply final sanitization pass, ensure safe rendering, and prevent client-side script execution
8. WHEN link content is processed THEN it SHALL validate URLs for safety, prevent javascript: and data: schemes, and apply proper link formatting
9. WHEN file attachments are processed THEN it SHALL validate MIME types against allowlist, scan for embedded scripts, and ensure safe file serving
10. WHEN content filtering fails THEN it SHALL log security events, fall back to safe defaults, and notify administrators of potential threats

### Requirement 18: Advanced OpenGraph Implementation with Security

**User Story:** As a user, I want automatic link previews with rich metadata that are fetched securely without exposing the system to SSRF attacks, so that I can share links with rich context while maintaining system security.

#### Acceptance Criteria

1. WHEN OpenGraph fetching occurs THEN it SHALL implement RestrictedHTTP::PrivateNetworkGuard to prevent SSRF attacks against internal networks, localhost, and private IP ranges
2. WHEN URLs are validated THEN it SHALL check against private network ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16), localhost (127.0.0.1, ::1), and link-local addresses
3. WHEN HTTP requests are made THEN it SHALL enforce 7-second timeout, limit redirects to 10 maximum, validate SSL certificates, and handle network errors gracefully
4. WHEN metadata is extracted THEN it SHALL use Nokogiri-compatible HTML parsing, extract title/description/image safely, validate content length limits, and sanitize extracted text
5. WHEN images are processed THEN it SHALL validate image URLs for safety, enforce size limits (5MB maximum), check MIME types, and generate secure cached versions
6. WHEN document parsing occurs THEN it SHALL handle malformed HTML gracefully, prevent XML external entity attacks, limit parsing time, and validate character encoding
7. WHEN OpenGraph data is cached THEN it SHALL store metadata with expiration times, validate cached content on retrieval, and handle cache invalidation properly
8. WHEN network restrictions are bypassed THEN it SHALL log security events, block the request, and notify administrators of attempted SSRF attacks
9. WHEN OpenGraph processing fails THEN it SHALL fall back gracefully without breaking message display, log errors appropriately, and provide user feedback
10. WHEN rate limiting is applied THEN it SHALL limit OpenGraph requests per user/IP, implement exponential backoff for failures, and prevent abuse

### Requirement 19: Advanced Membership Management and Presence Tracking

**User Story:** As a user, I want sophisticated membership management with real-time presence tracking, involvement levels, and batch operations that scale efficiently with large user bases, so that I can manage room access and see who's active.

#### Acceptance Criteria

1. WHEN presence is tracked THEN it SHALL implement connection counting with atomic increment/decrement, track connected_at timestamps with 60-second TTL, and support connected/disconnected scopes efficiently
2. WHEN involvement levels are managed THEN it SHALL support invisible (no notifications), nothing (room access only), mentions (mention notifications), everything (all notifications) with proper filtering and scoping
3. WHEN batch operations occur THEN it SHALL implement grant_to and revoke_from for multiple users, revise for bulk membership changes, and handle large user sets (10K+) efficiently
4. WHEN membership changes THEN it SHALL reset remote connections on membership destruction, broadcast presence updates via Turbo Streams, and maintain connection state consistency
5. WHEN direct rooms are managed THEN it SHALL use Set comparison of user_ids for singleton behavior, optimize for large room counts, and handle membership efficiently
6. WHEN room types are processed THEN it SHALL auto-grant open room memberships to new users, restrict closed room access to explicit invites, and manage direct room creation automatically
7. WHEN presence refreshes occur THEN it SHALL send refresh actions every 50 seconds, maintain connection heartbeat, and handle connection timeouts gracefully
8. WHEN visibility changes THEN it SHALL implement 5-second delay using VISIBILITY_CHANGE_DELAY, track wasVisible state, and send present/absent actions appropriately
9. WHEN performance optimization is needed THEN it SHALL use efficient SQL queries with proper indexing, implement connection pooling, and optimize for concurrent access
10. WHEN membership data is accessed THEN it SHALL provide fast lookups for user permissions, cache frequently accessed data, and maintain data consistency across connections

### Requirement 20: Message Threading and Positioning Logic

**User Story:** As a user, I want messages to be grouped and positioned intelligently with proper threading, timestamps, and visual organization that makes conversations easy to follow, so that I can understand message flow and context.

#### Acceptance Criteria

1. WHEN message threading occurs THEN it SHALL group messages within 5-minute windows using precise timing calculations, handle timezone differences, and maintain thread integrity
2. WHEN thread styles are applied THEN it SHALL reduce avatar display for subsequent messages in thread, apply proper CSS classes, and maintain visual hierarchy
3. WHEN sort values are calculated THEN it SHALL generate proper message positioning values, handle concurrent message insertion, and maintain chronological order
4. WHEN first-of-day detection occurs THEN it SHALL identify day boundaries accurately, insert date separators, and handle timezone conversions properly
5. WHEN message positioning happens THEN it SHALL calculate insertion points efficiently, handle out-of-order message arrival, and maintain scroll position during updates
6. WHEN thread breaks occur THEN it SHALL detect thread interruptions (time gaps, different users), insert appropriate separators, and reset thread styling
7. WHEN message updates occur THEN it SHALL maintain thread relationships during edits, handle message deletion gracefully, and update positioning as needed
8. WHEN concurrent messages arrive THEN it SHALL handle race conditions in positioning, maintain thread consistency, and resolve conflicts deterministically
9. WHEN performance optimization is needed THEN it SHALL use efficient algorithms for position calculation, cache thread state, and minimize DOM updates
10. WHEN thread display changes THEN it SHALL animate transitions smoothly, maintain user scroll position, and provide visual feedback for thread changes

### Requirement 16: Caching and Performance Optimization

**User Story:** As a system operator, I want intelligent caching and performance optimization throughout the system, so that response times are minimized and resource usage is efficient.

#### Acceptance Criteria

1. WHEN HTTP responses are cached THEN it SHALL use proper cache headers, ETags for conditional requests, and cache invalidation strategies
2. WHEN database queries are optimized THEN it SHALL use prepared statements, connection pooling, query result caching, and efficient indexing
3. WHEN static assets are served THEN it SHALL use embedded assets with compression, proper cache headers, and efficient serving
4. WHEN WebSocket messages are broadcast THEN it SHALL optimize message serialization, batch broadcasts, and minimize memory allocations
5. WHEN search results are cached THEN it SHALL cache FTS5 query results, implement cache invalidation, and optimize search performance
6. WHEN user sessions are managed THEN it SHALL cache user data, optimize session lookups, and minimize database queries
7. WHEN file serving occurs THEN it SHALL use efficient streaming, range request support, and proper content delivery
8. WHEN memory usage is optimized THEN it SHALL implement efficient data structures, minimize allocations, and use zero-copy operations where possible

### Requirement 17: Advanced UI Components and Interactions (Detailed)

**User Story:** As a user, I want sophisticated UI interactions including lightbox viewing, drag-and-drop uploads, popup menus, and reply functionality, so that I have a polished and intuitive interface.

#### Acceptance Criteria

1. WHEN lightbox is used THEN it SHALL implement LightboxController with modal dialog, image/video display, download/share buttons, backdrop blur, and proper keyboard navigation
2. WHEN drag-and-drop occurs THEN it SHALL use DropTargetController with dragenter/dragover/drop events, visual feedback, and file validation
3. WHEN popup menus are shown THEN it SHALL implement PopupController with orientation detection (90px bottom threshold), click-outside-to-close, and proper positioning
4. WHEN file uploads are previewed THEN it SHALL use UploadPreviewController with URL.createObjectURL, proper cleanup, and thumbnail generation
5. WHEN reply functionality is used THEN it SHALL implement ReplyController with blockquote formatting, mention stripping, unfurled link handling, and composer integration
6. WHEN scroll is maintained THEN it SHALL use MaintainScrollController with ScrollManager integration, above-fold detection, and proper stream rendering
7. WHEN PWA installation is handled THEN it SHALL use PwaInstallController with beforeinstallprompt event, deferred prompt, and installation detection
8. WHEN sound effects are played THEN it SHALL use SoundController with Audio API, proper asset loading, and 50+ sound file support
9. WHEN UI elements are toggled THEN it SHALL implement ToggleClassController for sidebar, modals, and other interactive elements
10. WHEN forms are auto-submitted THEN it SHALL use AutoSubmitController with proper debouncing and validation

### Requirement 18: Complete Styling System Implementation

**User Story:** As a user, I want pixel-perfect visual design with comprehensive CSS styling including animations, responsive design, and accessibility features, so that the interface is beautiful and usable.

#### Acceptance Criteria

1. WHEN lightbox styling is applied THEN it SHALL use backdrop-filter blur (66px), full viewport coverage, grid layout, and proper button positioning
2. WHEN navigation is styled THEN it SHALL implement responsive nav with sidebar width calculations, account logo positioning, and proper z-index layering
3. WHEN sidebar is displayed THEN it SHALL use backdrop-filter blur (12px), fixed positioning, responsive width, and unread badge indicators
4. WHEN buttons are styled THEN it SHALL support multiple variants (reversed, borderless, negative), proper sizing (2.65em), and icon integration
5. WHEN animations are applied THEN it SHALL include pulsing outlines, smooth transitions, hover effects, and loading states
6. WHEN responsive design is implemented THEN it SHALL adapt to mobile (max-width: 100ch), handle soft keyboards, and maintain usability
7. WHEN color schemes are supported THEN it SHALL implement light/dark mode with proper CSS custom properties and media queries
8. WHEN accessibility is ensured THEN it SHALL include proper focus indicators, screen reader support, and keyboard navigation
9. WHEN layout is structured THEN it SHALL use CSS Grid and Flexbox appropriately, proper spacing variables, and semantic HTML
10. WHEN custom styles are applied THEN it SHALL safely inject account custom CSS while maintaining security and interface integrity

### Requirement 19: Form Handling and Input Management

**User Story:** As a user, I want comprehensive form handling with validation, rich text editing, file uploads, and autocomplete functionality, so that data entry is smooth and error-free.

#### Acceptance Criteria

1. WHEN rich text editing is used THEN it SHALL implement Trix editor integration with ActionText compatibility, toolbar controls, and proper serialization
2. WHEN autocomplete is implemented THEN it SHALL support user mentions with fuzzy search, keyboard navigation, avatar display, and selection management
3. WHEN file uploads are handled THEN it SHALL support multiple files, drag-and-drop, paste from clipboard, progress tracking, and thumbnail previews
4. WHEN form validation occurs THEN it SHALL provide real-time feedback, error highlighting, and proper accessibility announcements
5. WHEN composer functionality is used THEN it SHALL support rich text, file attachments, keyboard shortcuts, and toolbar toggling
6. WHEN form submission happens THEN it SHALL handle loading states, error recovery, and proper user feedback
7. WHEN input focus is managed THEN it SHALL maintain proper tab order, focus trapping in modals, and keyboard accessibility
8. WHEN form data is processed THEN it SHALL handle CSRF protection, proper encoding, and secure transmission

### Requirement 20: Helper Functions and Utility Systems

**User Story:** As a developer, I want comprehensive helper functions for UI generation, data formatting, and common operations, so that the React implementation can replicate Rails helper functionality.

#### Acceptance Criteria

1. WHEN page metadata is generated THEN it SHALL create proper title tags, current user meta tags, VAPID keys, and custom styles injection
2. WHEN message rendering occurs THEN it SHALL generate proper data attributes, CSS classes, timestamps, and controller bindings
3. WHEN room links are created THEN it SHALL include proper data attributes for sorting, badges, and room identification
4. WHEN user avatars are displayed THEN it SHALL handle signed URLs, proper sizing, fallbacks, and caching
5. WHEN timestamps are formatted THEN it SHALL use proper localization, relative time display, and timezone handling
6. WHEN navigation elements are generated THEN it SHALL create proper links, buttons, icons, and accessibility attributes
7. WHEN form helpers are used THEN it SHALL generate proper form tags, input elements, validation attributes, and CSRF tokens
8. WHEN utility functions are implemented THEN it SHALL handle string manipulation, URL generation, and data transformation

### Requirement 17: Idiomatic Rust Development Process

**User Story:** As a developer, I want the Rust implementation to follow a structured three-layer idiomatic approach with comprehensive pattern documentation, so that the codebase is maintainable, safe, and follows Rust best practices.

#### Acceptance Criteria

1. WHEN implementing core logic THEN the system SHALL use L1 (Core/no_std) patterns with Result<T,E> for error handling, Option<T> for optional data, and zero unsafe code blocks
2. WHEN using standard library features THEN the system SHALL apply L2 (std) idioms including RAII for resource management, iterator chains over manual loops, and proper borrowing with &str and &[T]
3. WHEN integrating external crates THEN the system SHALL follow L3 (ecosystem) patterns using Axum extractors, Serde derive macros, and async/await with tokio::spawn_blocking for CPU-bound tasks
4. WHEN handling errors THEN the system SHALL use ? operator for propagation, implement From/Into traits for conversions, and avoid .unwrap()/.expect() in production code
5. WHEN managing state THEN the system SHALL avoid global mutable variables, use Arc<Mutex<T>> for shared state, and pass dependencies via function parameters
6. WHEN writing async code THEN the system SHALL never block the event loop, use structured concurrency with JoinSet, and implement proper timeout and backpressure strategies
7. WHEN implementing traits THEN the system SHALL leverage type system for compile-time guarantees, make invalid states unrepresentable, and use builder patterns for complex configuration
8. WHEN following anti-patterns THEN the system SHALL avoid unnecessary clones, mixing async runtimes, and reinventing functionality provided by well-tested crates
9. WHEN documenting patterns THEN the system SHALL maintain SIS (Structured Idiom Schema) entries with context, solution snippets, rationale, and anti-pattern examples
10. WHEN validating code quality THEN the system SHALL pass cargo clippy with zero warnings, use rustfmt for consistent formatting, and achieve compile-first success

### Requirement 18: LLM-Assisted Development Workflow

**User Story:** As a development team, I want to use AI coding agents with structured prompts and validation loops to accelerate the Rails-to-Rust migration while ensuring code quality, so that we can achieve faster development with fewer bugs.

#### Acceptance Criteria

1. WHEN using LLM for code generation THEN the system SHALL provide context-rich prompts including Rails code, architectural mapping guidance, and idiomatic constraints
2. WHEN generating Rust modules THEN the LLM SHALL produce complete, compilable code with proper use statements, error handling, and documentation comments
3. WHEN validating LLM output THEN the system SHALL immediately compile with cargo check, resolve errors through iterative feedback, and achieve zero compile errors
4. WHEN applying code quality checks THEN the system SHALL run cargo clippy with -D warnings, fix all linting issues, and ensure idiomatic patterns are followed
5. WHEN testing generated code THEN the system SHALL validate behavior through unit tests, integration tests, or manual verification against Rails functionality
6. WHEN mapping Rails concepts THEN the LLM SHALL convert ActiveRecord to Diesel/SQLx, Rails controllers to Axum handlers, and ERB templates to Askama/Tera
7. WHEN handling complex logic THEN the system SHALL break down large modules into smaller prompts, validate each piece independently, and integrate systematically
8. WHEN encountering errors THEN the system SHALL feed compiler messages back to LLM for correction, maintain error context, and learn from common mistakes
9. WHEN documenting AI decisions THEN the system SHALL record prompt templates, response summaries, and iteration logs in prompts/ directory for traceability
10. WHEN ensuring consistency THEN the system SHALL use standardized system prompts, maintain idiomatic guidelines, and validate against established patterns

### Requirement 19: Advanced Performance Optimization and Resource Efficiency

**User Story:** As a system operator, I want the Rust implementation to achieve dramatic performance improvements with comprehensive monitoring and optimization strategies, so that I can reduce infrastructure costs by 75-90% while improving user experience.

#### Acceptance Criteria

1. WHEN measuring baseline performance THEN the system SHALL achieve <2MB memory footprint at startup vs Rails 50-100MB, with <100ms cold start time
2. WHEN handling concurrent connections THEN the system SHALL support 10,000+ WebSocket connections on single instance vs Rails ~1,000, using async/await efficiently
3. WHEN processing HTTP requests THEN the system SHALL achieve 10-12k requests/second vs Rails few hundred, with <5ms response times for API calls
4. WHEN managing memory usage THEN the system SHALL maintain stable memory consumption, avoid memory leaks, and use efficient data structures with proper capacity allocation
5. WHEN optimizing database operations THEN the system SHALL use connection pooling, prepared statements, and achieve <2ms query times with compile-time SQL validation
6. WHEN handling file operations THEN the system SHALL use async I/O for uploads, stream large files without blocking, and generate thumbnails in background tasks
7. WHEN implementing caching THEN the system SHALL use in-memory caches with Arc<Mutex<T>>, implement proper TTL, and avoid cache stampedes
8. WHEN monitoring performance THEN the system SHALL expose Prometheus metrics, track request latency, memory usage, and connection counts
9. WHEN scaling under load THEN the system SHALL handle traffic spikes gracefully, implement circuit breakers, and maintain low latency under high concurrency
10. WHEN optimizing for cost THEN the system SHALL achieve 5-10x resource efficiency improvement, enabling single instance to replace multiple Rails servers

### Requirement 20: Comprehensive Development Tooling and Governance

**User Story:** As a development team, I want sophisticated tooling for code quality, automated checks, and governance processes, so that we can maintain high standards and continuous improvement throughout the project lifecycle.

#### Acceptance Criteria

1. WHEN setting up development environment THEN the system SHALL provide Rust workspace with proper crate organization, dependency management, and IDE integration
2. WHEN implementing continuous integration THEN the system SHALL run cargo check, cargo test, cargo clippy, and rustfmt on every commit with zero tolerance for warnings
3. WHEN enforcing code quality THEN the system SHALL implement custom lints for idiom compliance, security checks, and performance anti-patterns
4. WHEN managing dependencies THEN the system SHALL use cargo audit for security vulnerabilities, maintain minimal dependency tree, and prefer well-maintained crates
5. WHEN documenting architecture THEN the system SHALL maintain ARCHITECTURE.md with Mermaid diagrams, design decisions, and component relationships
6. WHEN tracking idiom usage THEN the system SHALL maintain IDIOMS_USED.md with pattern references, code locations, and rationale documentation
7. WHEN governing code changes THEN the system SHALL require PR reviews, idiom compliance checks, and architectural alignment validation
8. WHEN evolving patterns THEN the system SHALL use RFC-style process for new idioms, peer review for pattern validation, and versioned idiom archive
9. WHEN onboarding developers THEN the system SHALL provide training materials, idiom documentation, and hands-on workshops for Rust best practices
10. WHEN measuring success THEN the system SHALL track compile-first success rate, bug reduction metrics, and development velocity improvements


### Requirement 21: Advanced UI Components and Interaction Patterns

**User Story:** As a user, I want sophisticated UI components including autocomplete, lightbox, modals, and keyboard navigation that provide a polished and efficient user experience matching the current Stimulus implementation exactly.

#### Acceptance Criteria

1. WHEN autocomplete is used THEN it SHALL implement AutocompleteHandler with 300ms debounce timing, fuzzy user search with ranking, keyboard navigation (up/down/enter/escape), and proper selection management
2. WHEN lightbox is displayed THEN it SHALL support image/video viewing with zoom, keyboard navigation (left/right/escape), proper modal behavior, and accessibility features
3. WHEN modals are shown THEN it SHALL implement focus management with focus trapping, escape key handling, click-outside-to-close behavior, and proper ARIA attributes
4. WHEN keyboard shortcuts are used THEN it SHALL support Enter/Cmd+Enter for sending messages, up arrow for editing last message, escape for closing dialogs, and tab navigation
5. WHEN responsive design is applied THEN it SHALL adapt to mobile devices with touch-friendly controls, handle soft keyboard resizing, support swipe gestures, and maintain usability
6. WHEN animations are used THEN it SHALL implement smooth transitions with CSS transforms, loading states with spinners, hover effects, and performance optimization
7. WHEN accessibility is implemented THEN it SHALL support screen readers with proper ARIA labels, keyboard navigation for all functions, semantic HTML structure, and high contrast support
8. WHEN error states are handled THEN it SHALL show appropriate error messages with retry options, loading states with progress indicators, and graceful degradation for network issues
9. WHEN drag-and-drop is used THEN it SHALL provide visual feedback during drag operations, handle file drops with validation, and support multiple file selection
10. WHEN touch interactions occur THEN it SHALL support touch gestures for mobile devices, handle touch events properly, and provide haptic feedback where appropriate

### Requirement 22: Performance Optimization Patterns and Efficiency

**User Story:** As a system operator, I want the Rust implementation to replicate Rails-specific optimizations while achieving superior performance through modern techniques, so that the system scales efficiently and reduces resource usage.

#### Acceptance Criteria

1. WHEN database queries are executed THEN it SHALL use prepared statements with compile-time validation via Diesel, implement connection pooling with configurable limits, and maintain <2ms query times
2. WHEN SQL optimization is applied THEN it SHALL use proper indexing strategies for FTS5 search, foreign key relationships, and frequently accessed columns with query plan analysis
3. WHEN WebSocket broadcasting occurs THEN it SHALL use efficient message serialization with minimal allocations, batch broadcasts to multiple connections, and implement backpressure handling
4. WHEN memory management is optimized THEN it SHALL use Rust's zero-cost abstractions, minimize heap allocations, implement object pooling for frequently used objects, and maintain <2MB baseline memory
5. WHEN concurrent processing occurs THEN it SHALL use async/await with tokio runtime, handle 10,000+ concurrent connections, and implement proper task scheduling
6. WHEN asset serving is optimized THEN it SHALL embed static assets in binary with zero-copy serving, implement proper caching headers, and compress responses efficiently
7. WHEN real-time updates are processed THEN it SHALL use Turbo Stream optimizations with minimal DOM updates, batch UI changes, and maintain 60fps performance
8. WHEN file processing occurs THEN it SHALL use async image processing with tokio::spawn_blocking, generate thumbnails without blocking, and implement streaming for large files
9. WHEN search operations are performed THEN it SHALL leverage SQLite FTS5 with optimized queries, implement result caching, and achieve sub-millisecond search times
10. WHEN performance monitoring is active THEN it SHALL provide metrics for response times, memory usage, connection counts, and database performance with Prometheus compatibility

### Requirement 23: Sound System and Media Effects

**User Story:** As a user, I want the complete sound system with all 50+ sound effects, proper command parsing, and visual responses that work identically to the current implementation, so that I can use sounds to enhance conversations.

#### Acceptance Criteria

1. WHEN sound commands are parsed THEN it SHALL recognize /play soundname format, support all 50+ predefined sounds (56k, bell, bezos, bueller, etc.), and handle case-insensitive matching
2. WHEN sound responses are generated THEN it SHALL return appropriate text responses for each sound, display associated images where applicable, and maintain response consistency
3. WHEN sound assets are served THEN it SHALL embed all sound files in binary, serve with proper MIME types and caching headers, and support efficient streaming
4. WHEN sound metadata is processed THEN it SHALL use Sound::Image structure with name, width, height dimensions, maintain WebP format compatibility, and handle missing assets gracefully
5. WHEN sound commands are broadcast THEN it SHALL send sound events via WebSocket to all room members, include sound metadata in message content, and handle playback synchronization
6. WHEN sound validation occurs THEN it SHALL validate sound names against predefined list, prevent arbitrary file access, and handle invalid sound requests gracefully
7. WHEN sound history is maintained THEN it SHALL track recently played sounds per room, implement sound usage statistics, and provide sound discovery features
8. WHEN sound permissions are checked THEN it SHALL respect room-level sound settings, handle user preferences for sound playback, and implement sound muting options
9. WHEN sound assets are optimized THEN it SHALL compress sound files efficiently, implement lazy loading for sound assets, and minimize bandwidth usage
10. WHEN sound system fails THEN it SHALL fall back gracefully without breaking message functionality, log sound errors appropriately, and provide user feedback

### Requirement 24: Progressive Web App and Offline Support

**User Story:** As a mobile user, I want full PWA functionality with offline support, push notifications, and native app-like behavior that works seamlessly across devices, so that I can use Campfire like a native application.

#### Acceptance Criteria

1. WHEN PWA manifest is served THEN it SHALL provide /webmanifest with complete app metadata, proper icons for all sizes, theme colors, and display mode configuration
2. WHEN service worker is active THEN it SHALL serve /service-worker.js with offline support, cache management, push notification handling, and background sync capabilities
3. WHEN offline functionality is used THEN it SHALL cache essential resources, provide offline message composition, queue messages for sending when online, and show connection status
4. WHEN push notifications are managed THEN it SHALL use WebPush with VAPID keys, handle subscription lifecycle, manage notification permissions, and provide rich notification content
5. WHEN notification payloads are delivered THEN it SHALL include title, body, path for navigation, badge with unread count, and account logo as icon
6. WHEN app installation is triggered THEN it SHALL provide install prompts, handle beforeinstallprompt events, support add-to-homescreen functionality, and track installation metrics
7. WHEN background sync occurs THEN it SHALL queue failed network requests, retry when connection restored, sync message state, and handle conflict resolution
8. WHEN app updates are available THEN it SHALL notify users of updates, handle service worker updates gracefully, and provide update installation options
9. WHEN native features are accessed THEN it SHALL support file sharing via Web Share API, handle deep links, provide native-like navigation, and integrate with OS notifications
10. WHEN PWA performance is optimized THEN it SHALL achieve fast loading times, smooth animations, efficient caching strategies, and minimal resource usage

### Requirement 25: Advanced Search and Full-Text Capabilities

**User Story:** As a user, I want powerful search functionality with full-text search, search history, result highlighting, and advanced filtering that helps me find messages and content quickly across all my conversations.

#### Acceptance Criteria

1. WHEN FTS5 search is performed THEN it SHALL use SQLite FTS5 virtual table with Porter stemming, support phrase queries with quotes, and handle boolean operators (AND, OR, NOT)
2. WHEN search history is managed THEN it SHALL maintain user search history with limit of 10 recent searches, provide search suggestions, and allow history clearing
3. WHEN search results are displayed THEN it SHALL highlight matching terms in results, show message context with surrounding text, and provide relevance scoring
4. WHEN search indexing occurs THEN it SHALL index message content in real-time, handle rich text content properly, and maintain index consistency during updates
5. WHEN search performance is optimized THEN it SHALL achieve sub-millisecond search times, implement result caching, and handle large message volumes efficiently
6. WHEN search filtering is applied THEN it SHALL support filtering by room, user, date range, and content type with proper query optimization
7. WHEN search pagination occurs THEN it SHALL provide efficient result pagination, maintain search context across pages, and handle large result sets
8. WHEN search syntax is processed THEN it SHALL support advanced query syntax, handle special characters properly, and provide query validation with error messages
9. WHEN search results are navigated THEN it SHALL provide direct links to messages, maintain scroll position in conversations, and highlight target messages
10. WHEN search analytics are tracked THEN it SHALL log search queries for optimization, track search performance metrics, and identify popular search patterns

### Requirement 26: Session Management and Cross-Device Functionality

**User Story:** As a user, I want seamless session management with secure authentication, session transfer between devices, and proper session lifecycle management that keeps me logged in securely across devices.

#### Acceptance Criteria

1. WHEN sessions are created THEN it SHALL generate secure tokens using cryptographically strong randomness, set httponly SameSite=Lax cookies, and track session metadata
2. WHEN session transfer occurs THEN it SHALL generate secure transfer tokens with expiration, support QR code generation for easy transfer, and maintain security during cross-device authentication
3. WHEN session lifecycle is managed THEN it SHALL track last_active_at with 1-hour refresh rate, implement automatic session cleanup, and handle concurrent sessions properly
4. WHEN session security is enforced THEN it SHALL validate session tokens on each request, detect session hijacking attempts, and implement session fixation protection
5. WHEN device tracking occurs THEN it SHALL track IP addresses and user agents, detect suspicious login patterns, and provide session management interface
6. WHEN session expiration occurs THEN it SHALL handle graceful session expiration, provide re-authentication prompts, and maintain user data during re-auth
7. WHEN concurrent sessions are managed THEN it SHALL support multiple active sessions per user, provide session listing and termination, and handle session conflicts
8. WHEN session storage is optimized THEN it SHALL use efficient session storage in SQLite, implement session cleanup jobs, and maintain session performance
9. WHEN session events are logged THEN it SHALL log login/logout events, track session security events, and provide audit trails for administrators
10. WHEN session recovery occurs THEN it SHALL handle browser crashes gracefully, restore session state when possible, and provide session recovery options

### Requirement 27: Account Customization and Branding

**User Story:** As an account administrator, I want comprehensive customization options including custom styles, branding, and account-specific configuration that allows full personalization of the Campfire instance.

#### Acceptance Criteria

1. WHEN custom styles are applied THEN it SHALL store CSS in custom_styles field, sanitize CSS for security, apply styles globally across interface, and validate CSS syntax
2. WHEN account logos are managed THEN it SHALL use has_one_attached :logo pattern, process images with VIPS-compatible handling, serve with caching headers, and support multiple formats
3. WHEN account branding is displayed THEN it SHALL show account name in interface headers, use custom logo in notifications and PWA manifest, and maintain brand consistency
4. WHEN join codes are managed THEN it SHALL allow administrator regeneration, validate codes during registration, and provide secure code distribution methods
5. WHEN account settings are updated THEN it SHALL broadcast changes via Turbo Streams, update cached account data across connections, and validate setting changes
6. WHEN custom CSS is processed THEN it SHALL prevent XSS through CSS injection, validate CSS properties against allowlist, and handle CSS parsing errors gracefully
7. WHEN account themes are applied THEN it SHALL support light/dark theme switching, maintain user theme preferences, and handle theme transitions smoothly
8. WHEN branding assets are optimized THEN it SHALL compress images efficiently, generate multiple sizes for different contexts, and implement lazy loading
9. WHEN account configuration is managed THEN it SHALL provide interface for all account settings, validate configuration changes, and maintain configuration history
10. WHEN multi-tenant isolation is enforced THEN it SHALL ensure complete tenant data separation, prevent cross-tenant data access, and maintain security boundaries

### Requirement 28: Error Handling and System Resilience

**User Story:** As a system operator, I want comprehensive error handling with graceful degradation, proper logging, and system resilience that maintains service availability even during partial failures.

#### Acceptance Criteria

1. WHEN errors occur THEN it SHALL provide user-friendly error messages, log detailed error information for debugging, and implement proper error categorization
2. WHEN network failures happen THEN it SHALL handle connection timeouts gracefully, implement retry logic with exponential backoff, and provide offline functionality
3. WHEN database errors occur THEN it SHALL handle connection failures, implement transaction rollback, provide data consistency guarantees, and recover from corruption
4. WHEN WebSocket connections fail THEN it SHALL implement automatic reconnection, maintain message queue during disconnection, and provide connection status indicators
5. WHEN file operations fail THEN it SHALL handle disk space issues, implement file validation, provide upload retry mechanisms, and clean up failed uploads
6. WHEN memory pressure occurs THEN it SHALL implement memory monitoring, provide garbage collection optimization, and handle out-of-memory conditions gracefully
7. WHEN system overload happens THEN it SHALL implement rate limiting, provide backpressure mechanisms, and maintain service availability under load
8. WHEN security violations occur THEN it SHALL log security events, implement automatic blocking, and provide administrator notifications
9. WHEN data validation fails THEN it SHALL provide clear validation messages, prevent data corruption, and maintain referential integrity
10. WHEN system recovery is needed THEN it SHALL implement health checks, provide automatic restart mechanisms, and maintain service continuity during recovery