# Dobby Technical Architecture v2.0
## Database-to-Summary Pipeline with CozoDB + Candle RS

**Created**: 2025-10-27
**Scope**: Comprehensive technical architecture from HLL to LLD to Test Implementation
**Approach**: TDD-First with idiomatic Rust patterns and performance contracts

---

## Executive Summary

This document provides the complete technical architecture for migrating the Dobby code summarizer from a file-based ONNX Runtime system to a database-first pipeline using CozoDB + Candle RS, while preserving the existing 20x parallel processing strengths.

### Key Architectural Shifts
- **Input**: File system → CozoDB database tables
- **Processing**: ONNX Runtime → Candle RS with Metal acceleration
- **Architecture**: Multi-crate workspace → Single crate with layered modules
- **Testing**: Post-hoc → TDD-first with executable specifications

### Performance Targets
- **Throughput**: 1000+ records/minute with 20x parallelism
- **Latency**: < 50ms average inference time per record
- **Memory**: < 8GB total footprint with session pooling
- **Scalability**: Linear scaling up to 20 concurrent agents

---

## 1. High-Level Architecture (HLL)

### 1.1 System Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CozoDB Table  │───▶│  Database Layer  │───▶│  Inference Layer│───▶│  Summary Output │
│  (id, content)  │    │ (Connection Pool) │    │ (Candle RS +   │    │   + Metadata    │
│  Primary Keys   │    │ Query Processing  │    │ Metal + 20x)   │    │  Database Link │
└─────────────────┘    └──────────────────┘    └─────────────────┘    └─────────────────┘
```

### 1.2 Architectural Principles

#### Non-Negotiable TDD Principles
1. **Executable Specifications**: Every requirement has measurable test outcomes
2. **Layered Rust Architecture**: L1 (core) → L2 (standard) → L3 (external)
3. **Dependency Injection**: All components depend on traits, enabling test doubles
4. **RAII Resource Management**: Automatic cleanup with Drop implementations
5. **Performance Contracts**: Every performance claim validated by automated tests
6. **Structured Error Handling**: thiserror for library, anyhow for application
7. **Complex Domain Support**: Real-world data models with comprehensive features
8. **Concurrency Validation**: Thread safety tested with stress tests
9. **MVP-First Rigor**: Proven architectures over theoretical abstractions

### 1.3 Core Components

#### Component Hierarchy
```
Database-to-Summary Pipeline (Orchestration)
├── Database Layer (CozoDB Integration)
│   ├── Connection Pool Manager
│   ├── Query Builder & Optimizer
│   └── Transaction Management
├── Inference Layer (Candle RS + Metal)
│   ├── Model Loading & Management
│   ├── Session Pool (20x Parallel)
│   └── Metal GPU Acceleration
├── Processing Layer (Parallel Pipeline)
│   ├── Stream Processing with Backpressure
│   ├── Error Recovery & Retry Logic
│   └── Performance Monitoring
└── Storage Layer (Results & Metadata)
    ├── Summary Storage
    ├── Performance Metrics
    └── Audit Logging
```

### 1.4 Data Flow Architecture

#### Processing Pipeline
```
Input (Database Records)
    ↓
Stream Processing (Backpressure Control)
    ↓
Batch Processing (Semaphore Controlled)
    ↓
Parallel Inference (20x Candle RS Sessions)
    ↓
Result Aggregation (Memory Efficient)
    ↓
Storage & Monitoring (CozoDB + Metrics)
```

---

## 2. Low-Level Design (LLD)

### 2.1 Module Structure

#### Layered Module Organization
```
src/
├── lib.rs                              # Main library interface
├── prelude.rs                          # Common re-exports
├── layer1/                             # L1: Core abstractions (no_std compatible)
│   ├── mod.rs
│   ├── traits/                         # Core trait definitions
│   │   ├── mod.rs
│   │   ├── database.rs                 # Database provider trait
│   │   ├── inference.rs                # Inference engine trait
│   │   ├── pipeline.rs                 # Pipeline orchestration trait
│   │   └── error.rs                    # Error trait definitions
│   ├── types/                          # Newtype patterns and core types
│   │   ├── mod.rs
│   │   ├── identifiers.rs              # ID types (DatabaseId, ChunkId, etc.)
│   │   ├── records.rs                  # Database record types
│   │   └── results.rs                  # Result wrapper types
│   └── result.rs                       # Result/Option utilities
├── layer2/                             # L2: Standard library patterns
│   ├── mod.rs
│   ├── database/                       # Database abstractions
│   │   ├── mod.rs
│   │   ├── pool.rs                     # Connection pool implementation
│   │   ├── query.rs                    # Query builder
│   │   └── transaction.rs              # Transaction management
│   ├── inference/                      # Inference abstractions
│   │   ├── mod.rs
│   │   ├── session.rs                  # Session management
│   │   ├── cache.rs                    # Inference caching
│   │   └── metal.rs                    # Metal acceleration
│   ├── parallel/                       # Parallel processing patterns
│   │   ├── mod.rs
│   │   ├── semaphore.rs                # Semaphore control
│   │   ├── stream.rs                   # Stream processing
│   │   └── backpressure.rs             # Backpressure management
│   └── monitoring/                     # Performance monitoring
│       ├── mod.rs
│       ├── metrics.rs                  # Metrics collection
│       ├── health.rs                   # Health checking
│       └── telemetry.rs                # Telemetry data
├── layer3/                             # L3: External integrations
│   ├── mod.rs
│   ├── cozo_db/                        # CozoDB implementation
│   │   ├── mod.rs
│   │   ├── client.rs                   # CozoDB client
│   │   ├── schema.rs                   # Database schema
│   │   └── migrations.rs               # Migration management
│   ├── candle_rs/                      # Candle RS implementation
│   │   ├── mod.rs
│   │   ├── engine.rs                   # Inference engine
│   │   ├── models/                     # Model implementations
│   │   │   ├── mod.rs
│   │   │   ├── qwen2.rs                # Qwen2.5 model
│   │   │   └── smollm.rs               # SmolLM models
│   │   ├── session_pool.rs             # Session pool management
│   │   └── metal_acceleration.rs       # Metal optimization
│   └── config/                         # Configuration management
│       ├── mod.rs
│       ├── database.rs                 # Database configuration
│       ├── inference.rs                # Inference configuration
│       └── pipeline.rs                 # Pipeline configuration
├── components/                         # Concrete components
│   ├── mod.rs
│   ├── pipeline/                       # Pipeline orchestration
│   │   ├── mod.rs
│   │   ├── database_to_summary.rs      # Main pipeline implementation
│   │   ├── stream_processor.rs         # Stream processing
│   │   └── batch_processor.rs          # Batch processing
│   ├── metrics/                        # Performance monitoring
│   │   ├── mod.rs
│   │   ├── collector.rs                # Metrics collector
│   │   ├── reporter.rs                 # Metrics reporter
│   │   └── storage.rs                  # Metrics storage
│   └── config/                         # Configuration utilities
│       ├── mod.rs
│       ├── loader.rs                   # Configuration loader
│       ├── validator.rs                # Configuration validator
│       └── defaults.rs                 # Default configurations
└── utils/                              # Utility modules
    ├── mod.rs
    ├── memory.rs                       # Memory management utilities
    ├── text.rs                         # Text processing utilities
    └── time.rs                         # Time utilities
