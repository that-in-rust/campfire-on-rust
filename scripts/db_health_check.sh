#!/bin/bash
# db_health_check.sh - Comprehensive database health assessment

DB_PATH="/var/lib/campfire/campfire.db"
BACKUP_DIR="/var/backups/campfire"
LOG_FILE="/var/log/campfire/db_health.log"

# Create log directory if it doesn't exist
mkdir -p "$(dirname "$LOG_FILE")"

echo "=== Database Health Check $(date) ===" | tee -a "$LOG_FILE"

# Check if database file exists and is accessible
if [ ! -f "$DB_PATH" ]; then
    echo "ERROR: Database file not found: $DB_PATH" | tee -a "$LOG_FILE"
    exit 1
fi

# Check file permissions
if [ ! -r "$DB_PATH" ] || [ ! -w "$DB_PATH" ]; then
    echo "ERROR: Insufficient permissions for database file" | tee -a "$LOG_FILE"
    echo "Current permissions: $(ls -la "$DB_PATH")" | tee -a "$LOG_FILE"
    exit 1
fi

# Check database integrity
echo "Checking database integrity..." | tee -a "$LOG_FILE"
INTEGRITY_RESULT=$(sqlite3 "$DB_PATH" "PRAGMA integrity_check;" 2>&1)

if [ "$INTEGRITY_RESULT" = "ok" ]; then
    echo "✅ Database integrity: OK" | tee -a "$LOG_FILE"
else
    echo "❌ Database integrity: FAILED" | tee -a "$LOG_FILE"
    echo "Integrity check result: $INTEGRITY_RESULT" | tee -a "$LOG_FILE"
    
    # Check specific table integrity
    echo "Checking individual tables..." | tee -a "$LOG_FILE"
    for table in users rooms messages sessions room_memberships; do
        TABLE_CHECK=$(sqlite3 "$DB_PATH" "PRAGMA integrity_check($table);" 2>&1)
        if [ "$TABLE_CHECK" = "ok" ]; then
            echo "✅ Table $table: OK" | tee -a "$LOG_FILE"
        else
            echo "❌ Table $table: $TABLE_CHECK" | tee -a "$LOG_FILE"
        fi
    done
fi

# Check WAL file status
if [ -f "${DB_PATH}-wal" ]; then
    WAL_SIZE=$(stat -f%z "${DB_PATH}-wal" 2>/dev/null || stat -c%s "${DB_PATH}-wal" 2>/dev/null)
    echo "WAL file size: ${WAL_SIZE} bytes" | tee -a "$LOG_FILE"
    
    if [ "$WAL_SIZE" -gt 10485760 ]; then  # 10MB
        echo "⚠️  Large WAL file detected (${WAL_SIZE} bytes), consider checkpoint" | tee -a "$LOG_FILE"
        
        # Attempt WAL checkpoint
        echo "Attempting WAL checkpoint..." | tee -a "$LOG_FILE"
        CHECKPOINT_RESULT=$(sqlite3 "$DB_PATH" "PRAGMA wal_checkpoint(TRUNCATE);" 2>&1)
        echo "Checkpoint result: $CHECKPOINT_RESULT" | tee -a "$LOG_FILE"
    fi
fi

# Check SHM file
if [ -f "${DB_PATH}-shm" ]; then
    SHM_SIZE=$(stat -f%z "${DB_PATH}-shm" 2>/dev/null || stat -c%s "${DB_PATH}-shm" 2>/dev/null)
    echo "SHM file size: ${SHM_SIZE} bytes" | tee -a "$LOG_FILE"
fi

# Check database statistics
echo "Database statistics:" | tee -a "$LOG_FILE"
sqlite3 "$DB_PATH" "
SELECT 'Users: ' || COUNT(*) FROM users
UNION ALL
SELECT 'Rooms: ' || COUNT(*) FROM rooms
UNION ALL
SELECT 'Messages: ' || COUNT(*) FROM messages
UNION ALL
SELECT 'Sessions: ' || COUNT(*) FROM sessions
UNION ALL
SELECT 'Room Memberships: ' || COUNT(*) FROM room_memberships;
" | tee -a "$LOG_FILE"

# Check database configuration
echo "Database configuration:" | tee -a "$LOG_FILE"
sqlite3 "$DB_PATH" "
SELECT 'Journal Mode: ' || PRAGMA journal_mode
UNION ALL
SELECT 'Synchronous: ' || PRAGMA synchronous
UNION ALL
SELECT 'Cache Size: ' || PRAGMA cache_size
UNION ALL
SELECT 'Page Size: ' || PRAGMA page_size
UNION ALL
SELECT 'User Version: ' || PRAGMA user_version;
" | tee -a "$LOG_FILE"

