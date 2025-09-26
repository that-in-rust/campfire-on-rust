# Project Overview

## Structure

```
campfire-on-rust/
├── src/           # Rust source code
├── templates/     # HTML templates  
├── assets/        # Static files (CSS, JS, images, sounds)
├── scripts/       # Deployment and utility scripts
├── tests/         # Test suite
├── docs/          # Documentation
├── archive/       # Moved files (validation reports, project docs)
└── README.md      # Start here
```

## Key Files

- `src/main.rs` - Application entry point
- `Cargo.toml` - Rust dependencies and metadata
- `scripts/install.sh` - One-line installer
- `railway.toml` - Railway deployment config
- `docker-compose.yml` - Local Docker setup

## Development

```bash
# Clone and run locally
git clone https://github.com/that-in-rust/campfire-on-rust.git
cd campfire-on-rust
cargo run

# Run tests
cargo test

# Build release
cargo build --release
```

## Architecture

- **Backend**: Rust + Axum web framework
- **Database**: SQLite (embedded, zero-config)
- **Frontend**: Vanilla HTML/CSS/JS (no build step)
- **Real-time**: WebSockets for instant messaging
- **Assets**: Embedded in binary (single-file deployment)

## Contributing

See `archive/project-docs/CONTRIBUTING.md` for detailed contribution guidelines.