# Campfire Rust - Production Deployment Guide

This comprehensive guide covers deploying Campfire Rust to production using Docker and Docker Compose, including monitoring, backup strategies, and performance optimization.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Configuration](#configuration)
3. [Deployment Methods](#deployment-methods)
4. [Database Management](#database-management)
5. [Monitoring and Alerting](#monitoring-and-alerting)
6. [Security](#security)
7. [Performance Tuning](#performance-tuning)
8. [Scaling](#scaling)
9. [Backup Strategy](#backup-strategy)
10. [Troubleshooting](#troubleshooting)
11. [Maintenance](#maintenance)

## Quick Start

1. **Clone and configure**:
   ```bash
   git clone <repository-url>
   cd campfire-on-rust
   cp .env.example .env.production
   # Edit .env.production with your settings
   ```

2. **Deploy with Docker Compose**:
   ```bash
   ./scripts/deploy.sh deploy
   ```

3. **Access the application**:
   - Web interface: http://localhost:3000
   - Health check: http://localhost:3000/health
   - Metrics: http://localhost:3000/metrics

## Configuration

### Environment Variables

Copy `.env.example` to `.env.production` and customize:

#### Required Settings
```bash
# Server
CAMPFIRE_HOST=0.0.0.0
CAMPFIRE_PORT=3000

# Database
CAMPFIRE_DATABASE_URL=/app/data/campfire.db

# Security (generate secure values)
CAMPFIRE_SESSION_TOKEN_LENGTH=32
CAMPFIRE_CORS_ORIGINS=https://your-domain.com
```

#### Push Notifications
Generate VAPID keys for push notifications:
```bash
# Generate private key
openssl ecparam -genkey -name prime256v1 -noout -out vapid_private.pem

# Extract public key
openssl ec -in vapid_private.pem -pubout -outform DER | tail -c 65 | base64 | tr -d '=' | tr '/+' '_-'

# Extract private key
openssl ec -in vapid_private.pem -outform DER | tail -c +8 | head -c 32 | base64 | tr -d '=' | tr '/+' '_-'
```

Set in `.env.production`:
```bash
CAMPFIRE_PUSH_ENABLED=true
CAMPFIRE_VAPID_PRIVATE_KEY=<private_key_base64>
CAMPFIRE_VAPID_PUBLIC_KEY=<public_key_base64>
CAMPFIRE_VAPID_SUBJECT=mailto:admin@your-domain.com
```

## Deployment Methods

### Method 1: Docker Compose (Recommended)

1. **Basic deployment**:
   ```bash
   # Start with default configuration
   docker-compose up -d
   
   # Check status
   docker-compose ps
   
   # View logs
   docker-compose logs -f campfire
   ```

2. **With monitoring stack**:
   ```bash
   # Start with Prometheus and Grafana
   docker-compose --profile monitoring up -d
   
   # Access monitoring
   # - Prometheus: http://localhost:9090
   # - Grafana: http://localhost:3001 (admin/admin)
   ```

3. **With reverse proxy and SSL**:
   ```bash
   # Start with Traefik reverse proxy
   docker-compose --profile proxy up -d
   
   # Access via proxy
   # - Application: http://campfire.localhost
   # - Traefik Dashboard: http://traefik.localhost:8080
   ```

4. **Full production stack**:
   ```bash
   # Start everything (app + monitoring + proxy)
   docker-compose --profile monitoring --profile proxy up -d
   
   # Verify all services are running
   docker-compose ps
   ```

### Method 2: Deployment Script

The deployment script provides additional automation:

```bash
# Build and deploy
./scripts/deploy.sh deploy

# Deploy with fresh build (no cache)
./scripts/deploy.sh deploy --no-cache

# Check status
./scripts/deploy.sh status

# View logs
./scripts/deploy.sh logs

# Create backup
./scripts/deploy.sh backup
```

### Method 3: Manual Docker

1. **Build image**:
   ```bash
   docker build -t campfire-on-rust:latest .
   ```

2. **Create directories**:
   ```bash
   mkdir -p data logs backups
   ```

3. **Run container**:
   ```bash
   docker run -d \
     --name campfire \
     --restart unless-stopped \
     -p 3000:3000 \
     -v $(pwd)/data:/app/data \
     -v $(pwd)/logs:/app/logs \
     -v $(pwd)/backups:/app/backups \
     --env-file .env.production \
     campfire-on-rust:latest
   ```

## Database Management

### Backups

**Automatic backups** (recommended):
```bash
# Add to crontab for daily backups at 2 AM
0 2 * * * /path/to/campfire/scripts/backup.sh
```

**Manual backup**:
```bash
./scripts/backup.sh
```

**Backup via Docker**:
```bash
docker exec campfire /app/scripts/backup.sh
```

### Restore

**From backup file**:
```bash
./scripts/restore.sh backups/campfire_backup_20240101_120000.db.gz
```

**Interactive restore**:
```bash
./scripts/restore.sh
```

### Migrations

**Check migration status**:
```bash
./scripts/migrate.sh status
```

**Run migrations**:
```bash
./scripts/migrate.sh migrate
```

**Create new migration**:
```bash
./scripts/migrate.sh create add_new_feature
```

## Monitoring and Alerting

### Health Checks

The application provides comprehensive health check endpoints:

- **Basic health**: `GET /health` - Simple alive check
- **Readiness**: `GET /health/ready` - Ready to serve traffic
- **Liveness**: `GET /health/live` - Application is functioning
- **Detailed**: `GET /health/detailed` - Component-level status

```bash
# Check application health
curl -f http://localhost:3000/health

# Get detailed health information
curl http://localhost:3000/health/detailed | jq
```

### Metrics Collection

Prometheus metrics are available at `/metrics` endpoint:

#### Application Metrics
- `campfire_http_requests_total` - HTTP request counter
- `campfire_http_request_duration_seconds` - Request duration histogram
- `campfire_websocket_connections_active` - Active WebSocket connections
- `campfire_messages_sent_total` - Total messages sent
- `campfire_database_operations_total` - Database operation counter
- `campfire_database_operation_duration_seconds` - Database operation duration

#### System Metrics
- `campfire_memory_usage_bytes` - Memory usage
- `campfire_cpu_usage_percent` - CPU usage percentage
- `campfire_disk_usage_bytes` - Disk usage
- `campfire_uptime_seconds` - Application uptime

### Monitoring Stack Setup

Deploy the complete monitoring stack:

```bash
# Start with monitoring
docker-compose --profile monitoring up -d

# Or add monitoring to existing deployment
docker-compose up -d prometheus grafana
```

#### Prometheus Configuration

The Prometheus configuration includes:
- Application metrics scraping every 10 seconds
- System metrics collection
- 30-day retention policy
- Alerting rules for critical conditions

#### Grafana Dashboards

Access Grafana at http://localhost:3001:
- **Username**: admin
- **Password**: admin (change on first login)

Pre-configured dashboards:
1. **Campfire Overview** - High-level application metrics
2. **Performance Dashboard** - Response times and throughput
3. **System Resources** - CPU, memory, disk usage
4. **WebSocket Monitoring** - Real-time connection metrics
5. **Database Performance** - Query performance and connection pool

### Alerting Rules

Create alerting rules in `monitoring/rules/campfire.yml`:

```yaml
groups:
  - name: campfire.rules
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: rate(campfire_http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors per second"

      # High response time
      - alert: HighResponseTime
        expr: histogram_quantile(0.95, rate(campfire_http_request_duration_seconds_bucket[5m])) > 1.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High response time detected"
          description: "95th percentile response time is {{ $value }}s"

      # Database connection issues
      - alert: DatabaseConnectionFailure
        expr: campfire_database_operations_total{status="error"} > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Database connection failures"
          description: "Database operations are failing"

      # Memory usage
      - alert: HighMemoryUsage
        expr: campfire_memory_usage_bytes / (1024*1024*1024) > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}GB"

      # WebSocket connection issues
      - alert: WebSocketConnectionDrop
        expr: decrease(campfire_websocket_connections_active[5m]) > 10
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "WebSocket connections dropping"
          description: "{{ $value }} WebSocket connections dropped in 5 minutes"
```

### External Monitoring Integration

#### Datadog Integration
```bash
# Add Datadog agent to docker-compose.yml
datadog:
  image: datadog/agent:latest
  environment:
    - DD_API_KEY=${DD_API_KEY}
    - DD_SITE=datadoghq.com
    - DD_LOGS_ENABLED=true
    - DD_PROCESS_AGENT_ENABLED=true
  volumes:
    - /var/run/docker.sock:/var/run/docker.sock:ro
    - /proc/:/host/proc/:ro
    - /sys/fs/cgroup/:/host/sys/fs/cgroup:ro
```

#### New Relic Integration
```bash
# Environment variables for New Relic
CAMPFIRE_NEWRELIC_LICENSE_KEY=your_license_key
CAMPFIRE_NEWRELIC_APP_NAME=campfire-production
```

### Log Aggregation

#### ELK Stack Integration
```yaml
# Add to docker-compose.yml
elasticsearch:
  image: docker.elastic.co/elasticsearch/elasticsearch:8.11.0
  environment:
    - discovery.type=single-node
    - xpack.security.enabled=false
  ports:
    - "9200:9200"

logstash:
  image: docker.elastic.co/logstash/logstash:8.11.0
  volumes:
    - ./monitoring/logstash/pipeline:/usr/share/logstash/pipeline:ro
  depends_on:
    - elasticsearch

kibana:
  image: docker.elastic.co/kibana/kibana:8.11.0
  ports:
    - "5601:5601"
  environment:
    - ELASTICSEARCH_HOSTS=http://elasticsearch:9200
  depends_on:
    - elasticsearch
```

#### Structured Logging Configuration
```bash
# Enable structured logging
CAMPFIRE_LOG_FORMAT=json
CAMPFIRE_LOG_STRUCTURED=true
CAMPFIRE_LOG_LEVEL=info

# Log sampling for high-traffic environments
CAMPFIRE_LOG_SAMPLE_RATE=0.1  # Log 10% of requests
```

### Log Management

**View logs**:
```bash
# Container logs
docker logs campfire

# Application logs (if file logging enabled)
tail -f logs/campfire.log

# Via deployment script
./scripts/deploy.sh logs
```

**Log rotation**:
Logs are automatically rotated by Docker. For file-based logging, implement log rotation:

```bash
# Add to logrotate
cat > /etc/logrotate.d/campfire << EOF
/path/to/campfire/logs/*.log {
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

## Security

### HTTPS Setup

1. **With Traefik** (included in docker-compose):
   ```bash
   docker-compose --profile proxy up -d
   ```

2. **With external reverse proxy**:
   Configure your reverse proxy (nginx, Apache, etc.) to:
   - Terminate SSL
   - Proxy to `http://localhost:3000`
   - Set `CAMPFIRE_TRUST_PROXY=true`

3. **Direct HTTPS** (not recommended):
   - Set `CAMPFIRE_FORCE_HTTPS=true`
   - Configure SSL certificates in the container

### Security Headers

The application automatically sets security headers:
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Referrer-Policy: strict-origin-when-cross-origin`
- `Strict-Transport-Security` (when HTTPS is enabled)

### Rate Limiting

Configure rate limiting:
```bash
CAMPFIRE_RATE_LIMIT_RPM=60  # Requests per minute per IP
```

## Performance Tuning

### Resource Limits

In `docker-compose.yml`:
```yaml
deploy:
  resources:
    limits:
      memory: 512M
      cpus: '1.0'
    reservations:
      memory: 256M
      cpus: '0.5'
```

### Database Optimization

```bash
# Enable WAL mode for better concurrency
CAMPFIRE_DB_WAL_MODE=true

# Adjust connection pool
CAMPFIRE_DB_MAX_CONNECTIONS=20
```

### Worker Threads

```bash
# Set worker threads (0 = auto-detect)
CAMPFIRE_WORKER_THREADS=4
```

## Troubleshooting

### Common Issues

1. **Container won't start**:
   ```bash
   # Check logs
   docker logs campfire
   
   # Check configuration
   docker exec campfire env | grep CAMPFIRE
   ```

2. **Database connection errors**:
   ```bash
   # Check database file permissions
   ls -la data/
   
   # Check database integrity
   sqlite3 data/campfire.db "PRAGMA integrity_check;"
   ```

3. **Push notifications not working**:
   ```bash
   # Verify VAPID keys are set
   docker exec campfire env | grep VAPID
   
   # Check push service logs
   docker logs campfire | grep -i push
   ```

### Debug Mode

Enable debug logging:
```bash
CAMPFIRE_LOG_LEVEL=debug
RUST_LOG=campfire_on_rust=debug,tower_http=debug
```

### Health Check Failures

```bash
# Manual health check
curl -f http://localhost:3000/health

# Check container health
docker inspect campfire | grep -A 10 Health
```

## Scaling

### Performance Monitoring

Monitor application performance with the built-in monitoring script:

```bash
# Monitor for 10 minutes with 5-second intervals
./scripts/performance-monitor.sh -d 600 -i 5

# Continuous monitoring
./scripts/performance-monitor.sh --continuous

# Generate report from existing data
./scripts/performance-monitor.sh --report-only
```

### Horizontal Scaling

For multiple instances, follow the comprehensive scaling guide:

1. **Database Migration**: Migrate from SQLite to PostgreSQL
   ```bash
   # PostgreSQL configuration
   CAMPFIRE_DATABASE_URL=postgresql://user:pass@postgres:5432/campfire
   CAMPFIRE_DB_MAX_CONNECTIONS=100
   ```

2. **Session Storage**: Use Redis for shared sessions
   ```bash
   CAMPFIRE_SESSION_STORE=redis
   CAMPFIRE_REDIS_URL=redis://redis-cluster:6379
   ```

3. **Load Balancer**: Configure HAProxy or Nginx with sticky sessions
   ```bash
   # Start with load balancer
   docker-compose --profile proxy up -d
   ```

4. **WebSocket Clustering**: Enable Redis-based WebSocket clustering
   ```bash
   CAMPFIRE_WEBSOCKET_CLUSTERING=true
   CAMPFIRE_REDIS_PUBSUB_URL=redis://redis:6379
   ```

### Vertical Scaling

Optimize single instance performance:

```yaml
# docker-compose.yml
deploy:
  resources:
    limits:
      memory: 4G
      cpus: '4.0'
    reservations:
      memory: 2G
      cpus: '2.0'
```

#### Performance Tuning

```bash
# Database optimization
CAMPFIRE_DB_WAL_MODE=true
CAMPFIRE_DB_MAX_CONNECTIONS=50
CAMPFIRE_DB_CACHE_SIZE=10000

# Application optimization
CAMPFIRE_WORKER_THREADS=8
CAMPFIRE_MESSAGE_BUFFER_SIZE=10000
CAMPFIRE_CACHE_ENABLED=true

# Memory optimization
CAMPFIRE_MEMORY_POOL_SIZE=1073741824  # 1GB
```

### Capacity Planning

| Deployment Size | Users | Instances | CPU | Memory | Storage |
|----------------|-------|-----------|-----|--------|---------|
| **Small** | 100 | 1 | 1 core | 1GB | 10GB |
| **Medium** | 1,000 | 2-3 | 2-4 cores | 2-4GB | 50GB |
| **Large** | 10,000 | 5-10 | 4-8 cores | 4-8GB | 500GB |
| **Enterprise** | 100,000+ | 20+ | 8+ cores | 8-16GB | 5TB+ |

For detailed scaling strategies, see [Scaling Guide](docs/scaling-guide.md).

## Backup Strategy

### Automated Backup Schedule

```bash
# Daily backups at 2 AM
0 2 * * * /path/to/campfire/scripts/backup.sh

# Weekly cleanup of old backups
0 3 * * 0 find /path/to/campfire/backups -name "*.db*" -mtime +30 -delete
```

### Backup Verification

```bash
# Test backup integrity
./scripts/restore.sh --dry-run backup_file.db.gz

# Verify backup in test environment
docker run --rm -v $(pwd)/backups:/backups campfire-on-rust:latest \
  sqlite3 /backups/latest.db "PRAGMA integrity_check;"
```

## Maintenance

### Updates

1. **Backup database**:
   ```bash
   ./scripts/backup.sh
   ```

2. **Pull latest image**:
   ```bash
   docker pull campfire-on-rust:latest
   ```

3. **Deploy update**:
   ```bash
   ./scripts/deploy.sh deploy
   ```

### Cleanup

```bash
# Clean up old Docker images and containers
./scripts/deploy.sh cleanup

# Clean up old logs
find logs/ -name "*.log.*" -mtime +30 -delete

# Clean up old backups
find backups/ -name "*.db*" -mtime +90 -delete
```

## Support

### Getting Help

1. Check application logs
2. Verify configuration
3. Test health endpoints
4. Check resource usage
5. Review security settings

### Performance Monitoring

Monitor these metrics:
- Response times
- Error rates
- Database query performance
- Memory usage
- WebSocket connections
- Push notification delivery rates

For production support, ensure you have:
- Monitoring and alerting set up
- Regular backups tested
- Log aggregation configured
- Performance baselines established