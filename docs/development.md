# Development Guide

## Development Overview

This guide covers the development workflow, testing strategies, and contribution guidelines for the Campfire Rust rewrite project.

## Development Workflow

```mermaid
graph TD
    subgraph "Development Cycle"
        direction TB
        SETUP[Initial Setup<br/>Clone + Dependencies]
        BRANCH[Create Feature Branch<br/>git checkout -b feature/name]
        DEVELOP[Write Code<br/>TDD Approach]
        TEST[Run Tests<br/>Unit + Integration]
        REVIEW[Code Review<br/>Pull Request]
        MERGE[Merge to Main<br/>CI/CD Pipeline]
    end
    
    subgraph "Testing Strategy"
        direction TB
        UNIT[Unit Tests<br/>Individual Functions]
        INTEGRATION[Integration Tests<br/>Service Interactions]
        E2E[End-to-End Tests<br/>Full User Flows]
        PROPERTY[Property Tests<br/>Invariant Validation]
    end
    
    subgraph "Quality Gates"
        direction TB
        LINT[Linting<br/>cargo clippy]
        FORMAT[Formatting<br/>cargo fmt]
        AUDIT[Security Audit<br/>cargo audit]
        COVERAGE[Test Coverage<br/>tarpaulin]
    end
    
    SETUP --> BRANCH
    BRANCH --> DEVELOP
    DEVELOP --> TEST
    TEST --> REVIEW
    REVIEW --> MERGE
    
    DEVELOP --> UNIT
    DEVELOP --> INTEGRATION
    DEVELOP --> E2E
    DEVELOP --> PROPERTY
    
    TEST --> LINT
    TEST --> FORMAT
    TEST --> AUDIT
    TEST --> COVERAGE
    
    classDef workflow fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef testing fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef quality fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class SETUP,BRANCH,DEVELOP,TEST,REVIEW,MERGE workflow
    class UNIT,INTEGRATION,E2E,PROPERTY testing
    class LINT,FORMAT,AUDIT,COVERAGE quality
```

## Project Structure Deep Dive

### Source Code Organization

```mermaid
graph TD
    subgraph "src/ Directory Structure"
        direction TB
        MAIN[main.rs<br/>Application Entry Point]
        LIB[lib.rs<br/>Library Root]
        CONFIG[config.rs<br/>Configuration Management]
        ERRORS[errors.rs<br/>Error Definitions]
    end
    
    subgraph "Core Modules"
        direction TB
        HANDLERS[handlers/<br/>HTTP Request Handlers]
        SERVICES[services/<br/>Business Logic]
        MODELS[models/<br/>Domain Models]
        MIDDLEWARE[middleware/<br/>Request Processing]
        DATABASE[database/<br/>Data Access]
    end
    
    subgraph "Supporting Modules"
        direction TB
        ASSETS[assets.rs<br/>Static Asset Serving]
        HEALTH[health.rs<br/>Health Checks]
        LOGGING[logging.rs<br/>Structured Logging]
        METRICS[metrics.rs<br/>Prometheus Metrics]
        SHUTDOWN[shutdown.rs<br/>Graceful Shutdown]
    end
    
    MAIN --> LIB
    LIB --> CONFIG
    LIB --> ERRORS
    
    LIB --> HANDLERS
    LIB --> SERVICES
    LIB --> MODELS
    LIB --> MIDDLEWARE
    LIB --> DATABASE
    
    LIB --> ASSETS
    LIB --> HEALTH
    LIB --> LOGGING
    LIB --> METRICS
    LIB --> SHUTDOWN
    
    classDef core fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef modules fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef support fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class MAIN,LIB,CONFIG,ERRORS core
    class HANDLERS,SERVICES,MODELS,MIDDLEWARE,DATABASE modules
    class ASSETS,HEALTH,LOGGING,METRICS,SHUTDOWN support
```

### Handler Architecture

