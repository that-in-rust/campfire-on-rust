use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteJournalMode, SqliteSynchronous},
    Pool, Sqlite, Error as SqlxError,
};
use std::time::{Duration, Instant};
use std::str::FromStr;
use tracing::{info, warn, error};
use crate::metrics::get_performance_monitor;

/// Optimized SQLite connection pool with performance monitoring
pub struct OptimizedConnectionPool {
    pool: Pool<Sqlite>,
    pool_config: PoolConfig,
}

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub test_before_acquire: bool,
    pub journal_mode: SqliteJournalMode,
    pub synchronous: SqliteSynchronous,
    pub cache_size: i32,
    pub temp_store: String,
    pub mmap_size: i64,
    pub busy_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 50,
            min_connections: 5,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            test_before_acquire: true,
            journal_mode: SqliteJournalMode::Wal,
            synchronous: SqliteSynchronous::Normal,
            cache_size: 20000, // 80MB cache (20000 * 4KB pages)
            temp_store: "memory".to_string(),
            mmap_size: 1073741824, // 1GB memory mapping
            busy_timeout: Duration::from_secs(30),
        }
    }
}

impl OptimizedConnectionPool {
    /// Create a new optimized connection pool
    pub async fn new(database_url: &str, config: Option<PoolConfig>) -> Result<Self, SqlxError> {
        let config = config.unwrap_or_default();
        
        info!("Creating optimized SQLite connection pool with config: {:?}", config);
        
        // Configure SQLite connection options for optimal performance
        let connect_options = SqliteConnectOptions::from_str(database_url)?
            .journal_mode(config.journal_mode)
            .synchronous(config.synchronous)
            .busy_timeout(config.busy_timeout)
            .pragma("cache_size", config.cache_size.to_string())
            .pragma("temp_store", config.temp_store.clone())
            .pragma("mmap_size", config.mmap_size.to_string())
            .pragma("optimize", "")
            .pragma("foreign_keys", "ON")
            .pragma("recursive_triggers", "ON");
        
        // Create connection pool with optimized settings
        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(Some(config.idle_timeout))
            .max_lifetime(Some(config.max_lifetime))
            .test_before_acquire(config.test_before_acquire)
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Additional optimizations after connection
                    sqlx::query("PRAGMA optimize").execute(conn).await?;
                    Ok(())
                })
            })
            .connect_with(connect_options)
            .await?;
        
        info!("SQLite connection pool created successfully");
        
        let optimized_pool = Self {
            pool,
            pool_config: config,
        };
        
        // Start monitoring task
        optimized_pool.start_monitoring().await;
        
        Ok(optimized_pool)
    }
    
    /// Get the underlying SQLite pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
    
    /// Get pool configuration
    pub fn config(&self) -> &PoolConfig {
        &self.pool_config
    }
    
    /// Execute a query with performance monitoring
    pub async fn execute_monitored<'q>(
        &self,
        query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
        operation_name: &str,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, SqlxError> {
        let start = Instant::now();
        
        let result = query.execute(&self.pool).await;
        
        let duration = start.elapsed();
        let success = result.is_ok();
        
        // Record metrics
        crate::metrics::record_database_query(operation_name, duration, success);
        
        if !success {
            error!("Database query failed: {} - {:?}", operation_name, result);
        } else if duration > Duration::from_millis(100) {
            warn!("Slow database query: {} took {:?}", operation_name, duration);
        }
        
        result
    }
    
    /// Fetch one row with performance monitoring
    pub async fn fetch_one_monitored<'q, T>(
        &self,
        query: sqlx::query::QueryAs<'q, Sqlite, T, sqlx::sqlite::SqliteArguments<'q>>,
        operation_name: &str,
    ) -> Result<T, SqlxError>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin + std::fmt::Debug,
    {
        let start = Instant::now();
        
        let result = query.fetch_one(&self.pool).await;
        
        let duration = start.elapsed();
        let success = result.is_ok();
        
        // Record metrics
        crate::metrics::record_database_query(operation_name, duration, success);
        
        if !success {
            error!("Database query failed: {} - {:?}", operation_name, result);
        } else if duration > Duration::from_millis(100) {
            warn!("Slow database query: {} took {:?}", operation_name, duration);
        }
        
        result
    }
    
    /// Fetch optional row with performance monitoring
    pub async fn fetch_optional_monitored<'q, T>(
        &self,
        query: sqlx::query::QueryAs<'q, Sqlite, T, sqlx::sqlite::SqliteArguments<'q>>,
        operation_name: &str,
    ) -> Result<Option<T>, SqlxError>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin + std::fmt::Debug,
    {
        let start = Instant::now();
        
        let result = query.fetch_optional(&self.pool).await;
        
        let duration = start.elapsed();
        let success = result.is_ok();
        
        // Record metrics
        crate::metrics::record_database_query(operation_name, duration, success);
        
        if !success {
            error!("Database query failed: {} - {:?}", operation_name, result);
        } else if duration > Duration::from_millis(100) {
            warn!("Slow database query: {} took {:?}", operation_name, duration);
        }
        
        result
    }
    
    /// Fetch all rows with performance monitoring
    pub async fn fetch_all_monitored<'q, T>(
        &self,
        query: sqlx::query::QueryAs<'q, Sqlite, T, sqlx::sqlite::SqliteArguments<'q>>,
        operation_name: &str,
    ) -> Result<Vec<T>, SqlxError>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin + std::fmt::Debug,
    {
        let start = Instant::now();
        
        let result = query.fetch_all(&self.pool).await;
        
        let duration = start.elapsed();
        let success = result.is_ok();
        
        // Record metrics
        crate::metrics::record_database_query(operation_name, duration, success);
        
        if !success {
            error!("Database query failed: {} - {:?}", operation_name, result);
        } else if duration > Duration::from_millis(100) {
            warn!("Slow database query: {} took {:?}", operation_name, duration);
        }
        
        result
    }
    
    /// Start background monitoring task
    async fn start_monitoring(&self) {
        let pool = self.pool.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Update connection pool metrics
                let monitor = get_performance_monitor();
                monitor.update_connection_pool_stats(|stats| {
                    stats.active_connections = pool.size();
                    stats.idle_connections = pool.num_idle() as u32;
                    stats.total_connections = pool.size();
                    // Note: SQLx doesn't expose wait time directly, so we use 0 as placeholder
                    stats.connection_wait_time_ms = 0.0;
                }).await;
            }
        });
    }
    
    /// Optimize database for better performance
    pub async fn optimize(&self) -> Result<(), SqlxError> {
        info!("Running database optimization");
        
        let start = Instant::now();
        
        // Run SQLite optimization commands
        sqlx::query("PRAGMA optimize").execute(&self.pool).await?;
        sqlx::query("ANALYZE").execute(&self.pool).await?;
        
        // Rebuild FTS index if it exists
        let _ = sqlx::query("INSERT INTO messages_fts(messages_fts) VALUES('rebuild')")
            .execute(&self.pool)
            .await; // Ignore error if FTS table doesn't exist
        
        let duration = start.elapsed();
        info!("Database optimization completed in {:?}", duration);
        
        Ok(())
    }
    
    /// Get database statistics for monitoring
    pub async fn get_stats(&self) -> Result<DatabaseStats, SqlxError> {
        let start = Instant::now();
        
        // Get basic statistics
        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        
        let message_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages")
            .fetch_one(&self.pool)
            .await?;
        
        let room_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rooms")
            .fetch_one(&self.pool)
            .await?;
        
        // Get database file size
        let db_size: i64 = sqlx::query_scalar("SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        
        let query_time = start.elapsed();
        
        Ok(DatabaseStats {
            user_count: user_count as u64,
            message_count: message_count as u64,
            room_count: room_count as u64,
            database_size_bytes: db_size as u64,
            connection_count: self.pool.size(),
            idle_connections: self.pool.num_idle() as u32,
            query_time_ms: query_time.as_millis() as u64,
        })
    }
    
    /// Health check for the database connection
    pub async fn health_check(&self) -> Result<(), SqlxError> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseStats {
    pub user_count: u64,
    pub message_count: u64,
    pub room_count: u64,
    pub database_size_bytes: u64,
    pub connection_count: u32,
    pub idle_connections: u32,
    pub query_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_optimized_pool_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let database_url = format!("sqlite:{}", temp_file.path().display());
        
        let pool = OptimizedConnectionPool::new(&database_url, None).await.unwrap();
        
        // Test basic connectivity
        pool.health_check().await.unwrap();
        
        // Test configuration
        assert_eq!(pool.config().max_connections, 50);
        assert_eq!(pool.config().min_connections, 5);
    }
    
    #[tokio::test]
    async fn test_monitored_queries() {
        let temp_file = NamedTempFile::new().unwrap();
        let database_url = format!("sqlite:{}", temp_file.path().display());
        
        let pool = OptimizedConnectionPool::new(&database_url, None).await.unwrap();
        
        // Create a test table
        let query = sqlx::query("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)");
        pool.execute_monitored(query, "create_table").await.unwrap();
        
        // Insert test data
        let query = sqlx::query("INSERT INTO test (name) VALUES ('test')");
        pool.execute_monitored(query, "insert_test").await.unwrap();
        
        // Query test data
        let query = sqlx::query_as::<_, (i64, String)>("SELECT id, name FROM test");
        let results = pool.fetch_all_monitored(query, "select_test").await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "test");
    }
    
    #[tokio::test]
    async fn test_database_optimization() {
        let temp_file = NamedTempFile::new().unwrap();
        let database_url = format!("sqlite:{}", temp_file.path().display());
        
        let pool = OptimizedConnectionPool::new(&database_url, None).await.unwrap();
        
        // Run optimization
        pool.optimize().await.unwrap();
        
        // Should still be healthy after optimization
        pool.health_check().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_database_stats() {
        let temp_file = NamedTempFile::new().unwrap();
        let database_url = format!("sqlite:{}", temp_file.path().display());
        
        let pool = OptimizedConnectionPool::new(&database_url, None).await.unwrap();
        
        // Create tables to get meaningful stats
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY)")
            .execute(pool.pool())
            .await
            .unwrap();
        
        sqlx::query("CREATE TABLE messages (id INTEGER PRIMARY KEY)")
            .execute(pool.pool())
            .await
            .unwrap();
        
        sqlx::query("CREATE TABLE rooms (id INTEGER PRIMARY KEY)")
            .execute(pool.pool())
            .await
            .unwrap();
        
        let stats = pool.get_stats().await.unwrap();
        
        assert_eq!(stats.user_count, 0);
        assert_eq!(stats.message_count, 0);
        assert_eq!(stats.room_count, 0);
        assert!(stats.database_size_bytes > 0);
    }
}