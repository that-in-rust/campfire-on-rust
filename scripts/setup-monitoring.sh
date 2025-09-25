#!/bin/bash

# Campfire Monitoring Setup Script
# Sets up comprehensive monitoring with Prometheus, Grafana, and alerting

set -euo pipefail

# Configuration
MONITORING_DIR="./monitoring"
GRAFANA_ADMIN_PASSWORD="${GRAFANA_ADMIN_PASSWORD:-admin}"
ALERT_EMAIL="${ALERT_EMAIL:-admin@campfire.local}"
SLACK_WEBHOOK_URL="${SLACK_WEBHOOK_URL:-}"

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

# Create monitoring directories
create_directories() {
    log "Creating monitoring directories..."
    
    mkdir -p "$MONITORING_DIR"/{rules,grafana/{dashboards,datasources,provisioning},alertmanager,blackbox}
    
    log "Monitoring directories created"
}

# Create Prometheus alerting rules
create_alerting_rules() {
    log "Creating Prometheus alerting rules..."
    
    cat > "$MONITORING_DIR/rules/campfire.yml" <<'EOF'
groups:
  - name: campfire.rules
    rules:
      # Application availability
      - alert: CampfireDown
        expr: up{job="campfire"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Campfire application is down"
          description: "Campfire has been down for more than 1 minute"

      # High error rate
      - alert: HighErrorRate
        expr: rate(campfire_http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanize }} errors per second"

      # High response time
      - alert: HighResponseTime
        expr: histogram_quantile(0.95, rate(campfire_http_request_duration_seconds_bucket[5m])) > 1.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High response time detected"
          description: "95th percentile response time is {{ $value | humanizeDuration }}"

      # Database issues
      - alert: DatabaseConnectionFailure
        expr: increase(campfire_database_operations_total{status="error"}[5m]) > 5
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Database connection failures"
          description: "{{ $value }} database operations failed in the last 5 minutes"

      # Memory usage
      - alert: HighMemoryUsage
        expr: (campfire_memory_usage_bytes / (1024*1024*1024)) > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value | humanize }}GB (>80%)"

      # Disk usage
      - alert: HighDiskUsage
        expr: (campfire_disk_usage_bytes / (1024*1024*1024)) > 5.0
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High disk usage"
          description: "Disk usage is {{ $value | humanize }}GB"

      # WebSocket connections
      - alert: WebSocketConnectionDrop
        expr: decrease(campfire_websocket_connections_active[5m]) > 10
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "WebSocket connections dropping"
          description: "{{ $value }} WebSocket connections dropped in 5 minutes"

      # Message processing
      - alert: MessageProcessingDelay
        expr: rate(campfire_messages_sent_total[5m]) == 0 and campfire_websocket_connections_active > 0
        for: 3m
        labels:
          severity: warning
        annotations:
          summary: "Message processing stopped"
          description: "No messages processed despite active connections"

      # Search performance
      - alert: SlowSearchQueries
        expr: histogram_quantile(0.95, rate(campfire_search_duration_seconds_bucket[5m])) > 2.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Slow search queries"
          description: "95th percentile search time is {{ $value | humanizeDuration }}"
EOF

    log "Alerting rules created"
}

