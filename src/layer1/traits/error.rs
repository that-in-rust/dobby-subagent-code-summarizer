//! Error Trait Definitions
//!
//! ## Executable Specification Contract
//!
//! ### Preconditions:
//! - Error types must implement std::error::Error
//! - Error messages must be actionable and informative
//! - Error types must support proper context chaining
//!
//! ### Postconditions:
//! - All errors can be converted to strings
//! - Error sources are preserved for debugging
//! - Error contexts are maintained through the call stack
//!
//! ### Error Conditions:
//! - Database errors → DatabaseError variants
//! - Inference errors → InferenceError variants
//! - Pipeline errors → PipelineError variants
//! - System errors → SystemError variants
//!
//! ### Performance Contracts:
//! - Error creation: < 1ms
//! - Error string conversion: < 10ms
//! - Error context addition: < 5ms

use std::fmt::Debug;
use std::error::Error;
use thiserror::Error;

/// Database-specific errors with detailed context
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {message}")]
    Connection { message: String, source: Option<Box<dyn Error + Send + Sync>> },

    #[error("Query execution failed: {query} with params {params:?}")]
    QueryExecution {
        query: String,
        params: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("Transaction failed: {message}")]
    Transaction { message: String, source: Option<Box<dyn Error + Send + Sync>> },

    #[error("Schema validation failed: {field} {expected} but got {actual}")]
    SchemaValidation {
        field: String,
        expected: String,
        actual: String,
    },

    #[error("Migration failed: {version} -> {target_version}")]
    Migration {
        version: String,
        target_version: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("Connection pool exhausted: {requested} sessions requested, {available} available")]
    ConnectionPoolExhausted { requested: usize, available: usize },

    #[error("Database unavailable: {reason}")]
    Unavailable { reason: String },

    #[error("Resource limit exceeded: {resource} (used: {used}, limit: {limit})")]
    ResourceLimitExceeded {
        resource: String,
        used: usize,
        limit: usize,
    },

    #[error("Invalid connection string: {connection_string}")]
    InvalidConnectionString { connection_string: String },

    #[error("Invalid query: {query}")]
    InvalidQuery { query: String },

    #[error("Timeout occurred: {operation} after {duration:?}")]
    Timeout {
        operation: String,
        duration: std::time::Duration,
    },
}

/// Inference-specific errors with model context
#[derive(Debug)]
#[derive(Error)]
pub enum InferenceError {
    #[error("Model loading failed: {model_path}")]
    ModelLoading {
        model_path: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("Inference execution failed: {stage}")]
    Execution {
        stage: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("Metal acceleration unavailable: {reason}")]
    MetalUnavailable { reason: String },

    #[error("Session pool exhausted: {requested} sessions requested, {available} available")]
    SessionPoolExhausted { requested: usize, available: usize },

    #[error("Quantization failed: {quantization_type}")]
    Quantization {
        quantization_type: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("Input validation failed: {field} {issue}")]
    InputValidation { field: String, issue: String },

    #[error("Model not found: {model_id}")]
    ModelNotFound { model_id: ModelId },

    #[error("Insufficient memory: {required_mb}MB required, {available_mb}MB available")]
    InsufficientMemory { required_mb: usize, available_mb: usize },

    #[error("Device unavailable: {device_type}")]
    DeviceUnavailable { device_type: String },

    #[error("Inference timeout: {operation} after {duration:?}")]
    InferenceTimeout {
        operation: String,
        duration: std::time::Duration,
    },

    #[error("Invalid input format: {input_type}")]
    InvalidInputFormat { input_type: String },

    #[error("Output generation failed: {reason}")]
    OutputGenerationFailed { reason: String },

    #[error("Batch processing failed: {failed_count}/{total_count} items failed")]
    BatchProcessingFailed {
        failed_count: usize,
        total_count: usize,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("Streaming error: {reason}")]
    StreamingError { reason: String },

    #[error("Resource temporarily unavailable: {resource}")]
    ResourceTemporarilyUnavailable { resource: String },

    #[error("Model configuration error: {parameter} = {value} is invalid")]
    ConfigurationError {
        parameter: String,
        value: String,
    },

    #[error("Performance contract violation: {contract_type} - {details}")]
    PerformanceContractViolation {
        contract_type: String,
        details: String,
    },

    #[error("Session creation failed: {reason}")]
    SessionCreationFailed { reason: String },

    #[error("Tokenization failed: {reason}")]
    TokenizationError { reason: String },