```

### 2.2 Core Trait System

#### Database Provider Trait
```rust
/// Core database abstraction with async support and comprehensive error handling
#[async_trait]
pub trait DatabaseProvider: Send + Sync + 'static {
    /// Connection type with lifetime management
    type Connection: DatabaseConnection + Send + Sync;
    /// Error type with detailed context
    type Error: DatabaseError + Send + Sync + 'static;

    /// Establish connection with automatic retry and circuit breaking
    async fn connect(&self) -> Result<Self::Connection, Self::Error>;

    /// Execute query with parameter binding and result mapping
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
    async fn fetch_records_stream(
        &self,
        query: &str,
        params: QueryParams,
    ) -> Result<impl Stream<Item = Result<DatabaseRecord, Self::Error>> + Send, Self::Error>;

    /// Batch operation with transaction support
    async fn execute_batch<T>(
        &self,
        operations: impl IntoIterator<Item = T> + Send,
    ) -> Result<BatchResult, Self::Error>
    where
        T: BatchOperation + Send + Sync;

    /// Health check with connection validation
    async fn health_check(&self) -> Result<HealthStatus, Self::Error>;
}
```

#### Inference Engine Trait
```rust
/// Core inference abstraction with session pooling and Metal acceleration
#[async_trait]
pub trait InferenceEngine: Send + Sync + 'static {
    /// Input type for inference
    type Input: Send + Sync + 'static;
    /// Output type from inference
    type Output: Send + Sync + 'static;
    /// Error type with detailed inference context
    type Error: InferenceError + Send + Sync + 'static;

    /// Model information with capabilities
    type ModelInfo: ModelInfo + Send + Sync;

    /// Load model with device selection and quantization options
    async fn load_model(
        &self,
        config: ModelConfig,
    ) -> Result<Self::ModelInfo, Self::Error>;

    /// Single inference with automatic batching optimization
    async fn infer(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;

    /// Batch inference with parallel processing
    async fn infer_batch(
        &self,
        inputs: Vec<Self::Input>,
        options: BatchOptions,
    ) -> Result<Vec<Self::Output>, Self::Error>;

    /// Streaming inference for large inputs
    async fn infer_stream(
        &self,
        input_stream: impl Stream<Item = Self::Input> + Send,
    ) -> Result<impl Stream<Item = Result<Self::Output, Self::Error>> + Send, Self::Error>;

    /// Model capabilities and performance characteristics
    fn model_info(&self) -> &Self::ModelInfo;

    /// Performance benchmarking with contract validation
    async fn benchmark(&self, test_cases: &[BenchmarkCase]) -> Result<BenchmarkResults, Self::Error>;
}
```

#### Pipeline Orchestration Trait
```rust
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
    type Error: PipelineError + Send + Sync + 'static;

    /// Execute complete pipeline with monitoring and error recovery
    async fn execute_pipeline(
        &self,
        config: Self::Config,
    ) -> Result<PipelineResults, Self::Error>;

    /// Stream processing with backpressure and flow control
    async fn execute_stream_pipeline(
        &self,
        config: Self::Config,
    ) -> Result<impl Stream<Item = Result<PipelineProgress, Self::Error>> + Send, Self::Error>;

    /// Resume pipeline from checkpoint with state recovery
    async fn resume_pipeline(
        &self,
        checkpoint: PipelineCheckpoint,
    ) -> Result<PipelineResults, Self::Error>;

    /// Cancel pipeline with graceful shutdown and cleanup
    async fn cancel_pipeline(&self, pipeline_id: PipelineId) -> Result<(), Self::Error>;

    /// Real-time monitoring with telemetry
    async fn monitor_pipeline(
        &self,
        pipeline_id: PipelineId,
    ) -> Result<impl Stream<Item = PipelineMetrics> + Send, Self::Error>;
}
```

### 2.3 Type System Design

#### Core Type Definitions
```rust
/// Strongly-typed identifiers for type safety and compile-time guarantees
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatabaseId(pub uuid::Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordId(pub uuid::Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SummaryId(pub uuid::Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineId(pub uuid::Uuid);

/// Database record with metadata for tracking and processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseRecord {
    pub id: RecordId,
    pub content: Content,
    pub metadata: RecordMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Content with zero-copy processing support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    Text(String),
    Binary(Vec<u8>),
    Structured(serde_json::Value),
}

/// Record metadata with processing state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    pub source: String,
    pub content_type: ContentType,
    pub size_bytes: usize,
    pub processing_state: ProcessingState,
    pub priority: Priority,
    pub custom_fields: std::collections::HashMap<String, serde_json::Value>,
}

/// Processing state with comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingState {
    Pending,
    InProgress { started_at: chrono::DateTime<chrono::Utc> },
    Completed {
        completed_at: chrono::DateTime<chrono::Utc>,
        summary_id: SummaryId,
    },
    Failed {
        failed_at: chrono::DateTime<chrono::Utc>,
        error: String,
        retry_count: u32,
    },
    Skipped {
        skipped_at: chrono::DateTime<chrono::Utc>,
        reason: String,
    },
}
```

### 2.4 Error Handling System

#### Comprehensive Error Taxonomy
```rust
/// Library-level errors with structured context using thiserror
#[derive(Error, Debug)]
pub enum DobbyError {
    #[error("Database operation failed: {operation} on {table}")]
    Database {
        operation: String,
        table: String,
        #[source]
        source: DatabaseError,
    },

    #[error("Inference failed for model {model}: {message}")]
    Inference {
        model: String,
        message: String,
        #[source]
        source: InferenceError,
    },

