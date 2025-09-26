#!/bin/bash
# Campfire v0.1 - Zero-Friction Local Installation Script
# Usage: curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash

set -e

# Error handler for unexpected failures
error_handler() {
    local exit_code=$?
    local line_number=$1
    
    echo -e "\n${RED}❌ Installation failed unexpectedly${NC}"
    echo -e "${YELLOW}💡 Error details:${NC}"
    echo -e "${YELLOW}   Exit code: ${exit_code}${NC}"
    echo -e "${YELLOW}   Line: ${line_number}${NC}"
    echo -e "${YELLOW}🆘 Need help? Report this issue:${NC}"
    echo -e "${YELLOW}   GitHub: https://github.com/that-in-rust/campfire-on-rust/issues${NC}"
    echo -e "${YELLOW}   Email: campfire-support@that-in-rust.dev${NC}"
    echo -e "${YELLOW}   Include: Your OS ($(uname -s)), architecture ($(uname -m)), and this error${NC}"
    
    # Track the failure
    track_install_result false "Unexpected error at line $line_number (exit code $exit_code)"
    
    exit $exit_code
}

# Set up error handling
trap 'error_handler $LINENO' ERR

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="that-in-rust/campfire-on-rust"
VERSION="v0.1.0"
BINARY_NAME="campfire-on-rust"
INSTALL_DIR="$HOME/.local/bin"

# Detect OS and architecture
detect_platform() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="darwin" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *)          
            echo -e "${RED}❌ Unsupported OS: $(uname -s)${NC}"
            echo -e "${YELLOW}💡 Supported platforms: Linux, macOS, Windows${NC}"
            echo -e "${YELLOW}📖 For manual installation: https://github.com/that-in-rust/campfire-on-rust/releases${NC}"
            exit 1 
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        arm64|aarch64)  arch="aarch64" ;;
        *)              
            echo -e "${RED}❌ Unsupported architecture: $(uname -m)${NC}"
            echo -e "${YELLOW}💡 Supported architectures: x86_64, aarch64 (ARM64)${NC}"
            echo -e "${YELLOW}🔧 Try building from source: git clone https://github.com/that-in-rust/campfire-on-rust.git${NC}"
            exit 1 
            ;;
    esac
    
    echo "${os}-${arch}"
}

# Track install script execution (privacy-friendly)
track_install_start() {
    # Send anonymous tracking data to help improve the installation process
    # This is completely optional and privacy-friendly (no personal data)
    if command -v curl >/dev/null 2>&1; then
        curl -s -X POST "https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/api/analytics/track/install-download" \
            -H "Content-Type: application/json" \
            -d '{"platform":"'$(detect_platform)'","version":"'$VERSION'"}' \
            >/dev/null 2>&1 || true
    fi
}

# Track install result (success/failure)
track_install_result() {
    local success=$1
    local error_msg=${2:-""}
    local platform=$(detect_platform)
    
    if command -v curl >/dev/null 2>&1; then
        local payload='{"success":'$success',"platform":"'$platform'","version":"'$VERSION'"'
        if [[ -n "$error_msg" ]]; then
            payload+=', "error_message":"'$error_msg'"'
        fi
        payload+='}'
        
        curl -s -X POST "https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/api/analytics/track/install-result" \
            -H "Content-Type: application/json" \
            -d "$payload" \
            >/dev/null 2>&1 || true
    fi
}

