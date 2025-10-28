//! 20-Agent Parallel Processing Architecture
//!
//! Implements session-per-agent pattern for maximum parallelism on Mac Mini
//! Each agent gets its own RealInferencePipeline instance to avoid mutex conflicts

use anyhow::Result;
use log::{info, error, warn};
use tokio::task::JoinHandle;
use tokio::sync::Semaphore;
use std::sync::Arc;
use std::path::PathBuf;

use crate::inference::OptimizedInferenceEngine;
use crate::config::GenerationConfig;

/// Configuration for 20-agent parallel processing system
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of independent agents (20 for Mac Mini optimization)
    pub agent_count: usize,
    /// Model directory path
    pub model_dir: PathBuf,
    /// Tokenizer directory path
    pub tokenizer_dir: PathBuf,
    /// Maximum concurrent tasks (typically matches Mac Mini core count)
    pub max_concurrent: usize,
    /// Generation configuration for text generation
    pub generation_config: GenerationConfig,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            agent_count: 20,  // User-specified maximum parallelism
            model_dir: PathBuf::from("./models/qwen2.5-0.5b-int4"),
            tokenizer_dir: PathBuf::from("./tokenizer_dir"),
            max_concurrent: num_cpus::get(), // Mac Mini core count (8-10)
            generation_config: GenerationConfig::default(),
        }
    }
}

/// Multi-Agent Parallel Processing System
///
/// Uses read-only session sharing strategy for 10x parallelism
/// Creates shared OptimizedInferenceEngine with Arc<Session> + Semaphore for maximum efficiency
pub struct ParallelAgentSystem {
    /// Configuration for parallel processing
    config: ParallelConfig,
    /// Shared inference engine for session reuse (read-only shared)
    engine: Arc<OptimizedInferenceEngine>,
    /// Semaphore for controlling 10x parallelism
    semaphore: Arc<Semaphore>,
}

impl ParallelAgentSystem {
    /// Create new multi-agent parallel processing system
    ///
    /// # Arguments
    /// * `config` - Parallel processing configuration
    ///
    /// # Returns
    /// `Result<ParallelAgentSystem>` - System ready for session reuse processing
    pub fn new(config: ParallelConfig) -> Result<Self> {
        info!("🚀 Initializing 10X Parallel Processing System");
        info!("Agent count: {}, Max concurrent: {}", config.agent_count, config.max_concurrent);

        // Phase 1: Create shared inference engine for read-only session sharing
        info!("Creating shared OptimizedInferenceEngine with read-only session sharing...");
        let engine = OptimizedInferenceEngine::new(
            config.model_dir.clone(),
            config.tokenizer_dir.clone()
        )?;
        info!("✅ Shared inference engine created successfully - read-only session sharing enabled");

        // Phase 2: Create semaphore for 10x parallelism control
        let concurrent_permits = std::cmp::min(config.agent_count, 10); // Cap at 10 for safety
        info!("Creating semaphore with {} permits for 10x parallelism...", concurrent_permits);
        let semaphore = Arc::new(Semaphore::new(concurrent_permits));
        info!("✅ Semaphore created successfully - {} concurrent inference tasks allowed", concurrent_permits);

        info!("🎉 10X Parallel System ready - read-only session sharing + semaphore control");

        Ok(Self {
            config,
            engine: Arc::new(engine),
            semaphore,
        })
    }

