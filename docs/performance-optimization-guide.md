# Performance Optimization Guide for Campfire Rust

This guide provides comprehensive performance optimization strategies for Campfire deployments, covering application-level optimizations, database tuning, infrastructure improvements, and scaling techniques.

## Table of Contents

1. [Performance Baseline](#performance-baseline)
2. [Application Optimization](#application-optimization)
3. [Database Performance](#database-performance)
4. [Memory Optimization](#memory-optimization)
5. [Network Optimization](#network-optimization)
6. [WebSocket Performance](#websocket-performance)
7. [Caching Strategies](#caching-strategies)
8. [Infrastructure Tuning](#infrastructure-tuning)
9. [Monitoring and Profiling](#monitoring-and-profiling)
10. [Load Testing](#load-testing)

## Performance Baseline

### Target Performance Metrics

| Metric | Small Deployment | Medium Deployment | Large Deployment | Enterprise |
|--------|------------------|-------------------|------------------|------------|
| **Response Time (95th)** | <200ms | <150ms | <100ms | <50ms |
| **Throughput** | 100 RPS | 1,000 RPS | 10,000 RPS | 50,000+ RPS |
| **Concurrent Users** | 100 | 1,000 | 10,000 | 100,000+ |
| **WebSocket Connections** | 500 | 5,000 | 50,000 | 500,000+ |
| **Memory Usage** | <512MB | <2GB | <8GB | <32GB |
| **CPU Usage** | <50% | <70% | <80% | <85% |

### Performance Testing Framework

```bash
#!/bin/bash
# scripts/performance-baseline.sh

set -euo pipefail

echo "Running Campfire performance baseline tests..."

# Configuration
BASE_URL="http://localhost:3000"
CONCURRENT_USERS=100
TEST_DURATION=300  # 5 minutes
RAMP_UP_TIME=60    # 1 minute

# Test scenarios
declare -A SCENARIOS=(
    ["health_check"]="GET /health"
    ["message_send"]="POST /api/messages"
    ["message_list"]="GET /api/rooms/{room_id}/messages"
    ["websocket"]="WebSocket /ws"
    ["search"]="GET /api/search?q=test"
)

# Run baseline tests
for scenario in "${!SCENARIOS[@]}"; do
    echo "Testing scenario: $scenario"
    
    case $scenario in
        "health_check")
            ab -n 10000 -c 50 "$BASE_URL/health"
            ;;
        "message_send")
            # Use custom load test script for POST requests
            ./scripts/load-test-messages.sh
            ;;
        "websocket")
            # Use WebSocket load test
            ./scripts/websocket-load-test.sh
            ;;
        *)
            echo "Scenario $scenario not implemented"
            ;;
    esac
done

# Generate performance report
./scripts/generate-performance-report.sh
```

## Application Optimization

### Rust-Specific Optimizations

#### Compiler Optimizations

```toml
# Cargo.toml - Production profile
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

# CPU-specific optimizations
[profile.release-native]
inherits = "release"
rustflags = ["-C", "target-cpu=native"]

# Profile-guided optimization
[profile.pgo]
inherits = "release"
rustflags = ["-C", "profile-generate=/tmp/pgo-data"]
```

#### Memory Pool Optimization

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use object_pool::{Pool, Reusable};

// Message buffer pool to reduce allocations
pub struct MessagePool {
    pool: Pool<Vec<u8>>,
}

impl MessagePool {
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: Pool::new(capacity, || Vec::with_capacity(1024)),
        }
    }
    
    pub fn get_buffer(&self) -> Reusable<Vec<u8>> {
        let mut buffer = self.pool.try_pull().unwrap_or_else(|| {
            self.pool.attach(Vec::with_capacity(1024))
        });
        buffer.clear();
        buffer
    }
}

// Connection pool optimization
pub struct OptimizedConnectionManager {
    connections: Arc<RwLock<HashMap<UserId, Vec<WebSocketSender>>>>,
    message_pool: MessagePool,
    broadcast_buffer: Arc<RwLock<Vec<u8>>>,
}

impl OptimizedConnectionManager {
    pub async fn broadcast_optimized(&self, room_id: RoomId, message: &[u8]) -> Result<()> {
        // Use pre-allocated buffer for serialization
        let mut buffer = self.broadcast_buffer.write().await;
        buffer.clear();
        buffer.extend_from_slice(message);
        
        // Batch send to reduce system calls
        let connections = self.connections.read().await;
        let mut send_futures = Vec::new();
        
        for (_, senders) in connections.iter() {
            for sender in senders {
                send_futures.push(sender.send(buffer.clone()));
            }
        }
        
        // Send all messages concurrently
        futures::future::join_all(send_futures).await;
        
        Ok(())
    }
}
```

#### Async Runtime Optimization

```rust
// Custom runtime configuration for high performance
use tokio::runtime::{Builder, Runtime};

pub fn create_optimized_runtime() -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .max_blocking_threads(512)
        .thread_stack_size(2 * 1024 * 1024)  // 2MB stack
        .thread_name("campfire-worker")
        .enable_all()
        .build()
        .expect("Failed to create runtime")
}

// Optimize task spawning
pub async fn spawn_cpu_intensive_task<F, R>(task: F) -> Result<R, JoinError>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(task).await
}
```

### HTTP Performance Optimization

```rust
use axum::{
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method},
    middleware,
    Router,
};
use tower::{
    buffer::BufferLayer,
    limit::RateLimitLayer,
    timeout::TimeoutLayer,
    ServiceBuilder,
};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};

pub fn create_optimized_router() -> Router {
    Router::new()
        .route("/api/messages", post(send_message))
        .route("/api/rooms/:id/messages", get(get_messages))
        .layer(
            ServiceBuilder::new()
                // Compression for responses
                .layer(CompressionLayer::new())
                
                // Request timeout
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                
                // Rate limiting
                .layer(RateLimitLayer::new(100, Duration::from_secs(60)))
                
                // Buffer requests to handle bursts
                .layer(BufferLayer::new(1024))
                
                // Request size limit
                .layer(DefaultBodyLimit::max(16 * 1024 * 1024))  // 16MB
                
                // CORS with optimized headers
                .layer(
                    CorsLayer::new()
                        .allow_methods([Method::GET, Method::POST])
                        .allow_headers([
                            "content-type",
                            "authorization",
                            "x-request-id",
                        ])
                        .max_age(Duration::from_secs(3600))
                )
                
                // Tracing with sampling
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &Request<_>| {
                            tracing::info_span!(
                                "http_request",
                                method = %request.method(),
                                uri = %request.uri(),
                                version = ?request.version(),
                            )
                        })
                )
        )
}
```

## Database Performance

### SQLite Optimization

```sql
-- Performance-optimized SQLite configuration
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = 20000;  -- 80MB cache (20000 * 4KB)
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 1073741824;  -- 1GB memory mapping
PRAGMA optimize;

-- Analyze tables for better query planning
ANALYZE;

-- Create performance indexes
CREATE INDEX IF NOT EXISTS idx_messages_room_created_desc 
ON messages(room_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_messages_creator_created 
ON messages(creator_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_room_memberships_user_room 
ON room_memberships(user_id, room_id);

CREATE INDEX IF NOT EXISTS idx_sessions_token_expires 
ON sessions(token, expires_at);

-- Optimize FTS5 for better search performance
CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
    content,
    content=messages,
    content_rowid=id,
    tokenize='porter unicode61'
);

-- Rebuild FTS index for optimal performance
INSERT INTO messages_fts(messages_fts) VALUES('rebuild');
```

### Connection Pool Optimization

```rust
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteJournalMode},
    Pool, Sqlite,
};

pub async fn create_optimized_pool(database_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    let connect_options = SqliteConnectOptions::from_str(database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(30))
        .pragma("cache_size", "20000")
        .pragma("temp_store", "memory")
        .pragma("mmap_size", "1073741824")
        .pragma("optimize", "");

    SqlitePoolOptions::new()
        .max_connections(50)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)
        .connect_with(connect_options)
        .await
}
```

### Query Optimization

```rust
// Optimized message retrieval with prepared statements
pub struct OptimizedMessageService {
    pool: Pool<Sqlite>,
    get_messages_stmt: String,
    insert_message_stmt: String,
}

