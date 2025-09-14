---
inclusion: always
---

# Campfire-on-Rust: Technology Stack Constraints

## REQUIRED STACK

- **Web Framework**: Axum only (Rails-inspired routing and middleware)
- **Database**: SQLite with sqlx (direct operations, Rails-compatible schema)
- **Real-time**: ActionCable-inspired WebSocket broadcasting
- **Frontend**: React 18 with embedded assets
- **Task Queue**: Basic tokio tasks only (webhook delivery, push notifications)
- **Authentication**: Rails-style session management
- **Deployment**: Single binary with embedded assets

## DATABASE RULES

- **SQLite only** - No PostgreSQL, MySQL, or distributed databases
- **Direct SQL with sqlx** - No ORMs, no query builders beyond sqlx macros
- **WAL mode** for basic concurrency
- **FTS5** for search (built into SQLite)
- **Connection pooling** with sqlx Pool
- **Dedicated Writer Task** pattern for write serialization

## FORBIDDEN DEPENDENCIES

- **NO Redis** - Use SQLite for everything
- **NO message queues** (RabbitMQ, Kafka, etc.)
- **NO event stores** or event sourcing libraries
- **NO coordination frameworks** (Akka, Orleans, etc.)
- **NO microservice frameworks**
- **NO complex async runtimes** beyond tokio

## RUST PATTERNS

- **Error handling**: Result<T, E> only, anyhow allowed in tests
- **Async**: tokio runtime, avoid complex async coordination
- **Serialization**: serde for JSON, no complex serialization
- **Newtypes**: UserId, RoomId, MessageId for type safety
- **Modules**: Rails-style organization (models, handlers, services)

## DEPLOYMENT CONSTRAINTS

- **Single binary** with embedded assets using rust-embed
- **No Docker orchestration** - simple container deployment
- **Environment variables** for configuration
- **No service discovery** or load balancing complexity
- **Database in mounted volume** - NEVER in container image

When complex dependencies are suggested, respond with: "This adds unnecessary complexity. The Rails-equivalent approach using our approved stack is..."