//! CozoDB Error Types
//!
//! Comprehensive error handling for CozoDB operations with proper error recovery
//! and detailed diagnostics for the database-first architecture.

use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CozoError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("Query execution failed: {message}")]
    QueryFailed { message: String, query: String },

    #[error("Transaction failed: {message}")]
    TransactionFailed { message: String },

    #[error("Resource limit exhausted: {resource} ({used}/{limit})")]
    ResourceLimitExhausted {
        resource: String,
        used: usize,
        limit: usize,
    },

    #[error("Database not found: {path}")]
    DatabaseNotFound { path: String },

    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },

    #[error("Serialization failed: {message}")]
    SerializationFailed { message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl CozoError {
    pub fn connection_failed(message: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            message: message.into(),
        }
    }

    pub fn query_failed(message: impl Into<String>, query: impl Into<String>) -> Self {
        Self::QueryFailed {
            message: message.into(),
            query: query.into(),
        }
    }

    pub fn transaction_failed(message: impl Into<String>) -> Self {
        Self::TransactionFailed {
            message: message.into(),
        }
    }

    pub fn resource_limit_exhausted(
        resource: impl Into<String>,
        used: usize,
        limit: usize,
    ) -> Self {
        Self::ResourceLimitExhausted {
            resource: resource.into(),
            used,
            limit,
        }
    }

    pub fn database_not_found(path: impl Into<String>) -> Self {
        Self::DatabaseNotFound { path: path.into() }
    }

    pub fn invalid_configuration(message: impl Into<String>) -> Self {
        Self::InvalidConfiguration {
            message: message.into(),
        }
    }

    pub fn serialization_failed(message: impl Into<String>) -> Self {
        Self::SerializationFailed {
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

pub type CozoResult<T> = Result<T, CozoError>;