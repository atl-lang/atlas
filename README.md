# Atlas

**A long-term research and development project building a programming language designed for the AI era**

---

Atlas is a strict, typed, REPL-first programming language with a bytecode VM and a single cross-platform binary. It is designed to be AI-friendly, deterministic, and cohesive.

**This is not a rush-to-release project.** Atlas is being built carefully and thoughtfully to create something genuinely special - a language that could one day stand alongside Go, Rust, and other production languages. We're talking years of development, not months.

---

## 🚀 Quick Start for AI Agents

**→ Read `STATUS.md` first** - It tells you exactly what to do next

**→ Read `docs/DOCUMENTATION_PHILOSOPHY.md`** - Understand the project's long-term vision and quality-first approach

---

## 🎯 Project Philosophy

**Building for Decades, Not Months:**
- Quality over speed - Always
- Get the fundamentals right the first time
- No artificial deadlines or timeline pressure
- No compromise on architecture or design
- Build something that will last

**Comparison:**
- Go: 5 years to first stable version, 18+ years of evolution
- Rust: 9 years to first stable version, 19+ years of evolution
- Python: 15+ years to mainstream adoption

**Atlas is maybe 1-2 years old. We're in the foundation phase. Act like it.**

---

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

# Install locally for development
cargo install --path crates/atlas-cli
```

### Development Workflow
```bash
# Check formatting
cargo fmt --check

# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings
```

---

## 🔒 Code Quality & Security

### Security Auditing
```bash
# Install audit tools (one time)
cargo install cargo-audit cargo-deny

# Run security audit
cargo audit

# Check dependencies
cargo deny check
```

Run these regularly during development to maintain supply chain security and code quality.

### Continuous Integration

Atlas uses GitHub Actions for automated quality verification:

**CI Pipeline (runs on every commit):**
- **Test** - Runs on Ubuntu, macOS, and Windows
  - Unit tests (`cargo test`)
  - Doc tests
  - All features and targets
- **Format** - Ensures code formatting (`cargo fmt --check`)
- **Clippy** - Lints with strict warnings (`cargo clippy -- -D warnings`)
- **Security Audit** - Checks for vulnerabilities (`cargo audit`)

**Weekly Dependency Checks:**
- **Security Audit** - Scans for CVEs in dependencies
- **License Check** - Validates all licenses are approved
- **Outdated Check** - Identifies dependencies with updates

All checks must pass to maintain code quality standards.

---

## 📦 Dependencies

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

---

## 🤝 Development Guidelines

This is an internal research and development project. The focus is on building something exceptional, not rushing to any deadline.

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development workflow and standards
- Code quality requirements
- Architecture principles
- Testing requirements

Please note our [Code of Conduct](CODE_OF_CONDUCT.md) - we are committed to maintaining a respectful and focused development environment.

---

## 🔧 API Stability

Atlas is in active development. The runtime API is currently **unstable** and will evolve as we discover better approaches.

See [API-COMPATIBILITY.md](API-COMPATIBILITY.md) for:
- API evolution policy
- Breaking vs non-breaking change definitions
- Deprecation approach
- API design principles

**We prioritize getting the API right over maintaining backward compatibility during this development phase.**

---

## 📄 License

Atlas is licensed under the [MIT License](LICENSE).

**Rationale:** MIT was chosen for its permissive nature, allowing maximum flexibility while maintaining attribution requirements. This aligns with Atlas's goal of being AI-friendly and widely adoptable when ready.

---

## 📚 Core Documentation

### **AI-First Philosophy**
- `docs/AI-MANIFESTO.md` 🌟 **Read this first - Why Atlas exists**
- `docs/ai-workflow.md` - How AI agents use Atlas
- `docs/ai-principles.md` - Design principles
- `docs/why-strict.md` - Why strictness helps AI

### **Project Organization**
- `STATUS.md` ⭐ **Current implementation status (start here for AI agents)**
- `docs/DOCUMENTATION_PHILOSOPHY.md` 🌟 **Project vision and quality standards**
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

### **Future Features Under Exploration**
- `docs/modules.md` - Module system (when ready)
- `docs/ir.md` - Compiler IR design (exploring)
- `docs/language-comparison.md` - Atlas vs other languages

---

## 📂 Project Structure

### Phases
- Build order: `phases/BUILD-ORDER.md`
- Phase sections live in `phases/`
- Each phase is comprehensive work, not quick tasks

### Tests
- Layout: `tests/README.md`
- Run: `cargo test`
- Philosophy: Comprehensive, not just coverage

---

## ⚡ Current Status

See [STATUS.md](STATUS.md) for current implementation progress and next steps.

**Remember:** This project is being built to last decades. Every decision is made with long-term excellence in mind, not short-term deadlines.

**Build it right, not fast.**
