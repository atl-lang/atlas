# Contributing to Atlas

Thank you for your interest in contributing to Atlas! This document provides guidelines for development.

## Development Setup

### Prerequisites
- Rust 1.70 or later
- Cargo (comes with Rust)

### Getting Started
```bash
# Clone the repository
git clone https://github.com/your-org/atlas.git
cd atlas

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run --bin atlas -- --version
```

## Code Standards

### Formatting
We use `rustfmt` with minimal configuration. All code must be formatted before committing:

```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt
```

### Linting
We use `clippy` to catch common mistakes and enforce best practices:

```bash
# Run clippy
cargo clippy -- -D warnings

# Run clippy with all features
cargo clippy --all-targets --all-features -- -D warnings
```

**Clippy Expectations:**
- All clippy warnings must be addressed (no `allow` unless justified)
- No `unsafe` code in v0.1 without explicit approval
- Prefer standard library solutions over external dependencies
- Keep functions focused and under 100 lines where practical

### Testing
All new features and bug fixes must include tests:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

**Testing Guidelines:**
- Unit tests for lexer/parser/typechecker/interpreter
- Golden tests for end-to-end behavior (input → output)
- Keep snapshots small and deterministic
- Avoid flaky tests (no time-based assertions)

## Code Style

Follow the guidelines in `docs/style.md`:

- **Modules:** `snake_case`
- **Types:** `PascalCase`
- **Functions:** `snake_case`
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Files:** One primary module per file, avoid files > 400 lines
- **Comments:** Only add comments for non-obvious logic

## Architecture

See `docs/engineering.md` for detailed architecture guidelines:

- `atlas-runtime` is library-first (no CLI logic)
- `atlas-cli` is a thin wrapper around runtime APIs
- All errors flow through the `Diagnostic` type
- Every diagnostic must include span, code, and message

## API Compatibility

When making changes to the runtime API (anything in `atlas-runtime`):

- **Read [API-COMPATIBILITY.md](API-COMPATIBILITY.md) first**
- Follow semantic versioning strictly
- Use the API change checklist
- Consider backward compatibility
- Document deprecations properly
- Add migration guides for breaking changes

**Key Rules:**
- Breaking changes require MAJOR version bump
- New features require MINOR version bump
- Bug fixes require PATCH version bump
- Deprecations must have 1 MINOR version window

## Security

### Dependency Auditing
Before submitting PRs or releases, run security audits:

```bash
# Install cargo-audit (one time)
cargo install cargo-audit

# Run audit
cargo audit

# Install cargo-deny (one time)
cargo install cargo-deny

# Check dependencies
cargo deny check
```

### Dependency Policy
- Keep external dependencies minimal and vetted
- Justify all new dependencies in PR description
- Prefer crates with:
  - Active maintenance
  - Good test coverage
  - Permissive licenses (MIT/Apache-2.0)

## Pull Request Process

1. **Create a branch** from `main`
2. **Make your changes** following the code standards
3. **Add tests** for new functionality
4. **Run checks** before submitting:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   cargo audit
   ```
5. **Write clear commit messages** explaining the "why" not just the "what"
6. **Submit PR** with description of changes and any relevant context
7. **Wait for CI** - All checks must pass:
   - Tests on Ubuntu, macOS, and Windows
   - Formatting check
   - Clippy lints
   - Security audit

## Questions?

- Check existing documentation in `docs/`
- Open an issue for clarification
- Read `docs/ai-workflow.md` for AI-assisted development patterns

---

Thank you for contributing to Atlas! 🚀
