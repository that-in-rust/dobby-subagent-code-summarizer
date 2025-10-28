# Dobby Subagent Code Summarizer - Development Backlog

## 🎯 Enhanced CLI Help Improvements (Planned)

### 1. **Default Help Display**
**Problem**: Users currently see validation errors when running CLI without arguments
**Solution**: Show helpful help automatically instead of errors

**Implementation**:
- Modify `validate_args()` to detect missing required arguments
- Show help display with examples before error messages
- Guide users through required parameters with templates

### 2. **Use Case Examples in CLI Help**
**Problem**: Users need proven parameter combinations for different scenarios
**Solution**: Add copy-paste ready examples for common use cases

**Examples to Add**:
```bash
# Quick Code Review (small files < 100 lines)
cargo run --release --bin parallel_summarizer -- \
    --file ./src/utils.rs \
    --output-file /tmp/code_review.md \
    --results-file /tmp/review_progress.log \
    --loc 30 --agent-count 2 --sampling-strategy beam \
    --temperature 0.3 --max-new-tokens 50 \
    --prompt "Summarize functionality:"

# Security Analysis (medium files 100-1000 lines)
cargo run --release --bin parallel_summarizer -- \
    --file ./src/auth.rs \
    --output-file /tmp/security_analysis.md \
    --results-file /tmp/security_progress.log \
    --loc 500 --agent-count 8 --sampling-strategy beam \
    --temperature 0.4 --max-new-tokens 100 \
    --prompt "Analyze security patterns:"

# Documentation Generation (large files > 1000 lines)
cargo run --release --bin parallel_summarizer -- \
    --file ./src/lib.rs \
    --output-file /tmp/api_docs.md \
    --results-file /tmp/docs_progress.log \
    --loc 1000 --agent-count 15 --sampling-strategy beam \
    --temperature 0.5 --max-new-tokens 120 \
    --prompt "Generate API documentation:"
```

### 3. **Interactive Terminal Guidance**
**Problem**: Users need guidance when they make mistakes
**Solution**: Smart validation messages with educational content

**Implementation Approach**:
- Replace generic error messages with helpful guidance
- Include copy-paste ready command templates
- Add file size recommendations
- Provide troubleshooting tips

**Example Enhanced Error Message**:
```
❌ Missing required arguments. Here's how to get started:

🚀 Quick Start Examples:

# For small files (< 100 lines):
[copy-paste command template]

💡 Tips:
- Use --loc based on file size (30 for small, 500 for medium, 1000 for large)
- Beam search (--sampling-strategy beam) works best for consistent results
- Temperature 0.3-0.5 provides focused, reliable summaries
```

### 4. **File Size Recommendations**
**Problem**: Users don't know optimal chunk sizes for different file types
**Solution**: Add recommendations based on testing

**Recommendations**:
- **Small files** (< 100 lines): --loc 30, --agent-count 2
- **Medium files** (100-1000 lines): --loc 500, --agent-count 8
- **Large files** (> 1000 lines): --loc 1000, --agent-count 15

## 🔧 Technical Implementation Notes

### Code Changes Required:
1. **Modify `src/bin/parallel_summarizer.rs`**:
   - Update `validate_args()` function
   - Add help examples to CLI definition
   - Implement smart error messages

2. **Update CLI Help Text**:
   - Use clap's `about` and `long_about` attributes
   - Add examples section
   - Include parameter recommendations

3. **Test New Help System**:
   - Verify help displays correctly
   - Test examples are copy-paste ready
   - Ensure error messages are helpful

### Files to Modify:
- `src/bin/parallel_summarizer.rs` - Main CLI logic
- `README.md` - Update with new help features

## 📋 Priority & Timeline

**High Priority** (Next Sprint):
- Default help display
- Use case examples in help text
- Smart validation messages

**Medium Priority** (Future):
- Interactive guidance enhancements
- Advanced parameter recommendations
- Preset configurations

**Low Priority** (Nice to Have):
- Auto-detection of optimal parameters
- Learning system from user behavior
- Context-aware help based on file type

## 🧪 Testing Requirements

### Manual Testing:
- Test CLI help displays correctly
- Verify examples are accurate and working
- Check error messages are helpful

### Automated Testing:
- Add tests for help display logic
- Validate parameter recommendations
- Test error handling improvements

---

## 🌳 Tree-sitter ELI15: Understanding Semantic Chunking

### **What Tree-sitter Actually Creates**

Think of tree-sitter like a **sentence diagrammer for code**. When you give it code like this:

```rust
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
```

Tree-sitter creates a **family tree** in computer memory that looks like:

```
        function_item (the whole thing)
              /        \
         name: "add_numbers"   parameters: ["a", "b"]
              |                    |
           body: {               types: i32, i32
             expression: +
                /    \
            variable "a"  variable "b"
           }
```

**Key point**: Tree-sitter doesn't create chunks directly - it creates a **complete tree** where every piece knows:
- What it is (function, variable, operator, etc.)
- Where it starts and ends (line numbers, character positions)
- Who its "parents" and "children" are in the tree

