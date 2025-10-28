# Mermaid Diagrams for Dobby Project Architecture

## Purpose
This document contains all Mermaid diagrams for the Dobby project, ensuring easy GitHub rendering and architectural visualization.

## 1. TDD-First Development Flow

```mermaid
graph TD
    A[STUB Phase] --> B[RED Phase]
    B --> C[GREEN Phase]
    C --> D[REFACTOR Phase]
    D --> A

    A --> A1[Define Trait Interface]
    A --> A2[Minimal Implementation]
    A --> A3[Ensure Compilation]

    B --> B1[Write Failing Tests]
    B --> B2[Define Contracts]
    B --> B3[Verify Test Failure]

    C --> C1[Minimal Working Code]
    C --> C2[Make Tests Pass]
    C --> C3[Basic Functionality]

    D --> D1[Optimize Performance]
    D --> D2[Improve Code Quality]
    D --> D3[Maintain Test Coverage]
```

## 2. Layered Architecture

```mermaid
graph TB
    subgraph "Layer 3: External Dependencies"
        L3A[Tokio Async Runtime]
        L3B[Candle RS ML Framework]
        L3C[Serde Serialization]
        L3D[Database Drivers]
    end

    subgraph "Layer 2: Standard Library"
        L2A[Arc/Rc Smart Pointers]
        L2B[HashMap/VecDeque Collections]
        L2C[Iterator Patterns]
        L2D[Thread Safety Send/Sync]
    end

    subgraph "Layer 1: Core Language Features"
        L1A[Ownership & Borrowing]
        L1B[Lifetimes & Traits]
        L1C[Result/Option Types]
        L1D[RAII Pattern]
    end

    subgraph "Application Layer"
        APP[Dobby Code Summarizer]
    end

    APP --> L1A
    APP --> L1B
    APP --> L1C
    APP --> L1D

    L1A --> L2A
    L1B --> L2B
    L1C --> L2C
    L1D --> L2D

    L2A --> L3A
    L2B --> L3B
    L2C --> L3C
    L2D --> L3D
```

## 3. Dependency Injection Architecture

```mermaid
graph LR
    subgraph "Production Implementation"
        PROD_DB[Production Database]
        PROD_INF[Real Inference Engine]
        PROD_PIPE[Production Pipeline]
    end

    subgraph "Mock Implementation"
        MOCK_DB[Mock Database]
        MOCK_INF[Mock Inference Engine]
        MOCK_PIPE[Mock Pipeline]
    end

    subgraph "Core Services"
        CHUNKING[Chunking Service]
        PROCESSING[Processing Service]
        ORCHESTRATION[Pipeline Orchestrator]
    end

    subgraph "Traits"
        DB_TRAIT[Database Trait]
        INF_TRAIT[Inference Trait]
        PIPE_TRAIT[Pipeline Trait]
    end

    PROD_DB -.-> DB_TRAIT
    MOCK_DB -.-> DB_TRAIT
    PROD_INF -.-> INF_TRAIT
    MOCK_INF -.-> INF_TRAIT
    PROD_PIPE -.-> PIPE_TRAIT
    MOCK_PIPE -.-> PIPE_TRAIT

    DB_TRAIT --> CHUNKING
    INF_TRAIT --> PROCESSING
    PIPE_TRAIT --> ORCHESTRATION
```

## 4. Performance Contract Validation Flow

```mermaid
sequenceDiagram
    participant Test as Test Suite
    participant Contract as Performance Contract
    participant Impl as Implementation
    participant Validator as Contract Validator

    Test->>Contract: validate_contract(input)

    Contract->>Impl: execute(input)
    Impl-->>Contract: result + metrics

    Contract->>Validator: check_latency(metrics)
    Contract->>Validator: check_memory(metrics)
    Contract->>Validator: check_throughput(metrics)

    alt All Contracts Pass
        Validator-->>Contract: validation_passed
        Contract-->>Test: PerformanceReport(passed=true)
    else Contract Violations
        Validator-->>Contract: validation_failed(violations)
        Contract-->>Test: PerformanceReport(passed=false, violations)
    end
```

## 5. RAII Resource Management

```mermaid
stateDiagram-v2
    [*] --> ResourceCreated
    ResourceCreated --> ResourceAcquired: acquire()
    ResourceAcquired --> ResourceInUse: using resource
    ResourceInUse --> ResourceReleased: release/drop
    ResourceReleased --> [*]: cleanup complete

    ResourceAcquired --> ResourceError: acquisition failed
    ResourceError --> [*]: error handling

    ResourceInUse --> ResourceError: usage failed
    ResourceError --> ResourceCleanup: emergency cleanup
    ResourceCleanup --> [*]: cleanup complete
```

