//! CozoDB Record Types
//!
//! Defines the data structures for storing and retrieving code records in CozoDB.
//! Supports the database-first neural processing architecture with efficient serialization.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Database record for code storage and processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeRecord {
    /// Unique identifier for the record
    pub id: String,

    /// The actual code content
    pub content: String,

    /// Programming language identifier
    pub language: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Additional metadata for neural processing
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CodeRecord {
    /// Create a new code record
    pub fn new(
        id: impl Into<String>,
        content: impl Into<String>,
        language: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            content: content.into(),
            language: language.into(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Create a new code record with metadata
    pub fn new_with_metadata(
        id: impl Into<String>,
        content: impl Into<String>,
        language: impl Into<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            content: content.into(),
            language: language.into(),
            created_at: now,
            updated_at: now,
            metadata,
        }
    }

    /// Update the content and refresh the updated_at timestamp
    pub fn update_content(&mut self, new_content: impl Into<String>) {
        self.content = new_content.into();
        self.updated_at = Utc::now();
    }

    /// Update metadata
    pub fn update_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
        self.updated_at = Utc::now();
    }

    /// Get a metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Check if record is recent (created within last N seconds)
    pub fn is_recent(&self, seconds: i64) -> bool {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.created_at);
        duration.num_seconds() <= seconds
    }

    /// Get content length in characters
    pub fn content_length(&self) -> usize {
        self.content.len()
    }

    /// Get content length in lines
    pub fn content_lines(&self) -> usize {
        self.content.lines().count()
    }
}

/// Database statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_records: usize,
    pub total_size_bytes: usize,
    pub languages_count: HashMap<String, usize>,
    pub last_updated: DateTime<Utc>,
    pub connection_pool_size: usize,
    pub active_connections: usize,
}

impl Default for DatabaseStats {
    fn default() -> Self {
        Self {
            total_records: 0,
            total_size_bytes: 0,
            languages_count: HashMap::new(),
            last_updated: Utc::now(),
            connection_pool_size: 0,
            active_connections: 0,
        }
    }
}

/// Connection pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub pool_id: String,
    pub pool_size: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_acquired: u64,
    pub total_released: u64,
    pub created_at: DateTime<Utc>,
}

impl PoolInfo {
    pub fn new(pool_size: usize) -> Self {
        let pool_id = uuid::Uuid::new_v4().to_string();
        Self {
            pool_id,
            pool_size,
            active_connections: 0,
            idle_connections: pool_size,
            total_acquired: 0,
            total_released: 0,
            created_at: Utc::now(),
        }
    }

    pub fn utilization_rate(&self) -> f64 {
        if self.pool_size == 0 {
            return 0.0;
        }
        self.active_connections as f64 / self.pool_size as f64
    }
}