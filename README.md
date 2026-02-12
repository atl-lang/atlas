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

### **Language & Implementation**
- `Atlas-SPEC.md` - Language specification
- `PRD.md` - Product requirements
- `docs/engineering.md` - Engineering standards
- `docs/implementation/` ⭐ **Implementation guides (start here for coding)**
- `docs/runtime.md`
- `docs/value-model.md`
- `docs/diagnostics.md`
- `docs/testing.md`
- `docs/repl.md`
- `docs/debug-info.md`
- `docs/versioning.md`
- `docs/style.md`
- `docs/runtime-api.md`
- `docs/runtime-api-evolution.md`
- `docs/prelude.md`
- `docs/warnings.md`
- `docs/prelude-test-plan.md`
- `docs/warnings-test-plan.md`
- `docs/bytecode-format.md`
- `docs/top-level-execution.md`
- `docs/operator-rules.md`
- `docs/string-semantics.md`
- `docs/array-aliasing.md`
- `docs/diagnostic-normalization.md`
- `docs/json-dump-stability.md`
- `docs/decision-log.md`
- `docs/coverage-matrix.md`
- `docs/phase-gates.md`
- `docs/numeric-edge-cases.md`
- `docs/keyword-policy.md`
- `docs/diagnostic-ordering.md`
- `docs/e2e-parity.md`
- `docs/repl-state.md`
- `docs/cli-e2e.md`

## Phases
- Build order: `phases/BUILD-ORDER.md`
- Phase sections live in `phases/`

## Tests
- Layout: `tests/README.md`
- Run: `cargo test`