    #[error("Pipeline {pipeline_id} failed at stage {stage}: {message}")]
    Pipeline {
        pipeline_id: PipelineId,
        stage: String,
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Performance contract violation: {operation} took {actual:?}, expected <{expected:?}")]
    PerformanceContract {
        operation: String,
        actual: std::time::Duration,
        expected: std::time::Duration
    },

    #[error("Resource exhaustion: {resource} (used: {used}, limit: {limit})")]
    ResourceExhaustion {
        resource: String,
        used: usize,
        limit: usize
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
}

/// Database-specific errors with detailed context
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {message}")]
    Connection { message: String },

    #[error("Query execution failed: {query} with params {params:?}")]
    QueryExecution {
        query: String,
        params: QueryParams,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Transaction failed: {message}")]
    Transaction { message: String },

    #[error("Schema validation failed: {field} {expected} but got {actual}")]
    SchemaValidation {
        field: String,
        expected: String,
        actual: String
    },

    #[error("Migration failed: {version} -> {target_version}")]
    Migration {
        version: String,
        target_version: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Inference-specific errors with model context
#[derive(Error, Debug)]
pub enum InferenceError {
    #[error("Model loading failed: {model_path}")]
    ModelLoading {
        model_path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Inference execution failed: {stage}")]
    Execution {
        stage: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Metal acceleration unavailable: {reason}")]
    MetalUnavailable { reason: String },

    #[error("Session pool exhausted: {requested} sessions requested, {available} available")]
    SessionPoolExhausted { requested: usize, available: usize },

    #[error("Quantization failed: {quantization_type}")]
    Quantization {
        quantization_type: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Input validation failed: {field} {issue}")]
    InputValidation { field: String, issue: String },
}
```

### 2.5 Configuration System

#### Hierarchical Configuration
```rust
/// Main pipeline configuration with comprehensive settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub database: DatabaseConfig,
    pub inference: InferenceConfig,
    pub parallel: ParallelConfig,
    pub monitoring: MonitoringConfig,
    pub performance: PerformanceConfig,
}

/// Database configuration with connection pooling and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub pool_size: usize,
    pub timeout: std::time::Duration,
    pub retry_attempts: usize,
    pub backoff_multiplier: f64,
    pub query_timeout: std::time::Duration,
    pub transaction_timeout: std::time::Duration,
    pub connection_validation_interval: std::time::Duration,
}

/// Inference configuration with model selection and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub model_path: String,
    pub model_name: String,
    pub device: DeviceConfig,
    pub quantization: QuantizationConfig,
    pub session_pool: SessionPoolConfig,
    pub batching: BatchingConfig,
    pub prompt_template: String,
    pub generation_params: GenerationParams,
}

/// Device configuration with automatic detection and fallback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub preferred_device: DeviceType,
    pub fallback_enabled: bool,
    pub memory_fraction: f64,
    pub enable_metal: bool,
    pub force_cpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Auto,
    Cpu,
    Metal { device_id: usize },
    Cuda { device_id: usize },
}

/// Parallel processing configuration with adaptive concurrency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    pub max_concurrent_tasks: usize,
    pub semaphore_permits: usize,
    pub batch_size: usize,
    pub adaptive_concurrency: bool,
    pub backpressure_threshold: usize,
    pub queue_depth: usize,
    pub worker_threads: Option<usize>,
}

/// Performance configuration with contracts and limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub throughput_targets: ThroughputTargets,
    pub latency_limits: LatencyLimits,
    pub memory_limits: MemoryLimits,
    pub cpu_limits: CpuLimits,
    pub contracts_enabled: bool,
    pub monitoring_interval: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputTargets {
    pub min_records_per_second: f64,
    pub min_tokens_per_second: f64,
    pub target_parallel_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyLimits {
    pub max_p50: std::time::Duration,
    pub max_p95: std::time::Duration,
    pub max_p99: std::time::Duration,
    pub max_timeout: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
    pub max_per_record_mb: usize,
    pub max_total_mb: usize,
    pub max_session_pool_mb: usize,
    pub gc_threshold_mb: usize,
}
```

---

## 3. Implementation Architecture

### 3.1 Database Layer Implementation

#### CozoDB Connection Pool
```rust
/// High-performance connection pool with circuit breaking and metrics
pub struct CozoConnectionPool {
    connections: Arc<Vec<PooledConnection>>,
    semaphore: Arc<Semaphore>,
    circuit_breaker: Arc<CircuitBreaker>,
    metrics: Arc<DatabaseMetrics>,
    config: DatabaseConfig,
}

impl CozoConnectionPool {
    /// Create new connection pool with health checking
    pub async fn new(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        let mut connections = Vec::with_capacity(config.pool_size);

        for i in 0..config.pool_size {
            let conn = CozoConnection::create(&config.connection_string, i).await?;
            let pooled = PooledConnection::new(conn, i);
            connections.push(pooled);
        }

        Ok(Self {
            connections: Arc::new(connections),
            semaphore: Arc::new(Semaphore::new(config.pool_size)),
            circuit_breaker: Arc::new(CircuitBreaker::new(
                config.retry_attempts,
                std::time::Duration::from_secs(30),
            )),
            metrics: Arc::new(DatabaseMetrics::new()),
            config,
        })
    }

    /// Get connection with circuit breaking and retry logic
    pub async fn get_connection(&self) -> Result<PooledConnection, DatabaseError> {
        let _permit = self.semaphore.acquire().await
            .map_err(|_| DatabaseError::SessionPoolExhausted {
                requested: 1,
                available: 0
            })?;

        self.circuit_breaker.execute(|| async {
            let index = fastrand::usize(0..self.connections.len());
            let conn = &self.connections[index];

            // Validate connection health
            if !conn.is_healthy().await {
                conn.reconnect().await?;
            }

            self.metrics.record_connection_acquired();
            Ok(conn.clone())
        }).await
    }

    /// Execute query with automatic retry and metrics
    pub async fn execute_query<Q, P, R>(
        &self,
        query: Q,
        params: P,
    ) -> Result<Vec<R>, DatabaseError>
    where
        Q: AsRef<str> + Send + Sync,
        P: Into<QueryParams> + Send,
        R: TryFromRow + Send,
    {
        let conn = self.get_connection().await?;
        let start_time = std::time::Instant::now();

        let result = conn.execute_query(query, params).await;
        let duration = start_time.elapsed();

        match &result {
            Ok(rows) => {
                self.metrics.record_successful_query(duration, rows.len());
            }
            Err(e) => {
                self.metrics.record_failed_query(duration, e);
            }
        }

        result
    }
}

/// Individual pooled connection with health monitoring
#[derive(Debug, Clone)]
pub struct PooledConnection {
    connection: Arc<Mutex<CozoConnection>>,
    id: usize,
    last_used: Arc<Mutex<std::time::Instant>>,
    health_status: Arc<Mutex<HealthStatus>>,
}

impl PooledConnection {
    pub async fn is_healthy(&self) -> bool {
        let health = self.health_status.lock().await;
        matches!(*health, HealthStatus::Healthy)
    }

    pub async fn reconnect(&self) -> Result<(), DatabaseError> {
        let mut conn = self.connection.lock().await;
        let mut health = self.health_status.lock().await;

        *conn = CozoConnection::create(&conn.connection_string(), self.id).await?;
        *health = HealthStatus::Healthy;

        Ok(())
    }
}
```

#### Query Builder with Parameterized Queries
```rust
/// Type-safe query builder with parameter binding and optimization
pub struct QueryBuilder {
    query_type: QueryType,
    table: String,
    conditions: Vec<QueryCondition>,
    order_by: Vec<OrderBy>,
    limit: Option<usize>,
    offset: Option<usize>,
    params: QueryParams,
}

#[derive(Debug, Clone)]
pub enum QueryType {
    Select { columns: Vec<String> },
    Insert { values: Vec<serde_json::Value> },
    Update { set_clauses: Vec<String> },
    Delete,
}

impl QueryBuilder {
    pub fn select(table: impl Into<String>, columns: Vec<impl Into<String>>) -> Self {
        Self {
            query_type: QueryType::Select {
                columns: columns.into_iter().map(Into::into).collect()
            },
            table: table.into(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            params: QueryParams::new(),
        }
    }

    pub fn where_eq(mut self, column: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        let param_name = format!("param_{}", self.params.len());
        self.conditions.push(QueryCondition::Eq(column.into(), param_name.clone()));
        self.params.insert(param_name, value.into());
        self
    }

    pub fn where_in(mut self, column: impl Into<String>, values: Vec<impl Into<serde_json::Value>>) -> Self {
        let param_names: Vec<String> = values.iter().enumerate()
            .map(|(i, value)| {
                let param_name = format!("param_{}", self.params.len());
                self.params.insert(param_name.clone(), value.into());
                param_name
            })
            .collect();

        self.conditions.push(QueryCondition::In(column.into(), param_names));
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn build(self) -> (String, QueryParams) {
        let query = match self.query_type {
            QueryType::Select { columns } => {
                let columns_str = if columns.is_empty() { "*" } else { &columns.join(", ") };
                format!(
                    "SELECT {} FROM {}{}{}{}",
                    columns_str,
                    self.table,
                    self.build_where_clause(),
                    self.build_order_clause(),
                    self.build_limit_clause(),
                )
            }
            // Other query types...
        };

        (query, self.params)
    }
}

/// Parameterized query parameters with type safety
#[derive(Debug, Clone, Default)]
pub struct QueryParams {
    params: std::collections::HashMap<String, serde_json::Value>,
}

impl QueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: String, value: serde_json::Value) {
        self.params.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&serde_json::Value> {
        self.params.get(name)
    }
}
```

### 3.2 Inference Layer Implementation

#### Candle RS Inference Engine with Metal Support
```rust
/// High-performance Candle RS inference engine with Metal acceleration
pub struct CandleInferenceEngine {
    model: Arc<candle_transformers::models::qwen2::Qwen2>,
    device: Device,
    tokenizer: Arc<hf::Tokenizer>,
    session_pool: Arc<SessionPool>,
    config: InferenceConfig,
    metrics: Arc<InferenceMetrics>,
}

impl CandleInferenceEngine {
    /// Create new inference engine with device selection and model loading
    pub async fn new(config: InferenceConfig) -> Result<Self, InferenceError> {
        let device = self.select_device(&config.device)?;
        let model = self.load_model(&config.model_path, &device).await?;
        let tokenizer = self.load_tokenizer(&config).await?;
        let session_pool = SessionPool::new(config.session_pool.clone());

        Ok(Self {
            model: Arc::new(model),
            device,
            tokenizer: Arc::new(tokenizer),
            session_pool: Arc::new(session_pool),
            config,
            metrics: Arc::new(InferenceMetrics::new()),
        })
    }

    /// Select optimal device with Metal preference and fallback
    fn select_device(&self, config: &DeviceConfig) -> Result<Device, InferenceError> {
        match config.preferred_device {
            DeviceType::Auto => {
                if cfg!(target_os = "macos") && config.enable_metal {
                    Device::new_metal(0)
                        .or_else(|_| Device::new_cuda(0))
                        .or_else(|_| Device::Cpu)
                } else {
                    Ok(Device::Cpu)
                }
            }
            DeviceType::Metal { device_id } => {
                Device::new_metal(device_id)
            }
            DeviceType::Cuda { device_id } => {
                Device::new_cuda(device_id)
            }
            DeviceType::Cpu => Ok(Device::Cpu),
        }.map_err(|e| InferenceError::MetalUnavailable {
            reason: format!("Device selection failed: {}", e)
        })
    }

    /// Load model with quantization and optimization
    async fn load_model(
        &self,
        model_path: &str,
        device: &Device,
    ) -> Result<candle_transformers::models::qwen2::Qwen2, InferenceError> {
        let start_time = std::time::Instant::now();

        let model = candle_transformers::models::qwen2::Qwen2::load(
            model_path,
            device,
        ).map_err(|e| InferenceError::ModelLoading {
            model_path: model_path.to_string(),
            source: Box::new(e),
        })?;

        let load_time = start_time.elapsed();
        self.metrics.record_model_load(load_time);

        Ok(model)
    }

    /// Load tokenizer with vocabulary validation
    async fn load_tokenizer(&self, config: &InferenceConfig) -> Result<hf::Tokenizer, InferenceError> {
        let tokenizer_path = Path::new(&config.model_path).join("tokenizer.json");

        hf::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| InferenceError::ModelLoading {
                model_path: format!("{} tokenizer", config.model_path),
                source: Box::new(e),
            })
    }
}

#[async_trait]
impl InferenceEngine for CandleInferenceEngine {
    type Input = String;
    type Output = InferenceResult;
    type Error = InferenceError;
    type ModelInfo = ModelInfo;

    async fn infer(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let session = self.session_pool.acquire().await?;
        let start_time = std::time::Instant::now();

        // Tokenize input
        let tokens = self.tokenizer.encode(input, false)
            .map_err(|e| InferenceError::InputValidation {
                field: "input",
                issue: format!("Tokenization failed: {}", e),
            })?;

        // Convert to tensor
        let input_ids = Tensor::new(
            tokens.get_ids().to_vec(),
            &self.device,
        )?.unsqueeze(0)?;

        // Generate with Metal acceleration
        let output = session.generate(&self.model, &input_ids, &self.config.generation_params)?;

        // Decode output
        let generated_text = self.tokenizer.decode(&output.token_ids, true)
            .map_err(|e| InferenceError::Execution {
                stage: "decoding",
                source: Box::new(e),
            })?;

        let duration = start_time.elapsed();
        self.metrics.record_inference(duration, input_ids.dim(0)?);

        Ok(InferenceResult {
            text: generated_text,
            token_count: output.token_ids.len(),
            confidence: output.confidence,
            metadata: output.metadata,
        })
    }

    async fn infer_batch(
        &self,
        inputs: Vec<Self::Input>,
        options: BatchOptions,
    ) -> Result<Vec<Self::Output>, Self::Error> {
        let batch_size = inputs.len().min(options.max_batch_size);
        let sessions = self.session_pool.acquire_batch(batch_size).await?;

        let start_time = std::time::Instant::now();
        let mut results = Vec::with_capacity(inputs.len());

        // Process inputs in parallel across sessions
        let futures = inputs.into_iter().zip(sessions.into_iter())
            .map(|(input, session)| async move {
                self.process_single_input(session, input).await
            });

        let batch_results = futures::future::join_all(futures).await;

        for result in batch_results {
            match result {
                Ok(output) => results.push(output),
                Err(e) => {
                    if options.fail_fast {
                        return Err(e);
                    } else {
                        // Log error and continue with partial results
                        self.metrics.record_failed_inference(&e);
                    }
                }
            }
        }

        let duration = start_time.elapsed();
        self.metrics.record_batch_inference(duration, results.len());

        Ok(results)
    }
}
```

#### Session Pool with Resource Management
```rust
/// High-performance session pool with RAII management and metrics
pub struct SessionPool {
    sessions: Arc<Vec<InferenceSession>>,
    semaphore: Arc<Semaphore>,
    metrics: Arc<SessionPoolMetrics>,
    config: SessionPoolConfig,
}

impl SessionPool {
    pub fn new(config: SessionPoolConfig) -> Self {
        let sessions = (0..config.pool_size)
            .map(|id| InferenceSession::new(id, config.clone()))
            .collect();

        Self {
            sessions: Arc::new(sessions),
            semaphore: Arc::new(Semaphore::new(config.pool_size)),
            metrics: Arc::new(SessionPoolMetrics::new()),
            config,
        }
    }

    /// Acquire session with automatic retry and load balancing
    pub async fn acquire(&self) -> Result<InferenceSession, InferenceError> {
        let _permit = self.semaphore.acquire().await
            .map_err(|_| InferenceError::SessionPoolExhausted {
                requested: 1,
                available: 0,
            })?;

        // Load balancing: select session with least usage
        let session_index = self.select_session_by_load().await;
        let session = self.sessions[session_index].clone();

        session.acquire().await?;
        self.metrics.record_session_acquired(session_index);

        Ok(session)
    }

    /// Acquire multiple sessions for batch processing
    pub async fn acquire_batch(&self, count: usize) -> Result<Vec<InferenceSession>, InferenceError> {
        let permits = self.semaphore.acquire_many(count).await
            .map_err(|_| InferenceError::SessionPoolExhausted {
                requested: count,
                available: 0,
            })?;

        let mut sessions = Vec::with_capacity(count);
        for permit in permits {
            let session_index = self.select_session_by_load().await;
            let session = self.sessions[session_index].clone();
            session.acquire().await?;
            sessions.push(session);
        }

        self.metrics.record_batch_acquired(sessions.len());
        Ok(sessions)
    }

    /// Select session with lowest current load
    async fn select_session_by_load(&self) -> usize {
        let mut min_load = usize::MAX;
        let mut selected_index = 0;

        for (index, session) in self.sessions.iter().enumerate() {
            let load = session.current_load().await;
            if load < min_load {
                min_load = load;
                selected_index = index;
            }
        }

        selected_index
    }
}

/// Individual inference session with state management
pub struct InferenceSession {
    id: usize,
    state: Arc<Mutex<SessionState>>,
    metrics: Arc<SessionMetrics>,
    device: Device,
    config: SessionPoolConfig,
}

#[derive(Debug)]
struct SessionState {
    status: SessionStatus,
    in_use_count: usize,
    last_used: std::time::Instant,
    memory_usage: usize,
    error_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SessionStatus {
    Idle,
    Busy,
    WarmingUp,
    Error,
}

impl InferenceSession {
    async fn acquire(&self) -> Result<(), InferenceError> {
        let mut state = self.state.lock().await;

        match state.status {
            SessionStatus::Idle => {
                state.status = SessionStatus::Busy;
                state.in_use_count += 1;
                state.last_used = std::time::Instant::now();
                Ok(())
            }
            SessionStatus::Error => {
                Err(InferenceError::SessionError {
                    session_id: self.id,
                    reason: "Session in error state".to_string(),
                })
            }
            _ => {
                Err(InferenceError::SessionError {
                    session_id: self.id,
                    reason: "Session not available".to_string(),
                })
            }
        }
    }

    async fn release(&self) {
        let mut state = self.state.lock().await;
        state.status = SessionStatus::Idle;
        state.in_use_count = state.in_use_count.saturating_sub(1);
    }

    pub async fn current_load(&self) -> usize {
        let state = self.state.lock().await;
        state.in_use_count
    }
}

impl Drop for InferenceSession {
    fn drop(&mut self) {
        // Cleanup resources when session is dropped
        self.metrics.record_session_dropped(self.id);
    }
}
```

### 3.3 Parallel Processing Implementation

#### Stream Processing with Backpressure
```rust
/// Stream processing pipeline with adaptive backpressure and flow control
pub struct StreamProcessor {
    db_pool: Arc<dyn DatabaseProvider>,
    inference_engine: Arc<dyn InferenceEngine>,
    config: ParallelConfig,
    metrics: Arc<StreamMetrics>,
    backpressure_controller: Arc<BackpressureController>,
}

impl StreamProcessor {
    /// Process database records as stream with adaptive concurrency
    pub async fn process_stream(
        &self,
        query: &str,
        params: QueryParams,
    ) -> Result<impl Stream<Item = Result<ProcessedRecord, PipelineError>>, PipelineError> {
        let (tx, rx) = tokio::sync::mpsc::channel(self.config.queue_depth);
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_tasks));

        // Database reader task
        let db_task = {
            let db_pool = self.db_pool.clone();
            let query = query.to_string();
            let params = params.clone();
            let tx = tx.clone();
            let backpressure = self.backpressure_controller.clone();

            tokio::spawn(async move {
                let mut stream = db_pool.fetch_records_stream(&query, params).await?;

                while let Some(record_result) = stream.next().await {
                    // Check backpressure before processing
                    backpressure.wait_if_needed().await;

                    match record_result {
                        Ok(record) => {
                            if tx.send(Ok(record)).await.is_err() {
                                break; // Downstream closed
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(PipelineError::from(e))).await;
                        }
                    }
                }

                Ok::<(), PipelineError>(())
            })
        };

        // Processing task with semaphore control
        let process_task = {
            let inference_engine = self.inference_engine.clone();
            let semaphore = semaphore.clone();
            let output_tx = tx.clone();

            tokio::spawn(async move {
                let mut rx = tx;

                while let Some(record_result) = rx.recv().await {
                    let permit = semaphore.acquire().await?;

                    let engine = inference_engine.clone();
                    let output_tx = output_tx.clone();

                    tokio::spawn(async move {
                        let _permit = permit; // Hold permit for duration of processing

                        match record_result {
                            Ok(record) => {
                                match engine.infer(record.content).await {
                                    Ok(summary) => {
                                        let processed = ProcessedRecord {
                                            id: record.id,
                                            summary,
                                            processed_at: chrono::Utc::now(),
                                        };
                                        let _ = output_tx.send(Ok(processed)).await;
                                    }
                                    Err(e) => {
                                        let _ = output_tx.send(Err(PipelineError::from(e))).await;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = output_tx.send(Err(e)).await;
                            }
                        }
                    });
                }

                Ok::<(), PipelineError>(())
            })
        };

        // Return the output stream
        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }
}

/// Adaptive backpressure controller
pub struct BackpressureController {
    target_latency: std::time::Duration,
    current_latency: Arc<Mutex<std::time::Duration>>,
    adjustment_factor: f64,
    min_concurrency: usize,
    max_concurrency: usize,
}

impl BackpressureController {
    pub async fn wait_if_needed(&self) {
        let current_latency = *self.current_latency.lock().await;

        if current_latency > self.target_latency {
            // Calculate backoff duration based on latency ratio
            let latency_ratio = current_latency.as_secs_f64() / self.target_latency.as_secs_f64();
            let backoff_duration = std::time::Duration::from_millis(
                (100.0 * latency_ratio * self.adjustment_factor) as u64
            );

            tokio::time::sleep(backoff_duration).await;
        }
    }

    pub async fn update_latency(&self, latency: std::time::Duration) {
        let mut current = self.current_latency.lock().await;
        // Exponential moving average
        *current = std::time::Duration::from_millis(
            ((current.as_millis() as f64 * 0.9) + (latency.as_millis() as f64 * 0.1)) as u64
        );
    }
}
```

#### Adaptive Concurrency Management
```rust
/// Adaptive concurrency manager with performance-based optimization
pub struct AdaptiveConcurrencyManager {
    current_concurrency: Arc<AtomicUsize>,
    min_concurrency: usize,
    max_concurrency: usize,
    target_throughput: f64,
    performance_window: Arc<Mutex<VecDeque<PerformanceMetric>>>,
    adjustment_interval: std::time::Duration,
}

impl AdaptiveConcurrencyManager {
    pub fn new(config: ParallelConfig) -> Self {
        Self {
            current_concurrency: Arc::new(AtomicUsize::new(config.max_concurrent_tasks)),
            min_concurrency: config.max_concurrent_tasks / 2,
            max_concurrency: config.max_concurrent_tasks,
            target_throughput: config.target_throughput,
            performance_window: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            adjustment_interval: config.adjustment_interval,
        }
    }

    /// Start adaptive adjustment task
    pub async fn start_adjustment_task(&self) -> tokio::task::JoinHandle<()> {
        let current_concurrency = self.current_concurrency.clone();
        let performance_window = self.performance_window.clone();
        let min_concurrency = self.min_concurrency;
        let max_concurrency = self.max_concurrency;
        let target_throughput = self.target_throughput;
        let adjustment_interval = self.adjustment_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(adjustment_interval);

            loop {
                interval.tick().await;

                let current = current_concurrency.load(Ordering::Relaxed);
                let new_concurrency = Self::calculate_optimal_concurrency(
                    current,
                    min_concurrency,
                    max_concurrency,
                    target_throughput,
                    &performance_window,
                ).await;

                if new_concurrency != current {
                    current_concurrency.store(new_concurrency, Ordering::Relaxed);
                    tracing::info!("Adjusted concurrency: {} -> {}", current, new_concurrency);
                }
            }
        })
    }

    /// Calculate optimal concurrency based on performance metrics
    async fn calculate_optimal_concurrency(
        current: usize,
        min_concurrency: usize,
        max_concurrency: usize,
        target_throughput: f64,
        performance_window: &Arc<Mutex<VecDeque<PerformanceMetric>>>,
    ) -> usize {
        let window = performance_window.lock().await;

        if window.len() < 10 {
            return current; // Not enough data
        }

        let avg_throughput = window.iter()
            .map(|m| m.throughput)
            .sum::<f64>() / window.len() as f64;

        let avg_latency = window.iter()
            .map(|m| m.latency.as_millis() as f64)
            .sum::<f64>() / window.len() as f64;

        // Simple heuristic: increase if below target, decrease if latency too high
        if avg_throughput < target_throughput && avg_latency < 1000.0 {
            // Can increase concurrency
            (current + 1).min(max_concurrency)
        } else if avg_latency > 2000.0 {
            // Too much latency, decrease concurrency
            (current - 1).max(min_concurrency)
        } else {
            current // Keep current concurrency
        }
    }

    /// Record performance metric for adaptive adjustment
    pub async fn record_metric(&self, metric: PerformanceMetric) {
        let mut window = self.performance_window.lock().await;

        window.push_back(metric);

        // Keep only recent metrics
        while window.len() > 100 {
            window.pop_front();
        }
    }

    /// Get current concurrency level
    pub fn current_concurrency(&self) -> usize {
        self.current_concurrency.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    pub throughput: f64,
    pub latency: std::time::Duration,
    pub error_rate: f64,
    pub timestamp: std::time::Instant,
}
```

---

## 4. Testing Architecture

### 4.1 Test Organization and Strategy

#### Test Structure
```
tests/
├── unit/                              # Unit tests for individual components
│   ├── layer1/                       # L1 trait tests
│   │   ├── traits_tests.rs
│   │   ├── types_tests.rs
│   │   └── error_tests.rs
│   ├── layer2/                       # L2 implementation tests
│   │   ├── database_tests.rs
│   │   ├── inference_tests.rs
│   │   └── parallel_tests.rs
│   └── layer3/                       # L3 integration tests
│       ├── cozo_db_tests.rs
│       ├── candle_rs_tests.rs
│       └── config_tests.rs
├── integration/                       # Integration tests
│   ├── pipeline_tests.rs             # End-to-end pipeline tests
│   ├── performance_tests.rs          # Performance contract tests
│   └── chaos_tests.rs               # Failure scenario tests
├── property/                         # Property-based tests
│   ├── database_properties.rs        # Database invariants
│   ├── inference_properties.rs       # Inference invariants
│   └── pipeline_properties.rs        # Pipeline invariants
├── benchmarks/                       # Performance benchmarks
│   ├── database_bench.rs            # Database performance
│   ├── inference_bench.rs           # Inference performance
│   └── pipeline_bench.rs            # End-to-end performance
└── fixtures/                         # Test data and mock factories
    ├── mock_data.rs                  # Mock data generators
    ├── test_databases.rs             # Test database schemas
    └── test_models.rs                # Test model configurations
```

### 4.2 Mock Data Strategy

#### Comprehensive Mock Data Generation
```rust
/// Mock data generator with realistic code repository simulation
pub struct MockDataGenerator {
    rng: StdRng,
    config: MockConfig,
}

impl MockDataGenerator {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
            config: MockConfig::default(),
        }
    }

    /// Generate realistic code repository with multiple languages
    pub fn generate_repository(&mut self, repo_config: &RepositoryConfig) -> MockRepository {
        let mut files = Vec::new();
        let mut total_lines = 0;

        for file_config in &repo_config.files {
            let file_content = match file_config.language {
                Language::Rust => self.generate_rust_file(file_config),
                Language::TypeScript => self.generate_typescript_file(file_config),
                Language::Python => self.generate_python_file(file_config),
                Language::JavaScript => self.generate_javascript_file(file_config),
                Language::Go => self.generate_go_file(file_config),
                Language::Java => self.generate_java_file(file_config),
                Language::Cpp => self.generate_cpp_file(file_config),
                Language::Markdown => self.generate_markdown_file(file_config),
                Language::Yaml => self.generate_yaml_file(file_config),
                Language::Json => self.generate_json_file(file_config),
            };

            let line_count = file_content.lines().count();
            total_lines += line_count;

            files.push(MockFile {
                path: file_config.path.clone(),
                content: file_content,
                language: file_config.language,
                line_count,
                size_bytes: file_content.len(),
            });
        }

        MockRepository {
            files,
            total_lines,
            generated_at: chrono::Utc::now(),
        }
    }

    /// Generate realistic Rust code with proper patterns
    fn generate_rust_file(&mut self, config: &FileConfig) -> String {
        let mut content = String::new();

        // Add module documentation
        if self.rng.gen_bool(0.7) {
            content.push_str(&format!("//! {}\n\n", self.generate_module_doc()));
        }

        // Add imports
        let import_count = self.rng.gen_range(0..config.num_imports);
        for _ in 0..import_count {
            content.push_str(&self.generate_rust_import());
        }

        if !content.is_empty() {
            content.push('\n');
        }

        // Add module attributes
        if self.rng.gen_bool(0.3) {
            content.push_str("#[allow(dead_code)]\n");
        }

        // Generate structs
        let struct_count = self.rng.gen_range(1..config.num_structs);
        for i in 0..struct_count {
            content.push_str(&self.generate_rust_struct(i));
        }

        // Generate enums
        let enum_count = self.rng.gen_range(0..config.num_structs / 2);
        for i in 0..enum_count {
            content.push_str(&self.generate_rust_enum(i));
        }

        // Generate traits
        if self.rng.gen_bool(0.4) {
            content.push_str(&self.generate_rust_trait());
        }

        // Generate impl blocks
        for i in 0..struct_count {
            content.push_str(&self.generate_rust_impl(i));
        }

        // Generate functions
        let function_count = self.rng.gen_range(2..config.num_functions);
        for i in 0..function_count {
            content.push_str(&self.generate_rust_function(i));
        }

        // Generate tests
        if self.rng.gen_bool(0.6) {
            content.push_str("\n#[cfg(test)]\nmod tests {\n");
            content.push_str("    use super::*;\n\n");

            let test_count = self.rng.gen_range(1..3);
            for i in 0..test_count {
                content.push_str(&self.generate_rust_test(i));
            }

            content.push_str("}\n");
        }

        content
    }

    fn generate_rust_function(&mut self, index: usize) -> String {
        let visibility = if self.rng.gen_bool(0.8) { "pub " } else { "" };
        let async_keyword = if self.rng.gen_bool(0.3) { "async " } else { "" };
        let function_name = self.generate_function_name(index);
        let params = self.generate_rust_params();
        let return_type = self.generate_rust_return_type();
        let doc_comment = self.generate_doc_comment();
        let body = self.generate_rust_function_body(&return_type);

        format!(
            r#"{}/// {}
{}{}fn {}{} -> {} {{
    {}
}}
"#,
            doc_comment, async_keyword, visibility, function_name, params, return_type, body
        )
    }

    fn generate_rust_struct(&mut self, index: usize) -> String {
        let visibility = if self.rng.gen_bool(0.8) { "pub " } else { "" };
        let struct_name = self.generate_struct_name(index);
        let fields = self.generate_rust_struct_fields();
        let derives = self.generate_rust_derives();

        format!(
            r#"{}#[derive({})]
{}struct {} {{
{}
}}
"#,
            derives, fields, struct_name, fields
        )
    }

    fn generate_doc_comment(&mut self) -> String {
        let templates = vec![
            "Executes {} with proper error handling",
            "Processes {} efficiently with validation",
            "Performs {} operation with logging",
            "Handles {} computation with caching",
        ];

        let template = templates[self.rng.gen_range(0..templates.len())];
        let operation = self.generate_operation_name();

        format!("{} {}", template, operation)
    }

    fn generate_operation_name(&mut self) -> String {
        let operations = vec![
            "database query",
            "inference request",
            "data transformation",
            "validation check",
            "computation task",
            "API call",
            "file operation",
            "network request",
        ];

        operations[self.rng.gen_range(0..operations.len())].to_string()
    }
}

/// Test data builders for specific scenarios
pub struct TestDataBuilder;

impl TestDataBuilder {
    /// Create test database with realistic data distribution
    pub fn create_test_database(
        record_count: usize,
        size_distribution: ChunkSizeDistribution,
    ) -> TestDatabase {
        let mut generator = MockDataGenerator::new();
        let mut records = Vec::with_capacity(record_count);

        for i in 0..record_count {
            let chunk_size = size_distribution.sample(&mut generator.rng);
            let content = generator.generate_code_chunk(chunk_size);

            records.push(DatabaseRecord {
                id: RecordId(uuid::Uuid::new_v4()),
                content: Content::Text(content),
                metadata: RecordMetadata {
                    source: format!("test_repository_{}", i % 10),
                    content_type: ContentType::Code,
                    size_bytes: chunk_size,
                    processing_state: ProcessingState::Pending,
                    priority: Priority::Normal,
                    custom_fields: std::collections::HashMap::new(),
                },
                created_at: chrono::Utc::now() - chrono::Duration::minutes(i as i64),
                updated_at: chrono::Utc::now(),
            });
        }

        TestDatabase::new(records)
    }