impl OptimizedMessageService {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            pool,
            get_messages_stmt: r#"
                SELECT m.id, m.room_id, m.creator_id, m.content, m.created_at,
                       u.name as creator_name
                FROM messages m
                JOIN users u ON m.creator_id = u.id
                WHERE m.room_id = ?1
                ORDER BY m.created_at DESC
                LIMIT ?2
            "#.to_string(),
            insert_message_stmt: r#"
                INSERT INTO messages (id, room_id, creator_id, content, client_message_id, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(client_message_id, room_id) DO NOTHING
                RETURNING id, created_at
            "#.to_string(),
        }
    }
    
    pub async fn get_messages_optimized(
        &self,
        room_id: RoomId,
        limit: u32,
    ) -> Result<Vec<MessageWithUser>, MessageError> {
        let messages = sqlx::query_as::<_, MessageWithUser>(&self.get_messages_stmt)
            .bind(room_id.0)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;
        
        Ok(messages)
    }
    
    // Batch insert for better performance
    pub async fn insert_messages_batch(
        &self,
        messages: Vec<NewMessage>,
    ) -> Result<Vec<Message>, MessageError> {
        let mut tx = self.pool.begin().await?;
        let mut results = Vec::with_capacity(messages.len());
        
        for message in messages {
            let result = sqlx::query_as::<_, Message>(&self.insert_message_stmt)
                .bind(message.id.0)
                .bind(message.room_id.0)
                .bind(message.creator_id.0)
                .bind(&message.content)
                .bind(message.client_message_id)
                .bind(message.created_at)
                .fetch_one(&mut *tx)
                .await?;
            
            results.push(result);
        }
        
        tx.commit().await?;
        Ok(results)
    }
}
```

## Memory Optimization

### Memory Pool Management

```rust
use std::sync::Arc;
use parking_lot::RwLock;

