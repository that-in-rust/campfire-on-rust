#!/bin/bash

# Campfire Database Restore Script
# Restores database from backup with safety checks

set -euo pipefail

# Configuration
DATABASE_PATH="${CAMPFIRE_DATABASE_URL:-campfire.db}"
BACKUP_DIR="${CAMPFIRE_BACKUP_DIR:-./backups}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" >&2
}

# Usage function
usage() {
    echo "Usage: $0 [BACKUP_FILE]"
    echo ""
    echo "Restore Campfire database from backup"
    echo ""
    echo "Arguments:"
    echo "  BACKUP_FILE    Path to backup file (optional - will prompt if not provided)"
    echo ""
    echo "Environment Variables:"
    echo "  CAMPFIRE_DATABASE_URL    Target database path (default: campfire.db)"
    echo "  CAMPFIRE_BACKUP_DIR      Backup directory (default: ./backups)"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Interactive mode"
    echo "  $0 backups/campfire_backup_20240101_120000.db"
    echo "  $0 backups/campfire_backup_20240101_120000.db.gz"
    exit 1
}

# Check for help flag
if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
fi

# Function to list available backups
list_backups() {
    log "Available backups in $BACKUP_DIR:"
    if ls "$BACKUP_DIR"/campfire_backup_*.db* 1> /dev/null 2>&1; then
        ls -lah "$BACKUP_DIR"/campfire_backup_*.db* | nl
    else
        warn "No backup files found in $BACKUP_DIR"
        return 1
    fi
}

# Function to select backup interactively
select_backup() {
    list_backups || return 1
    
    echo ""
    read -p "Enter the number of the backup to restore (or 'q' to quit): " selection
    
    if [[ "$selection" == "q" || "$selection" == "Q" ]]; then
        log "Restore cancelled by user"
        exit 0
    fi
    
    # Get the selected backup file
    BACKUP_FILE=$(ls "$BACKUP_DIR"/campfire_backup_*.db* | sed -n "${selection}p")
    
    if [[ -z "$BACKUP_FILE" ]]; then
        error "Invalid selection: $selection"
        exit 1
    fi
    
    echo "$BACKUP_FILE"
}

# Determine backup file to restore
if [[ $# -eq 0 ]]; then
    # Interactive mode
    BACKUP_FILE=$(select_backup)
else
    BACKUP_FILE="$1"
fi

# Validate backup file
if [[ ! -f "$BACKUP_FILE" ]]; then
    error "Backup file not found: $BACKUP_FILE"
    exit 1
fi

log "Selected backup: $BACKUP_FILE"

# Check if backup is compressed
IS_COMPRESSED=false
if [[ "$BACKUP_FILE" == *.gz ]]; then
    IS_COMPRESSED=true
    log "Backup is compressed"
fi

# Verify backup integrity before restore
log "Verifying backup integrity..."
if [[ "$IS_COMPRESSED" == "true" ]]; then
    # Test decompression and SQLite integrity
    TEMP_BACKUP="/tmp/campfire_restore_verify_$$.db"
    gunzip -c "$BACKUP_FILE" > "$TEMP_BACKUP"
    sqlite3 "$TEMP_BACKUP" "PRAGMA integrity_check;" > /dev/null
    INTEGRITY_RESULT=$?
    rm "$TEMP_BACKUP"
else
    sqlite3 "$BACKUP_FILE" "PRAGMA integrity_check;" > /dev/null
    INTEGRITY_RESULT=$?
fi

if [[ $INTEGRITY_RESULT -ne 0 ]]; then
    error "Backup integrity check failed. Backup may be corrupted."
    exit 1
fi

log "Backup integrity verified"

# Safety check - backup current database if it exists
if [[ -f "$DATABASE_PATH" ]]; then
    warn "Current database exists: $DATABASE_PATH"
    
    # Ask for confirmation
    read -p "This will replace the current database. Continue? (y/N): " confirm
    if [[ "$confirm" != "y" && "$confirm" != "Y" ]]; then
        log "Restore cancelled by user"
        exit 0
    fi
    
    # Create safety backup
    SAFETY_BACKUP="${DATABASE_PATH}.pre-restore.$(date +'%Y%m%d_%H%M%S')"
    log "Creating safety backup: $SAFETY_BACKUP"
    cp "$DATABASE_PATH" "$SAFETY_BACKUP"
    
    if [[ $? -eq 0 ]]; then
        log "Safety backup created successfully"
    else
        error "Failed to create safety backup"
        exit 1
    fi
fi

# Perform restore
log "Starting database restore..."

if [[ "$IS_COMPRESSED" == "true" ]]; then
    log "Decompressing and restoring backup..."
    gunzip -c "$BACKUP_FILE" > "$DATABASE_PATH"
else
    log "Copying backup to database location..."
    cp "$BACKUP_FILE" "$DATABASE_PATH"
fi

if [[ $? -eq 0 ]]; then
    log "Database restore completed successfully"
else
    error "Database restore failed"
    
    # Attempt to restore safety backup if it exists
    if [[ -f "${SAFETY_BACKUP:-}" ]]; then
        warn "Attempting to restore safety backup..."
        cp "$SAFETY_BACKUP" "$DATABASE_PATH"
        if [[ $? -eq 0 ]]; then
            log "Safety backup restored"
        else
            error "Failed to restore safety backup"
        fi
    fi
    
    exit 1
fi

# Verify restored database
log "Verifying restored database..."
sqlite3 "$DATABASE_PATH" "PRAGMA integrity_check;" > /dev/null

if [[ $? -eq 0 ]]; then
    log "Restored database integrity verified"
else
    error "Restored database integrity check failed"
    exit 1
fi

# Display database info
log "Database restore summary:"
log "Restored from: $BACKUP_FILE"
log "Restored to: $DATABASE_PATH"

# Get database size and table count
DB_SIZE=$(du -h "$DATABASE_PATH" | cut -f1)
TABLE_COUNT=$(sqlite3 "$DATABASE_PATH" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';" 2>/dev/null || echo "unknown")

log "Database size: $DB_SIZE"
log "Number of tables: $TABLE_COUNT"

# Clean up safety backup if restore was successful
if [[ -f "${SAFETY_BACKUP:-}" ]]; then
    read -p "Remove safety backup? (y/N): " remove_safety
    if [[ "$remove_safety" == "y" || "$remove_safety" == "Y" ]]; then
        rm "$SAFETY_BACKUP"
        log "Safety backup removed"
    else
        log "Safety backup kept: $SAFETY_BACKUP"
    fi
fi

log "Database restore completed successfully!"

exit 0