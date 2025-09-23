# Deployment Guide

## Deployment Overview

This guide covers deployment strategies for the Campfire Rust rewrite, from local development to production environments.

## Deployment Options

```mermaid
graph TD
    subgraph "Deployment Strategies"
        direction TB
        LOCAL[Local Development<br/>cargo run]
        DOCKER[Docker Container<br/>Single Binary]
        CLOUD[Cloud Deployment<br/>AWS/GCP/Azure]
        BARE[Bare Metal<br/>systemd Service]
    end
    
    subgraph "Environment Configurations"
        direction TB
        DEV[Development<br/>SQLite + Debug Logs]
        STAGING[Staging<br/>Production Config + Test Data]
        PROD[Production<br/>Optimized + Monitoring]
    end
    
    LOCAL --> DEV
    DOCKER --> DEV
    DOCKER --> STAGING
    DOCKER --> PROD
    CLOUD --> STAGING
    CLOUD --> PROD
    BARE --> PROD
    
    classDef deployment fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef environment fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    
    class LOCAL,DOCKER,CLOUD,BARE deployment
    class DEV,STAGING,PROD environment
```

## Local Development Setup

### Quick Start

```bash
# Clone repository
git clone <repository-url>
cd campfire-rust-rewrite

# Build and run
cargo build
cargo run

# Access application
open http://localhost:3000
```

### Development Workflow

```mermaid
graph TD
    subgraph "Development Cycle"
        direction TB
        EDIT[Edit Code<br/>src/ or templates/]
        BUILD[Cargo Build<br/>Compile Check]
        TEST[Run Tests<br/>cargo test]
        RUN[Local Server<br/>cargo run]
        VERIFY[Manual Testing<br/>Browser + API]
    end
    
    subgraph "Hot Reload Setup"
        direction TB
        WATCH[cargo-watch<br/>Auto-rebuild]
        RELOAD[Browser Refresh<br/>Manual or Auto]
    end
    
    EDIT --> BUILD
    BUILD --> TEST
    TEST --> RUN
    RUN --> VERIFY
    VERIFY --> EDIT
    
    EDIT --> WATCH
    WATCH --> RELOAD
    
    classDef dev fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef tools fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    
    class EDIT,BUILD,TEST,RUN,VERIFY dev
    class WATCH,RELOAD tools
```

### Development Dependencies

```bash
# Install development tools
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-outdated

# Run with hot reload
cargo watch -x run

# Security audit
cargo audit

# Check for outdated dependencies
cargo outdated
```

## Docker Deployment

### Single Container Deployment

```mermaid
graph TD
    subgraph "Docker Build Process"
        direction TB
        DOCKERFILE[Dockerfile<br/>Multi-stage Build]
        BUILD_STAGE[Build Stage<br/>Rust + Dependencies]
        RUNTIME_STAGE[Runtime Stage<br/>Alpine Linux]
        FINAL_IMAGE[Final Image<br/>~20MB]
    end
    
    subgraph "Container Runtime"
        direction TB
        CONTAINER[Docker Container<br/>Single Process]
        VOLUMES[Mounted Volumes<br/>Database + Logs]
        NETWORK[Network Ports<br/>3000:3000]
        ENV[Environment Variables<br/>Configuration]
    end
    
    DOCKERFILE --> BUILD_STAGE
    BUILD_STAGE --> RUNTIME_STAGE
    RUNTIME_STAGE --> FINAL_IMAGE
    
    FINAL_IMAGE --> CONTAINER
    CONTAINER --> VOLUMES
    CONTAINER --> NETWORK
    CONTAINER --> ENV
    
    classDef build fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef runtime fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    
    class DOCKERFILE,BUILD_STAGE,RUNTIME_STAGE,FINAL_IMAGE build
    class CONTAINER,VOLUMES,NETWORK,ENV runtime
```

### Docker Commands

```bash
# Build image
docker build -t campfire-rust .

# Run container
docker run -d \
  --name campfire \
  -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  -e RUST_LOG=info \
  campfire-rust

# View logs
docker logs -f campfire

# Stop container
docker stop campfire
```

### Docker Compose Setup

```yaml
# docker-compose.yml
version: '3.8'

services:
  campfire:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
    environment:
      - RUST_LOG=info
      - DATABASE_URL=sqlite:/app/data/campfire.db
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Optional: Reverse proxy
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - campfire
    restart: unless-stopped
```

## Production Deployment

### Cloud Deployment Architecture