    /// Create performance test scenarios with specific characteristics
    pub fn create_performance_scenarios() -> Vec<PerformanceTestScenario> {
        vec![
            // Small repository test
            PerformanceTestScenario {
                name: "Small Repository (1K LOC)".to_string(),
                record_count: 50,
                avg_chunk_size: 200,
                concurrency_levels: vec![1, 2, 4, 8],
                expected_throughput: 10.0,
                max_latency: Duration::from_millis(500),
                memory_limit_mb: 512,
            },

            // Medium repository test
            PerformanceTestScenario {
                name: "Medium Repository (50K LOC)".to_string(),
                record_count: 500,
                avg_chunk_size: 300,
                concurrency_levels: vec![4, 8, 16, 20],
                expected_throughput: 25.0,
                max_latency: Duration::from_millis(200),
                memory_limit_mb: 2048,
            },

            // Large repository stress test
            PerformanceTestScenario {
                name: "Large Repository (500K LOC)".to_string(),
                record_count: 2000,
                avg_chunk_size: 400,
                concurrency_levels: vec![8, 16, 20],
                expected_throughput: 50.0,
                max_latency: Duration::from_millis(100),
                memory_limit_mb: 8192,
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceTestScenario {
    pub name: String,
    pub record_count: usize,
    pub avg_chunk_size: usize,
    pub concurrency_levels: Vec<usize>,
    pub expected_throughput: f64,
    pub max_latency: Duration,
    pub memory_limit_mb: usize,
}

#[derive(Debug, Clone)]
pub enum ChunkSizeDistribution {
    Uniform { min: usize, max: usize },
    Normal { mean: f64, std_dev: f64 },
    Bimodal { small_mean: f64, large_mean: f64, small_prob: f64 },
}

impl ChunkSizeDistribution {
    pub fn sample(&self, rng: &mut StdRng) -> usize {
        match self {
            ChunkSizeDistribution::Uniform { min, max } => {
                rng.gen_range(*min..*max)
            }
            ChunkSizeDistribution::Normal { mean, std_dev } => {
                let normal = distributions::Normal::new(*mean, *std_dev).unwrap();
                normal.sample(rng).round().max(50.0) as usize
            }
            ChunkSizeDistribution::Bimodal { small_mean, large_mean, small_prob } => {
                if rng.gen::<f64>() < *small_prob {
                    let normal = distributions::Normal::new(*small_mean, small_mean / 4.0).unwrap();
                    normal.sample(rng).round().max(50.0) as usize
                } else {
                    let normal = distributions::Normal::new(*large_mean, large_mean / 4.0).unwrap();
                    normal.sample(rng).round().max(100.0) as usize
                }
            }
        }
    }
}
```

### 4.3 Property-Based Testing

#### Invariant-Based Testing
```rust
use proptest::prelude::*;
use proptest::collection::vec;

/// Property-based tests for database operations
proptest! {
    #[test]
    fn test_database_record_roundtrip(
        records in vec(any::<DatabaseRecord>(), 1..100)
    ) {
        // Property: Database records can be stored and retrieved without loss
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let db = create_test_database().await;

            // Store records
            let stored_ids = db.store_records(&records).await.unwrap();
            assert_eq!(stored_ids.len(), records.len());

            // Retrieve records
            let retrieved = db.get_records_by_ids(&stored_ids).await.unwrap();
            assert_eq!(retrieved.len(), records.len());

            // Verify content preservation
            for (original, retrieved) in records.iter().zip(retrieved.iter()) {
                prop_assert_eq!(original.content, retrieved.content);
                prop_assert_eq!(original.metadata.source, retrieved.metadata.source);
                prop_assert_eq!(original.metadata.content_type, retrieved.metadata.content_type);
            }
        });
    }

    #[test]
    fn test_chunking_invariants(
        content in prop::string::string_regex(r".{1,10000}", 100)?,
        chunk_size in 100..1000usize
    ) {
        // Property: Chunking preserves total content and maintains size constraints
        let chunker = SemanticChunker::new(chunk_size);
        let chunks = chunker.chunk_text(&content);

        // Invariant 1: Total content is preserved
        let reconstructed: String = chunks.iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        prop_assert_eq!(normalize_text(&content), normalize_text(&reconstructed));

        // Invariant 2: No chunk exceeds size limit significantly
        for chunk in &chunks {
            let lines = chunk.content.lines().count();
            prop_assert!(lines <= chunk_size + 10, "Chunk has {} lines, limit {}", lines, chunk_size);
        }

        // Invariant 3: Chunks are properly ordered
        for (i, chunk) in chunks.iter().enumerate() {
            prop_assert_eq!(chunk.index, i);
        }

        // Invariant 4: Each chunk has some content
        for chunk in &chunks {
            prop_assert!(!chunk.content.trim().is_empty(), "Empty chunk found");
        }
    }

    #[test]
    fn test_inference_consistency(
        inputs in vec(any::<String>(), 1..50)
    ) {
        // Property: Same input produces consistent output from inference engine
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let engine = create_mock_inference_engine().await;

            // First inference pass
            let mut results1 = Vec::new();
            for input in &inputs {
                let result = engine.infer(input.clone()).await.unwrap();
                results1.push(result);
            }

            // Second inference pass
            let mut results2 = Vec::new();
            for input in &inputs {
                let result = engine.infer(input.clone()).await.unwrap();
                results2.push(result);
            }

            // Verify consistency
            for (r1, r2) in results1.iter().zip(results2.iter()) {
                prop_assert_eq!(r1.text, r2.text);
                prop_assert_eq!(r1.token_count, r2.token_count);
            }
        });
    }

