//! Pipeline Orchestrator Trait
//!
//! ## Executable Specification Contract
//!
//! ### Preconditions:
//! - Database provider is configured and connected
//! - Inference engine is loaded and ready
//! - Valid configuration provided
//!
//! ### Postconditions:
//! - Pipeline executes end-to-end
//! - Results stored in database
//! - Metrics collected and reported
//! - Resources properly cleaned up
//!
//! ### Error Conditions:
//! - Database unavailable → PipelineError::DatabaseUnavailable
//! - Inference engine failure → PipelineError::InferenceFailure
//! - Configuration invalid → PipelineError::InvalidConfiguration
//! - Resource exhaustion → PipelineError::ResourceExhaustion
//!
//! ### Performance Contracts:
//! - Pipeline startup: < 10 seconds
//! - Record processing: > 10 records/second
//! - Memory usage: < 8GB total
//! - Error rate: < 1%

use async_trait::async_trait;
use std::fmt::Debug;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::error::{PipelineError, DobbyError};

/// Pipeline identifier for tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineId(pub uuid::Uuid);

/// Pipeline execution results with comprehensive metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResults {
    pub pipeline_id: PipelineId,
    pub processed_count: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub skipped_count: usize,
    pub total_duration: std::time::Duration,
    pub records: Vec<ProcessedRecord>,
    pub errors: Vec<PipelineError>,
    pub metrics: PipelineMetrics,
}

/// Individual processed record with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedRecord {
    pub record_id: RecordId,
    pub summary_id: Option<SummaryId>,
    pub status: ProcessingStatus,
    pub processing_time: std::time::Duration,
    pub error: Option<String>,
    pub metadata: ProcessingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Success,
    Failed(String),
    Skipped(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    pub stage: String,
    pub retry_count: u32,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_mb: usize,
    pub cpu_percent: f64,
    pub active_sessions: usize,
    pub active_connections: usize,
}

/// Pipeline progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineProgress {
    pub pipeline_id: PipelineId,
    pub stage: String,
    pub progress_percentage: f64,
    pub current_record: Option<RecordId>,
    pub records_processed: usize,
    pub total_records: usize,
    pub elapsed_time: std::time::Duration,
    pub estimated_remaining: Option<std::time::Duration>,
    pub current_throughput: f64,
    pub errors: Vec<String>,
}

/// Real-time pipeline metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetrics {
    pub timestamp: DateTime<Utc>,
    pub pipeline_id: PipelineId,
    pub stage: String,
    pub records_processed: usize,
    pub records_per_second: f64,
    pub summaries_generated: usize,
    pub avg_processing_time: std::time::Duration,
    pub p50_latency: std::time::Duration,
    pub p95_latency: std::time::Duration,
    pub p99_latency: std::time::Duration,
    pub memory_usage_mb: usize,
    pub cpu_usage_percent: f64,
    pub active_connections: usize,
    pub active_inference_sessions: usize,
    pub error_count: usize,
    pub error_rate: f64,
    pub avg_confidence: f64,
    pub avg_token_count: usize,
}

/// Pipeline checkpoint for resume capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineCheckpoint {
    pub pipeline_id: PipelineId,
    pub checkpoint_id: String,
    pub timestamp: DateTime<Utc>,
    pub stage: String,
    pub progress: PipelineProgress,
    pub state_data: serde_json::Value,
    pub processed_records: Vec<RecordId>,
    pub failed_records: Vec<RecordId>,
}

/// Pipeline configuration with comprehensive settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub database_config: DatabaseConfig,
    pub inference_config: InferenceConfig,
    pub processing_config: ProcessingConfig,
    pub monitoring_config: MonitoringConfig,
    pub performance_config: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub table_name: String,
    pub batch_size: usize,
    pub query_timeout: std::time::Duration,
    pub connection_pool_size: usize,
    pub retry_config: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub model_config: ModelConfig,
    pub batch_config: BatchConfig,
    pub session_config: SessionConfig,
    pub optimization_config: OptimizationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    pub max_concurrent_records: usize,
    pub chunk_size: usize,
    pub retry_config: RetryConfig,
    pub error_handling: ErrorHandlingConfig,
    pub backpressure_config: BackpressureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub progress_reporting_interval: std::time::Duration,
    pub alerting_enabled: bool,
    pub log_level: String,
    pub telemetry_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub min_throughput_records_per_second: f64,
    pub max_latency_ms: u64,
    pub max_memory_mb: usize,
    pub max_error_rate: f64,
    pub enable_performance_monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub base_delay: std::time::Duration,
    pub max_delay: std::time::Duration,
    pub backoff_multiplier: f64,
    pub retryable_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig {
    pub max_error_rate: f64,
    pub error_retention: usize,
    pub alert_on_errors: bool,
    pub continue_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    pub max_queue_size: usize,
    pub backpressure_threshold: f64,
    pub adaptive_scaling: bool,
    pub slow_query_threshold: std::time::Duration,
}

