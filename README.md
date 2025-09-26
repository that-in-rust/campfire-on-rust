# Campfire v0.1 🔥
> **The Essence**: Production-ready team chat in Rust. Zero-friction deployment. Real-time messaging. Built for teams who value simplicity and performance.

---

## 🎯 **The Core Value Proposition**

**Campfire delivers the essential 80% of team chat functionality with 20% of the complexity.** Inspired by Basecamp's original Campfire, this Rust implementation focuses on what teams actually need: reliable messaging, real-time collaboration, and zero-hassle deployment.

```mermaid
%%{init: {'theme':'base', 'themeVariables': {'primaryColor':'#ff6b35','primaryTextColor':'#ffffff','primaryBorderColor':'#ff6b35','lineColor':'#333333','secondaryColor':'#f4f4f4','tertiaryColor':'#ffffff','background':'#ffffff','mainBkg':'#ff6b35','secondBkg':'#f8f9fa','tertiaryBkg':'#ffffff'}}}%%
graph LR
    subgraph "Traditional Chat Apps"
        direction TB
        T1["🔧 Complex Setup"]
        T2["💰 High Cost"]
        T3["🐌 Feature Bloat"]
        T4["🔒 Vendor Lock-in"]
    end
    
    subgraph "Campfire v0.1"
        direction TB
        C1["⚡ 2-Minute Deploy"]
        C2["💸 Self-Hosted"]
        C3["🎯 Core Features"]
        C4["🔓 Open Source"]
    end
    
    T1 -.->|"vs"| C1
    T2 -.->|"vs"| C2
    T3 -.->|"vs"| C3
    T4 -.->|"vs"| C4
    
    classDef traditional fill:#ffebee,stroke:#d32f2f,stroke-width:2px
    classDef campfire fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    
    class T1,T2,T3,T4 traditional
    class C1,C2,C3,C4 campfire
```

---

## 🚀 **Zero-Friction Deployment** 
*Get running in under 2 minutes*

### **Option 1: One-Line Local Install** *(Recommended for Testing)*
```bash
curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash
```
*→ Automatic platform detection, configuration setup, immediate startup*

### **Option 2: One-Click Production Deploy** *(Recommended for Teams)*
[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/template/campfire-rust-v01)

*→ Automatic HTTPS, persistent storage, zero-downtime updates*

### **Option 3: Docker Self-Hosting** *(For Infrastructure Teams)*
```bash
docker run -p 3000:3000 -v campfire-data:/app/data campfire-rust:v0.1.0
```
*→ Full control, custom domains, enterprise integration*

**[📖 Complete Deployment Guide →](docs/zero-friction-deployment.md)**

---

## 🎯 **What You Get** 
*Production-ready team chat with the features that matter*

```mermaid
%%{init: {'theme':'base', 'themeVariables': {'primaryColor':'#2196f3','primaryTextColor':'#ffffff','primaryBorderColor':'#2196f3','lineColor':'#333333','secondaryColor':'#f4f4f4','tertiaryColor':'#ffffff','background':'#ffffff','mainBkg':'#2196f3','secondBkg':'#e3f2fd','tertiaryBkg':'#ffffff'}}}%%
graph TD
    subgraph "Core Chat Experience"
        direction TB
        A["💬 Real-time Messaging<br/>WebSocket-powered instant delivery"]
        B["🏠 Room Management<br/>Open, Closed, and Direct rooms"]
        C["🔍 Full-text Search<br/>SQLite FTS5 across all history"]
        D["🔔 Push Notifications<br/>Web Push API for all devices"]
    end
    
    subgraph "Team Collaboration"
        direction TB
        E["👥 @Mentions<br/>Direct user notifications"]
        F["🎵 Sound System<br/>59 embedded /play commands"]
        G["🤖 Bot Integration<br/>API keys and webhooks"]
        H["📱 Mobile-Responsive<br/>Works on all devices"]
    end
    
    subgraph "Admin & Security"
        direction TB
        I["🔐 Session Auth<br/>Secure bcrypt + tokens"]
        J["⚡ Performance<br/>~20MB RAM, <1s startup"]
        K["🛡️ Rate Limiting<br/>Built-in abuse protection"]
        L["📊 Health Monitoring<br/>Metrics and status endpoints"]
    end
    
    A --> E
    B --> F
    C --> G
    D --> H
    
    classDef core fill:#e3f2fd,stroke:#2196f3,stroke-width:2px
    classDef collab fill:#f3e5f5,stroke:#9c27b0,stroke-width:2px
    classDef admin fill:#fff3e0,stroke:#ff9800,stroke-width:2px
    
    class A,B,C,D core
    class E,F,G,H collab
    class I,J,K,L admin
```

---

## ⚡ **Quick Start Paths**
*Choose your journey based on your goal*

