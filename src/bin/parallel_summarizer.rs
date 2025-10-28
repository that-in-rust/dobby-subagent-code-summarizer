//! 20-Agent Parallel Code Summarizer CLI
//!
//! Real neural inference using 20 independent agents for maximum parallelism

use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use anyhow::Result;
use log::{info, error};
use env_logger::Env;
use chrono::Utc;

use dobby_subagent_code_summarizer::parallel_agents::{ParallelAgentSystem, ParallelConfig};
use dobby_subagent_code_summarizer::config::{GenerationConfig, ModelConfig, SamplingStrategy};

#[derive(Parser)]
#[command(name = "parallel_summarizer")]
#[command(about = "20-Agent Parallel Neural Code Summarizer with Advanced Generation Control")]
struct Args {
    // === REQUIRED INPUTS (NO DEFAULTS) ===
    #[arg(short, long, help = "Input code file to summarize (must exist)")]
    file: String,

    #[arg(long, help = "Absolute path for final summary output")]
    output_file: String,

    #[arg(long, help = "Absolute path for progress/results log")]
    results_file: String,

    // === PROCESSING PARAMETERS (NO DEFAULTS) ===
    #[arg(long, help = "Lines of code per chunk")]
    loc: usize,

    #[arg(long, help = "Custom prompt for summarization", conflicts_with = "prompt_file")]
    prompt: Option<String>,

    #[arg(long, help = "Absolute path to file containing prompt", conflicts_with = "prompt")]
    prompt_file: Option<String>,

    #[arg(long, help = "Number of parallel agents")]
    agent_count: usize,

    // === MODEL CONFIGURATION ===
    #[arg(long = "model-name", help = "Model identifier (qwen2.5-0.5b-int4, smollm2-135m, smollm2-360m, custom)")]
    model_name: String,

    #[arg(long = "model-path", help = "Custom model path (overrides default for model-name)")]
    model_path: Option<PathBuf>,

    #[arg(long, help = "Tokenizer directory path")]
    tokenizer_dir: Option<PathBuf>,

    // === GENERATION STRATEGY ===
    #[arg(long = "sampling-strategy", help = "Generation strategy", default_value = "sampling")]
    sampling_strategy: SamplingStrategy,

    // === SAMPLING PARAMETERS ===
    #[arg(long = "temperature", help = "Sampling temperature (0.0-2.0, lower = more deterministic)", default_value = "0.35")]
    temperature: f32,

    #[arg(long = "top-p", help = "Nucleus sampling threshold (0.0-1.0)", default_value = "0.85")]
    top_p: f32,

    #[arg(long = "top-k", help = "Top-k sampling limit (1-1000)", default_value = "40")]
    top_k: usize,

    // === BEAM SEARCH PARAMETERS ===
    #[arg(long = "num-beams", help = "Number of beam candidates (1-10)", default_value = "3")]
    num_beams: usize,

    #[arg(long = "length-penalty", help = "Length penalty for beam search (0.5-2.0)", default_value = "1.05")]
    length_penalty: f32,

    #[arg(long = "early-stopping", help = "Enable early stopping in beam search")]
    early_stopping: bool,

    // === UNIVERSAL GENERATION CONTROLS ===
    #[arg(long = "max-new-tokens", help = "Maximum tokens to generate (1-200)", default_value = "60")]
    max_new_tokens: usize,

    #[arg(long = "min-length", help = "Minimum summary length (1-100)", default_value = "35")]
    min_length: usize,

    #[arg(long = "repetition-penalty", help = "Repetition penalty (1.0-2.0)", default_value = "1.15")]
    repetition_penalty: f32,

    #[arg(long = "no-repeat-ngram-size", help = "Prevent n-gram repetition (0-10)", default_value = "3")]
    no_repeat_ngram_size: usize,

    #[arg(long = "stop-sequences", help = "Stop generation at these strings (comma-separated)", value_delimiter = ',')]
    stop_sequences: Vec<String>,

