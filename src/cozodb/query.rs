//! CozoDB Query Operations
//!
//! Provides streaming query functionality and parameter handling for efficient
//! database operations in the database-first architecture.

use crate::cozodb::{error::{CozoError, CozoResult}, record::CodeRecord};
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::{Duration, Instant};

/// Query parameters for database operations
#[derive(Debug, Clone)]
pub struct QueryParams {
    /// Maximum number of records to return
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,

    /// Order by clause
    pub order_by: Option<String>,

    /// Filter conditions
    pub filters: Vec<FilterCondition>,

    /// Query timeout
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

#[derive(Debug, Clone)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    In,
}

#[derive(Debug, Clone)]
pub enum FilterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<FilterValue>),
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            limit: None,
            offset: None,
            order_by: None,
            filters: Vec::new(),
            timeout: Some(Duration::from_secs(30)),
        }
    }
}

impl QueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_order_by(mut self, order_by: impl Into<String>) -> Self {
        self.order_by = Some(order_by.into());
        self
    }

    pub fn with_filter(mut self, filter: FilterCondition) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Build query string from parameters
    pub fn build_query(&self, table: &str) -> String {
        let mut query = format!("SELECT * FROM {}", table);

        if !self.filters.is_empty() {
            let filter_clauses: Vec<String> = self.filters
                .iter()
                .map(|f| f.to_sql_clause())
                .collect();
            query.push_str(&format!(" WHERE {}", filter_clauses.join(" AND ")));
        }

        if let Some(order_by) = &self.order_by {
            query.push_str(&format!(" ORDER BY {}", order_by));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }
}

impl FilterCondition {
    pub fn to_sql_clause(&self) -> String {
        match &self.operator {
            FilterOperator::Equals => format!("{} = {}", self.field, self.value.to_sql_value()),
            FilterOperator::NotEquals => format!("{} != {}", self.field, self.value.to_sql_value()),
            FilterOperator::GreaterThan => format!("{} > {}", self.field, self.value.to_sql_value()),
            FilterOperator::GreaterThanOrEqual => format!("{} >= {}", self.field, self.value.to_sql_value()),
            FilterOperator::LessThan => format!("{} < {}", self.field, self.value.to_sql_value()),
            FilterOperator::LessThanOrEqual => format!("{} <= {}", self.field, self.value.to_sql_value()),
            FilterOperator::Contains => format!("{} LIKE '%{}%'", self.field, self.value.to_string()),
            FilterOperator::StartsWith => format!("{} LIKE '{}%'", self.field, self.value.to_string()),
            FilterOperator::EndsWith => format!("{} LIKE '%{}'", self.field, self.value.to_string()),
            FilterOperator::In => {
                if let FilterValue::List(values) = &self.value {
                    let values_str: Vec<String> = values.iter()
                        .map(|v| v.to_sql_value())
                        .collect();
                    format!("{} IN ({})", self.field, values_str.join(", "))
                } else {
                    format!("{} IN ({})", self.field, self.value.to_sql_value())
                }
            }
        }
    }
}

impl FilterValue {
    pub fn to_sql_value(&self) -> String {
        match self {
            FilterValue::String(s) => format!("'{}'", s.replace('\'', "''")),
            FilterValue::Number(n) => n.to_string(),
            FilterValue::Boolean(b) => (if *b { "TRUE" } else { "FALSE" }).to_string(),
            FilterValue::List(_) => "NULL".to_string(), // Lists handled separately
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            FilterValue::String(s) => s.clone(),
            FilterValue::Number(n) => n.to_string(),
            FilterValue::Boolean(b) => b.to_string(),
            FilterValue::List(values) => format!("[{}]",
                values.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

/// Streaming query results
pub struct QueryStream {
    /// Records to stream
    records: Vec<CodeRecord>,

    /// Current position
    position: usize,

    /// Stream creation time
    created_at: Instant,

    /// Maximum stream duration
    timeout: Duration,
}

impl QueryStream {
    /// Create a new mock stream for testing
    pub async fn new_mock(count: usize) -> CozoResult<Self> {
        let mut records = Vec::with_capacity(count);

        for i in 0..count {
            let record = CodeRecord::new_with_metadata(
                format!("mock-record-{}", i),
                format!("// Mock record number {}\nfn function_{}() {{\n    // Implementation here\n}}", i, i),
                "rust".to_string(),
                std::collections::HashMap::from([
                    ("test_index".to_string(), serde_json::Value::Number(i as f64)),
                    ("batch_id".to_string(), serde_json::Value::String("test-batch".to_string())),
                ]),
            );
            records.push(record);
        }

        Ok(Self {
            records,
            position: 0,
            created_at: Instant::now(),
            timeout: Duration::from_secs(30),
        })
    }

    /// Create stream from actual query results
    pub fn new(records: Vec<CodeRecord>, timeout: Duration) -> Self {
        Self {
            records,
            position: 0,
            created_at: Instant::now(),
            timeout,
        }
    }

    /// Check if stream is still within timeout
    pub fn is_valid(&self) -> bool {
        self.created_at.elapsed() < self.timeout
    }

    /// Get remaining count of records
    pub fn remaining(&self) -> usize {
        self.records.len().saturating_sub(self.position)
    }
}

impl Stream for QueryStream {
    type Item = CozoResult<CodeRecord>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check timeout
        if !self.is_valid() {
            return Poll::Ready(Some(Err(CozoError::query_failed(
                "Stream timeout",
                "QueryStream timeout",
            ))));
        }

        // Check if we have more records
        if self.position < self.records.len() {
            let record = self.records[self.position].clone();
            self.position += 1;
            Poll::Ready(Some(Ok(record)))
        } else {
            Poll::Ready(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_params_building() {
        let params = QueryParams::new()
            .with_limit(100)
            .with_order_by("created_at DESC")
            .with_filter(FilterCondition {
                field: "language".to_string(),
                operator: FilterOperator::Equals,
                value: FilterValue::String("rust".to_string()),
            });

        let query = params.build_query("code_records");
        assert!(query.contains("SELECT * FROM code_records"));
        assert!(query.contains("LIMIT 100"));
        assert!(query.contains("ORDER BY created_at DESC"));
        assert!(query.contains("language = 'rust'"));
    }

    #[tokio::test]
    async fn test_mock_stream() {
        let mut stream = QueryStream::new_mock(10).await.unwrap();
        let mut count = 0;

        while let Some(record_result) = stream.next().await {
            assert!(record_result.is_ok());
            let record = record_result.unwrap();
            assert!(!record.id.is_empty());
            assert!(!record.content.is_empty());
            count += 1;
        }

        assert_eq!(count, 10);
        assert_eq!(stream.remaining(), 0);
    }
}