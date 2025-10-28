//! CozoDB Integration Layer
//!
//! This module provides the database-first integration with CozoDB for high-performance
//! neural processing operations. It implements connection pooling, CRUD operations,
//! and streaming queries to support 1000+ records/minute throughput.

pub mod connection_pool;
pub mod record;
pub mod connection;
pub mod error;
pub mod query;

#[cfg(test)]
mod tests;

pub use connection_pool::{CozoConnectionPool, ConnectionPoolConfig};
pub use record::CodeRecord;
pub use connection::CozoConnection;
pub use error::CozoError;
pub use query::{QueryStream, QueryParams};