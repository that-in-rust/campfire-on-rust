#!/bin/bash
# campfire-on-rust v0.1 - Zero-Friction Local Installation Script
# Usage: curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash

set -e

# Error handler with clear feedback
error_handler() {
    local exit_code=$?
    local line_number=$1
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "❌ INSTALLATION FAILED"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "🔍 What happened:"
    echo "   • Exit code: $exit_code"
    echo "   • Failed at line: $line_number"
    echo "   • OS: $(uname -s) $(uname -m)"
    echo ""
    echo "🛠️  Quick fixes to try:"
    echo "   1. Check internet connection"
    echo "   2. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "   3. Install Git: your package manager (apt, brew, etc.)"
    echo "   4. Try manual installation (see below)"
    echo ""
    echo "📖 Manual installation:"
    echo "   git clone https://github.com/that-in-rust/campfire-on-rust.git"
    echo "   cd campfire-on-rust"
    echo "   cargo run"
    echo ""
    echo "🆘 Still stuck? Get help:"
    echo "   • GitHub Issues: https://github.com/that-in-rust/campfire-on-rust/issues"
    echo "   • Include this error and your OS info"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    exit $exit_code
}

# Set up error handling
trap 'error_handler $LINENO' ERR

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
REPO="that-in-rust/campfire-on-rust"
VERSION="v0.1.0"
BINARY_NAME="campfire-on-rust"
INSTALL_DIR="$HOME/.campfire-on-rust"

# Check if required tools are available
check_requirements() {
    local missing_tools=()
    
    if ! command -v git >/dev/null 2>&1; then
        missing_tools+=("git")
    fi
    
    if ! command -v cargo >/dev/null 2>&1; then
        missing_tools+=("rust/cargo")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "⚠️  MISSING REQUIREMENTS"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "🔧 Please install these tools first:"
        
        for tool in "${missing_tools[@]}"; do
            case $tool in
                "git")
                    echo "   📦 Git:"
                    echo "      • macOS: brew install git"
                    echo "      • Ubuntu/Debian: sudo apt install git"
                    echo "      • CentOS/RHEL: sudo yum install git"
                    echo "      • Windows: https://git-scm.com/download/win"
                    ;;
                "rust/cargo")
                    echo "   🦀 Rust:"
                    echo "      • All platforms: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                    echo "      • Then restart your terminal or run: source ~/.cargo/env"
                    ;;
            esac
            echo ""
        done
        
        echo "💡 After installing, run this script again:"
        echo "   curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash"
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        exit 1
    fi
}

# Clone and build campfire-on-rust
install_campfire() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🔥 INSTALLING CAMPFIRE-ON-RUST"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    cd "$INSTALL_DIR"
    
    echo "📥 Step 1/4: Cloning repository..."
    if [ -d "campfire-on-rust" ]; then
        echo "   • Repository already exists, updating..."
        cd campfire-on-rust
        git pull origin main
    else
        echo "   • Cloning from GitHub..."
        git clone https://github.com/${REPO}.git
        cd campfire-on-rust
    fi
    echo "   ✅ Repository ready"
    echo ""
    
    echo "🔨 Step 2/4: Building campfire-on-rust (this may take a few minutes)..."
    echo "   • Compiling Rust code..."
    echo "   • This is normal for first-time builds"
    echo ""
    
    # Build in release mode for better performance
    if cargo build --release; then
        echo "   ✅ Build successful!"
    else
        echo "   ❌ Build failed"
        echo ""
        echo "🛠️  Try these solutions:"
        echo "   1. Update Rust: rustup update"
        echo "   2. Clean and retry: cargo clean && cargo build --release"
        echo "   3. Check Rust version: rustc --version (need 1.70+)"
        exit 1
    fi
    echo ""
    
    echo "📦 Step 3/4: Installing binary..."
    # Copy binary to a location in PATH
    cp target/release/campfire-on-rust "$HOME/.local/bin/" 2>/dev/null || {
        mkdir -p "$HOME/.local/bin"
        cp target/release/campfire-on-rust "$HOME/.local/bin/"
    }
    echo "   ✅ Binary installed to $HOME/.local/bin/campfire-on-rust"
    echo ""
}

# Setup environment and configuration
setup_environment() {
    echo "⚙️  Step 4/4: Setting up configuration..."
    
    # Create data directory
    local data_dir="$HOME/.campfire-on-rust-data"
    mkdir -p "$data_dir"
    
    # Create basic .env file if it doesn't exist
    local env_file="$data_dir/.env"
    if [[ ! -f "$env_file" ]]; then
        cat > "$env_file" << EOF
# campfire-on-rust Configuration
CAMPFIRE_DATABASE_URL=sqlite://$data_dir/campfire.db
CAMPFIRE_HOST=127.0.0.1
CAMPFIRE_PORT=3000
CAMPFIRE_LOG_LEVEL=info

# Enable demo mode with sample data (recommended for first try)
CAMPFIRE_DEMO_MODE=true

# Optional: Configure push notifications (generate at https://vapidkeys.com/)
# CAMPFIRE_VAPID_PUBLIC_KEY=your_public_key_here
# CAMPFIRE_VAPID_PRIVATE_KEY=your_private_key_here

# Optional: SSL configuration for production
# CAMPFIRE_SSL_DOMAIN=your-domain.com
EOF
        echo "   ✅ Created configuration with demo mode enabled"
    fi
    
    echo "   📁 Data directory: $data_dir"
    echo "   ⚙️  Configuration: $env_file"
    echo ""
}

