//! CozoDB Connection Pool
//!
//! High-performance connection pooling for CozoDB with health monitoring,
//! automatic recovery, and performance optimization for 1000+ records/minute throughput.

use crate::cozodb::{
    error::{CozoError, CozoResult},
    connection::{CozoConnection, ConnectionConfig},
    record::PoolInfo,
};
use std::sync::{Arc, RwLock};
use tokio::time::{Duration, Instant};
use futures::future::BoxFuture;

/// Configuration for the connection pool
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Database connection URL
    pub url: String,

    /// Maximum number of connections in the pool
    pub max_connections: usize,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Idle timeout for connections
    pub idle_timeout: Duration,

    /// Health check interval
    pub health_check_interval: Duration,

    /// Maximum retry attempts for failed operations
    pub max_retry_attempts: usize,

    /// Base delay for exponential backoff
    pub retry_base_delay: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            url: "cozodb://./cozo.db".to_string(),
            max_connections: 10,
            connection_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(10),
            max_retry_attempts: 3,
            retry_base_delay: Duration::from_millis(100),
        }
    }
}

/// High-performance connection pool for CozoDB
#[derive(Debug)]
pub struct CozoConnectionPool {
    /// Pool configuration
    config: ConnectionPoolConfig,

    /// Available connections in the pool
    available_connections: Arc<RwLock<Vec<CozoConnection>>>,

    /// All connections managed by the pool
    all_connections: Arc<RwLock<Vec<CozoConnection>>>,

    /// Pool statistics and metadata
    pool_info: Arc<RwLock<PoolInfo>>,

    /// Pool health status
    health_status: Arc<RwLock<bool>>,

    /// Background health check task handle
    health_check_handle: Option<tokio::task::JoinHandle<()>>,
}

impl CozoConnectionPool {
    /// Create a new connection pool
    pub async fn new(config: ConnectionPoolConfig) -> CozoResult<Self> {
        let pool = Self {
            config: config.clone(),
            available_connections: Arc::new(RwLock::new(Vec::with_capacity(config.max_connections))),
            all_connections: Arc::new(RwLock::new(Vec::with_capacity(config.max_connections))),
            pool_info: Arc::new(RwLock::new(PoolInfo::new(config.max_connections))),
            health_status: Arc::new(RwLock::new(true)),
            health_check_handle: None,
        };

        // Initialize the pool with connections
        pool.initialize_pool().await?;

        // Start background health monitoring
        let health_pool = pool.clone();
        tokio::spawn(async move {
            health_pool.health_monitor_loop().await;
        });

        // Note: In a real implementation, we would store the handle
        // For now, we let it run in the background

        Ok(pool)
    }

    /// Initialize the pool with connections
    async fn initialize_pool(&self) -> CozoResult<()> {
        let connection_config = ConnectionConfig::default();

        for _ in 0..self.config.max_connections {
            let connection = CozoConnection::new(
                &self.config.url,
                connection_config.clone(),
            ).await?;

            self.all_connections.write().unwrap().push(connection.clone());
            self.available_connections.write().unwrap().push(connection);
        }

        {
            let mut pool_info = self.pool_info.write().unwrap();
            pool_info.idle_connections = self.config.max_connections;
        }

        Ok(())
    }

    /// Acquire a connection from the pool
    pub async fn acquire_connection(&self) -> CozoResult<CozoConnection> {
        let start_time = Instant::now();

        // Check for available connection
        {
            let mut available = self.available_connections.write().unwrap();
            if let Some(connection) = available.pop() {
                // Verify connection is healthy
                if connection.is_healthy().await.unwrap_or(false) {
                    {
                        let mut pool_info = self.pool_info.write().unwrap();
                        pool_info.active_connections += 1;
                        pool_info.idle_connections -= 1;
                        pool_info.total_acquired += 1;
                    }

                    return Ok(connection);
                } else {
                    // Remove unhealthy connection and create new one
                    self.replace_unhealthy_connection(connection).await?;
                }
            }
        }

        // No available connections, wait for one to be released
        // In a real implementation, this would use a semaphore or notification system
        // For now, we simulate the wait
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Try again after waiting
        {
            let available = self.available_connections.read().unwrap();
            if let Some(connection) = available.last() {
                let connection = connection.clone();
                drop(available);

                {
                    let mut pool_info = self.pool_info.write().unwrap();
                    pool_info.active_connections += 1;
                    pool_info.idle_connections -= 1;
                    pool_info.total_acquired += 1;
                }

                return Ok(connection);
            }
        }

        // If we reach here, pool is exhausted
        {
            let pool_info = self.pool_info.read().unwrap();
            Err(CozoError::resource_limit_exhausted(
                "connections",
                pool_info.active_connections,
                self.config.max_connections,
            ))
        }
    }

    /// Release a connection back to the pool
    pub async fn release_connection(&self, connection: CozoConnection) -> CozoResult<()> {
        // Verify connection is healthy before returning to pool
        if connection.is_healthy().await.unwrap_or(false) {
            self.available_connections.write().unwrap().push(connection);

            {
                let mut pool_info = self.pool_info.write().unwrap();
                pool_info.active_connections -= 1;
                pool_info.idle_connections += 1;
                pool_info.total_released += 1;
            }
        } else {
            // Replace unhealthy connection
            self.replace_unhealthy_connection(connection).await?;
        }

        Ok(())
    }

