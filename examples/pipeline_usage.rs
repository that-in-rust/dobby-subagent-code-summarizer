//! Pipeline Usage Examples
//!
//! Demonstrates idiomatic Rust usage of the complete InferenceEngine + PipelineOrchestrator system
//! Shows production-ready patterns with proper error handling and resource management

use dobby_subagent_code_summarizer::layer1::traits::implementations::*;
use dobby_subagent_code_summarizer::layer1::traits::inference::*;
use dobby_subagent_code_summarizer::layer1::traits::pipeline::*;
use dobby_subagent_code_summarizer::layer1::traits::error::*;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🚀 Dobby Subagent Code Summarizer - Pipeline Usage Examples");

    // Example 1: Basic pipeline setup and usage
    basic_pipeline_example().await?;

    // Example 2: Batch processing multiple files
    batch_processing_example().await?;

    // Example 3: Streaming processing for large files
    streaming_processing_example().await?;

    // Example 4: Performance monitoring and health checks
    monitoring_example().await?;

    println!("✅ All examples completed successfully");
    Ok(())
}

/// Example 1: Basic pipeline setup and usage
async fn basic_pipeline_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Example 1: Basic Pipeline Setup");

    // Create database provider
    let database_provider = std::sync::Arc::new(
        SimpleDatabaseProvider::new("sqlite::memory:")?
    );

    // Create inference engine (handles missing models gracefully)
    let inference_engine = create_inference_engine().await?;

    // Configure pipeline
    let pipeline_config = PipelineConfig {
        max_concurrent_jobs: 5,
        chunk_size: 100, // lines per chunk
        max_processing_time: Duration::from_secs(120),
        retry_config: RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        },
        caching_config: CachingConfig {
            enable_caching: true,
            cache_ttl: Duration::from_secs(3600),
            max_cache_size: 1000,
        },
        monitoring_config: MonitoringConfig {
            enable_metrics: true,
            metrics_interval: Duration::from_secs(30),
            enable_tracing: true,
        },
    };

    // Create orchestrator
    let orchestrator = PipelineOrchestrator::new(inference_engine, database_provider, pipeline_config);

    // Define job
    let job = PipelineJob {
        job_id: JobId(Uuid::new_v4()),
        input: JobInput::Text {
            content: include_str!("../fixtures/sample_code.rs").to_string(),
            filename: Some("sample_code.rs".to_string()),
        },
        options: JobOptions {
            chunking_strategy: ChunkingStrategy::Lines { max_lines: 50 },
            inference_config: InferenceParams {
                temperature: Some(0.3),
                top_p: Some(0.8),
                top_k: Some(40),
                max_new_tokens: 80,
                min_length: 30,
                repetition_penalty: 1.1,
                stop_sequences: vec!["\n\n".to_string()],
                do_sample: true,
            },
            output_format: OutputFormat::Markdown,
            include_metadata: true,
        },
        created_at: chrono::Utc::now(),
    };

    // Process job
    println!("🔄 Processing single job...");
    let result = orchestrator.process_job(job).await?;

    // Display results
    match result.status {
        JobStatus::Completed => {
            println!("✅ Job completed successfully!");
            println!("📊 Results:");
            println!("   Chunks processed: {}", result.chunks_processed);
            println!("   Processing time: {:?}", result.processing_time);
            println!("   Summaries generated: {}", result.summaries.len());

            for (i, summary) in result.summaries.iter().take(3).enumerate() {
                println!("\n📝 Summary {} (confidence: {:.2}):", i + 1, summary.confidence);
                println!("   {}", summary.summary);
            }

            if result.summaries.len() > 3 {
                println!("\n... and {} more summaries", result.summaries.len() - 3);
            }
        }
        JobStatus::Failed { ref error } => {
            println!("❌ Job failed: {}", error);
            return Err(Box::new(error.clone()));
        }
        _ => {
            println!("⚠️  Job in unexpected state: {:?}", result.status);
        }
    }

    Ok(())
}

