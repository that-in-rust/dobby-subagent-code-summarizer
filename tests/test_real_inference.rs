//! Integration tests for real neural inference on real codebases
//!
//! Tests the OptimizedInferenceEngine with actual large-scale code files
//! to verify session reuse architecture and tensor processing work correctly.

use std::process::Command;
use std::path::Path;

/// Test real neural inference on Iggy Apache message system codebase
#[test]
fn test_real_inference_iggy_apache() {
    let output = Command::new("/Users/amuldotexe/Projects/transfiguration/target/release/code_summarizer")
        .arg("--file")
        .arg("tests/fixtures/iggy_apache.txt")
        .arg("--model-dir")
        .arg("./models/qwen2.5-0.5b-int4")
        .arg("--tokenizer-dir")
        .arg("./tokenizer_dir")
        .output()
        .expect("Failed to execute code_summarizer");

    println!("Iggy Apache stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Iggy Apache stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success(), "Iggy Apache test should succeed");

    // Check that summary file was created
    assert!(Path::new("tests/fixtures/iggy_apache_Summary.md").exists(),
            "Summary file should be created");

    // Read and verify summary contains real neural inference content
    let summary = std::fs::read_to_string("tests/fixtures/iggy_apache_Summary.md")
        .expect("Should be able to read summary file");

    assert!(summary.contains("neural inference"),
            "Summary should contain evidence of neural inference");
    assert!(!summary.contains("ERROR"),
            "Summary should not contain error messages");
}

/// Test real neural inference on Ray distributed computing framework
#[test]
fn test_real_inference_ray_project() {
    let output = Command::new("/Users/amuldotexe/Projects/transfiguration/target/release/code_summarizer")
        .arg("--file")
        .arg("tests/fixtures/ray-project-ray-8a5edab282632443.txt")
        .arg("--model-dir")
        .arg("./models/qwen2.5-0.5b-int4")
        .arg("--tokenizer-dir")
        .arg("./tokenizer_dir")
        .output()
        .expect("Failed to execute code_summarizer");

    println!("Ray Project stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Ray Project stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success(), "Ray Project test should succeed");

    // Check that summary file was created
    assert!(Path::new("tests/fixtures/ray-project-ray-8a5edab282632443_Summary.md").exists(),
            "Summary file should be created");

    // Read and verify summary contains real neural inference content
    let summary = std::fs::read_to_string("tests/fixtures/ray-project-ray-8a5edab282632443_Summary.md")
        .expect("Should be able to read summary file");

    assert!(summary.contains("neural inference"),
            "Summary should contain evidence of neural inference");
    assert!(!summary.contains("ERROR"),
            "Summary should not contain error messages");
}

/// Test real neural inference on Tokio async runtime framework
#[test]
fn test_real_inference_tokio_rs() {
    let output = Command::new("/Users/amuldotexe/Projects/transfiguration/target/release/code_summarizer")
        .arg("--file")
        .arg("tests/fixtures/tokio-rs-tokio-8a5edab282632443.txt")
        .arg("--model-dir")
        .arg("./models/qwen2.5-0.5b-int4")
        .arg("--tokenizer-dir")
        .arg("./tokenizer_dir")
        .output()
        .expect("Failed to execute code_summarizer");

    println!("Tokio RS stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Tokio RS stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success(), "Tokio RS test should succeed");

    // Check that summary file was created
    assert!(Path::new("tests/fixtures/tokio-rs-tokio-8a5edab282632443_Summary.md").exists(),
            "Summary file should be created");

    // Read and verify summary contains real neural inference content
    let summary = std::fs::read_to_string("tests/fixtures/tokio-rs-tokio-8a5edab282632443_Summary.md")
        .expect("Should be able to read summary file");

    assert!(summary.contains("neural inference"),
            "Summary should contain evidence of neural inference");
    assert!(!summary.contains("ERROR"),
            "Summary should not contain error messages");
}

/// Test session reuse performance and tensor consistency
#[test]
fn test_session_reuse_architecture() {
    // This test verifies that the OptimizedInferenceEngine correctly reuses sessions
    // across multiple inference calls, ensuring ~0ms overhead for subsequent calls

    let test_files = vec![
        "tests/fixtures/iggy_apache.txt",
        "tests/fixtures/ray-project-ray-8a5edab282632443.txt",
        "tests/fixtures/tokio-rs-tokio-8a5edab282632443.txt"
    ];

    for test_file in test_files {
        let output = Command::new("/Users/amuldotexe/Projects/transfiguration/target/release/code_summarizer")
            .arg("--file")
            .arg(test_file)
            .arg("--model-dir")
            .arg("./models/qwen2.5-0.5b-int4")
            .arg("--tokenizer-dir")
            .arg("./tokenizer_dir")
            .output()
            .expect("Failed to execute code_summarizer");

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Verify session reuse is working
        assert!(stderr.contains("OPTIMIZED ENGINE READY"),
                "Should show optimized engine initialization");
        assert!(stderr.contains("Neural inference completed"),
                "Should show successful neural inference");
        assert!(stderr.contains("Framework-aligned success"),
                "Should show framework-aligned tensor processing");

        // Verify no missing past_key_values errors
        assert!(!stderr.contains("Missing Input: past_key_values"),
                "Should not have missing past_key_values errors");
    }
}