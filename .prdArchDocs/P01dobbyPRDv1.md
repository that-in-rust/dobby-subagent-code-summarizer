# Dobby Code Summarizer - Product Requirements v1.1

## Project Overview

**REVISED SCOPE**: Dobby is a Rust-native database-to-summary pipeline that processes CozoDB tables containing code/content data using Candle RS-powered parallel AI inference. The system has been simplified from the original file-based parsing architecture to focus on high-performance database processing with true parallelism.

**Core Value Proposition**:
- **Database-First Processing**: Direct CozoDB table consumption (no file parsing/chunking)
- **Candle RS Native**: Rust-native inference with Apple Silicon Metal acceleration
- **Extreme Parallelism**: 20x concurrent processing for database record summarization
- **Simplified Architecture**: Streamlined pipeline focused on inference performance

## Architecture

Single-crate architecture optimized for database record processing:

```
dobby-subagent-code-summarizer/
├── Cargo.toml                        # Main dependencies (Candle RS + CozoDB)
├── src/
│   ├── lib.rs                       # Main library interface
│   ├── database/                    # NEW: CozoDB integration layer
│   │   ├── mod.rs                   # Database module interface
│   │   ├── client.rs                # CozoDB connection & queries
│   │   └── models.rs                # Database record structures
│   ├── candle_engine/               # NEW: Candle RS inference engine
│   │   ├── mod.rs                   # Candle module interface
│   │   ├── inference.rs             # Core inference logic
│   │   ├── session_pool.rs          # Parallel session management
│   │   └── metal_acceleration.rs    # Apple Silicon optimization
│   ├── parallel_agents.rs           # ADAPTED: 20-agent parallel processing
│   ├── config.rs                    # UPDATED: Database + model configuration
│   └── errors.rs                    # Error handling
└── src/bin/
    └── parallel_summarizer.rs       # Database-first CLI interface
```

### Data Flow Architecture
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   CozoDB Table  │───▶│  Candle Engine   │───▶│  Summary Output │
│  (id, content)  │    │  (20x Parallel)  │    │   + Metadata    │
│  Primary Keys   │    │  Metal Accelerated│    │  Database Link │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

---

## Core Feature: Database-to-Summary Pipeline

### Purpose
High-performance processing of CozoDB tables containing code/content data using Candle RS parallel inference with Apple Silicon Metal acceleration.

### Database Input Schema
**Expected CozoDB Table Structure**:
```sql
CREATE TABLE code_data (
    id TEXT PRIMARY KEY,           -- Primary key for record identification
    content TEXT NOT NULL,         -- Code/text content to summarize
    metadata TEXT,                 -- Optional metadata (JSON format)
    created_at DATETIME           -- Optional timestamp
);
```

### Key Capabilities
- **Direct Database Integration**: Native CozoDB client with efficient querying
- **Candle RS Inference**: Rust-native neural inference with no FFI overhead
- **Metal Acceleration**: Apple Silicon GPU acceleration for maximum throughput
- **20x Parallel Processing**: True concurrent processing of database records
- **Session Pool Management**: Efficient Candle RS session lifecycle and reuse
- **Memory Optimization**: Unified memory management for large-scale processing
- **Progress Monitoring**: Real-time progress tracking and performance metrics

### Database Processing Workflow
```rust
// Simplified processing pipeline
pub async fn process_database_table(
    db_config: &DatabaseConfig,
    candle_config: &CandleConfig,
    parallel_config: &ParallelConfig,
) -> Result<Vec<ProcessedSummary>> {

    // 1. Connect to CozoDB and fetch records
    let db_client = CozoDBClient::new(db_config).await?;
    let records = db_client.fetch_records().await?; // (id, content) pairs

    // 2. Initialize Candle RS parallel engine
    let candle_engine = CandleParallelEngine::new(candle_config).await?;

    // 3. Process records with 20x parallelism
    let futures = records.into_iter().map(|record| {
        candle_engine.summarize_record(record)
    });

    // 4. Collect results with structured metadata
    let results = futures::future::join_all(futures).await;

    Ok(results)
}
```

### CLI Interface

