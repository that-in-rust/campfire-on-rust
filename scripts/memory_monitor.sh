#!/bin/bash
# memory_monitor.sh - Track memory usage over time for Campfire Rust

LOG_FILE="/var/log/campfire/memory_usage.log"
INTERVAL=60  # Check every minute
ALERT_THRESHOLD=1048576  # 1GB in KB

# Create log directory if it doesn't exist
mkdir -p "$(dirname "$LOG_FILE")"

# Create log header if file doesn't exist
if [ ! -f "$LOG_FILE" ]; then
    echo "timestamp,rss_kb,vsz_kb,heap_bytes,connections,cpu_percent" > "$LOG_FILE"
fi

echo "Starting memory monitoring for Campfire Rust..."
echo "Log file: $LOG_FILE"
echo "Check interval: ${INTERVAL} seconds"
echo "Alert threshold: ${ALERT_THRESHOLD}KB RSS"

while true; do
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    PID=$(pgrep campfire-rust)
    
    if [ -n "$PID" ]; then
        # Get process memory info
        RSS=$(ps -o rss= -p $PID | tr -d ' ')
        VSZ=$(ps -o vsz= -p $PID | tr -d ' ')
        CPU_PERCENT=$(ps -o %cpu= -p $PID | tr -d ' ')
        
        # Get application metrics (with timeout)
        HEAP=$(timeout 5 curl -s http://localhost:3000/metrics 2>/dev/null | grep memory_usage_bytes | cut -d' ' -f2 || echo "0")
        CONNECTIONS=$(timeout 5 curl -s http://localhost:3000/metrics 2>/dev/null | grep websocket_connections_active | cut -d' ' -f2 || echo "0")
        
        # Log memory usage
        echo "$TIMESTAMP,$RSS,$VSZ,$HEAP,$CONNECTIONS,$CPU_PERCENT" >> "$LOG_FILE"
        
        # Alert if memory usage exceeds threshold
        if [ "$RSS" -gt "$ALERT_THRESHOLD" ]; then
            echo "ALERT: High memory usage detected: ${RSS}KB RSS at $TIMESTAMP" | logger -t campfire-memory
            echo "ALERT: High memory usage detected: ${RSS}KB RSS at $TIMESTAMP"
        fi
        
        # Optional: Rotate log file if it gets too large (>10MB)
        if [ -f "$LOG_FILE" ] && [ $(stat -f%z "$LOG_FILE" 2>/dev/null || stat -c%s "$LOG_FILE" 2>/dev/null) -gt 10485760 ]; then
            mv "$LOG_FILE" "${LOG_FILE}.old"
            echo "timestamp,rss_kb,vsz_kb,heap_bytes,connections,cpu_percent" > "$LOG_FILE"
            echo "Log file rotated at $TIMESTAMP" | logger -t campfire-memory
        fi
    else
        echo "$TIMESTAMP,0,0,0,0,0" >> "$LOG_FILE"
        echo "WARNING: Campfire process not found at $TIMESTAMP" | logger -t campfire-memory
    fi
    
    sleep $INTERVAL
done