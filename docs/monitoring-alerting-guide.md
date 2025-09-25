# Monitoring and Alerting Guide for Campfire Rust

This guide provides comprehensive instructions for setting up monitoring, alerting, and observability for Campfire deployments, from basic health checks to enterprise-grade monitoring solutions.

## Table of Contents

1. [Monitoring Overview](#monitoring-overview)
2. [Health Check Endpoints](#health-check-endpoints)
3. [Metrics Collection](#metrics-collection)
4. [Prometheus Setup](#prometheus-setup)
5. [Grafana Dashboards](#grafana-dashboards)
6. [Alerting Configuration](#alerting-configuration)
7. [Log Management](#log-management)
8. [Performance Monitoring](#performance-monitoring)
9. [External Monitoring](#external-monitoring)
10. [Troubleshooting](#troubleshooting)

## Monitoring Overview

### Monitoring Stack Architecture

```mermaid
graph TB
    subgraph "Application Layer"
        APP[Campfire Application]
        METRICS[/metrics endpoint]
        HEALTH[/health endpoints]
        LOGS[Application Logs]
    end
    
    subgraph "Collection Layer"
        PROM[Prometheus]
        LOKI[Loki]
        BLACKBOX[Blackbox Exporter]
    end
    
    subgraph "Storage Layer"
        PROMDB[(Prometheus TSDB)]
        LOKIDB[(Loki Storage)]
    end
    
    subgraph "Visualization Layer"
        GRAFANA[Grafana]
        ALERTS[Alertmanager]
    end
    
    subgraph "Notification Layer"
        EMAIL[Email]
        SLACK[Slack]
        WEBHOOK[Webhooks]
    end
    
    APP --> METRICS
    APP --> HEALTH
    APP --> LOGS
    
    PROM --> METRICS
    BLACKBOX --> HEALTH
    LOKI --> LOGS
    
    PROM --> PROMDB
    LOKI --> LOKIDB
    
    PROMDB --> GRAFANA
    LOKIDB --> GRAFANA
    PROMDB --> ALERTS
    
    ALERTS --> EMAIL
    ALERTS --> SLACK
    ALERTS --> WEBHOOK
```

### Key Metrics Categories

| Category | Metrics | Purpose |
|----------|---------|---------|
| **Application** | Request rate, response time, error rate | Performance monitoring |
| **Business** | Active users, messages sent, rooms created | Business KPIs |
| **Infrastructure** | CPU, memory, disk, network | Resource monitoring |
| **Database** | Query time, connections, cache hit rate | Database performance |
| **WebSocket** | Active connections, message throughput | Real-time features |

## Health Check Endpoints

### Available Health Endpoints

Campfire provides multiple health check endpoints for different monitoring needs:

```bash
# Basic health check (200 OK if running)
curl http://localhost:3000/health

# Readiness check (ready to serve traffic)
curl http://localhost:3000/health/ready

# Liveness check (application is functioning)
curl http://localhost:3000/health/live

# Detailed health with component status
curl http://localhost:3000/health/detailed | jq
```

### Health Check Response Format

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00Z",
  "version": "1.0.0",
  "uptime": 3600,
  "checks": {
    "database": {
      "status": "healthy",
      "response_time_ms": 2,
      "details": {
        "connection_pool": "8/20 connections",
        "last_query": "2024-01-01T11:59:58Z"
      }
    },
    "websocket": {
      "status": "healthy",
      "active_connections": 150,
      "details": {
        "connection_limit": 5000,
        "message_rate": "45/sec"
      }
    },
    "search": {
      "status": "healthy",
      "response_time_ms": 15,
      "details": {
        "index_size": "2.5MB",
        "last_index_update": "2024-01-01T11:58:00Z"
      }
    },
    "push_notifications": {
      "status": "healthy",
      "details": {
        "vapid_configured": true,
        "active_subscriptions": 89
      }
    }
  }
}
```

### Custom Health Checks

```rust
// Example custom health check implementation
use axum::{Json, response::Json as ResponseJson};
use serde_json::{json, Value};

pub async fn detailed_health_check(
    State(app_state): State<AppState>,
) -> Result<ResponseJson<Value>, AppError> {
    let mut checks = serde_json::Map::new();
    
    // Database health check
    let db_start = Instant::now();
    let db_result = app_state.database.health_check().await;
    let db_duration = db_start.elapsed();
    
    checks.insert("database".to_string(), json!({
        "status": if db_result.is_ok() { "healthy" } else { "unhealthy" },
        "response_time_ms": db_duration.as_millis(),
        "error": db_result.err().map(|e| e.to_string())
    }));
    
    // WebSocket health check
    let ws_connections = app_state.connection_manager.active_connections().await;
    checks.insert("websocket".to_string(), json!({
        "status": "healthy",
        "active_connections": ws_connections,
        "connection_limit": 5000
    }));
    
    let overall_status = if checks.values().all(|check| 
        check.get("status").and_then(|s| s.as_str()) == Some("healthy")
    ) {
        "healthy"
    } else {
        "unhealthy"
    };
    
    Ok(ResponseJson(json!({
        "status": overall_status,
        "timestamp": chrono::Utc::now(),
        "checks": checks
    })))
}
```

## Metrics Collection

### Application Metrics

Campfire exposes Prometheus-compatible metrics at `/metrics`:

```bash
# View all metrics
curl http://localhost:3000/metrics

# Filter specific metrics
curl http://localhost:3000/metrics | grep campfire_http_requests_total
```

### Core Application Metrics

```prometheus
# HTTP Request Metrics
campfire_http_requests_total{method="GET",status="200"} 1234
campfire_http_request_duration_seconds_bucket{method="GET",le="0.1"} 1000
campfire_http_request_size_bytes_bucket{method="POST",le="1024"} 500

# WebSocket Metrics
campfire_websocket_connections_active 150
campfire_websocket_messages_sent_total 5678
campfire_websocket_connection_duration_seconds_bucket{le="3600"} 100

# Database Metrics
campfire_database_operations_total{operation="select",status="success"} 9876
campfire_database_operation_duration_seconds{operation="insert"} 0.005
campfire_database_connections_active 8
campfire_database_connections_max 20

# Business Metrics
campfire_users_active_total 89
campfire_rooms_total 12
campfire_messages_sent_total 5678
campfire_search_queries_total 234

# System Metrics
campfire_memory_usage_bytes 536870912
campfire_cpu_usage_percent 25.5
campfire_disk_usage_bytes 1073741824
campfire_uptime_seconds 3600
```

### Custom Metrics Implementation

```rust
use prometheus::{Counter, Histogram, Gauge, register_counter, register_histogram, register_gauge};

// Define custom metrics
lazy_static! {
    static ref MESSAGE_COUNTER: Counter = register_counter!(
        "campfire_messages_sent_total",
        "Total number of messages sent"
    ).unwrap();
    
    static ref RESPONSE_TIME_HISTOGRAM: Histogram = register_histogram!(
        "campfire_http_request_duration_seconds",
        "HTTP request duration in seconds",
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]
    ).unwrap();
    
    static ref ACTIVE_CONNECTIONS: Gauge = register_gauge!(
        "campfire_websocket_connections_active",
        "Number of active WebSocket connections"
    ).unwrap();
}

// Use metrics in application code
pub async fn send_message(message: Message) -> Result<(), MessageError> {
    let start_time = Instant::now();
    
    // Send message logic here
    let result = send_message_impl(message).await;
    
    // Record metrics
    MESSAGE_COUNTER.inc();
    RESPONSE_TIME_HISTOGRAM.observe(start_time.elapsed().as_secs_f64());
    
    result
}
```

## Prometheus Setup

### Prometheus Configuration

Create `monitoring/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'campfire-prod'
    environment: 'production'

rule_files:
  - "rules/*.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  # Campfire application metrics
  - job_name: 'campfire'
    static_configs:
      - targets: ['campfire:3000']
    metrics_path: '/metrics'
    scrape_interval: 10s
    scrape_timeout: 5s
    
  # Blackbox monitoring for external endpoints
  - job_name: 'blackbox'
    metrics_path: /probe
    params:
      module: [http_2xx]
    static_configs:
      - targets:
        - https://chat.yourdomain.com/health
        - https://chat.yourdomain.com/health/ready
    relabel_configs:
      - source_labels: [__address__]
        target_label: __param_target
      - source_labels: [__param_target]
        target_label: instance
      - target_label: __address__
        replacement: blackbox:9115

  # Node exporter for system metrics (optional)
  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']

  # Docker container metrics (optional)
  - job_name: 'docker'
    static_configs:
      - targets: ['cadvisor:8080']
```

### Prometheus Alerting Rules

Create `monitoring/rules/campfire.yml`:

```yaml
groups:
  - name: campfire.application
    rules:
      # Application availability
      - alert: CampfireDown
        expr: up{job="campfire"} == 0
        for: 1m
        labels:
          severity: critical
          component: application
        annotations:
          summary: "Campfire application is down"
          description: "Campfire has been down for more than 1 minute"
          runbook_url: "https://docs.yourdomain.com/runbooks/campfire-down"

      # High error rate
      - alert: HighErrorRate
        expr: |
          (
            rate(campfire_http_requests_total{status=~"5.."}[5m]) /
            rate(campfire_http_requests_total[5m])
          ) > 0.05
        for: 2m
        labels:
          severity: warning
          component: application
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }} (>5%)"

      # High response time
      - alert: HighResponseTime
        expr: |
          histogram_quantile(0.95, 
            rate(campfire_http_request_duration_seconds_bucket[5m])
          ) > 1.0
        for: 5m
        labels:
          severity: warning
          component: performance
        annotations:
          summary: "High response time detected"
          description: "95th percentile response time is {{ $value | humanizeDuration }}"

  - name: campfire.database
    rules:
      # Database connection failures
      - alert: DatabaseConnectionFailure
        expr: |
          increase(campfire_database_operations_total{status="error"}[5m]) > 5
        for: 1m
        labels:
          severity: critical
          component: database
        annotations:
          summary: "Database connection failures"
          description: "{{ $value }} database operations failed in the last 5 minutes"

      # High database response time
      - alert: SlowDatabaseQueries
        expr: |
          histogram_quantile(0.95,
            rate(campfire_database_operation_duration_seconds_bucket[5m])
          ) > 0.1
        for: 3m
        labels:
          severity: warning
          component: database
        annotations:
          summary: "Slow database queries detected"
          description: "95th percentile database query time is {{ $value | humanizeDuration }}"

  - name: campfire.resources
    rules:
      # High memory usage
      - alert: HighMemoryUsage
        expr: |
          (campfire_memory_usage_bytes / (1024*1024*1024)) > 1.5
        for: 5m
        labels:
          severity: warning
          component: resources
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value | humanize }}GB"

      # High CPU usage
      - alert: HighCPUUsage
        expr: campfire_cpu_usage_percent > 80
        for: 5m
        labels:
          severity: warning
          component: resources
        annotations:
          summary: "High CPU usage"
          description: "CPU usage is {{ $value }}%"

  - name: campfire.websocket
    rules:
      # WebSocket connection drops
      - alert: WebSocketConnectionDrop
        expr: |
          decrease(campfire_websocket_connections_active[5m]) > 50
        for: 2m
        labels:
          severity: warning
          component: websocket
        annotations:
          summary: "Large WebSocket connection drop"
          description: "{{ $value }} WebSocket connections dropped in 5 minutes"

      # No WebSocket activity
      - alert: NoWebSocketActivity
        expr: |
          rate(campfire_websocket_messages_sent_total[5m]) == 0 and
          campfire_websocket_connections_active > 0
        for: 10m
        labels:
          severity: warning
          component: websocket
        annotations:
          summary: "No WebSocket message activity"
          description: "No WebSocket messages sent despite active connections"
```

## Grafana Dashboards

### Main Application Dashboard

Create `monitoring/grafana/dashboards/campfire-overview.json`:

```json
{
  "dashboard": {
    "id": null,
    "title": "Campfire Overview",
    "description": "Main dashboard for Campfire application monitoring",
    "tags": ["campfire", "overview"],
    "timezone": "browser",
    "refresh": "30s",
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "panels": [
      {
        "id": 1,
        "title": "Request Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(campfire_http_requests_total[5m])",
            "legendFormat": "Requests/sec"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "reqps",
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
        "title": "Error Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(campfire_http_requests_total{status=~\"5..\"}[5m]) / rate(campfire_http_requests_total[5m])",
            "legendFormat": "Error Rate"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percentunit",
            "color": {
              "mode": "thresholds"
            },
            "thresholds": {
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 0.01},
                {"color": "red", "value": 0.05}
              ]
            }
          }
        },
        "gridPos": {"h": 8, "w": 6, "x": 12, "y": 0}
      },
      {
        "id": 4,
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
                {"color": "yellow", "value": 1000},
                {"color": "red", "value": 4000}
              ]
            }
          }
        },
        "gridPos": {"h": 8, "w": 6, "x": 18, "y": 0}
      },
      {
        "id": 5,
        "title": "Request Rate Over Time",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(campfire_http_requests_total[5m])",
            "legendFormat": "{{method}} {{status}}"
          }
        ],
        "yAxes": [
          {
            "label": "Requests/sec",
            "min": 0
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8}
      },
      {
        "id": 6,
        "title": "Response Time Distribution",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(campfire_http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "50th percentile"
          },
          {
            "expr": "histogram_quantile(0.95, rate(campfire_http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "histogram_quantile(0.99, rate(campfire_http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "99th percentile"
          }
        ],
        "yAxes": [
          {
            "label": "Response Time (s)",
            "min": 0
          }
        ],
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8}
      }
    ]
  }
}
```

### Business Metrics Dashboard

Create `monitoring/grafana/dashboards/campfire-business.json`:

```json
{
  "dashboard": {
    "title": "Campfire Business Metrics",
    "panels": [
      {
        "title": "Active Users",
        "type": "stat",
        "targets": [
          {
            "expr": "campfire_users_active_total",
            "legendFormat": "Active Users"
          }
        ]
      },
      {
        "title": "Messages Sent Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(campfire_messages_sent_total[5m])",
            "legendFormat": "Messages/sec"
          }
        ]
      },
      {
        "title": "Room Activity",
        "type": "table",
        "targets": [
          {
            "expr": "topk(10, rate(campfire_messages_sent_total[1h]) by (room_id))",
            "format": "table"
          }
        ]
      }
    ]
  }
}
```

## Alerting Configuration

### Alertmanager Configuration

Create `monitoring/alertmanager/alertmanager.yml`:

```yaml
global:
  smtp_smarthost: 'smtp.yourdomain.com:587'
  smtp_from: 'alerts@yourdomain.com'
  smtp_auth_username: 'alerts@yourdomain.com'
  smtp_auth_password: 'your-smtp-password'