# Add to PATH if needed
update_path() {
    local bin_dir="$HOME/.local/bin"
    local shell_rc
    
    # Detect shell and update PATH
    case "$SHELL" in
        */bash)     shell_rc="$HOME/.bashrc" ;;
        */zsh)      shell_rc="$HOME/.zshrc" ;;
        */fish)     shell_rc="$HOME/.config/fish/config.fish" ;;
        *)          shell_rc="$HOME/.profile" ;;
    esac
    
    # Check if already in PATH
    if [[ ":$PATH:" != *":$bin_dir:"* ]]; then
        echo "🔧 Adding $bin_dir to PATH..."
        echo "export PATH=\"\$PATH:$bin_dir\"" >> "$shell_rc"
        export PATH="$PATH:$bin_dir"
        echo "   ✅ Added to PATH (restart terminal or run: source $shell_rc)"
    fi
}

# Start campfire-on-rust
start_campfire() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🚀 STARTING CAMPFIRE-ON-RUST"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Change to data directory
    cd "$HOME/.campfire-on-rust-data"
    
    echo "🔥 Starting server..."
    echo "📱 Open your browser to: http://localhost:3000"
    echo "⏹️  Press Ctrl+C to stop the server"
    echo ""
    echo "💡 First time? You'll see a setup page to create your admin account"
    echo "🎮 Demo mode is enabled - you'll see sample chat data"
    echo ""
    
    # Start the application
    if command -v campfire-on-rust >/dev/null 2>&1; then
        campfire-on-rust
    elif [[ -x "$HOME/.local/bin/campfire-on-rust" ]]; then
        "$HOME/.local/bin/campfire-on-rust"
    else
        echo "❌ Error: Could not find campfire-on-rust binary"
        echo ""
        echo "🛠️  Try these solutions:"
        echo "   1. Restart your terminal"
        echo "   2. Run: source ~/.bashrc (or ~/.zshrc)"
        echo "   3. Run directly: $HOME/.local/bin/campfire-on-rust"
        echo "   4. Check installation: ls -la $HOME/.local/bin/campfire-on-rust"
        exit 1
    fi
}

# Show usage instructions
show_usage() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🎉 INSTALLATION COMPLETE!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "🚀 Quick Start:"
    echo "   1. Run: campfire-on-rust"
    echo "   2. Open: http://localhost:3000"
    echo "   3. Create your admin account (first-run setup)"
    echo ""
    echo "📁 Files:"
    echo "   • Data: $HOME/.campfire-on-rust-data/"
    echo "   • Config: $HOME/.campfire-on-rust-data/.env"
    echo "   • Binary: $HOME/.local/bin/campfire-on-rust"
    echo ""
    echo "🎮 Demo Mode:"
    echo "   • Already enabled in your config"
    echo "   • You'll see sample chat rooms and messages"
    echo "   • Perfect for trying out features"
    echo ""
    echo "🔧 Commands:"
    echo "   • Start: campfire-on-rust"
    echo "   • Stop: Press Ctrl+C"
    echo "   • Logs: Check terminal output"
    echo ""
    echo "🌐 Deploy for your team:"
    echo "   • Railway: https://railway.app/template/campfire-rust"
    echo "   • Or run on your server with the same binary"
    echo ""
    echo "🆘 Need help?"
    echo "   • Docs: https://github.com/${REPO}#readme"
    echo "   • Issues: https://github.com/${REPO}/issues"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Main installation flow
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🔥 CAMPFIRE-ON-RUST INSTALLER"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "📋 What this script does:"
    echo "   • Clones the repository from GitHub"
    echo "   • Builds campfire-on-rust from source"
    echo "   • Sets up configuration with demo mode"
    echo "   • Installs to ~/.local/bin/campfire-on-rust"
    echo ""
    echo "⏱️  Estimated time: 2-5 minutes (depending on your internet and CPU)"
    echo ""
    
    # Check requirements first
    check_requirements
    
    # Install campfire-on-rust
    install_campfire
    setup_environment
    update_path
    
    # Ask if user wants to start immediately
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    read -p "🚀 Start campfire-on-rust now? [Y/n]: " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
        start_campfire
    else
        show_usage
    fi
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "🔥 CAMPFIRE-ON-RUST INSTALLER HELP"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "📖 Usage:"
        echo "   curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash"
        echo ""
        echo "🔧 Options:"
        echo "   --help, -h     Show this help message"
        echo "   --no-start     Install but don't start campfire-on-rust"
        echo ""
        echo "📋 Requirements:"
        echo "   • Git (for cloning repository)"
        echo "   • Rust/Cargo (for building from source)"
        echo "   • Internet connection"
        echo ""
        echo "🎯 What you get:"
        echo "   • campfire-on-rust binary in ~/.local/bin/"
        echo "   • Configuration in ~/.campfire-on-rust-data/"
        echo "   • Demo mode enabled for easy testing"
        echo "   • Ready to run on http://localhost:3000"
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        exit 0
        ;;
    --no-start)
        # Install but don't start
        check_requirements
        install_campfire
        setup_environment
        update_path
        show_usage
        ;;
    *)
        # Default: install and optionally start
        main
        ;;
esac