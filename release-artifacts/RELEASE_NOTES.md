# 🔥 Campfire v0.1.0 - Zero-Friction Team Chat

**The Rust rewrite of Basecamp's Campfire is here!**

A humble tribute to DHH and Jason Fried's original vision - simple, effective team communication that just works.

## 🎯 What's New

✅ **Complete Rust rewrite** with modern performance and reliability  
✅ **Zero-friction deployment** - from GitHub to working chat in 2-3 minutes  
✅ **Real-time messaging** with WebSocket delivery  
✅ **Full-text search** across all message history  
✅ **@mentions and notifications** system  
✅ **59 fun /play sound commands** for team personality  
✅ **Bot integration** via secure API and webhooks  
✅ **Mobile-responsive design** that works everywhere  
✅ **Single binary deployment** with zero dependencies  

## 🚀 Two Ways to Get Started

### 👀 Try it locally (2 minutes)
```bash
curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash
```
Then visit `http://localhost:3000`

### 🚂 Deploy for your team (3 minutes)
[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/template/campfire-rust)

## ⚡ Performance & Reliability

- **Starts in under 1 second** on modern hardware
- **Uses ~20MB RAM** baseline + ~1MB per active connection
- **Handles 100+ concurrent users** per instance
- **Sub-10ms message delivery** with WebSocket
- **Sub-millisecond search** with SQLite FTS5
- **Zero downtime** with graceful shutdown

## 🛠️ Technical Highlights

- **Built with Rust** for memory safety and performance
- **Axum web framework** for modern async HTTP
- **SQLite database** with automatic migrations
- **Embedded assets** for single-binary deployment
- **WebSocket real-time** communication
- **bcrypt password hashing** and secure sessions
- **Rate limiting** and security hardening
- **Structured logging** and metrics

## 📦 Download Binaries

Choose your platform:

| Platform | Architecture | Download |
|----------|--------------|----------|
| macOS | Apple Silicon (M1/M2) | [campfire-on-rust-darwin-aarch64](https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/campfire-on-rust-darwin-aarch64) |
| macOS | Intel x86_64 | [campfire-on-rust-darwin-x86_64](https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/campfire-on-rust-darwin-x86_64) |
| Linux | x86_64 | [campfire-on-rust-linux-x86_64](https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/campfire-on-rust-linux-x86_64) |
| Linux | ARM64 | [campfire-on-rust-linux-aarch64](https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/campfire-on-rust-linux-aarch64) |
| Windows | x86_64 | [campfire-on-rust-windows-x86_64.exe](https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/campfire-on-rust-windows-x86_64.exe) |

## 🔐 Security & Verification

All binaries are built with GitHub Actions and include SHA256 checksums for verification:

```bash
# Download checksums
curl -L -O https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/checksums.txt

# Verify your download
sha256sum -c checksums.txt
```

## 📋 Manual Installation

1. **Download** the binary for your platform
2. **Make executable**: `chmod +x campfire-on-rust-*`
3. **Run**: `./campfire-on-rust-*`
4. **Open browser**: `http://localhost:3000`
5. **Create admin account** on first run

## 🎮 Demo Mode

Want to see Campfire in action? Enable demo mode:

1. Add `CAMPFIRE_DEMO_MODE=true` to your `.env` file
2. Restart Campfire
3. Explore pre-loaded conversations and features

## 🔧 Configuration

Campfire uses a simple `.env` file for configuration:

```bash
# Basic configuration
CAMPFIRE_DATABASE_URL=sqlite:///path/to/campfire.db
CAMPFIRE_HOST=127.0.0.1
CAMPFIRE_PORT=3000
CAMPFIRE_LOG_LEVEL=info

# Optional: Demo mode
CAMPFIRE_DEMO_MODE=true

# Optional: Push notifications (generate at https://vapidkeys.com/)
CAMPFIRE_VAPID_PUBLIC_KEY=your_public_key
CAMPFIRE_VAPID_PRIVATE_KEY=your_private_key
```

## 🐳 Docker Deployment

```bash
# Run with Docker
docker run -p 3000:3000 -v campfire-data:/app/data campfire-rust:v0.1.0

# Or with docker-compose
curl -O https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/docker-compose.yml
docker-compose up -d
```

## 🤖 Bot Integration

Create bots and integrations with the secure API:

```bash
# Create a bot token
curl -X POST http://localhost:3000/api/bots \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "My Bot", "description": "Helpful bot"}'

# Send messages as bot
curl -X POST http://localhost:3000/api/messages \
  -H "Authorization: Bearer $BOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"room_id": "room-uuid", "content": "Hello from bot!"}'
```

## 🎵 Sound Commands

Bring personality to your team chat with 59 built-in sounds:

```
/play tada        # Celebration sound
/play rimshot     # Ba dum tss
/play nyan        # Nyan cat
/play airhorn     # Get attention
/play crickets    # Awkward silence
```

## 🔍 Search Everything

Powerful full-text search across all messages:

- Search by **content**: `rust programming`
- Search by **user**: `from:alice`
- Search by **room**: `in:general`
- Search by **date**: `after:2024-01-01`
- **Combine filters**: `rust from:alice in:dev after:yesterday`

## 🚨 Troubleshooting

### Installation Issues

**Download fails?**
- Check internet connection
- Try manual download from GitHub releases
- Use VPN if corporate firewall blocks GitHub

**Permission denied?**
- Run: `chmod +x campfire-on-rust-*`
- Check if binary is quarantined (macOS): `xattr -d com.apple.quarantine campfire-on-rust-*`

**Port 3000 in use?**
- Set `CAMPFIRE_PORT=8080` in `.env`
- Or kill process using port 3000: `lsof -ti:3000 | xargs kill`

### Runtime Issues

**Database errors?**
- Check disk space and permissions
- Delete `campfire.db` to reset (loses data)
- Check `CAMPFIRE_DATABASE_URL` in `.env`

**WebSocket connection fails?**
- Check firewall settings
- Ensure WebSocket support in proxy/load balancer
- Try different browser

### Performance Issues

**High memory usage?**
- Check number of active connections
- Restart Campfire periodically
- Consider multiple instances with load balancer

**Slow search?**
- Database may need optimization
- Consider archiving old messages
- Check available disk space

## 🆘 Getting Help

- 📖 **Documentation**: [README](https://github.com/that-in-rust/campfire-on-rust#readme)
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/that-in-rust/campfire-on-rust/issues)
- 💬 **Questions**: [GitHub Discussions](https://github.com/that-in-rust/campfire-on-rust/discussions)
- 📧 **Email**: campfire-support@that-in-rust.dev

## 🗺️ What's Next (v0.2)

- 📎 **File attachments** with drag-and-drop upload
- 👤 **Avatar uploads** and rich user profiles
- 🔗 **Link previews** with OpenGraph support
- 📊 **Analytics dashboard** for room activity
- 🔌 **Plugin system** for custom integrations
- 🌐 **Internationalization** and multi-language support
- 📱 **Native mobile apps** consideration

## 🙏 Acknowledgments

**Campfire** was originally created by **37signals** (now Basecamp) and pioneered simple, effective team communication. This Rust implementation is a humble tribute to **DHH** and **Jason Fried**'s vision.

**Built with ❤️ in Rust** - preserving the Campfire spirit with modern reliability, performance, and the joy of systems programming.

---

**Ready for team chat that actually works?** 🔥

**Full Changelog**: https://github.com/that-in-rust/campfire-on-rust/commits/v0.1.0