    #[test]
    fn test_pipeline_ordering_preservation(
        records in vec(any::<DatabaseRecord>(), 10..100)
    ) {
        // Property: Pipeline processing preserves input order
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let pipeline = create_test_pipeline().await;

            // Process records
            let results = pipeline.process_records(&records).await.unwrap();

            // Verify ordering preservation
            prop_assert_eq!(results.len(), records.len());

            for (original, processed) in records.iter().zip(results.iter()) {
                prop_assert_eq!(original.id, processed.record_id);
            }
        });
    }

    #[test]
    fn test_parallel_processing_idempotency(
        records in vec(any::<DatabaseRecord>(), 20..200),
        concurrency_levels in vec(1..=20, 2..=5)
    ) {
        // Property: Different concurrency levels produce same results
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let mut all_results = Vec::new();

            // Process with different concurrency levels
            for &concurrency in &concurrency_levels {
                let pipeline = create_test_pipeline_with_concurrency(concurrency).await;
                let results = pipeline.process_records(&records).await.unwrap();
                all_results.push(results);
            }

            // Verify all results are identical
            for results in &all_results[1..] {
                prop_assert_eq!(all_results[0].len(), results.len());

                for (r1, r2) in all_results[0].iter().zip(results.iter()) {
                    prop_assert_eq!(r1.record_id, r2.record_id);
                    prop_assert_eq!(r1.summary.text, r2.summary.text);
                }
            }
        });
    }
}

