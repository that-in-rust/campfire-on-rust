# Production Deployment Guide

## IMPORTANT FOR VISUALS AND DIAGRAMS

ALL DIAGRAMS WILL BE IN MERMAID ONLY TO ENSURE EASE WITH GITHUB - DO NOT SKIP THAT

## Overview

This comprehensive guide covers deploying Campfire Rust to production environments, from single-server deployments to scalable cloud architectures. The guide follows Basecamp's proven deployment patterns adapted for modern containerized environments.

```mermaid
graph TD
    subgraph "Deployment Journey"
        START[Start Deployment<br/>5-Minute Quick Start]
        DOCKER[Docker Setup<br/>Container + Volumes]
        PROXY[Reverse Proxy<br/>nginx/Caddy + SSL]
        MONITOR[Monitoring<br/>Health + Metrics]
        PRODUCTION[Production Ready<br/>Secure + Scalable]
    end
    
    START --> DOCKER
    DOCKER --> PROXY
    PROXY --> MONITOR
    MONITOR --> PRODUCTION
    
    classDef journey fill:#e8f5e8,stroke:#2e7d32,stroke-width:3px
    class START,DOCKER,PROXY,MONITOR,PRODUCTION journey
```

**Deployment Philosophy:**
- **Single Binary Simplicity**: Leverage Rust's static compilation for zero-dependency deployments
- **Container-First**: Docker containers for consistent environments across development and production
- **Infrastructure as Code**: Reproducible deployments with version-controlled configurations
- **Monitoring-First**: Built-in observability from day one
- **Security by Default**: Secure configurations with minimal attack surface

## Quick Start (5-Minute Deployment)

```bash
# 1. Clone and configure
git clone https://github.com/that-in-rust/campfire-on-rust.git
cd campfire-on-rust
cp .env.example .env.production

# 2. Deploy with Docker Compose
./scripts/deploy.sh deploy

# 3. Access application
open http://localhost:3000
```

## Deployment Architecture Overview

```mermaid
graph TD
    subgraph "Internet"
        USERS[Users<br/>Web + Mobile]
        BOTS[Bots<br/>API Clients]
    end
    
    subgraph "Load Balancer / Reverse Proxy"
        LB[Load Balancer<br/>nginx/Caddy/Traefik]
        SSL[SSL Termination<br/>Let's Encrypt]
        RATE[Rate Limiting<br/>DDoS Protection]
    end
    
    subgraph "Application Tier"
        APP1[Campfire Instance 1<br/>Docker Container]
        APP2[Campfire Instance 2<br/>Docker Container]
        APP3[Campfire Instance N<br/>Docker Container]
    end
    
    subgraph "Data Tier"
        DB[(SQLite Database<br/>Shared Volume)]
        BACKUP[Automated Backups<br/>S3/GCS/Local]
    end
    
    subgraph "Monitoring & Logging"
        METRICS[Prometheus<br/>Metrics Collection]
        LOGS[Structured Logs<br/>JSON Format]
        ALERTS[Alerting<br/>PagerDuty/Slack]
        DASHBOARD[Grafana<br/>Dashboards]
    end
    
    USERS --> LB
    BOTS --> LB
    
    LB --> SSL
    SSL --> RATE
    RATE --> APP1
    RATE --> APP2
    RATE --> APP3
    
    APP1 --> DB
    APP2 --> DB
    APP3 --> DB
    
    DB --> BACKUP
    
    APP1 --> METRICS
    APP2 --> METRICS
    APP3 --> METRICS
    
    APP1 --> LOGS
    APP2 --> LOGS
    APP3 --> LOGS
    
    METRICS --> ALERTS
    METRICS --> DASHBOARD
    
    classDef internet fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef lb fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    classDef app fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef data fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef monitoring fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class USERS,BOTS internet
    class LB,SSL,RATE lb
    class APP1,APP2,APP3 app
    class DB,BACKUP data
    class METRICS,LOGS,ALERTS,DASHBOARD monitoring
```

## Docker Deployment

### Single Container Deployment

The simplest production deployment uses a single Docker container with persistent volumes:

```mermaid
graph TD
    subgraph "Docker Host"
        subgraph "Campfire Container"
            BINARY[Rust Binary<br/>campfire-on-rust]
            CONFIG[Configuration<br/>Environment Variables]
            HEALTH[Health Checks<br/>/health endpoint]
        end
        
        subgraph "Persistent Volumes"
            DATA_VOL[Data Volume<br/>SQLite Database]
            LOGS_VOL[Logs Volume<br/>Application Logs]
            BACKUP_VOL[Backup Volume<br/>Database Backups]
        end
        
        subgraph "Network"
            PORT[Port 3000<br/>HTTP + WebSocket]
            INTERNAL[Internal Network<br/>Container Communication]
        end
    end
    
    subgraph "External Services"
        REVERSE_PROXY[Reverse Proxy<br/>nginx/Caddy]
        MONITORING[Monitoring<br/>Prometheus/Grafana]
        BACKUP_STORAGE[Backup Storage<br/>S3/GCS/Local]
    end
    
    BINARY --> CONFIG
    BINARY --> HEALTH
    BINARY --> DATA_VOL
    BINARY --> LOGS_VOL
    BINARY --> BACKUP_VOL
    
    PORT --> REVERSE_PROXY
    INTERNAL --> MONITORING
    BACKUP_VOL --> BACKUP_STORAGE
    
    classDef container fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef volume fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef network fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef external fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class BINARY,CONFIG,HEALTH container
    class DATA_VOL,LOGS_VOL,BACKUP_VOL volume
    class PORT,INTERNAL network
    class REVERSE_PROXY,MONITORING,BACKUP_STORAGE external
```

