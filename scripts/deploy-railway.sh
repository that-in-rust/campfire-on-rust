#!/bin/bash
# Campfire v0.1 - Railway Deployment Script
# Zero-friction deployment to Railway.app

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RAILWAY_TEMPLATE_URL="https://railway.app/template/campfire-rust-v01"
GITHUB_REPO="that-in-rust/campfire-on-rust"

# Check if Railway CLI is installed
check_railway_cli() {
    if ! command -v railway >/dev/null 2>&1; then
        echo -e "${YELLOW}Railway CLI not found. Installing...${NC}"
        
        # Install Railway CLI
        if command -v npm >/dev/null 2>&1; then
            npm install -g @railway/cli
        elif command -v curl >/dev/null 2>&1; then
            # Install via curl
            curl -fsSL https://railway.app/install.sh | sh
        else
            echo -e "${RED}Error: npm or curl is required to install Railway CLI${NC}"
            echo -e "${YELLOW}Please install Railway CLI manually: https://docs.railway.app/develop/cli${NC}"
            exit 1
        fi
        
        echo -e "${GREEN}✅ Railway CLI installed${NC}"
    else
        echo -e "${GREEN}✅ Railway CLI found${NC}"
    fi
}

# Login to Railway
railway_login() {
    echo -e "${BLUE}🔐 Logging into Railway...${NC}"
    
    if ! railway whoami >/dev/null 2>&1; then
        echo -e "${YELLOW}Please log in to Railway:${NC}"
        railway login
    else
        echo -e "${GREEN}✅ Already logged in to Railway${NC}"
    fi
}

# Deploy via template (easiest method)
deploy_via_template() {
    echo -e "${BLUE}🚀 Deploying Campfire via Railway template...${NC}"
    echo ""
    echo -e "${GREEN}🎯 One-Click Deployment:${NC}"
    echo -e "${YELLOW}1. Click this link: ${RAILWAY_TEMPLATE_URL}${NC}"
    echo -e "${YELLOW}2. Connect your GitHub account${NC}"
    echo -e "${YELLOW}3. Click 'Deploy Now'${NC}"
    echo -e "${YELLOW}4. Wait for deployment to complete${NC}"
    echo ""
    echo -e "${BLUE}📋 Template includes:${NC}"
    echo -e "  ✅ Optimized Dockerfile for Railway"
    echo -e "  ✅ Automatic HTTPS with custom domain support"
    echo -e "  ✅ Persistent SQLite database storage"
    echo -e "  ✅ Environment variable configuration"
    echo -e "  ✅ Health checks and auto-restart"
    echo -e "  ✅ Zero-downtime deployments"
    echo ""
    
    # Ask if user wants to open the template URL
    read -p "$(echo -e ${YELLOW}Open Railway template in browser? [Y/n]: ${NC})" -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
        if command -v open >/dev/null 2>&1; then
            open "$RAILWAY_TEMPLATE_URL"
        elif command -v xdg-open >/dev/null 2>&1; then
            xdg-open "$RAILWAY_TEMPLATE_URL"
        else
            echo -e "${YELLOW}Please open this URL in your browser:${NC}"
            echo "$RAILWAY_TEMPLATE_URL"
        fi
    fi
}

# Deploy from local repository
deploy_from_local() {
    echo -e "${BLUE}🚀 Deploying from local repository...${NC}"
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir >/dev/null 2>&1; then
        echo -e "${RED}Error: Not in a git repository${NC}"
        echo -e "${YELLOW}Please run this script from the Campfire project directory${NC}"
        exit 1
    fi
    
    # Initialize Railway project
    echo -e "${YELLOW}Initializing Railway project...${NC}"
    railway init
    
    # Set up environment variables
    echo -e "${YELLOW}Setting up environment variables...${NC}"
    railway variables set CAMPFIRE_HOST=0.0.0.0
    railway variables set CAMPFIRE_PORT=3000
    railway variables set CAMPFIRE_DATABASE_URL=sqlite:///app/data/campfire.db
    railway variables set RUST_LOG=campfire_on_rust=info
    
    # Deploy
    echo -e "${YELLOW}Deploying to Railway...${NC}"
    railway up
    
    # Get deployment URL
    local url
    url=$(railway domain)
    
    echo -e "${GREEN}🎉 Deployment complete!${NC}"
    echo -e "${GREEN}🌐 Your Campfire is available at: ${url}${NC}"
}