/// Property-based tests for performance characteristics
proptest! {
    #[test]
    fn test_memory_usage_scaling(
        base_record_count in 10..100usize,
        scale_factor in 2..10usize
    ) {
        // Property: Memory usage scales linearly with record count
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let pipeline = create_test_pipeline().await;

            // Test base size
            let base_records = generate_test_records(base_record_count);
            let base_memory = measure_memory_usage(|| async {
                pipeline.process_records(&base_records).await.unwrap()
            }).await;

            // Test scaled size
            let scaled_records = generate_test_records(base_record_count * scale_factor);
            let scaled_memory = measure_memory_usage(|| async {
                pipeline.process_records(&scaled_records).await.unwrap()
            }).await;

            // Memory should scale roughly linearly (allow 2x tolerance)
            let expected_max_memory = base_memory * scale_factor * 2;
            prop_assert!(scaled_memory <= expected_max_memory,
                "Memory scaled from {} to {} (factor {}), expected < {}",
                base_memory, scaled_memory, scaled_memory as f64 / base_memory as f64, expected_max_memory);
        });
    }

    #[test]
    fn test_throughput_scaling(
        base_concurrency in 1..5usize,
        max_concurrency in 6..20usize
    ) {
        // Property: Throughput increases with concurrency up to a point
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let test_records = generate_test_records(200);
            let mut throughputs = Vec::new();

            for concurrency in base_concurrency..=max_concurrency {
                let pipeline = create_test_pipeline_with_concurrency(concurrency).await;

                let start = std::time::Instant::now();
                pipeline.process_records(&test_records).await.unwrap();
                let duration = start.elapsed();

                let throughput = test_records.len() as f64 / duration.as_secs_f64();
                throughputs.push(throughput);
            }

            // Throughput should generally increase with concurrency
            // (allow some variance due to overhead)
            let avg_initial = throughputs[..3].iter().sum::<f64>() / 3.0;
            let avg_final = throughputs[throughputs.len()-3..].iter().sum::<f64>() / 3.0;

            prop_assert!(avg_final >= avg_initial * 0.8,
                "Throughput decreased significantly: {} -> {}", avg_initial, avg_final);
        });
    }
}