    // === SYSTEM PARAMETERS ===
    #[arg(long, help = "Maximum concurrent tasks")]
    max_concurrent: Option<usize>,
}


/// Validate all compulsory CLI arguments
fn validate_args(args: &Args) -> Result<(String, ModelConfig, GenerationConfig)> {
    let mut errors = Vec::new();

    // Validate input file exists
    if !Path::new(&args.file).exists() {
        errors.push(format!("Input file does not exist: {}", args.file));
    }

    // Validate absolute paths for output files
    if !args.output_file.starts_with('/') {
        errors.push(format!("--output-file must be absolute path (start with '/'), got: {}", args.output_file));
    }

    if !args.results_file.starts_with('/') {
        errors.push(format!("--results-file must be absolute path (start with '/'), got: {}", args.results_file));
    }

    // Validate prompt input (exactly one required)
    match (&args.prompt, &args.prompt_file) {
        (Some(_), Some(_)) => errors.push("Cannot use both --prompt and --prompt-file simultaneously".to_string()),
        (None, None) => errors.push("Either --prompt or --prompt-file is required".to_string()),
        (None, Some(file)) => {
            if !Path::new(file).exists() {
                errors.push(format!("Prompt file does not exist: {}", file));
            }
            if !Path::new(file).is_absolute() {
                errors.push(format!("--prompt-file must be absolute path, got: {}", file));
            }
        }
        _ => {} // Valid case: prompt provided
    }

    // Validate loc is reasonable
    if args.loc == 0 {
        errors.push("--loc must be greater than 0".to_string());
    }
    if args.loc > 50000 {
        errors.push("--loc should be less than 50000 lines for optimal processing".to_string());
    }

    // Validate agent count
    if args.agent_count == 0 {
        errors.push("--agent-count must be greater than 0".to_string());
    }
    if args.agent_count > 100 {
        errors.push("--agent-count should be less than 100 for system stability".to_string());
    }

    // Validate generation parameters
    if args.temperature < 0.0 || args.temperature > 2.0 {
        errors.push(format!("--temperature must be between 0.0 and 2.0, got: {}", args.temperature));
    }
    if args.top_p < 0.0 || args.top_p > 1.0 {
        errors.push(format!("--top-p must be between 0.0 and 1.0, got: {}", args.top_p));
    }
    if args.top_k == 0 || args.top_k > 1000 {
        errors.push(format!("--top-k must be between 1 and 1000, got: {}", args.top_k));
    }
    if args.num_beams == 0 || args.num_beams > 10 {
        errors.push(format!("--num-beams must be between 1 and 10, got: {}", args.num_beams));
    }
    if args.length_penalty < 0.5 || args.length_penalty > 2.0 {
        errors.push(format!("--length-penalty must be between 0.5 and 2.0, got: {}", args.length_penalty));
    }
    if args.max_new_tokens == 0 || args.max_new_tokens > 200 {
        errors.push(format!("--max-new-tokens must be between 1 and 200, got: {}", args.max_new_tokens));
    }
    if args.min_length > args.max_new_tokens {
        errors.push(format!("--min-length ({}) cannot be greater than --max-new-tokens ({})",
                          args.min_length, args.max_new_tokens));
    }
    if args.repetition_penalty < 1.0 || args.repetition_penalty > 2.0 {
        errors.push(format!("--repetition-penalty must be between 1.0 and 2.0, got: {}", args.repetition_penalty));
    }
    if args.no_repeat_ngram_size > 10 {
        errors.push(format!("--no-repeat-ngram-size must be <= 10, got: {}", args.no_repeat_ngram_size));
    }

    // Create model configuration
    let model_config = ModelConfig::from_name(
        &args.model_name,
        args.model_path.clone(),
        args.tokenizer_dir.clone(),
    );

    // Check model directory exists
    if !model_config.model_path.exists() {
        errors.push(format!("Model directory does not exist: {}", model_config.model_path.display()));
    }

    // Check tokenizer directory exists
    let tokenizer_path = model_config.tokenizer_path();
    if !tokenizer_path.exists() {
        errors.push(format!("Tokenizer directory does not exist: {}", tokenizer_path.display()));
    }

    if !errors.is_empty() {
        error!("❌ VALIDATION ERRORS:");
        for err in &errors {
            error!("   {}", err);
        }
        error!("\n📖 Use --help for complete usage information");
        error!("   cargo run --bin parallel_summarizer -- --help");
        return Err(anyhow::anyhow!("Validation failed: {}", errors.join(", ")));
    }

    // Create parent directories if needed
    if let Some(parent) = Path::new(&args.output_file).parent() {
        fs::create_dir_all(parent)?;
        info!("✅ Created output directory: {}", parent.display());
    }

    if let Some(parent) = Path::new(&args.results_file).parent() {
        fs::create_dir_all(parent)?;
        info!("✅ Created results directory: {}", parent.display());
    }

    // Load prompt from file or use inline prompt
    let prompt = match (args.prompt.as_ref(), args.prompt_file.as_ref()) {
        (Some(p), None) => p.clone(),
        (None, Some(file_path)) => {
            fs::read_to_string(file_path)
                .map_err(|e| anyhow::anyhow!("Failed to read prompt file {}: {}", file_path, e))?
                .trim().to_string()
        }
        _ => unreachable!(), // Validation ensures this can't happen
    };

    // Create generation configuration
    let strategy = match args.sampling_strategy {
        SamplingStrategy::Sampling => SamplingStrategy::Sampling,
        SamplingStrategy::Beam => SamplingStrategy::Beam,
    };

    let generation_config = GenerationConfig {
        strategy,
        temperature: args.temperature,
        top_p: args.top_p,
        top_k: args.top_k,
        num_beams: args.num_beams,
        length_penalty: args.length_penalty,
        early_stopping: args.early_stopping,
        max_new_tokens: args.max_new_tokens,
        min_length: args.min_length,
        repetition_penalty: args.repetition_penalty,
        no_repeat_ngram_size: args.no_repeat_ngram_size,
        stop_sequences: args.stop_sequences.clone(),
    };

    Ok((prompt, model_config, generation_config))
}