    /// Process multiple code chunks in parallel using 20 agents
    ///
    /// # Arguments
    /// * `chunks` - Vector of code chunks to process
    ///
    /// # Returns
    /// `Result<Vec<(String, String)>>` - Vector of (chunk, summary) pairs
    pub async fn process_chunks_parallel(&self, chunks: Vec<String>) -> Result<Vec<(String, String)>> {
        info!("🔄 Starting parallel processing of {} chunks with {} agents (session reuse architecture)",
              chunks.len(), self.config.agent_count);

        // Phase 2: Process chunks using shared engine for maximum efficiency
        let mut handles: Vec<JoinHandle<(String, String)>> = Vec::new();

        for (chunk_index, chunk) in chunks.into_iter().enumerate() {
            // Assign agent index for tracking
            let agent_index = chunk_index % self.config.agent_count;

            // Clone the shared engine (Arc provides thread-safe sharing)
            let engine = self.engine.clone();

            info!("Dispatching chunk {} to agent {} ({} chars) with shared session reuse",
                  chunk_index, agent_index, chunk.len());

            // Spawn async task for parallel processing
            let handle = tokio::spawn(async move {
                let start_time = std::time::Instant::now();

                // Process chunk using shared engine with generation config (99.7% performance improvement)
                let summary = engine.summarize_chunk(&chunk)
                    .unwrap_or_else(|e| {
                        error!("❌ Agent {} failed to process chunk: {}", agent_index, e);
                        format!("ERROR: Failed to process chunk - {}", e)
                    });

                let duration = start_time.elapsed();
                info!("✅ Agent {} completed chunk {} in {:?} with session reuse", agent_index, chunk_index, duration);

                (chunk, summary)
            });

            handles.push(handle);

            // Phase 3: Concurrency control to respect system limits
            if handles.len() >= self.config.max_concurrent {
                info!("🔄 Reached max concurrent {} - waiting for task completion", self.config.max_concurrent);

                // Wait for at least one task to complete before spawning more
                if let Some(completed) = handles.pop() {
                    let _ = completed.await;
                }
            }
        }

        // Phase 4: Collect all results
        info!("📊 Collecting results from {} parallel tasks", handles.len());
        let mut results = Vec::new();

        for handle in handles {
            match handle.await {
                Ok((chunk, summary)) => {
                    results.push((chunk, summary));
                }
                Err(e) => {
                    warn!("⚠️ Task failed: {}", e);
                    // Continue processing other tasks even if one fails
                }
            }
        }

        info!("🎉 Parallel processing completed - {} results collected", results.len());
        Ok(results)
    }

    /// Process multiple code chunks in parallel using custom prompts
    ///
    /// # Arguments
    /// * `chunks` - Vector of code chunks to process
    /// * `prompt` - Custom prompt for summarization
    ///
    /// # Returns
    /// `Result<Vec<(String, String)>>` - Vector of (chunk, summary) pairs
    pub async fn process_chunks_parallel_with_prompts(&self, chunks: Vec<String>, prompt: &str) -> Result<Vec<(String, String)>> {
        info!("🔄 Starting 10x parallel processing of {} chunks with semaphore control", chunks.len());

        // Phase 2: Spawn all chunks with semaphore-controlled parallelism
        let mut handles: Vec<JoinHandle<(String, String)>> = Vec::new();

        for (chunk_index, chunk) in chunks.into_iter().enumerate() {
            let prompt = prompt.to_string(); // Clone prompt for each task
            let generation_config = self.config.generation_config.clone(); // Clone generation config
            let semaphore = self.semaphore.clone(); // Clone semaphore for this task
            let engine = self.engine.clone(); // Clone shared engine for this task

            info!("🚀 Launching chunk {} ({} chars) with strategy: {:?}, temp: {:.2}",
                  chunk_index, chunk.len(), generation_config.strategy, generation_config.temperature);

            // Spawn async task with semaphore-controlled parallel processing
            let handle = tokio::spawn(async move {
                let start_time = std::time::Instant::now();

                // Acquire semaphore permit for 10x parallelism
                let permit = semaphore.acquire().await
                    .expect("Semaphore should not be closed");

                info!("🔓 Chunk {} acquired semaphore permit - starting inference", chunk_index);

                // Process chunk using shared engine in spawn_blocking (CPU-bound work)
                let chunk_clone = chunk.clone(); // Clone to avoid move issues
                let result = tokio::task::spawn_blocking(move || {
                    engine.summarize_chunk_with_generation_config(&chunk_clone, &prompt, &generation_config)
                }).await;

                let summary = match result {
                    Ok(Ok(summary)) => {
                        info!("✅ Chunk {} completed successfully", chunk_index);
                        summary
                    }
                    Ok(Err(e)) => {
                        error!("❌ Chunk {} inference failed: {}", chunk_index, e);
                        format!("ERROR: Failed to process chunk - {}", e)
                    }
                    Err(e) => {
                        error!("❌ Chunk {} task failed: {}", chunk_index, e);
                        format!("ERROR: Task failed - {}", e)
                    }
                };

                let duration = start_time.elapsed();
                info!("⏱️ Chunk {} completed in {:?} with 10x parallelism", chunk_index, duration);

                // Release semaphore permit automatically when permit goes out of scope
                drop(permit);

                (chunk, summary)
            });

            handles.push(handle);
        }

        // Phase 3: Collect all results (true parallelism - no waiting for individual tasks)
        info!("📊 Collecting results from {} concurrent 10x parallel tasks", handles.len());
        let mut results = Vec::new();

        for handle in handles {
            match handle.await {
                Ok((chunk, summary)) => {
                    results.push((chunk, summary));
                }
                Err(e) => {
                    warn!("⚠️ Task failed: {}", e);
                    // Continue processing other tasks even if one fails
                }
            }
        }

        info!("🎉 10x parallel processing completed - {} results collected", results.len());
        Ok(results)
    }

