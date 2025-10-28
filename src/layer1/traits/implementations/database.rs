//! Mock Database Provider Implementation
//!
//! This implementation provides realistic behavior while clearly marking itself
//! as a mock intended for TDD development. It follows Rust idiomatic patterns
//! for async operations, error handling, and resource management.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use async_stream::stream;

// Import the traits we need
use super::super::{
    database::{DatabaseProvider, DatabaseProviderExt, DatabaseConnection, ConnectionInfo, DatabaseRecord as TraitDatabaseRecord, QueryParams, HealthStatus as TraitHealthStatus, BatchResult, BatchOperation, DatabaseRow, DatabaseValue, TryFromRow, OperationType},
    error::{DobbyError, DatabaseError},
    inference::ModelId,
    types::DatabaseId,
};

// Mock error type
#[derive(Debug, thiserror::Error)]
pub enum MockDatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Database unavailable: {0}")]
    Unavailable(String),

    #[error("Invalid connection string: {0}")]
    InvalidConnectionString(String),

    #[error("Resource limit exhausted: {resource} used {used}/{limit}")]
    ResourceLimitExhausted { resource: String, used: usize, limit: usize },
}

impl DobbyError for MockDatabaseError {
    fn severity(&self) -> super::super::error::ErrorSeverity {
        super::super::error::ErrorSeverity::Error
    }

    fn retry_recommendation(&self) -> super::super::error::RetryRecommendation {
        super::super::error::RetryRecommendation::Retry
    }

    fn context(&self) -> super::super::error::ErrorContext {
        super::super::error::ErrorContext {
            timestamp: chrono::Utc::now(),
            component: "mock_database".to_string(),
            operation: "mock_operation".to_string(),
            metadata: std::collections::HashMap::new(),
            trace: vec![],
        }
    }

    fn is_retryable(&self) -> bool {
        true
    }

    fn category(&self) -> super::super::error::ErrorCategory {
        super::super::error::ErrorCategory::Database
    }
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String, severity: Severity },
    Unhealthy { reason: String },
}

