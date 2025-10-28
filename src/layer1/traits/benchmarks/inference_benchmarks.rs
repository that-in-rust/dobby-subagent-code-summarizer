//! Performance Benchmarking for InferenceEngine Trait Contracts
//!
//! Uses Criterion for microbenchmarking and validates performance contracts
//! Follows Rust benchmarking idioms with statistical significance

use crate::layer1::traits::inference::*;
use crate::layer1::traits::implementations::inference_engine::TraitInferenceEngine;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

/// Benchmark configuration with different input sizes
struct BenchmarkConfig {
    small_input: String,
    medium_input: String,
    large_input: String,
    batch_sizes: Vec<usize>,
    concurrent_requests: Vec<usize>,
}

impl BenchmarkConfig {
    fn new() -> Self {
        Self {
            small_input: "fn hello() { println!(\"Hello\"); }".to_string(),
            medium_input: r#"
pub async fn process_data(data: &[String]) -> Result<Vec<String>, Error> {
    let futures: Vec<_> = data.iter()
        .enumerate()
        .map(|(i, item)| async move {
            format!("Processed item {}: {}", i, item.to_uppercase())
        })
        .collect();

    let results = futures::future::try_join_all(futures).await?;
    Ok(results)
}
"#.to_string(),
            large_input: include_str!("../../../../../tests/fixtures/large_code_sample.rs").to_string(),
            batch_sizes: vec![1, 5, 10, 20],
            concurrent_requests: vec![1, 2, 4, 8, 16],
        }
    }
}

/// Get or create inference engine for benchmarking
async fn get_benchmark_engine() -> Option<TraitInferenceEngine> {
    let model_path = PathBuf::from("./models/qwen2.5-0.5b-int4");
    let tokenizer_path = PathBuf::from("./tokenizer_dir");

    // Check if model files exist
    if model_path.join("model_quantized.onnx").exists() &&
       tokenizer_path.join("tokenizer.json").exists() {
        match TraitInferenceEngine::new(model_path, tokenizer_path) {
            Ok(engine) => Some(engine),
            Err(e) => {
                eprintln!("Failed to create engine for benchmarking: {}", e);
                None
            }
        }
    } else {
        eprintln!("Model files not found for benchmarking, skipping performance tests");
        None
    }
}

/// Benchmark single inference latency
fn benchmark_single_inference(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    if let Some(engine) = rt.block_on(get_benchmark_engine()) {
        let config = BenchmarkConfig::new();

        let mut group = c.benchmark_group("single_inference");

        // Small input benchmark
        group.bench_function("small_input", |b| {
            b.to_async(&rt).iter(|| async {
                let result = engine.infer(black_box(config.small_input.clone())).await;
                black_box(result)
            })
        });

        // Medium input benchmark
        group.bench_function("medium_input", |b| {
            b.to_async(&rt).iter(|| async {
                let result = engine.infer(black_box(config.medium_input.clone())).await;
                black_box(result)
            })
        });

        // Large input benchmark
        group.bench_function("large_input", |b| {
            b.to_async(&rt).iter(|| async {
                let result = engine.infer(black_box(config.large_input.clone())).await;
                black_box(result)
            })
        });

        group.finish();
    }
}

/// Benchmark batch inference throughput
fn benchmark_batch_inference(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    if let Some(engine) = rt.block_on(get_benchmark_engine()) {
        let config = BenchmarkConfig::new();
        let test_input = config.medium_input.clone();

        let mut group = c.benchmark_group("batch_inference");

        for &batch_size in &config.batch_sizes {
            let inputs: Vec<String> = (0..batch_size)
                .map(|i| format!("// Batch item {}\n{}", i, test_input))
                .collect();

            group.throughput(Throughput::Elements(batch_size as u64));
            group.bench_with_input(
                BenchmarkId::new("throughput", batch_size),
                &batch_size,
                |b, _| {
                    b.to_async(&rt).iter(|| async {
                        let options = BatchOptions {
                            max_batch_size: batch_size,
                            parallel_sessions: batch_size,
                            timeout: Duration::from_secs(10),
                            fail_fast: false,
                        };

                        let result = engine.infer_batch(black_box(inputs.clone()), black_box(options)).await;
                        black_box(result)
                    })
                },
            );
        }

        group.finish();
    }
}

