#!/bin/bash

# Campfire Production Deployment Script
# Handles building, testing, and deploying the application

set -euo pipefail

# Configuration
IMAGE_NAME="${CAMPFIRE_IMAGE_NAME:-campfire-on-rust}"
IMAGE_TAG="${CAMPFIRE_IMAGE_TAG:-latest}"
CONTAINER_NAME="${CAMPFIRE_CONTAINER_NAME:-campfire}"
DATA_DIR="${CAMPFIRE_DATA_DIR:-./data}"
LOGS_DIR="${CAMPFIRE_LOGS_DIR:-./logs}"
BACKUPS_DIR="${CAMPFIRE_BACKUPS_DIR:-./backups}"
ENV_FILE="${CAMPFIRE_ENV_FILE:-.env.production}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
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
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Campfire deployment tool"
    echo ""
    echo "Commands:"
    echo "  build      Build Docker image"
    echo "  test       Run tests in container"
    echo "  deploy     Deploy application"
    echo "  start      Start application container"
    echo "  stop       Stop application container"
    echo "  restart    Restart application container"
    echo "  logs       Show application logs"
    echo "  status     Show deployment status"
    echo "  backup     Create database backup"
    echo "  cleanup    Clean up old images and containers"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  --no-cache     Build without Docker cache"
    echo "  --pull         Pull latest base images"
    echo ""
    echo "Environment Variables:"
    echo "  CAMPFIRE_IMAGE_NAME        Docker image name (default: campfire-on-rust)"
    echo "  CAMPFIRE_IMAGE_TAG         Docker image tag (default: latest)"
    echo "  CAMPFIRE_CONTAINER_NAME    Container name (default: campfire)"
    echo "  CAMPFIRE_DATA_DIR          Data directory (default: ./data)"
    echo "  CAMPFIRE_LOGS_DIR          Logs directory (default: ./logs)"
    echo "  CAMPFIRE_BACKUPS_DIR       Backups directory (default: ./backups)"
    echo "  CAMPFIRE_ENV_FILE          Environment file (default: .env.production)"
    exit 1
}

# Check if Docker is available
check_docker() {
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed or not in PATH"
        exit 1
    fi
    
    if ! docker info &> /dev/null; then
        error "Docker daemon is not running"
        exit 1
    fi
}

# Create necessary directories
create_directories() {
    log "Creating necessary directories..."
    mkdir -p "$DATA_DIR" "$LOGS_DIR" "$BACKUPS_DIR"
    
    # Set appropriate permissions
    chmod 755 "$DATA_DIR" "$LOGS_DIR" "$BACKUPS_DIR"
    
    log "Directories created successfully"
}

# Build Docker image
build_image() {
    local no_cache="${1:-false}"
    local pull="${2:-false}"
    
    log "Building Docker image: $IMAGE_NAME:$IMAGE_TAG"
    
    local build_args=""
    if [[ "$no_cache" == "true" ]]; then
        build_args="$build_args --no-cache"
    fi
    
    if [[ "$pull" == "true" ]]; then
        build_args="$build_args --pull"
    fi
    
    # Build the image
    docker build $build_args -t "$IMAGE_NAME:$IMAGE_TAG" .
    
    if [[ $? -eq 0 ]]; then
        log "Docker image built successfully"
        
        # Show image size
        local image_size=$(docker images "$IMAGE_NAME:$IMAGE_TAG" --format "{{.Size}}")
        info "Image size: $image_size"
    else
        error "Failed to build Docker image"
        exit 1
    fi
}

# Run tests in container
run_tests() {
    log "Running tests in container..."
    
    docker run --rm \
        -v "$(pwd)/target:/app/target" \
        "$IMAGE_NAME:$IMAGE_TAG" \
        cargo test --release
    
    if [[ $? -eq 0 ]]; then
        log "All tests passed"
    else
        error "Tests failed"
        exit 1
    fi
}

# Stop existing container
stop_container() {
    if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
        log "Stopping existing container: $CONTAINER_NAME"
        docker stop "$CONTAINER_NAME"
        docker rm "$CONTAINER_NAME"
    fi
}