// Custom allocator for message buffers
pub struct MessageBufferAllocator {
    small_buffers: Arc<RwLock<Vec<Vec<u8>>>>,  // < 1KB
    medium_buffers: Arc<RwLock<Vec<Vec<u8>>>>, // 1KB - 10KB
    large_buffers: Arc<RwLock<Vec<Vec<u8>>>>,  // > 10KB
}

impl MessageBufferAllocator {
    pub fn new() -> Self {
        Self {
            small_buffers: Arc::new(RwLock::new(Vec::new())),
            medium_buffers: Arc::new(RwLock::new(Vec::new())),
            large_buffers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn get_buffer(&self, size_hint: usize) -> Vec<u8> {
        let pool = match size_hint {
            0..=1024 => &self.small_buffers,
            1025..=10240 => &self.medium_buffers,
            _ => &self.large_buffers,
        };
        
        pool.write()
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(size_hint.max(1024)))
    }
    
    pub fn return_buffer(&self, mut buffer: Vec<u8>) {
        buffer.clear();
        
        let pool = match buffer.capacity() {
            0..=1024 => &self.small_buffers,
            1025..=10240 => &self.medium_buffers,
            _ => &self.large_buffers,
        };
        
        let mut pool_guard = pool.write();
        if pool_guard.len() < 100 {  // Limit pool size
            pool_guard.push(buffer);
        }
    }
}
```

### String Interning

```rust
use string_cache::{DefaultAtom, Atom};
use std::collections::HashMap;
use parking_lot::RwLock;

// Intern frequently used strings to reduce memory usage
pub struct StringInterner {
    cache: RwLock<HashMap<String, DefaultAtom>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn intern(&self, s: &str) -> DefaultAtom {
        // Check if already interned
        {
            let cache = self.cache.read();
            if let Some(atom) = cache.get(s) {
                return atom.clone();
            }
        }
        
        // Intern new string
        let mut cache = self.cache.write();
        let atom = DefaultAtom::from(s);
        cache.insert(s.to_string(), atom.clone());
        atom
    }
}

// Use interned strings for user names and room names
#[derive(Debug, Clone)]
pub struct OptimizedUser {
    pub id: UserId,
    pub name: DefaultAtom,  // Interned string
    pub email: String,
    // ... other fields
}
```

## Network Optimization

### HTTP/2 and Connection Optimization

```rust
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;

pub async fn start_optimized_server(app: Router) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    
    // Configure TLS for HTTP/2
    let config = RustlsConfig::from_pem_file(
        "cert.pem",
        "key.pem",
    ).await?;
    