# Check FTS5 index integrity
echo "Checking FTS5 search index..." | tee -a "$LOG_FILE"
FTS_CHECK=$(sqlite3 "$DB_PATH" "INSERT INTO messages_fts(messages_fts) VALUES('integrity-check');" 2>&1)
if [ $? -eq 0 ]; then
    echo "✅ FTS5 index: OK" | tee -a "$LOG_FILE"
else
    echo "❌ FTS5 index: FAILED - $FTS_CHECK" | tee -a "$LOG_FILE"
    
    # Attempt to rebuild FTS5 index
    echo "Attempting to rebuild FTS5 index..." | tee -a "$LOG_FILE"
    REBUILD_RESULT=$(sqlite3 "$DB_PATH" "INSERT INTO messages_fts(messages_fts) VALUES('rebuild');" 2>&1)
    if [ $? -eq 0 ]; then
        echo "✅ FTS5 index rebuilt successfully" | tee -a "$LOG_FILE"
    else
        echo "❌ FTS5 index rebuild failed: $REBUILD_RESULT" | tee -a "$LOG_FILE"
    fi
fi

# Check for database locks
echo "Checking for database locks..." | tee -a "$LOG_FILE"
LOCK_CHECK=$(timeout 5 sqlite3 "$DB_PATH" "BEGIN IMMEDIATE; ROLLBACK;" 2>&1)
if [ $? -eq 0 ]; then
    echo "✅ Database locks: OK" | tee -a "$LOG_FILE"
else
    echo "❌ Database may be locked: $LOCK_CHECK" | tee -a "$LOG_FILE"
fi

# Check database file size and growth
DB_SIZE=$(stat -f%z "$DB_PATH" 2>/dev/null || stat -c%s "$DB_PATH" 2>/dev/null)
DB_SIZE_MB=$((DB_SIZE / 1024 / 1024))
echo "Database file size: ${DB_SIZE_MB}MB" | tee -a "$LOG_FILE"

# Check available disk space
AVAILABLE_SPACE=$(df "$(dirname "$DB_PATH")" | tail -1 | awk '{print $4}')
AVAILABLE_SPACE_MB=$((AVAILABLE_SPACE / 1024))
echo "Available disk space: ${AVAILABLE_SPACE_MB}MB" | tee -a "$LOG_FILE"

if [ "$AVAILABLE_SPACE_MB" -lt 1024 ]; then  # Less than 1GB
    echo "⚠️  Low disk space warning: ${AVAILABLE_SPACE_MB}MB available" | tee -a "$LOG_FILE"
fi

# Performance check - simple query timing
echo "Checking query performance..." | tee -a "$LOG_FILE"
QUERY_START=$(date +%s%N)
QUERY_RESULT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM messages;" 2>&1)
QUERY_END=$(date +%s%N)
QUERY_TIME_MS=$(( (QUERY_END - QUERY_START) / 1000000 ))

if [ $? -eq 0 ]; then
    echo "✅ Query performance: ${QUERY_TIME_MS}ms for message count" | tee -a "$LOG_FILE"
    if [ "$QUERY_TIME_MS" -gt 1000 ]; then  # More than 1 second
        echo "⚠️  Slow query detected: ${QUERY_TIME_MS}ms" | tee -a "$LOG_FILE"
    fi
else
    echo "❌ Query failed: $QUERY_RESULT" | tee -a "$LOG_FILE"
fi

# Check backup status
echo "Checking backup status..." | tee -a "$LOG_FILE"
if [ -d "$BACKUP_DIR" ]; then
    LATEST_BACKUP=$(ls -t "$BACKUP_DIR"/*.db 2>/dev/null | head -1)
    if [ -n "$LATEST_BACKUP" ]; then
        BACKUP_AGE=$(( ($(date +%s) - $(stat -f%m "$LATEST_BACKUP" 2>/dev/null || stat -c%Y "$LATEST_BACKUP" 2>/dev/null)) / 3600 ))
        echo "✅ Latest backup: $(basename "$LATEST_BACKUP") (${BACKUP_AGE} hours old)" | tee -a "$LOG_FILE"
        
        if [ "$BACKUP_AGE" -gt 24 ]; then  # More than 24 hours
            echo "⚠️  Backup is older than 24 hours" | tee -a "$LOG_FILE"
        fi
    else
        echo "❌ No backups found in $BACKUP_DIR" | tee -a "$LOG_FILE"
    fi
else
    echo "❌ Backup directory not found: $BACKUP_DIR" | tee -a "$LOG_FILE"
fi

echo "=== Health Check Complete ===" | tee -a "$LOG_FILE"

# Exit with error code if critical issues found
if echo "$INTEGRITY_RESULT" | grep -q "ok" && [ "$AVAILABLE_SPACE_MB" -gt 100 ]; then
    echo "Overall health: ✅ HEALTHY"
    exit 0
else
    echo "Overall health: ❌ ISSUES DETECTED"
    exit 1
fi