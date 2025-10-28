//! Minimal setup validation - bypasses all compilation issues
//! Just validates that our file structure is correct

use std::path::Path;
use std::fs;

fn main() {
    println!("🚀 Validating Dobby Subagent Code Summarizer Setup");
    println!("=================================================");

    // Check model directory structure
    let model_path = Path::new("./models/qwen2.5-0.5b-int4");
    let model_file = model_path.join("model.safetensors");

    println!("\n📁 Checking model structure:");
    println!("   Model directory: {}", model_path.display());

    if model_path.exists() {
        println!("   ✅ Model directory exists");

        if model_file.exists() {
            let metadata = fs::metadata(&model_file).unwrap();
            let size_mb = metadata.len() / 1_000_000;
            println!("   ✅ Model file exists: {} MB", size_mb);
        } else {
            println!("   ❌ Model file missing: {}", model_file.display());
        }

        // List all files in model directory
        if let Ok(entries) = fs::read_dir(model_path) {
            println!("   📋 Model directory contents:");
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let size = entry.metadata().unwrap().len();
                    println!("      - {} ({} bytes)", path.file_name().unwrap().to_string_lossy(), size);
                }
            }
        }
    } else {
        println!("   ❌ Model directory missing");
    }

    // Check tokenizer structure
    let tokenizer_path = Path::new("./tokenizer_dir");
    let tokenizer_file = tokenizer_path.join("tokenizer.json");

    println!("\n📁 Checking tokenizer structure:");
    println!("   Tokenizer directory: {}", tokenizer_path.display());

    if tokenizer_path.exists() {
        println!("   ✅ Tokenizer directory exists");

        if tokenizer_file.exists() {
            let metadata = fs::metadata(&tokenizer_file).unwrap();
            let size_mb = metadata.len() / 1_000_000;
            println!("   ✅ Tokenizer file exists: {} MB", size_mb);
        } else {
            println!("   ❌ Tokenizer file missing: {}", tokenizer_file.display());
        }

        // List all files in tokenizer directory
        if let Ok(entries) = fs::read_dir(tokenizer_path) {
            println!("   📋 Tokenizer directory contents:");
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let size = entry.metadata().unwrap().len();
                    println!("      - {} ({} bytes)", path.file_name().unwrap().to_string_lossy(), size);
                }
            }
        }
    } else {
        println!("   ❌ Tokenizer directory missing");
    }

    // Check if we have the right structure for the OptimizedInferenceEngine
    println!("\n🎯 OptimizedInferenceEngine Requirements:");
    println!("   Expected model path: ./models/qwen2.5-0.5b-int4/model.safetensors");
    println!("   Expected tokenizer path: ./tokenizer_dir/tokenizer.json");

    let model_ok = model_file.exists();
    let tokenizer_ok = tokenizer_file.exists();

    if model_ok && tokenizer_ok {
        println!("\n🎉 SUCCESS: All required files are in place!");
        println!("   The OptimizedInferenceEngine should be able to load the models.");
        println!("   Ready to proceed with inference testing.");
    } else {
        println!("\n❌ SETUP INCOMPLETE: Some required files are missing.");
        if !model_ok {
            println!("   - Missing: model.safetensors");
        }
        if !tokenizer_ok {
            println!("   - Missing: tokenizer.json");
        }
    }

    println!("\n📝 Next Steps:");
    println!("   1. ✅ File structure validation (this script)");
    println!("   2. ⏳ Fix compilation issues in trait system");
    println!("   3. ⏳ Test OptimizedInferenceEngine model loading");
    println!("   4. ⏳ Run basic inference test");
    println!("   5. ⏳ Validate end-to-end pipeline");
}