# Download and install binary
install_campfire() {
    local platform
    platform=$(detect_platform)
    
    echo -e "${BLUE}🔥 Installing Campfire v0.1...${NC}"
    echo -e "${YELLOW}Platform: ${platform}${NC}"
    
    # Track installation start
    track_install_start
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Download URL
    local download_url="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}-${platform}"
    if [[ "$platform" == *"windows"* ]]; then
        download_url="${download_url}.exe"
        BINARY_NAME="${BINARY_NAME}.exe"
    fi
    
    echo -e "${YELLOW}Downloading from: ${download_url}${NC}"
    
    # Download binary
    if command -v curl >/dev/null 2>&1; then
        if curl -L -o "${INSTALL_DIR}/${BINARY_NAME}" "$download_url"; then
            echo -e "${GREEN}✅ Download successful${NC}"
        else
            echo -e "${RED}❌ Download failed${NC}"
            echo -e "${YELLOW}💡 Possible solutions:${NC}"
            echo -e "${YELLOW}   1. Check internet connection${NC}"
            echo -e "${YELLOW}   2. Try manual download: ${download_url}${NC}"
            echo -e "${YELLOW}   3. Build from source: https://github.com/that-in-rust/campfire-on-rust#building${NC}"
            echo -e "${YELLOW}   4. Report issue: https://github.com/that-in-rust/campfire-on-rust/issues${NC}"
            track_install_result false "Download failed"
            exit 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if wget -O "${INSTALL_DIR}/${BINARY_NAME}" "$download_url"; then
            echo -e "${GREEN}✅ Download successful${NC}"
        else
            echo -e "${RED}❌ Download failed${NC}"
            echo -e "${YELLOW}💡 Possible solutions:${NC}"
            echo -e "${YELLOW}   1. Check internet connection${NC}"
            echo -e "${YELLOW}   2. Try manual download: ${download_url}${NC}"
            echo -e "${YELLOW}   3. Build from source: https://github.com/that-in-rust/campfire-on-rust#building${NC}"
            echo -e "${YELLOW}   4. Report issue: https://github.com/that-in-rust/campfire-on-rust/issues${NC}"
            track_install_result false "Download failed"
            exit 1
        fi
    else
        echo -e "${RED}❌ Error: curl or wget is required${NC}"
        echo -e "${YELLOW}💡 Install a download tool:${NC}"
        echo -e "${YELLOW}   Ubuntu/Debian: sudo apt install curl${NC}"
        echo -e "${YELLOW}   CentOS/RHEL: sudo yum install curl${NC}"
        echo -e "${YELLOW}   macOS: brew install curl${NC}"
        echo -e "${YELLOW}   Or download manually: ${download_url}${NC}"
        track_install_result false "No download tool available"
        exit 1
    fi
    
    # Make executable
    if chmod +x "${INSTALL_DIR}/${BINARY_NAME}"; then
        echo -e "${GREEN}✅ Campfire installed to ${INSTALL_DIR}/${BINARY_NAME}${NC}"
        track_install_result true
    else
        echo -e "${RED}❌ Failed to make binary executable${NC}"
        echo -e "${YELLOW}💡 Try manually:${NC}"
        echo -e "${YELLOW}   chmod +x ${INSTALL_DIR}/${BINARY_NAME}${NC}"
        echo -e "${YELLOW}   ${INSTALL_DIR}/${BINARY_NAME}${NC}"
        track_install_result false "Failed to make executable"
        exit 1
    fi
}

# Setup environment
setup_environment() {
    echo -e "${BLUE}🔧 Setting up environment...${NC}"
    
    # Create data directory
    local data_dir="$HOME/.campfire"
    mkdir -p "$data_dir"
    
    # Create basic .env file if it doesn't exist
    local env_file="$data_dir/.env"
    if [[ ! -f "$env_file" ]]; then
        cat > "$env_file" << EOF
# Campfire Configuration
CAMPFIRE_DATABASE_URL=sqlite://$data_dir/campfire.db
CAMPFIRE_HOST=127.0.0.1
CAMPFIRE_PORT=3000
CAMPFIRE_LOG_LEVEL=info

# Optional: Enable demo mode for testing
# CAMPFIRE_DEMO_MODE=true

# Optional: Configure push notifications (generate at https://vapidkeys.com/)
# CAMPFIRE_VAPID_PUBLIC_KEY=your_public_key_here
# CAMPFIRE_VAPID_PRIVATE_KEY=your_private_key_here

# Optional: SSL configuration for production
# CAMPFIRE_SSL_DOMAIN=your-domain.com
EOF
        echo -e "${GREEN}✅ Created configuration file: ${env_file}${NC}"
    fi
    
    echo -e "${YELLOW}📁 Data directory: ${data_dir}${NC}"
    echo -e "${YELLOW}⚙️  Configuration: ${env_file}${NC}"
}

# Add to PATH
update_path() {
    local shell_rc
    
    # Detect shell and update PATH
    case "$SHELL" in
        */bash)     shell_rc="$HOME/.bashrc" ;;
        */zsh)      shell_rc="$HOME/.zshrc" ;;
        */fish)     shell_rc="$HOME/.config/fish/config.fish" ;;
        *)          shell_rc="$HOME/.profile" ;;
    esac
    
    # Check if already in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo -e "${YELLOW}Adding ${INSTALL_DIR} to PATH in ${shell_rc}${NC}"
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$shell_rc"
        export PATH="$PATH:$INSTALL_DIR"
        echo -e "${GREEN}✅ Added to PATH${NC}"
    else
        echo -e "${GREEN}✅ Already in PATH${NC}"
    fi
}