/// Utility functions for property-based testing
fn normalize_text(text: &str) -> String {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

async fn measure_memory_usage<F, Fut>(operation: F) -> usize
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let before = get_memory_usage();
    operation().await;
    let after = get_memory_usage();
    after.saturating_sub(before)
}

fn get_memory_usage() -> usize {
    use sysinfo::{SystemExt, ProcessExt};

    let mut system = sysinfo::System::new();
    system.refresh_all();

    let process = system.process(std::process::id() as usize);
    process.map(|p| p.memory()).unwrap_or(0)
}
```

### 4.4 Performance Contract Tests

#### Automated Performance Validation
```rust
/// Performance contract validation with automated metrics
pub struct PerformanceContractTests;

impl PerformanceContractTests {
    /// Test database query performance contracts
    pub async fn test_database_performance_contracts() -> TestResult {
        let db = create_test_database_with_real_data().await;
        let test_scenarios = vec![
            QueryPerformanceScenario {
                name: "Single record lookup".to_string(),
                query: "SELECT * FROM records WHERE id = ?".to_string(),
                params: vec!["test_id".into()],
                expected_max_latency: Duration::from_millis(10),
                expected_max_memory: 1024 * 1024, // 1MB
            },
            QueryPerformanceScenario {
                name: "Batch record retrieval".to_string(),
                query: "SELECT * FROM records WHERE created_at > ? LIMIT 100".to_string(),
                params: vec!["2024-01-01".into()],
                expected_max_latency: Duration::from_millis(50),
                expected_max_memory: 10 * 1024 * 1024, // 10MB
            },
            QueryPerformanceScenario {
                name: "Large aggregation query".to_string(),
                query: "SELECT COUNT(*) FROM records WHERE content LIKE ?".to_string(),
                params: vec!["%function%".into()],
                expected_max_latency: Duration::from_millis(200),
                expected_max_memory: 50 * 1024 * 1024, // 50MB
            },
        ];

        let mut results = Vec::new();

        for scenario in test_scenarios {
            let result = Self::run_query_performance_test(&db, &scenario).await;
            results.push((scenario.name.clone(), result));
        }

        let passed_count = results.iter().filter(|(_, r)| r.passed).count();
        let total_count = results.len();

        TestResult::performance_contract(
            format!("Database performance: {}/{}", passed_count, total_count),
            passed_count == total_count,
            results,
        )
    }

    /// Test inference engine performance contracts
    pub async fn test_inference_performance_contracts() -> TestResult {
        let engine = create_test_inference_engine().await;
        let test_cases = vec![
            InferenceTestCase {
                name: "Single short text".to_string(),
                input: "fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
                expected_max_latency: Duration::from_millis(100),
                expected_min_throughput: 10.0, // tokens/second
            },
            InferenceTestCase {
                name: "Medium code block".to_string(),
                input: generate_medium_code_block(),
                expected_max_latency: Duration::from_millis(500),
                expected_min_throughput: 20.0,
            },
            InferenceTestCase {
                name: "Large code block".to_string(),
                input: generate_large_code_block(),
                expected_max_latency: Duration::from_millis(2000),
                expected_min_throughput: 15.0,
            },
        ];

        let mut results = Vec::new();

        for test_case in test_cases {
            let result = Self::run_inference_performance_test(&engine, &test_case).await;
            results.push((test_case.name.clone(), result));
        }

        let passed_count = results.iter().filter(|(_, r)| r.passed).count();
        let total_count = results.len();

        TestResult::performance_contract(
            format!("Inference performance: {}/{}", passed_count, total_count),
            passed_count == total_count,
            results,
        )
    }

    /// Test pipeline end-to-end performance contracts
    pub async fn test_pipeline_performance_contracts() -> TestResult {
        let pipeline = create_test_pipeline().await;
        let scenarios = Self::create_pipeline_performance_scenarios();
        let mut results = Vec::new();

        for scenario in scenarios {
            let result = Self::run_pipeline_performance_test(&pipeline, &scenario).await;
            results.push((scenario.name.clone(), result));
        }

        let passed_count = results.iter().filter(|(_, r)| r.passed).count();
        let total_count = results.len();

        TestResult::performance_contract(
            format!("Pipeline performance: {}/{}", passed_count, total_count),
            passed_count == total_count,
            results,
        )
    }

    /// Run individual query performance test
    async fn run_query_performance_test(
        db: &dyn DatabaseProvider,
        scenario: &QueryPerformanceScenario,
    ) -> PerformanceTestResult {
        let start_time = std::time::Instant::now();
        let memory_before = get_memory_usage();

        // Execute query multiple times for average
        let mut latencies = Vec::new();
        for _ in 0..10 {
            let query_start = std::time::Instant::now();
            let _result = db.execute_query(&scenario.query, scenario.params.clone()).await;
            latencies.push(query_start.elapsed());
        }

        let memory_after = get_memory_usage();
        let total_duration = start_time.elapsed();
        let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        let memory_used = memory_after.saturating_sub(memory_before);

        let passed = avg_latency <= scenario.expected_max_latency
            && memory_used <= scenario.expected_max_memory;

        PerformanceTestResult {
            passed,
            actual_latency: avg_latency,
            expected_max_latency: scenario.expected_max_latency,
            memory_used,
            expected_max_memory: scenario.expected_max_memory,
            total_duration,
            details: format!(
                "Latency: {:?} (max: {:?}), Memory: {}MB (max: {}MB)",
                avg_latency, scenario.expected_max_latency,
                memory_used / 1024 / 1024, scenario.expected_max_memory / 1024 / 1024
            ),
        }
    }

    /// Run individual inference performance test
    async fn run_inference_performance_test(
        engine: &dyn InferenceEngine,
        test_case: &InferenceTestCase,
    ) -> PerformanceTestResult {
        let start_time = std::time::Instant::now();
        let memory_before = get_memory_usage();

        // Execute inference multiple times
        let mut latencies = Vec::new();
        let mut token_counts = Vec::new();

        for _ in 0..5 {
            let inference_start = std::time::Instant::now();
            let result = engine.infer(test_case.input.clone()).await.unwrap();
            latencies.push(inference_start.elapsed());
            token_counts.push(result.token_count);
        }

        let memory_after = get_memory_usage();
        let total_duration = start_time.elapsed();
        let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        let total_tokens: usize = token_counts.iter().sum();
        let avg_tokens = total_tokens / token_counts.len();
        let throughput = avg_tokens as f64 / avg_latency.as_secs_f64();
        let memory_used = memory_after.saturating_sub(memory_before);

        let passed = avg_latency <= test_case.expected_max_latency
            && throughput >= test_case.expected_min_throughput;

        PerformanceTestResult {
            passed,
            actual_latency: avg_latency,
            expected_max_latency: test_case.expected_max_latency,
            memory_used,
            expected_max_memory: 100 * 1024 * 1024, // 100MB default
            total_duration,
            details: format!(
                "Latency: {:?} (max: {:?}), Throughput: {:.1} tokens/s (min: {:.1})",
                avg_latency, test_case.expected_max_latency,
                throughput, test_case.expected_min_throughput
            ),
        }
    }