### Docker Compose Configuration

Create a production-ready `docker-compose.yml`:

```yaml
version: '3.8'

services:
  campfire:
    build:
      context: .
      dockerfile: Dockerfile
    image: campfire-rust:latest
    container_name: campfire
    restart: unless-stopped
    
    # Port mapping
    ports:
      - "3000:3000"
    
    # Environment configuration
    env_file:
      - .env.production
    
    # Volume mounts for persistent data
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
      - ./backups:/app/backups
    
    # Health check
    healthcheck:
      test: ["CMD", "/app/healthcheck.sh"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
    
    # Resource limits
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '1.0'
        reservations:
          memory: 256M
          cpus: '0.5'
    
    # Security settings
    security_opt:
      - no-new-privileges:true
    read_only: false
    tmpfs:
      - /tmp:noexec,nosuid,size=100m
    
    # Logging configuration
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

  # Optional: Reverse proxy with automatic SSL
  caddy:
    image: caddy:2-alpine
    container_name: campfire-caddy
    restart: unless-stopped
    profiles:
      - proxy
    
    ports:
      - "80:80"
      - "443:443"
    
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data
      - caddy_config:/config
    
    depends_on:
      - campfire

volumes:
  caddy_data:
  caddy_config:
```

### Environment Configuration

Create `.env.production` with production settings:

```bash
# Server Configuration
CAMPFIRE_HOST=0.0.0.0
CAMPFIRE_PORT=3000
CAMPFIRE_REQUEST_TIMEOUT=30
CAMPFIRE_MAX_REQUEST_SIZE=16777216
CAMPFIRE_SHUTDOWN_TIMEOUT=30
CAMPFIRE_WORKER_THREADS=4

# Database Configuration
CAMPFIRE_DATABASE_URL=/app/data/campfire.db
CAMPFIRE_DB_MAX_CONNECTIONS=20
CAMPFIRE_DB_TIMEOUT=30
CAMPFIRE_DB_WAL_MODE=true
CAMPFIRE_BACKUP_DIR=/app/backups

# Logging Configuration
CAMPFIRE_LOG_LEVEL=info
CAMPFIRE_LOG_FORMAT=json
CAMPFIRE_LOG_FILE=/app/logs/campfire.log
CAMPFIRE_LOG_STRUCTURED=true
CAMPFIRE_TRACE_REQUESTS=false

# Security Configuration
CAMPFIRE_CORS_ORIGINS=https://your-domain.com
CAMPFIRE_RATE_LIMIT_RPM=120
CAMPFIRE_SESSION_TOKEN_LENGTH=32
CAMPFIRE_SESSION_EXPIRY_HOURS=24
CAMPFIRE_FORCE_HTTPS=true
CAMPFIRE_TRUST_PROXY=true

# Push Notifications
CAMPFIRE_PUSH_ENABLED=true
CAMPFIRE_VAPID_PRIVATE_KEY=<your-vapid-private-key>
CAMPFIRE_VAPID_PUBLIC_KEY=<your-vapid-public-key>
CAMPFIRE_VAPID_SUBJECT=mailto:admin@your-domain.com

# Metrics Configuration
CAMPFIRE_METRICS_ENABLED=true
CAMPFIRE_METRICS_DETAILED=true

# Rust Configuration
RUST_LOG=campfire_on_rust=info,tower_http=info
```

## Reverse Proxy Setup

### nginx Configuration

Create `/etc/nginx/sites-available/campfire`:

```nginx
# Campfire Rust Production Configuration
upstream campfire_backend {
    server 127.0.0.1:3000;
    # Add more servers for load balancing
    # server 127.0.0.1:3001;
    # server 127.0.0.1:3002;
}

# Rate limiting
limit_req_zone $binary_remote_addr zone=campfire_api:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=campfire_websocket:10m rate=5r/s;

# HTTP to HTTPS redirect
server {
    listen 80;
    server_name your-domain.com www.your-domain.com;
    return 301 https://$server_name$request_uri;
}

# Main HTTPS server
server {
    listen 443 ssl http2;
    server_name your-domain.com www.your-domain.com;
    
    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:50m;
    ssl_session_tickets off;
    
    # Modern SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    
    # HSTS
    add_header Strict-Transport-Security "max-age=63072000" always;
    
    # Security headers
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' wss:; font-src 'self'; object-src 'none'; media-src 'self'; frame-src 'none';" always;
    
    # Logging
    access_log /var/log/nginx/campfire_access.log;
    error_log /var/log/nginx/campfire_error.log;
    
    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json;
    
    # Main application proxy
    location / {
        # Rate limiting
        limit_req zone=campfire_api burst=20 nodelay;
        
        proxy_pass http://campfire_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # Buffer settings
        proxy_buffering on;
        proxy_buffer_size 128k;
        proxy_buffers 4 256k;
        proxy_busy_buffers_size 256k;
    }
    
    # WebSocket proxy with special handling
    location /ws {
        # Rate limiting for WebSocket connections
        limit_req zone=campfire_websocket burst=10 nodelay;
        
        proxy_pass http://campfire_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket specific timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 3600s;  # 1 hour for long-lived connections
        
        # Disable buffering for WebSocket
        proxy_buffering off;
    }
    
    # Health check endpoint (no rate limiting)
    location /health {
        proxy_pass http://campfire_backend;
        proxy_set_header Host $host;
        access_log off;
    }
    
    # Metrics endpoint (restrict access)
    location /metrics {
        allow 127.0.0.1;
        allow 10.0.0.0/8;
        allow 172.16.0.0/12;
        allow 192.168.0.0/16;
        deny all;
        
        proxy_pass http://campfire_backend;
        proxy_set_header Host $host;
    }
    
    # Static assets caching (if serving static files)
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
        add_header X-Content-Type-Options nosniff;
    }
}
```