    #[error("Detokenization failed: {reason}")]
    DetokenizationError { reason: String },
}

/// Pipeline-level errors with comprehensive context
#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Database operation failed: {operation} on {table}")]
    Database {
        operation: String,
        table: String,
        source: DatabaseError,
    },

    #[error("Inference failed for model {model}: {message}")]
    Inference {
        model: String,
        message: String,
        source: InferenceError,
    },

    #[error("Pipeline {pipeline_id} failed at stage {stage}: {message}")]
    Pipeline {
        pipeline_id: PipelineId,
        stage: String,
        message: String,
    },

    #[error("Performance contract violation: {operation} took {actual:?}, expected <{expected:?}")]
    PerformanceContract {
        operation: String,
        actual: std::time::Duration,
        expected: std::time::Duration,
    },

    #[error("Resource exhaustion: {resource} (used: {used}, limit: {limit})")]
    ResourceExhaustion {
        resource: String,
        used: usize,
        limit: usize,
    },

    #[error("Configuration error in {section}: {field} = {value} is invalid")]
    Configuration {
        section: String,
        field: String,
        value: String,
    },

    #[error("Memory limit exceeded: {used_mb}MB > {limit_mb}MB")]
    MemoryLimitExceeded { used_mb: usize, limit_mb: usize },

    #[error("Timeout occurred in {operation} after {duration:?}")]
    Timeout {
        operation: String,
        duration: std::time::Duration,
    },

    #[error("Invalid state: {current_state} cannot transition to {target_state}")]
    InvalidStateTransition {
        current_state: String,
        target_state: String,
    },

    #[error("Concurrency limit exceeded: {current}/{max} concurrent operations")]
    ConcurrencyLimitExceeded { current: usize, max: usize },

    #[error("Checkpoint error: {checkpoint_id} - {reason}")]
    CheckpointError {
        checkpoint_id: String,
        reason: String,
    },

    #[error("Recovery failed: {pipeline_id} from checkpoint {checkpoint_id} - {reason}")]
    RecoveryFailed {
        pipeline_id: PipelineId,
        checkpoint_id: String,
        reason: String,
    },

    #[error("Monitoring error: {component} - {message}")]
    Monitoring {
        component: String,
        message: String,
    },
}

// Manual Clone implementation for PipelineError
impl Clone for PipelineError {
    fn clone(&self) -> Self {
        match self {
            PipelineError::Database { operation, table, source } => {
                PipelineError::Database {
                    operation: operation.clone(),
                    table: table.clone(),
                    source: source.clone(),
                }
            }
            PipelineError::Inference { model, message, source } => {
                PipelineError::Inference {
                    model: model.clone(),
                    message: message.clone(),
                    source: source.clone(),
                }
            }
            PipelineError::Pipeline { pipeline_id, stage, message } => {
                PipelineError::Pipeline {
                    pipeline_id: *pipeline_id,
                    stage: stage.clone(),
                    message: message.clone(),
                }
            }
            PipelineError::PerformanceContract { operation, actual, expected } => {
                PipelineError::PerformanceContract {
                    operation: operation.clone(),
                    actual: *actual,
                    expected: *expected,
                }
            }
            PipelineError::ResourceExhaustion { resource, used, limit } => {
                PipelineError::ResourceExhaustion {
                    resource: resource.clone(),
                    used: *used,
                    limit: *limit,
                }
            }
            PipelineError::Configuration { section, field, value } => {
                PipelineError::Configuration {
                    section: section.clone(),
                    field: field.clone(),
                    value: value.clone(),
                }
            }
            PipelineError::MemoryLimitExceeded { used_mb, limit_mb } => {
                PipelineError::MemoryLimitExceeded {
                    used_mb: *used_mb,
                    limit_mb: *limit_mb,
                }
            }
            PipelineError::Timeout { operation, duration } => {
                PipelineError::Timeout {
                    operation: operation.clone(),
                    duration: *duration,
                }
            }
            PipelineError::InvalidStateTransition { current_state, target_state } => {
                PipelineError::InvalidStateTransition {
                    current_state: current_state.clone(),
                    target_state: target_state.clone(),
                }
            }
            PipelineError::ConcurrencyLimitExceeded { current, max } => {
                PipelineError::ConcurrencyLimitExceeded {
                    current: *current,
                    max: *max,
                }
            }
            PipelineError::CheckpointError { checkpoint_id, reason } => {
                PipelineError::CheckpointError {
                    checkpoint_id: checkpoint_id.clone(),
                    reason: reason.clone(),
                }
            }
            PipelineError::RecoveryFailed { pipeline_id, checkpoint_id, reason } => {
                PipelineError::RecoveryFailed {
                    pipeline_id: *pipeline_id,
                    checkpoint_id: checkpoint_id.clone(),
                    reason: reason.clone(),
                }
            }
            PipelineError::Monitoring { component, message } => {
                PipelineError::Monitoring {
                    component: component.clone(),
                    message: message.clone(),
                }
            }
        }
    }
}