    /// Replace an unhealthy connection
    async fn replace_unhealthy_connection(&self, _old_connection: CozoConnection) -> CozoResult<()> {
        let connection_config = ConnectionConfig::default();
        let new_connection = CozoConnection::new(
            &self.config.url,
            connection_config,
        ).await?;

        // Update the connection in all_connections
        {
            let mut all_connections = self.all_connections.write().unwrap();
            if let Some(pos) = all_connections.iter().position(|c| c.id == _old_connection.id) {
                all_connections[pos] = new_connection.clone();
            }
        }

        self.available_connections.write().unwrap().push(new_connection);

        Ok(())
    }

    /// Check if the pool is healthy
    pub fn is_healthy(&self) -> bool {
        *self.health_status.read().unwrap()
    }

    /// Get the number of active connections
    pub fn active_connections(&self) -> usize {
        self.pool_info.read().unwrap().active_connections
    }

    /// Get the maximum number of connections
    pub fn max_connections(&self) -> usize {
        self.config.max_connections
    }

    /// Get pool information
    pub fn pool_info(&self) -> PoolInfo {
        self.pool_info.read().unwrap().clone()
    }

    /// Background health monitoring loop
    async fn health_monitor_loop(&self) {
        let mut interval = tokio::time::interval(self.config.health_check_interval);

        loop {
            interval.tick().await;

            // Check overall pool health
            let available_count = self.available_connections.read().unwrap().len();
            let total_count = self.all_connections.read().unwrap().len();
            let healthy_ratio = if total_count > 0 {
                available_count as f64 / total_count as f64
            } else {
                0.0
            };

            let is_healthy = healthy_ratio >= 0.5; // At least 50% connections available
            *self.health_status.write().unwrap() = is_healthy;

            // Log health status if changed
            if !is_healthy {
                tracing::warn!(
                    "CozoDB connection pool unhealthy: {}/{} connections available",
                    available_count, total_count
                );
            }
        }
    }

    /// Simulate database failure (for testing)
    pub async fn simulate_database_failure(&self) -> CozoResult<()> {
        *self.health_status.write().unwrap() = false;

        // Mark all connections as unhealthy
        let _all_connections = self.all_connections.read().unwrap();
        // In real implementation, would actually break connections
        // For simulation, we just mark the pool as unhealthy

        Ok(())
    }

    /// Simulate database recovery (for testing)
    pub async fn simulate_database_recovery(&self) -> CozoResult<()> {
        *self.health_status.write().unwrap() = true;

        // Reinitialize all connections
        self.available_connections.write().unwrap().clear();
        self.all_connections.write().unwrap().clear();
        self.initialize_pool().await?;

        Ok(())
    }

    /// Get performance metrics
    pub fn performance_metrics(&self) -> ConnectionPoolMetrics {
        let pool_info = self.pool_info.read().unwrap();
        ConnectionPoolMetrics {
            active_connections: pool_info.active_connections,
            idle_connections: pool_info.idle_connections,
            total_acquired: pool_info.total_acquired,
            total_released: pool_info.total_released,
            utilization_rate: pool_info.utilization_rate(),
            is_healthy: self.is_healthy(),
        }
    }
}

/// Performance metrics for the connection pool
#[derive(Debug, Clone)]
pub struct ConnectionPoolMetrics {
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_acquired: u64,
    pub total_released: u64,
    pub utilization_rate: f64,
    pub is_healthy: bool,
}

impl Clone for CozoConnectionPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            available_connections: Arc::clone(&self.available_connections),
            all_connections: Arc::clone(&self.all_connections),
            pool_info: Arc::clone(&self.pool_info),
            health_status: Arc::clone(&self.health_status),
            health_check_handle: None, // Don't clone the background task
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let config = ConnectionPoolConfig {
            url: "cozodb://./test.cozo".to_string(),
            max_connections: 5,
            connection_timeout: Duration::from_secs(3),
            idle_timeout: Duration::from_secs(15),
            health_check_interval: Duration::from_secs(5),
            max_retry_attempts: 3,
            retry_base_delay: Duration::from_millis(100),
        };

        let pool = CozoConnectionPool::new(config).await.unwrap();
        assert!(pool.is_healthy());
        assert_eq!(pool.active_connections(), 0);
        assert_eq!(pool.max_connections(), 5);
    }

    #[tokio::test]
    async fn test_connection_lifecycle() {
        let pool = CozoConnectionPool::new(ConnectionPoolConfig::default()).await.unwrap();

        let connection1 = pool.acquire_connection().await.unwrap();
        let connection2 = pool.acquire_connection().await.unwrap();

        assert_eq!(pool.active_connections(), 2);

        pool.release_connection(connection1).await.unwrap();
        pool.release_connection(connection2).await.unwrap();

        assert_eq!(pool.active_connections(), 0);
    }
}