### Caddy Configuration (Alternative)

Create `Caddyfile` for automatic HTTPS:

```caddyfile
# Campfire Rust Production Configuration
your-domain.com {
    # Automatic HTTPS with Let's Encrypt
    
    # Security headers
    header {
        # Enable HSTS
        Strict-Transport-Security "max-age=63072000; includeSubDomains; preload"
        
        # Prevent clickjacking
        X-Frame-Options "DENY"
        
        # Prevent MIME type sniffing
        X-Content-Type-Options "nosniff"
        
        # XSS protection
        X-XSS-Protection "1; mode=block"
        
        # Referrer policy
        Referrer-Policy "strict-origin-when-cross-origin"
        
        # Content Security Policy
        Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' wss:; font-src 'self'; object-src 'none'; media-src 'self'; frame-src 'none';"
    }
    
    # Rate limiting
    rate_limit {
        zone dynamic {
            key {remote_host}
            events 100
            window 1m
        }
    }
    
    # Gzip compression
    encode gzip
    
    # Main application proxy
    reverse_proxy campfire:3000 {
        # Health check
        health_uri /health
        health_interval 30s
        health_timeout 10s
        
        # Load balancing (if multiple instances)
        # lb_policy round_robin
    }
    
    # Logging
    log {
        output file /var/log/caddy/campfire.log {
            roll_size 100mb
            roll_keep 5
            roll_keep_for 720h
        }
        format json
    }
}

# Metrics endpoint (internal access only)
metrics.your-domain.com {
    # Restrict access to internal networks
    @internal {
        remote_ip 10.0.0.0/8 172.16.0.0/12 192.168.0.0/16 127.0.0.1/8
    }
    
    handle @internal {
        reverse_proxy campfire:3000
    }
    
    respond 403
}
```

## SSL Configuration

### Let's Encrypt with Certbot

```bash
# Install Certbot
sudo apt-get update
sudo apt-get install certbot python3-certbot-nginx

# Obtain SSL certificate
sudo certbot --nginx -d your-domain.com -d www.your-domain.com

# Test automatic renewal
sudo certbot renew --dry-run

# Set up automatic renewal (crontab)
echo "0 12 * * * /usr/bin/certbot renew --quiet" | sudo crontab -
```

### SSL Security Best Practices

```mermaid
graph TD
    subgraph "SSL/TLS Security Layers"
        CERT[SSL Certificate<br/>Let's Encrypt/Commercial]
        PROTOCOL[Protocol Version<br/>TLS 1.2 + TLS 1.3]
        CIPHER[Cipher Suites<br/>Modern + Secure]
        HSTS[HSTS Headers<br/>Force HTTPS]
    end
    
    subgraph "Certificate Management"
        AUTO_RENEW[Automatic Renewal<br/>Certbot/ACME]
        MONITORING[Certificate Monitoring<br/>Expiry Alerts]
        BACKUP[Certificate Backup<br/>Secure Storage]
    end
    
    subgraph "Security Headers"
        CSP[Content Security Policy<br/>XSS Prevention]
        FRAME[X-Frame-Options<br/>Clickjacking Protection]
        CONTENT_TYPE[X-Content-Type-Options<br/>MIME Sniffing Prevention]
        REFERRER[Referrer-Policy<br/>Information Leakage Prevention]
    end
    
    CERT --> PROTOCOL
    PROTOCOL --> CIPHER
    CIPHER --> HSTS
    
    CERT --> AUTO_RENEW
    AUTO_RENEW --> MONITORING
    MONITORING --> BACKUP
    
    HSTS --> CSP
    CSP --> FRAME
    FRAME --> CONTENT_TYPE
    CONTENT_TYPE --> REFERRER
    
    classDef ssl fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef management fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef headers fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    
    class CERT,PROTOCOL,CIPHER,HSTS ssl
    class AUTO_RENEW,MONITORING,BACKUP management
    class CSP,FRAME,CONTENT_TYPE,REFERRER headers
```

## Monitoring and Health Checks

### Health Check Endpoints

Campfire provides multiple health check endpoints:

```bash
# Basic health check
curl http://localhost:3000/health
# Response: {"status": "healthy", "timestamp": "2024-01-01T12:00:00Z"}

# Detailed readiness check
curl http://localhost:3000/health/ready
# Response: {"status": "ready", "database": "connected", "services": "operational"}

# Liveness check
curl http://localhost:3000/health/live
# Response: {"status": "alive", "uptime": "3600s", "memory": "45MB"}
```

### Prometheus Metrics

Configure Prometheus to scrape metrics:

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'campfire'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
    scrape_interval: 10s
    scrape_timeout: 5s
    
    # Relabel metrics for better organization
    metric_relabel_configs:
      - source_labels: [__name__]
        regex: 'campfire_(.*)'
        target_label: __name__
        replacement: 'campfire_${1}'