```mermaid
graph TD
    subgraph "HTTP Handlers"
        direction TB
        AUTH_H[auth.rs<br/>Authentication Endpoints]
        USERS_H[users.rs<br/>User Management]
        ROOMS_H[rooms.rs<br/>Room Operations]
        MESSAGES_H[messages.rs<br/>Message CRUD]
        SEARCH_H[search.rs<br/>Search Endpoints]
        WEBSOCKET_H[websocket.rs<br/>WebSocket Upgrade]
    end
    
    subgraph "Handler Pattern"
        direction TB
        EXTRACT[Request Extraction<br/>Path, Query, JSON]
        VALIDATE[Input Validation<br/>Serde + Custom]
        AUTHORIZE[Authorization<br/>Session + Permissions]
        BUSINESS[Business Logic<br/>Service Layer Call]
        RESPONSE[Response Building<br/>JSON or HTML]
    end
    
    subgraph "Error Handling"
        direction TB
        APP_ERROR[Application Errors<br/>Structured Types]
        HTTP_ERROR[HTTP Error Mapping<br/>Status Codes]
        CLIENT_ERROR[Client Error Response<br/>JSON Format]
    end
    
    AUTH_H --> EXTRACT
    USERS_H --> EXTRACT
    ROOMS_H --> EXTRACT
    MESSAGES_H --> EXTRACT
    SEARCH_H --> EXTRACT
    WEBSOCKET_H --> EXTRACT
    
    EXTRACT --> VALIDATE
    VALIDATE --> AUTHORIZE
    AUTHORIZE --> BUSINESS
    BUSINESS --> RESPONSE
    
    BUSINESS --> APP_ERROR
    APP_ERROR --> HTTP_ERROR
    HTTP_ERROR --> CLIENT_ERROR
    
    classDef handlers fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef pattern fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef errors fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class AUTH_H,USERS_H,ROOMS_H,MESSAGES_H,SEARCH_H,WEBSOCKET_H handlers
    class EXTRACT,VALIDATE,AUTHORIZE,BUSINESS,RESPONSE pattern
    class APP_ERROR,HTTP_ERROR,CLIENT_ERROR errors
```

### Service Layer Architecture

```mermaid
graph TD
    subgraph "Service Implementations"
        direction TB
        AUTH_S[AuthService<br/>Authentication Logic]
        MESSAGE_S[MessageService<br/>Message Operations]
        ROOM_S[RoomService<br/>Room Management]
        USER_S[UserService<br/>User Operations]
        SEARCH_S[SearchService<br/>FTS5 Search]
        PUSH_S[PushService<br/>Web Push Notifications]
    end
    
    subgraph "Service Traits"
        direction TB
        TRAIT_DEF[Trait Definitions<br/>Abstract Interfaces]
        MOCK_IMPL[Mock Implementations<br/>Testing Support]
        PROD_IMPL[Production Implementations<br/>Real Database]
    end
    
    subgraph "Cross-Cutting Concerns"
        direction TB
        LOGGING_S[Logging<br/>Structured Events]
        METRICS_S[Metrics<br/>Performance Tracking]
        CACHING[Caching<br/>In-Memory State]
        VALIDATION_S[Validation<br/>Business Rules]
    end
    
    AUTH_S --> TRAIT_DEF
    MESSAGE_S --> TRAIT_DEF
    ROOM_S --> TRAIT_DEF
    USER_S --> TRAIT_DEF
    SEARCH_S --> TRAIT_DEF
    PUSH_S --> TRAIT_DEF
    
    TRAIT_DEF --> MOCK_IMPL
    TRAIT_DEF --> PROD_IMPL
    
    PROD_IMPL --> LOGGING_S
    PROD_IMPL --> METRICS_S
    PROD_IMPL --> CACHING
    PROD_IMPL --> VALIDATION_S
    
    classDef services fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef traits fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef concerns fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class AUTH_S,MESSAGE_S,ROOM_S,USER_S,SEARCH_S,PUSH_S services
    class TRAIT_DEF,MOCK_IMPL,PROD_IMPL traits
    class LOGGING_S,METRICS_S,CACHING,VALIDATION_S concerns
```

