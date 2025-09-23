#!/bin/bash
# campfire_diagnostics.sh - Complete system diagnostic tool

DIAGNOSTIC_DIR="/tmp/campfire-diagnostics-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$DIAGNOSTIC_DIR"

echo "=== Campfire System Diagnostics ==="
echo "Output directory: $DIAGNOSTIC_DIR"

# System Information
echo "Collecting system information..."
{
    echo "=== System Information ==="
    uname -a
    echo ""
    echo "=== CPU Information ==="
    if command -v lscpu >/dev/null 2>&1; then
        lscpu
    else
        cat /proc/cpuinfo | head -20
    fi
    echo ""
    echo "=== Memory Information ==="
    free -h
    echo ""
    echo "=== Disk Usage ==="
    df -h
    echo ""
    echo "=== Network Configuration ==="
    if command -v ip >/dev/null 2>&1; then
        ip addr show
    else
        ifconfig
    fi
    echo ""
    echo "=== System Load ==="
    uptime
    echo ""
    echo "=== Process List ==="
    ps aux | head -20
    echo ""
    echo "=== System Limits ==="
    ulimit -a
} > "$DIAGNOSTIC_DIR/system_info.txt"

# Campfire Service Status
echo "Collecting Campfire service status..."
{
    echo "=== Service Status ==="
    if command -v systemctl >/dev/null 2>&1; then
        systemctl status campfire-rust 2>&1 || echo "Service not found or not using systemd"
        echo ""
        echo "=== Service Configuration ==="
        systemctl show campfire-rust 2>&1 || echo "Cannot show service configuration"
        echo ""
        echo "=== Environment Variables ==="
        systemctl show-environment 2>&1 || echo "Cannot show environment"
    else
        echo "systemd not available, checking for process..."
        ps aux | grep campfire-rust | grep -v grep
    fi
    echo ""
    echo "=== Docker Status (if applicable) ==="
    if command -v docker >/dev/null 2>&1; then
        docker ps | grep campfire || echo "No Campfire Docker containers found"
        echo ""
        docker-compose ps 2>/dev/null || echo "docker-compose not available or no compose file"
    else
        echo "Docker not available"
    fi
} > "$DIAGNOSTIC_DIR/service_status.txt"

# Application Logs
echo "Collecting application logs..."
if command -v journalctl >/dev/null 2>&1; then
    journalctl -u campfire-rust --since "24 hours ago" > "$DIAGNOSTIC_DIR/application.log" 2>&1 || echo "No systemd logs available" > "$DIAGNOSTIC_DIR/application.log"
    journalctl -u campfire-rust --since "1 hour ago" | grep -i error > "$DIAGNOSTIC_DIR/recent_errors.log" 2>&1 || echo "No recent errors found" > "$DIAGNOSTIC_DIR/recent_errors.log"
elif command -v docker >/dev/null 2>&1; then
    docker logs campfire-rust --since 24h > "$DIAGNOSTIC_DIR/application.log" 2>&1 || echo "No Docker logs available" > "$DIAGNOSTIC_DIR/application.log"
    docker logs campfire-rust --since 1h 2>&1 | grep -i error > "$DIAGNOSTIC_DIR/recent_errors.log" || echo "No recent errors found" > "$DIAGNOSTIC_DIR/recent_errors.log"
else
    echo "No log collection method available" > "$DIAGNOSTIC_DIR/application.log"
    echo "No recent errors found" > "$DIAGNOSTIC_DIR/recent_errors.log"
fi

# Configuration Files
echo "Collecting configuration files..."
{
    echo "=== Configuration Files ==="
    for config_path in "/etc/campfire/config.toml" "./config.toml" ".env"; do
        if [ -f "$config_path" ]; then
            echo "Found configuration: $config_path"
            echo "--- $config_path ---"
            # Sanitize sensitive information
            sed 's/password = .*/password = "[REDACTED]"/g; s/secret = .*/secret = "[REDACTED]"/g; s/token = .*/token = "[REDACTED]"/g' "$config_path" 2>/dev/null || echo "Cannot read $config_path"
            echo ""
        fi
    done
    
    echo "=== Environment Variables ==="
    env | grep -i campfire | sed 's/=.*PASSWORD.*/=[REDACTED]/g; s/=.*SECRET.*/=[REDACTED]/g; s/=.*TOKEN.*/=[REDACTED]/g'
} > "$DIAGNOSTIC_DIR/configuration.txt"

