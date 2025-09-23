# Campfire Rust Scaling Guide

## Overview

This guide provides comprehensive strategies for scaling Campfire Rust from a single-instance deployment to high-availability, multi-instance production environments. The scaling approach follows the **MVP-First Rigor** pattern, starting with proven single-instance optimizations before adding distributed complexity.

## Core Scaling Philosophy

**Single-Instance First**: Maximize single-instance performance using Rust's zero-cost abstractions and memory safety before considering distributed solutions.

**Evidence-Based Scaling**: Add complexity only when measurements prove it's necessary, following Rails-equivalent patterns adapted to Rust's async ecosystem.

**Rust Performance Advantages**: Leverage Rust's natural performance benefits - memory safety without garbage collection, zero-cost abstractions, and fearless concurrency.

## Architecture Overview

```mermaid
graph TD
    subgraph "Single Instance (MVP)"
        direction TB
        LB1[Load Balancer<br/>nginx/Caddy]
        APP1[Campfire Rust<br/>Single Binary]
        DB1[(SQLite<br/>WAL Mode)]
        
        LB1 --> APP1
        APP1 --> DB1
    end
    
    subgraph "Multi-Instance (Scale-Out)"
        direction TB
        LB2[Load Balancer<br/>Session Affinity]
        APP2[Campfire Instance 1]
        APP3[Campfire Instance 2]
        APP4[Campfire Instance N]
        REDIS[(Redis<br/>Session Store)]
        PG[(PostgreSQL<br/>Primary DB)]
        
        LB2 --> APP2
        LB2 --> APP3
        LB2 --> APP4
        APP2 --> REDIS
        APP3 --> REDIS
        APP4 --> REDIS
        APP2 --> PG
        APP3 --> PG
        APP4 --> PG
    end
```

## Single-Instance Optimization Strategies

### 1. SQLite Performance Tuning

**WAL Mode Configuration**:
```rust
// src/database/mod.rs
pub async fn configure_sqlite_performance(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    // Enable WAL mode for better concurrency
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(conn)
        .await?;
    
    // Optimize for performance
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(conn)
        .await?;
    
    // Increase cache size (in KB)
    sqlx::query("PRAGMA cache_size = -64000") // 64MB cache
        .execute(conn)
        .await?;
    
    // Optimize temp storage
    sqlx::query("PRAGMA temp_store = MEMORY")
        .execute(conn)
        .await?;
    
    Ok(())
}
```*
*Connection Pool Optimization**:
```rust
// src/config.rs
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,      // Optimal for SQLite
            min_connections: 5,       // Keep warm connections
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),    // 10 minutes
            max_lifetime: Duration::from_secs(1800),   // 30 minutes
        }
    }
}

pub async fn create_optimized_pool(config: &DatabaseConfig) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .connect(&database_url)
        .await
}
```

### 2. Memory Management Optimization

**Connection State Management**:
```rust
// src/services/connection.rs
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct OptimizedConnectionManager {
    // Use RwLock for read-heavy workloads
    connections: Arc<RwLock<HashMap<ConnectionId, ConnectionInfo>>>,
    // Separate presence tracking for better cache locality
    presence: Arc<RwLock<HashMap<RoomId, HashSet<UserId>>>>,
    // Connection cleanup task handle
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl OptimizedConnectionManager {
    pub async fn get_room_connections(&self, room_id: RoomId) -> Vec<ConnectionId> {
        let connections = self.connections.read().await;
        connections
            .iter()
            .filter_map(|(conn_id, info)| {
                if info.rooms.contains(&room_id) {
                    Some(*conn_id)
                } else {
                    None
                }
            })
            .collect()
    }
    
    // Batch operations for better performance
    pub async fn broadcast_to_multiple_rooms(
        &self,
        room_ids: &[RoomId],
        message: &WebSocketMessage,
    ) -> Result<(), BroadcastError> {
        let connections = self.connections.read().await;
        let mut target_connections = HashSet::new();
        
        // Collect all unique connections across rooms
        for room_id in room_ids {
            for (conn_id, info) in connections.iter() {
                if info.rooms.contains(room_id) {
                    target_connections.insert(*conn_id);
                }
            }
        }
        
        // Broadcast to unique connections only once
        self.send_to_connections(target_connections.into_iter().collect(), message).await
    }
}
```### 3. 
WebSocket Performance Optimization

