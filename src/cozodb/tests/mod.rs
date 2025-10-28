#[cfg(test)]
mod tests {
    use crate::cozodb::{CozoConnectionPool, ConnectionPoolConfig, CodeRecord};
    use tokio::time::{Duration, Instant};
    use futures::StreamExt;
    use serde_json::Number;

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let config = ConnectionPoolConfig {
            url: "cozodb://./test.cozo".to_string(),
            max_connections: 5,
            connection_timeout: Duration::from_secs(3),
            idle_timeout: Duration::from_secs(15),
            health_check_interval: Duration::from_secs(10),
            max_retry_attempts: 3,
            retry_base_delay: Duration::from_millis(100),
        };

        let pool = CozoConnectionPool::new(config).await.unwrap();
        assert!(pool.is_healthy());
        assert_eq!(pool.active_connections(), 0);
        assert_eq!(pool.max_connections(), 5);
    }

    #[tokio::test]
    async fn test_connection_lifecycle() {
        let pool = CozoConnectionPool::new(ConnectionPoolConfig::default()).await.unwrap();

        let connection1 = pool.acquire_connection().await.unwrap();
        let connection2 = pool.acquire_connection().await.unwrap();

        assert_eq!(pool.active_connections(), 2);

        pool.release_connection(connection1).await.unwrap();
        pool.release_connection(connection2).await.unwrap();

        assert_eq!(pool.active_connections(), 0);
    }

    #[tokio::test]
    async fn test_record_operations() {
        let pool = CozoConnectionPool::new(ConnectionPoolConfig::default()).await.unwrap();
        let connection = pool.acquire_connection().await.unwrap();

        // Test record creation
        let record = CodeRecord::new_with_metadata(
            "test-record-1",
            "fn main() { println!(\"Hello, Dobby!\"); }",
            "rust",
            std::collections::HashMap::from([
                ("complexity".to_string(), serde_json::Value::Number(Number::from(5))),
                ("lines".to_string(), serde_json::Value::Number(Number::from(2))),
            ]),
        );

        // Test insertion
        let inserted = connection.insert_record("code_records", &record).await.unwrap();
        assert_eq!(inserted.id, "test-record-1");
        assert_eq!(inserted.content, record.content);

        // Test retrieval
        let retrieved = connection.get_record_by_id("code_records", "test-record-1").await.unwrap();
        assert_eq!(retrieved.id, record.id);
        assert_eq!(retrieved.content, record.content);

        // Test update
        let mut updated_record = record.clone();
        updated_record.update_content("fn main() { println!(\"Hello, Updated Dobby!\"); }");
        let updated = connection.update_record("code_records", &updated_record).await.unwrap();
        assert_eq!(updated.id, record.id);
        assert!(updated.content != record.content);

        pool.release_connection(connection).await.unwrap();
        assert_eq!(pool.active_connections(), 0);
    }

    #[tokio::test]
    async fn test_streaming_query() {
        let pool = CozoConnectionPool::new(ConnectionPoolConfig::default()).await.unwrap();
        let connection = pool.acquire_connection().await.unwrap();

        let mut stream = connection.stream_records("code_records", "LIMIT 10").await.unwrap();
        let mut count = 0;
        let start_time = Instant::now();

        while let Some(record_result) = stream.next().await {
            let record = record_result.unwrap();
            assert!(!record.id.is_empty());
            assert!(!record.content.is_empty());
            count += 1;
        }

        let duration = start_time.elapsed();
        assert_eq!(count, 10);
        assert!(duration < Duration::from_millis(500));

        pool.release_connection(connection).await.unwrap();
    }

    #[tokio::test]
    async fn test_pool_performance() {
        let pool = CozoConnectionPool::new(ConnectionPoolConfig::default()).await.unwrap();

        let start_time = Instant::now();
        let operations_per_task = 10;
        let concurrent_tasks = 5;

        let futures: Vec<_> = (0..concurrent_tasks)
            .map(|task_id| {
                let pool = pool.clone();
                tokio::spawn(async move {
                    for i in 0..operations_per_task {
                        let connection = pool.acquire_connection().await.unwrap();

                        let record = CodeRecord::new(
                            format!("batch-{}-{}", task_id, i),
                            format!("// Concurrent operation {}-{}", task_id, i),
                            "rust",
                        );

                        connection.insert_record("test_table", &record).await.unwrap();
                        pool.release_connection(connection).await.unwrap();
                    }
                })
            })
            .collect();

        futures::future::join_all(futures).await;
        let duration = start_time.elapsed();

        assert!(duration < Duration::from_secs(5));
        assert_eq!(pool.active_connections(), 0);
        assert!(pool.is_healthy());
    }
}