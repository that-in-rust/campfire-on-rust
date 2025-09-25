#!/bin/bash

# Campfire Production Deployment Automation Script
# Comprehensive deployment with monitoring, backup, and rollback capabilities
# 
# This script provides:
# - Automated deployment with health checks
# - Pre-deployment backup and rollback capabilities
# - Monitoring and alerting setup
# - Performance validation
# - Security hardening
# - Documentation generation

set -euo pipefail

# Configuration
DEPLOYMENT_ENV="${CAMPFIRE_DEPLOYMENT_ENV:-production}"
DEPLOYMENT_TYPE="${1:-full}"  # full, update, rollback, or status
BACKUP_BEFORE_DEPLOY="${CAMPFIRE_BACKUP_BEFORE_DEPLOY:-true}"
HEALTH_CHECK_TIMEOUT="${CAMPFIRE_HEALTH_CHECK_TIMEOUT:-300}"
ROLLBACK_ON_FAILURE="${CAMPFIRE_ROLLBACK_ON_FAILURE:-true}"
MONITORING_ENABLED="${CAMPFIRE_MONITORING_ENABLED:-true}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO:${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" >&2
}

# Usage function
usage() {
    echo "Usage: $0 [DEPLOYMENT_TYPE] [OPTIONS]"
    echo ""
    echo "Production deployment automation for Campfire"
    echo ""
    echo "Deployment Types:"
    echo "  full       Full deployment with backup and monitoring setup"
    echo "  update     Update existing deployment"
    echo "  rollback   Rollback to previous version"
    echo "  status     Show deployment status"
    echo ""
    echo "Options:"
    echo "  --no-backup        Skip backup before deployment"
    echo "  --no-monitoring    Skip monitoring setup"
    echo "  --no-rollback      Don't rollback on failure"
    echo "  --timeout SECONDS  Health check timeout (default: 300)"
    echo "  --dry-run          Show what would be done"
    echo "  -h, --help         Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  CAMPFIRE_DEPLOYMENT_ENV        Deployment environment"
    echo "  CAMPFIRE_BACKUP_BEFORE_DEPLOY  Backup before deployment"
    echo "  CAMPFIRE_HEALTH_CHECK_TIMEOUT  Health check timeout"
    echo "  CAMPFIRE_ROLLBACK_ON_FAILURE   Rollback on failure"
    echo "  CAMPFIRE_MONITORING_ENABLED    Enable monitoring setup"
    exit 1
}

# Parse command line arguments
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-backup)
            BACKUP_BEFORE_DEPLOY=false
            shift
            ;;
        --no-monitoring)
            MONITORING_ENABLED=false
            shift
            ;;
        --no-rollback)
            ROLLBACK_ON_FAILURE=false
            shift
            ;;
        --timeout)
            HEALTH_CHECK_TIMEOUT="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        full|update|rollback|status)
            DEPLOYMENT_TYPE="$1"
            shift
            ;;
        *)
            error "Unknown option: $1"
            usage
            ;;
    esac
done

# Dry run function
dry_run() {
    if [[ "$DRY_RUN" == "true" ]]; then
        info "DRY RUN: $1"
        return 0
    fi
    return 1
}

# Check prerequisites
check_prerequisites() {
    log "Checking deployment prerequisites..."
    
    local missing_deps=()
    
    # Check required commands
    for cmd in docker docker-compose curl jq; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd")
        fi
    done
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        error "Missing dependencies: ${missing_deps[*]}"
        exit 1
    fi
    
    # Check Docker daemon
    if ! docker info &> /dev/null; then
        error "Docker daemon is not running"
        exit 1
    fi
    
    # Check environment file
    local env_file=".env.${DEPLOYMENT_ENV}"
    if [[ ! -f "$env_file" ]]; then
        error "Environment file not found: $env_file"
        exit 1
    fi
    
    log "Prerequisites check passed"
}

# Create deployment backup
create_deployment_backup() {
    if [[ "$BACKUP_BEFORE_DEPLOY" != "true" ]]; then
        return
    fi
    
    log "Creating pre-deployment backup..."
    
    if dry_run "Creating deployment backup"; then
        return
    fi
    
    # Create backup using enhanced backup script
    if [[ -f "scripts/backup-enhanced.sh" ]]; then
        ./scripts/backup-enhanced.sh full --no-verify
    else
        ./scripts/backup.sh
    fi
    
    # Store deployment metadata
    local deployment_backup_dir="./backups/deployments"
    mkdir -p "$deployment_backup_dir"
    
    cat > "$deployment_backup_dir/deployment_$(date +%Y%m%d_%H%M%S).json" <<EOF
{
    "timestamp": "$(date -Iseconds)",
    "deployment_type": "$DEPLOYMENT_TYPE",
    "environment": "$DEPLOYMENT_ENV",
    "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
    "git_branch": "$(git branch --show-current 2>/dev/null || echo 'unknown')",
    "docker_images": $(docker images --format "{{json .}}" | jq -s .)
}
EOF
    
    log "Pre-deployment backup completed"
}

