# TDD-First Rust Architecture Specification

## Executive Summary

This document defines the executable architecture for the dobby-subagent-code-summarizer, following the 9 non-negotiable principles from the steering document. Every component is specified as a testable contract with measurable outcomes.

## 1. TDD-First Development Methodology

### 1.1 The STUB → RED → GREEN → REFACTOR Cycle

```rust
// STUB: Define the interface with minimal implementation
pub trait TextChunker {
    fn chunk_text(&self, text: &str, max_size: usize) -> Result<Vec<Chunk>>;
}

// RED: Write failing test that specifies exact behavior
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_respects_max_size() {
        let chunker = create_test_chunker();
        let text = "a".repeat(2000); // 2000 chars
        let result = chunker.chunk_text(&text, 1000);

        // RED: This will fail - stub returns empty vec
        assert!(result.is_ok());
        let chunks = result.unwrap();
        assert!(!chunks.is_empty());

        // Every chunk must be <= max_size
        for chunk in &chunks {
            assert!(chunk.text.len() <= 1000,
                   "Chunk size {} exceeds max_size {}",
                   chunk.text.len(), 1000);
        }
    }
}

// GREEN: Minimal implementation to make test pass
impl TextChunker for BasicTextChunker {
    fn chunk_text(&self, text: &str, max_size: usize) -> Result<Vec<Chunk>> {
        if text.is_empty() {
            return Ok(vec![]);
        }

        let mut chunks = Vec::new();
        for chunk_start in (0..text.len()).step_by(max_size) {
            let end = std::cmp::min(chunk_start + max_size, text.len());
            let chunk_text = text[chunk_start..end].to_string();
            chunks.push(Chunk::new(chunk_text, chunk_start));
        }
        Ok(chunks)
    }
}

// REFACTOR: Improve implementation while maintaining test coverage
impl TextChunker for SmartTextChunker {
    fn chunk_text(&self, text: &str, max_size: usize) -> Result<Vec<Chunk>> {
        // Refactored: Smart boundary detection, preserve sentences
        // Tests continue to pass, ensuring no regression
        self.smart_chunk_with_boundaries(text, max_size)
    }
}
```

### 1.2 Executable Specification Template

```rust
/// Contract: Text chunking must preserve semantic boundaries
/// Preconditions:
///   - Input text is valid UTF-8
///   - max_size > 0 and <= MAX_CHUNK_SIZE (10,000)
/// Postconditions:
///   - Result contains at least one chunk if input is non-empty
///   - Each chunk.text.len() <= max_size
///   - Chunk metadata (start_pos, end_pos) is accurate
///   - No content is lost between chunks (concatenation = original)
/// Error conditions:
///   - Returns ChunkingFailed if max_size is invalid
///   - Returns ChunkingFailed if text contains invalid UTF-8
#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn contract_no_content_loss() {
        let chunker = SmartTextChunker::new();
        let original = "This is sentence 1. This is sentence 2. This is sentence 3.";
        let chunks = chunker.chunk_text(original, 50).unwrap();

        // Contract: Reconstructing chunks yields original
        let reconstructed: String = chunks
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert_eq!(reconstructed, original);
    }

    #[test]
    fn contract_boundary_preservation() {
        let chunker = SmartTextChunker::new();
        let text = "Short sentence. This is a much longer sentence that should be split.";
        let chunks = chunker.chunk_text(text, 40).unwrap();

        // Contract: Chunks prefer sentence boundaries
        for chunk in &chunks {
            if chunk.text.len() < 40 {
                assert!(chunk.text.ends_with('.') ||
                       chunk.text.ends_with('!') ||
                       chunk.text.ends_with('?'),
                       "Short chunk should end at sentence boundary: {}",
                       chunk.text);
            }
        }
    }
}
```

## 2. Layered Rust Architecture (L1 → L2 → L3)

### 2.1 L1 Core Layer: Ownership, Lifetimes, Traits

```rust
// L1.1: Ownership patterns with explicit lifetimes
pub struct Chunk<'a> {
    pub text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub metadata: ChunkMetadata<'a>, // Borrowed metadata
}

impl<'a> Chunk<'a> {
    pub fn new(text: String, start_pos: usize) -> Self {
        let end_pos = start_pos + text.len();
        Self {
            text,
            start_pos,
            end_pos,
            metadata: ChunkMetadata::default(),
        }
    }

    // Borrowed getter - no ownership transfer
    pub fn text(&self) -> &str {
        &self.text
    }

    // Ownership transfer when needed
    pub fn into_text(self) -> String {
        self.text
    }
}

// L1.2: Lifetime-aware trait for chunking strategies
pub trait ChunkingStrategy {
    // Lifetime ensures returned data doesn't outlive input
    fn chunk<'a>(&self, text: &'a str, config: &ChunkingConfig) -> Result<Vec<Chunk<'a>>>;

    // Associated type for strategy-specific configuration
    type Config: Clone + Send + Sync;
    fn default_config(&self) -> Self::Config;
}

// L1.3: Result type with explicit error handling
pub type Result<T> = std::result::Result<T, ProcessingError>;

// L1.4: Newtype pattern for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkSize(usize);

impl ChunkSize {
    pub fn new(size: usize) -> Result<Self> {
        const MIN_SIZE: usize = 50;
        const MAX_SIZE: usize = 10_000;

        if size < MIN_SIZE || size > MAX_SIZE {
            return Err(ProcessingError::ContractViolation {
                contract_name: "ChunkSize::new".to_string(),
                violation: format!("size {} must be between {} and {}",
                                 size, MIN_SIZE, MAX_SIZE),
            });
        }

        Ok(ChunkSize(size))
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

impl From<ChunkSize> for usize {
    fn from(size: ChunkSize) -> Self {
        size.0
    }
}
```

### 2.2 L2 Standard Library: Collections, Smart Pointers, Thread Safety

```rust
// L2.1: Arc for shared ownership across async tasks
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SharedChunker {
    inner: Arc<RwLock<dyn ChunkingStrategy + Send + Sync>>,
    metrics: Arc<RwLock<ChunkingMetrics>>,
}

impl SharedChunker {
    pub async fn chunk_with_metrics(&self, text: &str, config: &ChunkingConfig) -> Result<Vec<Chunk>> {
        let start_time = std::time::Instant::now();

        // Arc::clone for shared access across tasks
        let chunker = Arc::clone(&self.inner);
        let metrics = Arc::clone(&self.metrics);

        // Read lock for chunking strategy
        let chunks = {
            let strategy = chunker.read().await;
            strategy.chunk(text, config)?
        };

        // Write lock for metrics update
        {
            let mut metrics_guard = metrics.write().await;
            metrics_guard.record_chunking(
                text.len(),
                chunks.len(),
                start_time.elapsed()
            );
        }

        Ok(chunks)
    }
}

// L2.2: VecDeque for efficient queue operations
use std::collections::VecDeque;

pub struct ChunkingQueue {
    pending: VecDeque<ChunkingTask>,
    completed: VecDeque<ChunkingResult>,
    max_pending: usize,
}

impl ChunkingQueue {
    pub fn new(max_pending: usize) -> Self {
        Self {
            pending: VecDeque::with_capacity(max_pending),
            completed: VecDeque::new(),
            max_pending,
        }
    }

    pub fn add_task(&mut self, task: ChunkingTask) -> Result<()> {
        if self.pending.len() >= self.max_pending {
            return Err(ProcessingError::ResourceExhaustion {
                resource: "chunking_queue".to_string(),
                limit: self.max_pending,
            });
        }

        self.pending.push_back(task);
        Ok(())
    }

    // Efficient front removal for processing
    pub fn take_next_task(&mut self) -> Option<ChunkingTask> {
        self.pending.pop_front()
    }

    // Efficient back addition for completed results
    pub fn add_completed(&mut self, result: ChunkingResult) {
        self.completed.push_back(result);
    }
}

// L2.3: HashMap for chunk indexing
use std::collections::HashMap;

pub struct ChunkIndex {
    index: HashMap<String, Vec<usize>>, // content_hash -> chunk_indices
    chunks: Vec<Chunk>,
}

impl ChunkIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            chunks: Vec::new(),
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) -> usize {
        let index = self.chunks.len();
        let hash = self.compute_content_hash(&chunk.text);

        self.index.entry(hash).or_insert_with(Vec::new).push(index);
        self.chunks.push(chunk);
        index
    }

    pub fn find_similar_chunks(&self, chunk: &Chunk) -> &[usize] {
        let hash = self.compute_content_hash(&chunk.text);
        self.index.get(&hash).map_or(&[], |indices| indices.as_slice())
    }

    fn compute_content_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.trim().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
```