# Create Grafana dashboard
create_grafana_dashboard() {
    log "Creating Grafana dashboard..."
    
    cat > "$MONITORING_DIR/grafana/dashboards/campfire-overview.json" <<'EOF'
{
  "dashboard": {
    "id": null,
    "title": "Campfire Overview",
    "tags": ["campfire"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "HTTP Requests per Second",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(campfire_http_requests_total[5m])",
            "legendFormat": "RPS"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "color": {
              "mode": "thresholds"
            },
            "thresholds": {
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 10},
                {"color": "red", "value": 50}
              ]
            }
          }
        },
        "gridPos": {"h": 8, "w": 6, "x": 0, "y": 0}
      },
      {
        "id": 2,
        "title": "Response Time (95th percentile)",
        "type": "stat",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(campfire_http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "s",
            "color": {
              "mode": "thresholds"
            },
            "thresholds": {
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 0.5},
                {"color": "red", "value": 1.0}
              ]
            }
          }
        },
        "gridPos": {"h": 8, "w": 6, "x": 6, "y": 0}
      },
      {
        "id": 3,
        "title": "Active WebSocket Connections",
        "type": "stat",
        "targets": [
          {
            "expr": "campfire_websocket_connections_active",
            "legendFormat": "Active Connections"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "color": {
              "mode": "thresholds"
            },
            "thresholds": {
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 100},
                {"color": "red", "value": 500}
              ]
            }
          }
        },
        "gridPos": {"h": 8, "w": 6, "x": 12, "y": 0}
      },
      {
        "id": 4,
        "title": "Memory Usage",
        "type": "stat",
        "targets": [
          {
            "expr": "campfire_memory_usage_bytes / (1024*1024*1024)",
            "legendFormat": "Memory (GB)"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "decbytes",
            "color": {
              "mode": "thresholds"
            },
            "thresholds": {
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 0.5},
                {"color": "red", "value": 1.0}
              ]
            }
          }
        },
        "gridPos": {"h": 8, "w": 6, "x": 18, "y": 0}
      }
    ],
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "refresh": "5s"
  }
}
EOF

    log "Grafana dashboard created"
}

# Create Alertmanager configuration
create_alertmanager_config() {
    log "Creating Alertmanager configuration..."
    
    cat > "$MONITORING_DIR/alertmanager/alertmanager.yml" <<EOF
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alerts@campfire.local'

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'

receivers:
  - name: 'web.hook'
    email_configs:
      - to: '$ALERT_EMAIL'
        subject: 'Campfire Alert: {{ .GroupLabels.alertname }}'
        body: |
          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          {{ end }}
EOF

    if [[ -n "$SLACK_WEBHOOK_URL" ]]; then
        cat >> "$MONITORING_DIR/alertmanager/alertmanager.yml" <<EOF
    slack_configs:
      - api_url: '$SLACK_WEBHOOK_URL'
        channel: '#alerts'
        title: 'Campfire Alert'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'
EOF
    fi

    log "Alertmanager configuration created"
}

# Create blackbox exporter configuration
create_blackbox_config() {
    log "Creating blackbox exporter configuration..."
    
    cat > "$MONITORING_DIR/blackbox/blackbox.yml" <<'EOF'
modules:
  http_2xx:
    prober: http
    timeout: 5s
    http:
      valid_http_versions: ["HTTP/1.1", "HTTP/2.0"]
      valid_status_codes: []
      method: GET
      follow_redirects: true
      preferred_ip_protocol: "ip4"

  http_post_2xx:
    prober: http
    timeout: 5s
    http:
      valid_http_versions: ["HTTP/1.1", "HTTP/2.0"]
      method: POST
      headers:
        Content-Type: application/json
      body: '{"test": true}'

  tcp_connect:
    prober: tcp
    timeout: 5s

  websocket:
    prober: http
    timeout: 5s
    http:
      valid_http_versions: ["HTTP/1.1"]
      method: GET
      headers:
        Connection: Upgrade
        Upgrade: websocket
        Sec-WebSocket-Key: "dGhlIHNhbXBsZSBub25jZQ=="
        Sec-WebSocket-Version: "13"
EOF

    log "Blackbox exporter configuration created"
}

