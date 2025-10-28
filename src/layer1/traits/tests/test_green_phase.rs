//! GREEN PHASE Tests
//!
//! These tests validate that the GREEN phase implementations pass
//! the basic functionality requirements while maintaining the
//! TDD-first approach.

#[cfg(test)]
mod green_phase_tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_green_phase_database_connection() {
        // This test should now pass with the GREEN phase implementation
        let provider = crate::layer1::traits::implementations::MockDatabaseProvider::new("test://localhost");

        let start_time = std::time::Instant::now();
        let connection = provider.connect().await.expect("Connection should succeed");
        let elapsed = start_time.elapsed();

        // Validate performance contract
        assert!(elapsed < Duration::from_millis(100),
               "Connection acquisition must be < 100ms, took {}ms", elapsed.as_millis());

        // Test connection functionality
        let is_healthy = connection.is_healthy().await.expect("Health check should succeed");
        assert!(is_healthy, "New connection should be healthy");

        connection.close().await.expect("Connection should close cleanly");
    }

    #[tokio::test]
    async fn test_green_phase_error_handling() {
        let provider = crate::layer1::traits::implementations::MockDatabaseProvider::new("invalid_connection_string");

        let result = provider.connect().await;
        assert!(result.is_err(), "Invalid connection should fail");
    }

    #[tokio::test]
    async fn test_green_phase_performance_contract() {
        let factory = crate::layer1::traits::implementations::MockTestFactory::new()
            .with_latency(Duration::from_millis(10)); // Well within contract

        let provider = factory.database_provider();

        let start_time = std::time::Instant::now();
        let _connection = provider.connect().await.expect("Connection should succeed");
        let elapsed = start_time.elapsed();

        assert!(elapsed < Duration::from_millis(100),
               "Connection must meet performance contract, took {}ms", elapsed.as_millis());
    }
}