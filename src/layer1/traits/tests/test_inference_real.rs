//! GREEN Phase: Real model testing with fallback strategies
//!
//! Uses real small models when available, graceful fallbacks when not
//! Follows Rust idioms for conditional testing and resource management

use crate::layer1::traits::inference::*;
use crate::layer1::traits::error::*;
use crate::inference::OptimizedInferenceEngine;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;

/// Test configuration for real model testing
struct RealModelTestConfig {
    model_path: PathBuf,
    tokenizer_path: PathBuf,
    should_exist: bool,
}

impl RealModelTestConfig {
    async fn new() -> Vec<Self> {
        vec![
            Self {
                model_path: PathBuf::from("./models/qwen2.5-0.5b-int4/model_quantized.onnx"),
                tokenizer_path: PathBuf::from("./tokenizer_dir/tokenizer.json"),
                should_exist: true,
            },
            Self {
                model_path: PathBuf::from("./models/smollm2-135m/model.onnx"),
                tokenizer_path: PathBuf::from("./models/smollm2-135m/tokenizer.json"),
                should_exist: false, // Fallback test
            },
        ]
    }

    async fn model_exists(&self) -> bool {
        fs::metadata(&self.model_path).await.is_ok() &&
        fs::metadata(&self.tokenizer_path).await.is_ok()
    }
}

