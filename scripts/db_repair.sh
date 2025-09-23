#!/bin/bash
# db_repair.sh - Attempt automatic database repair

DB_PATH="/var/lib/campfire/campfire.db"
BACKUP_DIR="/var/backups/campfire"
REPAIR_LOG="/var/log/campfire/repair.log"

# Create directories if they don't exist
mkdir -p "$(dirname "$REPAIR_LOG")"
mkdir -p "$BACKUP_DIR"

echo "=== Database Repair Started $(date) ===" | tee -a "$REPAIR_LOG"

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    echo "❌ Database file not found: $DB_PATH" | tee -a "$REPAIR_LOG"
    exit 1
fi

# Check if we have sqlite3
if ! command -v sqlite3 >/dev/null 2>&1; then
    echo "❌ sqlite3 command not found. Please install sqlite3." | tee -a "$REPAIR_LOG"
    exit 1
fi

# Stop Campfire service
echo "Stopping Campfire service..." | tee -a "$REPAIR_LOG"
if command -v systemctl >/dev/null 2>&1; then
    systemctl stop campfire-rust 2>&1 | tee -a "$REPAIR_LOG"
elif command -v docker >/dev/null 2>&1; then
    docker-compose stop 2>&1 | tee -a "$REPAIR_LOG" || docker stop campfire-rust 2>&1 | tee -a "$REPAIR_LOG"
else
    echo "⚠️  Cannot automatically stop service. Please stop Campfire manually." | tee -a "$REPAIR_LOG"
    echo "Press Enter when service is stopped..."
    read -r
fi

# Wait a moment for service to fully stop
sleep 5

# Create emergency backup
EMERGENCY_BACKUP="${BACKUP_DIR}/emergency-$(date +%Y%m%d-%H%M%S).db"
echo "Creating emergency backup: $EMERGENCY_BACKUP" | tee -a "$REPAIR_LOG"
cp "$DB_PATH" "$EMERGENCY_BACKUP"

if [ $? -eq 0 ]; then
    echo "✅ Emergency backup created successfully" | tee -a "$REPAIR_LOG"
else
    echo "❌ Failed to create emergency backup" | tee -a "$REPAIR_LOG"
    exit 1
fi

# Check current database integrity
echo "Checking current database integrity..." | tee -a "$REPAIR_LOG"
CURRENT_INTEGRITY=$(sqlite3 "$DB_PATH" "PRAGMA integrity_check;" 2>&1)
echo "Current integrity status: $CURRENT_INTEGRITY" | tee -a "$REPAIR_LOG"

if [ "$CURRENT_INTEGRITY" = "ok" ]; then
    echo "✅ Database integrity is OK, no repair needed" | tee -a "$REPAIR_LOG"
    echo "Starting Campfire service..." | tee -a "$REPAIR_LOG"
    if command -v systemctl >/dev/null 2>&1; then
        systemctl start campfire-rust
    elif command -v docker >/dev/null 2>&1; then
        docker-compose start 2>/dev/null || docker start campfire-rust
    fi
    exit 0
fi

# Attempt SQLite recovery
echo "Attempting SQLite .recover operation..." | tee -a "$REPAIR_LOG"
RECOVERED_DB="${DB_PATH}.recovered"
RECOVERY_SQL="${RECOVERED_DB}.sql"

# Remove any existing recovery files
rm -f "$RECOVERED_DB" "$RECOVERY_SQL"

# Generate recovery SQL
echo "Generating recovery SQL..." | tee -a "$REPAIR_LOG"
sqlite3 "$DB_PATH" ".recover" > "$RECOVERY_SQL" 2>&1

