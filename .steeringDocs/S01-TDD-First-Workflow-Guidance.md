# TDD-First Workflow Guidance for Dobby Project

## Executive Summary

This document establishes the TDD-First development methodology for the dobby-subagent-code-summarizer project, following the executable specification patterns defined in the architecture specification.

## 1. TDD Development Cycle

### 1.1 The STUB → RED → GREEN → REFACTOR Process

**STUB Phase**: Define trait interfaces with minimal implementations
- Create trait definitions with all required methods
- Implement stub methods that return `Ok(())` or `unimplemented!()`
- Ensure the project compiles with stub implementations

**RED Phase**: Write failing tests that specify exact behavior
- Write tests for each trait method with specific input/output contracts
- Include performance tests for time/memory constraints
- Include property-based tests for edge cases
- Tests MUST fail initially

**GREEN Phase**: Minimal implementation to make tests pass
- Implement the simplest working solution
- Focus on correctness, not elegance
- All tests must pass

**REFACTOR Phase**: Improve implementation while maintaining test coverage
- Optimize performance without breaking tests
- Improve code readability and maintainability
- Ensure all tests continue to pass

### 1.2 Test Organization Structure

```
src/
├── layer1/
│   ├── traits/
│   │   ├── database.rs          # Trait definitions
│   │   ├── inference_engine.rs  # Trait definitions
│   │   ├── pipeline.rs          # Trait definitions
│   │   ├── tests/
│   │   │   ├── test_stub_phase.rs     # Stub compilation tests
│   │   │   ├── test_red_phase.rs      # Failing contract tests
│   │   │   ├── test_green_phase.rs    # Working implementation tests
│   │   │   ├── test_refactor_phase.rs # Optimized implementation tests
│   │   │   ├── test_common.rs         # Test utilities and fixtures
│   │   │   └── mod.rs
│   │   └── implementations/
│   │       ├── database.rs           # Mock and production implementations
│   │       ├── inference_engine.rs   # Mock and production implementations
│   │       └── pipeline_orchestrator.rs
```

## 2. Executable Specification Templates

### 2.1 Trait Contract Template

```rust
/// Contract: [Trait Name] must [Primary Function]
///
/// # Preconditions
/// - [List of required input conditions]
/// - [Parameter constraints and validation]
///
/// # Postconditions
/// - [Guaranteed output conditions]
/// - [State changes and side effects]
/// - [Performance requirements]
///
/// # Error Conditions
/// - [Specific error cases and handling]
/// - [Recovery strategies]
///
/// # Performance Contract
/// - Latency: < [duration] for [typical input]
/// - Memory: < [size] for [typical input]
/// - Throughput: > [rate] operations/second
#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn contract_precondition_validation() {
        // Test that invalid preconditions are rejected
    }

    #[test]
    fn contract_postcondition_guarantee() {
        // Test that postconditions are always met
    }

    #[test]
    fn contract_error_handling() {
        // Test specific error conditions
    }

    #[tokio::test]
    async fn contract_performance_validation() {
        // Test performance constraints are met
    }
}
```

### 2.2 Mock Implementation Template

```rust
/// Mock implementation for testing trait behavior
///
/// This implementation tracks all method calls for verification
/// and provides deterministic responses for test scenarios.
pub struct MockTraitName {
    // State tracking for test verification
    call_log: Arc<RwLock<Vec<String>>>,
    // Pre-configured responses
    responses: Arc<RwLock<HashMap<String, ResponseType>>>,
    // Error injection capability
    error_mode: Arc<RwLock<bool>>,
}

impl MockTraitName {
    pub fn new() -> Self {
        Self {
            call_log: Arc::new(RwLock::new(Vec::new())),
            responses: Arc::new(RwLock::new(HashMap::new())),
            error_mode: Arc::new(RwLock::new(false)),
        }
    }

    /// Configure mock to return specific response for given input
    pub async fn configure_response(&self, input_pattern: &str, response: ResponseType) {
        let mut responses = self.responses.write().await;
        responses.insert(input_pattern.to_string(), response);
    }

    /// Enable error injection for testing error handling
    pub async fn enable_error_mode(&self, enabled: bool) {
        let mut error_mode = self.error_mode.write().await;
        *error_mode = enabled;
    }

    /// Get list of method calls for verification
    pub async fn get_call_log(&self) -> Vec<String> {
        let call_log = self.call_log.read().await;
        call_log.clone()
    }

    /// Verify specific method was called with expected parameters
    pub async fn verify_call(&self, expected_call: &str) -> bool {
        let call_log = self.call_log.read().await;
        call_log.contains(&expected_call.to_string())
    }
}
```

## 3. Development Workflow

### 3.1 Daily Development Process

1. **Morning Planning**
   - Review failing tests from previous day
   - Identify which phase (STUB/RED/GREEN/REFACTOR) to work on
   - Plan specific method implementations

2. **Development Session**
   - Always run `cargo test` before starting
   - Make small, testable changes
   - Run tests after each change
   - Commit frequently with descriptive messages