/// Benchmark concurrent inference performance
fn benchmark_concurrent_inference(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    if let Some(engine) = rt.block_on(get_benchmark_engine()) {
        let config = BenchmarkConfig::new();
        let test_input = config.small_input.clone();

        let mut group = c.benchmark_group("concurrent_inference");

        for &concurrent_count in &config.concurrent_requests {
            group.throughput(Throughput::Elements(concurrent_count as u64));
            group.bench_with_input(
                BenchmarkId::new("concurrent", concurrent_count),
                &concurrent_count,
                |b, _| {
                    b.to_async(&rt).iter(|| async {
                        let mut handles = Vec::new();

                        for _ in 0..concurrent_count {
                            let engine_clone = engine.clone();
                            let input_clone = test_input.clone();
                            let handle = tokio::spawn(async move {
                                engine_clone.infer(black_box(input_clone)).await
                            });
                            handles.push(handle);
                        }

                        let mut results = Vec::new();
                        for handle in handles {
                            match handle.await {
                                Ok(result) => results.push(result),
                                Err(e) => eprintln!("Task failed: {}", e),
                            }
                        }

                        black_box(results)
                    })
                },
            );
        }

        group.finish();
    }
}

/// Benchmark streaming inference
fn benchmark_streaming_inference(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    if let Some(engine) = rt.block_on(get_benchmark_engine()) {
        let mut group = c.benchmark_group("streaming_inference");

        // Small stream benchmark
        group.bench_function("small_stream", |b| {
            b.to_async(&rt).iter(|| async {
                use futures::StreamExt;
                use tokio_stream::iter;

                let inputs: Vec<String> = (0..10)
                    .map(|i| format!("fn item_{}() {{ return {}; }}", i, i))
                    .collect();

                let input_stream = iter(inputs);
                let result_stream = engine.infer_stream(input_stream).await.unwrap();

                let results: Vec<_> = result_stream.collect().await;
                black_box(results)
            })
        });

        // Large stream benchmark
        group.bench_function("large_stream", |b| {
            b.to_async(&rt).iter(|| async {
                use futures::StreamExt;
                use tokio_stream::iter;

                let inputs: Vec<String> = (0..100)
                    .map(|i| format!("fn item_{}() {{ return {}; }}", i, i))
                    .collect();

                let input_stream = iter(inputs);
                let result_stream = engine.infer_stream(input_stream).await.unwrap();

                let results: Vec<_> = result_stream.collect().await;
                black_box(results)
            })
        });

        group.finish();
    }
}

/// Benchmark memory usage patterns
fn benchmark_memory_usage(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    if let Some(engine) = rt.block_on(get_benchmark_engine()) {
        let config = BenchmarkConfig::new();

        let mut group = c.benchmark_group("memory_usage");

        group.bench_function("memory_pressure", |b| {
            b.to_async(&rt).iter(|| async {
                let initial_memory = memory_stats::memory_stats()
                    .map(|stats| stats.physical_mem)
                    .unwrap_or(0);

                // Run multiple inferences to test memory pressure
                let mut results = Vec::new();
                for i in 0..20 {
                    let input = format!("// Memory test {}\n{}", i, config.medium_input);
                    let result = engine.infer(input).await;
                    results.push(result);
                }

                let final_memory = memory_stats::memory_stats()
                    .map(|stats| stats.physical_mem)
                    .unwrap_or(0);

                let memory_delta = final_memory.saturating_sub(initial_memory);

                black_box((results, memory_delta))
            })
        });

        group.finish();
    }
}