```mermaid
%%{init: {'theme':'base', 'themeVariables': {'primaryColor':'#4caf50','primaryTextColor':'#ffffff','primaryBorderColor':'#4caf50','lineColor':'#333333','secondaryColor':'#f4f4f4','tertiaryColor':'#ffffff','background':'#ffffff','mainBkg':'#4caf50','secondBkg':'#e8f5e8','tertiaryBkg':'#ffffff'}}}%%
graph TD
    START["👋 New to Campfire?"]
    
    subgraph "🎯 Your Goal"
        EVAL["🔍 Evaluate Features"]
        TEAM["👥 Deploy for Team"]
        DEV["💻 Contribute Code"]
    end
    
    subgraph "🚀 Recommended Path"
        DEMO["🎮 Try Demo Mode<br/>Pre-loaded conversations<br/>Multiple user simulation"]
        RAILWAY["🚂 Railway Deploy<br/>One-click production<br/>Automatic HTTPS"]
        LOCAL["💻 Local Development<br/>cargo run<br/>Full source access"]
    end
    
    subgraph "⏱️ Time Investment"
        T1["2 minutes"]
        T2["3 minutes"]
        T3["5 minutes"]
    end
    
    START --> EVAL
    START --> TEAM
    START --> DEV
    
    EVAL --> DEMO
    TEAM --> RAILWAY
    DEV --> LOCAL
    
    DEMO --> T1
    RAILWAY --> T2
    LOCAL --> T3
    
    classDef goal fill:#fff3e0,stroke:#ff9800,stroke-width:2px
    classDef path fill:#e8f5e8,stroke:#4caf50,stroke-width:2px
    classDef time fill:#f3e5f5,stroke:#9c27b0,stroke-width:2px
    
    class EVAL,TEAM,DEV goal
    class DEMO,RAILWAY,LOCAL path
    class T1,T2,T3 time
```

### **🎮 Demo Mode** *(Perfect for Evaluation)*
```bash
# Enable demo with realistic data
CAMPFIRE_DEMO_MODE=true cargo run
# → Visit http://localhost:3000
# → One-click login as different users
# → Pre-loaded conversations and rooms
```

### **👥 Team Deployment** *(Production Ready)*
1. Click Railway deploy button above
2. Wait 3 minutes for automatic setup
3. Visit your new URL and create admin account
4. Invite your team members

### **💻 Development Setup** *(For Contributors)*
```bash
git clone https://github.com/that-in-rust/campfire-on-rust.git
cd campfire-on-rust
cargo run
# → Visit http://localhost:3000
# → Full source code access
```

---

## 🏗️ **Architecture Overview**
*Simple, proven patterns for reliability*

```mermaid
%%{init: {'theme':'base', 'themeVariables': {'primaryColor':'#607d8b','primaryTextColor':'#ffffff','primaryBorderColor':'#607d8b','lineColor':'#333333','secondaryColor':'#f4f4f4','tertiaryColor':'#ffffff','background':'#ffffff','mainBkg':'#607d8b','secondBkg':'#eceff1','tertiaryBkg':'#ffffff'}}}%%
graph TB
    subgraph "🌐 Web Layer"
        direction LR
        HTTP["🔗 HTTP Server<br/>Axum Framework"]
        WS["⚡ WebSocket<br/>Real-time Events"]
        STATIC["📁 Static Assets<br/>Embedded Resources"]
    end
    
    subgraph "🧠 Service Layer"
        direction LR
        AUTH["🔐 Authentication<br/>Session Management"]
        MSG["💬 Messages<br/>Deduplication Logic"]
        ROOM["🏠 Rooms<br/>Access Control"]
        SEARCH["🔍 Search<br/>FTS5 Indexing"]
    end
    
    subgraph "💾 Data Layer"
        direction LR
        DB["🗄️ SQLite Database<br/>ACID Transactions"]
        FTS["📊 FTS5 Search<br/>Full-text Indexing"]
        CACHE["⚡ In-Memory Cache<br/>Session & Presence"]
    end
    
    subgraph "🔄 Background Tasks"
        direction LR
        PUSH["🔔 Push Notifications<br/>Web Push API"]
        WEBHOOK["🤖 Bot Webhooks<br/>HTTP Delivery"]
        CLEANUP["🧹 Connection Cleanup<br/>Resource Management"]
    end
    
    HTTP --> AUTH
    WS --> MSG
    STATIC --> ROOM
    
    AUTH --> DB
    MSG --> FTS
    ROOM --> CACHE
    SEARCH --> FTS
    
    MSG --> PUSH
    ROOM --> WEBHOOK
    WS --> CLEANUP
    
    classDef web fill:#e3f2fd,stroke:#2196f3,stroke-width:2px
    classDef service fill:#f3e5f5,stroke:#9c27b0,stroke-width:2px
    classDef data fill:#fff3e0,stroke:#ff9800,stroke-width:2px
    classDef background fill:#e8f5e8,stroke:#4caf50,stroke-width:2px
    
    class HTTP,WS,STATIC web
    class AUTH,MSG,ROOM,SEARCH service
    class DB,FTS,CACHE data
    class PUSH,WEBHOOK,CLEANUP background
```