```

### Key Metrics to Monitor

```mermaid
graph TD
    subgraph "Application Metrics"
        HTTP_REQ[HTTP Requests<br/>Rate, Duration, Status]
        WS_CONN[WebSocket Connections<br/>Active, Total, Errors]
        MSG_RATE[Message Rate<br/>Sent, Received, Queued]
        USER_ACTIVE[Active Users<br/>Connected, Authenticated]
    end
    
    subgraph "System Metrics"
        CPU[CPU Usage<br/>Percentage, Load Average]
        MEMORY[Memory Usage<br/>RSS, Heap, Available]
        DISK[Disk Usage<br/>Space, I/O, Database Size]
        NETWORK[Network<br/>Connections, Bandwidth]
    end
    
    subgraph "Database Metrics"
        DB_CONN[Database Connections<br/>Active, Pool Size]
        DB_QUERY[Query Performance<br/>Duration, Errors]
        DB_SIZE[Database Size<br/>Growth Rate, Vacuum Stats]
        DB_BACKUP[Backup Status<br/>Last Backup, Success Rate]
    end
    
    subgraph "Business Metrics"
        ROOMS[Active Rooms<br/>Count, Messages per Room]
        SEARCH[Search Queries<br/>Rate, Performance]
        PUSH[Push Notifications<br/>Sent, Delivery Rate]
        BOTS[Bot Activity<br/>API Calls, Webhooks]
    end
    
    HTTP_REQ --> CPU
    WS_CONN --> MEMORY
    MSG_RATE --> DISK
    USER_ACTIVE --> NETWORK
    
    DB_CONN --> DB_QUERY
    DB_QUERY --> DB_SIZE
    DB_SIZE --> DB_BACKUP
    
    ROOMS --> SEARCH
    SEARCH --> PUSH
    PUSH --> BOTS
    
    classDef app fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef system fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef database fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef business fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class HTTP_REQ,WS_CONN,MSG_RATE,USER_ACTIVE app
    class CPU,MEMORY,DISK,NETWORK system
    class DB_CONN,DB_QUERY,DB_SIZE,DB_BACKUP database
    class ROOMS,SEARCH,PUSH,BOTS business
```

### Grafana Dashboard Configuration

Create `grafana-dashboard.json`:

```json
{
  "dashboard": {
    "title": "Campfire Rust Production Dashboard",
    "panels": [
      {
        "title": "HTTP Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(campfire_http_requests_total[5m])",
            "legendFormat": "{{method}} {{status}}"
          }
        ]
      },
      {
        "title": "WebSocket Connections",
        "type": "singlestat",
        "targets": [
          {
            "expr": "campfire_websocket_connections_active",
            "legendFormat": "Active Connections"
          }
        ]
      },
      {
        "title": "Database Query Duration",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(campfire_database_query_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      }
    ]
  }
}
```

## Backup and Restore Procedures

### Automated Backup Strategy

```mermaid
graph TD
    subgraph "Backup Schedule"
        CONTINUOUS[Continuous Backup<br/>WAL File Streaming]
        HOURLY[Hourly Snapshots<br/>Incremental Changes]
        DAILY[Daily Full Backup<br/>Complete Database]
        WEEKLY[Weekly Archive<br/>Long-term Storage]
    end
    
    subgraph "Backup Storage"
        LOCAL[Local Storage<br/>Fast Recovery]
        REMOTE[Remote Storage<br/>S3/GCS/Azure]
        ENCRYPTED[Encryption<br/>AES-256]
        COMPRESSED[Compression<br/>gzip/lz4]
    end
    
    subgraph "Backup Verification"
        INTEGRITY[Integrity Check<br/>SQLite PRAGMA]
        RESTORE_TEST[Restore Test<br/>Automated Validation]
        MONITORING[Backup Monitoring<br/>Success/Failure Alerts]
    end
    
    CONTINUOUS --> LOCAL
    HOURLY --> LOCAL
    DAILY --> REMOTE
    WEEKLY --> REMOTE
    
    LOCAL --> ENCRYPTED
    REMOTE --> ENCRYPTED
    ENCRYPTED --> COMPRESSED
    
    COMPRESSED --> INTEGRITY
    INTEGRITY --> RESTORE_TEST
    RESTORE_TEST --> MONITORING
    
    classDef schedule fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef storage fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef verification fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    
    class CONTINUOUS,HOURLY,DAILY,WEEKLY schedule
    class LOCAL,REMOTE,ENCRYPTED,COMPRESSED storage
    class INTEGRITY,RESTORE_TEST,MONITORING verification
```

### Backup Script Configuration

Create automated backup with cron:

```bash
# Add to crontab for automated backups
# Hourly incremental backups
0 * * * * /app/scripts/backup.sh --incremental

# Daily full backups at 2 AM
0 2 * * * /app/scripts/backup.sh --full

# Weekly cleanup of old backups
0 3 * * 0 /app/scripts/backup.sh --cleanup