/// GREEN Phase Test: Successful model loading and inference
#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use tokio_stream::iter;

    /// GREEN Phase: Test successful model loading when files exist
    #[tokio::test]
    async fn test_green_successful_model_loading() {
        let configs = RealModelTestConfig::new().await;

        for config in configs {
            if config.model_exists().await {
                // Test with real model files
                let result = OptimizedInferenceEngine::new(
                    config.model_path.parent().unwrap().to_path_buf(),
                    config.tokenizer_path.parent().unwrap().to_path_buf(),
                );

                assert!(result.is_ok(),
                    "Engine should create successfully when model files exist at {:?}",
                    config.model_path);

                let engine = result.unwrap();

                // Test basic inference
                let test_chunk = r#"
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
"#;

                let inference_result = engine.summarize_chunk(test_chunk);
                assert!(inference_result.is_ok(),
                    "Basic inference should work with real model");

                let summary = inference_result.unwrap();
                assert!(!summary.is_empty(),
                    "Summary should not be empty for real model inference");

                println!("✅ Real model test passed: {}", summary);
                return; // Success with real model
            }
        }

        // Fallback: Skip test gracefully if no models available
        println!("⚠️  No real model files found, skipping real model inference test");
    }

    /// GREEN Phase: Test performance characteristics with real model
    #[tokio::test]
    async fn test_green_performance_characteristics() {
        let configs = RealModelTestConfig::new().await;

        for config in configs {
            if config.model_exists().await {
                let engine = OptimizedInferenceEngine::new(
                    config.model_path.parent().unwrap().to_path_buf(),
                    config.tokenizer_path.parent().unwrap().to_path_buf(),
                ).unwrap();

                let test_chunks = vec![
                    ("small_function", "fn hello() { println!(\"Hello\"); }"),
                    ("medium_function", r#"
fn process_data(data: &Vec<i32>) -> Result<Vec<i32>, String> {
    if data.is_empty() {
        return Err("No data to process".to_string());
    }

    let mut result = Vec::new();
    for &item in data {
        if item > 0 {
            result.push(item * 2);
        } else {
            result.push(item.abs());
        }
    }
    Ok(result)
}
"#),
                    ("large_struct", r#"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserService {
    repository: Arc<dyn UserRepository>,
    cache: Arc<dyn CacheService>,
    email_service: Arc<dyn EmailService>,
    logger: Arc<dyn Logger>,
    config: ServiceConfig,
}

impl UserService {
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<UserResponse, ServiceError> {
        // Validate input
        if request.email.is_empty() {
            return Err(ServiceError::ValidationError("Email required".to_string()));
        }

        // Check if user exists
        if self.repository.exists_by_email(&request.email).await? {
            return Err(ServiceError::Conflict("User already exists".to_string()));
        }

        // Create user entity
        let user = User::new(
            Uuid::new_v4(),
            request.name,
            request.email,
            request.password_hash,
            Utc::now(),
        );

        // Save to database
        let saved_user = self.repository.create(user).await?;

        // Send welcome email
        if let Err(e) = self.email_service.send_welcome(&saved_user.email).await {
            self.logger.warn(&format!("Failed to send welcome email: {}", e));
        }

        Ok(UserResponse::from(saved_user))
    }
}
"#),
                ];

                for (name, chunk) in test_chunks {
                    let start = std::time::Instant::now();
                    let result = engine.summarize_chunk(chunk);
                    let duration = start.elapsed();

                    assert!(result.is_ok(),
                        "Inference should succeed for {} chunk", name);

                    let summary = result.unwrap();
                    assert!(!summary.is_empty(),
                        "Summary should not be empty for {}", name);

                    // Performance assertions (adjusted for real model expectations)
                    assert!(duration < Duration::from_secs(5),
                        "Inference should complete within 5 seconds for {}, took {:?}", name, duration);

                    println!("✅ Performance test '{}' passed in {:?}: {}",
                        name, duration, summary);
                }

                return; // Success with real model
            }
        }

        println!("⚠️  No real model files found, skipping performance test");
    }

    /// GREEN Phase: Test concurrent inference capabilities
    #[tokio::test]
    async fn test_green_concurrent_inference() {
        let configs = RealModelTestConfig::new().await;

        for config in configs {
            if config.model_exists().await {
                let engine = std::sync::Arc::new(OptimizedInferenceEngine::new(
                    config.model_path.parent().unwrap().to_path_buf(),
                    config.tokenizer_path.parent().unwrap().to_path_buf(),
                ).unwrap());

                let test_chunks: Vec<String> = (0..10).map(|i| {
                    format!(r#"
fn function_{}() {{
    // Test function {} for concurrent processing
    let result = {} * 2;
    println!("Result: {{}}", result);
    result
}}
"#, i, i, i)
                }).collect();

                // Spawn concurrent inference tasks
                let mut handles = Vec::new();
                for (i, chunk) in test_chunks.into_iter().enumerate() {
                    let engine_clone = std::sync::Arc::clone(&engine);
                    let handle = tokio::spawn(async move {
                        let start = std::time::Instant::now();
                        let result = engine_clone.summarize_chunk(&chunk);
                        let duration = start.elapsed();

                        (i, result, duration)
                    });
                    handles.push(handle);
                }

                // Collect results
                let mut successful_inferences = 0;
                let mut total_duration = Duration::ZERO;

                for handle in handles {
                    match handle.await {
                        Ok((i, result, duration)) => {
                            match result {
                                Ok(summary) => {
                                    assert!(!summary.is_empty(),
                                        "Concurrent inference {} should produce non-empty summary", i);
                                    successful_inferences += 1;
                                    total_duration += duration;
                                    println!("✅ Concurrent inference {} completed in {:?}: {}",
                                        i, duration, summary.chars().take(50).collect::<String>());
                                }
                                Err(e) => {
                                    println!("❌ Concurrent inference {} failed: {}", i, e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Concurrent inference task panicked: {}", e);
                        }
                    }
                }

                // Assertions
                assert!(successful_inferences > 0,
                    "At least some concurrent inferences should succeed");

                let avg_duration = total_duration / successful_inferences as u32;
                assert!(avg_duration < Duration::from_secs(3),
                    "Average concurrent inference time should be reasonable: {:?}", avg_duration);

                println!("✅ Concurrent inference test passed: {}/{} successful, avg time: {:?}",
                    successful_inferences, 10, avg_duration);

                return; // Success with real model
            }
        }

        println!("⚠️  No real model files found, skipping concurrent inference test");
    }

    /// GREEN Phase: Test error handling with graceful degradation
    #[tokio::test]
    async fn test_green_error_handling_graceful_degradation() {
        let configs = RealModelTestConfig::new().await;

        for config in configs {
            if config.model_exists().await {
                let engine = OptimizedInferenceEngine::new(
                    config.model_path.parent().unwrap().to_path_buf(),
                    config.tokenizer_path.parent().unwrap().to_path_buf(),
                ).unwrap();

                // Test with various edge cases
                let test_cases = vec![
                    ("empty_string", ""),
                    ("very_long_string", &"x".repeat(10000)),
                    ("special_characters", "fn test() { println!(\"🚀 Hello 世界! 🦀\"); }"),
                    ("invalid_unicode", &format!("fn test() {{ println!(\"{}\"); }}",
                        [0xC0, 0x80].iter().map(|&b| b as char).collect::<String>())),
                ];

                for (name, input) in test_cases {
                    let result = engine.summarize_chunk(input);

                    // Should either succeed or fail gracefully
                    match result {
                        Ok(summary) => {
                            println!("✅ {} test succeeded: '{}'", name,
                                summary.chars().take(100).collect::<String>());
                        }
                        Err(e) => {
                            println!("⚠️  {} test failed gracefully: {}", name, e);
                            // Error should be informative and not panic
                            assert!(!e.to_string().is_empty(),
                                "Error message should be informative");
                        }
                    }
                }

                return; // Success with real model
            }
        }

        println!("⚠️  No real model files found, skipping error handling test");
    }

    /// GREEN Phase: Test memory usage patterns
    #[tokio::test]
    async fn test_green_memory_usage_patterns() {
        let configs = RealModelTestConfig::new().await;

        for config in configs {
            if config.model_exists().await {
                // Measure memory before engine creation
                let initial_memory = memory_stats::memory_stats()
                    .map(|stats| stats.physical_mem)
                    .unwrap_or(0);

                let engine = OptimizedInferenceEngine::new(
                    config.model_path.parent().unwrap().to_path_buf(),
                    config.tokenizer_path.parent().unwrap().to_path_buf(),
                ).unwrap();

                // Measure memory after engine creation
                let after_load_memory = memory_stats::memory_stats()
                    .map(|stats| stats.physical_mem)
                    .unwrap_or(0);

                // Run multiple inferences
                let test_chunk = r#"
pub async fn process_large_dataset(data: &[String]) -> Result<Vec<String>, Error> {
    let futures: Vec<_> = data.iter()
        .enumerate()
        .map(|(i, item)| async move {
            format!("Processed item {}: {}", i, item.to_uppercase())
        })
        .collect();

    let results = futures::future::try_join_all(futures).await?;
    Ok(results)
}
"#;

                for i in 0..5 {
                    let _result = engine.summarize_chunk(test_chunk);
                    if i % 10 == 0 {
                        tokio::task::yield_now().await;
                    }
                }

                // Measure memory after inference
                let after_inference_memory = memory_stats::memory_stats()
                    .map(|stats| stats.physical_mem)
                    .unwrap_or(0);

                // Memory usage should be reasonable
                let load_memory_increase = after_load_memory.saturating_sub(initial_memory);
                let inference_memory_increase = after_inference_memory.saturating_sub(after_load_memory);

                println!("📊 Memory usage:");
                println!("  Initial: {} MB", initial_memory / 1024 / 1024);
                println!("  After load: {} MB (+{} MB)",
                    after_load_memory / 1024 / 1024, load_memory_increase / 1024 / 1024);
                println!("  After inference: {} MB (+{} MB)",
                    after_inference_memory / 1024 / 1024, inference_memory_increase / 1024 / 1024);

                // Reasonable memory bounds (adjust based on your model size)
                assert!(load_memory_increase < 2_000_000_000, // < 2GB for model loading
                    "Model loading should not use excessive memory: {} MB",
                    load_memory_increase / 1024 / 1024);

                assert!(inference_memory_increase < 500_000_000, // < 500MB for inference
                    "Inference should not leak excessive memory: {} MB",
                    inference_memory_increase / 1024 / 1024);

                return; // Success with real model
            }
        }

        println!("⚠️  No real model files found, skipping memory usage test");
    }
}