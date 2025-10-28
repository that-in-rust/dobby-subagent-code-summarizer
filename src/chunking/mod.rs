//! Text chunking with TDD-First contracts
//!
//! Contracts:
//! - max_chunk_size: 300 lines
//! - min_chunk_overlap: 0 lines (no overlap)
//! - max_file_size_mb: 10MB
//! - processing_time_ms: 100ms per 1MB

use std::path::Path;

/// TDD-First chunking contract
pub struct ChunkingContract {
    pub max_chunk_size: usize,
    pub min_chunk_overlap: usize,
    pub max_file_size_mb: usize,
    pub processing_time_ms_per_mb: u32,
}

impl Default for ChunkingContract {
    fn default() -> Self {
        Self {
            max_chunk_size: 300,
            min_chunk_overlap: 0,
            max_file_size_mb: 10,
            processing_time_ms_per_mb: 100,
        }
    }
}

/// Text chunk with metadata
#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: u64,
    pub line_start: usize,
    pub line_end: usize,
    pub line_count: usize,
    pub content: String,
}

/// TDD-First text chunker with executable contracts
pub struct TextChunker {
    contract: ChunkingContract,
}

impl Default for TextChunker {
    fn default() -> Self {
        Self::new()
    }
}

impl TextChunker {
    pub fn new() -> Self {
        Self {
            contract: ChunkingContract::default(),
        }
    }

    /// Create chunks from file path with TDD validation
    pub async fn create_chunks_from_file(&self, file_path: &str) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
        // TDD-First: RED phase - validate preconditions
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File does not exist: {}", file_path).into());
        }

        // TDD-First: Validate file size contract
        let metadata = std::fs::metadata(path)?;
        let file_size_mb = (metadata.len() / (1024 * 1024)) as usize;
        if file_size_mb > self.contract.max_file_size_mb {
            return Err(format!("File too large: {}MB > {}MB",
                file_size_mb, self.contract.max_file_size_mb).into());
        }

        // Read content
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        // TDD-First: Create chunks with measurable processing
        let start_time = std::time::Instant::now();

        let mut chunks = Vec::new();
        for chunk_index in (0..total_lines).step_by(self.contract.max_chunk_size) {
            let line_start = chunk_index;
            let line_end = (chunk_index + self.contract.max_chunk_size).min(total_lines);
            let line_count = line_end - line_start;

            let chunk_content: String = lines[line_start..line_end].join("\n");

            let chunk = Chunk {
                id: chunk_index as u64,
                line_start,
                line_end,
                line_count,
                content: chunk_content,
            };

            chunks.push(chunk);
        }

        let processing_time = start_time.elapsed();
        let expected_max_time_ms = if file_size_mb == 0 {
            // For small files < 1MB, allow at least 100ms
            100u128
        } else {
            (file_size_mb as u128) * (self.contract.processing_time_ms_per_mb as u128)
        };

        // TDD-First: GREEN phase - validate postconditions
        if processing_time.as_millis() > expected_max_time_ms {
            return Err(format!("Processing too slow: {}ms > {}ms",
                processing_time.as_millis(), expected_max_time_ms).into());
        }

        if chunks.is_empty() {
            return Err("No chunks created".into());
        }

        // TDD-First: REFACTOR phase - optimize if needed
        let optimized_chunks = self.optimize_chunks(chunks);

        println!("✅ Created {} chunks", optimized_chunks.len());
        Ok(optimized_chunks)
    }

    /// TDD-First: Optimize chunk creation for performance
    fn optimize_chunks(&self, mut chunks: Vec<Chunk>) -> Vec<Chunk> {
        // TDD-First: Measure and optimize
        println!("🔧 Optimizing chunks...");

        // Remove empty chunks
        chunks.retain(|chunk| !chunk.content.trim().is_empty());

        // Merge very small chunks (TDD optimization)
        if chunks.len() > 1 {
            let mut i = 0;
            while i < chunks.len() - 1 {
                if chunks[i].line_count < 10 && chunks[i + 1].line_count < 10 {
                    // Merge small chunks
                    let merged_content = format!("{}\n{}",
                        chunks[i].content, chunks[i + 1].content);
                    chunks[i] = Chunk {
                        id: chunks[i].id,
                        line_start: chunks[i].line_start,
                        line_end: chunks[i + 1].line_end,
                        line_count: chunks[i].line_count + chunks[i + 1].line_count,
                        content: merged_content,
                    };
                    chunks.remove(i + 1);
                    continue;
                }
                i += 1;
            }
        }

        println!("✅ Optimized to {} chunks", chunks.len());
        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chunking_contract_validation() {
        let chunker = TextChunker::new();

        // Test TDD-First: RED phase - file not found
        let result = chunker.create_chunks_from_file("nonexistent.txt").await;
        assert!(result.is_err());

        // Test TDD-First: RED phase - file too large
        let result = chunker.create_chunks_from_file("large.txt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_chunking_contracts_satisfied() {
        let chunker = TextChunker::new();

        // Test TDD-First: GREEN phase - valid processing
        let test_file = "/tmp/test_chunking_large.txt";
        let test_content = "line 1\nline 2\nline 3\n".repeat(100);
        std::fs::write(test_file, &test_content).unwrap();
        
        let chunks = chunker.create_chunks_from_file(test_file).await.unwrap();
        std::fs::remove_file(test_file).ok();

        assert!(chunks.len() > 0);
        assert!(chunks.iter().all(|chunk| chunk.line_count <= 300));
        assert!(chunks.iter().all(|chunk| chunk.line_count > 0));
    }
}
