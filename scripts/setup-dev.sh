#!/bin/bash
# Development environment setup script for campfire-on-rust

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🏗️  Setting up campfire-on-rust development environment...${NC}"

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo -e "${YELLOW}📦 Rust not found. Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    echo -e "${GREEN}✅ Rust installed successfully${NC}"
else
    echo -e "${GREEN}✅ Rust is already installed${NC}"
    rustc --version
fi

# Update Rust to latest stable
echo -e "${YELLOW}🔄 Updating Rust to latest stable...${NC}"
rustup update stable
rustup default stable

# Install required components
echo -e "${YELLOW}🔧 Installing Rust components...${NC}"
rustup component add rustfmt clippy

# Install development tools
echo -e "${YELLOW}📦 Installing development tools...${NC}"
cargo install cargo-watch || echo "cargo-watch already installed"
cargo install cargo-audit || echo "cargo-audit already installed"
cargo install cargo-outdated || echo "cargo-outdated already installed"
cargo install cargo-tarpaulin || echo "cargo-tarpaulin already installed"
cargo install cargo-expand || echo "cargo-expand already installed"
cargo install cargo-bloat || echo "cargo-bloat already installed"

# Set up environment file
if [ ! -f ".env" ]; then
    echo -e "${YELLOW}📝 Creating .env file from template...${NC}"
    cp .env.example .env
    echo -e "${GREEN}✅ .env file created. Please review and update as needed.${NC}"
else
    echo -e "${GREEN}✅ .env file already exists${NC}"
fi

# Build the project
echo -e "${YELLOW}🏗️  Building the project...${NC}"
cargo build
echo -e "${GREEN}✅ Project built successfully${NC}"

# Run database migrations
echo -e "${YELLOW}🗄️  Running database migrations...${NC}"
if [ -f "src/bin/migrate.rs" ]; then
    cargo run --bin migrate
    echo -e "${GREEN}✅ Database migrations completed${NC}"
else
    echo -e "${YELLOW}⚠️  Migration binary not found, skipping database setup${NC}"
fi

# Run tests to verify setup
echo -e "${YELLOW}🧪 Running tests to verify setup...${NC}"
cargo test
echo -e "${GREEN}✅ Tests passed - setup verified${NC}"

# Set up pre-commit hooks
if [ -f "scripts/pre-commit" ]; then
    echo -e "${YELLOW}🪝 Setting up pre-commit hooks...${NC}"
    cp scripts/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo -e "${GREEN}✅ Pre-commit hooks installed${NC}"
else
    echo -e "${YELLOW}⚠️  Pre-commit hook script not found${NC}"
fi

# Create VS Code settings if not exists
if [ ! -d ".vscode" ]; then
    echo -e "${YELLOW}⚙️  Creating VS Code settings...${NC}"
    mkdir -p .vscode
    cat > .vscode/settings.json << 'EOF'
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
  "editor.formatOnSave": true,
  "editor.rulers": [100],
  "files.trimTrailingWhitespace": true,
  "files.insertFinalNewline": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true
}
EOF
    echo -e "${GREEN}✅ VS Code settings created${NC}"
else
    echo -e "${GREEN}✅ VS Code settings already exist${NC}"
fi

echo -e "${GREEN}🎉 Development environment setup complete!${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "1. Review and update .env file with your configuration"
echo "2. Install recommended VS Code extensions:"
echo "   - rust-analyzer"
echo "   - CodeLLDB"
echo "   - Better TOML"
echo "   - Error Lens"
echo "3. Start development server: cargo run"
echo "4. Or use hot reload: cargo watch -x run"
echo ""
echo -e "${BLUE}Useful commands:${NC}"
echo "- Run tests: cargo test"
echo "- Format code: cargo fmt"
echo "- Lint code: cargo clippy"
echo "- Security audit: cargo audit"
echo "- Run CI checks: ./scripts/ci-check.sh"