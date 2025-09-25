#!/bin/bash

# Campfire Enhanced Database Restore Script
# Restores database from backup with verification and rollback capability

set -euo pipefail

# Configuration
BACKUP_DIR="${CAMPFIRE_BACKUP_DIR:-./backups}"
DATABASE_URL="${CAMPFIRE_DATABASE_URL:-./data/campfire.db}"
BACKUP_FILE="${1:-}"
DRY_RUN="${CAMPFIRE_RESTORE_DRY_RUN:-false}"
AUTO_CONFIRM="${CAMPFIRE_RESTORE_AUTO_CONFIRM:-false}"

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
    echo "Usage: $0 [BACKUP_FILE] [OPTIONS]"
    echo ""
    echo "Restore Campfire database from backup"
    echo ""
    echo "Arguments:"
    echo "  BACKUP_FILE    Backup file to restore (or 'latest' for most recent)"
    echo ""
    echo "Options:"
    echo "  --dry-run      Show what would be done without executing"
    echo "  --auto-confirm Skip confirmation prompts"
    echo "  --list         List available backups"
    echo "  --verify-only  Only verify backup integrity"
    echo "  --rollback     Rollback to pre-restore backup"
    echo "  -h, --help     Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # List available backups"
    echo "  $0 backup_20240101_120000.db.gz     # Restore specific backup"
    echo "  $0 latest                            # Restore latest backup"
    echo "  $0 --list                            # List available backups"
    echo "  $0 backup.db --dry-run               # Test restore without executing"
    echo "  $0 --rollback                        # Rollback to pre-restore backup"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --auto-confirm)
            AUTO_CONFIRM=true
            shift
            ;;
        --list)
            BACKUP_FILE="--list"
            shift
            ;;
        --verify-only)
            BACKUP_FILE="--verify-only"
            shift
            ;;
        --rollback)
            BACKUP_FILE="--rollback"
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            if [[ -z "$BACKUP_FILE" ]]; then
                BACKUP_FILE="$1"
            fi
            shift
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

