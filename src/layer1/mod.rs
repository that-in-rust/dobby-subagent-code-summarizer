//! Layer 1: Core Abstractions
//!
//! This layer defines the foundational traits and abstractions for the dobby
//! code summarizer system. It follows TDD-first methodology with comprehensive
//! executable specifications.
//!
//! ## Architecture Layers:
//! - **L1 Core** (this layer): Foundational traits and abstractions
//! - **L2 Standard**: Concrete implementations for common use cases
//! - **L3 External**: Third-party integrations and adapters

pub mod traits;

// Re-export core traits
pub use traits::*;