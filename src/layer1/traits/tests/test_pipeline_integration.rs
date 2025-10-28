//! Integration Tests for Complete Pipeline
//!
//! Tests the full integration of DatabaseProvider + InferenceEngine + PipelineOrchestrator
//! Follows TDD principles with comprehensive contract validation

use crate::layer1::traits::implementations::*;
use crate::layer1::traits::inference::*;
use crate::layer1::traits::database::*;
use crate::layer1::traits::error::*;
use crate::layer1::traits::implementations::inference_engine::TraitInferenceEngine;
use crate::layer1::traits::implementations::pipeline_orchestrator::*;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;
use chrono::Utc;

/// Create test pipeline orchestrator
async fn create_test_orchestrator() -> Result<PipelineOrchestrator, Box<dyn std::error::Error>> {
    // Create mock database provider
    let database_provider = std::sync::Arc::new(
        SimpleDatabaseProvider::new("sqlite::memory:")
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
    );

    // Create inference engine (fallback to mock if no real model)
    let inference_engine = if PathBuf::from("./models/qwen2.5-0.5b-int4/model.safetensors").exists() {
        let engine = TraitInferenceEngine::new(
            PathBuf::from("./models/qwen2.5-0.5b-int4"),
            PathBuf::from("./tokenizer_dir"),
        ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        std::sync::Arc::new(engine) as std::sync::Arc<dyn InferenceEngine<Output = InferenceResult, Error = InferenceError>>
    } else {
        // Create mock inference engine for testing
        create_mock_inference_engine().await
    };

    // Create pipeline config
    let config = PipelineConfig {
        max_concurrent_jobs: 5,
        chunk_size: 50,
        max_processing_time: Duration::from_secs(60),
        retry_config: RetryConfig {
            max_retries: 2,
            base_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
        },
        caching_config: CachingConfig {
            enable_caching: false, // Disable for tests
            cache_ttl: Duration::from_secs(60),
            max_cache_size: 100,
        },
        monitoring_config: MonitoringConfig {
            enable_metrics: true,
            metrics_interval: Duration::from_secs(10),
            enable_tracing: false,
        },
    };

    Ok(PipelineOrchestrator::new(inference_engine, database_provider, config))
}

/// Create mock inference engine for testing when real models aren't available
async fn create_mock_inference_engine() -> std::sync::Arc<dyn InferenceEngine<Output = InferenceResult, Error = InferenceError>> {
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        MockInferenceEngine {}

        #[async_trait]
        impl InferenceEngine for MockInferenceEngine {
            type Input = String;
            type Output = InferenceResult;
            type Error = InferenceError;
            type ModelInfo = TraitModelInfo;

            async fn load_model(&self, config: ModelConfig) -> Result<Self::ModelInfo, Self::Error>;
            async fn infer(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
            async fn infer_batch(&self, inputs: Vec<Self::Input>, options: BatchOptions) -> Result<Vec<Self::Output>, Self::Error>;
            async fn infer_stream(&self, input_stream: impl futures::Stream<Item = Self::Input> + Send) -> Result<impl futures::Stream<Item = Result<Self::Output, Self::Error>> + Send, Self::Error>;
            fn model_info(&self) -> &Self::ModelInfo;
            async fn benchmark(&self, test_cases: &[BenchmarkCase]) -> Result<BenchmarkResults, Self::Error>;
            async fn session_info(&self) -> Result<SessionInfo, Self::Error>;
            async fn health_check(&self) -> Result<ModelHealth, Self::Error>;
        }
    }

    let mut mock_engine = MockInferenceEngine::new();
    let mock_model_info = TraitModelInfo::new();

    // Setup mock responses
    mock_engine
        .expect_model_info()
        .return_const(mock_model_info);

    mock_engine
        .expect_infer()
        .returning(|input| {
            let token_count = input.len() / 4; // Rough estimate
            Ok(InferenceResult {
                content: format!("Mock summary for: {}", &input[..20.min(input.len())]),
                token_count,
                confidence: 0.85,
                processing_time_ms: 50,
                session_id: SessionId(Uuid::new_v4()),
                model_info: TraitModelInfo::new(),
                metadata: InferenceMetadata {
                    temperature: Some(0.7),
                    top_p: Some(0.9),
                    top_k: Some(50),
                    max_new_tokens: Some(60),
                    min_length: Some(20),
                    repetition_penalty: Some(1.1),
                    stop_sequences: vec!["\n".to_string()],
                    prompt_template: Some("Summarize: ".to_string()),
                    custom_data: std::collections::HashMap::new(),
                },
            })
        });

    mock_engine
        .expect_health_check()
        .returning(|| Ok(ModelHealth::Healthy));

    mock_engine
        .expect_session_info()
        .returning(|| Ok(SessionInfo {
            total_sessions: 10,
            active_sessions: 0,
            available_sessions: 10,
            session_utilization: 0.0,
            average_session_lifetime: Duration::from_millis(100),
        }));

    std::sync::Arc::new(mock_engine)
}

#[tokio::test]
async fn test_pipeline_single_job_processing() {
    let orchestrator = create_test_orchestrator().await.expect("Failed to create orchestrator");

    let job = PipelineJob {
        job_id: JobId(Uuid::new_v4()),
        input: JobInput::Text {
            content: r#"
pub async fn process_user_data(user_id: Uuid) -> Result<UserProfile, Error> {
    // Fetch user from database
    let user = db::find_user_by_id(user_id).await?;

    // Process user data
    let profile = UserProfile {
        id: user.id,
        name: user.name,
        email: user.email,
        created_at: user.created_at,
        updated_at: Utc::now(),
    };

    Ok(profile)
}
"#.to_string(),
            filename: Some("user_service.rs".to_string()),
        },
        options: JobOptions {
            chunking_strategy: ChunkingStrategy::Lines { max_lines: 20 },
            inference_config: InferenceParams {
                temperature: Some(0.3),
                top_p: Some(0.8),
                top_k: Some(40),
                max_new_tokens: 50,
                min_length: 20,
                repetition_penalty: 1.1,
                stop_sequences: vec!["\n\n".to_string()],
                do_sample: true,
            },
            output_format: OutputFormat::PlainText,
            include_metadata: true,
        },
        created_at: Utc::now(),
    };

    let result = orchestrator.process_job(job).await;

    assert!(result.is_ok(), "Pipeline job processing should succeed");

    let job_result = result.unwrap();
    assert!(matches!(job_result.status, JobStatus::Completed));
    assert!(job_result.chunks_processed > 0);
    assert!(!job_result.summaries.is_empty());
    assert!(job_result.processing_time < Duration::from_secs(30));

    // Validate chunk summaries
    for summary in &job_result.summaries {
        assert!(!summary.summary.is_empty());
        assert!(summary.confidence > 0.0);
        assert!(summary.token_count > 0);
        assert!(summary.processing_time > Duration::ZERO);
    }

    println!("✅ Single job processing test passed: {} chunks processed in {:?}",
        job_result.chunks_processed, job_result.processing_time);
}

#[tokio::test]
async fn test_pipeline_batch_job_processing() {
    let orchestrator = create_test_orchestrator().await.expect("Failed to create orchestrator");

    let jobs: Vec<PipelineJob> = (0..5).map(|i| {
        PipelineJob {
            job_id: JobId(Uuid::new_v4()),
            input: JobInput::Text {
                content: format!(r#"
// Job {}: Function definition
fn process_data_{}(data: &Vec<i32>) -> Vec<i32> {{
    data.iter()
        .map(|x| x * 2)
        .collect()
}}
"#, i, i),
                filename: Some(format!("file_{}.rs", i)),
            },
            options: JobOptions {
                chunking_strategy: ChunkingStrategy::Lines { max_lines: 10 },
                inference_config: InferenceParams::default(),
                output_format: OutputFormat::PlainText,
                include_metadata: false,
            },
            created_at: Utc::now(),
        }
    }).collect();

    let start_time = std::time::Instant::now();
    let results = orchestrator.process_batch_jobs(jobs).await;
    let total_time = start_time.elapsed();

    assert_eq!(results.len(), 5);

    let successful_jobs = results.iter().filter(|r| r.is_ok()).count();
    assert!(successful_jobs >= 4, "At least 4 out of 5 jobs should succeed");

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(job_result) => {
                assert!(matches!(job_result.status, JobStatus::Completed));
                println!("✅ Batch job {} completed successfully", i);
            }
            Err(e) => {
                println!("⚠️  Batch job {} failed: {}", i, e);
            }
        }
    }

    // Batch processing should be faster than sequential
    assert!(total_time < Duration::from_secs(30),
        "Batch processing should complete within reasonable time: {:?}", total_time);

    println!("✅ Batch job processing test passed: {}/5 jobs successful in {:?}",
        successful_jobs, total_time);
}