if [ $? -eq 0 ] && [ -s "$RECOVERY_SQL" ]; then
    echo "✅ Recovery SQL generated successfully" | tee -a "$REPAIR_LOG"
    
    # Create new database from recovered data
    echo "Creating recovered database..." | tee -a "$REPAIR_LOG"
    sqlite3 "$RECOVERED_DB" < "$RECOVERY_SQL" 2>&1 | tee -a "$REPAIR_LOG"
    
    if [ $? -eq 0 ] && [ -f "$RECOVERED_DB" ]; then
        echo "✅ Recovered database created successfully" | tee -a "$REPAIR_LOG"
        
        # Verify recovered database integrity
        echo "Verifying recovered database integrity..." | tee -a "$REPAIR_LOG"
        RECOVERED_INTEGRITY=$(sqlite3 "$RECOVERED_DB" "PRAGMA integrity_check;" 2>&1)
        
        if [ "$RECOVERED_INTEGRITY" = "ok" ]; then
            echo "✅ Recovered database integrity verified" | tee -a "$REPAIR_LOG"
            
            # Check that essential tables exist and have data
            echo "Checking essential tables..." | tee -a "$REPAIR_LOG"
            TABLES_CHECK=$(sqlite3 "$RECOVERED_DB" "
            SELECT 
                'users: ' || COUNT(*) FROM users
            UNION ALL
            SELECT 
                'rooms: ' || COUNT(*) FROM rooms
            UNION ALL
            SELECT 
                'messages: ' || COUNT(*) FROM messages;
            " 2>&1)
            
            if [ $? -eq 0 ]; then
                echo "✅ Essential tables verified:" | tee -a "$REPAIR_LOG"
                echo "$TABLES_CHECK" | tee -a "$REPAIR_LOG"
                
                # Replace original with recovered database
                echo "Replacing original database with recovered version..." | tee -a "$REPAIR_LOG"
                mv "$DB_PATH" "${DB_PATH}.corrupted"
                mv "$RECOVERED_DB" "$DB_PATH"
                
                # Set proper permissions
                if command -v chown >/dev/null 2>&1; then
                    chown campfire:campfire "$DB_PATH" 2>/dev/null || echo "⚠️  Could not set ownership" | tee -a "$REPAIR_LOG"
                fi
                chmod 644 "$DB_PATH"
                
                # Rebuild FTS5 index
                echo "Rebuilding FTS5 search index..." | tee -a "$REPAIR_LOG"
                sqlite3 "$DB_PATH" "INSERT INTO messages_fts(messages_fts) VALUES('rebuild');" 2>&1 | tee -a "$REPAIR_LOG"
                
                if [ $? -eq 0 ]; then
                    echo "✅ FTS5 index rebuilt successfully" | tee -a "$REPAIR_LOG"
                else
                    echo "⚠️  FTS5 index rebuild failed, but database is functional" | tee -a "$REPAIR_LOG"
                fi
                
                echo "✅ Database repair completed successfully" | tee -a "$REPAIR_LOG"
                REPAIR_SUCCESS=true
            else
                echo "❌ Essential tables check failed: $TABLES_CHECK" | tee -a "$REPAIR_LOG"
            fi
        else
            echo "❌ Recovered database failed integrity check: $RECOVERED_INTEGRITY" | tee -a "$REPAIR_LOG"
        fi
    else
        echo "❌ Failed to create recovered database" | tee -a "$REPAIR_LOG"
    fi
else
    echo "❌ SQLite recovery failed or produced empty output" | tee -a "$REPAIR_LOG"
fi

# If repair failed, try alternative recovery methods
if [ "$REPAIR_SUCCESS" != "true" ]; then
    echo "Attempting alternative recovery methods..." | tee -a "$REPAIR_LOG"
    
    # Method 2: Try to dump and restore specific tables
    echo "Trying selective table recovery..." | tee -a "$REPAIR_LOG"
    SELECTIVE_DB="${DB_PATH}.selective"
    rm -f "$SELECTIVE_DB"
    
    # Create new database with schema
    sqlite3 "$SELECTIVE_DB" "
    CREATE TABLE users (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        email TEXT UNIQUE NOT NULL,
        password_hash TEXT NOT NULL,
        bio TEXT,
        admin BOOLEAN NOT NULL DEFAULT FALSE,
        bot_token TEXT UNIQUE,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    );
    
    CREATE TABLE rooms (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        topic TEXT,
        room_type TEXT NOT NULL CHECK (room_type IN ('open', 'closed', 'direct')),
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        last_message_at DATETIME
    );
    
    CREATE TABLE messages (
        id TEXT PRIMARY KEY,
        room_id TEXT NOT NULL REFERENCES rooms(id),
        creator_id TEXT NOT NULL REFERENCES users(id),
        content TEXT NOT NULL,
        client_message_id TEXT NOT NULL,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(client_message_id, room_id)
    );
    
    CREATE TABLE sessions (
        token TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id),
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        expires_at DATETIME NOT NULL
    );
    
    CREATE TABLE room_memberships (
        room_id TEXT NOT NULL REFERENCES rooms(id),
        user_id TEXT NOT NULL REFERENCES users(id),
        involvement_level TEXT NOT NULL CHECK (involvement_level IN ('member', 'admin')),
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (room_id, user_id)
    );
    " 2>&1 | tee -a "$REPAIR_LOG"
    
    # Try to copy data from each table
    for table in users rooms messages sessions room_memberships; do
        echo "Recovering table: $table" | tee -a "$REPAIR_LOG"
        sqlite3 "$DB_PATH" ".mode insert $table" ".output /tmp/${table}_data.sql" "SELECT * FROM $table;" 2>/dev/null
        if [ -f "/tmp/${table}_data.sql" ] && [ -s "/tmp/${table}_data.sql" ]; then
            sqlite3 "$SELECTIVE_DB" < "/tmp/${table}_data.sql" 2>&1 | tee -a "$REPAIR_LOG"
            rm -f "/tmp/${table}_data.sql"
            echo "✅ Table $table recovered" | tee -a "$REPAIR_LOG"
        else
            echo "⚠️  Could not recover table $table" | tee -a "$REPAIR_LOG"
        fi
    done
    
    # Check selective recovery
    SELECTIVE_INTEGRITY=$(sqlite3 "$SELECTIVE_DB" "PRAGMA integrity_check;" 2>&1)
    if [ "$SELECTIVE_INTEGRITY" = "ok" ]; then
        echo "✅ Selective recovery successful" | tee -a "$REPAIR_LOG"
        mv "$DB_PATH" "${DB_PATH}.corrupted"
        mv "$SELECTIVE_DB" "$DB_PATH"
        chmod 644 "$DB_PATH"
        REPAIR_SUCCESS=true
    else
        echo "❌ Selective recovery failed: $SELECTIVE_INTEGRITY" | tee -a "$REPAIR_LOG"
        rm -f "$SELECTIVE_DB"
    fi