/// System-level errors for infrastructure issues
#[derive(Error, Debug)]
pub enum SystemError {
    #[error("I/O error: {operation} failed: {source}")]
    Io {
        operation: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Network error: {operation} failed: {source}")]
    Network {
        operation: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("Serialization error: {format} failed: {source}")]
    Serialization {
        format: String,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },

    #[error("Permission denied: {operation} on {resource}")]
    PermissionDenied {
        operation: String,
        resource: String,
    },

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("System error: {message}")]
    System { message: String },
}

/// Base error trait for all custom error types
pub trait DobbyError: Error + Send + Sync + 'static {
    /// Get error severity for logging and alerting
    fn severity(&self) -> ErrorSeverity;

    /// Get retry recommendation
    fn retry_recommendation(&self) -> RetryRecommendation;

    /// Get error context for debugging
    fn context(&self) -> ErrorContext;

    /// Check if error is retryable
    fn is_retryable(&self) -> bool;

    /// Get error category for classification
    fn category(&self) -> ErrorCategory;
}

/// Error severity levels for logging and alerting
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Debug,      // Low priority, informational
    Info,       // Normal operation, important information
    Warning,    // Potential issue, but not critical
    Error,      // Error that affects functionality
    Critical,   // Critical error that requires immediate attention
}

/// Retry recommendations for error recovery
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryRecommendation {
    NoRetry,    // Error cannot be recovered from
    Retry,      // Retry immediately
    Backoff,    // Retry with exponential backoff
    Escalate,   // Escalate to human intervention
}

/// Error context information for debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub component: String,
    pub operation: String,
    pub metadata: std::collections::HashMap<String, String>,
    pub trace: Vec<String>,
}

/// Error categories for classification and routing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    Database,
    Inference,
    Pipeline,
    System,
    Configuration,
    Validation,
    Performance,
    Security,
    User,
}

/// Error types for detailed classification and handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorType {
    Logic,
    Infrastructure,
    Performance,
    Configuration,
    Resource,
    Data,
}

// Implement DobbyError for all error types
impl DobbyError for DatabaseError {
    fn severity(&self) -> ErrorSeverity {
        match self {
            DatabaseError::Connection { .. } => ErrorSeverity::Error,
            DatabaseError::QueryExecution { .. } => ErrorSeverity::Error,
            DatabaseError::Transaction { .. } => ErrorSeverity::Error,
            DatabaseError::SchemaValidation { .. } => ErrorSeverity::Warning,
            DatabaseError::Migration { .. } => ErrorSeverity::Error,
            DatabaseError::ConnectionPoolExhausted { .. } => ErrorSeverity::Error,
            DatabaseError::Unavailable { .. } => ErrorSeverity::Critical,
            DatabaseError::ResourceLimitExceeded { .. } => ErrorSeverity::Error,
            DatabaseError::InvalidConnectionString { .. } => ErrorSeverity::Error,
            DatabaseError::InvalidQuery { .. } => ErrorSeverity::Error,
            DatabaseError::Timeout { .. } => ErrorSeverity::Error,
        }
    }

    fn retry_recommendation(&self) -> RetryRecommendation {
        match self {
            DatabaseError::Connection { .. } => RetryRecommendation::Backoff,
            DatabaseError::QueryExecution { .. } => RetryRecommendation::Retry,
            DatabaseError::Transaction { .. } => RetryRecommendation::Retry,
            DatabaseError::SchemaValidation { .. } => RetryRecommendation::NoRetry,
            DatabaseError::Migration { .. } => RetryRecommendation::NoRetry,
            DatabaseError::ConnectionPoolExhausted { .. } => RetryRecommendation::Backoff,
            DatabaseError::Unavailable { .. } => RetryRecommendation::Backoff,
            DatabaseError::ResourceLimitExceeded { .. } => RetryRecommendation::Backoff,
            DatabaseError::InvalidConnectionString { .. } => RetryRecommendation::NoRetry,
            DatabaseError::InvalidQuery { .. } => RetryRecommendation::NoRetry,
            DatabaseError::Timeout { .. } => RetryRecommendation::Retry,
        }
    }