```bash
# Database processing with 20x parallel Candle RS inference
dobby summarize \
    --db-connection "sqlite:///data.db" \
    --table-name code_data \
    --key-column id \
    --value-column content \
    --output ./summaries.md \
    --parallel 20 \
    --model qwen2.5-0.5b-candle \
    --device metal

# High-throughput batch processing
dobby summarize \
    --db-connection "cozo://localhost:6421" \
    --table-name large_dataset \
    --key-column uuid \
    --value-column source_code \
    --output ./batch_summaries.md \
    --parallel 20 \
    --batch-size 1000 \
    --progress-log ./progress.log

# Development with CPU fallback
dobby summarize \
    --db-connection "./local.db" \
    --table-name test_data \
    --key-column record_id \
    --value-column text_content \
    --output ./dev_summaries.md \
    --parallel 5 \
    --device cpu \
    --debug-mode
```

### Core Parameters
- `--db-connection`: CozoDB connection string (required)
- `--table-name`: Source table name (required)
- `--key-column`: Primary key column name (required)
- `--value-column`: Content/value column name (required)
- `--output`: Summary output file path (required)
- `--parallel`: Number of parallel agents (default: 20, max: 50)
- `--model`: Candle RS model selection (qwen2.5-0.5b, smollm2-135m, etc.)
- `--device`: Target device (metal, cpu, auto)
- `--batch-size`: Records to process per batch (default: 500)
- `--progress-log`: Progress tracking log file
- `--debug-mode`: Enable debug logging and metrics

---

## Candle RS Integration

### Model Support
**Supported Models**:
```rust
// Candle RS model configurations
pub enum CandleModel {
    Qwen2_5_0_5B,      // Primary: Qwen2.5-0.5B-Instruct
    SmolLM2_135M,      // Fast: SmolLM2-135M
    SmolLM2_360M,      // Balanced: SmolLM2-360M
    Custom(String),    // Custom model path
}
```

### Metal Acceleration
**Apple Silicon Optimization**:
- **Metal Backend**: Native Apple Silicon GPU acceleration
- **Unified Memory**: Efficient RAM/VRAM management
- **Batch Processing**: Optimized for multiple concurrent inferences
- **Memory Coalescing**: Reduced memory allocation overhead

### Session Pool Architecture
```rust
pub struct CandleSessionPool {
    sessions: Arc<Vec<CandleSession>>,    // Pool of Candle sessions
    device: Device,                       // Metal/CPU device
    semaphore: Arc<Semaphore>,            // 20x concurrency control
    tokenizer: Arc<Tokenizer>,            // Shared tokenizer
}

pub struct CandleSession {
    model: Arc<candle_nn::Model>,         // Loaded model
    device: Device,                       // Session device
    tokenizer: Arc<Tokenizer>,            // Session tokenizer
}
```

---

## Configuration

### Database Configuration (`database.toml`)
```toml
[database]
connection_string = "sqlite:///data.db"
table_name = "code_data"
key_column = "id"
value_column = "content"
metadata_column = "metadata"  # Optional

[connection]
pool_size = 10
timeout_seconds = 30
retry_attempts = 3
```

### Candle RS Configuration (`candle.toml`)
```toml
[candle]
model = "qwen2.5-0.5b"
model_path = "./models/qwen2.5-0.5b-candle"
device = "metal"
quantization = "q4_0"

[performance]
max_parallel_sessions = 20
batch_size = 500
memory_limit_gb = 8
timeout_seconds = 120

[metal]
acceleration = true
memory_fraction = 0.8
unified_memory = true
```

### Parallel Processing Configuration (`parallel.toml`)
```toml
[parallel]
max_agents = 20
semaphore_permits = 20
queue_depth = 100
worker_threads = 8

[monitoring]
progress_tracking = true
performance_metrics = true
error_logging = true
resource_monitoring = true
```

---

## Implementation Strategy

### Migration Path (Current ONNX → Candle RS)

**Phase 1: Database Integration (Week 1-2)**
- Implement CozoDB client and connection management
- Create database record processing pipeline
- Replace file-based input with database table queries
- Update CLI for database-first operation

**Phase 2: Candle RS Migration (Week 2-4)**
- Replace ONNX Runtime dependencies with Candle RS
- Implement Metal acceleration for Apple Silicon
- Migrate session management patterns to Candle RS
- Preserve existing 20-agent parallel architecture

**Phase 3: Integration & Optimization (Week 4-5)**
- Unified database-to-summary pipeline testing
- Performance optimization for Metal backend
- Memory management and session pooling tuning
- Production deployment preparation

### Dependencies Update

**Remove Current Dependencies**:
```toml
# Remove from Cargo.toml
ort = "1.16.3"              # ONNX Runtime (replacing)
ndarray = "0.15"            # ONNX tensor format
```