# Database Health
echo "Checking database health..."
DB_PATH="/var/lib/campfire/campfire.db"
{
    echo "=== Database Health Check ==="
    if [ -f "$DB_PATH" ]; then
        echo "Database file found: $DB_PATH"
        echo ""
        echo "=== Database Integrity ==="
        sqlite3 "$DB_PATH" "PRAGMA integrity_check;" 2>&1 || echo "Cannot check database integrity"
        echo ""
        echo "=== Database Statistics ==="
        sqlite3 "$DB_PATH" "
        SELECT 'Users: ' || COUNT(*) FROM users
        UNION ALL
        SELECT 'Rooms: ' || COUNT(*) FROM rooms
        UNION ALL
        SELECT 'Messages: ' || COUNT(*) FROM messages
        UNION ALL
        SELECT 'Sessions: ' || COUNT(*) FROM sessions;
        " 2>&1 || echo "Cannot query database statistics"
        echo ""
        echo "=== Database Configuration ==="
        sqlite3 "$DB_PATH" "
        SELECT 'Journal Mode: ' || PRAGMA journal_mode
        UNION ALL
        SELECT 'Synchronous: ' || PRAGMA synchronous
        UNION ALL
        SELECT 'User Version: ' || PRAGMA user_version;
        " 2>&1 || echo "Cannot query database configuration"
        echo ""
        echo "=== Database File Info ==="
        ls -lh "$DB_PATH"* 2>/dev/null || echo "Cannot list database files"
        echo ""
        echo "=== Database Size Analysis ==="
        if command -v sqlite3 >/dev/null 2>&1; then
            sqlite3 "$DB_PATH" "
            SELECT 
                name,
                COUNT(*) as row_count
            FROM sqlite_master 
            WHERE type='table' 
            GROUP BY name;
            " 2>&1 || echo "Cannot analyze database size"
        fi
    else
        echo "Database file not found: $DB_PATH"
        echo "Checking alternative locations..."
        find /var -name "campfire.db" 2>/dev/null || echo "No database files found"
        find . -name "campfire.db" 2>/dev/null || echo "No database files found in current directory"
    fi
} > "$DIAGNOSTIC_DIR/database_health.txt"

# Network Connectivity
echo "Testing network connectivity..."
{
    echo "=== Port Binding ==="
    if command -v ss >/dev/null 2>&1; then
        ss -tuln | grep :3000 || echo "Port 3000 not bound"
    elif command -v netstat >/dev/null 2>&1; then
        netstat -tuln | grep :3000 || echo "Port 3000 not bound"
    else
        echo "No network tools available"
    fi
    echo ""
    echo "=== Health Check ==="
    if command -v curl >/dev/null 2>&1; then
        curl -v --connect-timeout 10 http://localhost:3000/health 2>&1 || echo "Health check failed"
    else
        echo "curl not available for health check"
    fi
    echo ""
    echo "=== Metrics Endpoint ==="
    if command -v curl >/dev/null 2>&1; then
        curl -v --connect-timeout 10 http://localhost:3000/metrics 2>&1 | head -20 || echo "Metrics endpoint failed"
    else
        echo "curl not available for metrics check"
    fi
    echo ""
    echo "=== DNS Resolution ==="
    nslookup localhost 2>&1 || echo "DNS resolution test failed"
} > "$DIAGNOSTIC_DIR/network_connectivity.txt"

# Performance Metrics
echo "Collecting performance metrics..."
{
    echo "=== Current Resource Usage ==="
    CAMPFIRE_PID=$(pgrep campfire-rust)
    if [ -n "$CAMPFIRE_PID" ]; then
        echo "Campfire process found (PID: $CAMPFIRE_PID)"
        ps -p "$CAMPFIRE_PID" -o pid,ppid,cmd,%mem,%cpu,etime 2>/dev/null || echo "Cannot get process info"
        echo ""
        echo "=== Memory Details ==="
        if [ -f "/proc/$CAMPFIRE_PID/status" ]; then
            cat "/proc/$CAMPFIRE_PID/status" | grep -E "(VmRSS|VmSize|VmPeak|VmHWM)" || echo "Cannot read memory details"
        fi
        echo ""
        echo "=== File Descriptors ==="
        if [ -d "/proc/$CAMPFIRE_PID/fd" ]; then
            FD_COUNT=$(ls /proc/"$CAMPFIRE_PID"/fd 2>/dev/null | wc -l)
            echo "Open file descriptors: $FD_COUNT"
            if [ -f "/proc/$CAMPFIRE_PID/limits" ]; then
                echo "File descriptor limit: $(cat /proc/$CAMPFIRE_PID/limits | grep 'Max open files')"
            fi
        fi
        echo ""
        echo "=== Network Connections ==="
        if command -v lsof >/dev/null 2>&1; then
            lsof -p "$CAMPFIRE_PID" -i 2>/dev/null | head -10 || echo "Cannot list network connections"
        fi
    else
        echo "Campfire process not running"
    fi
    echo ""
    echo "=== System Load Average ==="
    cat /proc/loadavg 2>/dev/null || uptime
    echo ""
    echo "=== Memory Usage ==="
    free -h
    echo ""
    echo "=== Disk I/O ==="
    if command -v iostat >/dev/null 2>&1; then
        iostat -x 1 1 2>/dev/null || echo "iostat not available"
    else
        echo "iostat not available"
    fi
} > "$DIAGNOSTIC_DIR/performance_metrics.txt"