    fn context(&self) -> ErrorContext {
        ErrorContext {
            timestamp: chrono::Utc::now(),
            component: "database".to_string(),
            operation: match self {
                DatabaseError::Connection { .. } => "connect".to_string(),
                DatabaseError::QueryExecution { query, .. } => format!("execute_query: {}", query),
                DatabaseError::Transaction { .. } => "transaction".to_string(),
                DatabaseError::SchemaValidation { .. } => "validate_schema".to_string(),
                DatabaseError::Migration { .. } => "migrate".to_string(),
                DatabaseError::ConnectionPoolExhausted { .. } => "acquire_connection".to_string(),
                DatabaseError::Unavailable { .. } => "check_availability".to_string(),
                DatabaseError::ResourceLimitExhausted { .. } => "check_resources".to_string(),
                DatabaseError::InvalidConnectionString { .. } => "validate_config".to_string(),
                DatabaseError::InvalidQuery { .. } => "validate_query".to_string(),
                DatabaseError::Timeout { .. } => "execute_operation".to_string(),
            },
            metadata: std::collections::HashMap::new(),
            trace: vec![],
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self.retry_recommendation(),
            RetryRecommendation::Retry | RetryRecommendation::Backoff
        )
    }

    fn category(&self) -> ErrorCategory {
        ErrorCategory::Database
    }
}

impl DobbyError for InferenceError {
    fn severity(&self) -> ErrorSeverity {
        match self {
            InferenceError::ModelLoading { .. } => ErrorSeverity::Critical,
            InferenceError::Execution { .. } => ErrorSeverity::Error,
            InferenceError::MetalUnavailable { .. } => ErrorSeverity::Warning,
            InferenceError::SessionPoolExhausted { .. } => ErrorSeverity::Error,
            InferenceError::Quantization { .. } => ErrorSeverity::Error,
            InferenceError::InputValidation { .. } => ErrorSeverity::Warning,
            InferenceError::ModelNotFound { .. } => ErrorSeverity::Critical,
            InferenceError::InsufficientMemory { .. } => ErrorSeverity::Critical,
            InferenceError::DeviceUnavailable { .. } => ErrorSeverity::Error,
            InferenceError::InferenceTimeout { .. } => ErrorSeverity::Error,
            InferenceError::InvalidInputFormat { .. } => ErrorSeverity::Warning,
            InferenceError::OutputGenerationFailed { .. } => ErrorSeverity::Error,
            InferenceError::BatchProcessingFailed { .. } => ErrorSeverity::Error,
            InferenceError::StreamingError { .. } => ErrorSeverity::Warning,
            InferenceError::ResourceTemporarilyUnavailable { .. } => ErrorSeverity::Warning,
            InferenceError::ConfigurationError { .. } => ErrorSeverity::Critical,
            InferenceError::PerformanceContractViolation { .. } => ErrorSeverity::Error,
            InferenceError::SessionCreationFailed { .. } => ErrorSeverity::Error,
            InferenceError::TokenizationError { .. } => ErrorSeverity::Error,
            InferenceError::DetokenizationError { .. } => ErrorSeverity::Error,
        }
    }

    fn retry_recommendation(&self) -> RetryRecommendation {
        match self {
            InferenceError::ModelLoading { .. } => RetryRecommendation::Backoff,
            InferenceError::Execution { .. } => RetryRecommendation::Retry,
            InferenceError::MetalUnavailable { .. } => RetryRecommendation::Retry,
            InferenceError::SessionPoolExhausted { .. } => RetryRecommendation::Backoff,
            InferenceError::Quantization { .. } => RetryRecommendation::Retry,
            InferenceError::InputValidation { .. } => RetryRecommendation::NoRetry,
            InferenceError::ModelNotFound { .. } => RetryRecommendation::NoRetry,
            InferenceError::InsufficientMemory { .. } => RetryRecommendation::Backoff,
            InferenceError::DeviceUnavailable { .. } => RetryRecommendation::NoRetry,
            InferenceError::InferenceTimeout { .. } => RetryRecommendation::Retry,
            InferenceError::InvalidInputFormat { .. } => RetryRecommendation::NoRetry,
            InferenceError::OutputGenerationFailed { .. } => RetryRecommendation::Retry,
            InferenceError::BatchProcessingFailed { .. } => RetryRecommendation::Retry,
            InferenceError::StreamingError { .. } => RetryRecommendation::Retry,
            InferenceError::ResourceTemporarilyUnavailable { .. } => RetryRecommendation::Backoff,
            InferenceError::ConfigurationError { .. } => RetryRecommendation::NoRetry,
            InferenceError::PerformanceContractViolation { .. } => RetryRecommendation::Escalate,
            InferenceError::SessionCreationFailed { .. } => RetryRecommendation::Backoff,
            InferenceError::TokenizationError { .. } => RetryRecommendation::Retry,
            InferenceError::DetokenizationError { .. } => RetryRecommendation::Retry,
        }
    }