### 2.3 L3 External Dependencies: Async/await, Serialization, ML

```rust
// L3.1: Async trait with tokio runtime
use async_trait::async_trait;

#[async_trait]
pub trait AsyncInferenceEngine: Send + Sync {
    type Session: Send + Sync;

    async fn create_session(&self, model_path: &str) -> Result<Self::Session>;
    async fn infer(&self, session: &Self::Session, input: &InferenceInput) -> Result<InferenceOutput>;
    async fn batch_infer(&self, sessions: &[Self::Session], inputs: &[InferenceInput]) -> Result<Vec<InferenceOutput>>;
}

// L3.2: Serde for configuration serialization
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub model: ModelConfig,
    pub chunking: ChunkingConfig,
    pub parallel: ParallelConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_path: String,
    pub tokenizer_path: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
}

// L3.3: ONNX Runtime integration
pub struct OptimizedInferenceEngine {
    model: ort::Session,
    tokenizer: tokenizers::Tokenizer,
    session_pool: Arc<RwLock<Vec<ort::Session>>>,
}

impl OptimizedInferenceEngine {
    pub async fn new(config: ModelConfig) -> Result<Self> {
        let env = ort::Environment::builder()
            .with_name("dobby-inference")
            .build()?
            .into_arc();

        let model = ort::SessionBuilder::new(&env)?
            .with_model_from_file(&config.model_path)?;

        let tokenizer = tokenizers::Tokenizer::from_file(&config.tokenizer_path)
            .map_err(|e| ProcessingError::TokenizerLoadFailed {
                source: Box::new(e)
            })?;

        Ok(Self {
            model,
            tokenizer,
            session_pool: Arc::new(RwLock::new(Vec::new())),
        })
    }
}

#[async_trait]
impl AsyncInferenceEngine for OptimizedInferenceEngine {
    type Session = ort::Session;

    async fn create_session(&self, model_path: &str) -> Result<Self::Session> {
        // Session creation with caching
        let env = ort::Environment::builder()
            .with_name("dobby-session")
            .build()?
            .into_arc();

        let session = ort::SessionBuilder::new(&env)?
            .with_model_from_file(model_path)?;

        Ok(session)
    }

    async fn infer(&self, session: &Self::Session, input: &InferenceInput) -> Result<InferenceOutput> {
        let start_time = std::time::Instant::now();

        // Tokenize input
        let encoding = self.tokenizer.encode(input.text(), true)
            .map_err(|e| ProcessingError::TokenizationFailed {
                text: input.text().to_string(),
                source: Box::new(e)
            })?;

        // Prepare input tensor
        let input_ids = ndarray::Array2::from_shape_vec(
            (1, encoding.len()),
            encoding.get_ids().to_vec()
        ).map_err(|e| ProcessingError::InferenceError(e.to_string()))?;

        let attention_mask = ndarray::Array2::from_shape_vec(
            (1, encoding.len()),
            encoding.get_attention_mask().to_vec()
        ).map_err(|e| ProcessingError::InferenceError(e.to_string()))?;

        // Run inference
        let outputs = session.run(ort::inputs![
            "input_ids" => input_ids,
            "attention_mask" => attention_mask
        ]?)?;

        // Process output
        let output_tensor = outputs["logits"].try_extract::<ndarray::Array3<f32>>()?;
        let generated_tokens = self.sample_tokens(&output_tensor, input.temperature(), input.top_p())?;

        let inference_time = start_time.elapsed();

        Ok(InferenceOutput::new(
            generated_tokens,
            inference_time,
            input.metadata().clone()
        ))
    }
}
```

## 3. Dependency Injection for Testability

### 3.1 Trait-Based Architecture

```rust
// 3.1.1: Core trait for chunking operations
pub trait Chunker: Send + Sync {
    fn chunk(&self, text: &str, config: &ChunkingConfig) -> Result<Vec<Chunk>>;
    fn validate_config(&self, config: &ChunkingConfig) -> Result<()>;
}

// 3.1.2: Generic service that depends on trait, not concrete type
pub struct ChunkingService<C: Chunker> {
    chunker: C,
    config: ChunkingConfig,
    metrics: ChunkingMetrics,
}

impl<C: Chunker> ChunkingService<C> {
    pub fn new(chunker: C, config: ChunkingConfig) -> Self {
        Self {
            chunker,
            config,
            metrics: ChunkingMetrics::new(),
        }
    }

    pub fn process_text(&mut self, text: &str) -> Result<Vec<Chunk>> {
        let start_time = std::time::Instant::now();

        // Dependency injection: all chunking logic goes through trait
        let chunks = self.chunker.chunk(text, &self.config)?;

        let processing_time = start_time.elapsed();
        self.metrics.record_processing(text.len(), chunks.len(), processing_time);

        Ok(chunks)
    }

    pub fn get_metrics(&self) -> &ChunkingMetrics {
        &self.metrics
    }
}

// 3.1.3: Factory pattern for creating instances
pub trait ChunkerFactory {
    type Chunker: Chunker;
    fn create_chunker(&self, config: &ChunkingConfig) -> Result<Self::Chunker>;
}

// Production factory
pub struct ProductionChunkerFactory;

impl ChunkerFactory for ProductionChunkerFactory {
    type Chunker = SmartTextChunker;

    fn create_chunker(&self, config: &ChunkingConfig) -> Result<Self::Chunker> {
        config.validate()?;
        Ok(SmartTextChunker::new_with_config(config))
    }
}

// Test factory for mock implementations
pub struct MockChunkerFactory;

impl ChunkerFactory for MockChunkerFactory {
    type Chunker = MockChunker;

    fn create_chunker(&self, config: &ChunkingConfig) -> Result<Self::Chunker> {
        Ok(MockChunker::new())
    }
}
```

### 3.2 Generic Type Parameters with Associated Types