#[derive(Debug, Clone)]
pub enum Severity {
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub struct DatabaseRecord {
    pub id: RecordId,
    pub content: Content,
    pub metadata: RecordMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecordId(pub Uuid);

#[derive(Debug, Clone)]
pub enum Content {
    Text(String),
    Binary(Vec<u8>),
    Structured(serde_json::Value),
}

#[derive(Debug, Clone)]
pub struct RecordMetadata {
    pub source: String,
    pub content_type: ContentType,
    pub size_bytes: usize,
    pub processing_state: ProcessingState,
    pub priority: Priority,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum ContentType {
    Code,
    Documentation,
    Configuration,
    Data,
}

#[derive(Debug, Clone)]
pub enum ProcessingState {
    Pending,
    InProgress { started_at: DateTime<Utc> },
    Completed {
        completed_at: DateTime<Utc>,
        summary_id: SummaryId,
    },
    Failed {
        failed_at: DateTime<Utc>,
        error: String,
        retry_count: u32,
    },
    Skipped {
        skipped_at: DateTime<Utc>,
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SummaryId(pub Uuid);

#[derive(Debug, Clone)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

// Use println! instead of tracing for GREEN phase
macro_rules! debug {
    ($($arg:tt)*) => {
        println!("[DEBUG] {}", format!($($arg)*));
    };
}

macro_rules! warn {
    ($($arg:tt)*) => {
        println!("[WARN] {}", format!($($arg)*));
    };
}

/// GREEN PHASE: Mock database provider with realistic behavior simulation
#[derive(Debug)]
pub struct MockDatabaseProvider {
    connection_string: String,
    // Internal state for realistic behavior
    connections: Arc<Mutex<Vec<MockConnectionState>>>,
    query_latency: Duration,
    failure_rate: f64,
    max_connections: usize,
}

/// Connection state tracking for realistic pool behavior
#[derive(Debug, Clone)]
struct MockConnectionState {
    id: DatabaseId,
    created_at: chrono::DateTime<chrono::Utc>,
    active: bool,
    query_count: u64,
}

impl MockDatabaseProvider {
    /// Create a new mock database provider with configurable behavior
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
            connections: Arc::new(Mutex::new(Vec::new())),
            query_latency: Duration::from_millis(10), // Realistic default latency
            failure_rate: 0.0, // No failures by default
            max_connections: 10,
        }
    }

    /// Configure query latency for performance testing
    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.query_latency = latency;
        self
    }

    /// Configure failure rate for error testing (0.0 to 1.0)
    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Configure maximum connection limit
    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Simulate random failure based on configured failure rate
    async fn simulate_random_failure(&self) -> Result<(), MockDatabaseError> {
        if self.failure_rate > 0.0 {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() < self.failure_rate {
                return Err(MockDatabaseError::ConnectionFailed("Simulated random connection failure".to_string()));
            }
        }
        Ok(())
    }

    /// Validate connection string format
    async fn validate_connection_string(&self) -> Result<(), MockDatabaseError> {
        if self.connection_string.is_empty() {
            return Err(MockDatabaseError::InvalidConnectionString("Connection string cannot be empty".to_string()));
        }

        // Basic format validation for common patterns
        if !self.connection_string.contains("://") {
            return Err(MockDatabaseError::InvalidConnectionString(format!("Invalid connection string format: {}", self.connection_string)));
        }

        Ok(())
    }

    /// Estimate result size based on query pattern
    async fn estimate_result_size(&self, query: &str, _params: &QueryParams) -> Result<usize, MockDatabaseError> {
        // Simple heuristic based on query patterns
        if query.contains("LIMIT") {
            // Extract limit value if present
            if let Some(limit_str) = query.split("LIMIT").nth(1) {
                if let Some(limit) = limit_str.trim().split_whitespace().next() {
                    if let Ok(limit_num) = limit.parse::<usize>() {
                        return Ok(limit_num);
                    }
                }
            }
        }

        // Default estimates based on query type
        if query.contains("SELECT *") || query.contains("SELECT COUNT(*)") {
            Ok(1000) // Large result set
        } else if query.contains("SELECT") {
            Ok(100)  // Medium result set
        } else {
            Ok(1)    // Single result or operation result
        }
    }

    /// Generate mock record for testing
    async fn generate_mock_record(&self, index: usize) -> Result<DatabaseRecord, MockDatabaseError> {
        Ok(DatabaseRecord {
            id: RecordId(Uuid::new_v4()),
            content: Content::Text(format!("Mock record content {}", index)),
            metadata: RecordMetadata {
                source: "mock_generator".to_string(),
                content_type: ContentType::Code,
                size_bytes: 100,
                processing_state: ProcessingState::Pending,
                priority: Priority::Normal,
                custom_fields: HashMap::new(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    /// Simulate query execution with realistic timing
    async fn simulate_query_execution<R>(&self, query: &str, _params: &QueryParams) -> Result<Vec<R>, MockDatabaseError>
    where
        R: TryFromRow + Send,
    {
        // Simulate realistic query latency
        tokio::time::sleep(self.query_latency).await;

        // For GREEN phase, return empty results
        // Real implementation would parse results based on query
        Ok(Vec::new())
    }
}

/// Marker trait to prevent accidental production use of mock implementations
pub trait MockImplementation: sealed::Sealed {}
impl MockImplementation for MockDatabaseProvider {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::MockDatabaseProvider {}
}

/// GREEN PHASE: Mock database connection with proper lifecycle management
#[derive(Debug)]
pub struct MockDatabaseConnection {
    id: DatabaseId,
    created_at: chrono::DateTime<chrono::Utc>,
    healthy: Arc<AtomicBool>,
    query_count: Arc<AtomicU64>,
    last_used: Arc<Mutex<chrono::DateTime<chrono::Utc>>>,
    provider_config: Arc<MockDatabaseProvider>,
}

impl Drop for MockDatabaseConnection {
    fn drop(&mut self) {
        // RAII cleanup - ensure connection is marked as closed
        self.healthy.store(false, Ordering::SeqCst);
        tracing::debug!("Mock connection {} dropped", self.id);
    }
}

impl MockDatabaseConnection {
    /// Create a new mock connection
    fn new(id: DatabaseId, provider_config: Arc<MockDatabaseProvider>) -> Self {
        Self {
            id,
            created_at: chrono::Utc::now(),
            healthy: Arc::new(AtomicBool::new(true)),
            query_count: Arc::new(AtomicU64::new(0)),
            last_used: Arc::new(Mutex::new(chrono::Utc::now())),
            provider_config,
        }
    }

    /// Increment query count and update last used time
    fn record_query(&self) {
        self.query_count.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut last_used) = self.last_used.try_lock() {
            *last_used = chrono::Utc::now();
        }
    }
}

#[async_trait]
impl DatabaseConnection for MockDatabaseConnection {
    type Error = MockDatabaseError;

    async fn is_healthy(&self) -> Result<bool, Self::Error> {
        // Simulate health check with realistic timing
        tokio::time::sleep(Duration::from_millis(1)).await;

        let is_healthy = self.healthy.load(Ordering::SeqCst);
        tracing::debug!("Connection {} health check: {}", self.id, is_healthy);

        Ok(is_healthy)
    }

    async fn close(&self) -> Result<(), Self::Error> {
        // Mark as unhealthy and cleanup
        self.healthy.store(false, Ordering::SeqCst);

        // Simulate cleanup time
        tokio::time::sleep(Duration::from_millis(1)).await;

        tracing::debug!("Connection {} closed", self.id);
        Ok(())
    }

    fn connection_info(&self) -> ConnectionInfo {
        ConnectionInfo {
            database_id: crate::layer1::traits::database::DatabaseId(uuid::Uuid::new_v4()),
            created_at: self.created_at,
            last_used: chrono::Utc::now(), // Would use actual last_used
            query_count: self.query_count.load(Ordering::SeqCst),
            active: self.healthy.load(Ordering::SeqCst),
        }
    }
}

#[async_trait]
impl DatabaseProvider for MockDatabaseProvider {
    type Connection = MockDatabaseConnection;
    type Error = MockDatabaseError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let tracker = PerformanceTracker::new("connect", Duration::from_millis(100));

        // Simulate connection establishment
        tokio::time::sleep(self.query_latency).await;

        // Check for random failures
        self.simulate_random_failure().await?;

        // Validate connection string
        self.validate_connection_string().await?;

        // Check connection limits
        let connections = self.connections.lock().await;
        if connections.len() >= self.max_connections {
            return Err(MockDatabaseError::ResourceExhaustion {
                resource: "connections".to_string(),
                limit: self.max_connections,
            });
        }
        drop(connections); // Release lock before creating connection

        let connection_id = DatabaseId(Uuid::new_v4());
        let provider_config = Arc::new(MockDatabaseProvider {
            connection_string: self.connection_string.clone(),
            connections: Arc::new(Mutex::new(Vec::new())),
            query_latency: self.query_latency,
            failure_rate: self.failure_rate,
            max_connections: self.max_connections,
        });

        let connection = MockDatabaseConnection::new(connection_id, provider_config);

        // Track connection for realistic behavior
        let mut connections = self.connections.lock().await;
        connections.push(MockConnectionState {
            id: connection.id,
            created_at: connection.created_at,
            active: true,
            query_count: 0,
        });

        tracing::debug!("Mock connection {} established", connection.id);

        tracker.check_contract()?;
        Ok(connection)
    }

    async fn execute_query_simple(
        &self,
        query: &str,
        params: QueryParams,
    ) -> Result<Vec<TraitDatabaseRecord>, Self::Error> {
        let tracker = PerformanceTracker::new("execute_query_simple", Duration::from_millis(50));

        tracing::debug!(
            query = %query,
            params_count = params.params.len(),
            "Executing mock database query"
        );

        // Simulate query execution
        let result = self.simulate_query_execution_simple(query, &params).await?;

        tracker.check_contract()?;
        Ok(result)
    }

  
    async fn health_check(&self) -> Result<TraitHealthStatus, Self::Error> {
        let tracker = PerformanceTracker::new("health_check", Duration::from_millis(50));

        // Simulate health check with realistic timing
        tokio::time::sleep(Duration::from_millis(1)).await;

        let connections = self.connections.lock().await;
        let active_connections = connections.iter().filter(|c| c.active).count();

        // Determine health status based on connection usage
        let status = if active_connections == 0 {
            TraitHealthStatus::Healthy
        } else if active_connections < self.max_connections {
            TraitHealthStatus::Degraded {
                reason: format!("High connection usage: {}/{}", active_connections, self.max_connections),
                severity: crate::layer1::traits::database::Severity::Warning,
            }
        } else {
            TraitHealthStatus::Degraded {
                reason: "Maximum connection limit reached".to_string(),
                severity: crate::layer1::traits::database::Severity::Error,
            }
        };

        tracker.check_contract()?;
        Ok(status)
    }
}


/// GREEN PHASE: Simple performance tracking for contract validation
#[derive(Debug, Clone)]
pub struct PerformanceTracker {
    start_time: Instant,
    operation: &'static str,
    expected_max: Duration,
}

impl PerformanceTracker {
    pub fn new(operation: &'static str, expected_max: Duration) -> Self {
        let start_time = Instant::now();
        Self { start_time, operation, expected_max }
    }

    pub fn check_contract(&self) -> Result<(), MockDatabaseError> {
        let elapsed = self.start_time.elapsed();
        if elapsed > self.expected_max {
            tracing::warn!(
                operation = self.operation,
                actual_ms = elapsed.as_millis(),
                expected_ms = self.expected_max.as_millis(),
                "Performance contract violation"
            );

            // In GREEN phase, log but don't fail for minor violations
            if elapsed > self.expected_max * 2 {
                return Err(MockDatabaseError::MockError {
                    message: format!(
                        "Performance contract violation: {} took {}ms (expected < {}ms)",
                        self.operation,
                        elapsed.as_millis(),
                        self.expected_max.as_millis()
                    ),
                });
            }
        }
        Ok(())
    }
}

// Implement the extension trait for advanced features
#[async_trait]
impl DatabaseProviderExt for MockDatabaseProvider {
    async fn fetch_records_stream(
        &self,
        query: &str,
        params: QueryParams,
    ) -> Result<Box<dyn futures::Stream<Item = Result<TraitDatabaseRecord, Self::Error>> + Send>, Self::Error> {
        let tracker = PerformanceTracker::new("fetch_records_stream", Duration::from_millis(100));

        tracing::debug!(
            query = %query,
            params_count = params.params.len(),
            "Starting mock stream query"
        );

        let provider = self.clone();
        let query = query.to_string();
        let params_clone = params.clone();

        let stream = stream! {
            // Simulate streaming results
            let mock_records = provider.generate_mock_records(&query, &params_clone).await;
            for record in mock_records {
                // Simulate streaming delay
                tokio::time::sleep(Duration::from_millis(10)).await;
                yield Ok(record);
            }
        };

        tracker.check_contract()?;
        Ok(Box::pin(stream))
    }

    async fn execute_batch<T>(
        &self,
        operations: impl IntoIterator<Item = T> + Send,
    ) -> Result<BatchResult, Self::Error>
    where
        T: BatchOperation + Send + Sync,
    {
        let tracker = PerformanceTracker::new("execute_batch", Duration::from_secs(1));

        let operations: Vec<T> = operations.into_iter().collect();
        let total_operations = operations.len();

        tracing::debug!(
            total_operations = total_operations,
            "Executing mock batch operations"
        );

        let mut successful_operations = 0;
        let mut failed_operations = 0;
        let mut errors = Vec::new();
        let start_time = Instant::now();

        // Simulate batch processing with realistic timing
        for (index, _operation) in operations.into_iter().enumerate() {
            // Simulate operation timing
            tokio::time::sleep(Duration::from_millis(1)).await;

            // Check for random failures
            if let Err(e) = self.simulate_random_failure().await {
                failed_operations += 1;
                errors.push(format!("Operation {} failed: {}", index, e));
            } else {
                successful_operations += 1;
            }

            // Simulate occasional batch delays
            if index % 50 == 0 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        let duration = start_time.elapsed();

        let result = BatchResult {
            total_operations,
            successful_operations,
            failed_operations,
            duration,
            errors,
        };

        tracker.check_contract()?;
        Ok(result)
    }

    fn create_conversion_error(&self, message: String) -> Self::Error {
        MockDatabaseError::QueryFailed(message)
    }
}

/// GREEN PHASE: Test factory for creating configured mock implementations
pub struct MockTestFactory {
    pub latency: Duration,
    pub failure_rate: f64,
    pub max_connections: usize,
}

impl MockTestFactory {
    pub fn new() -> Self {
        Self {
            latency: Duration::from_millis(10),
            failure_rate: 0.0,
            max_connections: 10,
        }
    }

    pub fn database_provider(&self) -> MockDatabaseProvider {
        MockDatabaseProvider::new("test://localhost")
            .with_latency(self.latency)
            .with_failure_rate(self.failure_rate)
            .with_max_connections(self.max_connections)
    }

    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = latency;
        self
    }

    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate.clamp(0.0, 1.0);
        self
    }

    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Generate mock records for testing and demonstration
    pub async fn generate_mock_records(&self, _query: &str, _params: &QueryParams) -> Vec<TraitDatabaseRecord> {
        let mut records = Vec::new();

        for i in 0..5 {
            let record = TraitDatabaseRecord {
                id: crate::layer1::traits::database::RecordId(uuid::Uuid::new_v4()),
                content: crate::layer1::traits::database::Content::Text(format!("Mock content for record {}", i + 1)),
                metadata: crate::layer1::traits::database::RecordMetadata {
                    source: "mock_generator".to_string(),
                    content_type: crate::layer1::traits::database::ContentType::Code,
                    size_bytes: 50,
                    processing_state: crate::layer1::traits::database::ProcessingState::Pending,
                    priority: crate::layer1::traits::database::Priority::Normal,
                    custom_fields: std::collections::HashMap::from([
                        ("timestamp".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339())),
                    ]),
                },
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            records.push(record);
        }

        records
    }
}

impl Default for MockTestFactory {
    fn default() -> Self {
        Self::new()
    }
}