    fn context(&self) -> ErrorContext {
        ErrorContext {
            timestamp: chrono::Utc::now(),
            component: "inference".to_string(),
            operation: match self {
                InferenceError::ModelLoading { .. } => "load_model".to_string(),
                InferenceError::Execution { stage, .. } => format!("execute_inference: {}", stage),
                InferenceError::MetalUnavailable { .. } => "initialize_metal".to_string(),
                InferenceError::SessionPoolExhausted { .. } => "acquire_session".to_string(),
                InferenceError::Quantization { .. } => "quantize_model".to_string(),
                InferenceError::InputValidation { .. } => "validate_input".to_string(),
                InferenceError::ModelNotFound { .. } => "find_model".to_string(),
                InferenceError::InsufficientMemory { .. } => "check_memory".to_string(),
                InferenceError::DeviceUnavailable { .. } => "initialize_device".to_string(),
                InferenceError::InferenceTimeout { operation, .. } => format!("run_inference: {}", operation),
                InferenceError::InvalidInputFormat { .. } => "validate_format".to_string(),
                InferenceError::OutputGenerationFailed { .. } => "generate_output".to_string(),
                InferenceError::BatchProcessingFailed { .. } => "process_batch".to_string(),
                InferenceError::StreamingError { .. } => "stream_inference".to_string(),
                InferenceError::ResourceTemporarilyUnavailable { resource, .. } => format!("acquire_resource: {}", resource),
                InferenceError::ConfigurationError { parameter, .. } => format!("configure_parameter: {}", parameter),
                InferenceError::PerformanceContractViolation { contract_type, .. } => format!("validate_contract: {}", contract_type),
                InferenceError::SessionCreationFailed { .. } => "create_session".to_string(),
                InferenceError::TokenizationError { .. } => "tokenize_text".to_string(),
                InferenceError::DetokenizationError { .. } => "detokenize_tokens".to_string(),
            },
            metadata: std::collections::HashMap::new(),
            trace: vec![],
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self.retry_recommendation(),
            RetryRecommendation::Retry | RetryRecommendation::Backoff
        )
    }

    fn category(&self) -> ErrorCategory {
        ErrorCategory::Inference
    }
}

impl DobbyError for PipelineError {
    fn severity(&self) -> ErrorSeverity {
        match self {
            PipelineError::Database { .. } => ErrorSeverity::Error,
            PipelineError::Inference { .. } => ErrorSeverity::Error,
            PipelineError::Pipeline { .. } => ErrorSeverity::Error,
            PipelineError::PerformanceContract { .. } => ErrorSeverity::Warning,
            PipelineError::ResourceExhaustion { .. } => ErrorSeverity::Error,
            PipelineError::Configuration { .. } => ErrorSeverity::Critical,
            PipelineError::MemoryLimitExceeded { .. } => ErrorSeverity::Critical,
            PipelineError::Timeout { .. } => ErrorSeverity::Error,
            PipelineError::InvalidStateTransition { .. } => ErrorSeverity::Error,
            PipelineError::ConcurrencyLimitExceeded { .. } => ErrorSeverity::Error,
            PipelineError::CheckpointError { .. } => ErrorSeverity::Error,
            PipelineError::RecoveryFailed { .. } => ErrorSeverity::Error,
            PipelineError::Monitoring { .. } => ErrorSeverity::Warning,
        }
    }