# Start application container
start_container() {
    log "Starting application container: $CONTAINER_NAME"
    
    # Create environment file if it doesn't exist
    if [[ ! -f "$ENV_FILE" ]]; then
        warn "Environment file not found: $ENV_FILE"
        warn "Creating default environment file..."
        create_default_env_file
    fi
    
    # Start container
    docker run -d \
        --name "$CONTAINER_NAME" \
        --restart unless-stopped \
        -p 3000:3000 \
        -v "$DATA_DIR:/app/data" \
        -v "$LOGS_DIR:/app/logs" \
        -v "$BACKUPS_DIR:/app/backups" \
        --env-file "$ENV_FILE" \
        --health-cmd="/app/healthcheck.sh" \
        --health-interval=30s \
        --health-timeout=10s \
        --health-retries=3 \
        "$IMAGE_NAME:$IMAGE_TAG"
    
    if [[ $? -eq 0 ]]; then
        log "Container started successfully"
        
        # Wait for health check
        log "Waiting for application to be healthy..."
        local attempts=0
        local max_attempts=30
        
        while [[ $attempts -lt $max_attempts ]]; do
            local health_status=$(docker inspect --format='{{.State.Health.Status}}' "$CONTAINER_NAME" 2>/dev/null || echo "unknown")
            
            case "$health_status" in
                "healthy")
                    log "Application is healthy and ready"
                    return 0
                    ;;
                "unhealthy")
                    error "Application health check failed"
                    show_logs 20
                    exit 1
                    ;;
                "starting"|"unknown")
                    info "Health check in progress... ($((attempts + 1))/$max_attempts)"
                    sleep 2
                    ((attempts++))
                    ;;
            esac
        done
        
        warn "Health check timeout, but container is running"
        show_logs 10
    else
        error "Failed to start container"
        exit 1
    fi
}

# Show application logs
show_logs() {
    local lines="${1:-50}"
    
    if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
        log "Showing last $lines lines of logs:"
        docker logs --tail "$lines" -f "$CONTAINER_NAME"
    else
        warn "Container $CONTAINER_NAME is not running"
    fi
}

# Show deployment status
show_status() {
    log "Deployment Status"
    echo "=================="
    
    # Container status
    if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
        info "Container: Running"
        
        local health_status=$(docker inspect --format='{{.State.Health.Status}}' "$CONTAINER_NAME" 2>/dev/null || echo "unknown")
        info "Health: $health_status"
        
        local uptime=$(docker inspect --format='{{.State.StartedAt}}' "$CONTAINER_NAME" 2>/dev/null || echo "unknown")
        info "Started: $uptime"
        
        # Show resource usage
        local stats=$(docker stats --no-stream --format "table {{.CPUPerc}}\t{{.MemUsage}}" "$CONTAINER_NAME" 2>/dev/null | tail -n 1)
        if [[ -n "$stats" ]]; then
            info "Resources: $stats"
        fi
    else
        warn "Container: Not running"
    fi
    
    # Image info
    if docker images -q "$IMAGE_NAME:$IMAGE_TAG" | grep -q .; then
        local image_created=$(docker inspect --format='{{.Created}}' "$IMAGE_NAME:$IMAGE_TAG" 2>/dev/null || echo "unknown")
        local image_size=$(docker images "$IMAGE_NAME:$IMAGE_TAG" --format "{{.Size}}")
        info "Image: $IMAGE_NAME:$IMAGE_TAG ($image_size, created: $image_created)"
    else
        warn "Image: Not found"
    fi
    
    # Directory sizes
    if [[ -d "$DATA_DIR" ]]; then
        local data_size=$(du -sh "$DATA_DIR" 2>/dev/null | cut -f1 || echo "unknown")
        info "Data directory: $data_size"
    fi
    
    if [[ -d "$LOGS_DIR" ]]; then
        local logs_size=$(du -sh "$LOGS_DIR" 2>/dev/null | cut -f1 || echo "unknown")
        info "Logs directory: $logs_size"
    fi
    
    if [[ -d "$BACKUPS_DIR" ]]; then
        local backups_size=$(du -sh "$BACKUPS_DIR" 2>/dev/null | cut -f1 || echo "unknown")
        local backup_count=$(find "$BACKUPS_DIR" -name "*.db*" -type f | wc -l)
        info "Backups directory: $backups_size ($backup_count files)"
    fi
}

# Create database backup
create_backup() {
    log "Creating database backup..."
    
    if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
        docker exec "$CONTAINER_NAME" /app/scripts/backup.sh
    else
        error "Container is not running"
        exit 1
    fi
}

