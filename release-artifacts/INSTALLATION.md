# Installation Instructions

## Automated Installation (Recommended)

The easiest way to get Campfire running:

```bash
curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash
```

This will:
- Detect your platform automatically
- Download the correct binary
- Set up the environment
- Start Campfire on `http://localhost:3000`

## Manual Installation

If you prefer to install manually:

### 1. Download Binary

Download the appropriate binary for your platform from the release assets.

### 2. Make Executable

```bash
chmod +x campfire-on-rust-*
```

### 3. Create Data Directory

```bash
mkdir -p ~/.campfire
```

### 4. Create Configuration

Create `~/.campfire/.env`:

```bash
CAMPFIRE_DATABASE_URL=sqlite:///Users/$(whoami)/.campfire/campfire.db
CAMPFIRE_HOST=127.0.0.1
CAMPFIRE_PORT=3000
CAMPFIRE_LOG_LEVEL=info
```

### 5. Run Campfire

```bash
cd ~/.campfire
./path/to/campfire-on-rust-*
```

### 6. Open Browser

Navigate to `http://localhost:3000` and create your admin account.

## Demo Mode

To try Campfire with demo data:

1. Add `CAMPFIRE_DEMO_MODE=true` to your `.env` file
2. Restart Campfire
3. Explore the pre-loaded demo conversations

## Troubleshooting

See the main README for comprehensive troubleshooting guidance.

## Support

- **GitHub Issues**: [Report bugs](https://github.com/that-in-rust/campfire-on-rust/issues)
- **GitHub Discussions**: [Ask questions](https://github.com/that-in-rust/campfire-on-rust/discussions)