# Security and Permissions
echo "Checking security and permissions..."
{
    echo "=== File Permissions ==="
    ls -la /var/lib/campfire/ 2>/dev/null || echo "Cannot access /var/lib/campfire/"
    ls -la /etc/campfire/ 2>/dev/null || echo "No /etc/campfire directory"
    echo ""
    echo "=== Process Security ==="
    CAMPFIRE_PID=$(pgrep campfire-rust)
    if [ -n "$CAMPFIRE_PID" ]; then
        ps -p "$CAMPFIRE_PID" -o pid,user,group,comm 2>/dev/null || echo "Cannot get process security info"
        echo ""
        echo "=== Process Capabilities ==="
        if [ -f "/proc/$CAMPFIRE_PID/status" ]; then
            cat "/proc/$CAMPFIRE_PID/status" | grep Cap || echo "Cannot read process capabilities"
        fi
    fi
    echo ""
    echo "=== Firewall Status ==="
    if command -v ufw >/dev/null 2>&1; then
        ufw status 2>/dev/null || echo "ufw not configured"
    elif command -v iptables >/dev/null 2>&1; then
        iptables -L INPUT | head -10 2>/dev/null || echo "Cannot read iptables rules"
    else
        echo "No firewall tools available"
    fi
    echo ""
    echo "=== SELinux Status ==="
    if command -v getenforce >/dev/null 2>&1; then
        getenforce 2>/dev/null || echo "SELinux not available"
    else
        echo "SELinux not available"
    fi
} > "$DIAGNOSTIC_DIR/security_permissions.txt"

# Application-specific diagnostics
echo "Collecting application-specific diagnostics..."
{
    echo "=== Application Metrics ==="
    if command -v curl >/dev/null 2>&1; then
        curl -s --connect-timeout 5 http://localhost:3000/metrics 2>/dev/null | head -50 || echo "Cannot retrieve application metrics"
    else
        echo "curl not available for metrics collection"
    fi
    echo ""
    echo "=== WebSocket Connections ==="
    if command -v curl >/dev/null 2>&1; then
        curl -s --connect-timeout 5 http://localhost:3000/metrics 2>/dev/null | grep -i websocket || echo "No WebSocket metrics available"
    fi
    echo ""
    echo "=== Database Connections ==="
    if command -v curl >/dev/null 2>&1; then
        curl -s --connect-timeout 5 http://localhost:3000/metrics 2>/dev/null | grep -i database || echo "No database metrics available"
    fi
} > "$DIAGNOSTIC_DIR/application_metrics.txt"

