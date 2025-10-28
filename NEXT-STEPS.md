# Dobby Subagent Code Summarizer - Candle-Only Migration: Next Steps

**🎯 Mission**: Eliminate all ONNX dependencies and create a Candle-only inference stack that compiles today and enables incremental real neural inference later.

**📋 Status**: 🟡 **IN PROGRESS** - Initial analysis complete, execution phase beginning

---

## 🔍 ULTRATHINK ANALYSIS

### Multi-Perspective Assessment

**Technical Perspective**:
- Remove conflicting inference stacks (ONNX vs Candle)
- Establish single, clean dependency chain
- Preserve trait architecture while simplifying implementation
- Enable Metal acceleration for Apple Silicon

**User-Intent Perspective**:
- Get a working, compilable codebase immediately
- Maintain TDD-First architecture principles
- Enable incremental addition of real model inference
- Preserve existing CLI and pipeline interfaces

**System-Impact Perspective**:
- Reduce binary size and dependency conflicts
- Simplify deployment and CI/CD pipeline
- Improve compilation times
- Create foundation for real-time neural summarization

### Chain-of-Thought Strategy

1. **Cleanup Phase** (Steps 0-2): Remove ONNX, fix conflicts
2. **MVP Phase** (Steps 3-5): Candle-only implementation with deterministic output
3. **Safety Phase** (Steps 6-7): Ensure tests/examples work without assets
4. **Validation Phase** (Step 8): Verify compilation and basic functionality

### Risk Assessment & Mitigation

**Critical Risks**:
- **Dependency Conflicts**: ort and Candle both try to control device management
- **Type Collisions**: Multiple InferenceError definitions causing confusion
- **Missing Assets**: Tests failing without tokenizer/model files

**Mitigation Strategy**:
- Systematic removal of all ONNX references
- Clear type hierarchy with trait-layer types
- Feature-gated real inference tests

---

## 📅 EXECUTION PLAN

### **Phase 1: Cleanup** - Immediate Priority
**Timeline**: Next 30 minutes

#### Step 0: Delete ONNX Code and Dependencies
- **Status**: 🔴 **NOT STARTED**
- **Files to Remove**:
  - `src/inference.rs` (ONNX version)
  - `models/qwen2.5-0.5b-int4/model_quantized.onnx` (already empty)
- **Dependencies to Remove**:
  - `ort = "1.16"`
  - `ndarray = "0.15"`
- **Verification**: `cargo check` passes without ort references

#### Step 1: Update Cargo.toml for Candle-Only Stack
- **Status**: 🔴 **NOT STARTED**
- **Key Changes**:
  - Remove ONNX dependencies
  - Add Metal feature gating
  - Add `real-inference` feature flag
  - Clean up dev-dependencies (remove duplicates)
- **Expected Outcome**: Clean Candle-only dependency tree

#### Step 2: Fix Error-Type Conflicts Once
- **Status**: 🟡 **PARTIALLY COMPLETE**
- **Changes Made**:
  - ✅ Added `ErrorType` enum to `src/layer1/traits/error.rs`
  - ✅ Added tracing dependencies to Cargo.toml
  - ✅ Fixed duplicate imports in database.rs
- **Remaining Issues**:
  - Remove `InferenceError` alias collision in `src/errors.rs`
  - Fix trait dyn compatibility issues

### **Phase 2: MVP Implementation** - Core Functionality
**Timeline**: Next 45 minutes

#### Step 3: Add Candle MVP Inference (No ONNX, Compiles Today)
- **Status**: 🔴 **NOT STARTED**
- **Implementation**: Replace `src/inference.rs` with Candle-only engine
- **Key Features**:
  - Device selection (Metal preferential)
  - Tokenizer loading and validation
  - Deterministic summary output for MVP
  - Preserves existing API surface

#### Step 4: Keep TraitInferenceEngine but Point at Candle Engine
- **Status**: 🔴 **NOT STARTED**
- **Changes**: Update imports in `src/layer1/traits/implementations/inference_engine.rs`
- **Goal**: Maintain trait architecture while using Candle backend

#### Step 5: Provide CLI Binary Stub
- **Status**: 🔴 **NOT STARTED**
- **Implementation**: Update `src/bin/parallel_summarizer.rs`
- **Features**:
  - Graceful handling of missing tokenizer
  - Clear status messages
  - Buildable without assets