    // Start server with optimized settings
    axum_server::bind_rustls(addr, config)
        .http2_initial_stream_window_size(Some(1024 * 1024))  // 1MB
        .http2_initial_connection_window_size(Some(8 * 1024 * 1024))  // 8MB
        .http2_max_frame_size(Some(32 * 1024))  // 32KB
        .tcp_keepalive(Some(Duration::from_secs(60)))
        .tcp_nodelay(true)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
```

### WebSocket Frame Optimization

```rust
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

pub struct OptimizedWebSocket {
    stream: WebSocketStream<TcpStream>,
    send_buffer: Vec<u8>,
    compression_enabled: bool,
}

impl OptimizedWebSocket {
    pub async fn send_optimized(&mut self, data: &[u8]) -> Result<(), WebSocketError> {
        // Use binary frames for better performance
        let message = if self.compression_enabled && data.len() > 1024 {
            // Compress large messages
            let compressed = compress_data(data)?;
            Message::Binary(compressed)
        } else {
            Message::Binary(data.to_vec())
        };
        
        self.stream.send(message).await?;
        Ok(())
    }
    
    // Batch multiple messages into a single frame
    pub async fn send_batch(&mut self, messages: &[&[u8]]) -> Result<(), WebSocketError> {
        self.send_buffer.clear();
        
        for msg in messages {
            self.send_buffer.extend_from_slice(&(msg.len() as u32).to_le_bytes());
            self.send_buffer.extend_from_slice(msg);
        }
        
        self.send_optimized(&self.send_buffer).await
    }
}
```

## WebSocket Performance

### Connection Management Optimization

```rust
use dashmap::DashMap;
use tokio::sync::broadcast;

pub struct HighPerformanceConnectionManager {
    // Use DashMap for concurrent access without locks
    connections: DashMap<UserId, Vec<WebSocketSender>>,
    room_subscribers: DashMap<RoomId, broadcast::Sender<Arc<[u8]>>>,
    connection_count: AtomicUsize,
}

impl HighPerformanceConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
            room_subscribers: DashMap::new(),
            connection_count: AtomicUsize::new(0),
        }
    }
    
    pub async fn add_connection_optimized(
        &self,
        user_id: UserId,
        room_id: RoomId,
        sender: WebSocketSender,
    ) -> Result<broadcast::Receiver<Arc<[u8]>>, ConnectionError> {
        // Add to user connections
        self.connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(sender);
        
        // Subscribe to room broadcasts
        let receiver = self.room_subscribers
            .entry(room_id)
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(1000);
                tx
            })
            .subscribe();
        
        self.connection_count.fetch_add(1, Ordering::Relaxed);
        
        Ok(receiver)
    }
    
    pub async fn broadcast_to_room_optimized(
        &self,
        room_id: RoomId,
        message: Arc<[u8]>,
    ) -> Result<usize, BroadcastError> {
        if let Some(sender) = self.room_subscribers.get(&room_id) {
            match sender.send(message) {
                Ok(subscriber_count) => Ok(subscriber_count),
                Err(_) => Ok(0), // No subscribers
            }
        } else {
            Ok(0)
        }
    }
}
```

### Message Serialization Optimization

```rust
use serde::{Serialize, Deserialize};
use rmp_serde::{Serializer, Deserializer};

// Use MessagePack for more efficient serialization
#[derive(Serialize, Deserialize)]
pub struct OptimizedWebSocketMessage {
    #[serde(rename = "t")]
    pub message_type: u8,
    
    #[serde(rename = "d")]
    pub data: Vec<u8>,
    
    #[serde(rename = "ts")]
    pub timestamp: u64,
}

impl OptimizedWebSocketMessage {
    pub fn serialize_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        let mut buf = Vec::new();
        self.serialize(&mut Serializer::new(&mut buf))?;
        Ok(buf)
    }
    
    pub fn deserialize_msgpack(data: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        let mut de = Deserializer::new(data);
        Self::deserialize(&mut de)
    }
}

