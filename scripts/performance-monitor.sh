#!/bin/bash

# Campfire Performance Monitoring Script
# Monitors system performance and generates reports

set -euo pipefail

# Configuration
MONITOR_DURATION="${CAMPFIRE_MONITOR_DURATION:-300}"  # 5 minutes default
SAMPLE_INTERVAL="${CAMPFIRE_MONITOR_INTERVAL:-5}"     # 5 seconds default
OUTPUT_DIR="${CAMPFIRE_MONITOR_OUTPUT:-./monitoring/reports}"
CONTAINER_NAME="${CAMPFIRE_CONTAINER_NAME:-campfire}"
ALERT_THRESHOLD_CPU="${CAMPFIRE_ALERT_CPU:-80}"
ALERT_THRESHOLD_MEMORY="${CAMPFIRE_ALERT_MEMORY:-80}"
ALERT_THRESHOLD_RESPONSE="${CAMPFIRE_ALERT_RESPONSE:-1000}"  # milliseconds

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
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Monitor Campfire performance and generate reports"
    echo ""
    echo "Options:"
    echo "  -d, --duration SECONDS    Monitoring duration (default: 300)"
    echo "  -i, --interval SECONDS    Sample interval (default: 5)"
    echo "  -o, --output DIR          Output directory (default: ./monitoring/reports)"
    echo "  -c, --container NAME      Container name (default: campfire)"
    echo "  --cpu-threshold PERCENT   CPU alert threshold (default: 80)"
    echo "  --memory-threshold PERCENT Memory alert threshold (default: 80)"
    echo "  --response-threshold MS   Response time alert threshold (default: 1000)"
    echo "  --continuous              Run continuously until stopped"
    echo "  --report-only             Generate report from existing data"
    echo "  -h, --help                Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                        # Monitor for 5 minutes"
    echo "  $0 -d 600 -i 10          # Monitor for 10 minutes, sample every 10 seconds"
    echo "  $0 --continuous           # Monitor continuously"
    echo "  $0 --report-only          # Generate report from existing data"
    exit 1
}

# Parse command line arguments
CONTINUOUS=false
REPORT_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--duration)
            MONITOR_DURATION="$2"
            shift 2
            ;;
        -i|--interval)
            SAMPLE_INTERVAL="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -c|--container)
            CONTAINER_NAME="$2"
            shift 2
            ;;
        --cpu-threshold)
            ALERT_THRESHOLD_CPU="$2"
            shift 2
            ;;
        --memory-threshold)
            ALERT_THRESHOLD_MEMORY="$2"
            shift 2
            ;;
        --response-threshold)
            ALERT_THRESHOLD_RESPONSE="$2"
            shift 2
            ;;
        --continuous)
            CONTINUOUS=true
            shift
            ;;
        --report-only)
            REPORT_ONLY=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            error "Unknown option: $1"
            usage
            ;;
    esac
done

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Generate timestamp for this monitoring session
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
SESSION_DIR="$OUTPUT_DIR/session_$TIMESTAMP"
mkdir -p "$SESSION_DIR"

# Check if container is running
check_container() {
    if ! docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
        error "Container '$CONTAINER_NAME' is not running"
        exit 1
    fi
}

# Get container stats
get_container_stats() {
    docker stats --no-stream --format "table {{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.NetIO}}\t{{.BlockIO}}" "$CONTAINER_NAME"
}

# Get system stats
get_system_stats() {
    # CPU usage
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    
    # Memory usage
    local mem_info=$(free -m | grep "Mem:")
    local mem_total=$(echo $mem_info | awk '{print $2}')
    local mem_used=$(echo $mem_info | awk '{print $3}')
    local mem_percent=$(( mem_used * 100 / mem_total ))
    
    # Disk usage
    local disk_usage=$(df -h / | tail -1 | awk '{print $5}' | cut -d'%' -f1)
    
    # Load average
    local load_avg=$(uptime | awk -F'load average:' '{print $2}' | awk '{print $1}' | cut -d',' -f1)
    
    echo "$cpu_usage,$mem_percent,$disk_usage,$load_avg"
}

# Test application response time
test_response_time() {
    local url="http://localhost:3000/health"
    local response_time=$(curl -o /dev/null -s -w '%{time_total}' "$url" 2>/dev/null || echo "0")
    local response_ms=$(echo "$response_time * 1000" | bc -l | cut -d'.' -f1)
    echo "$response_ms"
}

# Get application metrics from Prometheus endpoint
get_app_metrics() {
    local metrics_url="http://localhost:3000/metrics"
    curl -s "$metrics_url" 2>/dev/null || echo ""
}

# Monitor WebSocket connections
monitor_websockets() {
    local metrics=$(get_app_metrics)
    if [[ -n "$metrics" ]]; then
        echo "$metrics" | grep "campfire_websocket_connections_active" | awk '{print $2}' || echo "0"
    else
        echo "0"
    fi
}

