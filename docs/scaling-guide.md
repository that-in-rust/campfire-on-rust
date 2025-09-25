# Campfire Rust - Scaling and Performance Guide

This guide covers scaling strategies, performance optimization, and capacity planning for Campfire Rust deployments.

## Table of Contents

1. [Performance Baseline](#performance-baseline)
2. [Vertical Scaling](#vertical-scaling)
3. [Horizontal Scaling](#horizontal-scaling)
4. [Database Optimization](#database-optimization)
5. [Caching Strategies](#caching-strategies)
6. [Load Balancing](#load-balancing)
7. [WebSocket Scaling](#websocket-scaling)
8. [Monitoring and Alerting](#monitoring-and-alerting)
9. [Capacity Planning](#capacity-planning)
10. [Performance Testing](#performance-testing)

## Performance Baseline

### Single Instance Capacity

**Hardware Requirements (Baseline)**:
- **CPU**: 2 cores (4 vCPUs recommended)
- **Memory**: 2GB RAM (4GB recommended)
- **Storage**: 10GB SSD (with growth planning)
- **Network**: 1Gbps connection

**Expected Performance**:
- **Concurrent Users**: 500-1000 active WebSocket connections
- **Messages per Second**: 100-500 messages/second
- **HTTP Requests**: 1000-5000 requests/second
- **Database Operations**: 2000-10000 queries/second
- **Response Time**: <100ms for 95th percentile

### Performance Metrics

Monitor these key metrics:

```bash
# CPU and Memory
campfire_cpu_usage_percent
campfire_memory_usage_bytes

# HTTP Performance
campfire_http_request_duration_seconds
campfire_http_requests_total

# WebSocket Performance
campfire_websocket_connections_active
campfire_websocket_messages_sent_total

# Database Performance
campfire_database_operation_duration_seconds
campfire_database_connections_active
```

## Vertical Scaling

### Resource Optimization

#### CPU Optimization
```bash
# Increase worker threads for CPU-intensive workloads
CAMPFIRE_WORKER_THREADS=8  # Default: auto-detect

# Enable CPU affinity for dedicated servers
taskset -c 0-7 ./campfire-on-rust
```

#### Memory Optimization
```bash
# Increase connection pool size
CAMPFIRE_DB_MAX_CONNECTIONS=50  # Default: 10

# Adjust message buffer sizes
CAMPFIRE_MESSAGE_BUFFER_SIZE=10000  # Default: 1000

# Enable memory-mapped I/O for database
CAMPFIRE_DB_MMAP_SIZE=268435456  # 256MB
```

#### Storage Optimization
```bash
# Enable WAL mode for better concurrency
CAMPFIRE_DB_WAL_MODE=true

# Increase WAL checkpoint interval
CAMPFIRE_DB_WAL_CHECKPOINT_INTERVAL=10000

# Use faster storage (NVMe SSD recommended)
# Mount with optimized options
mount -o noatime,nodiratime /dev/nvme0n1 /app/data
```

### Container Resource Limits

```yaml
# docker-compose.yml
services:
  campfire:
    deploy:
      resources:
        limits:
          memory: 4G
          cpus: '4.0'
        reservations:
          memory: 2G
          cpus: '2.0'
    
    # Optimize for performance
    security_opt:
      - seccomp:unconfined  # Only for performance testing
    
    # Use host networking for maximum performance
    network_mode: host  # Use with caution
```

## Horizontal Scaling

### Multi-Instance Architecture

```mermaid
graph TB
    subgraph "Load Balancer"
        LB[HAProxy/Nginx]
    end
    
    subgraph "Application Tier"
        APP1[Campfire Instance 1]
        APP2[Campfire Instance 2]
        APP3[Campfire Instance 3]
    end
    
    subgraph "Data Tier"
        DB[(PostgreSQL Primary)]
        DB_REPLICA[(PostgreSQL Replica)]
        REDIS[(Redis Cluster)]
    end
    
    subgraph "Storage Tier"
        S3[Object Storage]
        NFS[Shared Storage]
    end
    
    LB --> APP1
    LB --> APP2
    LB --> APP3
    
    APP1 --> DB
    APP2 --> DB
    APP3 --> DB
    
    APP1 --> DB_REPLICA
    APP2 --> DB_REPLICA
    APP3 --> DB_REPLICA
    
    APP1 --> REDIS
    APP2 --> REDIS
    APP3 --> REDIS
    
    APP1 --> S3
    APP2 --> S3
    APP3 --> S3
```

### Database Migration to PostgreSQL

For horizontal scaling, migrate from SQLite to PostgreSQL:

```bash
# Environment variables for PostgreSQL
CAMPFIRE_DATABASE_URL=postgresql://user:pass@postgres:5432/campfire
CAMPFIRE_DB_MAX_CONNECTIONS=100
CAMPFIRE_DB_POOL_TIMEOUT=30
```

#### PostgreSQL Configuration

```sql
-- postgresql.conf optimizations
shared_buffers = 1GB
effective_cache_size = 3GB
maintenance_work_mem = 256MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
```

### Session Storage with Redis

```bash
# Redis configuration for session storage
CAMPFIRE_SESSION_STORE=redis
CAMPFIRE_REDIS_URL=redis://redis-cluster:6379
CAMPFIRE_SESSION_TTL=86400  # 24 hours
```

#### Redis Cluster Setup

```yaml
# docker-compose.yml
redis-cluster:
  image: redis:7-alpine
  command: redis-server --cluster-enabled yes --cluster-config-file nodes.conf
  ports:
    - "6379:6379"
  volumes:
    - redis_data:/data
```

## Database Optimization

### SQLite Optimizations (Single Instance)

```sql
-- Enable WAL mode for better concurrency
PRAGMA journal_mode = WAL;

-- Increase cache size (in pages, default page size is 4KB)
PRAGMA cache_size = 10000;  -- 40MB cache

-- Optimize for performance
PRAGMA synchronous = NORMAL;
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456;  -- 256MB

-- Analyze tables for better query planning
ANALYZE;
```

### PostgreSQL Optimizations

```sql
-- Create indexes for common queries
CREATE INDEX CONCURRENTLY idx_messages_room_created 
ON messages(room_id, created_at DESC);

CREATE INDEX CONCURRENTLY idx_messages_creator_created 
ON messages(creator_id, created_at DESC);

CREATE INDEX CONCURRENTLY idx_room_memberships_user 
ON room_memberships(user_id);

-- Optimize full-text search
CREATE INDEX CONCURRENTLY idx_messages_fts 
ON messages USING gin(to_tsvector('english', content));

-- Partition large tables by date
CREATE TABLE messages_2024_01 PARTITION OF messages
FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
```

### Database Connection Pooling

```rust
// Enhanced connection pool configuration
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(100)
    .min_connections(10)
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(&database_url)
    .await?;
```

## Caching Strategies

### Application-Level Caching

```rust
use moka::future::Cache;
use std::time::Duration;

// User cache
let user_cache = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(300))  // 5 minutes
    .build();

// Room membership cache
let membership_cache = Cache::builder()
    .max_capacity(50_000)
    .time_to_live(Duration::from_secs(600))  // 10 minutes
    .build();

// Message cache for recent messages
let message_cache = Cache::builder()
    .max_capacity(100_000)
    .time_to_live(Duration::from_secs(60))   // 1 minute
    .build();
```

### Redis Caching Layer

```bash
# Redis configuration
CAMPFIRE_CACHE_ENABLED=true
CAMPFIRE_CACHE_URL=redis://redis:6379
CAMPFIRE_CACHE_TTL_USERS=300
CAMPFIRE_CACHE_TTL_ROOMS=600
CAMPFIRE_CACHE_TTL_MESSAGES=60
```

### CDN for Static Assets

```yaml
# Use CDN for static assets
services:
  campfire:
    environment:
      - CAMPFIRE_CDN_URL=https://cdn.example.com
      - CAMPFIRE_STATIC_CACHE_TTL=86400  # 24 hours
```

## Load Balancing

### HAProxy Configuration

```haproxy
# /etc/haproxy/haproxy.cfg
global
    daemon
    maxconn 4096

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms

frontend campfire_frontend
    bind *:80
    bind *:443 ssl crt /etc/ssl/certs/campfire.pem
    redirect scheme https if !{ ssl_fc }
    
    # WebSocket detection
    acl is_websocket hdr(Upgrade) -i websocket
    use_backend campfire_websocket if is_websocket
    default_backend campfire_http

backend campfire_http
    balance roundrobin
    option httpchk GET /health
    server app1 campfire1:3000 check
    server app2 campfire2:3000 check
    server app3 campfire3:3000 check

backend campfire_websocket
    balance source  # Sticky sessions for WebSocket
    option httpchk GET /health
    server app1 campfire1:3000 check
    server app2 campfire2:3000 check
    server app3 campfire3:3000 check
```

### Nginx Configuration

```nginx
# /etc/nginx/nginx.conf
upstream campfire_backend {
    least_conn;
    server campfire1:3000 max_fails=3 fail_timeout=30s;
    server campfire2:3000 max_fails=3 fail_timeout=30s;
    server campfire3:3000 max_fails=3 fail_timeout=30s;
}

upstream campfire_websocket {
    ip_hash;  # Sticky sessions for WebSocket
    server campfire1:3000;
    server campfire2:3000;
    server campfire3:3000;
}

server {
    listen 80;
    listen 443 ssl http2;
    server_name campfire.example.com;
    
    # SSL configuration
    ssl_certificate /etc/ssl/certs/campfire.crt;
    ssl_certificate_key /etc/ssl/private/campfire.key;
    
    # WebSocket upgrade
    location /ws {
        proxy_pass http://campfire_websocket;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket timeouts
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
    }
    
    # HTTP requests
    location / {
        proxy_pass http://campfire_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Caching for static assets
        location ~* \.(css|js|png|jpg|jpeg|gif|ico|svg)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }
}
```

## WebSocket Scaling

### Sticky Sessions

For WebSocket connections, use sticky sessions to ensure connections stay with the same instance:

```bash
# HAProxy with source IP hashing
balance source

# Nginx with IP hash
ip_hash;

# Application-level session affinity
CAMPFIRE_WEBSOCKET_AFFINITY=true
```

### WebSocket Clustering with Redis

```rust
// WebSocket message broadcasting across instances
use redis::AsyncCommands;

pub struct ClusteredWebSocketManager {
    local_connections: Arc<RwLock<HashMap<UserId, Vec<WebSocketSender>>>>,
    redis: redis::Client,
}

impl ClusteredWebSocketManager {
    pub async fn broadcast_to_room(&self, room_id: RoomId, message: &str) -> Result<()> {
        // Broadcast to local connections
        self.broadcast_local(room_id, message).await?;
        
        // Publish to Redis for other instances
        let mut conn = self.redis.get_async_connection().await?;
        let channel = format!("campfire:room:{}", room_id);
        conn.publish(channel, message).await?;
        
        Ok(())
    }
}
```

### Connection Limits

```bash
# Per-instance connection limits
CAMPFIRE_MAX_WEBSOCKET_CONNECTIONS=5000

# System-wide limits
ulimit -n 65536  # File descriptors
echo 'net.core.somaxconn = 65536' >> /etc/sysctl.conf
```

## Monitoring and Alerting

### Performance Monitoring

```yaml
# Prometheus alerting rules for scaling
groups:
  - name: campfire.scaling
    rules:
      # High CPU usage
      - alert: HighCPUUsage
        expr: campfire_cpu_usage_percent > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage detected"
          description: "CPU usage is {{ $value }}% for 5 minutes"

      # High memory usage
      - alert: HighMemoryUsage
        expr: (campfire_memory_usage_bytes / (1024*1024*1024)) > 3.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}GB"

      # High connection count
      - alert: HighConnectionCount
        expr: campfire_websocket_connections_active > 4000
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High WebSocket connection count"
          description: "{{ $value }} active WebSocket connections"

      # Slow response times
      - alert: SlowResponseTimes
        expr: histogram_quantile(0.95, rate(campfire_http_request_duration_seconds_bucket[5m])) > 0.5
        for: 3m
        labels:
          severity: warning
        annotations:
          summary: "Slow response times"
          description: "95th percentile response time is {{ $value }}s"
```

### Auto-Scaling with Docker Swarm

```yaml
# docker-compose.yml for Docker Swarm
version: '3.8'

services:
  campfire:
    image: campfire-on-rust:latest
    deploy:
      replicas: 3
      update_config:
        parallelism: 1
        delay: 10s
      restart_policy:
        condition: on-failure
      resources:
        limits:
          memory: 2G
          cpus: '2.0'
        reservations:
          memory: 1G
          cpus: '1.0'
    
    # Health check for auto-scaling
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## Capacity Planning

### Growth Projections

| Metric | Small | Medium | Large | Enterprise |
|--------|-------|--------|-------|------------|
| **Users** | 100 | 1,000 | 10,000 | 100,000+ |
| **Concurrent** | 50 | 500 | 5,000 | 50,000+ |
| **Messages/day** | 10K | 100K | 1M | 10M+ |
| **Storage/month** | 1GB | 10GB | 100GB | 1TB+ |
| **Instances** | 1 | 2-3 | 5-10 | 20+ |

### Resource Planning

#### Small Deployment (100 users)
```yaml
resources:
  cpu: 1 core
  memory: 1GB
  storage: 10GB
  instances: 1
  database: SQLite
```

#### Medium Deployment (1,000 users)
```yaml
resources:
  cpu: 2-4 cores per instance
  memory: 2-4GB per instance
  storage: 50GB
  instances: 2-3
  database: PostgreSQL
  cache: Redis (optional)
```

#### Large Deployment (10,000 users)
```yaml
resources:
  cpu: 4-8 cores per instance
  memory: 4-8GB per instance
  storage: 500GB
  instances: 5-10
  database: PostgreSQL with replicas
  cache: Redis cluster
  cdn: Required
```

#### Enterprise Deployment (100,000+ users)
```yaml
resources:
  cpu: 8+ cores per instance
  memory: 8-16GB per instance
  storage: 5TB+
  instances: 20+
  database: PostgreSQL cluster with sharding
  cache: Redis cluster with multiple nodes
  cdn: Required with edge locations
  monitoring: Comprehensive observability stack
```

## Performance Testing

### Load Testing with Artillery

```yaml
# artillery-config.yml
config:
  target: 'http://localhost:3000'
  phases:
    - duration: 60
      arrivalRate: 10
      name: "Warm up"
    - duration: 300
      arrivalRate: 50
      name: "Sustained load"
    - duration: 60
      arrivalRate: 100
      name: "Peak load"

scenarios:
  - name: "HTTP API Load Test"
    weight: 70
    flow:
      - get:
          url: "/health"
      - post:
          url: "/api/messages"
          json:
            content: "Load test message {{ $randomString() }}"
            room_id: "{{ $randomUUID() }}"

  - name: "WebSocket Load Test"
    weight: 30
    engine: ws
    flow:
      - connect:
          url: "ws://localhost:3000/ws"
      - send:
          payload: '{"type": "message", "content": "WebSocket test"}'
      - think: 5
```

### Stress Testing Script

```bash
#!/bin/bash
# stress-test.sh

# Test HTTP endpoints
echo "Testing HTTP performance..."
ab -n 10000 -c 100 http://localhost:3000/health

# Test WebSocket connections
echo "Testing WebSocket performance..."
wscat -c ws://localhost:3000/ws &
WSPID=$!

# Monitor system resources during test
echo "Monitoring system resources..."
iostat -x 1 60 > iostat.log &
vmstat 1 60 > vmstat.log &

# Wait for tests to complete
sleep 60

# Cleanup
kill $WSPID
pkill iostat
pkill vmstat

echo "Stress test completed. Check logs for results."
```

### Performance Benchmarking

```bash
# Benchmark database operations
echo "Benchmarking database..."
sqlite3 campfire.db ".timer on" "SELECT COUNT(*) FROM messages;"

# Benchmark WebSocket throughput
echo "Benchmarking WebSocket throughput..."
node websocket-benchmark.js

# Benchmark memory usage
echo "Benchmarking memory usage..."
valgrind --tool=massif ./campfire-on-rust

# Generate performance report
echo "Generating performance report..."
./scripts/performance-report.sh
```

## Optimization Checklist

### Application Level
- [ ] Enable connection pooling
- [ ] Implement caching for frequently accessed data
- [ ] Optimize database queries with proper indexes
- [ ] Use async/await for I/O operations
- [ ] Implement rate limiting
- [ ] Enable compression for HTTP responses

### Database Level
- [ ] Enable WAL mode for SQLite or use PostgreSQL
- [ ] Create appropriate indexes
- [ ] Optimize query performance
- [ ] Implement connection pooling
- [ ] Consider read replicas for read-heavy workloads

### Infrastructure Level
- [ ] Use SSD storage
- [ ] Implement load balancing
- [ ] Set up CDN for static assets
- [ ] Configure proper caching headers
- [ ] Monitor resource usage
- [ ] Set up auto-scaling

### Network Level
- [ ] Use HTTP/2
- [ ] Enable gzip compression
- [ ] Implement proper SSL/TLS
- [ ] Optimize WebSocket connections
- [ ] Use sticky sessions for WebSocket

This scaling guide provides a comprehensive approach to growing your Campfire deployment from a single instance to enterprise scale while maintaining performance and reliability.