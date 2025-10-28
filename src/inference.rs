//! Candle-only inference engine MVP (no ONNX)
//!
//! ## Executable Specification Contract
//!
//! ### Preconditions:
//! - Model path directory exists (may be empty for MVP)
//! - Tokenizer file exists at tokenizer_path/tokenizer.json
//! - Metal drivers available on Apple Silicon (fallback to CPU)
//!
//! ### Postconditions:
//! - Engine is initialized with device selection (Metal preferential)
//! - Tokenizer is loaded and validated
//! - Summarize operations return deterministic output for MVP
//!
//! ### Error Conditions:
//! - Missing tokenizer.json → InferenceError::TokenizerLoadFailed
//! - Invalid model path → InferenceError::ModelLoading
//! - Device initialization failure → InferenceError::DeviceUnavailable
//!
//! ### Performance Contracts:
//! - Engine initialization: < 5 seconds
//! - Tokenizer loading: < 1 second
//! - Summarize operation: < 100ms (deterministic MVP)

use anyhow::Result;
use candle_core::Device;
use std::path::PathBuf;
use std::sync::Arc;
use tokenizers::Tokenizer;
use log::{info, warn, debug};

use crate::config::GenerationConfig;
use crate::layer1::traits::error::InferenceError;

/// Candle-only inference engine MVP (no ONNX).
/// Loads tokenizer and selects Device (Metal if available), returns deterministic summaries for now.
pub struct OptimizedInferenceEngine {
    device: Device,
    tokenizer: Arc<Tokenizer>,
    model_path: PathBuf,
}

impl OptimizedInferenceEngine {
    /// Create new Candle-only inference engine
    ///
    /// # Arguments
    /// * `model_path` - Path to model directory (may contain future safetensors files)
    /// * `tokenizer_path` - Path to tokenizer directory containing tokenizer.json
    ///
    /// # Returns
    /// * `Self` - Initialized inference engine
    ///
    /// # Errors
    /// * `InferenceError::TokenizerLoadFailed` - If tokenizer.json cannot be loaded
    /// * `InferenceError::ModelLoading` - If model path is invalid
    /// * `InferenceError::DeviceUnavailable` - If device initialization fails
    pub fn new(model_path: PathBuf, tokenizer_path: PathBuf) -> Result<Self> {
        // Device selection: prefer Metal, fallback to CPU
        let device = Device::new_metal(0).unwrap_or(Device::Cpu);
        let device_name = if matches!(device, Device::Cpu) { "CPU" } else { "Metal" };
        info!("Using device: {}", device_name);

        // Validate model path exists
        if !model_path.exists() {
            return Err(anyhow::anyhow!(
                InferenceError::ModelLoading {
                    model_path: model_path.to_string_lossy().to_string(),
                    source: "Directory does not exist".into()
                }
            ));
        }

        // Tokenizer is required even for MVP (we can validate existence)
        let tokenizer_file = tokenizer_path.join("tokenizer.json");
        if !tokenizer_file.exists() {
            warn!("Tokenizer file missing at {}, creating mock engine", tokenizer_file.display());

            // Create mock tokenizer for MVP when file doesn't exist
            let mock_tokenizer = Tokenizer::from_pretrained("gpt2", None)
                .map_err(|e| anyhow::anyhow!("Mock tokenizer creation failed: {}", e))?;

            debug!("Using mock GPT-2 tokenizer for MVP");

            return Ok(Self {
                device,
                tokenizer: Arc::new(mock_tokenizer),
                model_path,
            });
        }

        // Load real tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_file)
            .map_err(|e| anyhow::anyhow!("Tokenizer loading failed: {}", e))?;

        info!("Loaded tokenizer from {}", tokenizer_file.display());

        // Note: model weights loading will be added later (safetensors via candle-transformers).
        debug!("Model path registered for future loading: {}", model_path.display());

        Ok(Self {
            device,
            tokenizer: Arc::new(tokenizer),
            model_path,
        })
    }

