#!/bin/bash

# Campfire Performance Optimization Script
# Optimizes system settings, database, and application configuration for maximum performance

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DATABASE_PATH="${CAMPFIRE_DATABASE_PATH:-./data/campfire.db}"
BACKUP_DIR="${CAMPFIRE_BACKUP_DIR:-./backups}"
LOG_FILE="${CAMPFIRE_LOG_FILE:-./logs/optimization.log}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO:${NC} $1" | tee -a "$LOG_FILE"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" | tee -a "$LOG_FILE" >&2
}

# Usage function
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Optimize Campfire performance settings"
    echo ""
    echo "Options:"
    echo "  --database-only     Only optimize database settings"
    echo "  --system-only       Only optimize system settings"
    echo "  --app-only          Only optimize application settings"
    echo "  --benchmark         Run performance benchmarks after optimization"
    echo "  --backup            Create backup before optimization"
    echo "  --dry-run           Show what would be done without making changes"
    echo "  -h, --help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                  # Full optimization"
    echo "  $0 --database-only  # Only optimize database"
    echo "  $0 --benchmark      # Optimize and benchmark"
    exit 1
}

# Parse command line arguments
DATABASE_ONLY=false
SYSTEM_ONLY=false
APP_ONLY=false
BENCHMARK=false
BACKUP=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --database-only)
            DATABASE_ONLY=true
            shift
            ;;
        --system-only)
            SYSTEM_ONLY=true
            shift
            ;;
        --app-only)
            APP_ONLY=true
            shift
            ;;
        --benchmark)
            BENCHMARK=true
            shift
            ;;
        --backup)
            BACKUP=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            error "Unknown option: $1"
            usage
            ;;
    esac
done

# Create necessary directories
mkdir -p "$(dirname "$LOG_FILE")"
mkdir -p "$BACKUP_DIR"

# Check if running as root for system optimizations
check_root() {
    if [[ $EUID -eq 0 ]]; then
        warn "Running as root - system optimizations will be applied"
        return 0
    else
        warn "Not running as root - system optimizations will be skipped"
        return 1
    fi
}

# Create backup if requested
create_backup() {
    if [[ "$BACKUP" == "true" ]]; then
        log "Creating backup before optimization..."
        
        local backup_timestamp=$(date +%Y%m%d_%H%M%S)
        local backup_file="$BACKUP_DIR/campfire_backup_$backup_timestamp.tar.gz"
        
        if [[ "$DRY_RUN" == "true" ]]; then
            info "DRY RUN: Would create backup at $backup_file"
        else
            tar -czf "$backup_file" \
                --exclude="target" \
                --exclude="node_modules" \
                --exclude=".git" \
                "$PROJECT_ROOT"
            
            log "Backup created: $backup_file"
        fi
    fi
}