**Efficient Message Broadcasting**:
```rust
// src/handlers/websocket.rs
use tokio::sync::broadcast;

pub struct HighPerformanceWebSocket {
    // Use broadcast channel for efficient fan-out
    room_channels: Arc<RwLock<HashMap<RoomId, broadcast::Sender<WebSocketMessage>>>>,
    // Connection-specific senders
    connection_senders: Arc<RwLock<HashMap<ConnectionId, mpsc::UnboundedSender<WebSocketMessage>>>>,
}

impl HighPerformanceWebSocket {
    pub async fn broadcast_to_room(&self, room_id: RoomId, message: WebSocketMessage) -> Result<(), BroadcastError> {
        let channels = self.room_channels.read().await;
        
        if let Some(sender) = channels.get(&room_id) {
            // Broadcast channel automatically handles multiple subscribers
            match sender.send(message) {
                Ok(_) => Ok(()),
                Err(broadcast::error::SendError(_)) => {
                    // No active receivers, clean up channel
                    drop(channels);
                    let mut channels = self.room_channels.write().await;
                    channels.remove(&room_id);
                    Ok(())
                }
            }
        } else {
            // No active connections for this room
            Ok(())
        }
    }
    
    // Efficient connection management
    pub async fn add_connection_to_room(
        &self,
        connection_id: ConnectionId,
        room_id: RoomId,
        sender: mpsc::UnboundedSender<WebSocketMessage>,
    ) -> Result<(), ConnectionError> {
        // Add to connection senders
        {
            let mut senders = self.connection_senders.write().await;
            senders.insert(connection_id, sender.clone());
        }
        
        // Subscribe to room broadcast channel
        let mut channels = self.room_channels.write().await;
        let room_sender = channels.entry(room_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(1000); // Buffer for 1000 messages
            tx
        });
        
        let mut receiver = room_sender.subscribe();
        
        // Spawn task to forward room messages to connection
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                if sender.send(message).is_err() {
                    // Connection closed, exit task
                    break;
                }
            }
        });
        
        Ok(())
    }
}
```

### 4. CPU and Memory Profiling

**Performance Monitoring Integration**:
```rust
// src/metrics.rs
use prometheus::{Counter, Histogram, Gauge, register_counter, register_histogram, register_gauge};

lazy_static! {
    static ref MESSAGE_PROCESSING_TIME: Histogram = register_histogram!(
        "campfire_message_processing_seconds",
        "Time spent processing messages"
    ).unwrap();
    
    static ref ACTIVE_CONNECTIONS: Gauge = register_gauge!(
        "campfire_active_connections",
        "Number of active WebSocket connections"
    ).unwrap();
    
    static ref MEMORY_USAGE: Gauge = register_gauge!(
        "campfire_memory_usage_bytes",
        "Current memory usage in bytes"
    ).unwrap();
    
    static ref DATABASE_QUERY_TIME: Histogram = register_histogram!(
        "campfire_database_query_seconds",
        "Database query execution time"
    ).unwrap();
}

pub fn record_message_processing_time<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let timer = MESSAGE_PROCESSING_TIME.start_timer();
    let result = f();
    timer.observe_duration();
    result
}

pub async fn update_system_metrics() {
    // Update memory usage
    if let Ok(usage) = get_memory_usage().await {
        MEMORY_USAGE.set(usage as f64);
    }
    
    // Update connection count
    let conn_count = get_active_connection_count().await;
    ACTIVE_CONNECTIONS.set(conn_count as f64);
}
```## Multi-I
nstance Deployment Considerations

### 1. Session Management Strategy

**Redis-Based Session Store**:
```rust
// src/services/session.rs
use redis::AsyncCommands;

#[derive(Clone)]
pub struct DistributedSessionStore {
    redis: redis::Client,
    session_ttl: Duration,
}

impl DistributedSessionStore {
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            redis: client,
            session_ttl: Duration::from_secs(86400), // 24 hours
        })
    }
    
    pub async fn store_session(&self, token: &str, user_id: UserId) -> Result<(), SessionError> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("session:{}", token);
        let value = serde_json::to_string(&SessionData {
            user_id,
            created_at: Utc::now(),
        })?;
        
        conn.setex(key, self.session_ttl.as_secs(), value).await?;
        Ok(())
    }
    
    pub async fn get_session(&self, token: &str) -> Result<Option<SessionData>, SessionError> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("session:{}", token);
        
        let value: Option<String> = conn.get(key).await?;
        match value {
            Some(data) => {
                let session: SessionData = serde_json::from_str(&data)?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }
    
    pub async fn invalidate_session(&self, token: &str) -> Result<(), SessionError> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("session:{}", token);
        conn.del(key).await?;
        Ok(())
    }
}
```