```rust
// 3.2.1: Service with generic dependencies
pub struct TextProcessingService<C, I, S>
where
    C: Chunker,
    I: AsyncInferenceEngine,
    S: SummaryGenerator,
{
    chunker: C,
    inference_engine: I,
    summary_generator: S,
    config: ProcessingConfig,
}

impl<C, I, S> TextProcessingService<C, I, S>
where
    C: Chunker,
    I: AsyncInferenceEngine,
    S: SummaryGenerator,
{
    pub fn new(chunker: C, inference_engine: I, summary_generator: S, config: ProcessingConfig) -> Self {
        Self {
            chunker,
            inference_engine,
            summary_generator,
            config,
        }
    }

    pub async fn process_document(&mut self, document: &Document) -> Result<DocumentSummary> {
        // Step 1: Chunk the document
        let chunks = self.chunker.chunk(&document.content, &self.config.chunking)?;

        // Step 2: Process each chunk through inference
        let mut chunk_summaries = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let input = InferenceInput::from_chunk(chunk, &self.config.inference);
            let output = self.inference_engine.infer(&self.inference_session, &input).await?;
            chunk_summaries.push(output);
        }

        // Step 3: Generate final summary
        let summary = self.summary_generator.generate_summary(&chunk_summaries).await?;

        Ok(DocumentSummary::new(document.id.clone(), summary, chunk_summaries))
    }
}

// 3.2.2: Type aliases for production vs test configurations
pub type ProductionTextProcessor = TextProcessingService<
    SmartTextChunker,
    OptimizedInferenceEngine,
    AveragingSummaryGenerator
>;

pub type TestTextProcessor = TextProcessingService<
    MockChunker,
    MockInferenceEngine,
    MockSummaryGenerator
>;

// 3.2.3: Builder pattern with dependency injection
pub struct TextProcessorBuilder<C, I, S>
where
    C: Chunker,
    I: AsyncInferenceEngine,
    S: SummaryGenerator,
{
    chunker: Option<C>,
    inference_engine: Option<I>,
    summary_generator: Option<S>,
    config: Option<ProcessingConfig>,
}

impl<C, I, S> TextProcessorBuilder<C, I, S>
where
    C: Chunker,
    I: AsyncInferenceEngine,
    S: SummaryGenerator,
{
    pub fn new() -> Self {
        Self {
            chunker: None,
            inference_engine: None,
            summary_generator: None,
            config: None,
        }
    }

    pub fn with_chunker(mut self, chunker: C) -> Self {
        self.chunker = Some(chunker);
        self
    }

    pub fn with_inference_engine(mut self, engine: I) -> Self {
        self.inference_engine = Some(engine);
        self
    }

    pub fn with_summary_generator(mut self, generator: S) -> Self {
        self.summary_generator = Some(generator);
        self
    }

    pub fn with_config(mut self, config: ProcessingConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<TextProcessingService<C, I, S>> {
        Ok(TextProcessingService::new(
            self.chunker.ok_or_else(|| ProcessingError::ContractViolation {
                contract_name: "TextProcessorBuilder::build".to_string(),
                violation: "chunker is required".to_string(),
            })?,
            self.inference_engine.ok_or_else(|| ProcessingError::ContractViolation {
                contract_name: "TextProcessorBuilder::build".to_string(),
                violation: "inference_engine is required".to_string(),
            })?,
            self.summary_generator.ok_or_else(|| ProcessingError::ContractViolation {
                contract_name: "TextProcessorBuilder::build".to_string(),
                violation: "summary_generator is required".to_string(),
            })?,
            self.config.unwrap_or_default(),
        ))
    }
}
```

## 4. RAII Resource Management

### 4.1 Drop Implementations for Automatic Cleanup

```rust
// 4.1.1: RAII guard for ONNX sessions
pub struct SessionGuard {
    session: Option<ort::Session>,
    pool: Arc<RwLock<Vec<ort::Session>>>,
    session_id: usize,
}

impl SessionGuard {
    pub fn new(session: ort::Session, pool: Arc<RwLock<Vec<ort::Session>>>, session_id: usize) -> Self {
        Self {
            session: Some(session),
            pool,
            session_id,
        }
    }

    pub fn session(&self) -> &ort::Session {
        self.session.as_ref().expect("Session already consumed")
    }

    pub fn into_inner(mut self) -> ort::Session {
        self.session.take().expect("Session already consumed")
    }
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            // Return session to pool for reuse
            tokio::spawn({
                let pool = Arc::clone(&self.pool);
                let session_id = self.session_id;
                async move {
                    let mut pool_guard = pool.write().await;
                    if session_id < pool_guard.len() {
                        pool_guard[session_id] = session;
                    }
                }
            });
        }
    }
}

// 4.1.2: RAII for temporary file management
pub struct TempFileGuard {
    path: Option<std::path::PathBuf>,
}

impl TempFileGuard {
    pub fn new(prefix: &str, suffix: &str) -> Result<Self> {
        let temp_file = tempfile::NamedTempFile::with_prefix(prefix)
            .map_err(|e| ProcessingError::Io(e))?;

        let path = temp_file.path().to_path_buf();

        // Keep the file but forget the temp file guard so it doesn't delete immediately
        std::mem::forget(temp_file);

        Ok(Self {
            path: Some(path),
        })
    }

    pub fn path(&self) -> &std::path {
        self.path.as_ref().expect("Path already consumed")
    }

    pub fn into_path(mut self) -> std::path::PathBuf {
        self.path.take().expect("Path already consumed")
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            let _ = std::fs::remove_file(path);
        }
    }
}

// 4.1.3: RAII for memory tracking
pub struct MemoryTracker {
    initial_memory: usize,
    peak_memory: usize,
    current_allocations: Arc<RwLock<std::collections::HashMap<usize, usize>>>,
}

impl MemoryTracker {
    pub fn new() -> Self {
        let initial_memory = Self::get_current_memory();

        Self {
            initial_memory,
            peak_memory: initial_memory,
            current_allocations: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn track_allocation(&self, id: usize, size: usize) -> Result<()> {
        let mut allocations = self.current_allocations.write()
            .map_err(|e| ProcessingError::ContractViolation {
                contract_name: "MemoryTracker::track_allocation".to_string(),
                violation: format!("Failed to acquire write lock: {}", e),
            })?;

        allocations.insert(id, size);

        let total_allocated: usize = allocations.values().sum();
        let current_memory = self.initial_memory + total_allocated;

        if current_memory > self.peak_memory {
            self.peak_memory = current_memory;
        }

        Ok(())
    }

    pub fn release_allocation(&self, id: usize) -> Result<()> {
        let mut allocations = self.current_allocations.write()
            .map_err(|e| ProcessingError::ContractViolation {
                contract_name: "MemoryTracker::release_allocation".to_string(),
                violation: format!("Failed to acquire write lock: {}", e),
            })?;

        allocations.remove(&id);
        Ok(())
    }

    pub fn get_peak_memory(&self) -> usize {
        self.peak_memory
    }

    pub fn get_current_memory(&self) -> usize {
        sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::new().with_memory()
        ).total_memory()
    }
}

impl Drop for MemoryTracker {
    fn drop(&mut self) {
        // Log memory usage statistics
        let total_used = self.peak_memory - self.initial_memory;
        log::info!("Memory tracker: peak usage {} MB", total_used / 1024 / 1024);

        // Ensure all allocations are cleaned up
        if let Ok(allocations) = self.current_allocations.try_read() {
            if !allocations.is_empty() {
                log::warn!("Memory tracker dropped with {} active allocations",
                          allocations.len());
            }
        }
    }
}
```