# Optimize SQLite database settings
optimize_database() {
    log "Optimizing database performance..."
    
    if [[ ! -f "$DATABASE_PATH" ]]; then
        warn "Database file not found at $DATABASE_PATH - skipping database optimization"
        return
    fi
    
    local optimization_sql="
-- Performance optimization settings
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = 20000;
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 1073741824;
PRAGMA busy_timeout = 30000;
PRAGMA foreign_keys = ON;
PRAGMA recursive_triggers = ON;

-- Analyze tables for better query planning
ANALYZE;

-- Optimize database
PRAGMA optimize;

-- Create performance indexes if they don't exist
CREATE INDEX IF NOT EXISTS idx_messages_room_created_desc 
ON messages(room_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_messages_creator_created 
ON messages(creator_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_messages_client_id_room 
ON messages(client_message_id, room_id);

CREATE INDEX IF NOT EXISTS idx_room_memberships_user_room 
ON room_memberships(user_id, room_id);

CREATE INDEX IF NOT EXISTS idx_room_memberships_room_user 
ON room_memberships(room_id, user_id);

CREATE INDEX IF NOT EXISTS idx_sessions_token_expires 
ON sessions(token, expires_at);

CREATE INDEX IF NOT EXISTS idx_sessions_user_expires 
ON sessions(user_id, expires_at);

CREATE INDEX IF NOT EXISTS idx_push_subscriptions_user 
ON push_subscriptions(user_id);

-- Rebuild FTS index for optimal performance
INSERT INTO messages_fts(messages_fts) VALUES('rebuild');
"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        info "DRY RUN: Would execute database optimization SQL"
        echo "$optimization_sql"
    else
        echo "$optimization_sql" | sqlite3 "$DATABASE_PATH"
        log "Database optimization completed"
        
        # Get database statistics
        local db_size=$(du -h "$DATABASE_PATH" | cut -f1)
        local page_count=$(sqlite3 "$DATABASE_PATH" "PRAGMA page_count;")
        local page_size=$(sqlite3 "$DATABASE_PATH" "PRAGMA page_size;")
        
        info "Database size: $db_size"
        info "Pages: $page_count (${page_size} bytes each)"
    fi
}

# Optimize system settings for performance
optimize_system() {
    log "Optimizing system performance settings..."
    
    if ! check_root; then
        warn "Skipping system optimizations (not running as root)"
        return
    fi
    
    local sysctl_optimizations="
# Network optimizations
net.core.somaxconn = 65536
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_max_syn_backlog = 65536
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_keepalive_time = 1200
net.ipv4.tcp_keepalive_probes = 3
net.ipv4.tcp_keepalive_intvl = 15
net.ipv4.tcp_tw_reuse = 1

# Memory optimizations
vm.swappiness = 10
vm.dirty_ratio = 15
vm.dirty_background_ratio = 5
vm.overcommit_memory = 1

# File descriptor limits
fs.file-max = 2097152
"
    
    local limits_config="
# File descriptor limits for campfire user
campfire soft nofile 65536
campfire hard nofile 65536
campfire soft nproc 32768
campfire hard nproc 32768
"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        info "DRY RUN: Would apply system optimizations:"
        echo "$sysctl_optimizations"
        echo "$limits_config"
    else
        # Apply sysctl settings
        echo "$sysctl_optimizations" > /etc/sysctl.d/99-campfire-performance.conf
        sysctl -p /etc/sysctl.d/99-campfire-performance.conf
        
        # Apply limits
        echo "$limits_config" >> /etc/security/limits.conf
        
        log "System optimizations applied"
    fi
}

# Optimize application configuration
optimize_application() {
    log "Optimizing application configuration..."
    
    local env_file="$PROJECT_ROOT/.env"
    local optimized_config="
# Performance optimizations
RUST_LOG=campfire_on_rust=info,warn,error
RUST_BACKTRACE=0

# Database optimizations
CAMPFIRE_DB_MAX_CONNECTIONS=50
CAMPFIRE_DB_MIN_CONNECTIONS=5
CAMPFIRE_DB_ACQUIRE_TIMEOUT=30
CAMPFIRE_DB_IDLE_TIMEOUT=600
CAMPFIRE_DB_MAX_LIFETIME=1800

# WebSocket optimizations
CAMPFIRE_WS_MAX_CONNECTIONS_PER_USER=10
CAMPFIRE_WS_PRESENCE_TIMEOUT=60
CAMPFIRE_WS_TYPING_TIMEOUT=10
CAMPFIRE_WS_CLEANUP_INTERVAL=30

# Cache optimizations
CAMPFIRE_CACHE_SIZE=10000
CAMPFIRE_CACHE_TTL=300

# Memory optimizations
MALLOC_ARENA_MAX=2
"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        info "DRY RUN: Would update application configuration:"
        echo "$optimized_config"
    else
        # Backup existing .env file
        if [[ -f "$env_file" ]]; then
            cp "$env_file" "$env_file.backup.$(date +%Y%m%d_%H%M%S)"
        fi
        
        # Add optimized configuration
        echo "$optimized_config" >> "$env_file"
        
        log "Application configuration optimized"
    fi
}

# Compile with optimizations
optimize_compilation() {
    log "Optimizing compilation settings..."
    
    local cargo_config="$PROJECT_ROOT/.cargo/config.toml"
    local optimized_cargo_config="
[build]
rustflags = [\"-C\", \"target-cpu=native\", \"-C\", \"opt-level=3\"]

[profile.release]
lto = \"fat\"
codegen-units = 1
panic = \"abort\"
strip = true
opt-level = 3

[profile.release-optimized]
inherits = \"release\"
lto = \"fat\"
codegen-units = 1
"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        info "DRY RUN: Would create optimized Cargo configuration"
    else
        mkdir -p "$(dirname "$cargo_config")"
        echo "$optimized_cargo_config" > "$cargo_config"
        
        log "Compilation optimizations configured"
    fi
}

# Run performance benchmarks
run_benchmarks() {
    log "Running performance benchmarks..."
    
    cd "$PROJECT_ROOT"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        info "DRY RUN: Would run performance benchmarks"
        return
    fi
    
    # Build with optimizations
    cargo build --release --features performance-monitoring
    
    # Run benchmarks
    if command -v cargo-criterion &> /dev/null; then
        cargo criterion --features performance-monitoring
    else
        warn "cargo-criterion not installed, running basic benchmarks"
        cargo bench --features performance-monitoring
    fi
    
    # Run application-specific performance tests
    cargo test --release --features performance-monitoring performance_
    
    log "Benchmarks completed"
}

# Monitor performance after optimization
monitor_performance() {
    log "Starting performance monitoring..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        info "DRY RUN: Would start performance monitoring"
        return
    fi
    
    # Start the application in the background for monitoring
    cargo run --release --features performance-monitoring &
    local app_pid=$!
    
    # Wait for application to start
    sleep 5
    
    # Run performance monitoring script
    if [[ -f "$SCRIPT_DIR/performance-monitor.sh" ]]; then
        "$SCRIPT_DIR/performance-monitor.sh" --duration 60 --interval 5
    fi
    
    # Stop the application
    kill $app_pid 2>/dev/null || true
    
    log "Performance monitoring completed"
}

# Generate optimization report
generate_report() {
    log "Generating optimization report..."
    
    local report_file="$PROJECT_ROOT/performance_optimization_report.md"
    local timestamp=$(date +'%Y-%m-%d %H:%M:%S')
    
    cat > "$report_file" <<EOF
# Campfire Performance Optimization Report

**Generated:** $timestamp

## Optimizations Applied

### Database Optimizations
- WAL mode enabled for better concurrency
- Cache size increased to 80MB (20,000 pages)
- Memory-mapped I/O enabled (1GB)
- Performance indexes created
- FTS index rebuilt

### System Optimizations
$(if check_root; then
    echo "- Network buffer sizes increased"
    echo "- TCP settings optimized"
    echo "- File descriptor limits raised"
    echo "- Memory management tuned"
else
    echo "- Skipped (not running as root)"
fi)

### Application Optimizations
- Database connection pool optimized
- WebSocket connection limits configured
- Cache settings tuned
- Memory allocator optimized

### Compilation Optimizations
- Link-time optimization enabled
- Native CPU targeting enabled
- Debug symbols stripped
- Panic handling optimized

## Performance Metrics

$(if [[ "$BENCHMARK" == "true" ]]; then
    echo "Benchmark results available in target/criterion/"
else
    echo "Benchmarks not run (use --benchmark flag)"
fi)

## Recommendations

1. Monitor application performance using the built-in metrics endpoint
2. Adjust connection pool settings based on actual load
3. Consider using jemalloc for better memory allocation
4. Enable profiling features for detailed performance analysis

## Next Steps

1. Deploy optimized configuration to production
2. Monitor performance metrics continuously
3. Adjust settings based on real-world usage patterns
4. Consider horizontal scaling if needed

EOF

    log "Optimization report generated: $report_file"
}

# Main optimization function
main() {
    log "Starting Campfire performance optimization..."
    
    # Create backup if requested
    create_backup
    
    # Run optimizations based on flags
    if [[ "$DATABASE_ONLY" == "true" ]]; then
        optimize_database
    elif [[ "$SYSTEM_ONLY" == "true" ]]; then
        optimize_system
    elif [[ "$APP_ONLY" == "true" ]]; then
        optimize_application
        optimize_compilation
    else
        # Full optimization
        optimize_database
        optimize_system
        optimize_application
        optimize_compilation
    fi
    
    # Run benchmarks if requested
    if [[ "$BENCHMARK" == "true" ]]; then
        run_benchmarks
        monitor_performance
    fi
    
    # Generate report
    generate_report
    
    log "Performance optimization completed successfully!"
    
    if [[ "$DRY_RUN" == "false" ]]; then
        info "Restart the application to apply all optimizations"
        info "Monitor performance using: curl http://localhost:3000/api/performance/summary"
    fi
}

# Run main function
main "$@"