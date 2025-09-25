# Docker Deployment Guide for Campfire Rust

This guide provides comprehensive instructions for deploying Campfire using Docker and Docker Compose, including production configurations, environment variables, and best practices.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Environment Configuration](#environment-configuration)
3. [Docker Compose Profiles](#docker-compose-profiles)
4. [Production Deployment](#production-deployment)
5. [SSL/TLS Configuration](#ssltls-configuration)
6. [Monitoring Setup](#monitoring-setup)
7. [Backup and Restore](#backup-and-restore)
8. [Troubleshooting](#troubleshooting)
9. [Performance Tuning](#performance-tuning)

## Quick Start

### Prerequisites

- Docker Engine 20.10+
- Docker Compose 2.0+
- 2GB RAM minimum
- 10GB disk space

### Basic Deployment

1. **Clone and configure**:
   ```bash
   git clone <repository-url>
   cd campfire-on-rust
   
   # Copy environment template
   cp .env.example .env.production
   
   # Edit configuration (see Environment Configuration section)
   nano .env.production
   ```

2. **Start the application**:
   ```bash
   # Basic deployment
   docker-compose up -d
   
   # Check status
   docker-compose ps
   
   # View logs
   docker-compose logs -f campfire
   ```

3. **Access the application**:
   - Web interface: http://localhost:3000
   - Health check: http://localhost:3000/health
   - Metrics: http://localhost:3000/metrics

## Environment Configuration

### Required Environment Variables

Create `.env.production` with these essential settings:

```bash
# Server Configuration
CAMPFIRE_HOST=0.0.0.0
CAMPFIRE_PORT=3000
CAMPFIRE_WORKER_THREADS=4

# Database
CAMPFIRE_DATABASE_URL=/app/data/campfire.db
CAMPFIRE_DB_MAX_CONNECTIONS=20
CAMPFIRE_DB_WAL_MODE=true

# Security
CAMPFIRE_CORS_ORIGINS=https://your-domain.com
CAMPFIRE_RATE_LIMIT_RPM=120
CAMPFIRE_FORCE_HTTPS=true
CAMPFIRE_TRUST_PROXY=true

# Logging
CAMPFIRE_LOG_LEVEL=info
CAMPFIRE_LOG_FORMAT=json
CAMPFIRE_LOG_FILE=/app/logs/campfire.log
CAMPFIRE_LOG_STRUCTURED=true

# Push Notifications (optional)
CAMPFIRE_PUSH_ENABLED=true
CAMPFIRE_VAPID_PRIVATE_KEY=your_private_key
CAMPFIRE_VAPID_PUBLIC_KEY=your_public_key
CAMPFIRE_VAPID_SUBJECT=mailto:admin@your-domain.com
```

### Generating VAPID Keys

For push notifications, generate VAPID keys:

```bash
# Generate private key
openssl ecparam -genkey -name prime256v1 -noout -out vapid_private.pem

# Extract public key (base64 URL-safe)
openssl ec -in vapid_private.pem -pubout -outform DER | tail -c 65 | base64 | tr -d '=' | tr '/+' '_-'

# Extract private key (base64 URL-safe)
openssl ec -in vapid_private.pem -outform DER | tail -c +8 | head -c 32 | base64 | tr -d '=' | tr '/+' '_-'
```

## Docker Compose Profiles

The docker-compose.yml includes several profiles for different deployment scenarios:

### Default Profile (Application Only)

```bash
# Start just the Campfire application
docker-compose up -d
```

Services included:
- `campfire`: Main application container

### Monitoring Profile

```bash
# Start with monitoring stack
docker-compose --profile monitoring up -d
```

Additional services:
- `prometheus`: Metrics collection (port 9090)
- `grafana`: Metrics visualization (port 3001)
- `alertmanager`: Alert routing (port 9093)
- `blackbox`: External monitoring (port 9115)

### Proxy Profile

```bash
# Start with reverse proxy
docker-compose --profile proxy up -d
```

Additional services:
- `traefik`: Reverse proxy with automatic SSL (ports 80, 443, 8080)

### Full Production Stack

```bash
# Start everything
docker-compose --profile monitoring --profile proxy up -d
```

## Production Deployment

### 1. System Preparation

```bash
# Create deployment directory
sudo mkdir -p /opt/campfire
cd /opt/campfire

# Create data directories
sudo mkdir -p data logs backups monitoring
sudo chown -R 1001:1001 data logs backups

# Set up log rotation
sudo tee /etc/logrotate.d/campfire <<EOF
/opt/campfire/logs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 1001 1001
    postrotate
        docker kill -s USR1 campfire 2>/dev/null || true
    endscript
}
EOF
```

### 2. Production Configuration

Create production environment file:

```bash
# /opt/campfire/.env.production
CAMPFIRE_HOST=0.0.0.0
CAMPFIRE_PORT=3000
CAMPFIRE_WORKER_THREADS=8

# Database configuration
CAMPFIRE_DATABASE_URL=/app/data/campfire.db
CAMPFIRE_DB_MAX_CONNECTIONS=50
CAMPFIRE_DB_WAL_MODE=true

# Security settings
CAMPFIRE_CORS_ORIGINS=https://chat.yourdomain.com
CAMPFIRE_RATE_LIMIT_RPM=300
CAMPFIRE_FORCE_HTTPS=true
CAMPFIRE_TRUST_PROXY=true
CAMPFIRE_SESSION_EXPIRY_HOURS=168  # 7 days

# Production logging
CAMPFIRE_LOG_LEVEL=info
CAMPFIRE_LOG_FORMAT=json
CAMPFIRE_LOG_FILE=/app/logs/campfire.log
CAMPFIRE_LOG_STRUCTURED=true
CAMPFIRE_TRACE_REQUESTS=false

# Performance settings
CAMPFIRE_REQUEST_TIMEOUT=60
CAMPFIRE_MAX_REQUEST_SIZE=33554432  # 32MB
CAMPFIRE_SHUTDOWN_TIMEOUT=30

# Monitoring
CAMPFIRE_METRICS_ENABLED=true
CAMPFIRE_METRICS_DETAILED=true

# Push notifications
CAMPFIRE_PUSH_ENABLED=true
CAMPFIRE_VAPID_PRIVATE_KEY=${VAPID_PRIVATE_KEY}
CAMPFIRE_VAPID_PUBLIC_KEY=${VAPID_PUBLIC_KEY}
CAMPFIRE_VAPID_SUBJECT=mailto:admin@yourdomain.com
```

### 3. Production Docker Compose Override

Create `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  campfire:
    restart: always
    
    # Production resource limits
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '4.0'
        reservations:
          memory: 1G
          cpus: '2.0'
    
    # Production environment
    env_file:
      - .env.production
    
    # Enhanced health check
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health/ready"]
      interval: 15s
      timeout: 5s
      retries: 5
      start_period: 30s
    
    # Production logging
    logging:
      driver: "json-file"
      options:
        max-size: "50m"
        max-file: "5"
        compress: "true"
    
    # Security enhancements
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE
    
    # Labels for monitoring
    labels:
      - "prometheus.io/scrape=true"
      - "prometheus.io/port=3000"
      - "prometheus.io/path=/metrics"

  # Production Traefik configuration
  traefik:
    command:
      - "--api.dashboard=false"  # Disable dashboard in production
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--certificatesresolvers.letsencrypt.acme.tlschallenge=true"
      - "--certificatesresolvers.letsencrypt.acme.email=admin@yourdomain.com"
      - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
      - "--log.level=INFO"
      - "--accesslog=true"
    
    labels:
      # Redirect HTTP to HTTPS
      - "traefik.http.routers.redirect-to-https.rule=hostregexp(`{host:.+}`)"
      - "traefik.http.routers.redirect-to-https.entrypoints=web"
      - "traefik.http.routers.redirect-to-https.middlewares=redirect-to-https"
      - "traefik.http.middlewares.redirect-to-https.redirectscheme.scheme=https"
```

### 4. Deploy to Production

```bash
# Deploy with production overrides
docker-compose -f docker-compose.yml -f docker-compose.prod.yml --profile monitoring --profile proxy up -d

# Verify deployment
docker-compose ps
docker-compose logs campfire

# Test health endpoints
curl -f https://chat.yourdomain.com/health
curl -f https://chat.yourdomain.com/health/ready
```

## SSL/TLS Configuration

### Automatic SSL with Let's Encrypt (Recommended)

The Traefik configuration automatically handles SSL certificates:

```yaml
# In docker-compose.yml, Traefik is configured for automatic SSL
labels:
  - "traefik.enable=true"
  - "traefik.http.routers.campfire.rule=Host(`chat.yourdomain.com`)"
  - "traefik.http.routers.campfire.entrypoints=websecure"
  - "traefik.http.routers.campfire.tls.certresolver=letsencrypt"
```

### Manual SSL Certificate

For custom certificates, mount them into the Traefik container:

```yaml
traefik:
  volumes:
    - /path/to/certs:/certs:ro
  command:
    - "--providers.file.filename=/certs/dynamic.yml"
```

Create `/path/to/certs/dynamic.yml`:

```yaml
tls:
  certificates:
    - certFile: /certs/yourdomain.com.crt
      keyFile: /certs/yourdomain.com.key
```

### SSL Configuration Verification

```bash
# Test SSL configuration
curl -I https://chat.yourdomain.com

# Check certificate details
openssl s_client -connect chat.yourdomain.com:443 -servername chat.yourdomain.com

# Test SSL Labs rating
curl -s "https://api.ssllabs.com/api/v3/analyze?host=chat.yourdomain.com"
```

## Monitoring Setup

### Prometheus Configuration

Create `monitoring/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "rules/*.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  - job_name: 'campfire'
    static_configs:
      - targets: ['campfire:3000']
    metrics_path: '/metrics'
    scrape_interval: 10s

  - job_name: 'blackbox'
    metrics_path: /probe
    params:
      module: [http_2xx]
    static_configs:
      - targets:
        - http://campfire:3000/health
        - https://chat.yourdomain.com/health
    relabel_configs:
      - source_labels: [__address__]
        target_label: __param_target
      - source_labels: [__param_target]
        target_label: instance
      - target_label: __address__
        replacement: blackbox:9115
```

### Grafana Dashboards

The monitoring setup includes pre-configured dashboards:

1. **Campfire Overview**: High-level application metrics
2. **Performance Dashboard**: Response times and throughput
3. **System Resources**: CPU, memory, disk usage
4. **WebSocket Monitoring**: Real-time connection metrics
5. **Database Performance**: Query performance and connection pool

Access Grafana at http://localhost:3001 (admin/admin)

### Alerting Rules

Create `monitoring/rules/campfire.yml`:

```yaml
groups:
  - name: campfire.rules
    rules:
      - alert: CampfireDown
        expr: up{job="campfire"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Campfire application is down"

      - alert: HighErrorRate
        expr: rate(campfire_http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High error rate: {{ $value }} errors/sec"

      - alert: HighResponseTime
        expr: histogram_quantile(0.95, rate(campfire_http_request_duration_seconds_bucket[5m])) > 1.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High response time: {{ $value }}s"
```

## Backup and Restore

### Automated Backup Setup

```bash
# Create backup script
sudo tee /opt/campfire/backup-cron.sh <<'EOF'
#!/bin/bash
cd /opt/campfire
docker-compose exec -T campfire /app/scripts/backup-enhanced.sh full --no-verify
find ./backups -name "campfire_*_backup_*.db*" -mtime +30 -delete
EOF

sudo chmod +x /opt/campfire/backup-cron.sh

# Add to crontab
echo "0 2 * * * /opt/campfire/backup-cron.sh" | sudo crontab -
```

### Manual Backup and Restore

```bash
# Create backup
docker-compose exec campfire /app/scripts/backup-enhanced.sh full

# List backups
docker-compose exec campfire ls -la /app/backups/

# Restore from backup
docker-compose exec campfire /app/scripts/restore-enhanced.sh /app/backups/campfire_backup_20240101_120000.db.gz
```

### Backup to External Storage

```bash
# Backup to S3
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret
export CAMPFIRE_S3_BUCKET=your-backup-bucket

docker-compose exec -e AWS_ACCESS_KEY_ID -e AWS_SECRET_ACCESS_KEY -e CAMPFIRE_S3_BUCKET campfire \
  /app/scripts/backup-enhanced.sh full --remote
```

## Troubleshooting

### Common Issues

#### Container Won't Start

```bash
# Check logs
docker-compose logs campfire

# Check configuration
docker-compose config

# Verify environment file
cat .env.production

# Check file permissions
ls -la data/ logs/ backups/
```

#### Database Connection Errors

```bash
# Check database file
docker-compose exec campfire ls -la /app/data/

# Test database integrity
docker-compose exec campfire sqlite3 /app/data/campfire.db "PRAGMA integrity_check;"

# Check WAL files
docker-compose exec campfire ls -la /app/data/campfire.db*
```

#### SSL Certificate Issues

```bash
# Check Traefik logs
docker-compose logs traefik

# Verify domain DNS
nslookup chat.yourdomain.com

# Check Let's Encrypt rate limits
curl -s "https://crt.sh/?q=yourdomain.com&output=json" | jq length
```

#### Performance Issues

```bash
# Monitor resource usage
docker stats campfire

# Check application metrics
curl http://localhost:3000/metrics

# Run performance monitor
docker-compose exec campfire /app/scripts/performance-monitor.sh -d 300
```

### Debug Mode

Enable debug logging:

```bash
# Add to .env.production
CAMPFIRE_LOG_LEVEL=debug
RUST_LOG=campfire_on_rust=debug,tower_http=debug

# Restart container
docker-compose restart campfire

# Follow debug logs
docker-compose logs -f campfire
```

### Health Check Debugging

```bash
# Manual health checks
curl -v http://localhost:3000/health
curl -v http://localhost:3000/health/ready
curl -v http://localhost:3000/health/live

# Check container health status
docker inspect campfire | jq '.[0].State.Health'
```

## Performance Tuning

### Container Resource Optimization

```yaml
# docker-compose.prod.yml
services:
  campfire:
    deploy:
      resources:
        limits:
          memory: 4G      # Adjust based on usage
          cpus: '4.0'     # Adjust based on CPU cores
        reservations:
          memory: 2G
          cpus: '2.0'
    
    # Optimize for performance
    sysctls:
      - net.core.somaxconn=65536
      - net.ipv4.tcp_max_syn_backlog=65536
```

### Application Performance Settings

```bash
# High-performance configuration
CAMPFIRE_WORKER_THREADS=16
CAMPFIRE_DB_MAX_CONNECTIONS=100
CAMPFIRE_REQUEST_TIMEOUT=120
CAMPFIRE_MAX_REQUEST_SIZE=67108864  # 64MB

# Database optimization
CAMPFIRE_DB_WAL_MODE=true
CAMPFIRE_DB_CACHE_SIZE=20000  # 80MB cache
CAMPFIRE_DB_MMAP_SIZE=1073741824  # 1GB mmap
```

### System-Level Optimizations

```bash
# Increase file descriptor limits
echo "fs.file-max = 2097152" >> /etc/sysctl.conf
echo "1001 soft nofile 65536" >> /etc/security/limits.conf
echo "1001 hard nofile 65536" >> /etc/security/limits.conf

# Optimize network settings
echo "net.core.somaxconn = 65536" >> /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 65536" >> /etc/sysctl.conf
echo "net.core.netdev_max_backlog = 5000" >> /etc/sysctl.conf

# Apply settings
sysctl -p
```

### Load Testing

```bash
# Install testing tools
npm install -g artillery

# Create load test configuration
cat > artillery-config.yml <<EOF
config:
  target: 'https://chat.yourdomain.com'
  phases:
    - duration: 300
      arrivalRate: 50
scenarios:
  - name: "API Load Test"
    flow:
      - get:
          url: "/health"
      - post:
          url: "/api/messages"
          json:
            content: "Load test message"
EOF

# Run load test
artillery run artillery-config.yml
```

This comprehensive Docker deployment guide covers all aspects of deploying Campfire in production environments, from basic setup to advanced performance tuning and troubleshooting.