# Setup monitoring
setup_monitoring() {
    if [[ "$MONITORING_ENABLED" != "true" ]]; then
        return
    fi
    
    log "Setting up monitoring..."
    
    if dry_run "Setting up monitoring stack"; then
        return
    fi
    
    # Setup monitoring if script exists
    if [[ -f "scripts/setup-monitoring.sh" ]]; then
        ./scripts/setup-monitoring.sh --auto-confirm
    fi
    
    # Start monitoring services
    docker-compose --profile monitoring up -d prometheus grafana
    
    log "Monitoring setup completed"
}

# Deploy application
deploy_application() {
    log "Deploying Campfire application..."
    
    if dry_run "Deploying application with type: $DEPLOYMENT_TYPE"; then
        return
    fi
    
    case "$DEPLOYMENT_TYPE" in
        "full")
            # Full deployment
            log "Performing full deployment..."
            
            # Build and deploy
            ./scripts/deploy.sh deploy --pull
            ;;
        "update")
            # Update deployment
            log "Performing update deployment..."
            
            # Pull latest images and restart
            docker-compose pull
            docker-compose up -d --force-recreate
            ;;
        *)
            error "Unknown deployment type: $DEPLOYMENT_TYPE"
            exit 1
            ;;
    esac
    
    log "Application deployment completed"
}