## Testing Strategy

### Test Organization

```mermaid
graph TD
    subgraph "Unit Tests"
        direction TB
        MODEL_TESTS[Model Tests<br/>Domain Logic]
        SERVICE_TESTS[Service Tests<br/>Business Logic]
        HANDLER_TESTS[Handler Tests<br/>HTTP Logic]
        UTIL_TESTS[Utility Tests<br/>Helper Functions]
    end
    
    subgraph "Integration Tests"
        direction TB
        API_TESTS[API Tests<br/>End-to-End HTTP]
        DB_TESTS[Database Tests<br/>Real SQLite]
        WS_TESTS[WebSocket Tests<br/>Real Connections]
        AUTH_TESTS[Auth Flow Tests<br/>Complete Flows]
    end
    
    subgraph "Property Tests"
        direction TB
        INVARIANT_TESTS[Invariant Tests<br/>Business Rules]
        ROUNDTRIP_TESTS[Roundtrip Tests<br/>Serialization]
        FUZZ_TESTS[Fuzz Tests<br/>Input Validation]
    end
    
    subgraph "Performance Tests"
        direction TB
        LOAD_TESTS[Load Tests<br/>High Throughput]
        STRESS_TESTS[Stress Tests<br/>Resource Limits]
        BENCHMARK_TESTS[Benchmark Tests<br/>Performance Regression]
    end
    
    MODEL_TESTS --> API_TESTS
    SERVICE_TESTS --> DB_TESTS
    HANDLER_TESTS --> WS_TESTS
    UTIL_TESTS --> AUTH_TESTS
    
    API_TESTS --> INVARIANT_TESTS
    DB_TESTS --> ROUNDTRIP_TESTS
    WS_TESTS --> FUZZ_TESTS
    
    INVARIANT_TESTS --> LOAD_TESTS
    ROUNDTRIP_TESTS --> STRESS_TESTS
    FUZZ_TESTS --> BENCHMARK_TESTS
    
    classDef unit fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef integration fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef property fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef performance fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class MODEL_TESTS,SERVICE_TESTS,HANDLER_TESTS,UTIL_TESTS unit
    class API_TESTS,DB_TESTS,WS_TESTS,AUTH_TESTS integration
    class INVARIANT_TESTS,ROUNDTRIP_TESTS,FUZZ_TESTS property
    class LOAD_TESTS,STRESS_TESTS,BENCHMARK_TESTS performance
```

### Test-Driven Development Flow

```mermaid
graph TD
    subgraph "TDD Cycle"
        direction TB
        RED[Red Phase<br/>Write Failing Test]
        GREEN[Green Phase<br/>Make Test Pass]
        REFACTOR[Refactor Phase<br/>Improve Code Quality]
        REPEAT[Repeat Cycle<br/>Next Feature]
    end
    
    subgraph "Test Types by Phase"
        direction TB
        UNIT_TDD[Unit Tests<br/>Function Level]
        INTEGRATION_TDD[Integration Tests<br/>Component Level]
        ACCEPTANCE_TDD[Acceptance Tests<br/>Feature Level]
    end
    
    subgraph "Quality Checks"
        direction TB
        COVERAGE[Test Coverage<br/>Line + Branch]
        MUTATION[Mutation Testing<br/>Test Quality]
        PROPERTY_CHECK[Property Checking<br/>Invariants]
    end
    
    RED --> GREEN
    GREEN --> REFACTOR
    REFACTOR --> REPEAT
    REPEAT --> RED
    
    RED --> UNIT_TDD
    GREEN --> INTEGRATION_TDD
    REFACTOR --> ACCEPTANCE_TDD
    
    UNIT_TDD --> COVERAGE
    INTEGRATION_TDD --> MUTATION
    ACCEPTANCE_TDD --> PROPERTY_CHECK
    
    classDef tdd fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef types fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef quality fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class RED,GREEN,REFACTOR,REPEAT tdd
    class UNIT_TDD,INTEGRATION_TDD,ACCEPTANCE_TDD types
    class COVERAGE,MUTATION,PROPERTY_CHECK quality
```