### **How We Get "Chunks" from the Tree**

We're like **tree surgeons** - we walk through the tree and cut out specific branches:

1. **Find the big branches**: "I want all `function_item` nodes"
2. **Cut them out**: Extract the function and everything inside it
3. **Label them**: Save info like "this is a function called 'add_numbers'"

### **Where the Chunks Go (Storage Options)**

**Option 1: Save to Files (Like Notes)**
```json
{
  "id": "chunk_001",
  "type": "function",
  "name": "add_numbers",
  "start_line": 1,
  "end_line": 3,
  "content": "fn add_numbers(a: i32, b: i32) -> i32 {\n    a + b\n}"
}
```

**Option 2: Keep in Memory (Like Short-term Memory)**
- Computer RAM for fast access
- Good for processing right away
- Disappears when program closes

**Option 3: Database (Like Library Card Catalog)**
- Organized storage for easy searching
- Can handle thousands of chunks
- Persistent - stays there after program closes

### **The Complete Flow**

1. **Input**: Code file (like `main.rs`)
2. **Tree-sitter**: Creates family tree in memory
3. **Our Program**: Walks through tree, cuts out function/struct branches
4. **Chunks**: Get saved to files/memory/database
5. **AI**: Reads chunks and creates summaries

### **Why This is Better Than 300 Lines**

**Old way (dumb)**: Cut every 300 lines regardless of what's there
- Might cut a function in half! ❌

**New way (smart)**: Cut at natural boundaries (functions, structs, etc.)
- Always keeps complete code pieces together ✅

So tree-sitter gives us the **map**, and we use that map to make **smart cuts** that respect how code is actually structured!

---

## 🚀 Project TODOs (Migrated from Global List)

### **🔥 High Priority (Current Focus)**

#### **Workspace Refactoring**
- [ ] **Refactor project into workspace with two crates**: semantic-code-chunker and parallel-inference-engine
  - Create Cargo workspace structure
  - Split existing codebase into focused crates
  - Update dependencies and build configuration
  - Test workspace compilation and functionality

#### **Candle Framework Migration**
- [ ] **Research and validate Candle framework for LLM inference on Apple Silicon**
  - Test Candle with existing models (TinyLlama, DeepSeek-Coder, Gemma-2)
  - Benchmark performance vs. ONNX Runtime
  - Validate Metal GPU acceleration on Apple Silicon
  - Document performance improvements and any limitations

- [ ] **Migrate existing ONNX-based code to Candle framework**
  - Replace ONNX Runtime with Candle implementation
  - Update model loading and inference pipeline
  - Maintain existing API compatibility
  - Test with current parallel processing architecture

- [ ] **Evaluate Mistral 7B and TinyLlama models in safe tensors format for Candle**
  - Download and test models in Candle-compatible formats
  - Compare inference quality and performance
  - Validate model compatibility with 10x parallel processing
  - Update model configuration and routing logic

#### **Parallel Processing Architecture**
- [ ] **Design Session Pool Architecture for true 10x parallel processing**
  - Optimize session reuse for Candle framework
  - Implement efficient memory management for multiple concurrent sessions
  - Test scaling with 10+ parallel agents
  - Monitor resource usage and performance bottlenecks

### **🔧 Medium Priority (Next Phase)**

#### **Performance Optimization**
- [ ] **Implement Metal Performance Shaders integration for GPU acceleration**
  - Integrate Metal backend for Candle on Apple Silicon
  - Optimize GPU memory usage and transfer
  - Benchmark CPU vs. GPU performance
  - Implement fallback to CPU when GPU unavailable

- [ ] **Implement memory management for unified Apple Silicon architecture**
  - Optimize memory layout for Apple Silicon unified memory
  - Implement efficient buffer management
  - Reduce memory fragmentation and improve cache locality
  - Test with large codebases and multiple models

#### **Monitoring and Reliability**
- [ ] **Add production monitoring and observability for parallel processing**
  - Implement metrics collection for inference performance
  - Add real-time monitoring dashboard
  - Create alerting for performance degradation
  - Log and analyze processing patterns

- [ ] **Create comprehensive error handling and graceful degradation system**
  - Implement robust error recovery for model inference failures
  - Add retry mechanisms with exponential backoff
  - Create fallback strategies for model unavailability
  - Implement graceful degradation for partial failures

---

## 📋 Updated Priority & Timeline

**Phase 1: Foundation (Weeks 1-2)**
- Refactor project into Cargo workspace
- Migrate ONNX code to Candle framework
- Test basic functionality with new architecture

**Phase 2: Performance (Weeks 3-4)**
- Implement Metal GPU acceleration
- Optimize memory management
- Validate 10x parallel processing performance

**Phase 3: Production (Weeks 5-6)**
- Add monitoring and observability
- Implement comprehensive error handling
- Test with real-world codebases

---

**Documented**: 2025-10-26
**Status**: Active development plan with tree-sitter research foundation complete