/// Performance contract validation benchmarks
fn benchmark_contract_validation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    if let Some(engine) = rt.block_on(get_benchmark_engine()) {
        let mut group = c.benchmark_group("contract_validation");

        // Latency contract benchmark
        group.bench_function("latency_contract", |b| {
            b.to_async(&rt).iter(|| async {
                let test_cases = vec![
                    BenchmarkCase {
                        name: "small_case".to_string(),
                        input: "fn test() { return 42; }".to_string(),
                        expected_tokens: 20,
                        max_latency: Duration::from_millis(100),
                        min_confidence: 0.8,
                    },
                    BenchmarkCase {
                        name: "medium_case".to_string(),
                        input: r#"
pub async fn process_items(items: Vec<i32>) -> Result<Vec<i32>, Error> {
    let mut results = Vec::new();
    for item in items {
        if item > 0 {
            results.push(item * 2);
        } else {
            results.push(item.abs());
        }
    }
    Ok(results)
}
"#.to_string(),
                        expected_tokens: 50,
                        max_latency: Duration::from_millis(300),
                        min_confidence: 0.7,
                    },
                ];

                let benchmark_results = engine.benchmark(&test_cases).await;
                black_box(benchmark_results)
            })
        });

        // Throughput contract benchmark
        group.bench_function("throughput_contract", |b| {
            b.to_async(&rt).iter(|| async {
                let test_cases = vec![
                    BenchmarkCase {
                        name: "throughput_test".to_string(),
                        input: "fn throughput() { /* test */ }".to_string(),
                        expected_tokens: 30,
                        max_latency: Duration::from_millis(50),
                        min_confidence: 0.9,
                    },
                ];

                let benchmark_results = engine.benchmark(&test_cases).await;
                black_box(benchmark_results)
            })
        });

        group.finish();
    }
}

/// Custom criterion configuration for production-like benchmarking
fn configure_criterion() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(5))
        .sample_size(50) // More samples for statistical significance
        .noise_threshold(0.02) // 2% noise threshold
        .confidence_level(0.95) // 95% confidence level
}

criterion_group! {
    name = inference_benchmarks;
    config = configure_criterion();
    targets = benchmark_single_inference,
              benchmark_batch_inference,
              benchmark_concurrent_inference,
              benchmark_streaming_inference,
              benchmark_memory_usage,
              benchmark_contract_validation
}

criterion_main!(inference_benchmarks);

/// Post-benchmark analysis utilities
pub mod analysis {
    use super::*;
    use std::collections::HashMap;

    /// Analyze benchmark results for contract compliance
    pub fn analyze_contract_compliance(results: &BenchmarkResults) -> ContractAnalysis {
        let mut analysis = ContractAnalysis::new();

        // Analyze latency contracts
        for violation in &results.contract_violations {
            match violation {
                ContractViolation::Latency { case_name, actual, required } => {
                    analysis.add_latency_violation(case_name.clone(), *actual, *required);
                }
                ContractViolation::Throughput { case_name, actual, required } => {
                    analysis.add_throughput_violation(case_name.clone(), *actual, *required);
                }
                ContractViolation::Confidence { case_name, actual, required } => {
                    analysis.add_confidence_violation(case_name.clone(), *actual, *required);
                }
                ContractViolation::Memory { case_name, actual, required } => {
                    analysis.add_memory_violation(case_name.clone(), *actual, *required);
                }
            }
        }

        // Calculate overall compliance score
        let total_cases = results.test_cases.len();
        let successful_cases = results.test_cases.iter()
            .filter(|tc| tc.passed_contracts)
            .count();

        analysis.overall_compliance = successful_cases as f64 / total_cases as f64;

        analysis
    }

    #[derive(Debug)]
    pub struct ContractAnalysis {
        pub latency_violations: HashMap<String, (Duration, Duration)>,
        pub throughput_violations: HashMap<String, (f64, f64)>,
        pub confidence_violations: HashMap<String, (f64, f64)>,
        pub memory_violations: HashMap<String, (usize, usize)>,
        pub overall_compliance: f64,
    }

