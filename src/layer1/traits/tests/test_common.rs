//! Common test utilities for trait testing

use std::time::{Duration, Instant};
use uuid::Uuid;

/// Performance measurement helper
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    pub duration: Duration,
    pub memory_before: usize,
    pub memory_after: usize,
    pub memory_peak: usize,
    pub success: bool,
    pub error_message: Option<String>,
}

impl PerformanceMeasurement {
    pub fn new() -> Self {
        Self {
            duration: Duration::ZERO,
            memory_before: 0,
            memory_after: 0,
            memory_peak: 0,
            success: false,
            error_message: None,
        }
    }

    pub fn memory_increase(&self) -> usize {
        self.memory_after.saturating_sub(self.memory_before)
    }

    pub fn memory_efficiency(&self, baseline: usize) -> f64 {
        if baseline == 0 {
            return 1.0;
        }
        let efficiency = 1.0 - (self.memory_increase() as f64 / baseline as f64);
        efficiency.max(0.0).min(1.0)
    }
}

/// Performance timer helper
pub struct PerformanceTimer {
    start_time: Instant,
    initial_memory: usize,
}

impl PerformanceTimer {
    pub fn start() -> Self {
        Self {
            start_time: Instant::now(),
            initial_memory: get_memory_usage(),
        }
    }

    pub fn measure<T, F, E>(&mut self, operation: F) -> Result<PerformanceMeasurement, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let peak_before = get_memory_usage();

        match operation() {
            Ok(_) => {
                let duration = self.start_time.elapsed();
                let memory_after = get_memory_usage();
                let memory_peak = peak_before.max(memory_after);

                Ok(PerformanceMeasurement {
                    duration,
                    memory_before: self.initial_memory,
                    memory_after,
                    memory_peak,
                    success: true,
                    error_message: None,
                })
            }
            Err(e) => {
                let duration = self.start_time.elapsed();
                let memory_after = get_memory_usage();
                let memory_peak = peak_before.max(memory_after);

                Ok(PerformanceMeasurement {
                    duration,
                    memory_before: self.initial_memory,
                    memory_after,
                    memory_peak,
                    success: false,
                    error_message: Some(format!("{}", e)),
                })
            }
        }
    }

    pub async fn measure_async<T, F, E>(&mut self, operation: F) -> Result<PerformanceMeasurement, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let peak_before = get_memory_usage();

        match operation.await {
            Ok(_) => {
                let duration = self.start_time.elapsed();
                let memory_after = get_memory_usage();
                let memory_peak = peak_before.max(memory_after);

                Ok(PerformanceMeasurement {
                    duration,
                    memory_before: self.initial_memory,
                    memory_after,
                    memory_peak,
                    success: true,
                    error_message: None,
                })
            }
            Err(e) => {
                let duration = self.start_time.elapsed();
                let memory_after = get_memory_usage();
                let memory_peak = peak_before.max(memory_after);

                Ok(PerformanceMeasurement {
                    duration,
                    memory_before: self.initial_memory,
                    memory_after,
                    memory_peak,
                    success: false,
                    error_message: Some(format!("{}", e)),
                })
            }
        }
    }
}

/// Test data generators
pub mod generators {
    use super::*;
    use uuid::Uuid;

    /// Generate a unique test database connection string
    pub fn test_connection_string() -> String {
        format!("test://localhost/{}", Uuid::new_v4())
    }