// Pre-serialize common messages
pub struct MessageCache {
    cached_messages: RwLock<HashMap<String, Arc<[u8]>>>,
}

impl MessageCache {
    pub fn get_or_serialize(&self, key: &str, message: &impl Serialize) -> Arc<[u8]> {
        // Check cache first
        {
            let cache = self.cached_messages.read().unwrap();
            if let Some(cached) = cache.get(key) {
                return cached.clone();
            }
        }
        
        // Serialize and cache
        let serialized = rmp_serde::to_vec(message).unwrap();
        let arc_data: Arc<[u8]> = serialized.into();
        
        let mut cache = self.cached_messages.write().unwrap();
        cache.insert(key.to_string(), arc_data.clone());
        
        arc_data
    }
}
```

## Caching Strategies

### Application-Level Caching

```rust
use moka::future::Cache;
use std::time::Duration;

pub struct CampfireCacheManager {
    user_cache: Cache<UserId, Arc<User>>,
    room_cache: Cache<RoomId, Arc<Room>>,
    message_cache: Cache<String, Arc<Vec<Message>>>,
    search_cache: Cache<String, Arc<SearchResults>>,
}

impl CampfireCacheManager {
    pub fn new() -> Self {
        Self {
            user_cache: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(300))
                .time_to_idle(Duration::from_secs(60))
                .build(),
            
            room_cache: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(Duration::from_secs(600))
                .build(),
            
            message_cache: Cache::builder()
                .max_capacity(100_000)
                .time_to_live(Duration::from_secs(60))
                .weigher(|_key, value: &Arc<Vec<Message>>| value.len() as u32)
                .max_capacity(50_000_000)  // 50MB worth of messages
                .build(),
            
            search_cache: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(Duration::from_secs(300))
                .build(),
        }
    }
    
    pub async fn get_user_cached(&self, user_id: UserId) -> Option<Arc<User>> {
        self.user_cache.get(&user_id).await
    }
    
    pub async fn cache_user(&self, user: User) {
        self.user_cache.insert(user.id, Arc::new(user)).await;
    }
    
    // Cache invalidation
    pub async fn invalidate_user(&self, user_id: UserId) {
        self.user_cache.invalidate(&user_id).await;
    }
    
    // Batch cache operations
    pub async fn get_users_batch(&self, user_ids: &[UserId]) -> HashMap<UserId, Arc<User>> {
        let mut results = HashMap::new();
        
        for &user_id in user_ids {
            if let Some(user) = self.user_cache.get(&user_id).await {
                results.insert(user_id, user);
            }
        }
        
        results
    }
}
```

### Redis Caching Integration

```rust
use redis::{AsyncCommands, Client};
use serde::{Serialize, Deserialize};

pub struct RedisCacheLayer {
    client: Client,
    key_prefix: String,
}

