#!/bin/bash

# campfire-on-rust Enhanced Database Backup Script
# Creates compressed backups with timestamp, integrity verification, and multiple backup types

set -euo pipefail

# Configuration
BACKUP_DIR="${CAMPFIRE_BACKUP_DIR:-./backups}"
DATABASE_URL="${CAMPFIRE_DATABASE_URL:-./data/campfire.db}"
RETENTION_DAYS="${CAMPFIRE_BACKUP_RETENTION_DAYS:-30}"
COMPRESSION="${CAMPFIRE_BACKUP_COMPRESSION:-gzip}"
VERIFY_BACKUP="${CAMPFIRE_BACKUP_VERIFY:-true}"
BACKUP_TYPE="${1:-full}"  # full, incremental, or schema-only
REMOTE_BACKUP="${CAMPFIRE_REMOTE_BACKUP:-false}"
S3_BUCKET="${CAMPFIRE_S3_BUCKET:-}"
ENCRYPTION_KEY="${CAMPFIRE_BACKUP_ENCRYPTION_KEY:-}"

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
    echo "Usage: $0 [BACKUP_TYPE] [OPTIONS]"
    echo ""
    echo "Backup Types:"
    echo "  full         Full database backup (default)"
    echo "  incremental  Incremental backup since last full backup"
    echo "  schema       Schema-only backup (no data)"
    echo "  hot          Hot backup while database is running"
    echo ""
    echo "Options:"
    echo "  --no-compress    Skip compression"
    echo "  --no-verify      Skip backup verification"
    echo "  --encrypt        Encrypt backup with GPG"
    echo "  --remote         Upload to remote storage (S3)"
    echo "  --retention N    Keep backups for N days (default: 30)"
    echo "  --dry-run        Show what would be done without executing"
    echo "  -h, --help       Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  CAMPFIRE_BACKUP_DIR              Backup directory"
    echo "  CAMPFIRE_DATABASE_URL            Database file path"
    echo "  CAMPFIRE_BACKUP_RETENTION_DAYS   Retention period"
    echo "  CAMPFIRE_S3_BUCKET              S3 bucket for remote backups"
    echo "  CAMPFIRE_BACKUP_ENCRYPTION_KEY   GPG key for encryption"
    exit 1
}

# Parse command line arguments
DRY_RUN=false
ENCRYPT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-compress)
            COMPRESSION="none"
            shift
            ;;
        --no-verify)
            VERIFY_BACKUP="false"
            shift
            ;;
        --encrypt)
            ENCRYPT=true
            shift
            ;;
        --remote)
            REMOTE_BACKUP="true"
            shift
            ;;
        --retention)
            RETENTION_DAYS="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        full|incremental|schema|hot)
            BACKUP_TYPE="$1"
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

# Check dependencies
check_dependencies() {
    local missing_deps=()
    
    if ! command -v sqlite3 &> /dev/null; then
        missing_deps+=("sqlite3")
    fi
    
    if [[ "$COMPRESSION" == "gzip" ]] && ! command -v gzip &> /dev/null; then
        missing_deps+=("gzip")
    fi
    
    if [[ "$ENCRYPT" == "true" ]] && ! command -v gpg &> /dev/null; then
        missing_deps+=("gpg")
    fi
    
    if [[ "$REMOTE_BACKUP" == "true" ]] && ! command -v aws &> /dev/null; then
        missing_deps+=("aws-cli")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        error "Missing dependencies: ${missing_deps[*]}"
        exit 1
    fi
}

# Create backup directory if it doesn't exist
create_backup_dir() {
    if dry_run "mkdir -p $BACKUP_DIR"; then
        return
    fi
    
    mkdir -p "$BACKUP_DIR"
    chmod 755 "$BACKUP_DIR"
}

# Generate backup filename with timestamp
generate_backup_filename() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local hostname=$(hostname -s)
    echo "$BACKUP_DIR/campfire_${BACKUP_TYPE}_${hostname}_${timestamp}.db"
}

# Check database integrity
check_database_integrity() {
    log "Checking database integrity..."
    
    if dry_run "sqlite3 $DATABASE_URL PRAGMA integrity_check"; then
        return
    fi
    
    local integrity_result=$(sqlite3 "$DATABASE_URL" "PRAGMA integrity_check;")
    if [[ "$integrity_result" != "ok" ]]; then
        error "Database integrity check failed: $integrity_result"
        exit 1
    fi
    
    log "Database integrity check passed"
}

