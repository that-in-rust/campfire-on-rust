#!/bin/bash

# Campfire Database Backup Script
# Creates timestamped backups of the SQLite database

set -euo pipefail

# Configuration
DATABASE_PATH="${CAMPFIRE_DATABASE_URL:-campfire.db}"
BACKUP_DIR="${CAMPFIRE_BACKUP_DIR:-./backups}"
RETENTION_DAYS="${CAMPFIRE_BACKUP_RETENTION_DAYS:-30}"
COMPRESS="${CAMPFIRE_BACKUP_COMPRESS:-true}"

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

# Check if database exists
if [[ ! -f "$DATABASE_PATH" ]]; then
    error "Database file not found: $DATABASE_PATH"
    exit 1
fi

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Generate backup filename with timestamp
TIMESTAMP=$(date +'%Y%m%d_%H%M%S')
BACKUP_NAME="campfire_backup_${TIMESTAMP}.db"
BACKUP_PATH="$BACKUP_DIR/$BACKUP_NAME"

log "Starting database backup..."
log "Source: $DATABASE_PATH"
log "Destination: $BACKUP_PATH"

# Perform backup using SQLite's backup command
# This ensures a consistent backup even if the database is in use
sqlite3 "$DATABASE_PATH" ".backup '$BACKUP_PATH'"

if [[ $? -eq 0 ]]; then
    log "Database backup completed successfully"
    
    # Get backup file size
    BACKUP_SIZE=$(du -h "$BACKUP_PATH" | cut -f1)
    log "Backup size: $BACKUP_SIZE"
    
    # Compress backup if requested
    if [[ "$COMPRESS" == "true" ]]; then
        log "Compressing backup..."
        gzip "$BACKUP_PATH"
        BACKUP_PATH="${BACKUP_PATH}.gz"
        COMPRESSED_SIZE=$(du -h "$BACKUP_PATH" | cut -f1)
        log "Compressed size: $COMPRESSED_SIZE"
    fi
    
    # Verify backup integrity
    log "Verifying backup integrity..."
    if [[ "$COMPRESS" == "true" ]]; then
        # Decompress temporarily for verification
        TEMP_BACKUP="/tmp/campfire_verify_${TIMESTAMP}.db"
        gunzip -c "$BACKUP_PATH" > "$TEMP_BACKUP"
        sqlite3 "$TEMP_BACKUP" "PRAGMA integrity_check;" > /dev/null
        rm "$TEMP_BACKUP"
    else
        sqlite3 "$BACKUP_PATH" "PRAGMA integrity_check;" > /dev/null
    fi
    
    if [[ $? -eq 0 ]]; then
        log "Backup integrity verified"
    else
        error "Backup integrity check failed"
        exit 1
    fi
    
else
    error "Database backup failed"
    exit 1
fi

# Clean up old backups
if [[ "$RETENTION_DAYS" -gt 0 ]]; then
    log "Cleaning up backups older than $RETENTION_DAYS days..."
    
    # Find and remove old backup files
    OLD_BACKUPS=$(find "$BACKUP_DIR" -name "campfire_backup_*.db*" -type f -mtime +$RETENTION_DAYS)
    
    if [[ -n "$OLD_BACKUPS" ]]; then
        echo "$OLD_BACKUPS" | while read -r old_backup; do
            log "Removing old backup: $(basename "$old_backup")"
            rm "$old_backup"
        done
    else
        log "No old backups to clean up"
    fi
fi

# Display backup summary
log "Backup completed successfully!"
log "Backup location: $BACKUP_PATH"

# List recent backups
log "Recent backups:"
ls -lah "$BACKUP_DIR"/campfire_backup_*.db* 2>/dev/null | tail -5 || log "No backups found"

exit 0