    /// Create pipeline performance test scenarios
    fn create_pipeline_performance_scenarios() -> Vec<PipelinePerformanceScenario> {
        vec![
            PipelinePerformanceScenario {
                name: "Small batch (100 records)".to_string(),
                record_count: 100,
                concurrency_level: 4,
                expected_max_duration: Duration::from_secs(10),
                expected_max_memory: 512 * 1024 * 1024, // 512MB
                expected_min_throughput: 10.0, // records/second
            },
            PipelinePerformanceScenario {
                name: "Medium batch (500 records)".to_string(),
                record_count: 500,
                concurrency_level: 10,
                expected_max_duration: Duration::from_secs(30),
                expected_max_memory: 2 * 1024 * 1024 * 1024, // 2GB
                expected_min_throughput: 15.0,
            },
            PipelinePerformanceScenario {
                name: "Large batch (2000 records)".to_string(),
                record_count: 2000,
                concurrency_level: 20,
                expected_max_duration: Duration::from_secs(60),
                expected_max_memory: 8 * 1024 * 1024 * 1024, // 8GB
                expected_min_throughput: 30.0,
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub struct QueryPerformanceScenario {
    pub name: String,
    pub query: String,
    pub params: QueryParams,
    pub expected_max_latency: Duration,
    pub expected_max_memory: usize,
}

#[derive(Debug, Clone)]
pub struct InferenceTestCase {
    pub name: String,
    pub input: String,
    pub expected_max_latency: Duration,
    pub expected_min_throughput: f64,
}

#[derive(Debug, Clone)]
pub struct PipelinePerformanceScenario {
    pub name: String,
    pub record_count: usize,
    pub concurrency_level: usize,
    pub expected_max_duration: Duration,
    pub expected_max_memory: usize,
    pub expected_min_throughput: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub passed: bool,
    pub actual_latency: Duration,
    pub expected_max_latency: Duration,
    pub memory_used: usize,
    pub expected_max_memory: usize,
    pub total_duration: Duration,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_type: String,
    pub passed: bool,
    pub details: String,
    pub sub_results: Vec<(String, PerformanceTestResult)>,
}

impl TestResult {
    pub fn performance_contract(
        name: String,
        passed: bool,
        results: Vec<(String, PerformanceTestResult)>,
    ) -> Self {
        let details = if passed {
            "All performance contracts met".to_string()
        } else {
            format!(
                "Failed contracts: {}",
                results.iter()
                    .filter(|(_, r)| !r.passed)
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        Self {
            test_type: "performance_contract".to_string(),
            passed,
            details,
            sub_results: results,
        }
    }
}
```

---

## 5. Implementation Roadmap

### 5.1 Migration Phases

#### Phase 1: Foundation Infrastructure (Week 1-2)
**Objective**: Establish core TDD infrastructure and basic trait system

**Tasks**:
1. **Error Handling System**
   - Implement comprehensive error taxonomy with thiserror
   - Add error context and recovery strategies
   - Create error conversion traits for external dependencies
   - Add structured logging with tracing

2. **Trait System Foundation**
   - Define core traits for DatabaseProvider, InferenceEngine, PipelineOrchestrator
   - Implement associated types and lifetime management
   - Add trait bounds validation tests
   - Create mock implementations for testing

3. **Configuration Management**
   - Implement hierarchical configuration system
   - Add environment variable support and validation
   - Create configuration builders with type-state patterns
   - Add configuration migration and versioning

4. **Testing Infrastructure**
   - Set up comprehensive test organization (unit/integration/property)
   - Implement mock data generators and test factories
   - Add performance contract testing framework
   - Create test utilities and helpers

**Deliverables**:
- Working error handling system throughout codebase
- Core trait definitions with mock implementations
- Configuration system with validation
- Comprehensive test infrastructure

#### Phase 2: Database Integration (Week 2-3)
**Objective**: Implement CozoDB integration with connection pooling and streaming

**Tasks**:
1. **CozoDB Client Implementation**
   - Create CozoDB provider implementation with connection pooling
   - Implement query builder with parameter binding
   - Add transaction management and rollback support
   - Create database schema and migration system

2. **Connection Pool Management**
   - Implement circuit breaker pattern for database connections
   - Add health checking and automatic reconnection
   - Create connection metrics and monitoring
   - Add backpressure support for database operations

3. **Streaming and Batch Processing**
   - Implement stream processing for large datasets
   - Add batch operation support with transaction safety
   - Create query optimization and caching
   - Add database-specific performance optimizations

4. **Testing and Validation**
   - Add comprehensive database tests with integration testing
   - Create performance benchmarks for database operations
   - Implement chaos engineering for database failures
   - Add property-based tests for database invariants

**Deliverables**:
- Fully functional CozoDB integration
- Connection pooling with circuit breaking
- Streaming and batch processing capabilities
- Comprehensive database test suite

#### Phase 3: Candle RS Integration (Week 3-4)
**Objective**: Implement Candle RS inference engine with Metal acceleration

**Tasks**:
1. **Candle RS Engine Implementation**
   - Create Candle RS inference engine with model loading
   - Implement Metal GPU acceleration with fallback to CPU
   - Add quantization support and model optimization
   - Create session pool management for parallel processing

2. **Model Management**
   - Implement model loading with device selection
   - Add model caching and version management
   - Create model metadata and capability detection
   - Add model health checking and recovery

3. **Performance Optimization**
   - Implement batch inference with parallel processing
   - Add memory pooling and buffer reuse
   - Create SIMD-optimized text processing
   - Add cache-friendly data structures

4. **Testing and Validation**
   - Add comprehensive inference tests with performance validation
   - Create benchmarks for different model configurations
   - Implement integration tests with Metal acceleration
   - Add property-based tests for inference consistency

**Deliverables**:
- Fully functional Candle RS inference engine
- Metal GPU acceleration with CPU fallback
- Session pool management for parallel processing
- Comprehensive inference test suite

#### Phase 4: Pipeline Integration (Week 4-5)
**Objective**: Implement complete database-to-summary pipeline with monitoring

**Tasks**:
1. **Pipeline Orchestration**
   - Create main pipeline orchestrator with flow control
   - Implement stream processing with backpressure
   - Add adaptive concurrency management
   - Create pipeline state management and recovery

2. **Monitoring and Observability**
   - Implement comprehensive metrics collection
   - Add health checking and status reporting
   - Create performance monitoring with alerting
   - Add distributed tracing support

3. **Error Handling and Recovery**
   - Implement graceful degradation strategies
   - Add automatic retry with exponential backoff
   - Create checkpoint and resume functionality
   - Add circuit breaker patterns for resilience

4. **Integration and Testing**
   - Add end-to-end pipeline tests with realistic data
   - Create performance benchmarks for complete pipeline
   - Implement chaos engineering for failure scenarios
   - Add load testing and scalability validation

**Deliverables**:
- Complete database-to-summary pipeline
- Comprehensive monitoring and observability
- Robust error handling and recovery mechanisms
- End-to-end integration test suite

#### Phase 5: Production Hardening (Week 5-6)
**Objective**: Prepare system for production deployment with comprehensive validation

**Tasks**:
1. **Performance Optimization**
   - Fine-tune Metal GPU performance and memory usage
   - Optimize database queries and indexing strategies
   - Implement advanced caching strategies
   - Add performance profiling and optimization tools

2. **Security and Compliance**
   - Add authentication and authorization mechanisms
   - Implement data encryption at rest and in transit
   - Add audit logging and compliance reporting
   - Create security testing and validation

3. **Documentation and Deployment**
   - Create comprehensive API documentation
   - Add deployment guides and configuration reference
   - Create troubleshooting and operational runbooks
   - Add training materials and best practices

4. **Production Readiness**
   - Implement production monitoring and alerting
   - Add disaster recovery and backup procedures
   - Create capacity planning and scaling guides
   - Add production performance validation

**Deliverables**:
- Production-ready system with performance optimization
- Security and compliance features
- Comprehensive documentation and deployment guides
- Production readiness validation

### 5.2 Risk Mitigation Strategies

#### Technical Risks
1. **Candle RS Migration Complexity**
   - **Risk**: Incomplete understanding of Candle RS patterns
   - **Mitigation**: Incremental migration with feature flags, extensive prototyping

2. **Metal GPU Compatibility**
   - **Risk**: Metal acceleration issues on different Apple Silicon versions
   - **Mitigation**: Comprehensive testing across M1/M2/M3, CPU fallback path

3. **Database Performance Bottlenecks**
   - **Risk**: CozoDB performance issues with large datasets
   - **Mitigation**: Connection pooling, query optimization, performance monitoring

4. **Memory Management**
   - **Risk**: Memory leaks or excessive memory usage
   - **Mitigation**: RAII patterns, memory profiling, automated limits

#### Operational Risks
1. **Performance Regression**
   - **Risk**: New system slower than existing ONNX implementation
   - **Mitigation**: Comprehensive benchmarking, performance contracts

2. **Integration Complexity**
   - **Risk**: Difficulties integrating database and inference components
   - **Mitigation**: Modular design, comprehensive integration testing

3. **Deployment Complexity**
   - **Risk**: Production deployment challenges
   - **Mitigation**: Containerization, automated deployment, staging environment

### 5.3 Success Criteria

#### Technical Success Criteria
- ✅ **Performance**: Meet or exceed 1000+ records/minute throughput target
- ✅ **Memory**: Maintain < 8GB memory footprint under load
- ✅ **Latency**: Achieve < 50ms average inference time per record
- ✅ **Scalability**: Linear scaling up to 20 concurrent agents
- ✅ **Reliability**: 99.9% uptime with graceful error handling
- ✅ **Test Coverage**: > 90% test coverage with comprehensive property-based tests

#### Operational Success Criteria
- ✅ **Monitoring**: Complete observability with metrics and alerting
- ✅ **Documentation**: Comprehensive documentation for development and operations
- ✅ **Deployment**: Automated deployment with rollback capabilities
- ✅ **Security**: Authentication, authorization, and data encryption
- ✅ **Maintainability**: Clear code organization with good separation of concerns

---

## 6. Conclusion

This technical architecture provides a comprehensive foundation for implementing the Dobby database-to-summary pipeline using CozoDB and Candle RS while maintaining the high-performance parallel processing strengths of the existing system.

### Key Architectural Benefits

1. **TDD-First Development**: Every requirement backed by executable tests and measurable outcomes
2. **Idiomatic Rust Design**: Proper use of traits, ownership, and Rust patterns throughout
3. **Performance-Optimized**: Metal acceleration, memory pooling, and adaptive concurrency
4. **Production-Ready**: Comprehensive monitoring, error handling, and deployment strategies
5. **Future-Proof**: Extensible architecture with plugin system and clear interfaces

### Migration Strategy

The phased migration approach ensures minimal disruption while maximizing learning and validation at each step. The incremental approach allows for:

- **Risk Mitigation**: Early identification and resolution of technical challenges
- **Performance Validation**: Continuous benchmarking against performance contracts
- **Knowledge Building**: Gradual learning of Candle RS and CozoDB patterns
- **Stakeholder Confidence**: Regular delivery of working functionality

### Next Steps

1. **Begin Phase 1**: Implement error handling and trait system foundation
2. **Setup Development Environment**: Configure CozoDB and Candle RS development tools
3. **Create Test Infrastructure**: Implement mock data generators and test frameworks
4. **Start Development**: Begin with database integration while maintaining test-first approach

This architecture provides a solid foundation for building a high-performance, reliable, and maintainable database-to-summary pipeline that leverages the latest Rust ecosystem capabilities while following best practices for TDD development.