**Add New Dependencies**:
```toml
# Add to Cargo.toml
candle-core = "0.6"         # Core Candle RS library
candle-nn = "0.6"           # Neural network primitives
candle-transformers = "0.6" # Transformer models
candle-flash-attn = "0.6"   # Flash attention (optional)
cozo = "0.10"               # CozoDB client
tokio-util = "0.7"          # Async utilities
serde_cbor = "0.11"         # Efficient serialization
```

---

## Performance Targets

### Database Processing Performance
**Throughput Goals**:
- **Target**: 20x parallel processing with 5-10ms per record inference
- **Baseline**: Current 20-agent ONNX architecture (non-functional)
- **Success Metric**: Stable processing of 1000+ records/minute

**Memory Efficiency**:
- **Model Memory**: < 2GB for Qwen2.5-0.5B (Candle RS quantized)
- **Session Pool**: < 4GB total for 20 concurrent sessions
- **Database Connection**: < 100MB for connection pooling

### Metal Acceleration Benefits
**Expected Performance Gains**:
- **Inference Speed**: 3-5x faster than CPU inference
- **Memory Bandwidth**: 100+ GB/s unified memory access
- **Parallel Efficiency**: 90%+ scaling for 20 concurrent agents

---

## Risk Assessment & Mitigation

### Technical Risks

**Candle RS Migration (High Risk)**
- **Risk**: Complete inference stack replacement
- **Mitigation**: Incremental migration with fallback to CPU
- **Success Criteria**: Functional Metal acceleration with 20x parallelism

**CozoDB Integration (Medium Risk)**
- **Risk**: Database performance bottlenecks
- **Mitigation**: Connection pooling and batch processing
- **Success Criteria**: Stream processing without blocking inference

**Memory Management (Medium Risk)**
- **Risk**: Metal memory allocation conflicts
- **Mitigation**: Unified memory management and session pooling
- **Success Criteria**: Stable 20-session operation without memory leaks

### Operational Risks

**Performance Regression (Medium Risk)**
- **Risk**: Candle RS performance lower than ONNX expectations
- **Mitigation**: Performance benchmarking and optimization cycles
- **Success Criteria**: Meet or exceed target throughput metrics

**Apple Silicon Compatibility (Low Risk)**
- **Risk**: Metal backend compatibility issues
- **Mitigation**: CPU fallback path and extensive testing
- **Success Criteria**: Functional Metal acceleration on M1/M2/M3

---

## Success Criteria

### Technical Deliverables
✅ **Database Integration**: Functional CozoDB client with table processing
✅ **Candle RS Inference**: Working Rust-native inference pipeline
✅ **Metal Acceleration**: Apple Silicon GPU optimization
✅ **Parallel Processing**: 20x concurrent agent processing
✅ **CLI Interface**: Database-first command-line interface

### Performance Targets
✅ **Throughput**: 1000+ records/minute processing capability
✅ **Latency**: < 50ms average inference time per record
✅ **Scalability**: Linear scaling up to 20 parallel agents
✅ **Memory**: < 8GB total memory footprint for full pipeline
✅ **Stability**: 24+ hour continuous operation without failures

### Quality Gates
✅ **Error Handling**: Graceful degradation and retry mechanisms
✅ **Monitoring**: Progress tracking and performance metrics
✅ **Documentation**: Complete API and deployment documentation
✅ **Testing**: Comprehensive test coverage for database and inference components

---

## Development Workflow

### Environment Setup
```bash
# Prerequisites
rustup default stable
brew install cozo-db  # CozoDB installation

# Development setup
cargo build
cargo test --features metal-tests

# Model preparation
mkdir -p ./models/qwen2.5-0.5b-candle
# Download Candle RS compatible models
```

### Testing Strategy
**Unit Tests**:
- Database connection and query handling
- Candle RS model loading and inference
- Session pool management
- Configuration parsing and validation

**Integration Tests**:
- End-to-end database-to-summary pipeline
- Metal acceleration performance
- Parallel processing stress testing
- Error recovery and resilience

**Performance Tests**:
- Throughput benchmarking with various record counts
- Memory usage profiling under parallel load
- Metal vs CPU performance comparison
- Scalability testing up to 20 agents

---

**Last Updated**: 2025-10-27
**Version**: v1.1 - Database-First Architecture with Candle RS
**Status**: Ready for Implementation
**Next Milestone**: Phase 1 - Database Integration (Week 1-2)

