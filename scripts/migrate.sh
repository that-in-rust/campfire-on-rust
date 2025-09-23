#!/bin/bash

# Campfire Database Migration Script
# Handles database schema migrations and upgrades

set -euo pipefail

# Configuration
DATABASE_PATH="${CAMPFIRE_DATABASE_URL:-campfire.db}"
MIGRATIONS_DIR="${CAMPFIRE_MIGRATIONS_DIR:-./migrations}"
BACKUP_BEFORE_MIGRATE="${CAMPFIRE_BACKUP_BEFORE_MIGRATE:-true}"

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
    echo "Campfire database migration tool"
    echo ""
    echo "Commands:"
    echo "  init       Initialize migration tracking"
    echo "  status     Show migration status"
    echo "  migrate    Run pending migrations"
    echo "  rollback   Rollback last migration"
    echo "  create     Create new migration file"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  --dry-run      Show what would be done without executing"
    echo ""
    echo "Environment Variables:"
    echo "  CAMPFIRE_DATABASE_URL              Database path (default: campfire.db)"
    echo "  CAMPFIRE_MIGRATIONS_DIR            Migrations directory (default: ./migrations)"
    echo "  CAMPFIRE_BACKUP_BEFORE_MIGRATE     Backup before migration (default: true)"
    exit 1
}

# Initialize migration tracking table
init_migrations() {
    log "Initializing migration tracking..."
    
    sqlite3 "$DATABASE_PATH" <<EOF
CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    checksum TEXT NOT NULL
);
EOF
    
    if [[ $? -eq 0 ]]; then
        log "Migration tracking initialized"
    else
        error "Failed to initialize migration tracking"
        exit 1
    fi
}

# Get current schema version
get_current_version() {
    sqlite3 "$DATABASE_PATH" "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1;" 2>/dev/null || echo ""
}

# Get migration checksum
get_migration_checksum() {
    local migration_file="$1"
    sha256sum "$migration_file" | cut -d' ' -f1
}

# Check if migration was already applied
is_migration_applied() {
    local version="$1"
    local count=$(sqlite3 "$DATABASE_PATH" "SELECT COUNT(*) FROM schema_migrations WHERE version = '$version';" 2>/dev/null || echo "0")
    [[ "$count" -gt 0 ]]
}

# Apply single migration
apply_migration() {
    local migration_file="$1"
    local version=$(basename "$migration_file" .sql)
    local checksum=$(get_migration_checksum "$migration_file")
    
    log "Applying migration: $version"
    
    # Start transaction
    sqlite3 "$DATABASE_PATH" <<EOF
BEGIN TRANSACTION;

-- Apply migration
$(cat "$migration_file")

-- Record migration
INSERT INTO schema_migrations (version, checksum) VALUES ('$version', '$checksum');

COMMIT;
EOF
    
    if [[ $? -eq 0 ]]; then
        log "Migration $version applied successfully"
        return 0
    else
        error "Migration $version failed"
        return 1
    fi
}