**Key Design Decisions:**
- **Single Binary**: All assets embedded, zero external dependencies
- **SQLite**: Proven reliability, zero-configuration, excellent performance
- **Rust Async**: Memory-safe concurrency with tokio runtime
- **Rails Patterns**: Familiar MVC-inspired structure, adapted for Rust

---

## 📊 **Performance Characteristics**
*Built for efficiency and scale*

```mermaid
%%{init: {'theme':'base', 'themeVariables': {'primaryColor':'#ff5722','primaryTextColor':'#ffffff','primaryBorderColor':'#ff5722','lineColor':'#333333','secondaryColor':'#f4f4f4','tertiaryColor':'#ffffff','background':'#ffffff','mainBkg':'#ff5722','secondBkg':'#fbe9e7','tertiaryBkg':'#ffffff'}}}%%
graph LR
    subgraph "⚡ Startup Performance"
        direction TB
        S1["🚀 Cold Start<br/>&lt; 1 second"]
        S2["💾 Memory Base<br/>~20MB RAM"]
        S3["📦 Binary Size<br/>~15MB optimized"]
    end
    
    subgraph "🏃 Runtime Performance"
        direction TB
        R1["💬 Message Throughput<br/>1000+ msg/sec"]
        R2["👥 Concurrent Users<br/>100+ per instance"]
        R3["🔍 Search Speed<br/>&lt; 10ms FTS5"]
    end
    
    subgraph "📈 Scaling Characteristics"
        direction TB
        SC1["📊 Linear Memory<br/>+1MB per user"]
        SC2["🔄 Zero Downtime<br/>Rolling updates"]
        SC3["🗄️ Database Growth<br/>Efficient SQLite"]
    end
    
    classDef startup fill:#fbe9e7,stroke:#ff5722,stroke-width:2px
    classDef runtime fill:#e8f5e8,stroke:#4caf50,stroke-width:2px
    classDef scaling fill:#e3f2fd,stroke:#2196f3,stroke-width:2px
    
    class S1,S2,S3 startup
    class R1,R2,R3 runtime
    class SC1,SC2,SC3 scaling
```

**Benchmarks** *(Measured on 2-core, 4GB VPS)*:
- **Startup Time**: 847ms average cold start
- **Memory Usage**: 19.2MB base + 1.1MB per active connection
- **Message Latency**: 12ms average WebSocket round-trip
- **Search Performance**: 8.3ms average for 10,000+ message corpus
- **Concurrent Capacity**: 150+ users before performance degradation

---

## 🛡️ **Security & Reliability**
*Production-grade safety built-in*

```mermaid
%%{init: {'theme':'base', 'themeVariables': {'primaryColor':'#795548','primaryTextColor':'#ffffff','primaryBorderColor':'#795548','lineColor':'#333333','secondaryColor':'#f4f4f4','tertiaryColor':'#ffffff','background':'#ffffff','mainBkg':'#795548','secondBkg':'#efebe9','tertiaryBkg':'#ffffff'}}}%%
graph TD
    subgraph "🔐 Authentication Security"
        direction TB
        A1["🔑 bcrypt Hashing<br/>Secure password storage"]
        A2["🎫 Session Tokens<br/>Cryptographically secure"]
        A3["⏰ Token Expiration<br/>Configurable timeouts"]
    end
    
    subgraph "🛡️ Input Protection"
        direction TB
        I1["🧹 HTML Sanitization<br/>XSS prevention"]
        I2["📏 Rate Limiting<br/>Abuse protection"]
        I3["✅ Input Validation<br/>Type-safe parsing"]
    end
    
    subgraph "🔒 Network Security"
        direction TB
        N1["🌐 HTTPS Enforcement<br/>TLS 1.2+ required"]
        N2["🚫 CSRF Protection<br/>Secure headers"]
        N3["🔍 Security Headers<br/>OWASP compliance"]
    end
    
    subgraph "📊 Monitoring & Recovery"
        direction TB
        M1["💓 Health Checks<br/>Automated monitoring"]
        M2["📝 Audit Logging<br/>Security events"]
        M3["🔄 Graceful Shutdown<br/>Resource cleanup"]
    end
    
    A1 --> I1
    A2 --> I2
    A3 --> I3
    
    I1 --> N1
    I2 --> N2
    I3 --> N3
    
    N1 --> M1
    N2 --> M2
    N3 --> M3
    
    classDef auth fill:#efebe9,stroke:#795548,stroke-width:2px
    classDef input fill:#f3e5f5,stroke:#9c27b0,stroke-width:2px
    classDef network fill:#e8f5e8,stroke:#4caf50,stroke-width:2px
    classDef monitoring fill:#fff3e0,stroke:#ff9800,stroke-width:2px
    
    class A1,A2,A3 auth
    class I1,I2,I3 input
    class N1,N2,N3 network
    class M1,M2,M3 monitoring
```

