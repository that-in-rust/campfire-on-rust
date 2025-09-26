use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;
use tokio::runtime::Runtime;
use campfire_on_rust::models::*;
use campfire_on_rust::services::*;
use campfire_on_rust::database::*;
use std::sync::Arc;

/// Benchmark database operations
fn benchmark_database_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("database_operations");
    
    // Setup test database
    let db = rt.block_on(async {
        CampfireDatabase::new(":memory:").await.unwrap()
    });
    
    // Benchmark user creation
    group.bench_function("create_user", |b| {
        b.to_async(&rt).iter(|| async {
            let user = User {
                id: UserId::new(),
                name: "Test User".to_string(),
                email: format!("test{}@example.com", uuid::Uuid::new_v4()),
                password_hash: "hash".to_string(),
                bio: None,
                admin: false,
                bot_token: None,
                created_at: chrono::Utc::now(),
            };
            
            black_box(db.create_user(&user).await)
        });
    });
    
    // Benchmark message creation with deduplication
    group.bench_function("create_message_with_deduplication", |b| {
        b.to_async(&rt).iter(|| async {
            let message = Message {
                id: MessageId::new(),
                room_id: RoomId::new(),
                creator_id: UserId::new(),
                content: "Test message content".to_string(),
                client_message_id: uuid::Uuid::new_v4(),
                created_at: chrono::Utc::now(),
                html_content: None,
                mentions: Vec::new(),
                sound_commands: Vec::new(),
            };
            
            black_box(db.create_message_with_deduplication(&message).await)
        });
    });
    
    // Benchmark message retrieval
    let room_id = RoomId::new();
    group.bench_function("get_room_messages", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(db.get_room_messages(room_id, 50, None).await)
        });
    });
    
    group.finish();
}

/// Benchmark WebSocket connection management
fn benchmark_connection_management(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("connection_management");
    
    // Setup connection manager
    let db = rt.block_on(async {
        CampfireDatabase::new(":memory:").await.unwrap()
    });
    let manager = ConnectionManagerImpl::new(Arc::new(db));
    
    // Benchmark adding connections
    group.bench_function("add_connection", |b| {
        b.to_async(&rt).iter(|| async {
            let user_id = UserId::new();
            let connection_id = ConnectionId::new();
            let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();
            
            black_box(manager.add_connection(user_id, connection_id, sender).await)
        });
    });
    
    // Benchmark presence tracking
    group.bench_function("get_room_presence", |b| {
        b.to_async(&rt).iter(|| async {
            let room_id = RoomId::new();
            black_box(manager.get_room_presence(room_id).await)
        });
    });
    
    group.finish();
}

/// Benchmark message broadcasting
fn benchmark_message_broadcasting(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("message_broadcasting");
    
    // Setup connection manager with multiple connections
    let db = rt.block_on(async {
        CampfireDatabase::new(":memory:").await.unwrap()
    });
    let manager = ConnectionManagerImpl::new(Arc::new(db));
    let room_id = RoomId::new();
    
    // Add multiple connections for broadcasting
    rt.block_on(async {
        for i in 0..100 {
            let user_id = UserId::new();
            let connection_id = ConnectionId::new();
            let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();
            
            manager.add_connection(user_id, connection_id, sender).await.unwrap();
            manager.add_room_membership(room_id, vec![user_id]).await;
        }
    });
    
    // Benchmark broadcasting to different numbers of connections
    for connection_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("broadcast_to_room", connection_count),
            connection_count,
            |b, &_connection_count| {
                b.to_async(&rt).iter(|| async {
                    let message = WebSocketMessage::NewMessage {
                        message: Message {
                            id: MessageId::new(),
                            room_id,
                            creator_id: UserId::new(),
                            content: "Broadcast test message".to_string(),
                            client_message_id: uuid::Uuid::new_v4(),
                            created_at: chrono::Utc::now(),
                            html_content: None,
                            mentions: Vec::new(),
                            sound_commands: Vec::new(),
                        },
                    };
                    
                    black_box(manager.broadcast_to_room(room_id, message).await)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark serialization performance
fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    let message = Message {
        id: MessageId::new(),
        room_id: RoomId::new(),
        creator_id: UserId::new(),
        content: "Test message with some content that might be typical".to_string(),
        client_message_id: uuid::Uuid::new_v4(),
        created_at: chrono::Utc::now(),
        html_content: Some("<p>HTML content</p>".to_string()),
        mentions: vec!["@user1".to_string(), "@user2".to_string()],
        sound_commands: vec!["/play tada".to_string()],
    };
    
    let ws_message = WebSocketMessage::NewMessage { message };
    
    // Benchmark JSON serialization
    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&ws_message).unwrap())
        });
    });
    
    // Benchmark JSON deserialization
    let serialized = serde_json::to_string(&ws_message).unwrap();
    group.bench_function("json_deserialize", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<WebSocketMessage>(&serialized).unwrap())
        });
    });
    
    group.finish();
}