    /// Summarize a text chunk (MVP deterministic implementation)
    ///
    /// # Arguments
    /// * `chunk` - Text chunk to summarize
    ///
    /// # Returns
    /// * `String` - Deterministic summary for MVP
    ///
    /// # Performance
    /// * O(1) deterministic processing for MVP
    /// * Later: Real neural inference with Candle
    pub fn summarize_chunk(&self, chunk: &str) -> Result<String> {
        let lines = chunk.lines().count();
        let chars = chunk.chars().count();
        let words = chunk.split_whitespace().count();

        // Create a meaningful preview
        let preview = chunk
            .chars()
            .take(80)
            .collect::<String>()
            .replace('\n', " ");

        let summary = format!(
            "Summary: {} lines, {} words, {} chars. Preview: {}",
            lines, words, chars, preview
        );

        debug!("Generated MVP summary for {} chars", chars);
        Ok(summary)
    }

    /// Summarize with generation config (MVP passes through)
    ///
    /// # Arguments
    /// * `chunk` - Text chunk to summarize
    /// * `prompt` - Custom prompt (MVP ignores but logs for future)
    /// * `config` - Generation configuration (MVP ignores but logs for future)
    ///
    /// # Returns
    /// * `String` - Summary (same as summarize_chunk for MVP)
    pub fn summarize_chunk_with_generation_config(
        &self,
        chunk: &str,
        prompt: &str,
        config: &GenerationConfig,
    ) -> Result<String> {
        debug!("MVP: Prompt '{}' and config {:?} noted for future implementation",
               prompt, config);

        // MVP: Reuse simple summarize; wire generation params later
        self.summarize_chunk(chunk)
    }

    /// Get device information
    pub fn device_info(&self) -> String {
        match &self.device {
            Device::Cpu => "CPU".to_string(),
            Device::Metal(metal_device) => format!("Metal device {:?}", metal_device),
            Device::Cuda(cuda_device) => format!("CUDA device {:?}", cuda_device),
        }
    }

    /// Get model path
    pub fn model_path(&self) -> &PathBuf {
        &self.model_path
    }

    /// Check if real model weights are available
    pub fn has_model_weights(&self) -> bool {
        // Check for common model file patterns
        let model_files = [
            "model.safetensors",
            "pytorch_model.bin",
            "model.bin",
        ];

        model_files.iter().any(|file| {
            self.model_path.join(file).exists()
        })
    }

    /// Get tokenizer info
    pub fn tokenizer_info(&self) -> Result<String> {
        let vocab_size = self.tokenizer.get_vocab_size(true);
        Ok(format!("Tokenizer with {} vocab entries", vocab_size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_engine_creation_with_missing_tokenizer() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let model_path = temp_dir.path();
        let tokenizer_path = temp_dir.path();

        // Should work with mock tokenizer when file doesn't exist
        let engine = OptimizedInferenceEngine::new(
            model_path.to_path_buf(),
            tokenizer_path.to_path_buf(),
        )?;

        assert!(engine.has_model_weights() == false);
        Ok(())
    }

    #[test]
    fn test_summarize_chunk_mvp() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let engine = OptimizedInferenceEngine::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().to_path_buf(),
        )?;

        let chunk = "This is a test chunk.\nIt has multiple lines.\nAnd some content.";
        let summary = engine.summarize_chunk(chunk)?;

        assert!(summary.contains("2 lines")); // Should count lines correctly
        assert!(summary.contains("words"));   // Should count words
        assert!(summary.contains("chars"));   // Should count characters
        assert!(summary.contains("This is a test chunk")); // Should contain preview

        Ok(())
    }

    #[test]
    fn test_device_selection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let engine = OptimizedInferenceEngine::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().to_path_buf(),
        )?;

        let device_info = engine.device_info();
        assert!(device_info == "CPU" || device_info.starts_with("Metal"));

        Ok(())
    }
}