# Get database statistics
get_database_stats() {
    if dry_run "Getting database statistics"; then
        return
    fi
    
    local db_size=$(du -h "$DATABASE_URL" | cut -f1)
    local page_count=$(sqlite3 "$DATABASE_URL" "PRAGMA page_count;")
    local page_size=$(sqlite3 "$DATABASE_URL" "PRAGMA page_size;")
    local table_count=$(sqlite3 "$DATABASE_URL" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';")
    
    info "Database size: $db_size"
    info "Pages: $page_count (${page_size} bytes each)"
    info "Tables: $table_count"
}

# Perform backup based on type
perform_backup() {
    local backup_file="$1"
    
    log "Starting $BACKUP_TYPE backup of $DATABASE_URL"
    
    if dry_run "Performing $BACKUP_TYPE backup to $backup_file"; then
        return
    fi
    
    case "$BACKUP_TYPE" in
        "full")
            # Full database backup using SQLite backup API
            sqlite3 "$DATABASE_URL" ".backup '$backup_file'"
            ;;
        "hot")
            # Hot backup using WAL mode checkpoint
            sqlite3 "$DATABASE_URL" "PRAGMA wal_checkpoint(FULL);"
            cp "$DATABASE_URL" "$backup_file"
            ;;
        "incremental")
            # Incremental backup (requires WAL mode)
            if sqlite3 "$DATABASE_URL" "PRAGMA journal_mode;" | grep -q "wal"; then
                # Copy main database and WAL file
                cp "$DATABASE_URL" "$backup_file"
                if [[ -f "${DATABASE_URL}-wal" ]]; then
                    cp "${DATABASE_URL}-wal" "${backup_file}-wal"
                fi
                if [[ -f "${DATABASE_URL}-shm" ]]; then
                    cp "${DATABASE_URL}-shm" "${backup_file}-shm"
                fi
            else
                warn "WAL mode not enabled, performing full backup instead"
                sqlite3 "$DATABASE_URL" ".backup '$backup_file'"
            fi
            ;;
        "schema")
            # Schema-only backup
            sqlite3 "$DATABASE_URL" ".schema" > "${backup_file%.db}.sql"
            backup_file="${backup_file%.db}.sql"
            ;;
    esac
    
    if [[ $? -eq 0 ]]; then
        log "Database backup created: $backup_file"
    else
        error "Backup failed"
        exit 1
    fi
    
    echo "$backup_file"
}

# Verify backup integrity
verify_backup() {
    local backup_file="$1"
    
    if [[ "$VERIFY_BACKUP" != "true" || "$BACKUP_TYPE" == "schema" ]]; then
        return
    fi
    
    log "Verifying backup integrity..."
    
    if dry_run "Verifying backup integrity for $backup_file"; then
        return
    fi
    
    local temp_file="/tmp/campfire_verify_$(date +%s).db"
    
    # Handle compressed backups
    if [[ "$backup_file" == *.gz ]]; then
        gunzip -c "$backup_file" > "$temp_file"
    else
        cp "$backup_file" "$temp_file"
    fi
    
    # Verify integrity
    if sqlite3 "$temp_file" "PRAGMA integrity_check;" | grep -q "ok"; then
        log "Backup integrity verified"
    else
        error "Backup integrity check failed"
        rm -f "$temp_file"
        exit 1
    fi
    
    rm -f "$temp_file"
}

# Create backup metadata
create_metadata() {
    local backup_file="$1"
    local metadata_file="${backup_file}.meta"
    
    if dry_run "Creating metadata file $metadata_file"; then
        return
    fi
    
    local file_size=$(stat -f%z "$backup_file" 2>/dev/null || stat -c%s "$backup_file")
    local checksum=$(sha256sum "$backup_file" | cut -d' ' -f1)
    
    cat > "$metadata_file" <<EOF
{
    "backup_type": "$BACKUP_TYPE",
    "timestamp": "$(date -Iseconds)",
    "source_database": "$DATABASE_URL",
    "backup_file": "$backup_file",
    "compression": "$COMPRESSION",
    "encrypted": "$ENCRYPT",
    "verified": "$VERIFY_BACKUP",
    "size_bytes": $file_size,
    "checksum": "$checksum",
    "hostname": "$(hostname)",
    "version": "$(sqlite3 --version | cut -d' ' -f1)"
}
EOF
    
    log "Metadata created: $metadata_file"
}

# Compress backup
compress_backup() {
    local backup_file="$1"
    
    if [[ "$COMPRESSION" == "none" ]]; then
        echo "$backup_file"
        return
    fi
    
    log "Compressing backup..."
    
    if dry_run "Compressing $backup_file with $COMPRESSION"; then
        echo "${backup_file}.gz"
        return
    fi
    
    case "$COMPRESSION" in
        "gzip")
            gzip "$backup_file"
            backup_file="${backup_file}.gz"
            ;;
        "bzip2")
            bzip2 "$backup_file"
            backup_file="${backup_file}.bz2"
            ;;
        "xz")
            xz "$backup_file"
            backup_file="${backup_file}.xz"
            ;;
    esac
    
    local compressed_size=$(du -h "$backup_file" | cut -f1)
    log "Backup compressed: $backup_file ($compressed_size)"
    
    echo "$backup_file"
}