### Testing Commands

```bash
# Run all tests
cargo test

# Run specific test module
cargo test --test integration_test

# Run tests with output
cargo test -- --nocapture

# Run property tests
cargo test --features proptest

# Generate test coverage
cargo tarpaulin --out html

# Run benchmarks
cargo bench

# Mutation testing
cargo mutants

# Security audit
cargo audit
```

## Code Quality Standards

### Rust Idioms and Patterns

```mermaid
graph TD
    subgraph "Type Safety"
        direction TB
        NEWTYPE[Newtype Pattern<br/>UserId, RoomId]
        PHANTOM[Phantom Types<br/>State Machines]
        CONST_GENERIC[Const Generics<br/>Compile-time Validation]
        TYPESTATE[Typestate Pattern<br/>API Safety]
    end
    
    subgraph "Error Handling"
        direction TB
        RESULT[Result<T, E><br/>Explicit Error Handling]
        THISERROR[thiserror<br/>Library Errors]
        ANYHOW[anyhow<br/>Application Errors]
        CONTEXT[Error Context<br/>Debugging Information]
    end
    
    subgraph "Memory Management"
        direction TB
        OWNERSHIP[Ownership<br/>Move Semantics]
        BORROWING[Borrowing<br/>References]
        LIFETIMES[Lifetimes<br/>Memory Safety]
        RAII[RAII<br/>Resource Management]
    end
    
    subgraph "Concurrency"
        direction TB
        ASYNC_AWAIT[async/await<br/>Asynchronous Code]
        CHANNELS[Channels<br/>Message Passing]
        ATOMICS[Atomics<br/>Lock-free Operations]
        MUTEX[Mutex/RwLock<br/>Shared State]
    end
    
    NEWTYPE --> RESULT
    PHANTOM --> THISERROR
    CONST_GENERIC --> ANYHOW
    TYPESTATE --> CONTEXT
    
    RESULT --> OWNERSHIP
    THISERROR --> BORROWING
    ANYHOW --> LIFETIMES
    CONTEXT --> RAII
    
    OWNERSHIP --> ASYNC_AWAIT
    BORROWING --> CHANNELS
    LIFETIMES --> ATOMICS
    RAII --> MUTEX
    
    classDef types fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef errors fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef memory fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef concurrency fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class NEWTYPE,PHANTOM,CONST_GENERIC,TYPESTATE types
    class RESULT,THISERROR,ANYHOW,CONTEXT errors
    class OWNERSHIP,BORROWING,LIFETIMES,RAII memory
    class ASYNC_AWAIT,CHANNELS,ATOMICS,MUTEX concurrency
```

### Code Review Checklist

