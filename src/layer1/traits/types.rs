//! Common types used across the TDD-First architecture
//!
//! This module defines fundamental types that are shared across traits
//! and implementations, ensuring type consistency across the system.

use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database connection identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatabaseId(pub Uuid);

impl DatabaseId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DatabaseId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DatabaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Record identifier for database operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordId(pub Uuid);

impl RecordId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RecordId {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub max_batch_size: usize,
    pub timeout_ms: u64,
    pub enable_parallel_processing: bool,
    pub max_concurrent_batches: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 32,
            timeout_ms: 5000,
            enable_parallel_processing: true,
            max_concurrent_batches: 4,
        }
    }
}

/// Session configuration for inference engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_timeout_ms: u64,
    pub max_sessions_per_model: usize,
    pub enable_session_caching: bool,
    pub session_warmup: bool,
    pub memory_limit_mb: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_timeout_ms: 300_000, // 5 minutes
            max_sessions_per_model: 10,
            enable_session_caching: true,
            session_warmup: true,
            memory_limit_mb: 2048, // 2GB
        }
    }
}

/// Retry configuration for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_backoff: bool,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 10_000,
            exponential_backoff: true,
            jitter: true,
        }
    }
}

/// Configuration for database connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub pool_size: usize,
    pub timeout_ms: u64,
    pub retry_config: RetryConfig,
    pub enable_ssl: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connection_string: "sqlite://:memory:".to_string(),
            pool_size: 10,
            timeout_ms: 30_000,
            retry_config: RetryConfig::default(),
            enable_ssl: false,
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub metrics_export_interval_ms: u64,
    pub enable_health_checks: bool,
    pub health_check_interval_ms: u64,
    pub enable_tracing: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_export_interval_ms: 60_000, // 1 minute
            enable_health_checks: true,
            health_check_interval_ms: 30_000, // 30 seconds
            enable_tracing: false,
        }
    }
}

/// Error handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig {
    pub fail_fast: bool,
    pub max_error_rate_percent: f64,
    pub error_logging_enabled: bool,
    pub dead_letter_queue: bool,
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            fail_fast: false,
            max_error_rate_percent: 5.0,
            error_logging_enabled: true,
            dead_letter_queue: true,
        }
    }
}

/// Pipeline execution stage identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineStage {
    Initialization,
    DataExtraction,
    Chunking,
    Inference,
    Aggregation,
    Storage,
    Completion,
}

impl PipelineStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            PipelineStage::Initialization => "initialization",
            PipelineStage::DataExtraction => "data_extraction",
            PipelineStage::Chunking => "chunking",
            PipelineStage::Inference => "inference",
            PipelineStage::Aggregation => "aggregation",
            PipelineStage::Storage => "storage",
            PipelineStage::Completion => "completion",
        }
    }
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_mb: f64,
    pub network_io_bytes: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            disk_usage_mb: 0.0,
            network_io_bytes: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Operation result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult<T> {
    pub data: T,
    pub duration_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl<T> OperationResult<T> {
    pub fn success(data: T, duration_ms: u64) -> Self {
        Self {
            data,
            duration_ms,
            success: true,
            error_message: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn failure(error_message: String, duration_ms: u64) -> Self {
        Self {
            duration_ms,
            success: false,
            error_message: Some(error_message),
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// Performance contract specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceContract {
    pub name: String,
    pub max_latency_ms: u64,
    pub max_memory_mb: usize,
    pub min_throughput_per_second: f64,
    pub max_error_rate_percent: f64,
}

impl PerformanceContract {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            max_latency_ms: 1000,
            max_memory_mb: 1024,
            min_throughput_per_second: 10.0,
            max_error_rate_percent: 1.0,
        }
    }

    pub fn with_latency(mut self, ms: u64) -> Self {
        self.max_latency_ms = ms;
        self
    }

    pub fn with_memory(mut self, mb: usize) -> Self {
        self.max_memory_mb = mb;
        self
    }

    pub fn with_throughput(mut self, rate: f64) -> Self {
        self.min_throughput_per_second = rate;
        self
    }

    pub fn with_error_rate(mut self, rate: f64) -> Self {
        self.max_error_rate_percent = rate;
        self
    }
}