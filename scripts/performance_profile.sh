#!/bin/bash
# performance_profile.sh - Comprehensive performance profiling suite

PROFILE_DURATION=60  # Profile for 60 seconds by default
OUTPUT_DIR="/tmp/campfire-performance-$(date +%Y%m%d-%H%M%S)"
CAMPFIRE_PID=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--duration)
            PROFILE_DURATION="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  -d, --duration SECONDS  Profiling duration (default: 60)"
            echo "  -o, --output DIR        Output directory"
            echo "  -h, --help             Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

mkdir -p "$OUTPUT_DIR"

echo "=== Campfire Performance Profiling ==="
echo "Duration: ${PROFILE_DURATION} seconds"
echo "Output directory: $OUTPUT_DIR"

# Find Campfire process
CAMPFIRE_PID=$(pgrep campfire-rust)
if [ -z "$CAMPFIRE_PID" ]; then
    echo "❌ Campfire process not found. Please ensure Campfire is running."
    exit 1
fi

echo "Found Campfire process (PID: $CAMPFIRE_PID)"

# Function to cleanup background processes
cleanup() {
    echo "Cleaning up background processes..."
    jobs -p | xargs -r kill 2>/dev/null
    wait 2>/dev/null
}

trap cleanup EXIT

# 1. CPU Profiling with perf (if available)
if command -v perf >/dev/null 2>&1; then
    echo "Starting CPU profiling with perf..."
    perf record -g -p "$CAMPFIRE_PID" -o "$OUTPUT_DIR/perf.data" sleep "$PROFILE_DURATION" &
    PERF_PID=$!
else
    echo "⚠️  perf not available, skipping CPU profiling"
fi