### 2. Database Migration Strategy

**PostgreSQL Migration Path**:
```rust
// src/database/postgres.rs
use sqlx::postgres::PgPoolOptions;

pub struct PostgreSQLConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl PostgreSQLConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

pub async fn create_postgres_pool(config: &PostgreSQLConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&config.connection_string())
        .await
}

// Migration compatibility layer
pub trait DatabaseProvider: Send + Sync {
    async fn execute_query(&self, query: &str) -> Result<QueryResult, DatabaseError>;
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, DatabaseError>;
}

pub struct SQLiteProvider {
    pool: SqlitePool,
}

pub struct PostgreSQLProvider {
    pool: PgPool,
}

// Implement DatabaseProvider for both backends
impl DatabaseProvider for SQLiteProvider {
    async fn execute_query(&self, query: &str) -> Result<QueryResult, DatabaseError> {
        // SQLite implementation
        todo!()
    }
    
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, DatabaseError> {
        // SQLite transaction
        todo!()
    }
}

impl DatabaseProvider for PostgreSQLProvider {
    async fn execute_query(&self, query: &str) -> Result<QueryResult, DatabaseError> {
        // PostgreSQL implementation
        todo!()
    }
    
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, DatabaseError> {
        // PostgreSQL transaction
        todo!()
    }
}
```### 
3. WebSocket State Synchronization

**Redis Pub/Sub for Cross-Instance Communication**:
```rust
// src/services/distributed_websocket.rs
use redis::AsyncCommands;
use tokio::sync::broadcast;

pub struct DistributedWebSocketManager {
    local_connections: Arc<RwLock<HashMap<ConnectionId, ConnectionInfo>>>,
    redis_client: redis::Client,
    message_publisher: broadcast::Sender<CrossInstanceMessage>,
    instance_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossInstanceMessage {
    pub instance_id: String,
    pub room_id: RoomId,
    pub message: WebSocketMessage,
    pub exclude_instance: Option<String>,
}

impl DistributedWebSocketManager {
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let redis_client = redis::Client::open(redis_url)?;
        let (tx, _) = broadcast::channel(10000);
        let instance_id = format!("campfire-{}", Uuid::new_v4());
        
        let manager = Self {
            local_connections: Arc::new(RwLock::new(HashMap::new())),
            redis_client,
            message_publisher: tx,
            instance_id,
        };
        
        // Start Redis subscriber task
        manager.start_redis_subscriber().await?;
        
        Ok(manager)
    }
    
    pub async fn broadcast_to_room(&self, room_id: RoomId, message: WebSocketMessage) -> Result<(), BroadcastError> {
        // Broadcast locally first
        self.broadcast_locally(room_id, &message).await?;
        
        // Publish to other instances via Redis
        let cross_instance_msg = CrossInstanceMessage {
            instance_id: self.instance_id.clone(),
            room_id,
            message,
            exclude_instance: Some(self.instance_id.clone()),
        };
        
        self.publish_to_redis(&cross_instance_msg).await?;
        Ok(())
    }
    
    async fn broadcast_locally(&self, room_id: RoomId, message: &WebSocketMessage) -> Result<(), BroadcastError> {
        let connections = self.local_connections.read().await;
        let mut tasks = Vec::new();
        
        for (conn_id, info) in connections.iter() {
            if info.rooms.contains(&room_id) {
                let sender = info.sender.clone();
                let msg = message.clone();
                tasks.push(tokio::spawn(async move {
                    sender.send(msg).map_err(|_| BroadcastError::SendFailed)
                }));
            }
        }
        
        // Wait for all sends to complete
        for task in tasks {
            let _ = task.await;
        }
        
        Ok(())
    }
    
    async fn publish_to_redis(&self, message: &CrossInstanceMessage) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let serialized = serde_json::to_string(message)?;
        conn.publish("campfire:websocket", serialized).await?;
        Ok(())
    }
    
    async fn start_redis_subscriber(&self) -> Result<(), redis::RedisError> {
        let mut pubsub = self.redis_client.get_async_connection().await?.into_pubsub();
        pubsub.subscribe("campfire:websocket").await?;
        
        let local_connections = Arc::clone(&self.local_connections);
        let instance_id = self.instance_id.clone();
        
        tokio::spawn(async move {
            while let Ok(msg) = pubsub.on_message().next().await {
                if let Ok(payload) = msg.get_payload::<String>() {
                    if let Ok(cross_msg) = serde_json::from_str::<CrossInstanceMessage>(&payload) {
                        // Skip messages from this instance
                        if cross_msg.exclude_instance.as_ref() == Some(&instance_id) {
                            continue;
                        }
                        
                        // Broadcast to local connections
                        let connections = local_connections.read().await;
                        for (_, info) in connections.iter() {
                            if info.rooms.contains(&cross_msg.room_id) {
                                let _ = info.sender.send(cross_msg.message.clone());
                            }
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
}
```## Loa
d Balancing and Session Sharing Strategies