/// Example 2: Batch processing multiple files
async fn batch_processing_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📦 Example 2: Batch Processing");

    // Create orchestrator (reuse from previous example)
    let orchestrator = create_orchestrator().await?;

    // Create multiple jobs for batch processing
    let jobs: Vec<PipelineJob> = vec![
        create_text_job("user_service.rs", include_str!("../fixtures/user_service.rs")),
        create_text_job("database.rs", include_str!("../fixtures/database.rs")),
        create_text_job("api.rs", include_str!("../fixtures/api.rs")),
        create_text_job("utils.rs", include_str!("../fixtures/utils.rs")),
        create_text_job("config.rs", include_str!("../fixtures/config.rs")),
    ];

    println!("🔄 Processing {} jobs in batch...", jobs.len());

    let start_time = std::time::Instant::now();
    let results = orchestrator.process_batch_jobs(jobs).await;
    let total_time = start_time.elapsed();

    // Analyze results
    let successful_jobs = results.iter().filter(|r| r.is_ok()).count();
    let failed_jobs = results.len() - successful_jobs;

    println!("📊 Batch Processing Results:");
    println!("   Total jobs: {}", results.len());
    println!("   Successful: {}", successful_jobs);
    println!("   Failed: {}", failed_jobs);
    println!("   Total time: {:?}", total_time);
    println!("   Average time per job: {:?}", total_time / results.len() as u32);

    // Display successful results
    for (i, result) in results.iter().enumerate() {
        if let Ok(job_result) = result {
            println!("✅ Job {}: {} chunks in {:?}",
                i + 1, job_result.chunks_processed, job_result.processing_time);
        } else {
            println!("❌ Job {}: Failed", i + 1);
        }
    }

    Ok(())
}

/// Example 3: Streaming processing for large files
async fn streaming_processing_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌊 Example 3: Streaming Processing");

    let orchestrator = create_orchestrator().await?;

    // Create job for streaming
    let job = PipelineJob {
        job_id: JobId(Uuid::new_v4()),
        input: JobInput::Text {
            content: "Large file processing".to_string(),
            filename: Some("large_file.rs".to_string()),
        },
        options: JobOptions {
            chunking_strategy: ChunkingStrategy::Lines { max_lines: 20 },
            inference_config: InferenceParams::default(),
            output_format: OutputFormat::PlainText,
            include_metadata: false,
        },
        created_at: chrono::Utc::now(),
    };

    // Create content stream (simulating large file)
    use futures::StreamExt;
    use tokio_stream::iter;

    let content_lines: Vec<String> = (0..100).map(|i| {
        format!(r#"
// Function {} - This represents a large code file
pub async fn process_item_{}(item: &str) -> Result<String, Error> {{
    // Processing logic for item {}
    let processed = format!("Processed: {{}}", item.to_uppercase());

    // Simulate some async work
    tokio::time::sleep(Duration::from_millis(1)).await;

    Ok(processed)
}}
"#, i, i, i)
    }).collect();

    let content_stream = iter(content_lines);

    println!("🔄 Starting streaming processing...");
    let start_time = std::time::Instant::now();

    let result_stream = orchestrator.process_stream_job(job, content_stream).await?;
    let mut stream = Box::pin(result_stream);

    let mut processed_chunks = 0;
    let mut total_processing_time = Duration::ZERO;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk_summary) => {
                processed_chunks += 1;
                total_processing_time += chunk_summary.processing_time;

                if processed_chunks <= 5 {
                    println!("📝 Chunk {}: '{}' (confidence: {:.2})",
                        chunk_summary.chunk_id,
                        chunk_summary.summary.chars().take(50).collect::<String>(),
                        chunk_summary.confidence);
                }
            }
            Err(e) => {
                println!("❌ Stream chunk failed: {}", e);
            }
        }
    }

    let total_time = start_time.elapsed();

    println!("✅ Streaming completed:");
    println!("   Chunks processed: {}", processed_chunks);
    println!("   Total streaming time: {:?}", total_time);
    println!("   Average chunk time: {:?}", total_processing_time / processed_chunks as u32);

    Ok(())
}

