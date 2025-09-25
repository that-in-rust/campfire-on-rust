# Campfire Error Recovery Guide

This document provides comprehensive guidance for handling and recovering from errors in the Campfire Rust application.

## Table of Contents

1. [Error Categories](#error-categories)
2. [Common Error Scenarios](#common-error-scenarios)
3. [Automatic Recovery Procedures](#automatic-recovery-procedures)
4. [Manual Recovery Steps](#manual-recovery-steps)
5. [Prevention Strategies](#prevention-strategies)
6. [Monitoring and Alerting](#monitoring-and-alerting)
7. [Troubleshooting Checklist](#troubleshooting-checklist)

## Error Categories

### 1. Database Errors

**Error Codes:** `DATABASE_CONNECTION_FAILED`, `DATABASE_MIGRATION_FAILED`, `DATABASE_CONSTRAINT_VIOLATION`

**Common Causes:**
- Database file corruption
- Insufficient disk space
- File permission issues
- Concurrent access conflicts
- SQLite WAL mode issues

**Recovery Procedures:**
1. **Automatic Recovery:**
   - Connection retry with exponential backoff
   - Automatic database repair for minor corruption
   - WAL checkpoint and recovery

2. **Manual Recovery:**
   - Check disk space: `df -h`
   - Verify file permissions: `ls -la campfire.db`
   - Run database integrity check: `sqlite3 campfire.db "PRAGMA integrity_check;"`
   - Restore from backup if corruption is severe

### 2. Authentication Errors

**Error Codes:** `INVALID_CREDENTIALS`, `SESSION_EXPIRED`, `TOKEN_GENERATION_FAILED`

**Common Causes:**
- Expired sessions
- Invalid login credentials
- Token generation failures
- Session storage issues

**Recovery Procedures:**
1. **Automatic Recovery:**
   - Session refresh for expired tokens
   - Graceful logout and redirect to login

2. **Manual Recovery:**
   - Clear browser cookies and local storage
   - Reset user password if needed
   - Check session storage configuration

### 3. Message Processing Errors

**Error Codes:** `MESSAGE_TOO_LONG`, `RATE_LIMIT_EXCEEDED`, `ROOM_ACCESS_DENIED`

**Common Causes:**
- Content validation failures
- Rate limiting triggers
- Authorization issues
- WebSocket connection problems

**Recovery Procedures:**
1. **Automatic Recovery:**
   - Content truncation with user notification
   - Rate limit backoff and retry
   - WebSocket reconnection

2. **Manual Recovery:**
   - Review and adjust rate limits
   - Check room permissions
   - Restart WebSocket connections

### 4. Network and Connectivity Errors

**Error Codes:** `WEBSOCKET_CONNECTION_FAILED`, `PUSH_NOTIFICATION_FAILED`, `WEBHOOK_DELIVERY_FAILED`

**Common Causes:**
- Network connectivity issues
- Firewall blocking
- Service unavailability
- DNS resolution problems

**Recovery Procedures:**
1. **Automatic Recovery:**
   - Connection retry with circuit breaker
   - Fallback to polling for real-time features
   - Queue failed operations for retry

2. **Manual Recovery:**
   - Check network connectivity
   - Verify firewall rules
   - Test DNS resolution
   - Review service status

## Common Error Scenarios

### Scenario 1: Database Connection Lost

**Symptoms:**
- HTTP 500 errors on all database operations
- Log entries: "Database connection failed"
- Users unable to send messages or load rooms

**Immediate Actions:**
1. Check application logs for specific error details
2. Verify database file exists and is accessible
3. Check disk space availability
4. Restart application if connection pool is exhausted

**Recovery Steps:**
```bash
# Check database file
ls -la campfire.db

# Check disk space
df -h

# Test database connectivity
sqlite3 campfire.db "SELECT COUNT(*) FROM users;"

# Restart application
systemctl restart campfire
```

### Scenario 2: High Rate Limiting

**Symptoms:**
- Users receiving "Rate limit exceeded" errors
- Increased 429 HTTP responses
- Complaints about slow message sending

**Immediate Actions:**
1. Review rate limiting configuration
2. Check for bot or automated activity
3. Analyze traffic patterns

**Recovery Steps:**
```bash
# Check current rate limit settings
grep -r "RATE_LIMIT" /etc/campfire/

# Review recent activity logs
tail -f /var/log/campfire/audit.log | grep "RATE_LIMIT"

# Temporarily increase limits if needed
export CAMPFIRE_RATE_LIMIT_RPM=120
systemctl restart campfire
```

### Scenario 3: WebSocket Connection Issues

**Symptoms:**
- Real-time features not working
- Messages not appearing immediately
- Connection errors in browser console

**Immediate Actions:**
1. Check WebSocket endpoint accessibility
2. Verify proxy/load balancer configuration
3. Test WebSocket connections directly

**Recovery Steps:**
```bash
# Test WebSocket endpoint
wscat -c ws://localhost:3000/ws

# Check proxy configuration
nginx -t
systemctl reload nginx

# Review WebSocket logs
grep "websocket" /var/log/campfire/application.log
```

## Automatic Recovery Procedures

### Circuit Breaker Pattern

The application implements circuit breakers for external dependencies:

```rust
// Example circuit breaker configuration
let circuit_breaker = ErrorCircuitBreaker::new(
    5,  // failure_threshold
    Duration::from_secs(60)  // recovery_timeout
);
```

**States:**
- **Closed:** Normal operation, requests pass through
- **Open:** Failures exceeded threshold, requests fail fast
- **Half-Open:** Testing if service has recovered

### Retry Strategies

Different retry strategies are applied based on error type:

1. **Database Errors:** Exponential backoff, max 3 retries
2. **Network Errors:** Linear backoff, max 2 retries
3. **Validation Errors:** No retry (immediate failure)

### Graceful Degradation

When services are unavailable, the application provides:

1. **Offline Mode:** Cache recent messages for viewing
2. **Reduced Functionality:** Disable non-essential features
3. **User Notifications:** Clear status messages about service availability

## Manual Recovery Steps

### Database Recovery

1. **Check Database Integrity:**
   ```bash
   sqlite3 campfire.db "PRAGMA integrity_check;"
   ```

2. **Repair Minor Corruption:**
   ```bash
   sqlite3 campfire.db "PRAGMA wal_checkpoint(FULL);"
   sqlite3 campfire.db "VACUUM;"
   ```

3. **Restore from Backup:**
   ```bash
   cp /backup/campfire.db.backup campfire.db
   chown campfire:campfire campfire.db
   ```

### Session Recovery

1. **Clear All Sessions:**
   ```sql
   DELETE FROM sessions WHERE expires_at < datetime('now');
   ```

2. **Reset User Password:**
   ```bash
   # Use admin CLI tool
   ./campfire-admin reset-password user@example.com
   ```

### Performance Recovery

1. **Clear Connection Pool:**
   ```bash
   systemctl restart campfire
   ```

2. **Optimize Database:**
   ```bash
   sqlite3 campfire.db "ANALYZE;"
   sqlite3 campfire.db "REINDEX;"
   ```

## Prevention Strategies

### Monitoring

1. **Health Checks:**
   - Database connectivity: `/health/ready`
   - Application status: `/health/live`
   - Service dependencies: `/health`

2. **Metrics Collection:**
   - Response times
   - Error rates
   - Database performance
   - WebSocket connections

3. **Log Analysis:**
   - Error patterns
   - Performance trends
   - Security events

### Backup Procedures

1. **Automated Backups:**
   ```bash
   # Daily backup script
   #!/bin/bash
   DATE=$(date +%Y%m%d_%H%M%S)
   cp campfire.db "/backup/campfire_${DATE}.db"
   find /backup -name "campfire_*.db" -mtime +7 -delete
   ```

2. **Backup Verification:**
   ```bash
   # Test backup integrity
   sqlite3 /backup/campfire_latest.db "PRAGMA integrity_check;"
   ```

### Configuration Management

1. **Environment Variables:**
   - Use configuration management tools
   - Version control configuration files
   - Validate configuration on startup

2. **Resource Limits:**
   - Set appropriate connection pool sizes
   - Configure rate limits based on capacity
   - Monitor resource usage

## Monitoring and Alerting

### Key Metrics to Monitor

1. **Application Metrics:**
   - HTTP response times (p95 < 500ms)
   - Error rates (< 1%)
   - WebSocket connections (active count)
   - Database query times (p95 < 100ms)

2. **System Metrics:**
   - CPU usage (< 80%)
   - Memory usage (< 85%)
   - Disk space (> 20% free)
   - Network connectivity

3. **Business Metrics:**
   - Message throughput
   - User activity
   - Room utilization
   - Feature usage

### Alert Thresholds

```yaml
alerts:
  - name: "High Error Rate"
    condition: "error_rate > 5%"
    duration: "5m"
    severity: "critical"
    
  - name: "Database Slow Queries"
    condition: "db_query_p95 > 1s"
    duration: "2m"
    severity: "warning"
    
  - name: "WebSocket Connection Drop"
    condition: "websocket_connections < 50% of peak"
    duration: "1m"
    severity: "warning"
```

## Troubleshooting Checklist

### Initial Assessment

- [ ] Check application logs for error patterns
- [ ] Verify system resources (CPU, memory, disk)
- [ ] Test basic connectivity (HTTP, WebSocket, database)
- [ ] Review recent configuration changes
- [ ] Check external service status

### Database Issues

- [ ] Verify database file exists and is readable
- [ ] Check disk space availability
- [ ] Test database connectivity
- [ ] Run integrity check
- [ ] Review recent schema changes
- [ ] Check for long-running transactions

### Authentication Problems

- [ ] Test login with known good credentials
- [ ] Check session storage configuration
- [ ] Verify token generation settings
- [ ] Review authentication logs
- [ ] Test password reset functionality

### Performance Issues

- [ ] Check response time metrics
- [ ] Review database query performance
- [ ] Analyze WebSocket connection patterns
- [ ] Monitor resource utilization
- [ ] Check for memory leaks
- [ ] Review caching effectiveness

### Network Connectivity

- [ ] Test HTTP endpoints
- [ ] Verify WebSocket connections
- [ ] Check firewall rules
- [ ] Test DNS resolution
- [ ] Review proxy configuration
- [ ] Validate SSL certificates

## Emergency Contacts

- **Primary Administrator:** [Contact Information]
- **Database Administrator:** [Contact Information]
- **Network Administrator:** [Contact Information]
- **On-Call Engineer:** [Contact Information]

## Recovery Time Objectives (RTO)

- **Database Recovery:** 15 minutes
- **Application Restart:** 2 minutes
- **Configuration Changes:** 5 minutes
- **Full System Recovery:** 30 minutes

## Recovery Point Objectives (RPO)

- **Message Data:** 5 minutes (backup frequency)
- **User Data:** 1 hour (backup frequency)
- **Configuration:** Real-time (version control)

---

**Last Updated:** [Current Date]
**Version:** 1.0
**Next Review:** [Date + 3 months]