impl RedisCacheLayer {
    pub fn new(redis_url: &str, key_prefix: String) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client, key_prefix })
    }
    
    pub async fn get_cached<T>(&self, key: &str) -> Result<Option<T>, redis::RedisError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self.client.get_async_connection().await?;
        let full_key = format!("{}:{}", self.key_prefix, key);
        
        let data: Option<Vec<u8>> = conn.get(&full_key).await?;
        
        match data {
            Some(bytes) => {
                let value = bincode::deserialize(&bytes)
                    .map_err(|_| redis::RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed")))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
    
    pub async fn set_cached<T>(&self, key: &str, value: &T, ttl: u64) -> Result<(), redis::RedisError>
    where
        T: Serialize,
    {
        let mut conn = self.client.get_async_connection().await?;
        let full_key = format!("{}:{}", self.key_prefix, key);
        
        let serialized = bincode::serialize(value)
            .map_err(|_| redis::RedisError::from((redis::ErrorKind::TypeError, "Serialization failed")))?;
        
        conn.set_ex(&full_key, serialized, ttl).await?;
        Ok(())
    }
    
    // Pipeline operations for better performance
    pub async fn set_multiple<T>(&self, items: &[(String, T, u64)]) -> Result<(), redis::RedisError>
    where
        T: Serialize,
    {
        let mut conn = self.client.get_async_connection().await?;
        let mut pipe = redis::pipe();
        
        for (key, value, ttl) in items {
            let full_key = format!("{}:{}", self.key_prefix, key);
            let serialized = bincode::serialize(value)
                .map_err(|_| redis::RedisError::from((redis::ErrorKind::TypeError, "Serialization failed")))?;
            
            pipe.set_ex(&full_key, serialized, *ttl);
        }
        
        pipe.query_async(&mut conn).await?;
        Ok(())
    }
}
```

## Infrastructure Tuning

### Container Optimization

```dockerfile
# Multi-stage build with optimization
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy and build dependencies first (better caching)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source and build application
COPY src ./src
COPY templates ./templates
COPY assets ./assets

# Build with optimizations
ENV RUSTFLAGS="-C target-cpu=native -C opt-level=3"
RUN cargo build --release --bin campfire-on-rust

# Production image with minimal base
FROM debian:bookworm-slim

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create optimized user and directories
RUN useradd -m -u 1001 -s /bin/bash campfire && \
    mkdir -p /app/{data,logs,backups} && \
    chown -R campfire:campfire /app

WORKDIR /app

# Copy optimized binary
COPY --from=builder /app/target/release/campfire-on-rust /app/campfire-on-rust
RUN chmod +x /app/campfire-on-rust

# Performance-optimized environment
ENV RUST_LOG=campfire_on_rust=info
ENV RUST_BACKTRACE=0
ENV MALLOC_ARENA_MAX=2

USER campfire

# Optimized resource limits
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["/app/campfire-on-rust"]
```

### System-Level Optimizations

```bash
#!/bin/bash
# scripts/optimize-system.sh

set -euo pipefail

echo "Applying system-level optimizations for Campfire..."

# Network optimizations
sysctl -w net.core.somaxconn=65536
sysctl -w net.core.netdev_max_backlog=5000
sysctl -w net.ipv4.tcp_max_syn_backlog=65536
sysctl -w net.ipv4.tcp_fin_timeout=30
sysctl -w net.ipv4.tcp_keepalive_time=1200
sysctl -w net.ipv4.tcp_keepalive_probes=3
sysctl -w net.ipv4.tcp_keepalive_intvl=15

# File descriptor limits
echo "fs.file-max = 2097152" >> /etc/sysctl.conf
echo "campfire soft nofile 65536" >> /etc/security/limits.conf
echo "campfire hard nofile 65536" >> /etc/security/limits.conf

# Memory optimizations
sysctl -w vm.swappiness=10
sysctl -w vm.dirty_ratio=15
sysctl -w vm.dirty_background_ratio=5

# Apply changes
sysctl -p

echo "System optimizations applied successfully"
```

## Monitoring and Profiling

### Performance Profiling

```bash
#!/bin/bash
# scripts/profile-performance.sh

set -euo pipefail

PROFILE_DURATION=${1:-60}
OUTPUT_DIR="./profiling/$(date +%Y%m%d_%H%M%S)"

mkdir -p "$OUTPUT_DIR"

echo "Starting performance profiling for ${PROFILE_DURATION} seconds..."

# CPU profiling with perf
perf record -g -p $(pgrep campfire-on-rust) -o "$OUTPUT_DIR/perf.data" &
PERF_PID=$!

# Memory profiling with valgrind (if available)
if command -v valgrind &> /dev/null; then
    valgrind --tool=massif --massif-out-file="$OUTPUT_DIR/massif.out" \
        --pid=$(pgrep campfire-on-rust) &
    VALGRIND_PID=$!
fi

# Application metrics collection
curl -s http://localhost:3000/metrics > "$OUTPUT_DIR/metrics_start.txt"

# Wait for profiling duration
sleep "$PROFILE_DURATION"

# Stop profiling
kill $PERF_PID 2>/dev/null || true
kill $VALGRIND_PID 2>/dev/null || true

# Collect final metrics
curl -s http://localhost:3000/metrics > "$OUTPUT_DIR/metrics_end.txt"

# Generate reports
perf report -i "$OUTPUT_DIR/perf.data" > "$OUTPUT_DIR/cpu_profile.txt"

if [[ -f "$OUTPUT_DIR/massif.out" ]]; then
    ms_print "$OUTPUT_DIR/massif.out" > "$OUTPUT_DIR/memory_profile.txt"
fi

echo "Profiling completed. Results in: $OUTPUT_DIR"
```

### Continuous Performance Monitoring

```rust
use std::time::{Duration, Instant};
use tokio::time::interval;

pub struct PerformanceMonitor {
    metrics_collector: MetricsCollector,
    alert_thresholds: AlertThresholds,
}

#[derive(Clone)]
pub struct AlertThresholds {
    pub max_response_time_ms: u64,
    pub max_memory_mb: u64,
    pub max_cpu_percent: f64,
    pub max_error_rate: f64,
}

impl PerformanceMonitor {
    pub async fn start_monitoring(&self) {
        let mut interval = interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            let metrics = self.collect_current_metrics().await;
            
            // Check thresholds and alert if necessary
            self.check_and_alert(&metrics).await;
            
            // Log performance summary
            tracing::info!(
                response_time_ms = metrics.avg_response_time_ms,
                memory_mb = metrics.memory_usage_mb,
                cpu_percent = metrics.cpu_usage_percent,
                active_connections = metrics.active_connections,
                "Performance metrics collected"
            );
        }
    }
    
    async fn collect_current_metrics(&self) -> PerformanceMetrics {
        let start = Instant::now();
        
        // Collect various metrics
        let memory_usage = self.get_memory_usage().await;
        let cpu_usage = self.get_cpu_usage().await;
        let response_times = self.get_recent_response_times().await;
        let active_connections = self.get_active_connections().await;
        
        PerformanceMetrics {
            timestamp: chrono::Utc::now(),
            avg_response_time_ms: response_times.iter().sum::<u64>() / response_times.len() as u64,
            memory_usage_mb: memory_usage / 1024 / 1024,
            cpu_usage_percent: cpu_usage,
            active_connections,
            collection_time_ms: start.elapsed().as_millis() as u64,
        }
    }
}
```

## Load Testing

### Comprehensive Load Testing Suite

```bash
#!/bin/bash
# scripts/comprehensive-load-test.sh

set -euo pipefail

BASE_URL="${1:-http://localhost:3000}"
MAX_USERS="${2:-1000}"
RAMP_UP_TIME="${3:-300}"
TEST_DURATION="${4:-600}"

echo "Starting comprehensive load test..."
echo "Target: $BASE_URL"
echo "Max Users: $MAX_USERS"
echo "Ramp-up: ${RAMP_UP_TIME}s"
echo "Duration: ${TEST_DURATION}s"

# Create test configuration
cat > load-test-config.yml <<EOF
config:
  target: '$BASE_URL'
  phases:
    - duration: $RAMP_UP_TIME
      arrivalRate: 1
      rampTo: $(($MAX_USERS / 10))
      name: "Ramp up"
    - duration: $TEST_DURATION
      arrivalRate: $(($MAX_USERS / 10))
      name: "Sustained load"
    - duration: 60
      arrivalRate: 1
      name: "Cool down"
  
  processor: "./load-test-processor.js"

scenarios:
  - name: "HTTP API Load Test"
    weight: 60
    flow:
      - get:
          url: "/health"
          capture:
            - json: "$.status"
              as: "health_status"
      - think: 1
      - post:
          url: "/api/messages"
          json:
            content: "Load test message {{ \$randomString() }}"
            room_id: "{{ \$randomUUID() }}"
          capture:
            - json: "$.id"
              as: "message_id"
      - think: 2
      - get:
          url: "/api/rooms/{{ room_id }}/messages"
          qs:
            limit: 50

  - name: "WebSocket Load Test"
    weight: 30
    engine: ws
    flow:
      - connect:
          url: "/ws"
      - send:
          payload: '{"type": "join_room", "room_id": "{{ \$randomUUID() }}"}'
      - think: 5
      - loop:
        - send:
            payload: '{"type": "message", "content": "WS test {{ \$randomString() }}"}'
        - think: 10
        count: 10

  - name: "Search Load Test"
    weight: 10
    flow:
      - get:
          url: "/api/search"
          qs:
            q: "{{ \$randomString() }}"
            limit: 20
EOF

# Run load test
artillery run load-test-config.yml --output load-test-results.json

# Generate HTML report
artillery report load-test-results.json --output load-test-report.html

echo "Load test completed. Report: load-test-report.html"
```

### WebSocket Load Testing

```javascript
// websocket-load-test.js
const WebSocket = require('ws');
const { performance } = require('perf_hooks');

class WebSocketLoadTest {
    constructor(url, concurrentConnections = 100, messagesPerConnection = 100) {
        this.url = url;
        this.concurrentConnections = concurrentConnections;
        this.messagesPerConnection = messagesPerConnection;
        this.connections = [];
        this.metrics = {
            connectionsEstablished: 0,
            messagesSent: 0,
            messagesReceived: 0,
            errors: 0,
            responseTimes: []
        };
    }

    async runTest() {
        console.log(`Starting WebSocket load test with ${this.concurrentConnections} connections`);
        
        const startTime = performance.now();
        
        // Create connections
        const connectionPromises = [];
        for (let i = 0; i < this.concurrentConnections; i++) {
            connectionPromises.push(this.createConnection(i));
        }
        
        await Promise.all(connectionPromises);
        
        // Send messages
        await this.sendMessages();
        
        // Wait for responses
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        // Close connections
        this.connections.forEach(ws => ws.close());
        
        const totalTime = performance.now() - startTime;
        
        this.printResults(totalTime);
    }

    async createConnection(id) {
        return new Promise((resolve, reject) => {
            const ws = new WebSocket(this.url);
            
            ws.on('open', () => {
                this.metrics.connectionsEstablished++;
                this.connections.push(ws);
                resolve();
            });
            
            ws.on('message', (data) => {
                this.metrics.messagesReceived++;
                const message = JSON.parse(data);
                if (message.timestamp) {
                    const responseTime = Date.now() - message.timestamp;
                    this.metrics.responseTimes.push(responseTime);
                }
            });
            
            ws.on('error', (error) => {
                this.metrics.errors++;
                console.error(`Connection ${id} error:`, error);
                reject(error);
            });
            
            setTimeout(() => reject(new Error('Connection timeout')), 10000);
        });
    }

    async sendMessages() {
        const sendPromises = [];
        
        this.connections.forEach((ws, connectionId) => {
            for (let i = 0; i < this.messagesPerConnection; i++) {
                const message = {
                    type: 'message',
                    content: `Load test message ${i} from connection ${connectionId}`,
                    timestamp: Date.now()
                };
                
                sendPromises.push(
                    new Promise(resolve => {
                        ws.send(JSON.stringify(message));
                        this.metrics.messagesSent++;
                        setTimeout(resolve, Math.random() * 100); // Random delay
                    })
                );
            }
        });
        
        await Promise.all(sendPromises);
    }

    printResults(totalTime) {
        const avgResponseTime = this.metrics.responseTimes.length > 0 
            ? this.metrics.responseTimes.reduce((a, b) => a + b) / this.metrics.responseTimes.length 
            : 0;
        
        const p95ResponseTime = this.metrics.responseTimes.length > 0
            ? this.metrics.responseTimes.sort((a, b) => a - b)[Math.floor(this.metrics.responseTimes.length * 0.95)]
            : 0;

        console.log('\n=== WebSocket Load Test Results ===');
        console.log(`Total test time: ${(totalTime / 1000).toFixed(2)}s`);
        console.log(`Connections established: ${this.metrics.connectionsEstablished}`);
        console.log(`Messages sent: ${this.metrics.messagesSent}`);
        console.log(`Messages received: ${this.metrics.messagesReceived}`);
        console.log(`Errors: ${this.metrics.errors}`);
        console.log(`Average response time: ${avgResponseTime.toFixed(2)}ms`);
        console.log(`95th percentile response time: ${p95ResponseTime.toFixed(2)}ms`);
        console.log(`Message throughput: ${(this.metrics.messagesSent / (totalTime / 1000)).toFixed(2)} msg/s`);
    }
}

// Run the test
const test = new WebSocketLoadTest('ws://localhost:3000/ws', 500, 50);
test.runTest().catch(console.error);
```

This comprehensive performance optimization guide provides the tools and techniques needed to maximize Campfire's performance across all deployment scenarios, from small teams to enterprise-scale deployments.