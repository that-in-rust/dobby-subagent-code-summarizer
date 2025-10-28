//! Configuration structures for generation and model settings

use std::path::PathBuf;
use clap::ValueEnum;

/// Strategy for text generation
#[derive(Debug, Clone, ValueEnum)]
pub enum SamplingStrategy {
    #[value(name = "sampling")]
    Sampling,
    #[value(name = "beam")]
    Beam,
}

/// Generation configuration parameters
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub strategy: SamplingStrategy,

    // Sampling parameters
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: usize,

    // Beam search parameters
    pub num_beams: usize,
    pub length_penalty: f32,
    pub early_stopping: bool,

    // Universal generation controls
    pub max_new_tokens: usize,
    pub min_length: usize,
    pub repetition_penalty: f32,
    pub no_repeat_ngram_size: usize,
    pub stop_sequences: Vec<String>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            strategy: SamplingStrategy::Sampling,
            temperature: 0.35,
            top_p: 0.85,
            top_k: 40,
            num_beams: 3,
            length_penalty: 1.05,
            early_stopping: false,
            max_new_tokens: 60,
            min_length: 35,
            repetition_penalty: 1.15,
            no_repeat_ngram_size: 3,
            stop_sequences: vec!["\n\n".to_string()],
        }
    }
}

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub name: String,
    pub model_path: PathBuf,
    pub tokenizer_path: Option<PathBuf>,
}

impl ModelConfig {
    pub fn new(name: String, model_path: PathBuf, tokenizer_path: Option<PathBuf>) -> Self {
        Self {
            name,
            model_path,
            tokenizer_path,
        }
    }

    /// Resolve default model paths based on model name
    pub fn from_name(name: &str, custom_path: Option<PathBuf>, tokenizer_path: Option<PathBuf>) -> Self {
        let model_path = custom_path.unwrap_or_else(|| match name {
            "qwen2.5-0.5b-int4" => PathBuf::from("./models/qwen2.5-0.5b-int4"),
            "smollm2-135m" => PathBuf::from("./models/smollm2-135m"),
            "smollm2-360m" => PathBuf::from("./models/smollm2-360m"),
            _ => PathBuf::from(format!("./models/{}", name)),
        });

        Self {
            name: name.to_string(),
            model_path,
            tokenizer_path,
        }
    }

    /// Get tokenizer path (model_path/tokenizer_dir if not specified)
    pub fn tokenizer_path(&self) -> PathBuf {
        self.tokenizer_path.clone().unwrap_or_else(|| {
            match self.name.as_str() {
                "qwen2.5-0.5b-int4" => PathBuf::from("./tokenizer_dir"),
                _ => self.model_path.join("tokenizer"),
            }
        })
    }
}

/// System configuration with executable contracts
#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub tokio_source_path: PathBuf,
    pub model_path: PathBuf,
    pub max_concurrent_sessions: usize,
    pub max_memory_mb: usize,
    pub chunk_size: usize,
    pub max_chunk_processing_time_s: u64,
    pub min_parallel_efficiency: f32,
}

impl SystemConfig {
    /// Production configuration with validated contracts
    pub fn production() -> Self {
        Self {
            tokio_source_path: PathBuf::from("/Users/amuldotexe/Projects/transfiguration/tokio-rs-tokio-8a5edab282632443.txt"),
            model_path: PathBuf::from("models/model.safetensors"),
            max_concurrent_sessions: 20,
            max_memory_mb: 4096,
            chunk_size: 300,
            max_chunk_processing_time_s: 2,
            min_parallel_efficiency: 0.8,
        }
    }

    /// Test configuration for unit tests
    pub fn test_config() -> Self {
        Self {
            tokio_source_path: PathBuf::from("test_data/sample.txt"),
            model_path: PathBuf::from("models/model.safetensors"),
            max_concurrent_sessions: 4,
            max_memory_mb: 1024,
            chunk_size: 50,
            max_chunk_processing_time_s: 5,
            min_parallel_efficiency: 0.7,
        }
    }

    /// Performance test configuration
    pub fn performance_test_config() -> Self {
        Self {
            tokio_source_path: PathBuf::from("test_data/large.txt"),
            model_path: PathBuf::from("models/model.safetensors"),
            max_concurrent_sessions: 20,
            max_memory_mb: 4096,
            chunk_size: 300,
            max_chunk_processing_time_s: 2,
            min_parallel_efficiency: 0.8,
        }
    }

    /// Validate configuration contracts
    pub fn validate(&self) -> Result<(), String> {
        if self.max_concurrent_sessions == 0 || self.max_concurrent_sessions > 100 {
            return Err(format!(
                "Invalid max_concurrent_sessions: {} (must be 1-100)",
                self.max_concurrent_sessions
            ));
        }

        if self.chunk_size == 0 || self.chunk_size > 1000 {
            return Err(format!(
                "Invalid chunk_size: {} (must be 1-1000)",
                self.chunk_size
            ));
        }

        if self.min_parallel_efficiency < 0.0 || self.min_parallel_efficiency > 1.0 {
            return Err(format!(
                "Invalid min_parallel_efficiency: {} (must be 0.0-1.0)",
                self.min_parallel_efficiency
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation_contracts() {
        // TDD-First: GREEN phase - valid configs
        let config = SystemConfig::production();
        assert!(config.validate().is_ok());

        let test_config = SystemConfig::test_config();
        assert!(test_config.validate().is_ok());
    }

    #[test]
    fn test_config_contract_violations() {
        // TDD-First: RED phase - invalid configs
        let mut config = SystemConfig::production();
        
        config.max_concurrent_sessions = 0;
        assert!(config.validate().is_err());
        
        config.max_concurrent_sessions = 150;
        assert!(config.validate().is_err());
    }
}