```mermaid
graph TD
    subgraph "Load Balancer"
        direction TB
        LB[Load Balancer<br/>SSL Termination]
        HEALTH_CHECK[Health Checks<br/>/health endpoint]
    end
    
    subgraph "Application Tier"
        direction TB
        APP1[Campfire Instance 1<br/>Docker Container]
        APP2[Campfire Instance 2<br/>Docker Container]
        APP3[Campfire Instance N<br/>Docker Container]
    end
    
    subgraph "Data Tier"
        direction TB
        DB[SQLite Database<br/>Shared Volume]
        BACKUP[Backup Storage<br/>S3/GCS/Azure Blob]
        LOGS[Log Aggregation<br/>CloudWatch/Stackdriver]
    end
    
    subgraph "Monitoring"
        direction TB
        METRICS[Metrics Collection<br/>Prometheus/CloudWatch]
        ALERTS[Alerting<br/>PagerDuty/Slack]
        DASHBOARD[Dashboards<br/>Grafana/Cloud Console]
    end
    
    LB --> HEALTH_CHECK
    LB --> APP1
    LB --> APP2
    LB --> APP3
    
    APP1 --> DB
    APP2 --> DB
    APP3 --> DB
    
    DB --> BACKUP
    APP1 --> LOGS
    APP2 --> LOGS
    APP3 --> LOGS
    
    APP1 --> METRICS
    APP2 --> METRICS
    APP3 --> METRICS
    METRICS --> ALERTS
    METRICS --> DASHBOARD
    
    classDef lb fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef app fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef data fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef monitoring fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class LB,HEALTH_CHECK lb
    class APP1,APP2,APP3 app
    class DB,BACKUP,LOGS data
    class METRICS,ALERTS,DASHBOARD monitoring
```

### Environment Configuration

```bash
# Production environment variables
export RUST_LOG=info
export DATABASE_URL=sqlite:/app/data/campfire.db
export BIND_ADDRESS=0.0.0.0:3000
export SESSION_SECRET=<secure-random-key>
export VAPID_PRIVATE_KEY=<vapid-private-key>
export VAPID_PUBLIC_KEY=<vapid-public-key>
export WEBHOOK_SECRET=<webhook-secret>
export MAX_CONNECTIONS=1000
export RATE_LIMIT_REQUESTS=100
export RATE_LIMIT_WINDOW=60
```

### Systemd Service (Bare Metal)

```ini
# /etc/systemd/system/campfire.service
[Unit]
Description=Campfire Rust Chat Application
After=network.target

[Service]
Type=simple
User=campfire
Group=campfire
WorkingDirectory=/opt/campfire
ExecStart=/opt/campfire/campfire-rust
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=DATABASE_URL=sqlite:/opt/campfire/data/campfire.db
Environment=BIND_ADDRESS=127.0.0.1:3000

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/campfire/data /opt/campfire/logs

[Install]
WantedBy=multi-user.target
```

### Deployment Process

```mermaid
graph TD
    subgraph "CI/CD Pipeline"
        direction TB
        COMMIT[Git Commit<br/>Push to Main]
        BUILD[Build Binary<br/>cargo build --release]
        TEST[Run Tests<br/>cargo test]
        PACKAGE[Package Image<br/>Docker Build]
        DEPLOY[Deploy to Production<br/>Rolling Update]
    end
    
    subgraph "Health Checks"
        direction TB
        PRE_DEPLOY[Pre-deployment<br/>Health Check]
        DEPLOY_CHECK[Deployment<br/>Readiness Check]
        POST_DEPLOY[Post-deployment<br/>Smoke Tests]
        ROLLBACK{Rollback?}
    end
    
    COMMIT --> BUILD
    BUILD --> TEST
    TEST --> PACKAGE
    PACKAGE --> DEPLOY
    
    DEPLOY --> PRE_DEPLOY
    PRE_DEPLOY --> DEPLOY_CHECK
    DEPLOY_CHECK --> POST_DEPLOY
    POST_DEPLOY --> ROLLBACK
    
    classDef pipeline fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef health fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef decision fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class COMMIT,BUILD,TEST,PACKAGE,DEPLOY pipeline
    class PRE_DEPLOY,DEPLOY_CHECK,POST_DEPLOY health
    class ROLLBACK decision
```

## Database Management

### SQLite in Production

