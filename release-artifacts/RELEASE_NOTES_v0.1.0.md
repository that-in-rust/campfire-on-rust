# Campfire v0.1.0 - Zero-Friction Chat Application

🔥 **First stable release of Campfire in Rust!**

## What's New

- **Complete Rust rewrite** of Basecamp's Campfire chat application
- **Zero-friction deployment** with single binary and Docker support
- **Real-time messaging** with WebSocket support
- **Rich text features** including @mentions and /play sound commands
- **Full-text search** powered by SQLite FTS5
- **Push notifications** with Web Push API support
- **Bot integration** with API key authentication
- **First-run setup** with admin account creation
- **Demo mode** for easy evaluation

## Quick Start

### 🚀 One-Line Local Install
```bash
curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash
```

### 🚂 One-Click Railway Deployment
[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/template/campfire-rust-v01)

### 🐳 Docker
```bash
docker run -p 3000:3000 -v campfire-data:/app/data campfire-rust:v0.1.0
```

## Features

- ✅ **Real-time chat** with typing indicators and presence
- ✅ **Room management** (Open, Closed, Direct messages)
- ✅ **Rich text messaging** with HTML formatting
- ✅ **Sound system** with 59 embedded MP3 files
- ✅ **@mentions** and user notifications
- ✅ **Full-text search** across all messages
- ✅ **Push notifications** for desktop and mobile
- ✅ **Bot API** for integrations and automation
- ✅ **Session management** with secure authentication
- ✅ **SQLite database** with automatic migrations
- ✅ **Single binary deployment** with embedded assets

## System Requirements

- **Memory**: 64MB RAM minimum
- **Storage**: 100MB disk space
- **Network**: HTTP/HTTPS and WebSocket support
- **Browser**: Modern browser with WebSocket and Push API support

## Performance

- **Startup time**: < 1 second
- **Memory usage**: ~20MB base + ~1MB per active connection
- **Message throughput**: 1000+ messages/second
- **Concurrent users**: 100+ users per instance
- **Database**: SQLite with FTS5 for sub-millisecond search

## Security

- **bcrypt password hashing** with secure session tokens
- **Rate limiting** on all API endpoints
- **Input sanitization** and XSS protection
- **CSRF protection** with secure headers
- **Bot API authentication** with revokable tokens

## Download

Choose your platform:

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | [campfire-on-rust-linux-x86_64](https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/campfire-on-rust-linux-x86_64) |
| Linux | ARM64 | [campfire-on-rust-linux-aarch64](https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/campfire-on-rust-linux-aarch64) |
| macOS | x86_64 | [campfire-on-rust-darwin-x86_64](https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/campfire-on-rust-darwin-x86_64) |
| macOS | ARM64 | [campfire-on-rust-darwin-aarch64](https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/campfire-on-rust-darwin-aarch64) |
| Windows | x86_64 | [campfire-on-rust-windows-x86_64.exe](https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/campfire-on-rust-windows-x86_64.exe) |

## Verification

Verify your download with SHA256 checksums:
```bash
sha256sum -c checksums.txt
```

## What's Next (v0.2)

- 📎 **File attachments** with drag-and-drop upload
- 👤 **Avatar uploads** and user profiles
- 🔗 **OpenGraph previews** for shared links
- 📊 **Analytics dashboard** for room activity
- 🔌 **Plugin system** for custom integrations
- 🌐 **Multi-language support** and internationalization

## Support

- 📖 **Documentation**: [README](https://github.com/that-in-rust/campfire-on-rust#readme)
- 🐛 **Bug Reports**: [Issues](https://github.com/that-in-rust/campfire-on-rust/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/that-in-rust/campfire-on-rust/discussions)
- 📧 **Email**: support@campfire-rust.com

---

**Full Changelog**: https://github.com/that-in-rust/campfire-on-rust/compare/v0.0.1...v0.1.0