### 1. Load Balancer Configuration

**Nginx Configuration for WebSocket Support**:
```nginx
# /etc/nginx/sites-available/campfire
upstream campfire_backend {
    # Use IP hash for session affinity
    ip_hash;
    
    server 127.0.0.1:3000 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3001 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3002 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    server_name campfire.example.com;
    
    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name campfire.example.com;
    
    ssl_certificate /path/to/certificate.crt;
    ssl_certificate_key /path/to/private.key;
    
    # WebSocket upgrade support
    location /ws {
        proxy_pass http://campfire_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket timeout settings
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
    }
    
    # API and static content
    location / {
        proxy_pass http://campfire_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Enable caching for static assets
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }
    
    # Health check endpoint
    location /health {
        proxy_pass http://campfire_backend;
        access_log off;
    }
}
```

**HAProxy Configuration Alternative**:
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
    bind *:443 ssl crt /path/to/certificate.pem
    redirect scheme https if !{ ssl_fc }
    
    # WebSocket detection
    acl is_websocket hdr(Upgrade) -i websocket
    acl is_websocket_path path_beg /ws
    
    use_backend campfire_websocket if is_websocket or is_websocket_path
    default_backend campfire_http
    
backend campfire_http
    balance roundrobin
    option httpchk GET /health
    
    server campfire1 127.0.0.1:3000 check
    server campfire2 127.0.0.1:3001 check
    server campfire3 127.0.0.1:3002 check
    
backend campfire_websocket
    balance source  # Session affinity for WebSockets
    option httpchk GET /health
    
    server campfire1 127.0.0.1:3000 check
    server campfire2 127.0.0.1:3001 check
    server campfire3 127.0.0.1:3002 check
```

### 2. Session Affinity Strategies

**Consistent Hashing for WebSocket Connections**:
```rust
// src/services/load_balancer.rs
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct ConsistentHashRouter {
    instances: Vec<InstanceInfo>,
    virtual_nodes: HashMap<u64, usize>, // hash -> instance_index
}

#[derive(Debug, Clone)]
pub struct InstanceInfo {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub weight: u32,
}

impl ConsistentHashRouter {
    pub fn new(instances: Vec<InstanceInfo>) -> Self {
        let mut virtual_nodes = HashMap::new();
        
        for (index, instance) in instances.iter().enumerate() {
            // Create virtual nodes based on weight
            for i in 0..(instance.weight * 100) {
                let virtual_key = format!("{}:{}", instance.id, i);
                let mut hasher = DefaultHasher::new();
                virtual_key.hash(&mut hasher);
                let hash = hasher.finish();
                virtual_nodes.insert(hash, index);
            }
        }
        
        Self {
            instances,
            virtual_nodes,
        }
    }
    