templates:
  - '/etc/alertmanager/templates/*.tmpl'

route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'default'
  routes:
    # Critical alerts go to on-call
    - match:
        severity: critical
      receiver: 'critical-alerts'
      group_wait: 0s
      repeat_interval: 5m
    
    # Database alerts go to DBA team
    - match:
        component: database
      receiver: 'database-team'
    
    # Performance alerts during business hours
    - match:
        component: performance
      receiver: 'performance-team'
      active_time_intervals:
        - business-hours

inhibit_rules:
  # Inhibit warning alerts when critical alerts are firing
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'cluster', 'service']

time_intervals:
  - name: business-hours
    time_intervals:
      - times:
        - start_time: '09:00'
          end_time: '17:00'
        weekdays: ['monday:friday']

receivers:
  - name: 'default'
    email_configs:
      - to: 'team@yourdomain.com'
        subject: 'Campfire Alert: {{ .GroupLabels.alertname }}'
        body: |
          {{ range .Alerts }}
          Alert: {{ .Annotations.summary }}
          Description: {{ .Annotations.description }}
          Severity: {{ .Labels.severity }}
          {{ end }}

  - name: 'critical-alerts'
    email_configs:
      - to: 'oncall@yourdomain.com'
        subject: '🚨 CRITICAL: {{ .GroupLabels.alertname }}'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
        channel: '#alerts-critical'
        title: 'Critical Campfire Alert'
        text: |
          {{ range .Alerts }}
          *{{ .Annotations.summary }}*
          {{ .Annotations.description }}
          {{ end }}
    webhook_configs:
      - url: 'https://your-pagerduty-integration-url'

  - name: 'database-team'
    email_configs:
      - to: 'dba@yourdomain.com'
        subject: 'Database Alert: {{ .GroupLabels.alertname }}'

  - name: 'performance-team'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
        channel: '#performance'
        title: 'Performance Alert'