/// Example 4: Performance monitoring and health checks
async fn monitoring_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏥 Example 4: Monitoring and Health Checks");

    let orchestrator = create_orchestrator().await?;

    // Perform health check
    println!("🔍 Checking pipeline health...");
    let health = orchestrator.health_check().await?;

    println!("📊 Pipeline Health Status:");
    println!("   Status: {:?}", health.status);
    println!("   Inference engine: {:?}", health.inference_health);
    println!("   Database: {:?}", health.database_health);
    println!("   Available job slots: {}", health.available_jobs);
    println!("   Utilization rate: {:.1}%", health.utilization_rate * 100.0);

    // Get detailed metrics
    println!("\n📈 Retrieving detailed metrics...");
    let metrics = orchestrator.get_metrics().await?;

    println!("📊 System Metrics:");
    println!("   Active jobs: {}", metrics.active_jobs);
    println!("   Max concurrent jobs: {}", metrics.max_concurrent_jobs);
    println!("   Memory usage: {} MB", metrics.memory_usage_mb);
    println!("   Inference sessions: {}/{} available",
        metrics.inference_sessions.available_sessions,
        metrics.inference_sessions.total_sessions);
    println!("   System uptime: {:?}", metrics.uptime);

    // Run a benchmark if using real inference engine
    if let Ok(benchmark_results) = run_performance_benchmark(&orchestrator).await {
        println!("\n🏃 Performance Benchmark Results:");
        println!("   Average tokens/sec: {:.1}", benchmark_results.overall_performance.avg_tokens_per_second);
        println!("   Average latency: {:.1}ms", benchmark_results.overall_performance.avg_latency_ms);
        println!("   Success rate: {:.1}%", benchmark_results.overall_performance.success_rate * 100.0);

        if !benchmark_results.contract_violations.is_empty() {
            println!("⚠️  Contract violations detected:");
            for violation in &benchmark_results.contract_violations {
                println!("   - {:?}", violation);
            }
        } else {
            println!("✅ No contract violations");
        }
    }

    Ok(())
}

// Helper functions

async fn create_inference_engine() -> Result<std::sync::Arc<dyn InferenceEngine<Output = InferenceResult, Error = InferenceError>>, Box<dyn std::error::Error>> {
    // Try to create real inference engine first
    let model_path = PathBuf::from("./models/qwen2.5-0.5b-int4");
    let tokenizer_path = PathBuf::from("./tokenizer_dir");

    if model_path.join("model_quantized.onnx").exists() && tokenizer_path.join("tokenizer.json").exists() {
        println!("🤖 Using real ONNX inference engine");
        let engine = TraitInferenceEngine::new(model_path, tokenizer_path)?;
        Ok(std::sync::Arc::new(engine))
    } else {
        println!("🔧 Using mock inference engine (real model not found)");
        Ok(create_mock_inference_engine().await)
    }
}

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

    mock_engine
        .expect_model_info()
        .return_const(mock_model_info);

    mock_engine
        .expect_infer()
        .returning(|input| {
            let token_count = input.len() / 4;
            Ok(InferenceResult {
                content: format!("Mock summary: {} chars processed", input.len()),
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

    mock_engine
        .expect_infer_batch()
        .returning(|inputs, _options| {
            let results: Result<Vec<_>, _> = inputs.into_iter().map(|input| {
                let token_count = input.len() / 4;
                Ok(InferenceResult {
                    content: format!("Batch summary: {} chars", input.len()),
                    token_count,
                    confidence: 0.85,
                    processing_time_ms: 30,
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
            }).collect();
            results
        });

    std::sync::Arc::new(mock_engine)
}

async fn create_orchestrator() -> Result<PipelineOrchestrator, Box<dyn std::error::Error>> {
    let database_provider = std::sync::Arc::new(SimpleDatabaseProvider::new("sqlite::memory:")?);
    let inference_engine = create_inference_engine().await?;

    let config = PipelineConfig::default();

    Ok(PipelineOrchestrator::new(inference_engine, database_provider, config))
}

fn create_text_job(filename: &str, content: &str) -> PipelineJob {
    PipelineJob {
        job_id: JobId(Uuid::new_v4()),
        input: JobInput::Text {
            content: content.to_string(),
            filename: Some(filename.to_string()),
        },
        options: JobOptions {
            chunking_strategy: ChunkingStrategy::Lines { max_lines: 30 },
            inference_config: InferenceParams {
                temperature: Some(0.3),
                top_p: Some(0.8),
                top_k: Some(40),
                max_new_tokens: 60,
                min_length: 25,
                repetition_penalty: 1.1,
                stop_sequences: vec!["\n\n".to_string()],
                do_sample: true,
            },
            output_format: OutputFormat::Markdown,
            include_metadata: true,
        },
        created_at: chrono::Utc::now(),
    }
}

async fn run_performance_benchmark(
    orchestrator: &PipelineOrchestrator,
) -> Option<BenchmarkResults> {
    // This would run actual benchmarks if we have access to the inference engine
    // For now, return None to skip benchmarking
    None
}