/// Write progress to results file
fn write_progress(results_file: &str, message: &str) -> Result<()> {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let log_entry = format!("[{}] {}\n", timestamp, message);
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(results_file)?
        .write_all(log_entry.as_bytes())?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    // Phase 0: Validate all compulsory arguments and get configurations
    let (prompt, model_config, generation_config) = validate_args(&args)?;

    // Initialize progress file
    write_progress(&args.results_file, "🚀 Starting 20-Agent Parallel Code Summarizer")?;
    write_progress(&args.results_file, &format!("📁 Input file: {}", args.file))?;
    write_progress(&args.results_file, &format!("📄 Output file: {}", args.output_file))?;
    write_progress(&args.results_file, &format!("📊 Results file: {}", args.results_file))?;
    write_progress(&args.results_file, &format!("🔢 Lines per chunk: {}", args.loc))?;
    write_progress(&args.results_file, &format!("🤖 Agent count: {}", args.agent_count))?;
    write_progress(&args.results_file, &format!("🧠 Model: {}", model_config.name))?;
    write_progress(&args.results_file, &format!("⚙️  Strategy: {:?}", generation_config.strategy))?;
    write_progress(&args.results_file, &format!("🌡️  Temperature: {:.2}", generation_config.temperature))?;
    write_progress(&args.results_file, &format!("💭 Prompt source: {}",
        if args.prompt_file.is_some() { "file" } else { "inline" }))?;

    info!("🚀 Starting 20-Agent Parallel Code Summarizer");
    info!("File: {}", args.file);
    info!("Output: {}", args.output_file);
    info!("Results: {}", args.results_file);
    info!("Lines per chunk: {}", args.loc);
    info!("Agents: {}", args.agent_count);
    info!("Model: {}", model_config.name);
    info!("Strategy: {:?}", generation_config.strategy);
    info!("Temperature: {:.2}", generation_config.temperature);

    // Phase 1: Read the file
    let code = fs::read_to_string(&args.file)?;
    let file_size = code.len();
    let line_count = code.lines().count();
    info!("📖 Read file: {} ({} bytes, {} lines)", args.file, file_size, line_count);
    write_progress(&args.results_file, &format!("📖 Read file: {} ({} bytes, {} lines)", args.file, file_size, line_count))?;

    // Phase 2: Create chunks for parallel processing using LOC-based chunking
    let chunks = chunk_code_by_loc(&code, args.loc);
    info!("✅ Created {} chunks for parallel processing ({} lines each)", chunks.len(), args.loc);
    write_progress(&args.results_file, &format!("✅ Created {} chunks for parallel processing ({} lines each)", chunks.len(), args.loc))?;

    // Phase 3: Configure parallel system with user-specified parameters
    let max_concurrent = args.max_concurrent.unwrap_or_else(num_cpus::get);
    let config = ParallelConfig {
        agent_count: args.agent_count,
        model_dir: model_config.model_path.clone(),
        tokenizer_dir: model_config.tokenizer_path(),
        max_concurrent,
        generation_config: generation_config.clone(),
    };

    // Phase 4: Initialize parallel system
    info!("🔧 Initializing {}-agent parallel system...", args.agent_count);
    write_progress(&args.results_file, "🔧 Initializing parallel system...")?;
    let system = ParallelAgentSystem::new(config)?;

    let metrics = system.get_metrics();
    info!("✅ Parallel system ready: {}", metrics);
    write_progress(&args.results_file, &format!("✅ Parallel system ready: {}", metrics))?;

    // Phase 5: Process chunks in parallel using custom prompts
    info!("🔄 Starting parallel processing with {} agents...", args.agent_count);
    write_progress(&args.results_file, "🔄 Starting parallel processing...")?;
    let start_time = std::time::Instant::now();

    let results = system.process_chunks_parallel_with_prompts(chunks, &prompt).await?;

    let processing_time = start_time.elapsed();
    info!("✅ Parallel processing completed in {:?}", processing_time);
    write_progress(&args.results_file, &format!("✅ Parallel processing completed in {:?}", processing_time))?;

    // Phase 6: Display summary to console
    println!("\n🎯 PROCESSING SUMMARY:");
    println!("=====================");
    println!("Total chunks processed: {}", results.len());
    println!("Total processing time: {:?}", processing_time);
    println!("Average time per chunk: {}ms", processing_time.as_millis() / results.len() as u128);
    println!("Throughput: {:.2} chunks/second", results.len() as f64 / processing_time.as_secs_f64());
    println!("Generation strategy: {:?}", generation_config.strategy);
    println!("Temperature: {:.2}", generation_config.temperature);

    // Phase 7: Save final summary to specified output file
    let full_summary = results.iter()
        .map(|(_, s)| s.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    fs::write(&args.output_file, full_summary)?;
    info!("💾 Final summary saved to: {}", args.output_file);
    write_progress(&args.results_file, &format!("💾 Final summary saved to: {}", args.output_file))?;

    // Phase 8: Final progress update
    write_progress(&args.results_file, "🎉 PARALLEL PROCESSING COMPLETE!")?;
    write_progress(&args.results_file, &format!("📊 Final metrics: {} chunks, {:?} total time", results.len(), processing_time))?;

    println!("\n🎉 PARALLEL PROCESSING COMPLETE!");
    println!("📄 Summary saved to: {}", args.output_file);
    println!("📊 Progress logged to: {}", args.results_file);
    Ok(())
}

/// LOC-based chunking function - replaces char-based chunking
fn chunk_code_by_loc(code: &str, loc: usize) -> Vec<String> {
    code.split('\n')
        .collect::<Vec<_>>()
        .chunks(loc)
        .map(|lines| lines.join("\n"))
        .collect()
}