```

### Custom Alert Templates

Create `monitoring/alertmanager/templates/campfire.tmpl`:

```go
{{ define "campfire.title" }}
[{{ .Status | toUpper }}{{ if eq .Status "firing" }}:{{ .Alerts.Firing | len }}{{ end }}] 
{{ .GroupLabels.SortedPairs.Values | join " " }} 
{{ if gt (len .CommonLabels) (len .GroupLabels) }}({{ with .CommonLabels.Remove .GroupLabels.Names }}{{ .Values | join " " }}{{ end }}){{ end }}
{{ end }}

{{ define "campfire.slack.text" }}
{{ range .Alerts }}
*Alert:* {{ .Annotations.summary }}
*Description:* {{ .Annotations.description }}
*Severity:* {{ .Labels.severity }}
*Component:* {{ .Labels.component }}
{{ if .Annotations.runbook_url }}*Runbook:* {{ .Annotations.runbook_url }}{{ end }}
{{ end }}
{{ end }}
```

## Log Management

### Structured Logging Configuration

```bash
# Enable structured logging in production
CAMPFIRE_LOG_FORMAT=json
CAMPFIRE_LOG_STRUCTURED=true
CAMPFIRE_LOG_LEVEL=info
CAMPFIRE_LOG_FILE=/app/logs/campfire.log
```

### Log Aggregation with Loki

Create `monitoring/loki/loki.yml`:

```yaml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    address: 127.0.0.1
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
    final_sleep: 0s
  chunk_idle_period: 5m
  chunk_retain_period: 30s

