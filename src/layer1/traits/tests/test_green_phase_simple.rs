//! GREEN PHASE Simple Tests
//!
//! These tests validate the GREEN phase simple implementation works
//! and demonstrates the TDD cycle completion.

#[cfg(test)]
mod green_phase_simple_tests {
    use std::time::Duration;

    #[tokio::test]
    async fn test_green_phase_simple_connection_success() {
        // GREEN PHASE: This test should now PASS with working implementation
        let provider = crate::layer1::traits::implementations::database_simple::SimpleMockDatabaseProvider::new("test://localhost");

        let start_time = std::time::Instant::now();
        let connection = provider.connect().await.expect("Connection should succeed");
        let elapsed = start_time.elapsed();

        println!("GREEN PHASE SUCCESS: Connection established in {}ms", elapsed.as_millis());

        // Validate performance contract
        assert!(elapsed < Duration::from_millis(100),
               "Connection acquisition must be < 100ms, took {}ms", elapsed.as_millis());

        // Test connection functionality
        let is_healthy = connection.is_healthy().await.expect("Health check should succeed");
        assert!(is_healthy, "New connection should be healthy");

        connection.close().await.expect("Connection should close cleanly");

        println!("GREEN PHASE: All tests passed - implementation works!");
    }

    #[tokio::test]
    async fn test_green_phase_simple_error_handling() {
        let provider = crate::layer1::traits::implementations::database_simple::SimpleMockDatabaseProvider::new("invalid_connection_string");

        let result = provider.connect().await;
        assert!(result.is_err(), "Invalid connection should fail");

        match result.unwrap_err() {
            crate::layer1::traits::implementations::database_simple::SimpleError::InvalidConnectionString => {
                println!("GREEN PHASE: Error handling works correctly");
            }
            _ => panic!("Expected InvalidConnectionString error"),
        }
    }

    #[tokio::test]
    async fn test_green_phase_simple_performance_contract() {
        let factory = crate::layer1::traits::implementations::database_simple::SimpleMockTestFactory::new()
            .with_latency(10); // Well within contract

        let provider = factory.database_provider();

        let start_time = std::time::Instant::now();
        let _connection = provider.connect().await.expect("Connection should succeed");
        let elapsed = start_time.elapsed();

        println!("GREEN PHASE: Performance contract test - took {}ms", elapsed.as_millis());

        assert!(elapsed < Duration::from_millis(100),
               "Connection must meet performance contract, took {}ms", elapsed.as_millis());

        // Test health check performance
        let health_start = std::time::Instant::now();
        let _status = provider.health_check().await.expect("Health check should succeed");
        let health_elapsed = health_start.elapsed();

        assert!(health_elapsed < Duration::from_millis(50),
               "Health check must meet performance contract, took {}ms", health_elapsed.as_millis());
    }

    #[tokio::test]
    async fn test_green_phase_simple_performance_violation() {
        let provider = crate::layer1::traits::implementations::database_simple::SimpleMockDatabaseProvider::new("test://localhost")
            .with_latency(150); // Intentionally violate performance contract

        let result = provider.connect().await;
        assert!(result.is_err(), "Performance violation should fail");

        match result.unwrap_err() {
            crate::layer1::traits::implementations::database_simple::SimpleError::PerformanceViolation { operation, actual_ms, limit_ms } => {
                println!("GREEN PHASE: Performance violation correctly detected - {} took {}ms (limit: {}ms)", operation, actual_ms, limit_ms);
                assert_eq!(operation, "connect");
                assert!(actual_ms > limit_ms);
            }
            _ => panic!("Expected PerformanceViolation error"),
        }
    }

    #[tokio::test]
    async fn test_green_phase_vs_red_phase() {
        println!("=== GREEN PHASE vs RED PHASE Comparison ===");

        // RED phase test should still fail
        println!("Testing RED phase behavior (should fail)...");
        let red_result = std::panic::catch_unwind(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                use crate::layer1::traits::tests::test_simple::simple_red_phase_tests::{SimpleTrait, MockImplementation};
                let mock = MockImplementation;
                mock.do_something()
            })
        });

        match red_result {
            Ok(_) => panic!("RED phase test should have failed!"),
            Err(_) => println!("✅ RED phase: Test fails as expected"),
        }

        // GREEN phase test should pass
        println!("Testing GREEN phase behavior (should succeed)...");
        let provider = crate::layer1::traits::implementations::database_simple::SimpleMockDatabaseProvider::new("test://localhost");
        let result = provider.connect().await;

        match result {
            Ok(_) => println!("✅ GREEN phase: Test passes as expected"),
            Err(e) => panic!("GREEN phase test should have passed! Error: {}", e),
        }

        println!("=== TDD Cycle Complete: STUB → RED → GREEN ===");
    }
}