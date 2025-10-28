//! GREEN PHASE: Simple Mock Database Implementation
//!
//! A minimal implementation that demonstrates the GREEN phase working
//! with the TDD approach without complex trait dependencies.

use std::time::Duration;

/// GREEN PHASE: Simple mock database provider
#[derive(Debug)]
pub struct SimpleMockDatabaseProvider {
    connection_string: String,
    latency_ms: u64,
}

impl SimpleMockDatabaseProvider {
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
            latency_ms: 10, // Default latency well within contract
        }
    }

    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    /// Validate connection string format
    fn validate_connection_string(&self) -> Result<(), SimpleError> {
        if self.connection_string.is_empty() {
            return Err(SimpleError::InvalidConnectionString);
        }

        if !self.connection_string.contains("://") {
            return Err(SimpleError::InvalidConnectionString);
        }

        Ok(())
    }
}

/// Simple connection implementation
#[derive(Debug)]
pub struct SimpleMockConnection {
    id: String,
    created_at: std::time::Instant,
    healthy: bool,
}

impl SimpleMockConnection {
    pub async fn is_healthy(&self) -> Result<bool, SimpleError> {
        // Simulate health check
        tokio::time::sleep(Duration::from_millis(1)).await;
        Ok(self.healthy)
    }

    pub async fn close(&self) -> Result<(), SimpleError> {
        // Simulate cleanup
        tokio::time::sleep(Duration::from_millis(1)).await;
        println!("Connection {} closed", self.id);
        Ok(())
    }
}

/// Simple database provider implementation
impl SimpleMockDatabaseProvider {
    pub async fn connect(&self) -> Result<SimpleMockConnection, SimpleError> {
        let start_time = std::time::Instant::now();

        // Simulate connection establishment
        tokio::time::sleep(Duration::from_millis(self.latency_ms)).await;

        // Validate connection string
        self.validate_connection_string()?;

        let connection = SimpleMockConnection {
            id: format!("conn_{}", uuid::Uuid::new_v4()),
            created_at: start_time,
            healthy: true,
        };

        println!("GREEN PHASE: Mock connection {} established", connection.id);

        // Check performance contract: < 100ms
        let elapsed = start_time.elapsed();
        if elapsed > Duration::from_millis(100) {
            return Err(SimpleError::PerformanceViolation {
                operation: "connect".to_string(),
                actual_ms: elapsed.as_millis() as u64,
                limit_ms: 100,
            });
        }

        Ok(connection)
    }

    pub async fn health_check(&self) -> Result<SimpleHealthStatus, SimpleError> {
        let start_time = std::time::Instant::now();

        // Simulate health check
        tokio::time::sleep(Duration::from_millis(1)).await;

        // Check performance contract: < 50ms
        let elapsed = start_time.elapsed();
        if elapsed > Duration::from_millis(50) {
            return Err(SimpleError::PerformanceViolation {
                operation: "health_check".to_string(),
                actual_ms: elapsed.as_millis() as u64,
                limit_ms: 50,
            });
        }

        Ok(SimpleHealthStatus::Healthy)
    }
}

/// Simple error types for GREEN phase
#[derive(Debug, thiserror::Error)]
pub enum SimpleError {
    #[error("Invalid connection string")]
    InvalidConnectionString,

    #[error("Performance violation: {operation} took {actual_ms}ms (limit: {limit_ms}ms)")]
    PerformanceViolation {
        operation: String,
        actual_ms: u64,
        limit_ms: u64,
    },

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}

#[derive(Debug, Clone)]
pub enum SimpleHealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

/// GREEN PHASE: Test factory for creating configured mock implementations
pub struct SimpleMockTestFactory {
    pub latency_ms: u64,
}

impl SimpleMockTestFactory {
    pub fn new() -> Self {
        Self { latency_ms: 10 }
    }

    pub fn database_provider(&self) -> SimpleMockDatabaseProvider {
        SimpleMockDatabaseProvider::new("test://localhost")
            .with_latency(self.latency_ms)
    }

    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = latency_ms;
        self
    }
}

impl Default for SimpleMockTestFactory {
    fn default() -> Self {
        Self::new()
    }
}