    fn retry_recommendation(&self) -> RetryRecommendation {
        match self {
            PipelineError::Database { source, .. } => source.retry_recommendation(),
            PipelineError::Inference { source, .. } => source.retry_recommendation(),
            PipelineError::Pipeline { .. } => RetryRecommendation::Retry,
            PipelineError::PerformanceContract { .. } => RetryRecommendation::NoRetry,
            PipelineError::ResourceExhaustion { .. } => RetryRecommendation::Backoff,
            PipelineError::Configuration { .. } => RetryRecommendation::NoRetry,
            PipelineError::MemoryLimitExceeded { .. } => RetryRecommendation::Backoff,
            PipelineError::Timeout { .. } => RetryRecommendation::Retry,
            PipelineError::InvalidStateTransition { .. } => RetryRecommendation::NoRetry,
            PipelineError::ConcurrencyLimitExceeded { .. } => RetryRecommendation::Backoff,
            PipelineError::CheckpointError { .. } => RetryRecommendation::NoRetry,
            PipelineError::RecoveryFailed { .. } => RetryRecommendation::Retry,
            PipelineError::Monitoring { .. } => RetryRecommendation::Retry,
        }
    }

    fn context(&self) -> ErrorContext {
        ErrorContext {
            timestamp: chrono::Utc::now(),
            component: "pipeline".to_string(),
            operation: match self {
                PipelineError::Database { operation, .. } => format!("database_{}", operation),
                PipelineError::Inference { model, .. } => format!("inference_{}", model),
                PipelineError::Pipeline { stage, .. } => format!("pipeline_{}", stage),
                PipelineError::PerformanceContract { operation, .. } => format!("validate_performance_{}", operation),
                PipelineError::ResourceExhaustion { .. } => "check_resources".to_string(),
                PipelineError::Configuration { section, .. } => format!("configure_{}", section),
                PipelineError::MemoryLimitExceeded { .. } => "check_memory".to_string(),
                PipelineError::Timeout { operation, .. } => format!("execute_{}", operation),
                PipelineError::InvalidStateTransition { current_state, target_state, .. } => {
                    format!("transition_{}_to_{}", current_state, target_state)
                }
                PipelineError::ConcurrencyLimitExceeded { .. } => "check_concurrency".to_string(),
                PipelineError::CheckpointError { checkpoint_id, .. } => "manage_checkpoint".to_string(),
                PipelineError::RecoveryFailed { .. } => "recover_pipeline".to_string(),
                PipelineError::Monitoring { component, .. } => format!("monitor_{}", component),
            },
            metadata: std::collections::HashMap::new(),
            trace: vec![],
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self.retry_recommendation(),
            RetryRecommendation::Retry | RetryRecommendation::Backoff
        )
    }

    fn category(&self) -> ErrorCategory {
        match self {
            PipelineError::Database { .. } => ErrorCategory::Database,
            PipelineError::Inference { .. } => ErrorCategory::Inference,
            PipelineError::Pipeline { .. } => ErrorCategory::Pipeline,
            PipelineError::PerformanceContract { .. } => ErrorCategory::Performance,
            PipelineError::ResourceExhaustion { .. } => ErrorCategory::System,
            PipelineError::Configuration { .. } => ErrorCategory::Configuration,
            PipelineError::MemoryLimitExceeded { .. } => ErrorCategory::System,
            PipelineError::Timeout { .. } => ErrorCategory::System,
            PipelineError::InvalidStateTransition { .. } => ErrorCategory::Pipeline,
            PipelineError::ConcurrencyLimitExceeded { .. } => ErrorCategory::Pipeline,
            PipelineError::CheckpointError { .. } => ErrorCategory::Pipeline,
            PipelineError::RecoveryFailed { .. } => ErrorCategory::Pipeline,
            PipelineError::Monitoring { .. } => ErrorCategory::System,
        }
    }
}

// Re-export ModelId from inference module
pub use super::inference::ModelId;
pub use super::pipeline::PipelineId;

/// Result type alias for convenience
pub type DobbyResult<T> = Result<T, Box<dyn DobbyError>>;

/// Error context builder for detailed debugging information
pub struct ErrorContextBuilder {
    timestamp: chrono::DateTime<chrono::Utc>,
    component: String,
    operation: String,
    metadata: std::collections::HashMap<String, String>,
    trace: Vec<String>,
}

impl ErrorContextBuilder {
    pub fn new(component: &str, operation: &str) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            component: component.to_string(),
            operation: operation.to_string(),
            metadata: std::collections::HashMap::new(),
            trace: Vec::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_trace(mut self, trace_line: String) -> Self {
        self.trace.push(trace_line);
        self
    }

    pub fn build(self) -> ErrorContext {
        ErrorContext {
            timestamp: self.timestamp,
            component: self.component,
            operation: self.operation,
            metadata: self.metadata,
            trace: self.trace,
        }
    }
}