```mermaid
graph TD
    subgraph "Database Setup"
        direction TB
        INIT[Initialize Database<br/>Run Migrations]
        WAL[Enable WAL Mode<br/>Concurrent Reads]
        BACKUP_CONFIG[Configure Backups<br/>Automated Schedule]
        MONITOR[Monitor Size<br/>Growth Tracking]
    end
    
    subgraph "Backup Strategy"
        direction TB
        CONTINUOUS[Continuous Backup<br/>WAL File Copying]
        SNAPSHOT[Periodic Snapshots<br/>Full Database Copy]
        RETENTION[Retention Policy<br/>30 days + Archives]
        RESTORE[Restore Procedures<br/>Point-in-time Recovery]
    end
    
    subgraph "Maintenance"
        direction TB
        VACUUM[VACUUM Operations<br/>Reclaim Space]
        ANALYZE[ANALYZE Statistics<br/>Query Optimization]
        INTEGRITY[Integrity Checks<br/>PRAGMA integrity_check]
        REINDEX[Reindex FTS<br/>Search Performance]
    end
    
    INIT --> WAL
    WAL --> BACKUP_CONFIG
    BACKUP_CONFIG --> MONITOR
    
    CONTINUOUS --> SNAPSHOT
    SNAPSHOT --> RETENTION
    RETENTION --> RESTORE
    
    VACUUM --> ANALYZE
    ANALYZE --> INTEGRITY
    INTEGRITY --> REINDEX
    
    classDef setup fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef backup fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef maintenance fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class INIT,WAL,BACKUP_CONFIG,MONITOR setup
    class CONTINUOUS,SNAPSHOT,RETENTION,RESTORE backup
    class VACUUM,ANALYZE,INTEGRITY,REINDEX maintenance
```

### Migration Management

```bash
# Run database migrations
cargo run --bin migrate

# Create new migration
cargo run --bin migrate -- create add_user_preferences

# Check migration status
cargo run --bin migrate -- status

# Rollback migration (if supported)
cargo run --bin migrate -- rollback
```

## Monitoring and Observability

### Monitoring Stack

```mermaid
graph TD
    subgraph "Application Metrics"
        direction TB
        APP_METRICS[Application Metrics<br/>/metrics endpoint]
        CUSTOM[Custom Metrics<br/>Business Logic]
        HEALTH[Health Checks<br/>/health + /ready]
    end
    
    subgraph "Infrastructure Metrics"
        direction TB
        SYSTEM[System Metrics<br/>CPU, Memory, Disk]
        NETWORK[Network Metrics<br/>Connections, Bandwidth]
        CONTAINER[Container Metrics<br/>Docker Stats]
    end
    
    subgraph "Log Aggregation"
        direction TB
        STRUCTURED[Structured Logs<br/>JSON Format]
        COLLECTION[Log Collection<br/>Fluentd/Filebeat]
        STORAGE[Log Storage<br/>Elasticsearch/CloudWatch]
        SEARCH[Log Search<br/>Kibana/Cloud Console]
    end
    
    subgraph "Alerting"
        direction TB
        RULES[Alert Rules<br/>Thresholds + Conditions]
        CHANNELS[Notification Channels<br/>Email, Slack, PagerDuty]
        ESCALATION[Escalation Policies<br/>On-call Rotation]
    end
    
    APP_METRICS --> RULES
    CUSTOM --> RULES
    HEALTH --> RULES
    
    SYSTEM --> RULES
    NETWORK --> RULES
    CONTAINER --> RULES
    
    STRUCTURED --> COLLECTION
    COLLECTION --> STORAGE
    STORAGE --> SEARCH
    
    RULES --> CHANNELS
    CHANNELS --> ESCALATION
    
    classDef app fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef infra fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef logs fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef alerts fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class APP_METRICS,CUSTOM,HEALTH app
    class SYSTEM,NETWORK,CONTAINER infra
    class STRUCTURED,COLLECTION,STORAGE,SEARCH logs
    class RULES,CHANNELS,ESCALATION alerts
```

### Key Metrics to Monitor

```bash
# Application metrics
campfire_http_requests_total
campfire_websocket_connections_active
campfire_messages_sent_total
campfire_database_query_duration_seconds
campfire_memory_usage_bytes

# System metrics
cpu_usage_percent
memory_usage_percent
disk_usage_percent
network_connections_active

# Business metrics
active_users_count
messages_per_minute
rooms_active_count
search_queries_per_minute
```

## Security Considerations

### Production Security Checklist