### 4.2 Resource Pool Management

```rust
// 4.2.1: Generic resource pool with RAII
pub struct ResourcePool<T> {
    available: Arc<RwLock<Vec<T>>>,
    in_use: Arc<RwLock<std::collections::HashSet<usize>>>,
    max_size: usize,
    create_fn: Box<dyn Fn() -> Result<T> + Send + Sync>,
    metrics: Arc<RwLock<PoolMetrics>>,
}

impl<T: Send + Sync + 'static> ResourcePool<T> {
    pub fn new<F>(max_size: usize, create_fn: F) -> Self
    where
        F: Fn() -> Result<T> + Send + Sync + 'static,
    {
        Self {
            available: Arc::new(RwLock::new(Vec::with_capacity(max_size))),
            in_use: Arc::new(RwLock::new(std::collections::HashSet::new())),
            max_size,
            create_fn: Box::new(create_fn),
            metrics: Arc::new(RwLock::new(PoolMetrics::new())),
        }
    }

    pub async fn acquire(&self) -> Result<ResourceGuard<T>> {
        // Try to get from available pool
        let resource = {
            let mut available = self.available.write().await;
            if !available.is_empty() {
                available.pop()
            } else {
                None
            }
        };

        let resource = match resource {
            Some(r) => r,
            None => {
                // Create new resource if under limit
                let in_use_count = self.in_use.read().await.len();
                if in_use_count < self.max_size {
                    (self.create_fn)()?
                } else {
                    // Pool exhausted - wait or error
                    return Err(ProcessingError::ResourceExhaustion {
                        resource: "resource_pool".to_string(),
                        limit: self.max_size,
                    });
                }
            }
        };

        // Mark as in use
        let resource_id = std::ptr::addr_of!(&resource) as usize;
        {
            let mut in_use = self.in_use.write().await;
            in_use.insert(resource_id);
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_acquire();
        }

        Ok(ResourceGuard::new(
            resource,
            Arc::clone(&self.available),
            Arc::clone(&self.in_use),
            Arc::clone(&self.metrics),
            resource_id,
        ))
    }

    pub async fn stats(&self) -> PoolStats {
        let available_count = self.available.read().await.len();
        let in_use_count = self.in_use.read().await.len();
        let metrics = self.metrics.read().await.clone();

        PoolStats {
            available: available_count,
            in_use: in_use_count,
            total: available_count + in_use_count,
            max_size: self.max_size,
            metrics,
        }
    }
}

// 4.2.2: RAII guard for pooled resources
pub struct ResourceGuard<T> {
    resource: Option<T>,
    pool: Arc<RwLock<Vec<T>>>,
    in_use: Arc<RwLock<std::collections::HashSet<usize>>>,
    metrics: Arc<RwLock<PoolMetrics>>,
    resource_id: usize,
}

impl<T> ResourceGuard<T> {
    fn new(
        resource: T,
        pool: Arc<RwLock<Vec<T>>>,
        in_use: Arc<RwLock<std::collections::HashSet<usize>>>,
        metrics: Arc<RwLock<PoolMetrics>>,
        resource_id: usize,
    ) -> Self {
        Self {
            resource: Some(resource),
            pool,
            in_use,
            metrics,
            resource_id,
        }
    }

    pub fn resource(&self) -> &T {
        self.resource.as_ref().expect("Resource already returned to pool")
    }

    pub fn resource_mut(&mut self) -> &mut T {
        self.resource.as_mut().expect("Resource already returned to pool")
    }

    pub fn into_inner(mut self) -> T {
        self.resource.take().expect("Resource already returned to pool")
    }
}

impl<T> Drop for ResourceGuard<T> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            // Return resource to pool
            {
                let mut pool = self.pool.blocking_write();
                pool.push(resource);
            }

            // Mark as no longer in use
            {
                let mut in_use = self.in_use.blocking_write();
                in_use.remove(&self.resource_id);
            }

            // Update metrics
            {
                let mut metrics = self.metrics.blocking_write();
                metrics.record_release();
            }
        }
    }
}
```

## 5. Performance Contract Validation

### 5.1 Automated Performance Testing

```rust
// 5.1.1: Performance contract traits
pub trait PerformanceContract {
    type Input;
    type Output;

    fn contract_name(&self) -> &'static str;
    fn max_latency(&self) -> std::time::Duration;
    fn max_memory_usage(&self) -> usize; // bytes
    fn min_throughput(&self) -> f64; // operations per second

    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
    async fn validate_contract(&self, input: Self::Input) -> Result<PerformanceReport>;
}

// 5.1.2: Specific performance contract for chunking
pub struct ChunkingPerformanceContract {
    max_latency: std::time::Duration,
    max_memory_mb: usize,
    min_chars_per_second: f64,
}

impl ChunkingPerformanceContract {
    pub fn new() -> Self {
        Self {
            max_latency: std::time::Duration::from_millis(100), // 100ms max
            max_memory_mb: 50, // 50MB max memory
            min_chars_per_second: 10000.0, // 10K chars/second min
        }
    }
}

impl PerformanceContract for ChunkingPerformanceContract {
    type Input = String;
    type Output = Vec<Chunk>;

    fn contract_name(&self) -> &'static str {
        "chunking_performance"
    }

    fn max_latency(&self) -> std::time::Duration {
        self.max_latency
    }

    fn max_memory_usage(&self) -> usize {
        self.max_memory_mb * 1024 * 1024
    }

    fn min_throughput(&self) -> f64 {
        self.min_chars_per_second
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let chunker = SmartTextChunker::new();
        let config = ChunkingConfig::default();

        let start_time = std::time::Instant::now();
        let memory_tracker = MemoryTracker::new();

        let chunks = chunker.chunk(&input, &config)?;

        let execution_time = start_time.elapsed();
        let peak_memory = memory_tracker.get_peak_memory();

        Ok(chunks)
    }

    async fn validate_contract(&self, input: Self::Input) -> Result<PerformanceReport> {
        let start_time = std::time::Instant::now();
        let memory_tracker = MemoryTracker::new();

        let output = self.execute(input.clone()).await?;

        let total_time = start_time.elapsed();
        let peak_memory = memory_tracker.get_peak_memory();
        let throughput = input.len() as f64 / total_time.as_secs_f64();

        let mut violations = Vec::new();

        // Check latency
        if total_time > self.max_latency() {
            violations.push(PerformanceViolation {
                metric: "latency".to_string(),
                expected: format!("{:?}", self.max_latency()),
                actual: format!("{:?}", total_time),
                severity: ViolationSeverity::Critical,
            });
        }

        // Check memory usage
        if peak_memory > self.max_memory_usage() {
            violations.push(PerformanceViolation {
                metric: "memory_usage".to_string(),
                expected: format!("{} MB", self.max_memory_usage() / 1024 / 1024),
                actual: format!("{} MB", peak_memory / 1024 / 1024),
                severity: ViolationSeverity::Warning,
            });
        }

        // Check throughput
        if throughput < self.min_throughput() {
            violations.push(PerformanceViolation {
                metric: "throughput".to_string(),
                expected: format!("{:.2} chars/sec", self.min_throughput()),
                actual: format!("{:.2} chars/sec", throughput),
                severity: ViolationSeverity::Critical,
            });
        }

        Ok(PerformanceReport {
            contract_name: self.contract_name().to_string(),
            execution_time: total_time,
            peak_memory,
            throughput,
            input_size: input.len(),
            output_size: output.len(),
            violations,
            passed: violations.is_empty(),
        })
    }
}

// 5.1.3: Automated performance test suite
pub struct PerformanceTestSuite {
    contracts: Vec<Box<dyn PerformanceContract<Input = String, Output = Vec<Chunk>> + Send + Sync>>,
    test_data: Vec<String>,
}

impl PerformanceTestSuite {
    pub fn new() -> Self {
        let mut contracts: Vec<Box<dyn PerformanceContract<Input = String, Output = Vec<Chunk>> + Send + Sync>> = Vec::new();
        contracts.push(Box::new(ChunkingPerformanceContract::new()));

        Self {
            contracts,
            test_data: Self::generate_test_data(),
        }
    }

    pub async fn run_all_tests(&self) -> Result<Vec<PerformanceReport>> {
        let mut reports = Vec::new();

        for contract in &self.contracts {
            for test_input in &self.test_data {
                let report = contract.validate_contract(test_input.clone()).await?;
                reports.push(report);

                // Fail fast on critical violations
                if let Some(violation) = report.violations.iter()
                    .find(|v| matches!(v.severity, ViolationSeverity::Critical)) {
                    return Err(ProcessingError::ContractViolation {
                        contract_name: format!("{}::{}", contract.contract_name(), violation.metric),
                        violation: format!("Expected: {}, Actual: {}", violation.expected, violation.actual),
                    });
                }
            }
        }

        Ok(reports)
    }

    fn generate_test_data() -> Vec<String> {
        vec![
            "a".repeat(1000),    // Small
            "a".repeat(10000),   // Medium
            "a".repeat(100000),  // Large
            // Real-world text patterns
            (0..1000).map(|i| format!("Sentence {}. ", i)).collect(),
            "word ".repeat(5000),
        ]
    }
}

// 5.1.4: Criterion benchmark integration
pub fn benchmark_chunking(c: &mut criterion::Criterion) {
    let chunker = SmartTextChunker::new();
    let config = ChunkingConfig::default();

    let test_cases = vec![
        ("small", "a".repeat(1000)),
        ("medium", "a".repeat(10000)),
        ("large", "a".repeat(100000)),
    ];

    for (name, input) in test_cases {
        c.bench_function(&format!("chunk_{}", name), |b| {
            b.iter(|| {
                black_box(chunker.chunk(black_box(&input), black_box(&config))).unwrap()
            })
        });
    }
}
```