/// Benchmark search operations
fn benchmark_search_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("search_operations");
    
    // Setup database with test data
    let db = rt.block_on(async {
        let db = CampfireDatabase::new(":memory:").await.unwrap();
        
        // Insert test messages for searching
        for i in 0..1000 {
            let message = Message {
                id: MessageId::new(),
                room_id: RoomId::new(),
                creator_id: UserId::new(),
                content: format!("Test message {} with searchable content", i),
                client_message_id: uuid::Uuid::new_v4(),
                created_at: chrono::Utc::now(),
                html_content: None,
                mentions: Vec::new(),
                sound_commands: Vec::new(),
            };
            
            let _ = db.create_message_with_deduplication(&message).await;
        }
        
        db
    });
    
    let search_service = SearchServiceImpl::new(Arc::new(db));
    
    // Benchmark different search query types
    let search_queries = [
        "test",
        "message",
        "searchable content",
        "test message 500",
    ];
    
    for query in search_queries.iter() {
        group.bench_with_input(
            BenchmarkId::new("search_messages", query),
            query,
            |b, &query| {
                b.to_async(&rt).iter(|| async {
                    black_box(search_service.search_messages(
                        query,
                        UserId::new(),
                        None,
                        50,
                        0,
                    ).await)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark concurrent operations
fn benchmark_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_operations");
    group.throughput(Throughput::Elements(100));
    
    // Setup
    let db = rt.block_on(async {
        CampfireDatabase::new(":memory:").await.unwrap()
    });
    let manager = ConnectionManagerImpl::new(Arc::new(db));
    
    // Benchmark concurrent connection additions
    group.bench_function("concurrent_add_connections", |b| {
        b.to_async(&rt).iter(|| async {
            let mut tasks = Vec::new();
            
            for _ in 0..100 {
                let manager_clone = manager.clone();
                let task = tokio::spawn(async move {
                    let user_id = UserId::new();
                    let connection_id = ConnectionId::new();
                    let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();
                    
                    manager_clone.add_connection(user_id, connection_id, sender).await
                });
                tasks.push(task);
            }
            
            black_box(futures::future::join_all(tasks).await)
        });
    });
    
    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_usage");
    
    // Benchmark message creation and cleanup
    group.bench_function("message_lifecycle", |b| {
        b.to_async(&rt).iter(|| async {
            let mut messages = Vec::new();
            
            // Create many messages
            for i in 0..1000 {
                let message = Message {
                    id: MessageId::new(),
                    room_id: RoomId::new(),
                    creator_id: UserId::new(),
                    content: format!("Message {} content", i),
                    client_message_id: uuid::Uuid::new_v4(),
                    created_at: chrono::Utc::now(),
                    html_content: None,
                    mentions: Vec::new(),
                    sound_commands: Vec::new(),
                };
                messages.push(message);
            }
            
            // Process messages
            for message in &messages {
                black_box(serde_json::to_string(message).unwrap());
            }
            
            // Clear messages (simulate cleanup)
            messages.clear();
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_database_operations,
    benchmark_connection_management,
    benchmark_message_broadcasting,
    benchmark_serialization,
    benchmark_search_operations,
    benchmark_concurrent_operations,
    benchmark_memory_usage
);

criterion_main!(benches);