# Update docker-compose with monitoring services
update_docker_compose() {
    log "Updating docker-compose with monitoring services..."
    
    # Check if monitoring services already exist
    if grep -q "prometheus:" docker-compose.yml; then
        info "Monitoring services already configured in docker-compose.yml"
        return
    fi
    
    # Add monitoring services to docker-compose.yml
    cat >> docker-compose.yml <<'EOF'

  # Alertmanager for alert routing
  alertmanager:
    image: prom/alertmanager:latest
    container_name: campfire-alertmanager
    restart: unless-stopped
    profiles:
      - monitoring
    
    ports:
      - "9093:9093"
    
    volumes:
      - ./monitoring/alertmanager:/etc/alertmanager:ro
      - alertmanager_data:/alertmanager
    
    command:
      - '--config.file=/etc/alertmanager/alertmanager.yml'
      - '--storage.path=/alertmanager'
      - '--web.external-url=http://localhost:9093'

  # Blackbox exporter for external monitoring
  blackbox:
    image: prom/blackbox-exporter:latest
    container_name: campfire-blackbox
    restart: unless-stopped
    profiles:
      - monitoring
    
    ports:
      - "9115:9115"
    
    volumes:
      - ./monitoring/blackbox:/etc/blackbox_exporter:ro
    
    command:
      - '--config.file=/etc/blackbox_exporter/blackbox.yml'

# Additional volumes for monitoring
volumes:
  alertmanager_data:
    driver: local
EOF

    log "Docker compose updated with monitoring services"
}

# Create monitoring startup script
create_startup_script() {
    log "Creating monitoring startup script..."
    
    cat > "$MONITORING_DIR/start-monitoring.sh" <<'EOF'
#!/bin/bash

# Start monitoring stack
set -euo pipefail

echo "Starting Campfire monitoring stack..."

# Start core monitoring
docker-compose --profile monitoring up -d prometheus grafana

# Wait for Prometheus to be ready
echo "Waiting for Prometheus to be ready..."
timeout=60
while ! curl -s http://localhost:9090/-/ready > /dev/null; do
    sleep 2
    timeout=$((timeout - 2))
    if [ $timeout -le 0 ]; then
        echo "Timeout waiting for Prometheus"
        exit 1
    fi
done

# Start alerting if configured
if [ -f "./monitoring/alertmanager/alertmanager.yml" ]; then
    echo "Starting Alertmanager..."
    docker-compose up -d alertmanager
fi

# Start blackbox monitoring
echo "Starting Blackbox exporter..."
docker-compose up -d blackbox

echo "Monitoring stack started successfully!"
echo ""
echo "Access points:"
echo "- Prometheus: http://localhost:9090"
echo "- Grafana: http://localhost:3001 (admin/admin)"
echo "- Alertmanager: http://localhost:9093"
echo "- Blackbox: http://localhost:9115"
EOF

    chmod +x "$MONITORING_DIR/start-monitoring.sh"
    
    log "Monitoring startup script created"
}

# Main setup function
setup_monitoring() {
    log "Setting up Campfire monitoring..."
    
    create_directories
    create_alerting_rules
    create_grafana_dashboard
    create_alertmanager_config
    create_blackbox_config
    update_docker_compose
    create_startup_script
    
    log "Monitoring setup completed!"
    echo ""
    info "Next steps:"
    info "1. Review and customize alerting rules in $MONITORING_DIR/rules/"
    info "2. Configure email/Slack notifications in $MONITORING_DIR/alertmanager/"
    info "3. Start monitoring with: $MONITORING_DIR/start-monitoring.sh"
    info "4. Or use: docker-compose --profile monitoring up -d"
}

# Usage function
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Setup comprehensive monitoring for Campfire"
    echo ""
    echo "Options:"
    echo "  -e, --email EMAIL      Alert email address (default: admin@campfire.local)"
    echo "  -s, --slack URL        Slack webhook URL for notifications"
    echo "  -p, --password PASS    Grafana admin password (default: admin)"
    echo "  -h, --help             Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  ALERT_EMAIL           Alert email address"
    echo "  SLACK_WEBHOOK_URL     Slack webhook URL"
    echo "  GRAFANA_ADMIN_PASSWORD Grafana admin password"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--email)
            ALERT_EMAIL="$2"
            shift 2
            ;;
        -s|--slack)
            SLACK_WEBHOOK_URL="$2"
            shift 2
            ;;
        -p|--password)
            GRAFANA_ADMIN_PASSWORD="$2"
            shift 2
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

# Run setup
setup_monitoring