```mermaid
graph TD
    subgraph "Functionality"
        direction TB
        REQUIREMENTS[Requirements Met<br/>Feature Complete]
        EDGE_CASES[Edge Cases Handled<br/>Error Conditions]
        PERFORMANCE[Performance Acceptable<br/>No Regressions]
        SECURITY[Security Considerations<br/>Input Validation]
    end
    
    subgraph "Code Quality"
        direction TB
        READABILITY[Code Readability<br/>Clear Intent]
        MAINTAINABILITY[Maintainability<br/>Easy to Modify]
        TESTABILITY[Testability<br/>Good Test Coverage]
        DOCUMENTATION[Documentation<br/>Comments + Examples]
    end
    
    subgraph "Rust Specifics"
        direction TB
        IDIOMS[Rust Idioms<br/>Idiomatic Code]
        SAFETY[Memory Safety<br/>No Unsafe Code]
        EFFICIENCY[Efficiency<br/>Zero-cost Abstractions]
        CLIPPY[Clippy Lints<br/>No Warnings]
    end
    
    subgraph "Integration"
        direction TB
        API_COMPAT[API Compatibility<br/>Breaking Changes]
        DEPENDENCIES[Dependencies<br/>Minimal + Justified]
        BACKWARDS[Backwards Compatibility<br/>Migration Path]
        DEPLOYMENT[Deployment Impact<br/>Configuration Changes]
    end
    
    REQUIREMENTS --> READABILITY
    EDGE_CASES --> MAINTAINABILITY
    PERFORMANCE --> TESTABILITY
    SECURITY --> DOCUMENTATION
    
    READABILITY --> IDIOMS
    MAINTAINABILITY --> SAFETY
    TESTABILITY --> EFFICIENCY
    DOCUMENTATION --> CLIPPY
    
    IDIOMS --> API_COMPAT
    SAFETY --> DEPENDENCIES
    EFFICIENCY --> BACKWARDS
    CLIPPY --> DEPLOYMENT
    
    classDef functionality fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef quality fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef rust fill:#e3f2fd,stroke:#1976d2,stroke-width:2px
    classDef integration fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class REQUIREMENTS,EDGE_CASES,PERFORMANCE,SECURITY functionality
    class READABILITY,MAINTAINABILITY,TESTABILITY,DOCUMENTATION quality
    class IDIOMS,SAFETY,EFFICIENCY,CLIPPY rust
    class API_COMPAT,DEPENDENCIES,BACKWARDS,DEPLOYMENT integration
```

## Development Tools

### Essential Tools Setup

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-watch      # Hot reload
cargo install cargo-audit      # Security audit
cargo install cargo-outdated   # Dependency updates
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-mutants    # Mutation testing
cargo install cargo-expand     # Macro expansion
cargo install cargo-bloat      # Binary size analysis

# Install IDE extensions (VS Code)
# - rust-analyzer
# - CodeLLDB (debugging)
# - Better TOML
# - Error Lens
```

### Development Workflow Commands

```bash
# Development server with hot reload
cargo watch -x run

# Run tests continuously
cargo watch -x test

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Check without building
cargo check

# Build optimized binary
cargo build --release

# Run specific test
cargo test test_message_creation

# Run integration tests
cargo test --test integration_test

# Generate documentation
cargo doc --open

# Analyze dependencies
cargo tree
```

## Debugging and Profiling

### Debugging Setup

```mermaid
graph TD
    subgraph "Debugging Tools"
        direction TB
        GDB[GDB/LLDB<br/>Native Debugging]
        VSCODE[VS Code<br/>Integrated Debugging]
        LOGS[Structured Logs<br/>tracing crate]
        METRICS[Metrics<br/>Performance Data]
    end
    
    subgraph "Profiling Tools"
        direction TB
        PERF[perf<br/>System Profiling]
        FLAMEGRAPH[Flamegraph<br/>CPU Profiling]
        VALGRIND[Valgrind<br/>Memory Analysis]
        HEAPTRACK[Heaptrack<br/>Heap Profiling]
    end
    
    subgraph "Monitoring"
        direction TB
        TOKIO_CONSOLE[tokio-console<br/>Async Runtime]
        PROMETHEUS[Prometheus<br/>Metrics Collection]
        JAEGER[Jaeger<br/>Distributed Tracing]
        GRAFANA[Grafana<br/>Visualization]
    end
    
    GDB --> PERF
    VSCODE --> FLAMEGRAPH
    LOGS --> VALGRIND
    METRICS --> HEAPTRACK
    
    PERF --> TOKIO_CONSOLE
    FLAMEGRAPH --> PROMETHEUS
    VALGRIND --> JAEGER
    HEAPTRACK --> GRAFANA
    
    classDef debug fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef profile fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef monitor fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class GDB,VSCODE,LOGS,METRICS debug
    class PERF,FLAMEGRAPH,VALGRIND,HEAPTRACK profile
    class TOKIO_CONSOLE,PROMETHEUS,JAEGER,GRAFANA monitor