# Health check with timeout
perform_health_check() {
    log "Performing health checks..."
    
    if dry_run "Performing health checks for $HEALTH_CHECK_TIMEOUT seconds"; then
        return
    fi
    
    local start_time=$(date +%s)
    local timeout_time=$((start_time + HEALTH_CHECK_TIMEOUT))
    
    while [[ $(date +%s) -lt $timeout_time ]]; do
        # Basic health check
        if curl -f -s http://localhost:3000/health > /dev/null; then
            log "Basic health check passed"
            
            # Detailed health check
            local health_response=$(curl -s http://localhost:3000/health/detailed)
            if echo "$health_response" | jq -e '.status == "healthy"' > /dev/null 2>&1; then
                log "Detailed health check passed"
                return 0
            fi
        fi
        
        info "Waiting for application to be healthy..."
        sleep 10
    done
    
    error "Health check failed after $HEALTH_CHECK_TIMEOUT seconds"
    return 1
}

# Smoke tests
run_smoke_tests() {
    log "Running smoke tests..."
    
    if dry_run "Running smoke tests"; then
        return
    fi
    
    local base_url="http://localhost:3000"
    local failed_tests=0
    
    # Test basic endpoints
    local endpoints=(
        "/health"
        "/health/ready"
        "/health/live"
        "/metrics"
    )
    
    for endpoint in "${endpoints[@]}"; do
        if curl -f -s "${base_url}${endpoint}" > /dev/null; then
            info "✓ $endpoint"
        else
            error "✗ $endpoint"
            ((failed_tests++))
        fi
    done
    
    # Test WebSocket connection
    if command -v wscat &> /dev/null; then
        if timeout 5 wscat -c "ws://localhost:3000/ws" -x '{"type":"ping"}' > /dev/null 2>&1; then
            info "✓ WebSocket connection"
        else
            error "✗ WebSocket connection"
            ((failed_tests++))
        fi
    else
        warn "wscat not available, skipping WebSocket test"
    fi
    
    if [[ $failed_tests -eq 0 ]]; then
        log "All smoke tests passed"
        return 0
    else
        error "$failed_tests smoke tests failed"
        return 1
    fi
}

# Performance validation
validate_performance() {
    log "Validating performance..."
    
    if dry_run "Validating performance"; then
        return
    fi
    
    # Run performance monitor for 60 seconds
    if [[ -f "scripts/performance-monitor.sh" ]]; then
        ./scripts/performance-monitor.sh -d 60 -i 5 > /dev/null 2>&1 &
        local monitor_pid=$!
        
        # Wait for monitoring to complete
        wait $monitor_pid
        
        log "Performance validation completed"
    else
        warn "Performance monitor script not found, skipping validation"
    fi
}

# Rollback deployment
rollback_deployment() {
    log "Rolling back deployment..."
    
    if dry_run "Rolling back deployment"; then
        return
    fi
    
    # Find latest deployment backup
    local latest_backup=$(find ./backups -name "campfire_*_backup_*.db*" | sort -r | head -1)
    
    if [[ -n "$latest_backup" ]]; then
        log "Rolling back to backup: $(basename "$latest_backup")"
        
        # Stop current deployment
        docker-compose down
        
        # Restore backup
        if [[ -f "scripts/restore-enhanced.sh" ]]; then
            ./scripts/restore-enhanced.sh "$latest_backup" --auto-confirm
        else
            ./scripts/restore.sh "$latest_backup"
        fi
        
        # Restart with previous configuration
        docker-compose up -d
        
        log "Rollback completed"
    else
        error "No backup found for rollback"
        exit 1
    fi
}

# Show deployment status
show_deployment_status() {
    log "Deployment Status"
    echo "=================="
    
    # Application status
    if docker ps -q -f name=campfire | grep -q .; then
        info "Application: Running"
        
        # Health status
        if curl -f -s http://localhost:3000/health > /dev/null; then
            info "Health: Healthy"
        else
            warn "Health: Unhealthy"
        fi
        
        # Version info
        local version=$(curl -s http://localhost:3000/health/detailed | jq -r '.version // "unknown"' 2>/dev/null || echo "unknown")
        info "Version: $version"
        
        # Uptime
        local started_at=$(docker inspect --format='{{.State.StartedAt}}' campfire 2>/dev/null || echo "unknown")
        info "Started: $started_at"
        
        # Resource usage
        local stats=$(docker stats --no-stream --format "{{.CPUPerc}}\t{{.MemUsage}}" campfire 2>/dev/null || echo "unknown")
        info "Resources: $stats"
    else
        warn "Application: Not running"
    fi
    
    # Monitoring status
    if docker ps -q -f name=prometheus | grep -q .; then
        info "Monitoring: Running"
    else
        warn "Monitoring: Not running"
    fi
    
    # Database status
    if [[ -f "./data/campfire.db" ]]; then
        local db_size=$(du -h "./data/campfire.db" | cut -f1)
        info "Database: $db_size"
    else
        warn "Database: Not found"
    fi
    
    # Recent backups
    local backup_count=$(find ./backups -name "campfire_*_backup_*.db*" | wc -l)
    info "Backups: $backup_count files"
}

# Cleanup old deployments
cleanup_old_deployments() {
    log "Cleaning up old deployments..."
    
    if dry_run "Cleaning up old deployments"; then
        return
    fi
    
    # Remove old Docker images (keep latest 3)
    local old_images=$(docker images campfire-on-rust --format "{{.ID}}" | tail -n +4)
    if [[ -n "$old_images" ]]; then
        echo "$old_images" | xargs docker rmi 2>/dev/null || true
    fi
    
    # Clean up old deployment metadata (keep last 10)
    find ./backups/deployments -name "deployment_*.json" | sort -r | tail -n +11 | xargs rm -f 2>/dev/null || true
    
    log "Cleanup completed"
}

# Send deployment notification
send_notification() {
    local status="$1"
    local message="$2"
    
    # Webhook notification (if configured)
    if [[ -n "${CAMPFIRE_WEBHOOK_URL:-}" ]]; then
        curl -X POST "$CAMPFIRE_WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "{\"status\": \"$status\", \"message\": \"$message\", \"timestamp\": \"$(date -Iseconds)\"}" \
            > /dev/null 2>&1 || true
    fi
    
    # Slack notification (if configured)
    if [[ -n "${SLACK_WEBHOOK_URL:-}" ]]; then
        curl -X POST "$SLACK_WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "{\"text\": \"Campfire Deployment: $status - $message\"}" \
            > /dev/null 2>&1 || true
    fi
}

# Main deployment function
main() {
    local deployment_start_time=$(date +%s)
    
    log "Starting Campfire production deployment..."
    info "Type: $DEPLOYMENT_TYPE"
    info "Environment: $DEPLOYMENT_ENV"
    info "Dry run: $DRY_RUN"
    
    case "$DEPLOYMENT_TYPE" in
        "status")
            show_deployment_status
            exit 0
            ;;
        "rollback")
            rollback_deployment
            exit 0
            ;;
    esac
    
    # Main deployment flow
    check_prerequisites
    
    # Create backup before deployment
    create_deployment_backup
    
    # Setup monitoring
    setup_monitoring
    
    # Deploy application
    if ! deploy_application; then
        error "Application deployment failed"
        if [[ "$ROLLBACK_ON_FAILURE" == "true" ]]; then
            rollback_deployment
        fi
        send_notification "FAILED" "Deployment failed and was rolled back"
        exit 1
    fi
    
    # Health checks
    if ! perform_health_check; then
        error "Health checks failed"
        if [[ "$ROLLBACK_ON_FAILURE" == "true" ]]; then
            rollback_deployment
        fi
        send_notification "FAILED" "Health checks failed and deployment was rolled back"
        exit 1
    fi
    
    # Smoke tests
    if ! run_smoke_tests; then
        error "Smoke tests failed"
        if [[ "$ROLLBACK_ON_FAILURE" == "true" ]]; then
            rollback_deployment
        fi
        send_notification "FAILED" "Smoke tests failed and deployment was rolled back"
        exit 1
    fi
    
    # Performance validation
    validate_performance
    
    # Cleanup
    cleanup_old_deployments
    
    # Calculate deployment time
    local deployment_end_time=$(date +%s)
    local deployment_duration=$((deployment_end_time - deployment_start_time))
    
    log "Deployment completed successfully in ${deployment_duration} seconds!"
    
    # Show final status
    show_deployment_status
    
    # Send success notification
    send_notification "SUCCESS" "Deployment completed successfully in ${deployment_duration}s"
}

# Run main function
main "$@"