schema_config:
  configs:
    - from: 2020-10-24
      store: boltdb
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 168h

storage_config:
  boltdb:
    directory: /loki/index
  filesystem:
    directory: /loki/chunks

limits_config:
  enforce_metric_name: false
  reject_old_samples: true
  reject_old_samples_max_age: 168h

chunk_store_config:
  max_look_back_period: 0s

table_manager:
  retention_deletes_enabled: false
  retention_period: 0s
```

### Promtail Configuration

Create `monitoring/promtail/promtail.yml`:

```yaml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: campfire
    static_configs:
      - targets:
          - localhost
        labels:
          job: campfire
          __path__: /app/logs/*.log
    
    pipeline_stages:
      # Parse JSON logs
      - json:
          expressions:
            timestamp: timestamp
            level: level
            message: message
            module: module
            request_id: request_id
      
      # Extract labels
      - labels:
          level:
          module:
      
      # Parse timestamp
      - timestamp:
          source: timestamp
          format: RFC3339Nano
      
      # Output formatting
      - output:
          source: message

  - job_name: docker
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
        refresh_interval: 5s
    
    relabel_configs:
      - source_labels: ['__meta_docker_container_name']
        regex: '/campfire'
        target_label: 'container'
      - source_labels: ['__meta_docker_container_log_stream']
        target_label: 'stream'
```

## Performance Monitoring

### Application Performance Monitoring (APM)

```rust
// Example APM integration with tracing
use tracing::{info, warn, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[instrument(skip(app_state))]
pub async fn send_message(
    State(app_state): State<AppState>,
    Json(request): Json<SendMessageRequest>,
) -> Result<Json<Message>, AppError> {
    let span = tracing::Span::current();
    span.record("user_id", &request.user_id.to_string());
    span.record("room_id", &request.room_id.to_string());
    
    let start_time = Instant::now();
    
    // Message processing logic
    let result = process_message(app_state, request).await;
    
    let duration = start_time.elapsed();
    
    match &result {
        Ok(_) => {
            info!(
                duration_ms = duration.as_millis(),
                "Message sent successfully"
            );
        }
        Err(e) => {
            error!(
                duration_ms = duration.as_millis(),
                error = %e,
                "Failed to send message"
            );
        }
    }
    
    result
}
```

### Database Performance Monitoring

```sql
-- Create performance monitoring views
CREATE VIEW message_performance_stats AS
SELECT 
    DATE(created_at) as date,
    COUNT(*) as message_count,
    AVG(LENGTH(content)) as avg_message_length,
    COUNT(DISTINCT creator_id) as active_users,
    COUNT(DISTINCT room_id) as active_rooms
FROM messages 
WHERE created_at >= DATE('now', '-30 days')
GROUP BY DATE(created_at)
ORDER BY date DESC;

-- Query performance analysis
EXPLAIN QUERY PLAN 
SELECT m.*, u.name as creator_name 
FROM messages m 
JOIN users u ON m.creator_id = u.id 
WHERE m.room_id = ? 
ORDER BY m.created_at DESC 
LIMIT 50;
```

## External Monitoring

### Uptime Monitoring

```bash
#!/bin/bash
# scripts/uptime-monitor.sh

ENDPOINTS=(
    "https://chat.yourdomain.com/health"
    "https://chat.yourdomain.com/health/ready"
    "https://api.yourdomain.com/health"
)

WEBHOOK_URL="https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"

for endpoint in "${ENDPOINTS[@]}"; do
    if ! curl -f -s --max-time 10 "$endpoint" > /dev/null; then
        curl -X POST "$WEBHOOK_URL" \
            -H 'Content-type: application/json' \
            --data "{\"text\":\"🚨 Endpoint down: $endpoint\"}"
    fi
done
```

### Third-party Monitoring Integration

#### Datadog Integration

```yaml
# docker-compose.monitoring.yml
services:
  datadog:
    image: datadog/agent:latest
    environment:
      - DD_API_KEY=${DD_API_KEY}
      - DD_SITE=datadoghq.com
      - DD_LOGS_ENABLED=true
      - DD_PROCESS_AGENT_ENABLED=true
      - DD_DOCKER_LABELS_AS_TAGS=true
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - /proc/:/host/proc/:ro
      - /sys/fs/cgroup/:/host/sys/fs/cgroup:ro
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
    labels:
      - "com.datadoghq.ad.logs=[{\"source\": \"campfire\", \"service\": \"campfire\"}]"
```

#### New Relic Integration

```bash
# Environment variables for New Relic
CAMPFIRE_NEWRELIC_LICENSE_KEY=your_license_key
CAMPFIRE_NEWRELIC_APP_NAME=campfire-production
CAMPFIRE_NEWRELIC_ENABLED=true
```

## Troubleshooting

### Common Monitoring Issues

#### Metrics Not Appearing

```bash
# Check if metrics endpoint is accessible
curl http://localhost:3000/metrics

# Verify Prometheus can scrape the target
curl http://prometheus:9090/api/v1/targets

# Check Prometheus logs
docker-compose logs prometheus
```

#### Alerts Not Firing

```bash
# Check alerting rules syntax
promtool check rules monitoring/rules/*.yml

# Verify alert evaluation
curl http://prometheus:9090/api/v1/rules

# Check Alertmanager configuration
amtool config show --alertmanager.url=http://localhost:9093
```

#### Dashboard Not Loading

```bash
# Check Grafana logs
docker-compose logs grafana

# Verify data source connection
curl -u admin:admin http://localhost:3001/api/datasources

# Test Prometheus queries
curl "http://prometheus:9090/api/v1/query?query=up"
```

### Performance Troubleshooting

#### High Memory Usage

```bash
# Check memory metrics
curl -s http://localhost:3000/metrics | grep memory

# Analyze memory allocation
docker stats campfire

# Check for memory leaks
valgrind --tool=massif ./campfire-on-rust
```

#### Slow Response Times

```bash
# Check response time metrics
curl -s http://localhost:3000/metrics | grep duration

# Analyze slow queries
sqlite3 campfire.db "PRAGMA compile_options;"

# Profile application performance
perf record -g ./campfire-on-rust
perf report
```

This comprehensive monitoring and alerting guide ensures complete observability for Campfire deployments, enabling proactive issue detection and resolution.