# Clean up old images and containers
cleanup() {
    log "Cleaning up old images and containers..."
    
    # Remove stopped containers
    local stopped_containers=$(docker ps -aq -f status=exited)
    if [[ -n "$stopped_containers" ]]; then
        docker rm $stopped_containers
        log "Removed stopped containers"
    fi
    
    # Remove dangling images
    local dangling_images=$(docker images -q -f dangling=true)
    if [[ -n "$dangling_images" ]]; then
        docker rmi $dangling_images
        log "Removed dangling images"
    fi
    
    # Remove old versions of our image (keep latest 3)
    local old_images=$(docker images "$IMAGE_NAME" --format "{{.ID}}" | tail -n +4)
    if [[ -n "$old_images" ]]; then
        docker rmi $old_images 2>/dev/null || true
        log "Removed old image versions"
    fi
    
    log "Cleanup completed"
}

# Create default environment file
create_default_env_file() {
    cat > "$ENV_FILE" <<EOF
# Campfire Production Configuration
# Generated on $(date)

# Server Configuration
CAMPFIRE_HOST=0.0.0.0
CAMPFIRE_PORT=3000
CAMPFIRE_REQUEST_TIMEOUT=30
CAMPFIRE_MAX_REQUEST_SIZE=16777216
CAMPFIRE_SHUTDOWN_TIMEOUT=30

# Database Configuration
CAMPFIRE_DATABASE_URL=/app/data/campfire.db
CAMPFIRE_DB_MAX_CONNECTIONS=10
CAMPFIRE_DB_TIMEOUT=30
CAMPFIRE_DB_WAL_MODE=true
CAMPFIRE_BACKUP_DIR=/app/backups

# Logging Configuration
CAMPFIRE_LOG_LEVEL=info
CAMPFIRE_LOG_FORMAT=json
CAMPFIRE_LOG_FILE=/app/logs/campfire.log
CAMPFIRE_LOG_STRUCTURED=true
CAMPFIRE_TRACE_REQUESTS=false

# Security Configuration
CAMPFIRE_CORS_ORIGINS=
CAMPFIRE_RATE_LIMIT_RPM=60
CAMPFIRE_SESSION_TOKEN_LENGTH=32
CAMPFIRE_SESSION_EXPIRY_HOURS=24
CAMPFIRE_FORCE_HTTPS=false
CAMPFIRE_TRUST_PROXY=false

# Push Notifications (configure these for production)
CAMPFIRE_PUSH_ENABLED=false
CAMPFIRE_VAPID_PRIVATE_KEY=
CAMPFIRE_VAPID_PUBLIC_KEY=
CAMPFIRE_VAPID_SUBJECT=mailto:admin@campfire.local

# Metrics Configuration
CAMPFIRE_METRICS_ENABLED=true
CAMPFIRE_METRICS_ENDPOINT=/metrics
CAMPFIRE_METRICS_DETAILED=false

# Feature Flags
CAMPFIRE_FEATURE_WEBSOCKETS=true
CAMPFIRE_FEATURE_PUSH=true
CAMPFIRE_FEATURE_BOTS=true
CAMPFIRE_FEATURE_SEARCH=true
CAMPFIRE_FEATURE_SOUNDS=true
CAMPFIRE_FEATURE_FILES=false

# Rust Configuration
RUST_LOG=campfire_on_rust=info,tower_http=info
EOF
    
    log "Created default environment file: $ENV_FILE"
    warn "Please review and customize the configuration before deploying to production"
}

# Deploy application (build + start)
deploy() {
    local no_cache="${1:-false}"
    local pull="${2:-false}"
    
    log "Starting deployment process..."
    
    # Create directories
    create_directories
    
    # Build image
    build_image "$no_cache" "$pull"
    
    # Run tests
    run_tests
    
    # Stop existing container
    stop_container
    
    # Start new container
    start_container
    
    log "Deployment completed successfully!"
    show_status
}

# Main script logic
main() {
    local command="${1:-}"
    local no_cache=false
    local pull=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                ;;
            --no-cache)
                no_cache=true
                shift
                ;;
            --pull)
                pull=true
                shift
                ;;
            *)
                if [[ -z "$command" ]]; then
                    command="$1"
                fi
                shift
                ;;
        esac
    done
    
    # Check Docker availability
    check_docker
    
    # Execute command
    case "$command" in
        build)
            create_directories
            build_image "$no_cache" "$pull"
            ;;
        test)
            run_tests
            ;;
        deploy)
            deploy "$no_cache" "$pull"
            ;;
        start)
            create_directories
            start_container
            ;;
        stop)
            stop_container
            ;;
        restart)
            stop_container
            start_container
            ;;
        logs)
            show_logs
            ;;
        status)
            show_status
            ;;
        backup)
            create_backup
            ;;
        cleanup)
            cleanup
            ;;
        "")
            usage
            ;;
        *)
            error "Unknown command: $command"
            usage
            ;;
    esac
}

# Run main function with all arguments
main "$@"