    /// Generate test database records
    pub fn generate_test_records(count: usize) -> Vec<crate::layer1::traits::database::DatabaseRecord> {
        (0..count).map(|i| crate::layer1::traits::database::DatabaseRecord {
            id: crate::layer1::traits::database::RecordId(Uuid::new_v4()),
            content: crate::layer1::traits::database::Content::Text(
                format!("Test record content {}", i)
            ),
            metadata: crate::layer1::traits::database::RecordMetadata {
                source: "test_generator".to_string(),
                content_type: crate::layer1::traits::database::ContentType::Code,
                size_bytes: 100,
                processing_state: crate::layer1::traits::database::ProcessingState::Pending,
                priority: crate::layer1::traits::database::Priority::Normal,
                custom_fields: std::collections::HashMap::new(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }).collect()
    }

    /// Generate test inference inputs
    pub fn generate_test_inputs(count: usize) -> Vec<String> {
        (0..count).map(|i| {
            match i % 4 {
                0 => format!("What is the capital of country {}?", i),
                1 => format!("Explain the concept of topic {} in simple terms.", i),
                2 => format!("Write a function that implements algorithm {}.", i),
                _ => format!("Summarize the following text about subject {}: Lorem ipsum dolor sit amet.", i),
            }
        }).collect()
    }

    /// Generate test model configuration
    pub fn generate_test_model_config() -> crate::layer1::traits::inference::ModelConfig {
        crate::layer1::traits::inference::ModelConfig {
            model_path: format!("/test/models/model_{}.gguf", Uuid::new_v4()),
            model_name: format!("test-model-{}", Uuid::new_v4()),
            device: crate::layer1::traits::inference::DeviceConfig {
                device_type: crate::layer1::traits::inference::DeviceType::Cpu,
                fallback_enabled: true,
                memory_fraction: 0.8,
                enable_metal: false,
                force_cpu: false,
            },
            quantization: crate::layer1::traits::inference::QuantizationConfig {
                quantization_type: crate::layer1::traits::inference::QuantizationType::None,
                quantization_bits: None,
                model_format: "gguf".to_string(),
            },
            session_pool: crate::layer1::traits::inference::SessionPoolConfig {
                max_sessions: 4,
                session_timeout: Duration::from_secs(300),
                max_idle_time: Duration::from_secs(60),
                enable_session_reuse: true,
            },
            inference_params: crate::layer1::traits::inference::InferenceParams {
                temperature: Some(0.7),
                top_p: Some(0.9),
                top_k: Some(50),
                max_new_tokens: 512,
                min_length: 1,
                repetition_penalty: 1.1,
                stop_sequences: vec![],
                do_sample: true,
            },
            optimization: crate::layer1::traits::inference::OptimizationConfig {
                use_flash_attention: false,
                enable_kvcache: true,
                memory_efficient: true,
                compilation_target: "cpu".to_string(),
            },
        }
    }
}

/// Assertion helpers for performance contracts
pub mod assertions {
    use super::*;
    // use super::test_constants::*;  // Commented out to avoid compilation errors

    pub fn assert_performance_contract(
        measurement: &PerformanceMeasurement,
        timeout: Duration,
        operation_name: &str,
    ) {
        assert!(
            measurement.duration < timeout,
            "{}: Performance contract violated - took {}ms, limit is {}ms",
            operation_name,
            measurement.duration.as_millis(),
            timeout.as_millis()
        );

        assert!(
            measurement.success,
            "{}: Operation failed - {}",
            operation_name,
            measurement.error_message.as_deref().unwrap_or("unknown error")
        );
    }

    pub fn assert_memory_contract(
        measurement: &PerformanceMeasurement,
        max_memory_mb: usize,
        operation_name: &str,
    ) {
        let memory_mb = measurement.memory_increase() / 1024 / 1024;
        assert!(
            memory_mb <= max_memory_mb,
            "{}: Memory contract violated - used {}MB, limit is {}MB",
            operation_name,
            memory_mb,
            max_memory_mb
        );
    }

    pub fn assert_efficiency_contract(
        measurement: &PerformanceMeasurement,
        baseline: usize,
        min_efficiency: f64,
        operation_name: &str,
    ) {
        let efficiency = measurement.memory_efficiency(baseline);
        assert!(
            efficiency >= min_efficiency,
            "{}: Efficiency contract violated - {:.1}% efficiency, required {:.1}%",
            operation_name,
            efficiency * 100.0,
            min_efficiency * 100.0
        );
    }

    pub fn assert_throughput_contract(
        items_processed: usize,
        duration: Duration,
        min_throughput: f64,
        operation_name: &str,
    ) {
        let throughput = items_processed as f64 / duration.as_secs_f64();
        assert!(
            throughput >= min_throughput,
            "{}: Throughput contract violated - {:.1} items/sec, required {:.1} items/sec",
            operation_name,
            throughput,
            min_throughput
        );
    }
}

/// Memory monitoring placeholder
fn get_memory_usage() -> usize {
    // RED PHASE: Stub implementation
    // TODO: Implement real memory monitoring for:
    // - macOS: use mach task_info
    // - Linux: use /proc/self/status
    // - Windows: use GetProcessMemoryInfo
    0
}