# Monthly archive to remote storage
0 4 1 * * /app/scripts/backup.sh --archive
```

### Disaster Recovery Procedures

```mermaid
graph TD
    subgraph "Disaster Scenarios"
        DATA_CORRUPTION[Data Corruption<br/>Database Issues]
        HARDWARE_FAILURE[Hardware Failure<br/>Server Down]
        SECURITY_BREACH[Security Breach<br/>Compromised System]
        HUMAN_ERROR[Human Error<br/>Accidental Deletion]
    end
    
    subgraph "Recovery Actions"
        ASSESS[Assess Damage<br/>Determine Scope]
        ISOLATE[Isolate System<br/>Prevent Further Damage]
        RESTORE[Restore from Backup<br/>Point-in-time Recovery]
        VERIFY[Verify Integrity<br/>Data Validation]
        RESUME[Resume Operations<br/>Service Restoration]
    end
    
    subgraph "Recovery Targets"
        RTO[Recovery Time Objective<br/>< 1 hour]
        RPO[Recovery Point Objective<br/>< 15 minutes]
        COMMUNICATION[Communication Plan<br/>Stakeholder Updates]
        POST_MORTEM[Post-mortem<br/>Lessons Learned]
    end
    
    DATA_CORRUPTION --> ASSESS
    HARDWARE_FAILURE --> ASSESS
    SECURITY_BREACH --> ASSESS
    HUMAN_ERROR --> ASSESS
    
    ASSESS --> ISOLATE
    ISOLATE --> RESTORE
    RESTORE --> VERIFY
    VERIFY --> RESUME
    
    RESUME --> RTO
    RESUME --> RPO
    RESUME --> COMMUNICATION
    COMMUNICATION --> POST_MORTEM
    
    classDef disaster fill:#ffebee,stroke:#d32f2f,stroke-width:2px
    classDef recovery fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef targets fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    
    class DATA_CORRUPTION,HARDWARE_FAILURE,SECURITY_BREACH,HUMAN_ERROR disaster
    class ASSESS,ISOLATE,RESTORE,VERIFY,RESUME recovery
    class RTO,RPO,COMMUNICATION,POST_MORTEM targets
```

## Security Configuration

### Production Security Checklist

```mermaid
graph TD
    subgraph "Network Security"
        FIREWALL[Firewall Rules<br/>Port Restrictions]
        VPN[VPN Access<br/>Admin Interfaces]
        DDoS[DDoS Protection<br/>Rate Limiting]
        SSL_CONFIG[SSL Configuration<br/>TLS 1.2+ Only]
    end
    
    subgraph "Application Security"
        AUTH[Authentication<br/>Strong Passwords]
        AUTHZ[Authorization<br/>Role-based Access]
        INPUT_VAL[Input Validation<br/>XSS/SQL Prevention]
        SESSION[Session Security<br/>Secure Tokens]
    end
    
    subgraph "Infrastructure Security"
        UPDATES[Security Updates<br/>OS + Dependencies]
        MONITORING[Security Monitoring<br/>Intrusion Detection]
        SECRETS[Secret Management<br/>Environment Variables]
        BACKUP_SEC[Backup Security<br/>Encrypted Storage]
    end
    
    subgraph "Compliance & Auditing"
        AUDIT_LOG[Audit Logging<br/>Access Tracking]
        COMPLIANCE[Compliance<br/>GDPR/SOC2]
        PENETRATION[Penetration Testing<br/>Regular Assessments]
        INCIDENT[Incident Response<br/>Security Procedures]
    end
    
    FIREWALL --> AUTH
    VPN --> AUTHZ
    DDoS --> INPUT_VAL
    SSL_CONFIG --> SESSION
    
    AUTH --> UPDATES
    AUTHZ --> MONITORING
    INPUT_VAL --> SECRETS
    SESSION --> BACKUP_SEC
    
    UPDATES --> AUDIT_LOG
    MONITORING --> COMPLIANCE
    SECRETS --> PENETRATION
    BACKUP_SEC --> INCIDENT
    
    classDef network fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef application fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef infrastructure fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef compliance fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class FIREWALL,VPN,DDoS,SSL_CONFIG network
    class AUTH,AUTHZ,INPUT_VAL,SESSION application
    class UPDATES,MONITORING,SECRETS,BACKUP_SEC infrastructure
    class AUDIT_LOG,COMPLIANCE,PENETRATION,INCIDENT compliance
```

### Security Headers Configuration

Ensure these security headers are set:

```bash
# Security headers (set by reverse proxy or application)
Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' wss:; font-src 'self'; object-src 'none'; media-src 'self'; frame-src 'none';
```

### Firewall Configuration

```bash
# UFW (Ubuntu Firewall) configuration
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH (change port if using non-standard)
sudo ufw allow 22/tcp