```

### Performance Profiling

```bash
# CPU profiling with flamegraph
cargo install flamegraph
cargo flamegraph --bin campfire-rust

# Memory profiling with valgrind
valgrind --tool=massif target/debug/campfire-rust

# Async runtime monitoring
cargo install tokio-console
RUSTFLAGS="--cfg tokio_unstable" cargo run

# Benchmark specific functions
cargo bench --bench message_processing

# Profile binary size
cargo bloat --release
```

## Contributing Guidelines

### Contribution Process

```mermaid
graph TD
    subgraph "Contribution Flow"
        direction TB
        ISSUE[Create Issue<br/>Bug Report or Feature Request]
        DISCUSS[Discuss Approach<br/>Design Review]
        FORK[Fork Repository<br/>Create Branch]
        IMPLEMENT[Implement Changes<br/>TDD Approach]
        TEST[Test Changes<br/>All Test Types]
        PR[Create Pull Request<br/>Detailed Description]
        REVIEW[Code Review<br/>Feedback Loop]
        MERGE[Merge to Main<br/>Squash Commits]
    end
    
    subgraph "Quality Gates"
        direction TB
        TESTS_PASS[All Tests Pass<br/>CI Pipeline]
        LINT_PASS[Linting Pass<br/>cargo clippy]
        FORMAT_CHECK[Format Check<br/>cargo fmt]
        AUDIT_PASS[Security Audit<br/>cargo audit]
        COVERAGE[Coverage Maintained<br/>≥80%]
    end
    
    ISSUE --> DISCUSS
    DISCUSS --> FORK
    FORK --> IMPLEMENT
    IMPLEMENT --> TEST
    TEST --> PR
    PR --> REVIEW
    REVIEW --> MERGE
    
    TEST --> TESTS_PASS
    TEST --> LINT_PASS
    TEST --> FORMAT_CHECK
    TEST --> AUDIT_PASS
    TEST --> COVERAGE
    
    classDef flow fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef quality fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    
    class ISSUE,DISCUSS,FORK,IMPLEMENT,TEST,PR,REVIEW,MERGE flow
    class TESTS_PASS,LINT_PASS,FORMAT_CHECK,AUDIT_PASS,COVERAGE quality
```

### Commit Message Format

```
type(scope): brief description

Detailed explanation of the change, including:
- What was changed and why
- Any breaking changes
- References to issues

Examples:
feat(auth): add session token validation
fix(websocket): handle connection cleanup properly
docs(api): update search endpoint documentation
test(message): add property tests for deduplication
refactor(service): extract common validation logic
```

### Branch Naming Convention

```
feature/short-description    # New features
bugfix/issue-description     # Bug fixes
hotfix/critical-issue        # Critical production fixes
docs/documentation-update    # Documentation changes
refactor/code-improvement    # Code refactoring
test/test-improvements       # Test additions/improvements
```

## Environment Setup

### Local Development Environment

```bash
# Clone repository
git clone <repository-url>
cd campfire-rust-rewrite

# Set up environment
cp .env.example .env
# Edit .env with local configuration

# Install dependencies
cargo build

# Set up database
cargo run --bin migrate

# Run tests
cargo test

# Start development server
cargo run

# Or with hot reload
cargo watch -x run
```

### IDE Configuration

#### VS Code Settings

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
  "editor.formatOnSave": true,
  "editor.rulers": [100],
  "files.trimTrailingWhitespace": true,
  "files.insertFinalNewline": true
}
```

#### Debugging Configuration

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Campfire",
      "cargo": {
        "args": ["build", "--bin=campfire-rust"],
        "filter": {
          "name": "campfire-rust",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

This development guide provides comprehensive coverage of the development workflow, testing strategies, code quality standards, and contribution guidelines for the Campfire Rust rewrite project.