# Start Campfire
start_campfire() {
    echo -e "${BLUE}🚀 Starting Campfire...${NC}"
    echo -e "${YELLOW}Note: This will start Campfire in the foreground. Press Ctrl+C to stop.${NC}"
    echo ""
    
    # Change to data directory
    cd "$HOME/.campfire"
    
    # Start the application
    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]] || [[ -x "${INSTALL_DIR}/${BINARY_NAME}" ]]; then
        echo -e "${GREEN}🔥 Campfire is starting...${NC}"
        echo -e "${GREEN}📱 Open your browser to: http://localhost:3000${NC}"
        echo ""
        
        # Try to start from PATH first, then from install directory
        if command -v "$BINARY_NAME" >/dev/null 2>&1; then
            "$BINARY_NAME"
        else
            "${INSTALL_DIR}/${BINARY_NAME}"
        fi
    else
        echo -e "${RED}❌ Error: Could not find Campfire binary${NC}"
        echo -e "${YELLOW}💡 Try these solutions:${NC}"
        echo -e "${YELLOW}   1. Run directly: ${INSTALL_DIR}/${BINARY_NAME}${NC}"
        echo -e "${YELLOW}   2. Add to PATH: export PATH=\"\$PATH:${INSTALL_DIR}\"${NC}"
        echo -e "${YELLOW}   3. Restart your terminal${NC}"
        echo -e "${YELLOW}   4. Check installation: ls -la ${INSTALL_DIR}/${BINARY_NAME}${NC}"
        exit 1
    fi
}

# Show usage instructions
show_usage() {
    echo -e "${GREEN}🎉 Campfire v0.1 Installation Complete!${NC}"
    echo ""
    echo -e "${BLUE}Quick Start:${NC}"
    echo -e "  1. ${YELLOW}campfire-on-rust${NC}                    # Start Campfire"
    echo -e "  2. Open ${YELLOW}http://localhost:3000${NC}          # Access web interface"
    echo -e "  3. Create your admin account           # First-run setup"
    echo ""
    echo -e "${BLUE}Configuration:${NC}"
    echo -e "  📁 Data: ${YELLOW}$HOME/.campfire/${NC}"
    echo -e "  ⚙️  Config: ${YELLOW}$HOME/.campfire/.env${NC}"
    echo ""
    echo -e "${BLUE}Demo Mode (Optional):${NC}"
    echo -e "  Add ${YELLOW}CAMPFIRE_DEMO_MODE=true${NC} to .env file"
    echo -e "  Restart Campfire to try pre-loaded demo data"
    echo ""
    echo -e "${BLUE}Production Deployment:${NC}"
    echo -e "  🚂 Railway: ${YELLOW}https://railway.app/template/campfire-rust${NC}"
    echo -e "  🐳 Docker: ${YELLOW}docker run -p 3000:3000 campfire-rust:v0.1${NC}"
    echo ""
    echo -e "${BLUE}Need Help?${NC}"
    echo -e "  📖 Docs: ${YELLOW}https://github.com/${REPO}#readme${NC}"
    echo -e "  🐛 Issues: ${YELLOW}https://github.com/${REPO}/issues${NC}"
}

# Main installation flow
main() {
    echo -e "${GREEN}"
    echo "  ____                        __ _            "
    echo " / ___|__ _ _ __ ___  _ __  / _(_)_ __ ___   "
    echo "| |   / _\` | '_ \` _ \\| '_ \\| |_| | '__/ _ \\  "
    echo "| |__| (_| | | | | | | |_) |  _| | | |  __/  "
    echo " \\____\\__,_|_| |_| |_| .__/|_| |_|_|  \\___|  "
    echo "                     |_|                     "
    echo -e "${NC}"
    echo -e "${BLUE}Zero-Friction Installation Script v0.1${NC}"
    echo ""
    
    # Check for required tools
    if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
        echo -e "${RED}❌ Error: curl or wget is required for installation${NC}"
        echo -e "${YELLOW}💡 Install a download tool:${NC}"
        echo -e "${YELLOW}   Ubuntu/Debian: sudo apt install curl${NC}"
        echo -e "${YELLOW}   CentOS/RHEL: sudo yum install curl${NC}"
        echo -e "${YELLOW}   macOS: brew install curl${NC}"
        echo -e "${YELLOW}   Windows: Install Git Bash or WSL${NC}"
        echo -e "${YELLOW}📖 Manual installation: https://github.com/that-in-rust/campfire-on-rust/releases${NC}"
        exit 1
    fi
    
    # Install Campfire
    install_campfire
    setup_environment
    update_path
    
    # Ask if user wants to start immediately
    echo ""
    read -p "$(echo -e ${YELLOW}Start Campfire now? [Y/n]: ${NC})" -n 1 -r
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
        echo "Campfire v0.1 Installation Script"
        echo ""
        echo "Usage:"
        echo "  curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash"
        echo "  curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash -s -- --no-start"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --no-start     Install but don't start Campfire"
        exit 0
        ;;
    --no-start)
        # Install but don't start
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