# Show migration status
show_status() {
    log "Migration Status"
    echo "=================="
    
    # Check if migrations table exists
    if ! sqlite3 "$DATABASE_PATH" "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_migrations';" | grep -q schema_migrations; then
        warn "Migration tracking not initialized. Run 'migrate init' first."
        return 1
    fi
    
    local current_version=$(get_current_version)
    if [[ -n "$current_version" ]]; then
        info "Current version: $current_version"
    else
        info "No migrations applied yet"
    fi
    
    echo ""
    info "Applied migrations:"
    sqlite3 "$DATABASE_PATH" "SELECT version, applied_at FROM schema_migrations ORDER BY version;" | while IFS='|' read -r version applied_at; do
        echo "  ✓ $version (applied: $applied_at)"
    done
    
    echo ""
    info "Pending migrations:"
    local has_pending=false
    
    if [[ -d "$MIGRATIONS_DIR" ]]; then
        for migration_file in "$MIGRATIONS_DIR"/*.sql; do
            if [[ -f "$migration_file" ]]; then
                local version=$(basename "$migration_file" .sql)
                if ! is_migration_applied "$version"; then
                    echo "  ○ $version"
                    has_pending=true
                fi
            fi
        done
    fi
    
    if [[ "$has_pending" == "false" ]]; then
        echo "  No pending migrations"
    fi
}

# Run migrations
run_migrations() {
    local dry_run="${1:-false}"
    
    # Check if migrations table exists
    if ! sqlite3 "$DATABASE_PATH" "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_migrations';" | grep -q schema_migrations; then
        warn "Migration tracking not initialized. Initializing now..."
        init_migrations
    fi
    
    # Create backup if requested
    if [[ "$BACKUP_BEFORE_MIGRATE" == "true" && "$dry_run" == "false" ]]; then
        log "Creating backup before migration..."
        local backup_script="$(dirname "$0")/backup.sh"
        if [[ -f "$backup_script" ]]; then
            "$backup_script"
        else
            warn "Backup script not found, skipping backup"
        fi
    fi
    
    # Find and apply pending migrations
    local migrations_applied=0
    
    if [[ ! -d "$MIGRATIONS_DIR" ]]; then
        warn "Migrations directory not found: $MIGRATIONS_DIR"
        return 0
    fi
    
    for migration_file in "$MIGRATIONS_DIR"/*.sql; do
        if [[ -f "$migration_file" ]]; then
            local version=$(basename "$migration_file" .sql)
            
            if ! is_migration_applied "$version"; then
                if [[ "$dry_run" == "true" ]]; then
                    info "Would apply migration: $version"
                else
                    apply_migration "$migration_file"
                    if [[ $? -eq 0 ]]; then
                        ((migrations_applied++))
                    else
                        error "Migration failed, stopping"
                        exit 1
                    fi
                fi
            fi
        fi
    done
    
    if [[ "$dry_run" == "true" ]]; then
        log "Dry run completed"
    elif [[ $migrations_applied -eq 0 ]]; then
        log "No pending migrations to apply"
    else
        log "Applied $migrations_applied migration(s) successfully"
    fi
}

# Create new migration file
create_migration() {
    local name="$1"
    
    if [[ -z "$name" ]]; then
        error "Migration name is required"
        echo "Usage: $0 create <migration_name>"
        exit 1
    fi
    
    # Create migrations directory if it doesn't exist
    mkdir -p "$MIGRATIONS_DIR"
    
    # Generate timestamp-based version
    local timestamp=$(date +'%Y%m%d_%H%M%S')
    local filename="${timestamp}_${name}.sql"
    local filepath="$MIGRATIONS_DIR/$filename"
    
    # Create migration template
    cat > "$filepath" <<EOF
-- Migration: $name
-- Created: $(date)
-- Description: Add description here

-- Up migration
BEGIN TRANSACTION;

-- Add your migration SQL here
-- Example:
-- CREATE TABLE new_table (
--     id TEXT PRIMARY KEY,
--     name TEXT NOT NULL,
--     created_at DATETIME DEFAULT CURRENT_TIMESTAMP
-- );

COMMIT;
EOF
    
    log "Created migration file: $filepath"
    info "Edit the file to add your migration SQL"
}

# Rollback last migration
rollback_migration() {
    local dry_run="${1:-false}"
    
    local current_version=$(get_current_version)
    if [[ -z "$current_version" ]]; then
        warn "No migrations to rollback"
        return 0
    fi
    
    if [[ "$dry_run" == "true" ]]; then
        info "Would rollback migration: $current_version"
        return 0
    fi
    
    warn "Rolling back migration: $current_version"
    warn "Note: This will only remove the migration record, not reverse the changes"
    
    read -p "Are you sure you want to rollback? (y/N): " confirm
    if [[ "$confirm" != "y" && "$confirm" != "Y" ]]; then
        log "Rollback cancelled"
        return 0
    fi
    
    sqlite3 "$DATABASE_PATH" "DELETE FROM schema_migrations WHERE version = '$current_version';"
    
    if [[ $? -eq 0 ]]; then
        log "Migration $current_version rolled back"
    else
        error "Failed to rollback migration"
        exit 1
    fi
}

# Main script logic
main() {
    local command="${1:-}"
    local dry_run=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                ;;
            --dry-run)
                dry_run=true
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
    
    # Check if database exists
    if [[ ! -f "$DATABASE_PATH" && "$command" != "init" ]]; then
        error "Database file not found: $DATABASE_PATH"
        exit 1
    fi
    
    # Execute command
    case "$command" in
        init)
            init_migrations
            ;;
        status)
            show_status
            ;;
        migrate)
            run_migrations "$dry_run"
            ;;
        rollback)
            rollback_migration "$dry_run"
            ;;
        create)
            create_migration "$2"
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