# Create summary report
echo "Generating summary report..."
{
    echo "=== Campfire Diagnostics Summary ==="
    echo "Generated: $(date)"
    echo "Hostname: $(hostname)"
    echo "System: $(uname -s) $(uname -r)"
    echo ""
    
    # Service status summary
    if command -v systemctl >/dev/null 2>&1 && systemctl is-active campfire-rust >/dev/null 2>&1; then
        echo "✅ Service Status: Running (systemd)"
    elif pgrep campfire-rust >/dev/null 2>&1; then
        echo "✅ Service Status: Running (process found)"
    else
        echo "❌ Service Status: Not Running"
    fi
    
    # Health check summary
    if command -v curl >/dev/null 2>&1 && curl -f --connect-timeout 5 http://localhost:3000/health >/dev/null 2>&1; then
        echo "✅ Health Check: Passing"
    else
        echo "❌ Health Check: Failing"
    fi
    
    # Database integrity summary
    if [ -f "$DB_PATH" ]; then
        if command -v sqlite3 >/dev/null 2>&1; then
            DB_INTEGRITY=$(sqlite3 "$DB_PATH" "PRAGMA integrity_check;" 2>&1)
            if [ "$DB_INTEGRITY" = "ok" ]; then
                echo "✅ Database Integrity: OK"
            else
                echo "❌ Database Integrity: Failed"
            fi
        else
            echo "⚠️  Database: Cannot check (sqlite3 not available)"
        fi
    else
        echo "❌ Database: Not Found"
    fi
    
    # Resource usage summary
    CAMPFIRE_PID=$(pgrep campfire-rust)
    if [ -n "$CAMPFIRE_PID" ]; then
        if command -v ps >/dev/null 2>&1; then
            MEMORY_MB=$(ps -p "$CAMPFIRE_PID" -o rss= 2>/dev/null | awk '{print int($1/1024)}' || echo "unknown")
            CPU_PERCENT=$(ps -p "$CAMPFIRE_PID" -o %cpu= 2>/dev/null | awk '{print $1}' || echo "unknown")
            echo "📊 Memory Usage: ${MEMORY_MB}MB"
            echo "📊 CPU Usage: ${CPU_PERCENT}%"
        fi
    fi
    
    # Recent errors summary
    ERROR_COUNT=$(grep -c "ERROR\|WARN" "$DIAGNOSTIC_DIR/recent_errors.log" 2>/dev/null || echo "0")
    echo "⚠️  Recent Errors: $ERROR_COUNT"
    
    # Disk space summary
    AVAILABLE_SPACE=$(df "$(dirname "$DB_PATH")" 2>/dev/null | tail -1 | awk '{print $4}' || echo "0")
    if [ "$AVAILABLE_SPACE" != "0" ]; then
        AVAILABLE_SPACE_MB=$((AVAILABLE_SPACE / 1024))
        echo "💾 Available Disk Space: ${AVAILABLE_SPACE_MB}MB"
    fi
    
    echo ""
    echo "=== Recommendations ==="
    
    # Generate recommendations based on findings
    if [ "$ERROR_COUNT" -gt 10 ]; then
        echo "- High error rate detected, check application.log for details"
    fi
    
    if [ -n "$MEMORY_MB" ] && [ "$MEMORY_MB" != "unknown" ] && [ "$MEMORY_MB" -gt 1024 ]; then
        echo "- High memory usage detected (${MEMORY_MB}MB), consider memory profiling"
    fi
    
    if [ ! -f "$DB_PATH" ]; then
        echo "- Database file missing, check configuration and permissions"
    fi
    
    if ! command -v curl >/dev/null 2>&1 || ! curl -f --connect-timeout 5 http://localhost:3000/health >/dev/null 2>&1; then
        echo "- Health check failing, service may not be responding"
    fi
    
    if [ -n "$AVAILABLE_SPACE_MB" ] && [ "$AVAILABLE_SPACE_MB" -lt 1024 ]; then
        echo "- Low disk space warning: ${AVAILABLE_SPACE_MB}MB available"
    fi
    
    if ! pgrep campfire-rust >/dev/null 2>&1; then
        echo "- Campfire process not running, check service status and logs"
    fi
    
} > "$DIAGNOSTIC_DIR/summary.txt"

# Create archive
echo "Creating diagnostic archive..."
ARCHIVE_NAME="${DIAGNOSTIC_DIR}.tar.gz"
tar -czf "$ARCHIVE_NAME" -C "$(dirname "$DIAGNOSTIC_DIR")" "$(basename "$DIAGNOSTIC_DIR")" 2>/dev/null

if [ -f "$ARCHIVE_NAME" ]; then
    echo ""
    echo "=== Diagnostics Complete ==="
    echo "Archive created: $ARCHIVE_NAME"
    echo "Archive size: $(ls -lh "$ARCHIVE_NAME" | awk '{print $5}')"
else
    echo ""
    echo "=== Diagnostics Complete ==="
    echo "Archive creation failed, files available in: $DIAGNOSTIC_DIR"
fi

echo ""
echo "=== Summary ==="
cat "$DIAGNOSTIC_DIR/summary.txt"
echo ""

if [ -f "$ARCHIVE_NAME" ]; then
    echo "To share diagnostics, send the archive file: $ARCHIVE_NAME"
else
    echo "To share diagnostics, send the directory: $DIAGNOSTIC_DIR"
fi

echo ""
echo "=== File Contents ==="
echo "Generated files:"
ls -la "$DIAGNOSTIC_DIR/"