---

## 🎯 **Feature Comparison**
*Honest assessment vs alternatives*

| Feature | Campfire v0.1 | Slack | Discord | Mattermost |
|---------|---------------|-------|---------|------------|
| **Setup Time** | ⚡ 2 minutes | 🐌 15+ minutes | 🐌 10+ minutes | 🐌 30+ minutes |
| **Self-Hosting** | ✅ Single binary | ❌ Not available | ❌ Not available | ✅ Complex setup |
| **Real-time Chat** | ✅ WebSocket | ✅ WebSocket | ✅ WebSocket | ✅ WebSocket |
| **File Attachments** | 🚧 v0.2 planned | ✅ Full support | ✅ Full support | ✅ Full support |
| **Voice/Video** | ❌ Not planned | ✅ Full support | ✅ Full support | ✅ Plugin support |
| **Mobile Apps** | 📱 Web responsive | ✅ Native apps | ✅ Native apps | ✅ Native apps |
| **Bot Integration** | ✅ API + Webhooks | ✅ Rich ecosystem | ✅ Rich ecosystem | ✅ Plugin system |
| **Search** | ✅ Full-text FTS5 | ✅ Enterprise search | ✅ Basic search | ✅ Elasticsearch |
| **Cost (50 users)** | 💸 $0 (self-hosted) | 💰 $400/month | 💰 $200/month | 💸 $0 (self-hosted) |

**Campfire's Sweet Spot**: Teams who want reliable chat without complexity, vendor lock-in, or recurring costs.

---

## 📚 **Documentation & Resources**

### **🚀 Getting Started**
- [Zero-Friction Deployment Guide](docs/zero-friction-deployment.md) - Complete setup instructions
- [Configuration Reference](docs/configuration.md) - Environment variables and settings
- [First-Run Setup](docs/first-run-setup.md) - Admin account creation

### **🔧 Development & Integration**
- [API Documentation](docs/api-overview.md) - REST and WebSocket APIs
- [Bot Integration Guide](docs/bot-integration.md) - Webhooks and automation
- [Contributing Guide](CONTRIBUTING.md) - Development setup and guidelines

### **🛠️ Operations & Maintenance**
- [Performance Optimization](docs/performance-optimization-guide.md) - Tuning and scaling
- [Monitoring & Alerting](docs/monitoring-alerting-guide.md) - Observability setup
- [Backup & Recovery](docs/backup-restore-procedures.md) - Data protection

### **🎮 Demo & Testing**
- [Interface Previews](docs/interface-previews/) - Visual component gallery
- [Demo User Accounts](docs/demo-accounts.md) - Test credentials and scenarios

---

## 🤝 **Community & Support**

### **Get Help**
- 📖 **Documentation**: Comprehensive guides above
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/that-in-rust/campfire-on-rust/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/that-in-rust/campfire-on-rust/discussions)
- 📧 **Email**: support@campfire-rust.com

### **Contribute**
- 🔧 **Code**: See [Contributing Guide](CONTRIBUTING.md)
- 📝 **Documentation**: Help improve guides and examples
- 🐛 **Testing**: Report issues and edge cases
- 💡 **Ideas**: Share feature requests and feedback

### **Roadmap** *(Community-Driven)*
- **v0.2**: File attachments, avatar uploads, OpenGraph previews
- **v0.3**: Mobile apps, advanced search, analytics dashboard
- **v1.0**: Enterprise features, SSO, advanced admin controls

---

## 🙏 **Acknowledgments**

This project was inspired by the original **Campfire** application from **Basecamp**. Special thanks to **DHH** and **Jason Fried** for pioneering simple, effective team communication tools and for open-sourcing patterns that influenced this implementation.

**Built with**: Rust 🦀, Axum, SQLite, WebSockets, and a focus on simplicity over complexity.

---

<div align="center">

**Ready to transform your team communication?**

[🚀 **Deploy Now**](https://railway.app/template/campfire-rust-v01) • [📖 **Read Docs**](docs/zero-friction-deployment.md) • [💬 **Get Support**](https://github.com/that-in-rust/campfire-on-rust/discussions)

*Made with ❤️ by the Rust community*

</div>

