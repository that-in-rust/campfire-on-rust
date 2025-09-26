#!/bin/bash
# Simple Analytics Tracker for Campfire Deployment Success
# Privacy-friendly tracking of deployment clicks and success rates

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ANALYTICS_DIR="$HOME/.campfire-analytics"
LOG_FILE="$ANALYTICS_DIR/deployment-tracking.log"

# Create analytics directory
mkdir -p "$ANALYTICS_DIR"

# Function to log events
log_event() {
    local event_type="$1"
    local platform="$2"
    local success="$3"
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    echo "${timestamp},${event_type},${platform},${success}" >> "$LOG_FILE"
}

# Track deployment click
track_deployment_click() {
    local platform="${1:-railway}"
    log_event "deployment_click" "$platform" "true"
    echo -e "${GREEN}📊 Tracked deployment click: ${platform}${NC}"
}

# Track deployment success
track_deployment_success() {
    local platform="${1:-railway}"
    local success="${2:-true}"
    log_event "deployment_result" "$platform" "$success"
    echo -e "${GREEN}📊 Tracked deployment result: ${platform} - ${success}${NC}"
}

# Track install script download
track_install_download() {
    local platform=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    log_event "install_download" "${platform}-${arch}" "true"
    echo -e "${GREEN}📊 Tracked install download: ${platform}-${arch}${NC}"
}

# Generate analytics report
generate_report() {
    echo -e "${BLUE}📊 Campfire Deployment Analytics Report${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""
    
    if [[ ! -f "$LOG_FILE" ]]; then
        echo -e "${YELLOW}⚠️  No analytics data found${NC}"
        echo -e "${YELLOW}   Log file: ${LOG_FILE}${NC}"
        return
    fi
    
    local total_events=$(wc -l < "$LOG_FILE")
    echo -e "${YELLOW}📈 Total Events: ${total_events}${NC}"
    echo ""
    
    # Count deployment clicks
    local deployment_clicks=$(grep "deployment_click" "$LOG_FILE" | wc -l)
    echo -e "${YELLOW}🖱️  Deployment Clicks: ${deployment_clicks}${NC}"
    
    # Count deployment results
    local deployment_attempts=$(grep "deployment_result" "$LOG_FILE" | wc -l)
    local deployment_successes=$(grep "deployment_result.*true" "$LOG_FILE" | wc -l)
    local deployment_failures=$(grep "deployment_result.*false" "$LOG_FILE" | wc -l)
    
    echo -e "${YELLOW}🚀 Deployment Attempts: ${deployment_attempts}${NC}"
    echo -e "${GREEN}✅ Successful Deployments: ${deployment_successes}${NC}"
    echo -e "${RED}❌ Failed Deployments: ${deployment_failures}${NC}"
    
    if [[ $deployment_attempts -gt 0 ]]; then
        local success_rate=$((deployment_successes * 100 / deployment_attempts))
        echo -e "${BLUE}📊 Success Rate: ${success_rate}%${NC}"
    fi
    
    echo ""
    
    # Count install downloads
    local install_downloads=$(grep "install_download" "$LOG_FILE" | wc -l)
    echo -e "${YELLOW}📥 Install Downloads: ${install_downloads}${NC}"
    
    # Platform breakdown
    echo ""
    echo -e "${YELLOW}🖥️  Platform Breakdown:${NC}"
    if grep -q "railway" "$LOG_FILE"; then
        local railway_events=$(grep "railway" "$LOG_FILE" | wc -l)
        echo -e "   🚂 Railway: ${railway_events} events"
    fi
    
    if grep -q "linux" "$LOG_FILE"; then
        local linux_events=$(grep "linux" "$LOG_FILE" | wc -l)
        echo -e "   🐧 Linux: ${linux_events} events"
    fi
    
    if grep -q "darwin" "$LOG_FILE"; then
        local macos_events=$(grep "darwin" "$LOG_FILE" | wc -l)
        echo -e "   🍎 macOS: ${macos_events} events"
    fi
    
    if grep -q "windows" "$LOG_FILE"; then
        local windows_events=$(grep "windows" "$LOG_FILE" | wc -l)
        echo -e "   🪟 Windows: ${windows_events} events"
    fi
    
    echo ""
    echo -e "${BLUE}📅 Recent Activity (Last 10 Events):${NC}"
    tail -10 "$LOG_FILE" | while IFS=',' read -r timestamp event_type platform success; do
        local status_icon="✅"
        if [[ "$success" == "false" ]]; then
            status_icon="❌"
        fi
        echo -e "   ${status_icon} ${timestamp} - ${event_type} (${platform})"
    done
}

# Clear analytics data
clear_analytics() {
    if [[ -f "$LOG_FILE" ]]; then
        rm "$LOG_FILE"
        echo -e "${GREEN}✅ Analytics data cleared${NC}"
    else
        echo -e "${YELLOW}⚠️  No analytics data to clear${NC}"
    fi
}

# Export analytics data
export_analytics() {
    local export_file="${1:-campfire-analytics-$(date +%Y%m%d).csv}"
    
    if [[ ! -f "$LOG_FILE" ]]; then
        echo -e "${YELLOW}⚠️  No analytics data to export${NC}"
        return
    fi
    
    # Add CSV header
    echo "timestamp,event_type,platform,success" > "$export_file"
    cat "$LOG_FILE" >> "$export_file"
    
    echo -e "${GREEN}✅ Analytics exported to: ${export_file}${NC}"
    echo -e "${YELLOW}📊 Total events: $(wc -l < "$LOG_FILE")${NC}"
}

# Show usage
show_usage() {
    echo "Campfire Analytics Tracker"
    echo ""
    echo "Usage:"
    echo "  $0 click [platform]           # Track deployment click"
    echo "  $0 success [platform]         # Track successful deployment"
    echo "  $0 failure [platform]         # Track failed deployment"
    echo "  $0 download                   # Track install script download"
    echo "  $0 report                     # Generate analytics report"
    echo "  $0 export [filename]          # Export data to CSV"
    echo "  $0 clear                      # Clear all analytics data"
    echo ""
    echo "Examples:"
    echo "  $0 click railway              # Track Railway deployment click"
    echo "  $0 success railway            # Track successful Railway deployment"
    echo "  $0 download                   # Track install script download"
    echo "  $0 report                     # Show analytics report"
}

# Main execution
case "${1:-}" in
    click)
        track_deployment_click "${2:-railway}"
        ;;
    success)
        track_deployment_success "${2:-railway}" "true"
        ;;
    failure)
        track_deployment_success "${2:-railway}" "false"
        ;;
    download)
        track_install_download
        ;;
    report)
        generate_report
        ;;
    export)
        export_analytics "$2"
        ;;
    clear)
        clear_analytics
        ;;
    --help|-h|help)
        show_usage
        ;;
    *)
        echo -e "${RED}❌ Unknown command: ${1:-}${NC}"
        echo ""
        show_usage
        exit 1
        ;;
esac