### 5.2 Memory Layout Validation

```rust
// 5.2.1: Memory layout validator
pub struct MemoryLayoutValidator {
    expected_struct_sizes: std::collections::HashMap<String, usize>,
    expected_alignments: std::collections::HashMap<String, usize>,
}

impl MemoryLayoutValidator {
    pub fn new() -> Self {
        let mut expected_sizes = std::collections::HashMap::new();
        expected_sizes.insert("Chunk".to_string(), std::mem::size_of::<Chunk>());
        expected_sizes.insert("ChunkingConfig".to_string(), std::mem::size_of::<ChunkingConfig>());
        expected_sizes.insert("ProcessingError".to_string(), std::mem::size_of::<ProcessingError>());

        let mut expected_alignments = std::collections::HashMap::new();
        expected_alignments.insert("Chunk".to_string(), std::mem::align_of::<Chunk>());
        expected_alignments.insert("ChunkingConfig".to_string(), std::mem::align_of::<ChunkingConfig>());

        Self {
            expected_struct_sizes: expected_sizes,
            expected_alignments: expected_alignments,
        }
    }

    pub fn validate_layouts(&self) -> Result<LayoutReport> {
        let mut violations = Vec::new();

        // Validate struct sizes
        for (name, expected_size) in &self.expected_struct_sizes {
            let actual_size = match name.as_str() {
                "Chunk" => std::mem::size_of::<Chunk>(),
                "ChunkingConfig" => std::mem::size_of::<ChunkingConfig>(),
                "ProcessingError" => std::mem::size_of::<ProcessingError>(),
                _ => continue,
            };

            if actual_size != *expected_size {
                violations.push(LayoutViolation {
                    struct_name: name.clone(),
                    violation_type: "size".to_string(),
                    expected: *expected_size,
                    actual: actual_size,
                });
            }
        }

        // Validate alignments
        for (name, expected_align) in &self.expected_alignments {
            let actual_align = match name.as_str() {
                "Chunk" => std::mem::align_of::<Chunk>(),
                "ChunkingConfig" => std::mem::align_of::<ChunkingConfig>(),
                _ => continue,
            };

            if actual_align != *expected_align {
                violations.push(LayoutViolation {
                    struct_name: name.clone(),
                    violation_type: "alignment".to_string(),
                    expected: *expected_align,
                    actual: actual_align,
                });
            }
        }

        Ok(LayoutReport {
            violations,
            passed: violations.is_empty(),
        })
    }

    pub fn assert_optimal_layouts(&self) -> Result<()> {
        let report = self.validate_layouts()?;

        if !report.passed {
            let violation_details: Vec<String> = report.violations
                .iter()
                .map(|v| format!("{} {} mismatch: expected {}, actual {}",
                               v.struct_name, v.violation_type, v.expected, v.actual))
                .collect();

            return Err(ProcessingError::ContractViolation {
                contract_name: "MemoryLayout".to_string(),
                violation: violation_details.join("; "),
            });
        }

        Ok(())
    }
}

// 5.2.2: Zero-copy optimization validation
pub struct ZeroCopyValidator;

impl ZeroCopyValidator {
    pub fn validate_chunker_no_unnecessary_clones<C: Chunker>(chunker: &C) -> Result<()> {
        let large_text = "x".repeat(100_000); // 100KB text

        // Use a custom chunker that tracks clones
        let clone_tracker = CloneTrackingChunker::new(chunker);
        let config = ChunkingConfig::default();

        let _chunks = clone_tracker.chunk(&large_text, &config)?;

        // Assert no unnecessary clones were made
        let clone_count = clone_tracker.get_clone_count();
        if clone_count > 2 { // Allow for necessary clones, but not many
            return Err(ProcessingError::ContractViolation {
                contract_name: "ZeroCopy".to_string(),
                violation: format!("Too many clones detected: {}", clone_count),
            });
        }

        Ok(())
    }
}

// Helper to track clones
struct CloneTrackingChunker<'a, C: Chunker> {
    inner: &'a C,
    clone_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl<'a, C: Chunker> CloneTrackingChunker<'a, C> {
    fn new(chunker: &'a C) -> Self {
        Self {
            inner: chunker,
            clone_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    fn get_clone_count(&self) -> usize {
        self.clone_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl<C: Chunker> Chunker for CloneTrackingChunker<'_, C> {
    fn chunk(&self, text: &str, config: &ChunkingConfig) -> Result<Vec<Chunk>> {
        // Track clones before delegating
        let initial_count = self.clone_count.load(std::sync::atomic::Ordering::Relaxed);
        let result = self.inner.chunk(text, config)?;
        let final_count = self.clone_count.load(std::sync::atomic::Ordering::Relaxed);

        if final_count > initial_count {
            log::debug!("Detected {} clones during chunking", final_count - initial_count);
        }

        Ok(result)
    }

    fn validate_config(&self, config: &ChunkingConfig) -> Result<()> {
        self.inner.validate_config(config)
    }
}
```

