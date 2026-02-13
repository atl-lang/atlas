# Atlas

[![CI](https://github.com/YOUR_ORG/atlas/workflows/CI/badge.svg)](https://github.com/YOUR_ORG/atlas/actions/workflows/ci.yml)
[![Dependencies](https://github.com/YOUR_ORG/atlas/workflows/Dependencies/badge.svg)](https://github.com/YOUR_ORG/atlas/actions/workflows/dependencies.yml)

Atlas is a strict, typed, REPL-first programming language with a bytecode VM and a single cross-platform binary. It is designed to be AI-friendly, deterministic, and cohesive.

## 🚀 Quick Start for AI Agents

**→ Read `STATUS.md` first** - It tells you exactly what to do next

## 🛠️ Building and Running

### Prerequisites
- Rust 1.70 or later
- Cargo (comes with Rust)

### Build
```bash
cargo build --release
```

### Run
```bash
# Run tests
cargo test

# Build and run the CLI
cargo run --bin atlas -- --version

# Install locally
cargo install --path crates/atlas-cli
```

### Development
```bash
# Check formatting
cargo fmt --check

# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings
```

### Security Auditing
```bash
# Install audit tools (one time)
cargo install cargo-audit cargo-deny

# Run security audit
cargo audit

# Check dependencies
cargo deny check
```

Run these before submitting PRs or creating releases to ensure supply chain security.

## Continuous Integration

Atlas uses GitHub Actions for automated testing and quality checks:

### CI Pipeline (runs on every push/PR)
- **Test** - Runs on Ubuntu, macOS, and Windows
  - Unit tests (`cargo test`)
  - Doc tests
  - All features and targets
- **Format** - Ensures code formatting (`cargo fmt --check`)
- **Clippy** - Lints with strict warnings (`cargo clippy -- -D warnings`)
- **Security Audit** - Checks for vulnerabilities (`cargo audit`)

### Weekly Dependency Checks
- **Security Audit** - Scans for CVEs in dependencies
- **License Check** - Validates all licenses are approved
- **Outdated Check** - Identifies dependencies with updates

All checks must pass before merging to `main`.

## Dependencies

Atlas maintains a minimal dependency footprint. See [DEPENDENCIES.md](DEPENDENCIES.md) for complete dependency documentation, rationale, and alternatives considered.

### Core Dependencies Summary

| Crate | Version | Purpose |
|-------|---------|---------|
| **Runtime (atlas-runtime)** |
| thiserror | 2.0 | Error type derivation |
| serde + serde_json | 1.0 | JSON diagnostics & bytecode serialization |
| insta | 1.40 | Snapshot testing (dev-only) |
| **CLI (atlas-cli)** |
| clap | 4.5 | Command-line argument parsing |
| rustyline | 14.0 | REPL line editing |
| anyhow | 1.0 | CLI error handling |

**Note:** No parser generators used - hand-written lexer/parser for full control.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines, code standards, and PR process.

Please note our [Code of Conduct](CODE_OF_CONDUCT.md) - we are committed to providing a welcoming environment for all contributors.

## API Compatibility

Atlas maintains strict backward compatibility guarantees for the runtime API. See [API-COMPATIBILITY.md](API-COMPATIBILITY.md) for:
- Version policy and semantic versioning rules
- Breaking vs non-breaking change definitions
- Deprecation policy and timeline
- API change checklist

**Stability:** The runtime API is currently **unstable** (v0.x.x). Expect changes before v1.0.0.

## Release & Distribution

See [RELEASE.md](RELEASE.md) for:
- Release artifact naming conventions
- Supported platforms and architectures
- Build and packaging instructions
- Release checklist and procedures

Atlas releases single-binary executables for Linux, macOS, and Windows via GitHub Releases.

## License

Atlas is licensed under the [MIT License](LICENSE).

**Rationale:** MIT was chosen for its permissive nature, allowing maximum flexibility for users and contributors while maintaining attribution requirements. This aligns with Atlas's goal of being AI-friendly and widely adoptable.

## Core Docs

### **AI-First Philosophy**
- `docs/AI-MANIFESTO.md` 🌟 **Read this first - Why Atlas exists**
- `docs/ai-workflow.md` - How AI agents use Atlas
- `docs/ai-principles.md` - Design principles
- `docs/why-strict.md` - Why strictness helps AI

### **Project Organization**
- `STATUS.md` ⭐ **Current implementation status (start here for AI agents)**
- `docs/AI_AGENT_CHECKLIST.md` - Pre/post phase verification steps
- `docs/CODE_ORGANIZATION.md` - File size limits and refactoring rules

### **Language & Implementation**
- `Atlas-SPEC.md` - Language specification
- `PRD.md` - Product requirements
- `docs/engineering.md` - Engineering standards
- `docs/implementation/` ⭐ **Implementation guides (start here for coding)**

### **Core Components**
- `docs/runtime.md` - Runtime architecture
- `docs/value-model.md` - Value representation
- `docs/diagnostics.md` - Diagnostic system
- `docs/testing.md` - Testing strategy
- `docs/versioning.md` - Versioning policy

### **Frontend (Lexer, Parser, AST)**
- `docs/ast.md` - AST structure
- `docs/ast-dump.md` - AST JSON dump format
- `docs/typecheck-dump.md` - Type checker JSON dump format
- `docs/GRAMMAR_CONFORMANCE.md` - Grammar conformance testing
- `docs/parser-recovery-policy.md` - Error recovery strategy
- `docs/keyword-policy.md` - Keyword management

### **Type System & Semantics**
- `docs/warnings.md` - Warning diagnostics
- `docs/operator-rules.md` - Operator type rules
- `docs/string-semantics.md` - String behavior
- `docs/array-aliasing.md` - Array mutation semantics
- `docs/numeric-edge-cases.md` - Numeric edge cases
- `docs/diagnostic-normalization.md` - Diagnostic consistency
- `docs/diagnostic-ordering.md` - Diagnostic ordering
- `docs/top-level-execution.md` - Top-level execution order

### **Runtime & VM**
- `docs/bytecode-format.md` - Bytecode specification
- `docs/debug-info.md` - Debug information format
- `docs/runtime-api.md` - Runtime API
- `docs/runtime-api-evolution.md` - API evolution policy

### **Standard Library & Prelude**
- `docs/stdlib.md` - Standard library specification
- `docs/prelude.md` - Prelude built-ins
- `docs/io-security-model.md` - I/O security boundaries

### **CLI & REPL**
- `docs/repl.md` - REPL implementation
- `docs/repl-modes.md` - REPL operation modes
- `docs/repl-state.md` - REPL state management
- `docs/cli-config.md` - CLI configuration
- `docs/cli-e2e.md` - CLI end-to-end testing

### **Quality & Process**
- `docs/coverage-matrix.md` - Spec to phase mapping
- `docs/phase-gates.md` - Phase exit criteria
- `docs/json-dump-stability.md` - JSON dump stability
- `docs/e2e-parity.md` - Interpreter/VM parity testing
- `docs/decision-log.md` - Technical decisions
- `docs/style.md` - Code style guide

### **Future Features**
- `docs/modules.md` - Module system (v1.0+)
- `docs/ir.md` - Compiler IR design (future)
- `docs/language-comparison.md` - Atlas vs other languages

## Phases
- Build order: `phases/BUILD-ORDER.md`
- Phase sections live in `phases/`

## Tests
- Layout: `tests/README.md`
- Run: `cargo test`