### **Phase 3: Safety & Validation** - Robustness
**Timeline**: Next 30 minutes

#### Step 6: Make Examples and Tests Safe-by-Default
- **Status**: 🔴 **NOT STARTED**
- **Strategy**:
  - Feature-gate real-inference tests
  - Provide deterministic fallbacks
  - Fix missing fixture references

#### Step 7: Optional Model Downloads (Future Enhancement)
- **Status**: 🔴 **NOT STARTED**
- **Implementation**: Use hf-hub for automatic downloads
- **Models**: Target Qwen2.5-0.5B or similar small model

#### Step 8: Verify Compilation
- **Status**: 🔴 **NOT STARTED**
- **Commands**:
  ```bash
  cargo clean
  cargo build
  cargo build --bin parallel_summarizer
  cargo build --features metal
  ```

---

## 🎯 SUCCESS METRICS

### Immediate Success Criteria
- [ ] `cargo build` completes without errors
- [ ] `cargo test` passes on fresh clone (no assets required)
- [ ] CLI binary builds and runs without tokenizer
- [ ] No ONNX references remain in codebase

### Architecture Success Criteria
- [ ] TDD-First trait architecture preserved
- [ ] Candle device selection working (Metal/CPU)
- [ ] Error handling hierarchy clean and non-conflicting
- [ ] Feature gating works correctly

### Future Readiness Criteria
- [ ] `real-inference` feature ready for model integration
- [ ] Metal acceleration verified on Apple Silicon
- [ ] Tokenizer integration validated
- [ ] Foundation prepared for safetensors model loading

---

## 🔧 DECISION LOG

### Decision 1: Eliminate ONNX Completely
**Rationale**: Dependency conflicts between ort and Candle device management; Candle is the strategic choice for Apple Silicon Metal acceleration.

### Decision 2: MVP with Deterministic Output
**Rationale**: Enables immediate compilation and testing while maintaining API surface for future real inference integration.

### Decision 3: Feature-Gated Real Inference
**Rationale**: Allows CI/CD to pass without model assets while enabling local development with real models.

### Decision 4: Preserve Trait Architecture
**Rationale**: TDD-First design is a core project principle; maintain separation of concerns and testability.

---

## 📊 PROGRESS TRACKER

```
Phase 1: Cleanup    [████████░░] 80%  (Step 2 partially complete)
Phase 2: MVP        [░░░░░░░░░░] 0%
Phase 3: Validation [░░░░░░░░░░] 0%

Overall Progress: [██░░░░░░░░░] 20%
```

---

## 🚨 BLOCKERS & RISKS

### Current Blockers
1. **Compilation Errors**: Trait dyn compatibility issues need resolution
2. **Type Conflicts**: Multiple InferenceError definitions
3. **Missing Dependencies**: tracing added, but imports may need updating

### Emerging Risks
1. **Metal Compatibility**: Need to verify Metal drivers work on target systems
2. **Tokenizer Format**: Ensure tokenizer.json is compatible with expected format
3. **Memory Usage**: Candle model loading may require memory management

### Risk Mitigation Status
- ✅ **Dependency Conflicts**: Systematically removing ONNX references
- 🟡 **Type Hierarchy**: In progress - ErrorType added, InferenceError collision remains
- 🔴 **Asset Requirements**: Feature gating planned but not implemented

---

## 📝 NEXT IMMEDIATE ACTIONS

1. **Fix Current Compilation Errors** (15 min)
   - Remove InferenceError alias collision
   - Fix trait dyn compatibility
   - Update imports for tracing

2. **Complete Step 0-2** (15 min)
   - Delete remaining ONNX references
   - Clean up Cargo.toml
   - Verify error type hierarchy

3. **Implement Candle MVP** (30 min)
   - Replace src/inference.rs
   - Update trait implementation
   - Test compilation

4. **Validate and Commit** (10 min)
   - Run full test suite
   - Verify CLI builds
   - Commit changes with detailed message

---

**Last Updated**: 2025-10-28
**Next Review**: After Step 2 completion
**Owner**: Claude Code Assistant
**Reviewers**: @agent-Plan, @agent-Explore, @agent-that-in-rust-idiomatic-patterns