# Monitor database performance
monitor_database() {
    local db_file="./data/campfire.db"
    if [[ -f "$db_file" ]]; then
        local db_size=$(du -h "$db_file" | cut -f1)
        local table_count=$(sqlite3 "$db_file" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';" 2>/dev/null || echo "0")
        local message_count=$(sqlite3 "$db_file" "SELECT COUNT(*) FROM messages;" 2>/dev/null || echo "0")
        echo "$db_size,$table_count,$message_count"
    else
        echo "0,0,0"
    fi
}

# Check for alerts
check_alerts() {
    local cpu_percent="$1"
    local mem_percent="$2"
    local response_ms="$3"
    
    local alerts=()
    
    # CPU alert
    if (( $(echo "$cpu_percent > $ALERT_THRESHOLD_CPU" | bc -l) )); then
        alerts+=("HIGH_CPU:${cpu_percent}%")
    fi
    
    # Memory alert
    if (( mem_percent > ALERT_THRESHOLD_MEMORY )); then
        alerts+=("HIGH_MEMORY:${mem_percent}%")
    fi
    
    # Response time alert
    if (( response_ms > ALERT_THRESHOLD_RESPONSE )); then
        alerts+=("SLOW_RESPONSE:${response_ms}ms")
    fi
    
    if [[ ${#alerts[@]} -gt 0 ]]; then
        warn "ALERTS: ${alerts[*]}"
        echo "$(date -Iseconds),${alerts[*]}" >> "$SESSION_DIR/alerts.log"
    fi
}

# Single monitoring sample
take_sample() {
    local timestamp=$(date -Iseconds)
    
    # Container stats
    local container_stats=$(get_container_stats | tail -1)
    local cpu_percent=$(echo "$container_stats" | awk '{print $1}' | cut -d'%' -f1)
    local mem_usage=$(echo "$container_stats" | awk '{print $2}')
    local mem_percent=$(echo "$container_stats" | awk '{print $3}' | cut -d'%' -f1)
    
    # System stats
    local system_stats=$(get_system_stats)
    local sys_cpu=$(echo "$system_stats" | cut -d',' -f1)
    local sys_mem=$(echo "$system_stats" | cut -d',' -f2)
    local sys_disk=$(echo "$system_stats" | cut -d',' -f3)
    local sys_load=$(echo "$system_stats" | cut -d',' -f4)
    
    # Application metrics
    local response_ms=$(test_response_time)
    local websocket_count=$(monitor_websockets)
    local db_stats=$(monitor_database)
    
    # Log to CSV files
    echo "$timestamp,$cpu_percent,$mem_percent,$mem_usage" >> "$SESSION_DIR/container_stats.csv"
    echo "$timestamp,$sys_cpu,$sys_mem,$sys_disk,$sys_load" >> "$SESSION_DIR/system_stats.csv"
    echo "$timestamp,$response_ms,$websocket_count" >> "$SESSION_DIR/app_stats.csv"
    echo "$timestamp,$db_stats" >> "$SESSION_DIR/database_stats.csv"
    
    # Check for alerts
    check_alerts "$cpu_percent" "$sys_mem" "$response_ms"
    
    # Display current stats
    printf "%-20s CPU: %5s%% | Mem: %5s%% | Response: %5sms | WS: %5s | Load: %5s\n" \
        "$(date +'%H:%M:%S')" "$cpu_percent" "$sys_mem" "$response_ms" "$websocket_count" "$sys_load"
}

# Initialize CSV files with headers
initialize_csv_files() {
    echo "timestamp,cpu_percent,memory_percent,memory_usage" > "$SESSION_DIR/container_stats.csv"
    echo "timestamp,cpu_percent,memory_percent,disk_percent,load_average" > "$SESSION_DIR/system_stats.csv"
    echo "timestamp,response_time_ms,websocket_connections" > "$SESSION_DIR/app_stats.csv"
    echo "timestamp,database_size,table_count,message_count" > "$SESSION_DIR/database_stats.csv"
}

# Generate performance report
generate_report() {
    local report_file="$SESSION_DIR/performance_report.html"
    
    log "Generating performance report: $report_file"
    
    cat > "$report_file" <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>Campfire Performance Report - $TIMESTAMP</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .chart-container { width: 800px; height: 400px; margin: 20px 0; }
        .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; margin: 20px 0; }
        .stat-card { border: 1px solid #ddd; padding: 15px; border-radius: 5px; }
        .alert { background-color: #ffebee; border-left: 4px solid #f44336; padding: 10px; margin: 10px 0; }
        .summary { background-color: #e8f5e8; border-left: 4px solid #4caf50; padding: 15px; margin: 20px 0; }
    </style>
</head>
<body>
    <h1>Campfire Performance Report</h1>
    <p><strong>Generated:</strong> $(date)</p>
    <p><strong>Duration:</strong> $MONITOR_DURATION seconds</p>
    <p><strong>Sample Interval:</strong> $SAMPLE_INTERVAL seconds</p>
    
    <div class="summary">
        <h3>Summary</h3>
EOF

    # Calculate summary statistics
    if [[ -f "$SESSION_DIR/container_stats.csv" ]]; then
        local avg_cpu=$(tail -n +2 "$SESSION_DIR/container_stats.csv" | awk -F',' '{sum+=$2; count++} END {if(count>0) print sum/count; else print 0}')
        local max_cpu=$(tail -n +2 "$SESSION_DIR/container_stats.csv" | awk -F',' '{if($2>max) max=$2} END {print max+0}')
        local avg_mem=$(tail -n +2 "$SESSION_DIR/container_stats.csv" | awk -F',' '{sum+=$3; count++} END {if(count>0) print sum/count; else print 0}')
        local max_mem=$(tail -n +2 "$SESSION_DIR/container_stats.csv" | awk -F',' '{if($3>max) max=$3} END {print max+0}')
        
        cat >> "$report_file" <<EOF
        <p><strong>Average CPU Usage:</strong> $(printf "%.1f" $avg_cpu)%</p>
        <p><strong>Peak CPU Usage:</strong> $(printf "%.1f" $max_cpu)%</p>
        <p><strong>Average Memory Usage:</strong> $(printf "%.1f" $avg_mem)%</p>
        <p><strong>Peak Memory Usage:</strong> $(printf "%.1f" $max_mem)%</p>
EOF
    fi

    if [[ -f "$SESSION_DIR/app_stats.csv" ]]; then
        local avg_response=$(tail -n +2 "$SESSION_DIR/app_stats.csv" | awk -F',' '{sum+=$2; count++} END {if(count>0) print sum/count; else print 0}')
        local max_response=$(tail -n +2 "$SESSION_DIR/app_stats.csv" | awk -F',' '{if($2>max) max=$2} END {print max+0}')
        local max_ws=$(tail -n +2 "$SESSION_DIR/app_stats.csv" | awk -F',' '{if($3>max) max=$3} END {print max+0}')
        
        cat >> "$report_file" <<EOF
        <p><strong>Average Response Time:</strong> $(printf "%.0f" $avg_response)ms</p>
        <p><strong>Peak Response Time:</strong> $(printf "%.0f" $max_response)ms</p>
        <p><strong>Peak WebSocket Connections:</strong> $(printf "%.0f" $max_ws)</p>
EOF
    fi

    cat >> "$report_file" <<EOF
    </div>
    
    <!-- Alerts Section -->
EOF

    if [[ -f "$SESSION_DIR/alerts.log" ]]; then
        cat >> "$report_file" <<EOF
    <h3>Alerts</h3>
EOF
        while IFS= read -r alert_line; do
            echo "    <div class=\"alert\">$alert_line</div>" >> "$report_file"
        done < "$SESSION_DIR/alerts.log"
    fi

    cat >> "$report_file" <<EOF
    
    <!-- Charts would go here in a full implementation -->
    <h3>Performance Charts</h3>
    <p><em>Charts require JavaScript implementation with actual data parsing.</em></p>
    
    <!-- Raw Data Links -->
    <h3>Raw Data Files</h3>
    <ul>
        <li><a href="container_stats.csv">Container Statistics</a></li>
        <li><a href="system_stats.csv">System Statistics</a></li>
        <li><a href="app_stats.csv">Application Statistics</a></li>
        <li><a href="database_stats.csv">Database Statistics</a></li>
    </ul>
    
</body>
</html>
EOF

    log "Performance report generated: $report_file"
}

# Main monitoring loop
monitor_performance() {
    log "Starting performance monitoring..."
    info "Duration: $MONITOR_DURATION seconds"
    info "Interval: $SAMPLE_INTERVAL seconds"
    info "Output: $SESSION_DIR"
    
    # Check container
    check_container
    
    # Initialize CSV files
    initialize_csv_files
    
    # Display header
    echo ""
    printf "%-20s %-15s %-15s %-15s %-10s %-10s\n" "Time" "CPU" "Memory" "Response" "WebSocket" "Load"
    echo "--------------------------------------------------------------------------------"
    
    local samples=0
    local max_samples=$((MONITOR_DURATION / SAMPLE_INTERVAL))
    
    while true; do
        take_sample
        ((samples++))
        
        # Check if we should stop (unless continuous mode)
        if [[ "$CONTINUOUS" != "true" ]] && [[ $samples -ge $max_samples ]]; then
            break
        fi
        
        sleep "$SAMPLE_INTERVAL"
    done
    
    echo ""
    log "Monitoring completed. Samples taken: $samples"
}

# Continuous monitoring with signal handling
continuous_monitor() {
    log "Starting continuous monitoring (Ctrl+C to stop)..."
    
    # Set up signal handler
    trap 'log "Stopping continuous monitoring..."; generate_report; exit 0' INT TERM
    
    CONTINUOUS=true
    monitor_performance
}

# Main function
main() {
    if [[ "$REPORT_ONLY" == "true" ]]; then
        # Find the latest session directory
        local latest_session=$(find "$OUTPUT_DIR" -name "session_*" -type d | sort | tail -1)
        if [[ -n "$latest_session" ]]; then
            SESSION_DIR="$latest_session"
            generate_report
        else
            error "No monitoring sessions found in $OUTPUT_DIR"
            exit 1
        fi
    elif [[ "$CONTINUOUS" == "true" ]]; then
        continuous_monitor
    else
        monitor_performance
        generate_report
    fi
}

# Run main function
main "$@"