# Allow HTTP and HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Allow specific monitoring ports (if needed)
sudo ufw allow from 10.0.0.0/8 to any port 9090  # Prometheus
sudo ufw allow from 10.0.0.0/8 to any port 3001  # Grafana

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status verbose
```

## Performance Optimization

### Resource Allocation

```mermaid
graph TD
    subgraph "CPU Optimization"
        WORKER_THREADS[Worker Threads<br/>Match CPU Cores]
        ASYNC_RUNTIME[Async Runtime<br/>Tokio Configuration]
        CPU_AFFINITY[CPU Affinity<br/>Process Binding]
    end
    
    subgraph "Memory Optimization"
        HEAP_SIZE[Heap Size<br/>JVM-style Limits]
        CONNECTION_POOL[Connection Pool<br/>Database Connections]
        CACHE_SIZE[Cache Size<br/>In-memory Caching]
    end
    
    subgraph "I/O Optimization"
        DB_WAL[Database WAL Mode<br/>Write-Ahead Logging]
        DISK_IO[Disk I/O<br/>SSD Optimization]
        NETWORK_BUFFER[Network Buffers<br/>Socket Configuration]
    end
    
    subgraph "Application Optimization"
        COMPRESSION[Response Compression<br/>gzip/brotli]
        STATIC_CACHE[Static Asset Caching<br/>CDN Integration]
        WEBSOCKET_POOL[WebSocket Pool<br/>Connection Reuse]
    end
    
    WORKER_THREADS --> CONNECTION_POOL
    ASYNC_RUNTIME --> CACHE_SIZE
    CPU_AFFINITY --> DB_WAL
    
    CONNECTION_POOL --> DISK_IO
    CACHE_SIZE --> NETWORK_BUFFER
    DB_WAL --> COMPRESSION
    
    DISK_IO --> STATIC_CACHE
    NETWORK_BUFFER --> WEBSOCKET_POOL
    
    classDef cpu fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef memory fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef io fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef app fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class WORKER_THREADS,ASYNC_RUNTIME,CPU_AFFINITY cpu
    class HEAP_SIZE,CONNECTION_POOL,CACHE_SIZE memory
    class DB_WAL,DISK_IO,NETWORK_BUFFER io
    class COMPRESSION,STATIC_CACHE,WEBSOCKET_POOL app
```

### Performance Tuning Configuration

```bash
# Environment variables for performance tuning
CAMPFIRE_WORKER_THREADS=8                    # Match CPU cores
CAMPFIRE_DB_MAX_CONNECTIONS=50               # Database pool size
CAMPFIRE_REQUEST_TIMEOUT=30                  # Request timeout
CAMPFIRE_WEBSOCKET_PING_INTERVAL=30          # WebSocket keepalive
CAMPFIRE_COMPRESSION_LEVEL=6                 # gzip compression level
CAMPFIRE_CACHE_SIZE=100MB                    # In-memory cache size
CAMPFIRE_DB_CACHE_SIZE=64MB                  # SQLite cache size
CAMPFIRE_MAX_CONCURRENT_REQUESTS=1000        # Concurrent request limit
```

## Scaling Strategies

### Horizontal Scaling Architecture

```mermaid
graph TD
    subgraph "Load Balancer Tier"
        LB[Load Balancer<br/>nginx/HAProxy]
        HEALTH_LB[Health Checks<br/>Automatic Failover]
        SSL_TERM[SSL Termination<br/>Certificate Management]
    end
    
    subgraph "Application Tier (Scaled)"
        APP1[Campfire Instance 1<br/>Docker Container]
        APP2[Campfire Instance 2<br/>Docker Container]
        APP3[Campfire Instance 3<br/>Docker Container]
        APPN[Campfire Instance N<br/>Docker Container]
    end
    
    subgraph "Shared Services"
        REDIS[Redis<br/>Session Storage + Pub/Sub]
        POSTGRES[PostgreSQL<br/>Shared Database]
        S3[S3/MinIO<br/>File Storage]
    end
    
    subgraph "Monitoring (Scaled)"
        PROMETHEUS[Prometheus<br/>Metrics Aggregation]
        GRAFANA[Grafana<br/>Dashboards]
        ALERTMANAGER[AlertManager<br/>Notification Routing]
    end
    
    LB --> HEALTH_LB
    HEALTH_LB --> SSL_TERM
    SSL_TERM --> APP1
    SSL_TERM --> APP2
    SSL_TERM --> APP3
    SSL_TERM --> APPN
    
    APP1 --> REDIS
    APP2 --> REDIS
    APP3 --> REDIS
    APPN --> REDIS
    
    APP1 --> POSTGRES
    APP2 --> POSTGRES
    APP3 --> POSTGRES
    APPN --> POSTGRES
    
    APP1 --> S3
    APP2 --> S3
    APP3 --> S3
    APPN --> S3
    
    APP1 --> PROMETHEUS
    APP2 --> PROMETHEUS
    APP3 --> PROMETHEUS
    APPN --> PROMETHEUS
    
    PROMETHEUS --> GRAFANA
    PROMETHEUS --> ALERTMANAGER
    
    classDef lb fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef app fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef shared fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef monitoring fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class LB,HEALTH_LB,SSL_TERM lb
    class APP1,APP2,APP3,APPN app
    class REDIS,POSTGRES,S3 shared
    class PROMETHEUS,GRAFANA,ALERTMANAGER monitoring
```

### Database Scaling Considerations

For high-scale deployments, consider migrating from SQLite to PostgreSQL:

```sql
-- PostgreSQL configuration for scaled deployment
-- postgresql.conf optimizations

# Connection settings
max_connections = 200
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 4MB
maintenance_work_mem = 64MB

# WAL settings
wal_buffers = 16MB
checkpoint_completion_target = 0.9
wal_writer_delay = 200ms

# Query planner
random_page_cost = 1.1  # For SSD storage
effective_io_concurrency = 200