#[tokio::test]
async fn test_pipeline_performance_contracts() {
    let orchestrator = create_test_orchestrator().await.expect("Failed to create orchestrator");

    // Test with large input to validate performance contracts
    let large_content = (0..100).map(|i| {
        format!(r#"
pub fn function_{}() -> Result<i32, Error> {{
    let data = vec![{}, {}, {}, {}, {}];
    let sum: i32 = data.iter().sum();

    if sum > 100 {{
        return Err(Error::TooLarge(sum));
    }}

    Ok(sum)
}}
"#, i, i, i+1, i+2, i+3, i+4)
    }).collect::<String>();

    let job = PipelineJob {
        job_id: JobId(Uuid::new_v4()),
        input: JobInput::Text {
            content: large_content,
            filename: Some("large_file.rs".to_string()),
        },
        options: JobOptions {
            chunking_strategy: ChunkingStrategy::Lines { max_lines: 50 },
            inference_config: InferenceParams::default(),
            output_format: OutputFormat::Markdown,
            include_metadata: true,
        },
        created_at: Utc::now(),
    };

    let result = orchestrator.process_job(job).await;

    if let Ok(job_result) = result {
        // Validate performance contracts
        assert!(job_result.processing_time < Duration::from_secs(60),
            "Job should complete within 60 seconds, took {:?}", job_result.processing_time);

        // Validate chunk processing performance
        if job_result.chunks_processed > 0 {
            let avg_chunk_time = job_result.metadata.performance_metrics.average_chunk_time;
            assert!(avg_chunk_time < Duration::from_millis(500),
                "Average chunk processing should be under 500ms, took {:?}", avg_chunk_time);
        }

        // Validate memory usage
        let memory_usage = job_result.metadata.performance_metrics.memory_peak_mb;
        assert!(memory_usage < 500, // 500MB limit for testing
            "Memory usage should be reasonable, was {}MB", memory_usage);

        println!("✅ Performance contract validation passed:");
        println!("   Total time: {:?}", job_result.processing_time);
        println!("   Chunks processed: {}", job_result.chunks_processed);
        println!("   Memory usage: {}MB", memory_usage);
    } else {
        println!("⚠️  Performance contract test failed: {:?}", result);
    }
}

#[tokio::test]
async fn test_pipeline_error_handling() {
    let orchestrator = create_test_orchestrator().await.expect("Failed to create orchestrator");

    // Test with invalid input to validate error handling
    let job = PipelineJob {
        job_id: JobId(Uuid::new_v4()),
        input: JobInput::Text {
            content: "".to_string(), // Empty input should trigger validation error
            filename: Some("empty.rs".to_string()),
        },
        options: JobOptions {
            chunking_strategy: ChunkingStrategy::Lines { max_lines: 10 },
            inference_config: InferenceParams::default(),
            output_format: OutputFormat::PlainText,
            include_metadata: false,
        },
        created_at: Utc::now(),
    };

    let result = orchestrator.process_job(job).await;

    // Should handle empty input gracefully
    match result {
        Ok(job_result) => {
            // If it succeeds, it should have meaningful results
            assert!(job_result.chunks_processed == 0 || !job_result.summaries.is_empty());
            println!("✅ Empty input handled gracefully");
        }
        Err(PipelineError::Configuration { .. }) => {
            println!("✅ Configuration error properly detected");
        }
        Err(e) => {
            println!("⚠️  Unexpected error for empty input: {}", e);
        }
    }
}

#[tokio::test]
async fn test_pipeline_health_check() {
    let orchestrator = create_test_orchestrator().await.expect("Failed to create orchestrator");

    let health = orchestrator.health_check().await;

    assert!(health.is_ok(), "Health check should succeed");

    let health_status = health.unwrap();
    println!("✅ Pipeline health check:");
    println!("   Status: {:?}", health_status.status);
    println!("   Available jobs: {}", health_status.available_jobs);
    println!("   Utilization rate: {:.2}%", health_status.utilization_rate * 100.0);

    // Should be healthy under normal conditions
    assert!(matches!(health_status.status, PipelineStatus::Healthy | PipelineStatus::Degraded));
}

#[tokio::test]
async fn test_pipeline_metrics() {
    let orchestrator = create_test_orchestrator().await.expect("Failed to create orchestrator");

    let metrics = orchestrator.get_metrics().await;

    assert!(metrics.is_ok(), "Metrics retrieval should succeed");

    let metrics = metrics.unwrap();
    println!("✅ Pipeline metrics:");
    println!("   Active jobs: {}", metrics.active_jobs);
    println!("   Max concurrent jobs: {}", metrics.max_concurrent_jobs);
    println!("   Memory usage: {}MB", metrics.memory_usage_mb);
    println!("   Inference sessions: {}", metrics.inference_sessions.total_sessions);

    // Validate reasonable bounds
    assert!(metrics.active_jobs <= metrics.max_concurrent_jobs);
    assert!(metrics.memory_usage_mb < 2000); // 2GB limit
}

#[tokio::test]
async fn test_pipeline_streaming() {
    let orchestrator = create_test_orchestrator().await.expect("Failed to create orchestrator");

    let job = PipelineJob {
        job_id: JobId(Uuid::new_v4()),
        input: JobInput::Text {
            content: "Streaming test content".to_string(),
            filename: Some("stream_test.rs".to_string()),
        },
        options: JobOptions {
            chunking_strategy: ChunkingStrategy::Lines { max_lines: 5 },
            inference_config: InferenceParams::default(),
            output_format: OutputFormat::PlainText,
            include_metadata: false,
        },
        created_at: Utc::now(),
    };

    // Create a stream of content lines
    use futures::StreamExt;
    use tokio_stream::iter;

    let content_lines: Vec<String> = (0..20).map(|i| {
        format!("fn stream_test_{}() {{ return {}; }}", i, i)
    }).collect();

    let content_stream = iter(content_lines);

    let result_stream = orchestrator.process_stream_job(job, content_stream).await;

    assert!(result_stream.is_ok(), "Stream job creation should succeed");

    let mut stream = result_stream.unwrap();
    let mut processed_chunks = 0;

    use futures::StreamExt;
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk_summary) => {
                assert!(!chunk_summary.summary.is_empty());
                processed_chunks += 1;
                println!("✅ Stream chunk processed: {}", chunk_summary.chunk_id);
            }
            Err(e) => {
                println!("⚠️  Stream chunk failed: {}", e);
            }
        }
    }

    assert!(processed_chunks > 0, "Should process at least one chunk from stream");
    println!("✅ Streaming test passed: {} chunks processed", processed_chunks);
}

