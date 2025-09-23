# Campfire Rust - Production Deployment Guide

This guide covers deploying Campfire Rust to production using Docker and Docker Compose.

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
   docker-compose up -d
   ```

2. **With monitoring**:
   ```bash
   docker-compose --profile monitoring up -d
   ```

3. **With reverse proxy**:
   ```bash
   docker-compose --profile proxy up -d
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

## Monitoring

### Health Checks

- **Basic health**: `GET /health`
- **Readiness**: `GET /health/ready`
- **Liveness**: `GET /health/live`

### Metrics

Prometheus metrics available at `/metrics`:

- HTTP request metrics
- Database operation metrics
- WebSocket connection metrics
- Application-specific metrics

### Grafana Dashboard

If using the monitoring profile:
- Grafana: http://localhost:3001
- Username: admin
- Password: admin

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

### Horizontal Scaling

For multiple instances:

1. **Use external database** (PostgreSQL recommended for production)
2. **Shared session storage** (Redis)
3. **Load balancer** with sticky sessions for WebSocket
4. **Shared file storage** for uploads

### Vertical Scaling

Increase container resources:
```yaml
deploy:
  resources:
    limits:
      memory: 1G
      cpus: '2.0'
```

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