# Logging
log_min_duration_statement = 1000  # Log slow queries
log_checkpoints = on
log_connections = on
log_disconnections = on
```

## Troubleshooting Guide

### Common Issues and Solutions

```mermaid
graph TD
    subgraph "Performance Issues"
        SLOW_RESPONSE[Slow Response Times<br/>Check Database Queries]
        HIGH_MEMORY[High Memory Usage<br/>Check Connection Leaks]
        HIGH_CPU[High CPU Usage<br/>Check Background Tasks]
        DISK_FULL[Disk Space Full<br/>Check Logs + Database]
    end
    
    subgraph "Connectivity Issues"
        WS_DISCONNECT[WebSocket Disconnects<br/>Check Network/Proxy]
        DB_TIMEOUT[Database Timeouts<br/>Check Connection Pool]
        SSL_ERRORS[SSL Certificate Errors<br/>Check Expiry/Config]
        CORS_ERRORS[CORS Errors<br/>Check Origin Configuration]
    end
    
    subgraph "Application Issues"
        STARTUP_FAIL[Startup Failures<br/>Check Configuration]
        CRASH_LOOP[Crash Loop<br/>Check Error Logs]
        MIGRATION_FAIL[Migration Failures<br/>Check Database State]
        HEALTH_FAIL[Health Check Failures<br/>Check Dependencies]
    end
    
    subgraph "Diagnostic Tools"
        LOG_ANALYSIS[Log Analysis<br/>grep/awk/jq]
        METRICS_CHECK[Metrics Check<br/>Prometheus Queries]
        DB_ANALYSIS[Database Analysis<br/>SQLite Commands]
        NETWORK_TEST[Network Testing<br/>curl/telnet]
    end
    
    SLOW_RESPONSE --> LOG_ANALYSIS
    HIGH_MEMORY --> METRICS_CHECK
    HIGH_CPU --> METRICS_CHECK
    DISK_FULL --> LOG_ANALYSIS
    
    WS_DISCONNECT --> NETWORK_TEST
    DB_TIMEOUT --> DB_ANALYSIS
    SSL_ERRORS --> NETWORK_TEST
    CORS_ERRORS --> LOG_ANALYSIS
    
    STARTUP_FAIL --> LOG_ANALYSIS
    CRASH_LOOP --> LOG_ANALYSIS
    MIGRATION_FAIL --> DB_ANALYSIS
    HEALTH_FAIL --> METRICS_CHECK
    
    classDef performance fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef connectivity fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef application fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    classDef diagnostic fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    
    class SLOW_RESPONSE,HIGH_MEMORY,HIGH_CPU,DISK_FULL performance
    class WS_DISCONNECT,DB_TIMEOUT,SSL_ERRORS,CORS_ERRORS connectivity
    class STARTUP_FAIL,CRASH_LOOP,MIGRATION_FAIL,HEALTH_FAIL application
    class LOG_ANALYSIS,METRICS_CHECK,DB_ANALYSIS,NETWORK_TEST diagnostic
```

### Diagnostic Commands

```bash
# Application health and status
curl -f http://localhost:3000/health
docker logs -f campfire --tail 100
docker stats campfire

# Database diagnostics
sqlite3 /app/data/campfire.db "PRAGMA integrity_check;"
sqlite3 /app/data/campfire.db "PRAGMA table_info(messages);"
sqlite3 /app/data/campfire.db "SELECT COUNT(*) FROM messages;"

# System resource monitoring
htop
iotop
netstat -tulpn | grep 3000
ss -tuln | grep 3000

# Network connectivity testing
curl -I https://your-domain.com
curl -v wss://your-domain.com/ws
nslookup your-domain.com

# Log analysis
journalctl -u campfire -f
tail -f /var/log/nginx/campfire_error.log
grep -i error /app/logs/campfire.log | tail -20

# Metrics analysis
curl http://localhost:3000/metrics | grep campfire_http_requests_total
curl http://localhost:3000/metrics | grep campfire_websocket_connections
```

## Maintenance Procedures

### Regular Maintenance Tasks

```mermaid
graph TD
    subgraph "Daily Tasks"
        HEALTH_CHECK[Health Check<br/>Automated Monitoring]
        LOG_REVIEW[Log Review<br/>Error Analysis]
        BACKUP_VERIFY[Backup Verification<br/>Success Confirmation]
        METRICS_REVIEW[Metrics Review<br/>Performance Trends]
    end
    
    subgraph "Weekly Tasks"
        SECURITY_SCAN[Security Scan<br/>Vulnerability Assessment]
        PERFORMANCE_REVIEW[Performance Review<br/>Optimization Opportunities]
        CAPACITY_PLANNING[Capacity Planning<br/>Resource Forecasting]
        UPDATE_CHECK[Update Check<br/>Dependencies + OS]
    end
    
    subgraph "Monthly Tasks"
        FULL_BACKUP_TEST[Full Backup Test<br/>Disaster Recovery Drill]
        SECURITY_AUDIT[Security Audit<br/>Access Review]
        PERFORMANCE_BASELINE[Performance Baseline<br/>Benchmark Updates]
        DOCUMENTATION_UPDATE[Documentation Update<br/>Runbook Maintenance]
    end
    
    subgraph "Quarterly Tasks"
        DISASTER_RECOVERY[Disaster Recovery Test<br/>Full System Recovery]
        SECURITY_PENETRATION[Penetration Testing<br/>External Assessment]
        ARCHITECTURE_REVIEW[Architecture Review<br/>Scaling Assessment]
        COMPLIANCE_AUDIT[Compliance Audit<br/>Regulatory Requirements]
    end
    
    HEALTH_CHECK --> SECURITY_SCAN
    LOG_REVIEW --> PERFORMANCE_REVIEW
    BACKUP_VERIFY --> CAPACITY_PLANNING
    METRICS_REVIEW --> UPDATE_CHECK
    
    SECURITY_SCAN --> FULL_BACKUP_TEST
    PERFORMANCE_REVIEW --> SECURITY_AUDIT
    CAPACITY_PLANNING --> PERFORMANCE_BASELINE
    UPDATE_CHECK --> DOCUMENTATION_UPDATE
    
    FULL_BACKUP_TEST --> DISASTER_RECOVERY
    SECURITY_AUDIT --> SECURITY_PENETRATION
    PERFORMANCE_BASELINE --> ARCHITECTURE_REVIEW
    DOCUMENTATION_UPDATE --> COMPLIANCE_AUDIT
    
    classDef daily fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef weekly fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef monthly fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef quarterly fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class HEALTH_CHECK,LOG_REVIEW,BACKUP_VERIFY,METRICS_REVIEW daily
    class SECURITY_SCAN,PERFORMANCE_REVIEW,CAPACITY_PLANNING,UPDATE_CHECK weekly
    class FULL_BACKUP_TEST,SECURITY_AUDIT,PERFORMANCE_BASELINE,DOCUMENTATION_UPDATE monthly
    class DISASTER_RECOVERY,SECURITY_PENETRATION,ARCHITECTURE_REVIEW,COMPLIANCE_AUDIT quarterly