fi

# Cleanup temporary files
rm -f "$RECOVERY_SQL" "$RECOVERED_DB" "$SELECTIVE_DB"

# Start service
echo "Starting Campfire service..." | tee -a "$REPAIR_LOG"
if command -v systemctl >/dev/null 2>&1; then
    systemctl start campfire-rust 2>&1 | tee -a "$REPAIR_LOG"
elif command -v docker >/dev/null 2>&1; then
    docker-compose start 2>&1 | tee -a "$REPAIR_LOG" || docker start campfire-rust 2>&1 | tee -a "$REPAIR_LOG"
fi

# Wait for service to start
sleep 10

# Test service health
echo "Testing service health..." | tee -a "$REPAIR_LOG"
if command -v curl >/dev/null 2>&1; then
    if curl -f --connect-timeout 10 http://localhost:3000/health >/dev/null 2>&1; then
        echo "✅ Service is healthy after repair" | tee -a "$REPAIR_LOG"
    else
        echo "⚠️  Service health check failed after repair" | tee -a "$REPAIR_LOG"
    fi
else
    echo "⚠️  Cannot test service health (curl not available)" | tee -a "$REPAIR_LOG"
fi

echo "=== Database Repair Completed $(date) ===" | tee -a "$REPAIR_LOG"

if [ "$REPAIR_SUCCESS" = "true" ]; then
    echo ""
    echo "✅ Database repair completed successfully!"
    echo "   - Original database backed up to: $EMERGENCY_BACKUP"
    echo "   - Corrupted database saved as: ${DB_PATH}.corrupted"
    echo "   - Repair log: $REPAIR_LOG"
    echo ""
    echo "Please verify that your data is intact and the application is working correctly."
    exit 0
else
    echo ""
    echo "❌ Database repair failed!"
    echo "   - Original database backed up to: $EMERGENCY_BACKUP"
    echo "   - You may need to restore from a backup or contact support"
    echo "   - Repair log: $REPAIR_LOG"
    echo ""
    echo "To restore from backup, run:"
    echo "   ./scripts/restore_backup.sh $EMERGENCY_BACKUP"
    exit 1
fi