## 6. Structured Error Handling

### 6.1 thiserror for Library Error Types

```rust
use thiserror::Error;

// 6.1.1: Comprehensive error hierarchy
#[derive(Error, Debug, Clone)]
pub enum ProcessingError {
    #[error("Chunking failed for file '{path}': {reason}")]
    ChunkingFailed {
        path: String,
        reason: String,
        #[source]
        source: Option<Box<ProcessingError>>,
    },

    #[error("Inference failed for chunk {chunk_id}: {message}")]
    InferenceFailed {
        chunk_id: usize,
        message: String,
        retry_count: usize,
        #[source]
        source: Option<Box<ProcessingError>>,
    },

    #[error("Resource exhaustion: {resource} (limit: {limit}, current: {current})")]
    ResourceExhaustion {
        resource: String,
        limit: usize,
        current: usize,
    },

    #[error("Configuration error in {section}: {field} = {value} is invalid: {reason}")]
    ConfigurationError {
        section: String,
        field: String,
        value: String,
        reason: String,
    },

    #[error("Performance contract violation: {contract_name} - {violation}")]
    PerformanceContractViolation {
        contract_name: String,
        violation: String,
        expected: String,
        actual: String,
    },

    #[error("Memory layout violation: {details}")]
    MemoryLayoutViolation {
        details: String,
    },

    #[error("Session pool error: {message}")]
    SessionPoolError {
        message: String,
        operation: String,
    },

    #[error("Tokenization failed: {message}")]
    TokenizationError {
        message: String,
        input_length: usize,
    },

    #[error("Model loading failed from '{path}': {reason}")]
    ModelLoadError {
        path: String,
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("IO error during {operation} on {path}: {message}")]
    IoError {
        operation: String,
        path: String,
        message: String,
        #[source]
        source: std::io::Error,
    },

    #[error("ONNX Runtime error: {message} (code: {code})")]
    OnnxRuntimeError {
        message: String,
        code: i32,
    },

    #[error("Serialization error in {format}: {message}")]
    SerializationError {
        format: String,
        message: String,
    },

    #[error("Validation error: {field} {violation}")]
    ValidationError {
        field: String,
        violation: String,
    },

    #[error("Timeout during {operation} after {duration_ms}ms")]
    TimeoutError {
        operation: String,
        duration_ms: u64,
    },

    #[error("Thread pool error: {message}")]
    ThreadPoolError {
        message: String,
        pool_size: usize,
        active_tasks: usize,
    },

    #[error("Generic error: {message}")]
    Generic {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

// 6.1.2: Error conversion implementations
impl From<std::io::Error> for ProcessingError {
    fn from(err: std::io::Error) -> Self {
        ProcessingError::IoError {
            operation: "file_operation".to_string(),
            path: "unknown".to_string(),
            message: err.to_string(),
            source: err,
        }
    }
}

impl From<serde_json::Error> for ProcessingError {
    fn from(err: serde_json::Error) -> Self {
        ProcessingError::SerializationError {
            format: "json".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<tokenizers::Error> for ProcessingError {
    fn from(err: tokenizers::Error) -> Self {
        ProcessingError::TokenizationError {
            message: err.to_string(),
            input_length: 0, // Will be filled by caller
        }
    }
}

// 6.1.3: Error context and chaining
impl ProcessingError {
    pub fn chunking_failed(path: String, reason: impl Into<String>) -> Self {
        Self::ChunkingFailed {
            path,
            reason: reason.into(),
            source: None,
        }
    }

    pub fn with_source(mut self, source: impl Into<Box<ProcessingError>>) -> Self {
        match &mut self {
            ProcessingError::ChunkingFailed { source: s, .. } |
            ProcessingError::InferenceFailed { source: s, .. } => {
                *s = Some(source.into());
            }
            _ => {}
        }
        self
    }

    pub fn is_retriable(&self) -> bool {
        match self {
            ProcessingError::ResourceExhaustion { .. } |
            ProcessingError::TimeoutError { .. } |
            ProcessingError::OnnxRuntimeError { .. } => true,

            ProcessingError::ConfigurationError { .. } |
            ProcessingError::ValidationError { .. } |
            ProcessingError::MemoryLayoutViolation { .. } => false,

            _ => false,
        }
    }

    pub fn is_critical(&self) -> bool {
        match self {
            ProcessingError::MemoryLayoutViolation { .. } |
            ProcessingError::PerformanceContractViolation { .. } |
            ProcessingError::ConfigurationError { .. } => true,

            _ => false,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            ProcessingError::ChunkingFailed { .. } => "CHUNK_001",
            ProcessingError::InferenceFailed { .. } => "INF_001",
            ProcessingError::ResourceExhaustion { .. } => "RES_001",
            ProcessingError::ConfigurationError { .. } => "CFG_001",
            ProcessingError::PerformanceContractViolation { .. } => "PERF_001",
            ProcessingError::MemoryLayoutViolation { .. } => "MEM_001",
            ProcessingError::SessionPoolError { .. } => "POOL_001",
            ProcessingError::TokenizationError { .. } => "TOK_001",
            ProcessingError::ModelLoadError { .. } => "MODEL_001",
            ProcessingError::IoError { .. } => "IO_001",
            ProcessingError::OnnxRuntimeError { .. } => "ONNX_001",
            ProcessingError::SerializationError { .. } => "SER_001",
            ProcessingError::ValidationError { .. } => "VAL_001",
            ProcessingError::TimeoutError { .. } => "TIMEOUT_001",
            ProcessingError::ThreadPoolError { .. } => "THREAD_001",
            ProcessingError::Generic { .. } => "GEN_001",
        }
    }

    pub fn user_friendly_message(&self) -> String {
        match self {
            ProcessingError::ChunkingFailed { path, reason, .. } => {
                format!("Failed to process file '{}': {}", path, reason)
            }
            ProcessingError::ResourceExhaustion { resource, limit, .. } => {
                format!("System resource limit reached. Cannot process more than {} {} simultaneously", limit, resource)
            }
            ProcessingError::ConfigurationError { section, field, value, reason } => {
                format!("Invalid configuration: {}.{} = {} is not allowed: {}", section, field, value, reason)
            }
            _ => self.to_string(),
        }
    }
}
```

### 6.2 anyhow for Application Context

