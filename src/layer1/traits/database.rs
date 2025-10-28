//! Database Provider Trait
//!
//! ## Executable Specification Contract
//!
//! ### Preconditions:
//! - Valid connection string provided
//! - Sufficient system resources available
//! - Network connectivity (if remote database)
//!
//! ### Postconditions:
//! - Returns functional connection on success
//! - Maintains connection pool within limits
//! - Provides query execution with parameter binding
//!
//! ### Error Conditions:
//! - Connection string invalid → DatabaseError::InvalidConnectionString
//! - Network unreachable → DatabaseError::ConnectionFailed
//! - Insufficient resources → DatabaseError::ResourceExhaustion
//! - Query syntax error → DatabaseError::InvalidQuery
//!
//! ### Performance Contracts:
//! - Connection acquisition: < 100ms (p95)
//! - Query execution (100 records): < 500ms (p95)
//! - Connection pool efficiency: > 90%
//! - Memory usage: < 100MB per 10 connections

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::error::{DatabaseError, DobbyError};

/// Strongly-typed database identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatabaseId(pub uuid::Uuid);

/// Database connection trait with lifecycle management
pub trait DatabaseConnection: Send + Sync {
    type Error: DobbyError + Send + Sync + 'static;

    /// Check if connection is healthy and ready for queries
    async fn is_healthy(&self) -> Result<bool, Self::Error>;

    /// Close connection and cleanup resources
    async fn close(&self) -> Result<(), Self::Error>;

    /// Get connection metadata
    fn connection_info(&self) -> ConnectionInfo;
}

/// Core database abstraction with async support and comprehensive error handling
#[async_trait]
pub trait DatabaseProvider: Send + Sync + 'static {
    /// Connection type with lifetime management
    type Connection: DatabaseConnection + Send + Sync;
    /// Error type with detailed context
    type Error: DobbyError + Send + Sync + 'static;

    /// Establish connection with automatic retry and circuit breaking
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Valid configuration provided
    /// - System has sufficient memory
    ///
    /// ### Postconditions:
    /// - Returns Ok(connection) on success
    /// - Connection is ready for queries
    /// - Connection added to pool tracking
    ///
    /// ### Performance Contract:
    /// - Completion time: < 1 second
    /// - Memory allocation: < 10MB
    async fn connect(&self) -> Result<Self::Connection, Self::Error>;

    /// Execute query with parameter binding and result mapping
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Valid SQL query syntax
    /// - Parameters match query placeholders
    /// - Connection is active
    ///
    /// ### Postconditions:
    /// - Returns Vec<R> on successful execution
    /// - All rows properly mapped to result type
    /// - Connection remains valid after execution
    ///
    /// ### Performance Contract:
    /// - Single record query: < 50ms
    /// - 100 record query: < 500ms
    /// - 1000 record query: < 5000ms
    async fn execute_query<Q, P, R>(
        &self,
        query: Q,
        params: P,
    ) -> Result<Vec<R>, Self::Error>
    where
        Q: AsRef<str> + Send + Sync,
        P: Into<QueryParams> + Send,
        R: TryFromRow + Send;

    /// Fetch records with streaming support for large datasets
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Valid query with stream-compatible result set
    /// - Sufficient memory for stream buffer
    ///
    /// ### Postconditions:
    /// - Returns stream that yields results
    /// - Stream handles backpressure properly
    /// - Connection remains valid during streaming
    ///
    /// ### Performance Contract:
    /// - First result: < 100ms
    /// - Throughput: > 1000 records/second
    /// - Memory usage: < 50MB buffer
    async fn fetch_records_stream(
        &self,
        query: &str,
        params: QueryParams,
    ) -> Result<impl futures::Stream<Item = Result<DatabaseRecord, Self::Error>> + Send, Self::Error>;

    /// Batch operation with transaction support
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - All operations are valid
    /// - Sufficient transaction log space
    ///
    /// ### Postconditions:
    /// - All operations executed atomically
    /// - Returns success count and any failures
    /// - Database remains consistent
    ///
    /// ### Performance Contract:
    /// - 100 operations: < 1 second
    /// - 1000 operations: < 10 seconds
    /// - Rollback time: < 5 seconds
    async fn execute_batch<T>(
        &self,
        operations: impl IntoIterator<Item = T> + Send,
    ) -> Result<BatchResult, Self::Error>
    where
        T: BatchOperation + Send + Sync;

    /// Health check with connection validation
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Connection pool is accessible
    ///
    /// ### Postconditions:
    /// - Returns current health status
    /// - Updates internal health metrics
    ///
    /// ### Performance Contract:
    /// - Response time: < 50ms
    async fn health_check(&self) -> Result<HealthStatus, Self::Error>;
}

/// Parameterized query parameters with type safety
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryParams {
    params: HashMap<String, serde_json::Value>,
}

impl QueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, name: String, value: impl Into<serde_json::Value>) -> Self {
        self.params.insert(name, value.into());
        self
    }

    pub fn get(&self, name: &str) -> Option<&serde_json::Value> {
        self.params.get(name)
    }
}

/// Database record for processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseRecord {
    pub id: RecordId,
    pub content: Content,
    pub metadata: RecordMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordId(pub uuid::Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    Text(String),
    Binary(Vec<u8>),
    Structured(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    pub source: String,
    pub content_type: ContentType,
    pub size_bytes: usize,
    pub processing_state: ProcessingState,
    pub priority: Priority,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Code,
    Documentation,
    Configuration,
    Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SummaryId(pub uuid::Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Connection metadata for monitoring
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub database_id: DatabaseId,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub query_count: u64,
    pub active: bool,
}

/// Batch operation result
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub duration: std::time::Duration,
    pub errors: Vec<String>,
}

/// Health status indicators
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

/// Trait for converting database rows to domain objects
pub trait TryFromRow: Sized {
    type Error: std::error::Error + Send + Sync + 'static;

    fn try_from_row(row: &DatabaseRow) -> Result<Self, Self::Error>;
}

/// Database row representation
#[derive(Debug, Clone)]
pub struct DatabaseRow {
    pub columns: HashMap<String, DatabaseValue>,
}

#[derive(Debug, Clone)]
pub enum DatabaseValue {
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Binary(Vec<u8>),
    Null,
}

/// Trait for batch operations
pub trait BatchOperation: Send + Sync {
    fn execute(&self, connection: &mut dyn std::any::Any) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn operation_type(&self) -> OperationType;
}

#[derive(Debug, Clone)]
pub enum OperationType {
    Insert,
    Update,
    Delete,
    Create,
    Drop,
}

/// Database health status
#[derive(Debug, Clone)]
pub enum DatabaseHealth {
    Healthy,
    Degraded { reason: String, impact: DegradationImpact },
    Unhealthy { reason: String, error: DatabaseError },
}

/// Database degradation impact levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DegradationImpact {
    Low,
    Medium,
    High,
    Critical,
}