    pub fn route_connection(&self, user_id: UserId) -> Option<&InstanceInfo> {
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Find the next virtual node
        let mut keys: Vec<_> = self.virtual_nodes.keys().collect();
        keys.sort();
        
        for &key in keys.iter() {
            if *key >= hash {
                let instance_index = self.virtual_nodes[key];
                return Some(&self.instances[instance_index]);
            }
        }
        
        // Wrap around to first node
        if let Some(&first_key) = keys.first() {
            let instance_index = self.virtual_nodes[first_key];
            Some(&self.instances[instance_index])
        } else {
            None
        }
    }
}
```#
# Capacity Planning and Resource Monitoring

### 1. Performance Benchmarking

**Load Testing Configuration**:
```rust
// tests/load_test.rs
use tokio::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[tokio::test]
async fn benchmark_concurrent_connections() {
    let base_url = "ws://localhost:3000/ws";
    let concurrent_connections = 1000;
    let messages_per_connection = 100;
    let test_duration = Duration::from_secs(60);
    
    let success_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));
    let total_latency = Arc::new(AtomicU64::new(0));
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for i in 0..concurrent_connections {
        let url = base_url.to_string();
        let success_count = Arc::clone(&success_count);
        let error_count = Arc::clone(&error_count);
        let total_latency = Arc::clone(&total_latency);
        
        let handle = tokio::spawn(async move {
            match simulate_user_session(&url, messages_per_connection).await {
                Ok(avg_latency) => {
                    success_count.fetch_add(1, Ordering::Relaxed);
                    total_latency.fetch_add(avg_latency.as_millis() as u64, Ordering::Relaxed);
                }
                Err(_) => {
                    error_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        
        handles.push(handle);
        
        // Stagger connection attempts
        if i % 10 == 0 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    // Wait for all connections to complete
    for handle in handles {
        let _ = handle.await;
    }
    
    let elapsed = start_time.elapsed();
    let successful = success_count.load(Ordering::Relaxed);
    let failed = error_count.load(Ordering::Relaxed);
    let avg_latency = if successful > 0 {
        total_latency.load(Ordering::Relaxed) / successful
    } else {
        0
    };
    
    println!("Load Test Results:");
    println!("  Duration: {:?}", elapsed);
    println!("  Successful connections: {}", successful);
    println!("  Failed connections: {}", failed);
    println!("  Success rate: {:.2}%", (successful as f64 / concurrent_connections as f64) * 100.0);
    println!("  Average latency: {}ms", avg_latency);
    println!("  Connections per second: {:.2}", concurrent_connections as f64 / elapsed.as_secs_f64());
    
    // Assert performance requirements
    assert!(successful >= (concurrent_connections * 95 / 100), "Success rate below 95%");
    assert!(avg_latency < 100, "Average latency above 100ms");
}

async fn simulate_user_session(url: &str, message_count: u32) -> Result<Duration, Box<dyn std::error::Error>> {
    let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    let start_time = Instant::now();
    let mut total_latency = Duration::ZERO;
    
    for i in 0..message_count {
        let message_start = Instant::now();
        
        // Send message
        let test_message = format!("Test message {} from load test", i);
        ws_sender.send(Message::Text(test_message)).await?;
        
        // Wait for response (echo or broadcast)
        if let Some(response) = ws_receiver.next().await {
            let _ = response?;
            total_latency += message_start.elapsed();
        }
        
        // Small delay between messages
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    Ok(total_latency / message_count)
}
```

### 2. Resource Monitoring Setup

**Prometheus Metrics Configuration**:
```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "campfire_rules.yml"

scrape_configs:
  - job_name: 'campfire'
    static_configs:
      - targets: ['localhost:3000', 'localhost:3001', 'localhost:3002']
    metrics_path: '/metrics'
    scrape_interval: 5s
    
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['localhost:9100']
      
  - job_name: 'redis'
    static_configs:
      - targets: ['localhost:9121']
      
  - job_name: 'postgres'
    static_configs:
      - targets: ['localhost:9187']

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

**Grafana Dashboard Configuration**:
```json
{
  "dashboard": {
    "title": "Campfire Rust Performance",
    "panels": [
      {
        "title": "Active WebSocket Connections",
        "type": "graph",
        "targets": [
          {
            "expr": "campfire_active_connections",
            "legendFormat": "{{instance}}"
          }
        ]
      },
      {
        "title": "Message Processing Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(campfire_messages_processed_total[5m])",
            "legendFormat": "Messages/sec - {{instance}}"
          }
        ]
      },
      {
        "title": "Database Query Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, campfire_database_query_seconds)",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "histogram_quantile(0.50, campfire_database_query_seconds)",
            "legendFormat": "50th percentile"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "campfire_memory_usage_bytes / 1024 / 1024",
            "legendFormat": "Memory MB - {{instance}}"
          }
        ]
      }
    ]
  }
}
```##
# 3. Capacity Planning Guidelines

**Resource Requirements by Scale**:

```mermaid
graph TD
    subgraph "Small Scale (1-100 users)"
        S1[Single Instance<br/>2 CPU, 4GB RAM]
        S2[SQLite Database<br/>SSD Storage]
        S3[~50 concurrent connections<br/>~1000 messages/hour]
    end
    
    subgraph "Medium Scale (100-1000 users)"
        M1[2-3 Instances<br/>4 CPU, 8GB RAM each]
        M2[PostgreSQL<br/>Dedicated server]
        M3[Redis Session Store<br/>2GB RAM]
        M4[~500 concurrent connections<br/>~10000 messages/hour]
    end
    
    subgraph "Large Scale (1000+ users)"
        L1[5+ Instances<br/>8 CPU, 16GB RAM each]
        L2[PostgreSQL Cluster<br/>Primary + Replicas]
        L3[Redis Cluster<br/>High Availability]
        L4[CDN for Static Assets<br/>Global Distribution]
        L5[~2000+ concurrent connections<br/>~50000+ messages/hour]
    end
```

**Performance Targets by Scale**:

| Scale | Users | Concurrent Connections | Messages/Hour | Response Time | Uptime |
|-------|-------|----------------------|---------------|---------------|---------|
| Small | 1-100 | 50 | 1,000 | <100ms | 99.0% |
| Medium | 100-1,000 | 500 | 10,000 | <50ms | 99.5% |
| Large | 1,000+ | 2,000+ | 50,000+ | <25ms | 99.9% |

**Resource Monitoring Thresholds**:
```rust
// src/monitoring/thresholds.rs
pub struct PerformanceThresholds {
    pub max_cpu_usage: f64,           // 80%
    pub max_memory_usage: f64,        // 85%
    pub max_connection_count: u32,    // Based on scale
    pub max_response_time_ms: u64,    // Based on scale
    pub min_success_rate: f64,        // 99.5%
}

impl PerformanceThresholds {
    pub fn for_scale(scale: Scale) -> Self {
        match scale {
            Scale::Small => Self {
                max_cpu_usage: 0.8,
                max_memory_usage: 0.85,
                max_connection_count: 100,
                max_response_time_ms: 100,
                min_success_rate: 0.99,
            },
            Scale::Medium => Self {
                max_cpu_usage: 0.75,
                max_memory_usage: 0.80,
                max_connection_count: 1000,
                max_response_time_ms: 50,
                min_success_rate: 0.995,
            },
            Scale::Large => Self {
                max_cpu_usage: 0.70,
                max_memory_usage: 0.75,
                max_connection_count: 5000,
                max_response_time_ms: 25,
                min_success_rate: 0.999,
            },
        }
    }
}

pub async fn check_performance_thresholds(
    metrics: &SystemMetrics,
    thresholds: &PerformanceThresholds,
) -> Vec<PerformanceAlert> {
    let mut alerts = Vec::new();
    
    if metrics.cpu_usage > thresholds.max_cpu_usage {
        alerts.push(PerformanceAlert::HighCpuUsage {
            current: metrics.cpu_usage,
            threshold: thresholds.max_cpu_usage,
        });
    }
    
    if metrics.memory_usage > thresholds.max_memory_usage {
        alerts.push(PerformanceAlert::HighMemoryUsage {
            current: metrics.memory_usage,
            threshold: thresholds.max_memory_usage,
        });
    }
    
    if metrics.active_connections > thresholds.max_connection_count {
        alerts.push(PerformanceAlert::TooManyConnections {
            current: metrics.active_connections,
            threshold: thresholds.max_connection_count,
        });
    }
    
    if metrics.avg_response_time_ms > thresholds.max_response_time_ms {
        alerts.push(PerformanceAlert::SlowResponse {
            current: metrics.avg_response_time_ms,
            threshold: thresholds.max_response_time_ms,
        });
    }
    
    alerts
}
```

## Deployment Automation

### 1. Docker Compose for Multi-Instance Setup

**Production Docker Compose**:
```yaml
# docker-compose.production.yml
version: '3.8'

services:
  campfire-1:
    build: .
    environment:
      - INSTANCE_ID=campfire-1
      - DATABASE_URL=postgresql://user:pass@postgres:5432/campfire
      - REDIS_URL=redis://redis:6379
      - PORT=3000
    ports:
      - "3000:3000"
    depends_on:
      - postgres
      - redis
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 4G
        reservations:
          cpus: '1.0'
          memory: 2G
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      
  campfire-2:
    build: .
    environment:
      - INSTANCE_ID=campfire-2
      - DATABASE_URL=postgresql://user:pass@postgres:5432/campfire
      - REDIS_URL=redis://redis:6379
      - PORT=3000
    ports:
      - "3001:3000"
    depends_on:
      - postgres
      - redis
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 4G
        reservations:
          cpus: '1.0'
          memory: 2G
          
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - campfire-1
      - campfire-2
      
  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=campfire
      - POSTGRES_USER=campfire_user
      - POSTGRES_PASSWORD=secure_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 8G
        reservations:
          cpus: '2.0'
          memory: 4G
          
  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 2G
        reservations:
          cpus: '0.5'
          memory: 1G
          
  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
      
  grafana:
    image: grafana/grafana
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana:/etc/grafana/provisioning

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
  grafana_data:
```##
# 2. Kubernetes Deployment

**Kubernetes Manifests**:
```yaml
# k8s/deployment.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: campfire-rust
  labels:
    app: campfire-rust
spec:
  replicas: 3
  selector:
    matchLabels:
      app: campfire-rust
  template:
    metadata:
      labels:
        app: campfire-rust
    spec:
      containers:
      - name: campfire-rust
        image: campfire-rust:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: campfire-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: campfire-secrets
              key: redis-url
        - name: INSTANCE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: campfire-service
spec:
  selector:
    app: campfire-rust
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: ClusterIP

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: campfire-ingress
  annotations:
    nginx.ingress.kubernetes.io/websocket-services: "campfire-service"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "86400"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "86400"
spec:
  tls:
  - hosts:
    - campfire.example.com
    secretName: campfire-tls
  rules:
  - host: campfire.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: campfire-service
            port:
              number: 80
```

## Scaling Decision Tree

```mermaid
graph TD
    A[Current Performance] --> B{CPU > 80%?}
    B -->|Yes| C[Scale Up CPU]
    B -->|No| D{Memory > 85%?}
    
    D -->|Yes| E[Scale Up Memory]
    D -->|No| F{Connections > Limit?}
    
    F -->|Yes| G{Single Instance?}
    G -->|Yes| H[Add Second Instance]
    G -->|No| I[Add More Instances]
    
    F -->|No| J{Response Time > Target?}
    J -->|Yes| K[Optimize Database]
    J -->|No| L[Monitor and Maintain]
    
    C --> M[Monitor Results]
    E --> M
    H --> N[Setup Load Balancer]
    I --> O[Check Load Distribution]
    K --> P[Add Database Indexes]
    
    M --> A
    N --> A
    O --> A
    P --> A
    L --> A
```

## Performance Optimization Checklist

### Single-Instance Optimization
- [ ] Enable SQLite WAL mode
- [ ] Optimize connection pool settings
- [ ] Implement efficient WebSocket broadcasting
- [ ] Add performance monitoring
- [ ] Profile memory usage
- [ ] Optimize database queries
- [ ] Enable compression for static assets
- [ ] Implement proper caching headers

### Multi-Instance Preparation
- [ ] Migrate to PostgreSQL
- [ ] Implement Redis session store
- [ ] Set up cross-instance WebSocket sync
- [ ] Configure load balancer
- [ ] Implement health checks
- [ ] Set up monitoring and alerting
- [ ] Test failover scenarios
- [ ] Document deployment procedures

### Production Scaling
- [ ] Implement auto-scaling policies
- [ ] Set up database replication
- [ ] Configure CDN for static assets
- [ ] Implement circuit breakers
- [ ] Set up log aggregation
- [ ] Create disaster recovery plan
- [ ] Implement blue-green deployments
- [ ] Set up performance testing pipeline

## Conclusion

This scaling guide provides a comprehensive roadmap for growing Campfire Rust from a single-instance MVP to a highly available, multi-instance production system. The key principles are:

1. **Start Simple**: Maximize single-instance performance before adding distributed complexity
2. **Measure Everything**: Use metrics to drive scaling decisions
3. **Scale Incrementally**: Add complexity only when proven necessary
4. **Maintain Rust Advantages**: Leverage memory safety and zero-cost abstractions throughout the scaling journey

The Rust implementation provides significant advantages for scaling:
- **Memory Safety**: No garbage collection pauses under high load
- **Zero-Cost Abstractions**: High-level code compiles to efficient machine code
- **Fearless Concurrency**: Safe parallelism without data races
- **Single Binary**: Simplified deployment and reduced operational complexity

Follow this guide to scale Campfire Rust efficiently while maintaining the performance and reliability advantages that Rust provides.