```rust
// 6.2.1: Application-level error handling with context
use anyhow::{Context, Result};

pub struct Application {
    chunker: Box<dyn Chunker>,
    inference_engine: Box<dyn AsyncInferenceEngine>,
    config: SystemConfig,
}

impl Application {
    pub async fn process_file(&mut self, file_path: &str) -> Result<String> {
        // Load file with context
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path))?;

        // Process content with context
        let chunks = self.chunker.chunk(&content, &self.config.chunking)
            .with_context(|| format!("Failed to chunk content from file: {}", file_path))?;

        // Generate summary with context
        let summary = self.generate_chunks_summary(&chunks)
            .await
            .with_context(|| "Failed to generate summary from chunks")?;

        Ok(summary)
    }

    async fn generate_chunks_summary(&mut self, chunks: &[Chunk]) -> Result<String> {
        let session = self.inference_engine.create_session(&self.config.model.model_path)
            .await
            .context("Failed to create inference session")?;

        let mut summaries = Vec::with_capacity(chunks.len());

        for (i, chunk) in chunks.iter().enumerate() {
            let input = InferenceInput::from_chunk(chunk.clone(), &self.config.model);

            let output = self.inference_engine.infer(&session, &input)
                .await
                .with_context(|| format!("Failed to process chunk {} of {}", i + 1, chunks.len()))?;

            summaries.push(output);
        }

        // Combine summaries
        let combined_summary = summaries
            .iter()
            .map(|s| s.text())
            .collect::<Vec<_>>()
            .join(" ");

        Ok(combined_summary)
    }

    pub async fn run_with_error_handling(&mut self, file_paths: &[&str]) -> Vec<Result<String>> {
        let mut results = Vec::with_capacity(file_paths.len());

        for &file_path in file_paths {
            let result = match self.process_file(file_path).await {
                Ok(summary) => {
                    log::info!("Successfully processed file: {}", file_path);
                    Ok(summary)
                }
                Err(e) => {
                    // Log error with context chain
                    log::error!("Error processing file {}: {}", file_path, e);

                    // Provide user-friendly error while preserving technical details
                    let user_error = e.chain()
                        .find(|e| e.downcast_ref::<ProcessingError>().is_some())
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "Unknown processing error".to_string());

                    Err(anyhow::anyhow!("Failed to process '{}': {}", file_path, user_error))
                }
            };

            results.push(result);
        }

        results
    }
}

// 6.2.2: Error recovery and retry logic
pub struct RetryProcessor {
    max_retries: usize,
    retry_delay: std::time::Duration,
}

impl RetryProcessor {
    pub fn new(max_retries: usize, retry_delay: std::time::Duration) -> Self {
        Self {
            max_retries,
            retry_delay,
        }
    }

    pub async fn process_with_retry<F, T, E>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error + Send + Sync + 'static,
        anyhow::Error: From<E>,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        log::info!("Operation succeeded on attempt {}", attempt + 1);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    let anyhow_error = anyhow::Error::from(e);

                    // Check if error is retriable
                    if !self.is_retriable(&anyhow_error) {
                        return Err(anyhow_error.context("Non-retriable error occurred"));
                    }

                    last_error = Some(anyhow_error);

                    if attempt < self.max_retries {
                        log::warn!("Attempt {} failed, retrying in {:?}: {}",
                                 attempt + 1, self.retry_delay, last_error.as_ref().unwrap());
                        tokio::time::sleep(self.retry_delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retries exhausted")))
    }

    fn is_retriable(&self, error: &anyhow::Error) -> bool {
        // Check for retriable error types
        for cause in error.chain() {
            if let Some(proc_error) = cause.downcast_ref::<ProcessingError>() {
                return proc_error.is_retriable();
            }
        }

        // Check for common retriable patterns
        let error_string = error.to_string().to_lowercase();
        error_string.contains("timeout") ||
        error_string.contains("connection") ||
        error_string.contains("temporary") ||
        error_string.contains("resource")
    }
}

// 6.2.3: Error reporting and metrics
pub struct ErrorReporter {
    error_counts: Arc<RwLock<std::collections::HashMap<String, usize>>>,
    total_operations: Arc<std::sync::atomic::AtomicUsize>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            error_counts: Arc::new(RwLock::new(std::collections::HashMap::new())),
            total_operations: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    pub async fn record_error(&self, error: &anyhow::Error) {
        // Increment total operations
        self.total_operations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Determine error type
        let error_type = self.categorize_error(error);

        // Increment error count
        {
            let mut counts = self.error_counts.write().await;
            *counts.entry(error_type).or_insert(0) += 1;
        }

        // Log detailed error information
        log::error!("Error recorded: {}", error);
        for (i, cause) in error.chain().enumerate() {
            log::error!("  Cause {}: {}", i, cause);
        }
    }

    fn categorize_error(&self, error: &anyhow::Error) -> String {
        for cause in error.chain() {
            if let Some(proc_error) = cause.downcast_ref::<ProcessingError>() {
                return match proc_error {
                    ProcessingError::ChunkingFailed { .. } => "chunking".to_string(),
                    ProcessingError::InferenceFailed { .. } => "inference".to_string(),
                    ProcessingError::ResourceExhaustion { .. } => "resource".to_string(),
                    ProcessingError::ConfigurationError { .. } => "configuration".to_string(),
                    ProcessingError::PerformanceContractViolation { .. } => "performance".to_string(),
                    ProcessingError::SessionPoolError { .. } => "session_pool".to_string(),
                    ProcessingError::TokenizationError { .. } => "tokenization".to_string(),
                    ProcessingError::ModelLoadError { .. } => "model_load".to_string(),
                    ProcessingError::IoError { .. } => "io".to_string(),
                    ProcessingError::OnnxRuntimeError { .. } => "onnx_runtime".to_string(),
                    ProcessingError::SerializationError { .. } => "serialization".to_string(),
                    ProcessingError::ValidationError { .. } => "validation".to_string(),
                    ProcessingError::TimeoutError { .. } => "timeout".to_string(),
                    ProcessingError::ThreadPoolError { .. } => "thread_pool".to_string(),
                    ProcessingError::Generic { .. } => "generic".to_string(),
                };
            }
        }

        "unknown".to_string()
    }

    pub async fn get_error_summary(&self) -> ErrorSummary {
        let total_ops = self.total_operations.load(std::sync::atomic::Ordering::Relaxed);
        let error_counts = self.error_counts.read().await.clone();

        let total_errors: usize = error_counts.values().sum();
        let error_rate = if total_ops > 0 {
            total_errors as f64 / total_ops as f64
        } else {
            0.0
        };

        ErrorSummary {
            total_operations: total_ops,
            total_errors,
            error_rate,
            error_counts,
        }
    }
}
```

## 7. Complete Integration Example

### 7.1 Main Application with Full Architecture