```mermaid
graph TD
    subgraph "Network Security"
        direction TB
        TLS[TLS/SSL Encryption<br/>HTTPS + WSS]
        FIREWALL[Firewall Rules<br/>Port Restrictions]
        VPN[VPN Access<br/>Admin Interfaces]
    end
    
    subgraph "Application Security"
        direction TB
        AUTH[Strong Authentication<br/>Session Management]
        AUTHZ[Authorization<br/>Role-based Access]
        INPUT[Input Validation<br/>XSS Prevention]
        RATE[Rate Limiting<br/>DDoS Protection]
    end
    
    subgraph "Data Security"
        direction TB
        ENCRYPT[Data Encryption<br/>At Rest + Transit]
        BACKUP_SEC[Secure Backups<br/>Encrypted Storage]
        AUDIT[Audit Logging<br/>Access Tracking]
    end
    
    subgraph "Infrastructure Security"
        direction TB
        UPDATES[Security Updates<br/>OS + Dependencies]
        MONITORING[Security Monitoring<br/>Intrusion Detection]
        SECRETS[Secret Management<br/>Environment Variables]
    end
    
    TLS --> AUTH
    FIREWALL --> AUTHZ
    VPN --> INPUT
    
    AUTH --> ENCRYPT
    AUTHZ --> BACKUP_SEC
    INPUT --> AUDIT
    RATE --> AUDIT
    
    ENCRYPT --> UPDATES
    BACKUP_SEC --> MONITORING
    AUDIT --> SECRETS
    
    classDef network fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef app fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef data fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef infra fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class TLS,FIREWALL,VPN network
    class AUTH,AUTHZ,INPUT,RATE app
    class ENCRYPT,BACKUP_SEC,AUDIT data
    class UPDATES,MONITORING,SECRETS infra
```

## Scaling Considerations

### Horizontal Scaling Strategy

```mermaid
graph TD
    subgraph "Current Architecture (Single Instance)"
        direction TB
        SINGLE[Single Binary<br/>All Components]
        SQLITE[SQLite Database<br/>Local File]
        MEMORY[In-Memory Cache<br/>Sessions + Presence]
    end
    
    subgraph "Scaled Architecture (Multiple Instances)"
        direction TB
        LB_SCALE[Load Balancer<br/>Session Affinity]
        APP_SCALE1[App Instance 1<br/>Stateless]
        APP_SCALE2[App Instance 2<br/>Stateless]
        SHARED_DB[Shared Database<br/>PostgreSQL/MySQL]
        REDIS[Redis Cache<br/>Shared Sessions]
    end
    
    subgraph "WebSocket Scaling"
        direction TB
        WS_LB[WebSocket Load Balancer<br/>Sticky Sessions]
        PUBSUB[Redis Pub/Sub<br/>Cross-instance Messaging]
        PRESENCE_SYNC[Presence Synchronization<br/>Distributed State]
    end
    
    SINGLE --> LB_SCALE
    SQLITE --> SHARED_DB
    MEMORY --> REDIS
    
    LB_SCALE --> APP_SCALE1
    LB_SCALE --> APP_SCALE2
    APP_SCALE1 --> SHARED_DB
    APP_SCALE2 --> SHARED_DB
    APP_SCALE1 --> REDIS
    APP_SCALE2 --> REDIS
    
    WS_LB --> PUBSUB
    PUBSUB --> PRESENCE_SYNC
    
    classDef current fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef scaled fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef websocket fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class SINGLE,SQLITE,MEMORY current
    class LB_SCALE,APP_SCALE1,APP_SCALE2,SHARED_DB,REDIS scaled
    class WS_LB,PUBSUB,PRESENCE_SYNC websocket
```

## Troubleshooting

### Common Issues and Solutions

```mermaid
graph TD
    subgraph "Performance Issues"
        direction TB
        SLOW[Slow Response Times<br/>Check Database Queries]
        MEMORY[High Memory Usage<br/>Check Connection Leaks]
        CPU[High CPU Usage<br/>Check Background Tasks]
    end
    
    subgraph "Connectivity Issues"
        direction TB
        WS_FAIL[WebSocket Failures<br/>Check Authentication]
        DB_CONN[Database Connection<br/>Check File Permissions]
        NETWORK[Network Issues<br/>Check Firewall Rules]
    end
    
    subgraph "Application Issues"
        direction TB
        CRASH[Application Crashes<br/>Check Error Logs]
        STARTUP[Startup Failures<br/>Check Configuration]
        MIGRATION[Migration Issues<br/>Check Database State]
    end
    
    SLOW --> MEMORY
    MEMORY --> CPU
    
    WS_FAIL --> DB_CONN
    DB_CONN --> NETWORK
    
    CRASH --> STARTUP
    STARTUP --> MIGRATION
    
    classDef performance fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef connectivity fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef application fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class SLOW,MEMORY,CPU performance
    class WS_FAIL,DB_CONN,NETWORK connectivity
    class CRASH,STARTUP,MIGRATION application
```

### Diagnostic Commands

```bash
# Check application health
curl http://localhost:3000/health

# View application logs
docker logs -f campfire

# Check database integrity
sqlite3 campfire.db "PRAGMA integrity_check;"

# Monitor resource usage
docker stats campfire

# Check WebSocket connections
ss -tuln | grep 3000

# View system metrics
curl http://localhost:3000/metrics
```

This deployment guide provides comprehensive coverage of deployment strategies, from local development to production environments, with proper monitoring and security considerations.