## 6. Test Organization Structure

```mermaid
graph TD
    subgraph "Source Code"
        SRC[src/]
        TRAITS[layer1/traits/]
        IMPL[implementations/]
        TESTS[tests/]
    end

    subgraph "Test Phases"
        STUB[test_stub_phase.rs]
        RED[test_red_phase.rs]
        GREEN[test_green_phase.rs]
        REFACTOR[test_refactor_phase.rs]
    end

    subgraph "Test Types"
        UNIT[Unit Tests]
        INTEG[Integration Tests]
        PERF[Performance Tests]
        PROP[Property Tests]
    end

    SRC --> TRAITS
    TRAITS --> IMPL
    TRAITS --> TESTS

    TESTS --> STUB
    TESTS --> RED
    TESTS --> GREEN
    TESTS --> REFACTOR

    STUB --> UNIT
    RED --> UNIT
    GREEN --> UNIT
    REFACTOR --> UNIT

    GREEN --> INTEG
    REFACTOR --> INTEG

    REFACTOR --> PERF
    GREEN --> PROP
```

## 7. Pipeline Processing Flow

```mermaid
flowchart TD
    INPUT[Input Document] --> CHUNK[Text Chunking]
    CHUNK --> CHUNKS[Text Chunks]

    CHUNKS --> INF1[Inference Engine 1]
    CHUNKS --> INF2[Inference Engine 2]
    CHUNKS --> INFN[Inference Engine N]

    INF1 --> SUM1[Chunk Summary 1]
    INF2 --> SUM2[Chunk Summary 2]
    INFN --> SUMN[Chunk Summary N]

    SUM1 --> AGG[Summary Aggregation]
    SUM2 --> AGG
    SUMN --> AGG

    AGG --> FINAL[Final Document Summary]

    subgraph "Performance Monitoring"
        METRICS[Performance Metrics]
        CONTRACTS[Contract Validation]
    end

    CHUNK --> METRICS
    INF1 --> METRICS
    INF2 --> METRICS
    INFN --> METRICS
    AGG --> METRICS

    METRICS --> CONTRACTS
    CONTRACTS --> FINAL
```

## 8. Error Handling Hierarchy

```mermaid
graph TD
    subgraph "Library Errors (thiserror)"
        PROC_ERR[ProcessingError]
        INF_ERR[InferenceError]
        DB_ERR[DatabaseError]
        CONFIG_ERR[ConfigurationError]
    end

    subgraph "Application Errors (anyhow)"
        APP_ERR[ApplicationError]
        CONTEXT_ERR[ContextualError]
    end

    PROC_ERR --> APP_ERR
    INF_ERR --> APP_ERR
    DB_ERR --> APP_ERR
    CONFIG_ERR --> APP_ERR

    APP_ERR --> CONTEXT_ERR

    subgraph "Error Recovery"
        RETRY[Retry Logic]
        FALLBACK[Fallback Strategy]
        REPORT[Error Reporting]
    end

    CONTEXT_ERR --> RETRY
    CONTEXT_ERR --> FALLBACK
    CONTEXT_ERR --> REPORT
```

## 9. Session Pool Management

```mermaid
sequenceDiagram
    participant App as Application
    participant Pool as SessionPool
    participant Session as InferenceSession
    participant Tracker as ResourceTracker

    App->>Pool: acquire_session()

    alt Session Available
        Pool->>App: SessionGuard(session)
        App->>Session: inference(input)
        Session-->>App: result
        App->>Tracker: release_usage_metrics()
    else Pool Exhausted
        Pool->>Pool: create_new_session()
        Pool->>App: SessionGuard(new_session)
        Note over Pool: Update pool metrics
    end

    App->>Pool: drop(SessionGuard)
    Pool->>Session: return_to_pool()
    Pool->>Tracker: update_pool_metrics()
```

## 10. Development Workflow Integration

```mermaid
journey
    title TDD-First Development Journey
    section Development Phase
      Write Trait Stub: 5: Developer
      Write Failing Test: 5: Developer
      Make Test Pass: 5: Developer
      Refactor Code: 5: Developer
    section Quality Assurance
      Run Test Suite: 5: CI/CD
      Check Performance: 5: CI/CD
      Validate Contracts: 5: CI/CD
    section Deployment
      Merge to Main: 5: Developer
      Release: 5: DevOps
```

## Usage Instructions

1. **Copy-paste these diagrams** into GitHub issues, PRs, or README files
2. **Mermaid automatically renders** in GitHub's markdown viewer
3. **Update diagrams** when architecture changes
4. **Keep this as single source of truth** for all architectural diagrams

These diagrams provide clear visualization of the TDD-First architecture and development workflow for the Dobby project.