# Configure environment variables
configure_environment() {
    echo -e "${BLUE}⚙️  Environment Configuration${NC}"
    echo ""
    echo -e "${YELLOW}Optional configurations (you can set these later in Railway dashboard):${NC}"
    echo ""
    
    # VAPID keys for push notifications
    echo -e "${BLUE}📱 Push Notifications:${NC}"
    echo -e "  Generate VAPID keys at: ${YELLOW}https://vapidkeys.com/${NC}"
    echo -e "  Set CAMPFIRE_VAPID_PUBLIC_KEY and CAMPFIRE_VAPID_PRIVATE_KEY"
    echo ""
    
    # Custom domain
    echo -e "${BLUE}🌐 Custom Domain:${NC}"
    echo -e "  Set CAMPFIRE_SSL_DOMAIN to enable automatic HTTPS"
    echo -e "  Example: CAMPFIRE_SSL_DOMAIN=chat.yourcompany.com"
    echo ""
    
    # Security settings
    echo -e "${BLUE}🔒 Security Settings:${NC}"
    echo -e "  CAMPFIRE_SESSION_TIMEOUT_HOURS (default: 24)"
    echo -e "  CAMPFIRE_ENABLE_USER_REGISTRATION (default: true)"
    echo ""
    
    # Performance settings
    echo -e "${BLUE}⚡ Performance Settings:${NC}"
    echo -e "  CAMPFIRE_MAX_CONNECTIONS (default: 100)"
    echo -e "  CAMPFIRE_CONNECTION_TIMEOUT_SECONDS (default: 30)"
    echo ""
}

# Show post-deployment instructions
show_post_deployment() {
    echo -e "${GREEN}🎉 Campfire v0.1 Railway Deployment Complete!${NC}"
    echo ""
    echo -e "${BLUE}Next Steps:${NC}"
    echo -e "  1. ${YELLOW}Visit your deployment URL${NC}"
    echo -e "  2. ${YELLOW}Create your admin account${NC} (first-run setup)"
    echo -e "  3. ${YELLOW}Invite your team members${NC}"
    echo -e "  4. ${YELLOW}Configure push notifications${NC} (optional)"
    echo ""
    echo -e "${BLUE}Railway Dashboard:${NC}"
    echo -e "  📊 Metrics: View performance and usage"
    echo -e "  ⚙️  Variables: Configure environment settings"
    echo -e "  📝 Logs: Monitor application logs"
    echo -e "  🔄 Deployments: Manage deployments and rollbacks"
    echo ""
    echo -e "${BLUE}Scaling:${NC}"
    echo -e "  Railway automatically scales based on usage"
    echo -e "  Persistent SQLite database included"
    echo -e "  Zero-downtime deployments for updates"
    echo ""
    echo -e "${BLUE}Support:${NC}"
    echo -e "  📖 Docs: ${YELLOW}https://github.com/${GITHUB_REPO}#readme${NC}"
    echo -e "  🐛 Issues: ${YELLOW}https://github.com/${GITHUB_REPO}/issues${NC}"
    echo -e "  💬 Railway: ${YELLOW}https://railway.app/help${NC}"
}

# Main deployment flow
main() {
    echo -e "${GREEN}"
    echo "  ____                        __ _            "
    echo " / ___|__ _ _ __ ___  _ __  / _(_)_ __ ___   "
    echo "| |   / _\` | '_ \` _ \\| '_ \\| |_| | '__/ _ \\  "
    echo "| |__| (_| | | | | | | |_) |  _| | | |  __/  "
    echo " \\____\\__,_|_| |_| |_| .__/|_| |_|_|  \\___|  "
    echo "                     |_|                     "
    echo -e "${NC}"
    echo -e "${BLUE}Railway Deployment Script v0.1${NC}"
    echo ""
    
    # Check Railway CLI
    check_railway_cli
    railway_login
    
    # Choose deployment method
    echo -e "${YELLOW}Choose deployment method:${NC}"
    echo -e "  1. ${GREEN}Template deployment${NC} (recommended - one-click)"
    echo -e "  2. ${BLUE}Local repository deployment${NC} (for developers)"
    echo ""
    
    read -p "$(echo -e ${YELLOW}Select option [1-2]: ${NC})" -n 1 -r
    echo ""
    
    case $REPLY in
        1)
            deploy_via_template
            ;;
        2)
            deploy_from_local
            ;;
        *)
            echo -e "${YELLOW}Defaulting to template deployment...${NC}"
            deploy_via_template
            ;;
    esac
    
    configure_environment
    show_post_deployment
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Campfire v0.1 Railway Deployment Script"
        echo ""
        echo "Usage:"
        echo "  ./scripts/deploy-railway.sh              # Interactive deployment"
        echo "  ./scripts/deploy-railway.sh --template   # Template deployment only"
        echo "  ./scripts/deploy-railway.sh --local      # Local repository deployment"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --template     Deploy via Railway template (one-click)"
        echo "  --local        Deploy from local repository"
        exit 0
        ;;
    --template)
        check_railway_cli
        railway_login
        deploy_via_template
        configure_environment
        show_post_deployment
        ;;
    --local)
        check_railway_cli
        railway_login
        deploy_from_local
        configure_environment
        show_post_deployment
        ;;
    *)
        main
        ;;
esac