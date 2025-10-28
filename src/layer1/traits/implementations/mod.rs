//! GREEN PHASE → REFACTOR PHASE: Trait Implementations
//!
//! This module contains trait implementations progressing from GREEN phase
//! minimal implementations to REFACTOR phase production-ready features.
//!
//! ## Implementation Strategy
//! - GREEN phase: Simple mock implementations with clear production barriers
//! - REFACTOR phase: Production-ready patterns with connection pooling, circuit breakers
//! - Proper async patterns with resource management and fault tolerance
//! - Error handling integration with hierarchical error types
//!
//! ## Module Structure
//! - database_simple: GREEN phase minimal working implementation
//! - production_database: REFACTOR phase production-ready implementation
//! - inference_engine: Production-ready InferenceEngine trait implementation
//! - pipeline_orchestrator: High-level orchestration combining database and inference

pub mod database_simple;  // Simple GREEN phase implementation
pub mod production_database;  // REFACTOR phase production implementation
pub mod inference_engine;  // Production-ready InferenceEngine implementation
pub mod pipeline_orchestrator;  // High-level pipeline orchestration

// Re-export implementations for convenience
pub use database_simple::*;
pub use production_database::*;
pub use inference_engine::{TraitInferenceEngine, TraitModelInfo, SessionPool};
pub use pipeline_orchestrator::{
    PipelineOrchestrator, PipelineConfig, PipelineJob, PipelineJobResult, JobOptions,
    PipelineHealth, PipelineMetrics, PipelineOrchestratorTrait
};