3. **End of Day**
   - Ensure all tests pass
   - Run performance benchmarks
   - Update documentation if needed
   - Clean up any temporary files

### 3.2 Quality Gates

**No compilation errors allowed**
- Project must compile after every commit
- Use `cargo check --all-targets` to verify

**All tests must pass**
- Unit tests: `cargo test --lib`
- Integration tests: `cargo test --test '*'`
- Performance tests: `cargo test --release performance`

**Performance contracts must be met**
- All performance tests must pass
- Benchmark results must meet specified targets
- Memory usage must stay within limits

## 4. Test-Driven Trait Development

### 4.1 Writing Failing Tests First (RED Phase)

```rust
// 1. Write the test that describes expected behavior
#[tokio::test]
async fn test_database_connection_reuse() {
    let mock_db = MockDatabaseProvider::new();

    // First connection should succeed
    let conn1 = mock_db.get_connection("test_pool").await.unwrap();
    assert!(conn1.is_healthy());

    // Second connection should reuse existing connection
    let conn2 = mock_db.get_connection("test_pool").await.unwrap();
    assert_eq!(conn1.id(), conn2.id()); // Same connection reused

    // Verify connection pool behavior
    let metrics = mock_db.get_connection_metrics().await;
    assert_eq!(metrics.active_connections, 1);
    assert_eq!(metrics.total_connections_created, 1);
}

// 2. Run test - it will fail because methods aren't implemented yet
// 3. Implement minimal stub to allow compilation
// 4. Run test again - it should fail with specific assertion errors
// 5. Implement logic to make test pass
```

### 4.2 Performance Contract Testing

```rust
#[tokio::test]
async fn test_inference_latency_contract() {
    let engine = MockInferenceEngine::new();
    let test_input = create_test_input(1000); // 1K tokens

    let start = Instant::now();
    let result = engine.inference(&test_input).await.unwrap();
    let latency = start.elapsed();

    // Performance contract: <100ms for 1K tokens
    assert!(latency < Duration::from_millis(100),
            "Inference took {:?}, expected <100ms", latency);

    // Verify result quality
    assert!(!result.tokens.is_empty());
    assert!(result.confidence > 0.8);
}

#[test]
fn test_memory_usage_contract() {
    let engine = MockInferenceEngine::new();
    let memory_tracker = MemoryTracker::new();

    // Process large input
    let large_input = create_test_input(100_000); // 100K tokens
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        engine.inference(&large_input).await.unwrap();
    });

    // Memory contract: <1GB peak usage
    let peak_memory = memory_tracker.get_peak_usage();
    assert!(peak_memory < 1024 * 1024 * 1024, // 1GB
            "Peak memory {}MB exceeded 1GB limit",
            peak_memory / 1024 / 1024);
}
```

## 5. Common Pitfalls and Solutions

### 5.1 Test Organization Pitfalls

**Pitfall**: Tests in wrong locations
- **Solution**: Keep unit tests in `src/` directory alongside implementation
- Keep integration tests in `tests/` directory

**Pitfall**: Tests not isolated
- **Solution**: Each test should create its own mock instances
- Use `#[tokio::test]` for async tests
- Clean up resources in test teardown

**Pitfall**: Missing edge cases
- **Solution**: Use property-based testing with `proptest`
- Test error conditions explicitly
- Include performance regression tests

### 5.2 Mock Implementation Pitfalls

**Pitfall**: Mocks too complex
- **Solution**: Keep mocks simple and focused
- Use builder pattern for complex mock setup
- Document mock behavior clearly

**Pitfall**: Mocks diverging from real behavior
- **Solution**: Regularly validate mocks against production implementations
- Use contract tests to ensure consistency
- Document any differences explicitly

## 6. Continuous Integration

### 6.1 CI Pipeline Steps

1. **Compilation Check**
   ```bash
   cargo check --all-targets
   cargo clippy -- -D warnings
   ```

2. **Test Suite**
   ```bash
   cargo test --lib
   cargo test --test '*'
   cargo test --release performance
   ```

3. **Benchmark Validation**
   ```bash
   cargo bench --all
   ```

4. **Documentation Check**
   ```bash
   cargo doc --no-deps --document-private-items
   ```

### 6.2 Performance Regression Detection

- Store benchmark results in CI
- Alert on >10% performance degradation
- Track memory usage trends
- Monitor test execution time

## 7. Success Metrics

### 7.1 Code Quality Metrics
- Test coverage: >90%
- Compilation time: <30 seconds
- Test execution time: <2 minutes
- Zero clippy warnings

### 7.2 Development Velocity Metrics
- Time from STUB to GREEN: <4 hours per trait method
- Time from GREEN to REFACTOR: <8 hours per trait
- Bug fix time: <24 hours
- Feature delivery cycle: <1 week

### 7.3 Performance Metrics
- All performance contracts passing
- Zero performance regressions in releases
- Memory usage within specified limits
- Latency targets met consistently

This guidance document ensures systematic TDD-First development with measurable outcomes and continuous quality improvement.