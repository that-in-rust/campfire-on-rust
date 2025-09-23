# Campfire Rust

**A working Rust implementation of Basecamp's Campfire chat application.**

## What Works Now

**Text-based chat with real-time features** - A decent approximation of the original Campfire functionality, suitable for production use.

## Quick Start

```bash
git clone https://github.com/that-in-rust/campfire-on-rust.git
cd campfire-on-rust
cargo run
```

Access at `http://localhost:3000` with `admin@example.com` / `password`

## Current Features

```mermaid
graph TD
    subgraph "Working Features"
        direction TB
        A[Real-time Chat<br/>WebSocket messaging]
        B[Room Management<br/>Open/Closed/Direct rooms]
        C[User Authentication<br/>Session-based login]
        D[Message Search<br/>SQLite FTS5]
        E[Push Notifications<br/>Web Push API]
        F[Sound System<br/>59 embedded MP3s]
    end
    
    classDef working fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    class A,B,C,D,E,F working
```

## Not Currently in Scope

- **File attachments** - UI shows "Coming in v2.0" messaging
- **Avatar uploads** - Text initials displayed instead  
- **OpenGraph previews** - Links shown without preview cards

## Troubleshooting

**Database locked error:**
```bash
pkill -f campfire
rm -f campfire.db-wal campfire.db-shm
cargo run
```

**Port 3000 in use:**
```bash
export PORT=3001
cargo run
```

## Interface Previews

View the complete interface at [docs/interface-previews/](docs/interface-previews/):

- [Login Page](docs/interface-previews/01-login-page.html)
- [Main Chat Interface](docs/interface-previews/02-main-chat-interface.html)
- [Room Management](docs/interface-previews/03-room-management.html)
- [Search Interface](docs/interface-previews/04-search-interface.html)
- [User Settings](docs/interface-previews/05-user-settings.html)
- [Mobile Chat](docs/interface-previews/06-mobile-chat.html)
- [Sound System](docs/interface-previews/07-sound-system.html)
- [Push Notifications](docs/interface-previews/08-push-notifications.html)
- [Bot Integration](docs/interface-previews/09-bot-integration.html)
- [Admin Dashboard](docs/interface-previews/10-admin-dashboard.html)
- [Dark Mode](docs/interface-previews/11-dark-mode.html)
- [Error States](docs/interface-previews/12-error-states.html)

## Documentation

- [API Overview](docs/api-overview.md) - REST and WebSocket APIs
- [Deployment Guide](docs/deployment-guide.md) - Production deployment
- [Authentication Guide](docs/authentication-guide.md) - Session management
- [WebSocket Guide](docs/websocket-guide.md) - Real-time features
- [Search Guide](docs/search-guide.md) - Full-text search
- [Bot Integration](docs/bot-integration.md) - API and webhooks

## Acknowledgments

This project exists thanks to **Basecamp**, **DHH**, and **Jason Fried** for open-sourcing the original Campfire application.