# 2. Memory monitoring
echo "Starting memory monitoring..."
{
    echo "timestamp,rss_kb,vsz_kb,heap_mb,cpu_percent,threads"
    for i in $(seq 1 "$PROFILE_DURATION"); do
        TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
        
        # Get process stats
        if [ -f "/proc/$CAMPFIRE_PID/stat" ]; then
            PROC_STATS=$(cat "/proc/$CAMPFIRE_PID/stat" 2>/dev/null)
            RSS_PAGES=$(echo "$PROC_STATS" | awk '{print $24}')
            VSZ_BYTES=$(echo "$PROC_STATS" | awk '{print $23}')
            
            # Convert to KB/MB
            RSS_KB=$((RSS_PAGES * 4))  # Assuming 4KB pages
            VSZ_KB=$((VSZ_BYTES / 1024))
        else
            RSS_KB=0
            VSZ_KB=0
        fi
        
        # Get CPU percentage
        CPU_PERCENT=$(ps -p "$CAMPFIRE_PID" -o %cpu= 2>/dev/null | tr -d ' ' || echo "0")
        
        # Get thread count
        THREADS=$(ps -p "$CAMPFIRE_PID" -o nlwp= 2>/dev/null | tr -d ' ' || echo "0")
        
        # Get heap usage from metrics (if available)
        HEAP_BYTES=$(timeout 2 curl -s http://localhost:3000/metrics 2>/dev/null | grep memory_usage_bytes | cut -d' ' -f2 || echo "0")
        HEAP_MB=$((HEAP_BYTES / 1024 / 1024))
        
        echo "$TIMESTAMP,$RSS_KB,$VSZ_KB,$HEAP_MB,$CPU_PERCENT,$THREADS"
        sleep 1
    done
} > "$OUTPUT_DIR/memory_usage.csv" &
MEMORY_PID=$!

# 3. I/O monitoring with iostat (if available)
if command -v iostat >/dev/null 2>&1; then
    echo "Starting I/O monitoring..."
    iostat -x 1 "$PROFILE_DURATION" > "$OUTPUT_DIR/iostat.log" &
    IOSTAT_PID=$!
fi

# 4. Network monitoring
echo "Starting network monitoring..."
{
    echo "timestamp,tcp_connections,websocket_connections,bytes_sent,bytes_received"
    for i in $(seq 1 "$PROFILE_DURATION"); do
        TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
        
        # Count TCP connections
        TCP_CONN=$(ss -t state established 2>/dev/null | grep -c ":3000" || echo "0")
        
        # Get WebSocket connections from metrics
        WS_CONN=$(timeout 2 curl -s http://localhost:3000/metrics 2>/dev/null | grep websocket_connections_active | cut -d' ' -f2 || echo "0")
        
        # Get network bytes (simplified)
        if [ -f "/proc/$CAMPFIRE_PID/net/dev" ]; then
            NET_STATS=$(cat "/proc/$CAMPFIRE_PID/net/dev" 2>/dev/null | grep -E "eth0|wlan0|enp" | head -1)
            BYTES_RX=$(echo "$NET_STATS" | awk '{print $2}' || echo "0")
            BYTES_TX=$(echo "$NET_STATS" | awk '{print $10}' || echo "0")
        else
            BYTES_RX=0
            BYTES_TX=0
        fi
        
        echo "$TIMESTAMP,$TCP_CONN,$WS_CONN,$BYTES_TX,$BYTES_RX"
        sleep 1
    done
} > "$OUTPUT_DIR/network_usage.csv" &
NETWORK_PID=$!

# 5. Application metrics monitoring
echo "Starting application metrics monitoring..."
{
    echo "timestamp,http_requests_total,http_request_duration_avg,database_queries_total,database_query_duration_avg,websocket_messages_total"
    for i in $(seq 1 "$PROFILE_DURATION"); do
        TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
        
        # Get application metrics
        METRICS=$(timeout 3 curl -s http://localhost:3000/metrics 2>/dev/null || echo "")
        
        HTTP_REQUESTS=$(echo "$METRICS" | grep http_requests_total | tail -1 | cut -d' ' -f2 || echo "0")
        HTTP_DURATION=$(echo "$METRICS" | grep http_request_duration_seconds | grep -v bucket | tail -1 | cut -d' ' -f2 || echo "0")
        DB_QUERIES=$(echo "$METRICS" | grep database_queries_total | tail -1 | cut -d' ' -f2 || echo "0")
        DB_DURATION=$(echo "$METRICS" | grep database_query_duration_seconds | grep -v bucket | tail -1 | cut -d' ' -f2 || echo "0")
        WS_MESSAGES=$(echo "$METRICS" | grep websocket_messages_total | tail -1 | cut -d' ' -f2 || echo "0")
        
        echo "$TIMESTAMP,$HTTP_REQUESTS,$HTTP_DURATION,$DB_QUERIES,$DB_DURATION,$WS_MESSAGES"
        sleep 1
    done
} > "$OUTPUT_DIR/app_metrics.csv" &
METRICS_PID=$!

# 6. System load monitoring
echo "Starting system load monitoring..."
{
    echo "timestamp,load_1min,load_5min,load_15min,cpu_user,cpu_system,cpu_idle,memory_used_percent,swap_used_percent"
    for i in $(seq 1 "$PROFILE_DURATION"); do
        TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
        
        # Load averages
        LOAD_AVG=$(cat /proc/loadavg)
        LOAD_1=$(echo "$LOAD_AVG" | awk '{print $1}')
        LOAD_5=$(echo "$LOAD_AVG" | awk '{print $2}')
        LOAD_15=$(echo "$LOAD_AVG" | awk '{print $3}')
        
        # CPU usage
        CPU_STATS=$(grep "cpu " /proc/stat | awk '{print ($2+$4)*100/($2+$3+$4+$5), ($3)*100/($2+$3+$4+$5), ($5)*100/($2+$3+$4+$5)}')
        CPU_USER=$(echo "$CPU_STATS" | awk '{print $1}')
        CPU_SYS=$(echo "$CPU_STATS" | awk '{print $2}')
        CPU_IDLE=$(echo "$CPU_STATS" | awk '{print $3}')
        
        # Memory usage
        MEM_INFO=$(free | grep Mem:)
        MEM_TOTAL=$(echo "$MEM_INFO" | awk '{print $2}')
        MEM_USED=$(echo "$MEM_INFO" | awk '{print $3}')
        MEM_PERCENT=$((MEM_USED * 100 / MEM_TOTAL))
        
        # Swap usage
        SWAP_INFO=$(free | grep Swap:)
        SWAP_TOTAL=$(echo "$SWAP_INFO" | awk '{print $2}')
        SWAP_USED=$(echo "$SWAP_INFO" | awk '{print $3}')
        if [ "$SWAP_TOTAL" -gt 0 ]; then
            SWAP_PERCENT=$((SWAP_USED * 100 / SWAP_TOTAL))
        else
            SWAP_PERCENT=0
        fi
        
        echo "$TIMESTAMP,$LOAD_1,$LOAD_5,$LOAD_15,$CPU_USER,$CPU_SYS,$CPU_IDLE,$MEM_PERCENT,$SWAP_PERCENT"
        sleep 1
    done
} > "$OUTPUT_DIR/system_load.csv" &
LOAD_PID=$!

# Wait for profiling to complete
echo "Profiling in progress... (${PROFILE_DURATION}s)"
sleep "$PROFILE_DURATION"

# Wait for background processes to complete
echo "Waiting for monitoring processes to complete..."
wait $MEMORY_PID 2>/dev/null
wait $NETWORK_PID 2>/dev/null
wait $METRICS_PID 2>/dev/null
wait $LOAD_PID 2>/dev/null

if [ -n "$PERF_PID" ]; then
    wait $PERF_PID 2>/dev/null
fi

if [ -n "$IOSTAT_PID" ]; then
    wait $IOSTAT_PID 2>/dev/null
fi

# Generate reports
echo "Generating performance reports..."

# CPU Analysis (if perf data available)
if [ -f "$OUTPUT_DIR/perf.data" ] && command -v perf >/dev/null 2>&1; then
    echo "Generating CPU analysis..."
    perf report -i "$OUTPUT_DIR/perf.data" --stdio > "$OUTPUT_DIR/cpu_analysis.txt" 2>/dev/null
    
    # Generate flame graph if tools are available
    if command -v perf >/dev/null 2>&1 && command -v flamegraph >/dev/null 2>&1; then
        perf script -i "$OUTPUT_DIR/perf.data" | flamegraph > "$OUTPUT_DIR/flamegraph.svg" 2>/dev/null
    fi
fi

# Memory Analysis
if [ -f "$OUTPUT_DIR/memory_usage.csv" ]; then
    echo "Generating memory analysis..."
    {
        echo "=== Memory Usage Analysis ==="
        echo ""
        echo "Peak RSS Memory: $(tail -n +2 "$OUTPUT_DIR/memory_usage.csv" | cut -d',' -f2 | sort -n | tail -1) KB"
        echo "Average RSS Memory: $(tail -n +2 "$OUTPUT_DIR/memory_usage.csv" | cut -d',' -f2 | awk '{sum+=$1} END {print sum/NR}') KB"
        echo "Peak Heap Memory: $(tail -n +2 "$OUTPUT_DIR/memory_usage.csv" | cut -d',' -f4 | sort -n | tail -1) MB"
        echo "Average CPU Usage: $(tail -n +2 "$OUTPUT_DIR/memory_usage.csv" | cut -d',' -f5 | awk '{sum+=$1} END {print sum/NR}')%"
        echo "Thread Count Range: $(tail -n +2 "$OUTPUT_DIR/memory_usage.csv" | cut -d',' -f6 | sort -n | head -1)-$(tail -n +2 "$OUTPUT_DIR/memory_usage.csv" | cut -d',' -f6 | sort -n | tail -1)"
        echo ""
        echo "Memory usage over time (last 10 samples):"
        echo "Timestamp,RSS(KB),VSZ(KB),Heap(MB),CPU%,Threads"
        tail -10 "$OUTPUT_DIR/memory_usage.csv"
    } > "$OUTPUT_DIR/memory_analysis.txt"
fi

# Network Analysis
if [ -f "$OUTPUT_DIR/network_usage.csv" ]; then
    echo "Generating network analysis..."
    {
        echo "=== Network Usage Analysis ==="
        echo ""
        echo "Peak TCP Connections: $(tail -n +2 "$OUTPUT_DIR/network_usage.csv" | cut -d',' -f2 | sort -n | tail -1)"
        echo "Peak WebSocket Connections: $(tail -n +2 "$OUTPUT_DIR/network_usage.csv" | cut -d',' -f3 | sort -n | tail -1)"
        echo "Average TCP Connections: $(tail -n +2 "$OUTPUT_DIR/network_usage.csv" | cut -d',' -f2 | awk '{sum+=$1} END {print sum/NR}')"
        echo "Average WebSocket Connections: $(tail -n +2 "$OUTPUT_DIR/network_usage.csv" | cut -d',' -f3 | awk '{sum+=$1} END {print sum/NR}')"
        echo ""
        echo "Connection usage over time (last 10 samples):"
        echo "Timestamp,TCP,WebSocket,BytesSent,BytesReceived"
        tail -10 "$OUTPUT_DIR/network_usage.csv"
    } > "$OUTPUT_DIR/network_analysis.txt"
fi

# Application Metrics Analysis
if [ -f "$OUTPUT_DIR/app_metrics.csv" ]; then
    echo "Generating application metrics analysis..."
    {
        echo "=== Application Metrics Analysis ==="
        echo ""
        
        # Calculate request rate
        FIRST_REQUESTS=$(tail -n +2 "$OUTPUT_DIR/app_metrics.csv" | head -1 | cut -d',' -f2)
        LAST_REQUESTS=$(tail -1 "$OUTPUT_DIR/app_metrics.csv" | cut -d',' -f2)
        REQUEST_RATE=$(( (LAST_REQUESTS - FIRST_REQUESTS) / PROFILE_DURATION ))
        
        echo "HTTP Request Rate: ${REQUEST_RATE} requests/second"
        echo "Total HTTP Requests: $((LAST_REQUESTS - FIRST_REQUESTS))"
        
        # Database metrics
        FIRST_DB_QUERIES=$(tail -n +2 "$OUTPUT_DIR/app_metrics.csv" | head -1 | cut -d',' -f4)
        LAST_DB_QUERIES=$(tail -1 "$OUTPUT_DIR/app_metrics.csv" | cut -d',' -f4)
        DB_QUERY_RATE=$(( (LAST_DB_QUERIES - FIRST_DB_QUERIES) / PROFILE_DURATION ))
        
        echo "Database Query Rate: ${DB_QUERY_RATE} queries/second"
        echo "Total Database Queries: $((LAST_DB_QUERIES - FIRST_DB_QUERIES))"
        
        # WebSocket metrics
        FIRST_WS_MESSAGES=$(tail -n +2 "$OUTPUT_DIR/app_metrics.csv" | head -1 | cut -d',' -f6)
        LAST_WS_MESSAGES=$(tail -1 "$OUTPUT_DIR/app_metrics.csv" | cut -d',' -f6)
        WS_MESSAGE_RATE=$(( (LAST_WS_MESSAGES - FIRST_WS_MESSAGES) / PROFILE_DURATION ))
        
        echo "WebSocket Message Rate: ${WS_MESSAGE_RATE} messages/second"
        echo "Total WebSocket Messages: $((LAST_WS_MESSAGES - FIRST_WS_MESSAGES))"
        
        echo ""
        echo "Application metrics over time (last 10 samples):"
        echo "Timestamp,HTTPReqs,HTTPDuration,DBQueries,DBDuration,WSMessages"
        tail -10 "$OUTPUT_DIR/app_metrics.csv"
    } > "$OUTPUT_DIR/app_metrics_analysis.txt"
fi

# System Load Analysis
if [ -f "$OUTPUT_DIR/system_load.csv" ]; then
    echo "Generating system load analysis..."
    {
        echo "=== System Load Analysis ==="
        echo ""
        echo "Peak 1-min Load: $(tail -n +2 "$OUTPUT_DIR/system_load.csv" | cut -d',' -f2 | sort -n | tail -1)"
        echo "Average 1-min Load: $(tail -n +2 "$OUTPUT_DIR/system_load.csv" | cut -d',' -f2 | awk '{sum+=$1} END {print sum/NR}')"
        echo "Peak Memory Usage: $(tail -n +2 "$OUTPUT_DIR/system_load.csv" | cut -d',' -f8 | sort -n | tail -1)%"
        echo "Average Memory Usage: $(tail -n +2 "$OUTPUT_DIR/system_load.csv" | cut -d',' -f8 | awk '{sum+=$1} END {print sum/NR}')%"
        echo "Peak CPU User: $(tail -n +2 "$OUTPUT_DIR/system_load.csv" | cut -d',' -f5 | sort -n | tail -1)%"
        echo "Average CPU User: $(tail -n +2 "$OUTPUT_DIR/system_load.csv" | cut -d',' -f5 | awk '{sum+=$1} END {print sum/NR}')%"
        echo ""
        echo "System load over time (last 10 samples):"
        echo "Timestamp,Load1m,Load5m,Load15m,CPU_User%,CPU_Sys%,CPU_Idle%,Mem%,Swap%"
        tail -10 "$OUTPUT_DIR/system_load.csv"
    } > "$OUTPUT_DIR/system_load_analysis.txt"
fi

# Generate summary report
echo "Generating summary report..."
{
    echo "=== Campfire Performance Profile Summary ==="
    echo "Generated: $(date)"
    echo "Profile Duration: ${PROFILE_DURATION} seconds"
    echo "Process ID: $CAMPFIRE_PID"
    echo ""
    
    # Memory summary
    if [ -f "$OUTPUT_DIR/memory_analysis.txt" ]; then
        echo "=== Memory Performance ==="
        grep -E "(Peak|Average)" "$OUTPUT_DIR/memory_analysis.txt"
        echo ""
    fi
    
    # Network summary
    if [ -f "$OUTPUT_DIR/network_analysis.txt" ]; then
        echo "=== Network Performance ==="
        grep -E "(Peak|Average)" "$OUTPUT_DIR/network_analysis.txt"
        echo ""
    fi
    
    # Application summary
    if [ -f "$OUTPUT_DIR/app_metrics_analysis.txt" ]; then
        echo "=== Application Performance ==="
        grep -E "(Rate|Total)" "$OUTPUT_DIR/app_metrics_analysis.txt"
        echo ""
    fi
    
    # System summary
    if [ -f "$OUTPUT_DIR/system_load_analysis.txt" ]; then
        echo "=== System Performance ==="
        grep -E "(Peak|Average)" "$OUTPUT_DIR/system_load_analysis.txt"
        echo ""
    fi
    
    echo "=== Performance Recommendations ==="
    
    # Generate recommendations based on findings
    if [ -f "$OUTPUT_DIR/memory_usage.csv" ]; then
        PEAK_RSS=$(tail -n +2 "$OUTPUT_DIR/memory_usage.csv" | cut -d',' -f2 | sort -n | tail -1)
        if [ "$PEAK_RSS" -gt 1048576 ]; then  # > 1GB
            echo "- High memory usage detected (${PEAK_RSS}KB), consider memory optimization"
        fi
        
        AVG_CPU=$(tail -n +2 "$OUTPUT_DIR/memory_usage.csv" | cut -d',' -f5 | awk '{sum+=$1} END {print int(sum/NR)}')
        if [ "$AVG_CPU" -gt 80 ]; then
            echo "- High CPU usage detected (${AVG_CPU}%), consider CPU optimization"
        fi
    fi
    
    if [ -f "$OUTPUT_DIR/system_load.csv" ]; then
        PEAK_LOAD=$(tail -n +2 "$OUTPUT_DIR/system_load.csv" | cut -d',' -f2 | sort -n | tail -1 | cut -d'.' -f1)
        CPU_COUNT=$(nproc)
        if [ "$PEAK_LOAD" -gt "$CPU_COUNT" ]; then
            echo "- System overloaded (load: $PEAK_LOAD, CPUs: $CPU_COUNT), consider scaling"
        fi
    fi
    
    echo ""
    echo "=== Files Generated ==="
    ls -la "$OUTPUT_DIR/"
    
} > "$OUTPUT_DIR/performance_summary.txt"

# Create archive
echo "Creating performance profile archive..."
ARCHIVE_NAME="${OUTPUT_DIR}.tar.gz"
tar -czf "$ARCHIVE_NAME" -C "$(dirname "$OUTPUT_DIR")" "$(basename "$OUTPUT_DIR")" 2>/dev/null

echo ""
echo "=== Performance Profiling Complete ==="
if [ -f "$ARCHIVE_NAME" ]; then
    echo "Archive created: $ARCHIVE_NAME"
    echo "Archive size: $(ls -lh "$ARCHIVE_NAME" | awk '{print $5}')"
else
    echo "Files available in: $OUTPUT_DIR"
fi

echo ""
echo "=== Performance Summary ==="
cat "$OUTPUT_DIR/performance_summary.txt"

echo ""
echo "To analyze the results:"
echo "  - View summary: cat $OUTPUT_DIR/performance_summary.txt"
echo "  - Memory trends: cat $OUTPUT_DIR/memory_analysis.txt"
echo "  - Network usage: cat $OUTPUT_DIR/network_analysis.txt"
echo "  - App metrics: cat $OUTPUT_DIR/app_metrics_analysis.txt"
if [ -f "$OUTPUT_DIR/flamegraph.svg" ]; then
    echo "  - CPU flame graph: open $OUTPUT_DIR/flamegraph.svg"
fi