    impl ContractAnalysis {
        fn new() -> Self {
            Self {
                latency_violations: HashMap::new(),
                throughput_violations: HashMap::new(),
                confidence_violations: HashMap::new(),
                memory_violations: HashMap::new(),
                overall_compliance: 0.0,
            }
        }

        fn add_latency_violation(&mut self, case: String, actual: Duration, required: Duration) {
            self.latency_violations.insert(case, (actual, required));
        }

        fn add_throughput_violation(&mut self, case: String, actual: f64, required: f64) {
            self.throughput_violations.insert(case, (actual, required));
        }

        fn add_confidence_violation(&mut self, case: String, actual: f64, required: f64) {
            self.confidence_violations.insert(case, (actual, required));
        }

        fn add_memory_violation(&mut self, case: String, actual: usize, required: usize) {
            self.memory_violations.insert(case, (actual, required));
        }

        pub fn generate_report(&self) -> String {
            let mut report = String::new();

            report.push_str("=== Contract Compliance Analysis ===\n");
            report.push_str(&format!("Overall Compliance: {:.2}%\n\n", self.overall_compliance * 100.0));

            if !self.latency_violations.is_empty() {
                report.push_str("Latency Violations:\n");
                for (case, (actual, required)) in &self.latency_violations {
                    report.push_str(&format!("  {}: {:?} > {:?}\n", case, actual, required));
                }
                report.push('\n');
            }

            if !self.throughput_violations.is_empty() {
                report.push_str("Throughput Violations:\n");
                for (case, (actual, required)) in &self.throughput_violations {
                    report.push_str(&format!("  {}: {:.2} < {:.2}\n", case, actual, required));
                }
                report.push('\n');
            }

            if !self.confidence_violations.is_empty() {
                report.push_str("Confidence Violations:\n");
                for (case, (actual, required)) in &self.confidence_violations {
                    report.push_str(&format!("  {}: {:.3} < {:.3}\n", case, actual, required));
                }
                report.push('\n');
            }

            if !self.memory_violations.is_empty() {
                report.push_str("Memory Violations:\n");
                for (case, (actual, required)) in &self.memory_violations {
                    report.push_str(&format!("  {}: {} MB > {} MB\n", case, actual, required));
                }
            }

            report
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::analysis::*;

    #[test]
    fn test_contract_analysis() {
        let mut results = BenchmarkResults {
            test_cases: vec![
                BenchmarkResult {
                    case_name: "test_case_1".to_string(),
                    input_length: 100,
                    output_length: 50,
                    processing_time: Duration::from_millis(200),
                    tokens_per_second: 25.0,
                    confidence: 0.7,
                    memory_usage_mb: 1024,
                    passed_contracts: false,
                },
            ],
            overall_performance: PerformanceSummary {
                avg_tokens_per_second: 25.0,
                avg_latency_ms: 200.0,
                avg_confidence: 0.7,
                avg_memory_mb: 1024.0,
                success_rate: 0.0,
            },
            contract_violations: vec![
                ContractViolation::Latency {
                    case_name: "test_case_1".to_string(),
                    actual: Duration::from_millis(200),
                    required: Duration::from_millis(100),
                },
                ContractViolation::Confidence {
                    case_name: "test_case_1".to_string(),
                    actual: 0.7,
                    required: 0.8,
                },
            ],
        };

        let analysis = analyze_contract_compliance(&results);

        assert_eq!(analysis.overall_compliance, 0.0);
        assert_eq!(analysis.latency_violations.len(), 1);
        assert_eq!(analysis.confidence_violations.len(), 1);

        let report = analysis.generate_report();
        assert!(report.contains("Overall Compliance: 0.00%"));
        assert!(report.contains("Latency Violations"));
        assert!(report.contains("Confidence Violations"));
    }
}