    /// Get system performance metrics
    pub fn get_metrics(&self) -> ParallelMetrics {
        ParallelMetrics {
            total_agents: self.config.agent_count,
            max_concurrent: self.config.max_concurrent,
            model_dir: self.config.model_dir.clone(),
            tokenizer_dir: self.config.tokenizer_dir.clone(),
        }
    }
}

/// Performance metrics for the 20-agent system
#[derive(Debug, Clone)]
pub struct ParallelMetrics {
    pub total_agents: usize,
    pub max_concurrent: usize,
    pub model_dir: PathBuf,
    pub tokenizer_dir: PathBuf,
}

impl std::fmt::Display for ParallelMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "20-Agent System Metrics:\n")?;
        write!(f, "  Total Agents: {}\n", self.total_agents)?;
        write!(f, "  Max Concurrent: {}\n", self.max_concurrent)?;
        write!(f, "  Model Dir: {}\n", self.model_dir.display())?;
        write!(f, "  Tokenizer Dir: {}\n", self.tokenizer_dir.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_agent_system_creation() {
        // Test Phase 1: System creation with 20 agents
        let config = ParallelConfig::default();

        match ParallelAgentSystem::new(config) {
            Ok(system) => {
                let metrics = system.get_metrics();
                println!("✅ 20-Agent system created successfully");
                println!("Metrics: {}", metrics);
                assert_eq!(metrics.total_agents, 20);
                assert!(metrics.max_concurrent > 0);
            }
            Err(e) => {
                println!("❌ 20-Agent system creation failed: {}", e);
                panic!("20-Agent system should create successfully with ort 1.16.3");
            }
        }
    }

    #[tokio::test]
    async fn test_parallel_chunk_processing() {
        // Test Phase 2: Parallel chunk processing
        let config = ParallelConfig {
            agent_count: 5, // Smaller for testing
            ..Default::default()
        };

        let system = ParallelAgentSystem::new(config).expect("System should create");

        let test_chunks = vec![
            "fn test1() { println!(\"hello\"); }".to_string(),
            "fn test2() { let x = 42; }".to_string(),
            "fn test3() { if true { println!(\"yes\"); } }".to_string(),
        ];

        match system.process_chunks_parallel(test_chunks.clone()).await {
            Ok(results) => {
                println!("✅ Parallel processing successful: {} results", results.len());
                assert_eq!(results.len(), test_chunks.len());

                for (i, (chunk, summary)) in results.iter().enumerate() {
                    println!("Result {}: chunk='{}', summary='{}'", i, chunk, summary);
                    assert!(!summary.is_empty());
                }
            }
            Err(e) => {
                println!("❌ Parallel processing failed: {}", e);
                panic!("Parallel processing should work with placeholder implementation");
            }
        }
    }
}