```

### Update and Deployment Procedures

```bash
# Production update procedure
#!/bin/bash

# 1. Pre-deployment checks
./scripts/deploy.sh status
./scripts/backup.sh

# 2. Deploy to staging first
docker build -t campfire-rust:staging .
docker run --rm campfire-rust:staging cargo test

# 3. Deploy to production with zero downtime
./scripts/deploy.sh deploy --no-cache

# 4. Post-deployment verification
sleep 30
curl -f http://localhost:3000/health
./scripts/deploy.sh status

# 5. Monitor for issues
./scripts/deploy.sh logs --tail 100
```

## Cloud Platform Deployment

### AWS Deployment

```yaml
# docker-compose.aws.yml
version: '3.8'

services:
  campfire:
    image: your-account.dkr.ecr.region.amazonaws.com/campfire-rust:latest
    environment:
      - CAMPFIRE_DATABASE_URL=/app/data/campfire.db
      - CAMPFIRE_BACKUP_DIR=/app/backups
      - AWS_REGION=us-west-2
      - AWS_S3_BACKUP_BUCKET=campfire-backups
    volumes:
      - campfire_data:/app/data
      - campfire_logs:/app/logs
    deploy:
      replicas: 2
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
    healthcheck:
      test: ["CMD", "/app/healthcheck.sh"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  campfire_data:
    driver: local
    driver_opts:
      type: nfs
      o: addr=fs-12345.efs.region.amazonaws.com,rsize=1048576,wsize=1048576,hard,intr,nfsvers=4.1
      device: :/
```

### Google Cloud Platform Deployment

```yaml
# cloudbuild.yaml
steps:
  - name: 'gcr.io/cloud-builders/docker'
    args: ['build', '-t', 'gcr.io/$PROJECT_ID/campfire-rust:$COMMIT_SHA', '.']
  
  - name: 'gcr.io/cloud-builders/docker'
    args: ['push', 'gcr.io/$PROJECT_ID/campfire-rust:$COMMIT_SHA']
  
  - name: 'gcr.io/cloud-builders/gcloud'
    args:
      - 'run'
      - 'deploy'
      - 'campfire-rust'
      - '--image=gcr.io/$PROJECT_ID/campfire-rust:$COMMIT_SHA'
      - '--region=us-central1'
      - '--platform=managed'
      - '--allow-unauthenticated'
      - '--memory=512Mi'
      - '--cpu=1'
      - '--max-instances=10'
      - '--set-env-vars=CAMPFIRE_LOG_LEVEL=info'
```

### Azure Container Instances

```yaml
# azure-container-instance.yaml
apiVersion: 2019-12-01
location: eastus
name: campfire-rust
properties:
  containers:
  - name: campfire
    properties:
      image: your-registry.azurecr.io/campfire-rust:latest
      resources:
        requests:
          cpu: 1
          memoryInGb: 0.5
      ports:
      - port: 3000
        protocol: TCP
      environmentVariables:
      - name: CAMPFIRE_HOST
        value: 0.0.0.0
      - name: CAMPFIRE_PORT
        value: 3000
      - name: CAMPFIRE_LOG_LEVEL
        value: info
      volumeMounts:
      - name: campfire-data
        mountPath: /app/data
  osType: Linux
  restartPolicy: Always
  ipAddress:
    type: Public
    ports:
    - protocol: TCP
      port: 3000
  volumes:
  - name: campfire-data
    azureFile:
      shareName: campfire-data
      storageAccountName: your-storage-account
      storageAccountKey: your-storage-key
tags:
  environment: production
  application: campfire-rust
```

## Conclusion

This production deployment guide provides comprehensive coverage of deploying Campfire Rust in production environments, from simple single-server deployments to scalable cloud architectures. The guide emphasizes:

- **Security-first approach** with proper SSL, headers, and access controls
- **Monitoring and observability** with comprehensive metrics and alerting
- **Automated backup and recovery** procedures for data protection
- **Performance optimization** for production workloads
- **Scalability considerations** for growing user bases
- **Maintenance procedures** for long-term operational success

Following these patterns ensures a robust, secure, and maintainable production deployment that can scale with your organization's needs while maintaining the simplicity and performance benefits of the Rust implementation.

For additional support and advanced deployment scenarios, refer to the specific cloud platform documentation and consider engaging with the Campfire Rust community for deployment best practices and troubleshooting assistance.