/// High-level pipeline orchestration with comprehensive monitoring
#[async_trait]
pub trait PipelineOrchestrator: Send + Sync + 'static {
    /// Database provider type
    type Database: DatabaseProvider + Send + Sync;
    /// Inference engine type
    type Inference: InferenceEngine + Send + Sync;
    /// Pipeline configuration type
    type Config: PipelineConfig + Send + Sync;
    /// Pipeline error type
    type Error: DobbyError + Send + Sync + 'static;

    /// Execute complete pipeline with monitoring and error recovery
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - All dependencies are initialized
    /// - Configuration is validated
    /// - System resources are available
    ///
    /// ### Postconditions:
    /// - All eligible records processed
    /// - Summaries generated and stored
    /// - Metrics collected and stored
    /// - System returns to clean state
    ///
    /// ### Performance Contract:
    /// - 1000 records: < 5 minutes
    /// - 10000 records: < 45 minutes
    /// - Memory efficiency: > 90%
    /// - Success rate: > 99%
    async fn execute_pipeline(
        &self,
        config: Self::Config,
    ) -> Result<PipelineResults, Self::Error>;

    /// Stream processing with backpressure and flow control
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Database connection is stable
    /// - Inference engine is ready
    ///
    /// ### Postconditions:
    /// - Stream processes all records
    /// - Backpressure properly managed
    /// - Progress reported regularly
    ///
    /// ### Performance Contract:
    /// - Stream latency: < 1 second for first result
    /// - Throughput: > 20 records/second
    /// - Backpressure response: < 100ms
    async fn execute_stream_pipeline(
        &self,
        config: Self::Config,
    ) -> Result<impl futures::Stream<Item = Result<PipelineProgress, Self::Error>> + Send, Self::Error>;

    /// Resume pipeline from checkpoint with state recovery
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Checkpoint is valid and accessible
    /// - System state is recoverable
    ///
    /// ### Postconditions:
    /// - Processing resumes from checkpoint
    /// - No data duplication or loss
    /// - Performance maintained
    ///
    /// ### Performance Contract:
    /// - Resume time: < 30 seconds
    /// - Processing efficiency: > 95%
    async fn resume_pipeline(
        &self,
        checkpoint: PipelineCheckpoint,
    ) -> Result<PipelineResults, Self::Error>;

    /// Cancel pipeline with graceful shutdown and cleanup
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Pipeline is currently running
    ///
    /// ### Postconditions:
    /// - Pipeline stops processing new records
    /// - Current operations complete gracefully
    /// - Resources cleaned up properly
    /// - Cancellation status reported
    ///
    /// ### Performance Contract:
    /// - Cancellation response time: < 5 seconds
    /// - Resource cleanup: < 30 seconds
    /// - Data consistency: 100%
    async fn cancel_pipeline(&self, pipeline_id: PipelineId) -> Result<CancellationResult, Self::Error>;

    /// Real-time monitoring with telemetry
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Pipeline is running
    /// - Monitoring is enabled
    ///
    /// ### Postconditions:
    /// - Real-time metrics provided
    /// - Health status updated
    /// - Alerts generated for issues
    ///
    /// ### Performance Contract:
    /// - Metrics latency: < 100ms
    /// - Update frequency: > 1 Hz
    /// - Memory overhead: < 50MB
    async fn monitor_pipeline(
        &self,
        pipeline_id: PipelineId,
    ) -> Result<impl futures::Stream<Item = PipelineMetrics> + Send, Self::Error>;

    /// Create checkpoint for resume capability
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Pipeline is in a resumable state
    ///
    /// ### Postconditions:
    /// - Checkpoint created successfully
    /// - State data captured for recovery
    /// - Checkpoint ID returned for later use
    ///
    /// ### Performance Contract:
    /// - Checkpoint creation: < 5 seconds
    /// - State data size: < 10MB
    async fn create_checkpoint(
        &self,
        pipeline_id: PipelineId,
        checkpoint_id: String,
    ) -> Result<PipelineCheckpoint, Self::Error>;

    /// List available checkpoints
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Checkpoint storage is accessible
    ///
    /// ### Postconditions:
    /// - Returns list of available checkpoints
    /// - Sorted by timestamp (newest first)
    ///
    /// ### Performance Contract:
    /// - Response time: < 1 second
    async fn list_checkpoints(
        &self,
        pipeline_id: PipelineId,
    ) -> Result<Vec<PipelineCheckpoint>, Self::Error>;

    /// Delete checkpoint to free storage
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Checkpoint exists
    /// - No pipeline is using it
    ///
    /// ### Postconditions:
    /// - Checkpoint deleted successfully
    /// - Storage space recovered
    ///
    /// ### Performance Contract:
    /// - Deletion time: < 1 second
    async fn delete_checkpoint(
        &self,
        pipeline_id: PipelineId,
        checkpoint_id: String,
    ) -> Result<(), Self::Error>;

    /// Get pipeline status and health information
    ///
    /// ## Contract
    ///
    /// ### Preconditions:
    /// - Pipeline configuration is valid
    ///
    /// ### Postconditions:
    /// - Returns current status information
    /// - Health check performed
    ///
    /// ### Performance Contract:
    /// - Response time: < 500ms
    async fn get_pipeline_status(&self, pipeline_id: PipelineId) -> Result<PipelineStatus, Self::Error>;
}

/// Cancellation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationResult {
    pub pipeline_id: PipelineId,
    pub cancelled_at: DateTime<Utc>,
    pub records_processed: usize,
    pub records_in_progress: usize,
    pub cancellation_time: std::time::Duration,
    pub cleanup_success: bool,
}

/// Pipeline status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatus {
    pub pipeline_id: PipelineId,
    pub status: PipelineState,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub current_stage: String,
    pub progress: Option<PipelineProgress>,
    pub health: PipelineHealth,
    pub error: Option<PipelineError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineState {
    Initializing,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineHealth {
    Healthy,
    Degraded { issues: Vec<String>, severity: HealthSeverity },
    Unhealthy { error: PipelineError },
}

#[derive(Debug, Clone)]
pub enum HealthSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

// Re-export types from other modules for convenience
pub use super::database::DatabaseProvider;
pub use super::database::RecordId;
pub use super::inference::InferenceEngine;
pub use super::inference::ModelId;
pub use crate::layer1::traits::error::DatabaseError;
pub use crate::layer1::traits::error::InferenceError;