# List available backups with enhanced information
list_backups() {
    log "Available backups in $BACKUP_DIR:"
    
    if [[ ! -d "$BACKUP_DIR" ]]; then
        warn "Backup directory not found: $BACKUP_DIR"
        return 1
    fi
    
    local backups=($(find "$BACKUP_DIR" -name "campfire_*_backup_*.db*" -type f | sort -r))
    
    if [[ ${#backups[@]} -eq 0 ]]; then
        warn "No backups found"
        return 1
    fi
    
    echo ""
    printf "%-5s %-40s %-8s %-12s %-20s %-10s\n" "No." "Filename" "Type" "Size" "Date" "Status"
    echo "----------------------------------------------------------------------------------------"
    
    local i=1
    for backup in "${backups[@]}"; do
        local filename=$(basename "$backup")
        local size=$(du -h "$backup" | cut -f1)
        local date=$(stat -c %y "$backup" 2>/dev/null | cut -d' ' -f1,2 | cut -d'.' -f1 || echo "unknown")
        
        # Extract backup type from filename
        local backup_type="full"
        if [[ "$filename" =~ _incremental_ ]]; then
            backup_type="incr"
        elif [[ "$filename" =~ _schema_ ]]; then
            backup_type="schema"
        elif [[ "$filename" =~ _hot_ ]]; then
            backup_type="hot"
        fi
        
        # Check if metadata exists
        local status="OK"
        local meta_file="${backup}.meta"
        if [[ ! -f "$meta_file" ]]; then
            status="No Meta"
        fi
        
        printf "%-5s %-40s %-8s %-12s %-20s %-10s\n" "$i" "$filename" "$backup_type" "$size" "$date" "$status"
        ((i++))
    done
    
    echo ""
    echo "Use: $0 <filename> to restore a specific backup"
    echo "Use: $0 latest to restore the most recent backup"
}

# Get backup file path with enhanced logic
get_backup_path() {
    local input="$1"
    
    # Handle "latest" keyword
    if [[ "$input" == "latest" ]]; then
        local latest=$(find "$BACKUP_DIR" -name "campfire_*_backup_*.db*" -type f | sort -r | head -1)
        if [[ -z "$latest" ]]; then
            error "No backups found"
            exit 1
        fi
        echo "$latest"
        return
    fi
    
    # Handle full path
    if [[ -f "$input" ]]; then
        echo "$input"
        return
    fi
    
    # Handle filename in backup directory
    local full_path="$BACKUP_DIR/$input"
    if [[ -f "$full_path" ]]; then
        echo "$full_path"
        return
    fi
    
    # Try to find partial matches
    local matches=($(find "$BACKUP_DIR" -name "*$input*" -type f))
    if [[ ${#matches[@]} -eq 1 ]]; then
        echo "${matches[0]}"
        return
    elif [[ ${#matches[@]} -gt 1 ]]; then
        error "Multiple matches found for '$input':"
        for match in "${matches[@]}"; do
            echo "  $(basename "$match")"
        done
        exit 1
    fi
    
    error "Backup file not found: $input"
    exit 1
}

# Enhanced backup verification with metadata
verify_backup() {
    local backup_file="$1"
    local verify_only="${2:-false}"
    
    log "Verifying backup file: $(basename "$backup_file")"
    
    # Check if file exists and is readable
    if [[ ! -r "$backup_file" ]]; then
        error "Cannot read backup file: $backup_file"
        exit 1
    fi
    
    # Check metadata if available
    local meta_file="${backup_file}.meta"
    if [[ -f "$meta_file" ]]; then
        log "Reading backup metadata..."
        if command -v jq &> /dev/null; then
            local backup_type=$(jq -r '.backup_type // "unknown"' "$meta_file" 2>/dev/null || echo "unknown")
            local timestamp=$(jq -r '.timestamp // "unknown"' "$meta_file" 2>/dev/null || echo "unknown")
            local checksum=$(jq -r '.checksum // ""' "$meta_file" 2>/dev/null || echo "")
            
            info "Backup type: $backup_type"
            info "Created: $timestamp"
            
            # Verify checksum if available
            if [[ -n "$checksum" ]]; then
                log "Verifying checksum..."
                local current_checksum=$(sha256sum "$backup_file" | cut -d' ' -f1)
                if [[ "$current_checksum" == "$checksum" ]]; then
                    log "Checksum verified"
                else
                    error "Checksum mismatch - backup may be corrupted"
                    exit 1
                fi
            fi
        else
            warn "jq not available, skipping metadata parsing"
        fi
    else
        warn "No metadata file found for backup"
    fi
    
    # Handle compressed files
    local temp_file=""
    local test_file="$backup_file"
    
    if [[ "$backup_file" == *.gz ]]; then
        temp_file="/tmp/campfire_restore_verify_$(date +%s).db"
        log "Decompressing backup for verification..."
        
        if dry_run "gunzip -c $backup_file > $temp_file"; then
            if [[ "$verify_only" == "true" ]]; then
                return
            fi
        else
            gunzip -c "$backup_file" > "$temp_file"
        fi
        test_file="$temp_file"
    elif [[ "$backup_file" == *.gpg ]]; then
        error "Encrypted backups require decryption key"
        exit 1
    fi
    
    # Verify SQLite database integrity
    log "Checking database integrity..."
    
    if dry_run "sqlite3 $test_file PRAGMA integrity_check"; then
        if [[ "$verify_only" == "true" ]]; then
            log "Backup verification would succeed (dry run)"
            return
        fi
    else
        if sqlite3 "$test_file" "PRAGMA integrity_check;" | grep -q "ok"; then
            log "Backup integrity verified"
        else
            error "Backup integrity check failed"
            [[ -n "$temp_file" ]] && rm -f "$temp_file"
            exit 1
        fi
    fi
    
    # Show backup statistics
    if [[ "$verify_only" == "true" ]]; then
        local table_count=$(sqlite3 "$test_file" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';" 2>/dev/null || echo "unknown")
        local user_count=$(sqlite3 "$test_file" "SELECT COUNT(*) FROM users;" 2>/dev/null || echo "unknown")
        local message_count=$(sqlite3 "$test_file" "SELECT COUNT(*) FROM messages;" 2>/dev/null || echo "unknown")
        
        info "Tables: $table_count"
        info "Users: $user_count"
        info "Messages: $message_count"
    fi
    
    # Clean up temporary file
    [[ -n "$temp_file" ]] && rm -f "$temp_file"
}

# Create backup of current database with enhanced naming
backup_current() {
    if [[ ! -f "$DATABASE_URL" ]]; then
        info "No existing database to backup"
        return
    fi
    
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local current_backup="${DATABASE_URL}.pre-restore.${timestamp}"
    
    log "Creating backup of current database..."
    
    if dry_run "cp $DATABASE_URL $current_backup"; then
        return
    fi
    
    cp "$DATABASE_URL" "$current_backup"
    
    # Create metadata for the pre-restore backup
    cat > "${current_backup}.meta" <<EOF
{
    "backup_type": "pre-restore",
    "timestamp": "$(date -Iseconds)",
    "source_database": "$DATABASE_URL",
    "backup_file": "$current_backup",
    "size_bytes": $(stat -f%z "$current_backup" 2>/dev/null || stat -c%s "$current_backup"),
    "checksum": "$(sha256sum "$current_backup" | cut -d' ' -f1)"
}
EOF
    
    log "Current database backed up to: $(basename "$current_backup")"
    echo "PRE_RESTORE_BACKUP=$current_backup"
}

# Rollback to pre-restore backup
rollback_restore() {
    log "Looking for pre-restore backups..."
    
    local pre_restore_backups=($(find "$(dirname "$DATABASE_URL")" -name "$(basename "$DATABASE_URL").pre-restore.*" -type f | sort -r))
    
    if [[ ${#pre_restore_backups[@]} -eq 0 ]]; then
        error "No pre-restore backups found"
        exit 1
    fi
    
    echo ""
    echo "Available pre-restore backups:"
    local i=1
    for backup in "${pre_restore_backups[@]}"; do
        local date=$(stat -c %y "$backup" 2>/dev/null | cut -d' ' -f1,2 | cut -d'.' -f1 || echo "unknown")
        echo "$i. $(basename "$backup") ($date)"
        ((i++))
    done
    
    echo ""
    read -p "Select backup to rollback to (1-${#pre_restore_backups[@]}): " -r selection
    
    if [[ ! "$selection" =~ ^[0-9]+$ ]] || [[ "$selection" -lt 1 ]] || [[ "$selection" -gt ${#pre_restore_backups[@]} ]]; then
        error "Invalid selection"
        exit 1
    fi
    
    local selected_backup="${pre_restore_backups[$((selection-1))]}"
    
    log "Rolling back to: $(basename "$selected_backup")"
    
    if dry_run "cp $selected_backup $DATABASE_URL"; then
        return
    fi
    
    cp "$selected_backup" "$DATABASE_URL"
    
    log "Rollback completed successfully"
}

# Enhanced restore function
restore_database() {
    local backup_file="$1"
    
    log "Restoring database from: $(basename "$backup_file")"
    
    # Create database directory if it doesn't exist
    local db_dir=$(dirname "$DATABASE_URL")
    
    if dry_run "mkdir -p $db_dir"; then
        return
    fi
    
    mkdir -p "$db_dir"
    
    # Handle different backup formats
    if [[ "$backup_file" == *.gz ]]; then
        log "Decompressing and restoring..."
        if dry_run "gunzip -c $backup_file > $DATABASE_URL"; then
            return
        fi
        gunzip -c "$backup_file" > "$DATABASE_URL"
    elif [[ "$backup_file" == *.sql ]]; then
        log "Restoring from SQL dump..."
        if dry_run "sqlite3 $DATABASE_URL < $backup_file"; then
            return
        fi
        sqlite3 "$DATABASE_URL" < "$backup_file"
    else
        log "Copying backup to database location..."
        if dry_run "cp $backup_file $DATABASE_URL"; then
            return
        fi
        cp "$backup_file" "$DATABASE_URL"
    fi
    
    # Set appropriate permissions
    chmod 644 "$DATABASE_URL"
    
    # Verify restored database
    log "Verifying restored database..."
    if dry_run "sqlite3 $DATABASE_URL PRAGMA integrity_check"; then
        return
    fi
    
    if sqlite3 "$DATABASE_URL" "PRAGMA integrity_check;" | grep -q "ok"; then
        log "Database restored and verified successfully"
    else
        error "Restored database failed integrity check"
        exit 1
    fi
    
    # Show database info
    local db_size=$(du -h "$DATABASE_URL" | cut -f1)
    local table_count=$(sqlite3 "$DATABASE_URL" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';" 2>/dev/null || echo "unknown")
    local user_count=$(sqlite3 "$DATABASE_URL" "SELECT COUNT(*) FROM users;" 2>/dev/null || echo "unknown")
    local message_count=$(sqlite3 "$DATABASE_URL" "SELECT COUNT(*) FROM messages;" 2>/dev/null || echo "unknown")
    
    info "Database size: $db_size"
    info "Tables: $table_count"
    info "Users: $user_count"
    info "Messages: $message_count"
}

# Confirmation prompt
confirm_restore() {
    if [[ "$AUTO_CONFIRM" == "true" ]]; then
        return 0
    fi
    
    echo ""
    warn "This will replace the current database with the backup."
    warn "A backup of the current database will be created first."
    read -p "Are you sure you want to continue? (y/N): " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log "Restore cancelled by user"
        exit 0
    fi
}

# Main restore function
main() {
    # Handle special commands
    case "$BACKUP_FILE" in
        ""|"--list")
            list_backups
            exit 0
            ;;
        "--rollback")
            rollback_restore
            exit 0
            ;;
        "--verify-only")
            if [[ -z "${2:-}" ]]; then
                error "Backup file required for verification"
                exit 1
            fi
            local backup_path=$(get_backup_path "$2")
            verify_backup "$backup_path" "true"
            log "Backup verification completed"
            exit 0
            ;;
        "-h"|"--help")
            usage
            ;;
    esac
    
    # Get full backup file path
    local backup_path=$(get_backup_path "$BACKUP_FILE")
    
    log "Starting database restore process..."
    info "Backup file: $backup_path"
    info "Target database: $DATABASE_URL"
    info "Dry run: $DRY_RUN"
    
    # Verify backup file
    verify_backup "$backup_path"
    
    # Confirm restore operation
    confirm_restore
    
    # Backup current database
    backup_current
    
    # Restore database
    restore_database "$backup_path"
    
    log "Database restore completed successfully!"
    
    if [[ "$DRY_RUN" != "true" ]]; then
        echo ""
        info "To rollback this restore, run: $0 --rollback"
    fi
}

# Run main function
main "$@"