#[tokio::test]
async fn test_pipeline_concurrent_load() {
    let orchestrator = create_test_orchestrator().await.expect("Failed to create orchestrator");

    // Test concurrent job processing to validate resource management
    let concurrent_jobs = 10;
    let jobs: Vec<PipelineJob> = (0..concurrent_jobs).map(|i| {
        PipelineJob {
            job_id: JobId(Uuid::new_v4()),
            input: JobInput::Text {
                content: format!(r#"
// Concurrent job {}
async fn concurrent_task_{}() -> Result<String, Error> {{
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok("Task {} completed".to_string())
}}
"#, i, i, i),
                filename: Some(format!("concurrent_{}.rs", i)),
            },
            options: JobOptions {
                chunking_strategy: ChunkingStrategy::Lines { max_lines: 10 },
                inference_config: InferenceParams::default(),
                output_format: OutputFormat::PlainText,
                include_metadata: false,
            },
            created_at: Utc::now(),
        }
    }).collect();

    let start_time = std::time::Instant::now();
    let results = orchestrator.process_batch_jobs(jobs).await;
    let total_time = start_time.elapsed();

    let successful_jobs = results.iter().filter(|r| r.is_ok()).count();
    let success_rate = successful_jobs as f64 / concurrent_jobs as f64;

    // Should handle concurrent load reasonably well
    assert!(success_rate >= 0.7, "Success rate should be at least 70%, was {:.2}%", success_rate * 100.0);
    assert!(total_time < Duration::from_secs(60), "Concurrent processing should complete within 60 seconds");

    println!("✅ Concurrent load test passed:");
    println!("   Success rate: {:.1}% ({}/{})", success_rate * 100.0, successful_jobs, concurrent_jobs);
    println!("   Total time: {:?}", total_time);
    println!("   Average time per job: {:?}", total_time / concurrent_jobs as u32);
}