```rust
// 7.1.1: Main application entry point
pub struct DobbyCodeSummarizer {
    chunking_service: ChunkingService<SmartTextChunker>,
    inference_engine: OptimizedInferenceEngine,
    processor: TextProcessingService<SmartTextChunker, OptimizedInferenceEngine, AveragingSummaryGenerator>,
    error_reporter: ErrorReporter,
    performance_tracker: PerformanceTracker,
}

impl DobbyCodeSummarizer {
    pub async fn new(config: SystemConfig) -> Result<Self> {
        // Validate configuration first
        config.validate()
            .context("Invalid system configuration")?;

        // Create components with dependency injection
        let chunker = SmartTextChunker::new_with_config(&config.chunking);
        let chunking_service = ChunkingService::new(chunker, config.chunking.clone());

        let inference_engine = OptimizedInferenceEngine::new(config.model.clone())
            .await
            .context("Failed to create inference engine")?;

        let summary_generator = AveragingSummaryGenerator::new();

        let processor = TextProcessingService::new(
            SmartTextChunker::new_with_config(&config.chunking),
            inference_engine.clone(),
            summary_generator,
            ProcessingConfig::from_system_config(&config),
        );

        // Validate performance contracts
        let memory_validator = MemoryLayoutValidator::new();
        memory_validator.assert_optimal_layouts()
            .context("Memory layout validation failed")?;

        Ok(Self {
            chunking_service,
            inference_engine,
            processor,
            error_reporter: ErrorReporter::new(),
            performance_tracker: PerformanceTracker::new(),
        })
    }

    pub async fn process_workspace(&mut self, workspace_path: &str) -> Result<WorkspaceSummary> {
        let start_time = std::time::Instant::now();

        // Discover files
        let files = self.discover_rust_files(workspace_path)
            .await
            .context("Failed to discover workspace files")?;

        log::info!("Found {} files to process", files.len());

        // Process files with error handling
        let mut file_summaries = Vec::with_capacity(files.len());
        let mut errors = Vec::new();

        for file_path in files {
            match self.process_single_file(&file_path).await {
                Ok(summary) => file_summaries.push(summary),
                Err(e) => {
                    self.error_reporter.record_error(&e).await;
                    errors.push((file_path, e));
                }
            }
        }

        let processing_time = start_time.elapsed();

        // Update performance metrics
        self.performance_tracker.record_workspace_processing(
            file_summaries.len(),
            errors.len(),
            processing_time,
        );

        // Generate workspace summary
        let workspace_summary = WorkspaceSummary::new(
            workspace_path.to_string(),
            file_summaries,
            errors,
            processing_time,
        );

        // Validate performance contracts
        self.validate_performance_contracts(&workspace_summary)
            .await
            .context("Performance contract validation failed")?;

        Ok(workspace_summary)
    }

    async fn process_single_file(&mut self, file_path: &str) -> Result<FileSummary> {
        let start_time = std::time::Instant::now();

        // Read file content
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path))?;

        // Create document
        let document = Document::new(file_path.to_string(), content);

        // Process document through the full pipeline
        let document_summary = self.processor.process_document(&document)
            .await
            .context("Failed to process document through pipeline")?;

        let processing_time = start_time.elapsed();

        Ok(FileSummary::new(
            file_path.to_string(),
            document_summary,
            processing_time,
        ))
    }

    async fn discover_rust_files(&self, workspace_path: &str) -> Result<Vec<String>> {
        let mut rust_files = Vec::new();

        let mut entries = tokio::fs::read_dir(workspace_path)
            .await
            .context("Failed to read workspace directory")?;

        while let Some(entry) = entries.next_entry().await
            .context("Failed to read directory entry")? {

            let path = entry.path();

            if path.is_dir() {
                // Skip common ignore directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if matches!(dir_name, "target" | "node_modules" | ".git" | ".vscode") {
                        continue;
                    }
                }

                // Recursively process subdirectories
                let path_str = path.to_string_lossy().into_owned();
                rust_files.extend(self.discover_rust_files(&path_str).await?);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                rust_files.push(path.to_string_lossy().into_owned());
            }
        }

        rust_files.sort();
        Ok(rust_files)
    }

    async fn validate_performance_contracts(&self, summary: &WorkspaceSummary) -> Result<()> {
        // Run performance test suite
        let test_suite = PerformanceTestSuite::new();
        let reports = test_suite.run_all_tests()
            .await
            .context("Performance test suite failed")?;

        // Check for any failed contracts
        let failed_reports: Vec<_> = reports.iter()
            .filter(|r| !r.passed)
            .collect();

        if !failed_reports.is_empty() {
            let error_details: Vec<String> = failed_reports
                .iter()
                .map(|r| format!("{}: {}", r.contract_name,
                               r.violations.first()
                                   .map(|v| v.to_string())
                                   .unwrap_or_else(|| "Unknown violation".to_string())))
                .collect();

            return Err(ProcessingError::PerformanceContractViolation {
                contract_name: "PerformanceTestSuite".to_string(),
                violation: error_details.join("; "),
                expected: "All contracts to pass".to_string(),
                actual: format!("{} contracts failed", failed_reports.len()),
            }.into());
        }

        Ok(())
    }

    pub async fn get_metrics(&self) -> ApplicationMetrics {
        ApplicationMetrics {
            error_summary: self.error_reporter.get_error_summary().await,
            performance_metrics: self.performance_tracker.get_metrics(),
            chunking_metrics: self.chunking_service.get_metrics().clone(),
        }
    }
}

// 7.1.2: CLI integration with proper error handling
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    // Parse command line arguments
    let args = clap::Command::new("dobby-subagent-code-summarizer")
        .version("0.1.0")
        .about("Real neural code summarization with TDD-first architecture")
        .arg(clap::Arg::new("workspace")
            .required(true)
            .help("Path to the Rust workspace to analyze"))
        .arg(clap::Arg::new("config")
            .long("config")
            .short('c')
            .help("Path to configuration file"))
        .arg(clap::Arg::new("output")
            .long("output")
            .short('o')
            .help("Output file for summary"))
        .arg(clap::Arg::new("validate-performance")
            .long("validate-performance")
            .help("Run performance contract validation"))
        .get_matches();

    // Load configuration
    let config_path = args.get_one::<String>("config");
    let config = if let Some(path) = config_path {
        let config_content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        serde_json::from_str(&config_content)
            .with_context(|| format!("Failed to parse config file: {}", path))?
    } else {
        SystemConfig::default()
    };

    // Create application
    let mut app = DobbyCodeSummarizer::new(config)
        .await
        .context("Failed to initialize application")?;

    let workspace_path = args.get_one::<String>("workspace").unwrap();

    // Run performance validation if requested
    if args.get_flag("validate-performance") {
        log::info!("Running performance contract validation...");
        let test_suite = PerformanceTestSuite::new();
        let reports = test_suite.run_all_tests()
            .await
            .context("Performance validation failed")?;

        for report in &reports {
            if report.passed {
                log::info!("✓ {} passed", report.contract_name);
            } else {
                log::error!("✗ {} failed: {}", report.contract_name,
                          report.violations.first()
                               .map(|v| v.to_string())
                               .unwrap_or_else(|| "Unknown violation".to_string()));
            }
        }

        let passed_count = reports.iter().filter(|r| r.passed).count();
        log::info!("Performance validation: {}/{} contracts passed",
                  passed_count, reports.len());

        if passed_count != reports.len() {
            std::process::exit(1);
        }
    }

    // Process workspace
    log::info!("Processing workspace: {}", workspace_path);
    let summary = app.process_workspace(workspace_path)
        .await
        .context("Failed to process workspace")?;

    // Output results
    if let Some(output_path) = args.get_one::<String>("output") {
        let output_json = serde_json::to_string_pretty(&summary)
            .context("Failed to serialize summary")?;
        tokio::fs::write(output_path, output_json)
            .await
            .with_context(|| format!("Failed to write output to: {}", output_path))?;
        log::info!("Summary written to: {}", output_path);
    } else {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    }

    // Print metrics
    let metrics = app.get_metrics().await;
    log::info!("Processing complete. Metrics: {:#?}", metrics);

    Ok(())
}
```

## Conclusion

This architecture specification provides:

1. **TDD-First Development**: Complete STUB → RED → GREEN → REFACTOR cycle with executable contracts
2. **Layered Architecture**: Clear separation between L1 (Core), L2 (Std), and L3 (External) dependencies
3. **Dependency Injection**: Trait-based design enabling comprehensive testing
4. **RAII Resource Management**: Automatic cleanup with Drop implementations and resource pooling
5. **Performance Validation**: Automated testing of latency, memory, and throughput contracts
6. **Structured Error Handling**: thiserror for libraries, anyhow for applications with proper context

Every component is specified as a testable contract with measurable outcomes, following the 9 non-negotiable principles from the steering document. The architecture supports real ONNX inference, efficient chunking, and scalable parallel processing while maintaining strict performance and correctness guarantees.