# Encrypt backup
encrypt_backup() {
    local backup_file="$1"
    
    if [[ "$ENCRYPT" != "true" ]]; then
        echo "$backup_file"
        return
    fi
    
    if [[ -z "$ENCRYPTION_KEY" ]]; then
        error "Encryption requested but no key specified"
        exit 1
    fi
    
    log "Encrypting backup..."
    
    if dry_run "Encrypting $backup_file with GPG"; then
        echo "${backup_file}.gpg"
        return
    fi
    
    gpg --trust-model always --encrypt -r "$ENCRYPTION_KEY" --output "${backup_file}.gpg" "$backup_file"
    rm "$backup_file"
    backup_file="${backup_file}.gpg"
    
    log "Backup encrypted: $backup_file"
    echo "$backup_file"
}

# Upload to remote storage
upload_remote() {
    local backup_file="$1"
    
    if [[ "$REMOTE_BACKUP" != "true" ]]; then
        return
    fi
    
    if [[ -z "$S3_BUCKET" ]]; then
        error "Remote backup requested but no S3 bucket specified"
        exit 1
    fi
    
    log "Uploading to remote storage..."
    
    if dry_run "Uploading $backup_file to s3://$S3_BUCKET/"; then
        return
    fi
    
    local s3_key="campfire-backups/$(basename "$backup_file")"
    aws s3 cp "$backup_file" "s3://$S3_BUCKET/$s3_key"
    
    if [[ $? -eq 0 ]]; then
        log "Backup uploaded to s3://$S3_BUCKET/$s3_key"
    else
        error "Failed to upload backup to S3"
        exit 1
    fi
}

# Clean up old backups
cleanup_old_backups() {
    if [[ "$RETENTION_DAYS" -le 0 ]]; then
        return
    fi
    
    log "Cleaning up backups older than $RETENTION_DAYS days"
    
    if dry_run "Cleaning up old backups"; then
        return
    fi
    
    local deleted_count=0
    
    # Find and remove old backup files
    while IFS= read -r -d '' old_backup; do
        log "Removing old backup: $(basename "$old_backup")"
        rm -f "$old_backup"
        ((deleted_count++))
    done < <(find "$BACKUP_DIR" -name "campfire_*_backup_*.db*" -mtime +$RETENTION_DAYS -print0)
    
    # Remove old metadata files
    find "$BACKUP_DIR" -name "campfire_*_backup_*.meta" -mtime +$RETENTION_DAYS -delete
    
    if [[ $deleted_count -gt 0 ]]; then
        log "Removed $deleted_count old backup files"
    else
        log "No old backups to clean up"
    fi
}

# Create symlink to latest backup
create_latest_link() {
    local backup_file="$1"
    
    if dry_run "Creating symlink to latest backup"; then
        return
    fi
    
    local link_name="$BACKUP_DIR/latest_${BACKUP_TYPE}_backup"
    ln -sf "$(basename "$backup_file")" "$link_name"
    log "Latest backup link created: $link_name"
}

# Display backup summary
show_summary() {
    local backup_file="$1"
    
    log "Backup Summary"
    echo "=============="
    info "Type: $BACKUP_TYPE"
    info "File: $backup_file"
    info "Size: $(du -h "$backup_file" | cut -f1)"
    info "Compression: $COMPRESSION"
    info "Encrypted: $ENCRYPT"
    info "Verified: $VERIFY_BACKUP"
    info "Remote: $REMOTE_BACKUP"
    
    # Show backup statistics
    local total_backups=$(find "$BACKUP_DIR" -name "campfire_*_backup_*.db*" | wc -l)
    local total_size=$(du -sh "$BACKUP_DIR" | cut -f1)
    info "Total backups: $total_backups (using $total_size)"
    
    # Output for automation
    echo ""
    echo "BACKUP_FILE=$backup_file"
    echo "BACKUP_SIZE=$(du -h "$backup_file" | cut -f1)"
    echo "BACKUP_TYPE=$BACKUP_TYPE"
}

# Main backup function
main() {
    log "Starting campfire-on-rust database backup..."
    
    # Check dependencies
    check_dependencies
    
    # Check if database file exists
    if [[ ! -f "$DATABASE_URL" ]]; then
        error "Database file not found: $DATABASE_URL"
        exit 1
    fi
    
    # Create backup directory
    create_backup_dir
    
    # Check database integrity
    check_database_integrity
    
    # Get database statistics
    get_database_stats
    
    # Generate backup filename
    local backup_file=$(generate_backup_filename)
    
    # Perform backup
    backup_file=$(perform_backup "$backup_file")
    
    # Create metadata
    create_metadata "$backup_file"
    
    # Compress backup
    backup_file=$(compress_backup "$backup_file")
    
    # Encrypt backup
    backup_file=$(encrypt_backup "$backup_file")
    
    # Verify backup
    verify_backup "$backup_file"
    
    # Upload to remote storage
    upload_remote "$backup_file"
    
    # Create latest symlink
    create_latest_link "$backup_file"
    
    # Clean up old backups
    cleanup_old_backups
    
    # Show summary
    show_summary "$backup_file"
    
    log "Backup completed successfully!"
}

# Run main function
main "$@"