//! Real Neural Code Summarization Library
//!
//! Production-ready implementation with session reuse architecture
//! Real ONNX inference with Qwen2.5-0.5B-Instruct model

pub mod chunking;
pub mod inference;  // ort 1.16.3 real inference implementation
pub mod parallel_agents;  // 20-agent parallel processing architecture
pub mod config;
pub mod errors;

// Layer 1 Core Traits (TDD-First Architecture)
pub mod layer1;

// Re-export main components for parallel_summarizer
pub use chunking::{TextChunker, Chunk};
pub use config::SystemConfig;
pub use errors::{ProcessingError, Result};
pub use parallel_agents::{ParallelAgentSystem, ParallelConfig, ParallelMetrics};
pub use inference::{OptimizedInferenceEngine}; // Working session reuse architecture