# Campfire on Rust

A coordination-first Rust rewrite of Basecamp's Campfire chat application, designed for 85-90% cost reduction while maintaining complete UI compatibility and production-grade reliability.

## Architecture Overview

This implementation prioritizes **coordination mechanisms** over raw performance, ensuring reliable operation under real-world conditions including network partitions, concurrent operations, and partial failures.

### Key Features

- **Complete UI Compatibility**: All original Campfire assets and styling preserved
- **Coordination-First Design**: Atomic state management across WebSocket connections and database operations
- **Feature Flag Architecture**: Graceful degradation with clear upgrade messaging
- **Production-Grade Reliability**: Comprehensive fault tolerance and recovery mechanisms
- **Significant Cost Reduction**: 85-90% reduction in hosting costs vs Rails implementation

## Project Structure

```
campfire-on-rust/
├── src/                          # Coordination-first Rust backend
│   ├── coordination/             # Core coordination mechanisms
│   ├── database/                 # Coordinated database operations
│   ├── models/                   # Domain models with type safety
│   ├── handlers/                 # HTTP request handlers
│   ├── websocket/                # WebSocket state coordination
│   └── assets/                   # Embedded asset management
├── frontend/                     # React frontend with coordination hooks
├── assets/                       # Original Campfire assets (164 files)
│   ├── images/                   # 79 SVG icons and images
│   ├── sounds/                   # 59 MP3 files for /play commands
│   └── stylesheets/              # 26 CSS files
├── migrations/                   # Database schema evolution
├── tests/                        # Coordination testing under failure
└── docker/                       # Container deployment
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 18+
- Docker (optional)

### Development Setup

1. **Clone and setup**:
   ```bash
   git clone <repository-url>
   cd campfire-on-rust
   ```

2. **Install Rust dependencies**:
   ```bash
   cargo build
   ```

3. **Install frontend dependencies**:
   ```bash
   cd frontend
   npm install
   cd ..
   ```

4. **Run development server**:
   ```bash
   cargo run
   ```

### Docker Deployment

```bash
cd docker
docker-compose up -d
```

## Feature Flags (MVP Configuration)

The MVP focuses on text-based chat with complete UI:

```rust
FeatureFlags {
    files_enabled: false,        // Gracefully disabled with upgrade messaging
    avatars_enabled: false,      // Text initials with professional styling
    opengraph_enabled: false,    // Links shown as text with "Preview coming soon"
    max_file_size: 0,
    search_enabled: true,        // Full FTS5 search functionality
    push_notifications: true,    // Web Push with VAPID
    bot_integrations: true,      // Webhook-based bot system
}
```

## Core Coordination Mechanisms

### 1. Atomic State Coordination
- Global event sequencing prevents out-of-order message delivery
- Atomic database transactions with proper rollback
- Cross-system coordination with compensation patterns

### 2. WebSocket State Synchronization
- Connection establishment with atomic presence tracking
- State recovery on reconnection with missed event replay
- Cross-tab coordination with leader election

### 3. Database Coordination
- SQLite WAL mode with single-writer coordination
- FTS5 search index updates with eventual consistency
- Transaction boundaries aligned with coordination requirements

### 4. Fault Tolerance
- Circuit breakers prevent cascade failures
- Exponential backoff retry with persistent queues
- Graceful degradation during partial failures

## Performance Targets (Coordination-Aware)

- **Memory**: 30-60MB total (includes coordination overhead)
- **Connections**: 500+ concurrent WebSocket connections
- **Throughput**: 1K+ requests/second with full coordination
- **Message Rate**: 50 messages/second system-wide
- **Users per Room**: 100 concurrent users
- **Recovery Time**: <60 seconds for coordination re-establishment

## Testing Strategy

### Coordination Testing
```bash
# Test coordination under failure scenarios
cargo test coordination

# Test network partition handling
cargo test --test network_partition

# Test WebSocket reconnection coordination
cargo test --test websocket_recovery
```

### Integration Testing
```bash
# Full system integration tests
cargo test integration

# Frontend coordination tests
cd frontend && npm test
```

## Asset Integration

All original Campfire assets are preserved:

- **Sound Commands**: 59 MP3 files for `/play` commands (bell, trombone, nyan, etc.)
- **UI Icons**: 79 SVG icons for complete interface compatibility
- **Stylesheets**: 26 CSS files maintaining exact visual appearance

## Evolution Roadmap

See [Future Enhancements Backlog](.kiro/specs/campfire-rust-rewrite/future-enhancements-backlog.md) for detailed phase evolution:

1. **Phase 1 (MVP)**: Complete UI with text-only backend
2. **Phase 2**: Avatar uploads and basic file storage
3. **Phase 3**: Document sharing and file attachments
4. **Phase 4**: Full Rails parity with image/video processing

## Documentation

- [Requirements](.kiro/specs/campfire-rust-rewrite/requirements.md) - Detailed MVP requirements
- [Architecture](.kiro/specs/campfire-rust-rewrite/architecture.md) - High-level coordination architecture
- [Architecture L2](.kiro/specs/campfire-rust-rewrite/architecture-L2.md) - Implementation patterns and project structure
- [Cynical Analysis](.kiro/specs/campfire-rust-rewrite/cynical-implementation-analysis.md) - Coordination gaps and solutions
- [Future Backlog](.kiro/specs/campfire-rust-rewrite/future-enhancements-backlog.md) - Evolution strategy

## Contributing

This project follows a coordination-first development approach:

1. **Coordination Tests First**: Write tests that validate coordination under failure
2. **Atomic Operations**: Ensure all multi-step operations are atomic
3. **State Recovery**: Implement proper recovery for all stateful components
4. **Documentation**: Update architecture documents with coordination patterns

## License

MIT License - see [MIT-LICENSE](MIT-LICENSE) for details.

## Acknowledgments

Based on the original Campfire application by Basecamp (37signals). This rewrite maintains the spirit and functionality of the original while optimizing for modern deployment costs and reliability patterns.

Last version with React thinking