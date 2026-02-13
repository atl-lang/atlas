# Atlas v0.1.0 Release Checklist

**Target Release:** v0.1.0
**Release Manager:** TBD
**Target Date:** TBD
**Status:** In Progress

---

## Overview

This checklist ensures all release criteria are met for Atlas v0.1.0. Complete each section in order, checking off items as they're verified.

**Checklist Completion:** 0/100 items

---

## Phase 1: Pre-Release Verification (Critical)

### Documentation Review ✅ [COMPLETE]

- [x] STATUS.md reflects current implementation status
- [x] README.md is current and accurate
- [x] All documentation cross-references verified (no broken links)
- [x] Atlas-SPEC.md is finalized for v0.1
- [x] Implementation guides are complete
- [x] CONTRIBUTING.md is current
- [x] CODE_OF_CONDUCT.md is present
- [x] LICENSE file is present (MIT)

**Evidence:** Phase Polish-03 completed - documentation audit passed

### Test Stability ✅ [COMPLETE]

- [x] All tests passing (1,391 tests)
- [x] Zero flaky tests detected
- [x] Test suite run 3+ times with identical results
- [x] No panics or crashes in test suite
- [x] Snapshot tests all passing
- [x] Diagnostic format consistency verified

**Evidence:** Phase Polish-04 completed - stability audit report generated

### Code Quality (Pre-Flight)

- [ ] All compiler warnings resolved
- [ ] Clippy passes with zero warnings
  ```bash
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  ```
- [ ] Code formatting verified
  ```bash
  cargo fmt --check
  ```
- [ ] No TODO or FIXME comments in release-critical code
- [ ] All `#[ignore]` tests are intentional and documented

### Security Audit

- [ ] `cargo audit` passes (no known vulnerabilities)
  ```bash
  cargo audit
  ```
- [ ] `cargo deny check` passes (license compliance)
  ```bash
  cargo deny check
  ```
- [ ] DEPENDENCIES.md is up to date
- [ ] No unsafe code in critical paths (or documented justification)
- [ ] No hardcoded credentials or sensitive data

### Component Version Verification

- [ ] Runtime VERSION = 0.1.0 (`crates/atlas-runtime/src/lib.rs`)
- [ ] AST_VERSION = 1 (`crates/atlas-runtime/src/ast.rs`)
- [ ] DIAG_VERSION = 1 (`crates/atlas-runtime/src/diagnostic.rs`)
- [ ] BYTECODE_VERSION = 1 (`crates/atlas-runtime/src/bytecode/mod.rs`)
- [ ] TYPECHECK_VERSION = 1 (`crates/atlas-runtime/src/typecheck_dump.rs`)
- [ ] All component versions documented in `docs/versioning.md`

---

## Phase 2: Functional Verification

### Core Functionality Tests

- [ ] Lexer: All tokens recognized correctly
- [ ] Parser: Valid Atlas programs parse without errors
- [ ] Binder: Symbol resolution works for all scopes
- [ ] Type Checker: Type inference and checking operational
- [ ] Interpreter: Can execute all spec-compliant programs
- [ ] VM: Bytecode execution matches interpreter results
- [ ] Diagnostics: Error messages are actionable and consistent

### Standard Library Verification

- [ ] All prelude functions available: `len`, `toString`, `print`, `readLine`, `toNumber`
- [ ] Built-in functions work correctly
- [ ] Standard library tests passing
- [ ] Prelude binding tests passing

### CLI Functionality

- [ ] `atlas --version` displays correct version
- [ ] `atlas run <file>` executes Atlas programs
- [ ] `atlas repl` starts interactive REPL
- [ ] `atlas ast` dumps AST correctly
- [ ] `atlas typecheck` reports type errors
- [ ] `atlas --help` shows usage information
- [ ] Diagnostic output is properly formatted
- [ ] Error codes are correct (AT####)

### LSP Functionality

- [ ] LSP server starts without errors
- [ ] Diagnostics published to editor
- [ ] Go to definition works
- [ ] Code completion functional
- [ ] Document formatting works
- [ ] LSP tests passing

### REPL Tests

- [ ] REPL starts without errors
- [ ] Can execute expressions
- [ ] Can declare variables and functions
- [ ] Multi-line input works
- [ ] Error handling in REPL functional
- [ ] Exit commands work (Ctrl+D, :quit)

---

## Phase 3: Cross-Platform Testing

### Platform Build Matrix

#### Tier 1 Platforms (Required)

- [ ] **Linux x86_64** (`x86_64-unknown-linux-gnu`)
  - [ ] Build succeeds
  - [ ] All tests pass
  - [ ] Binary runs
  - [ ] REPL functional
  - [ ] Can execute sample programs

- [ ] **macOS x86_64** (`x86_64-apple-darwin`)
  - [ ] Build succeeds
  - [ ] All tests pass
  - [ ] Binary runs
  - [ ] REPL functional
  - [ ] Can execute sample programs

- [ ] **macOS aarch64** (`aarch64-apple-darwin`)
  - [ ] Build succeeds
  - [ ] All tests pass
  - [ ] Binary runs
  - [ ] REPL functional
  - [ ] Can execute sample programs

- [ ] **Windows x86_64** (`x86_64-pc-windows-msvc`)
  - [ ] Build succeeds
  - [ ] All tests pass
  - [ ] Binary runs
  - [ ] REPL functional
  - [ ] Can execute sample programs

#### Tier 2 Platforms (Best Effort)

- [ ] **Linux aarch64** (`aarch64-unknown-linux-gnu`)
  - [ ] Build succeeds (if possible)
  - [ ] Basic functionality verified

### Cross-Platform Issues

- [ ] No platform-specific bugs identified
- [ ] Line ending handling correct (CRLF vs LF)
- [ ] Path separators handled correctly
- [ ] File I/O works on all platforms

---

## Phase 4: Version Bumping

### Update Version Numbers

- [ ] Update `Cargo.toml` workspace version to `0.1.0`
- [ ] Update `docs/versioning.md` current version to `0.1.0`
- [ ] Update `RELEASE.md` if needed
- [ ] Verify all crates inherit workspace version

### Create CHANGELOG (if not exists)

- [ ] Create `CHANGELOG.md` following Keep a Changelog format
- [ ] Document all features in v0.1.0
- [ ] Document known limitations
- [ ] Document breaking changes from any pre-releases

### Git Preparation

- [ ] All changes committed
- [ ] Working directory clean (`git status`)
- [ ] Main branch up to date
- [ ] No uncommitted files

---

## Phase 5: Build Release Binaries

### Build Configuration

- [ ] Release profile configured in `Cargo.toml`
  - [ ] `opt-level = 3`
  - [ ] `lto = true`
  - [ ] `codegen-units = 1`
  - [ ] `strip = true`
  - [ ] `panic = "abort"`

### Build Each Platform Binary

#### Linux x86_64
```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
strip target/x86_64-unknown-linux-gnu/release/atlas
```
- [ ] Build successful
- [ ] Binary size reasonable (< 10MB)
- [ ] Binary executes on target platform

#### macOS x86_64
```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
strip target/x86_64-apple-darwin/release/atlas
```
- [ ] Build successful
- [ ] Binary size reasonable (< 10MB)
- [ ] Binary executes on target platform

#### macOS aarch64
```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
strip target/aarch64-apple-darwin/release/atlas
```
- [ ] Build successful
- [ ] Binary size reasonable (< 10MB)
- [ ] Binary executes on target platform

#### Windows x86_64
```bash
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```
- [ ] Build successful
- [ ] Binary size reasonable (< 10MB)
- [ ] Binary executes on target platform

---

## Phase 6: Package Artifacts

### Create Archives

- [ ] **Linux x86_64**
  ```bash
  cd target/x86_64-unknown-linux-gnu/release
  tar czf atlas-v0.1.0-linux-x86_64.tar.gz atlas
  shasum -a 256 atlas-v0.1.0-linux-x86_64.tar.gz > atlas-v0.1.0-linux-x86_64.tar.gz.sha256
  ```

- [ ] **macOS x86_64**
  ```bash
  cd target/x86_64-apple-darwin/release
  tar czf atlas-v0.1.0-macos-x86_64.tar.gz atlas
  shasum -a 256 atlas-v0.1.0-macos-x86_64.tar.gz > atlas-v0.1.0-macos-x86_64.tar.gz.sha256
  ```

- [ ] **macOS aarch64**
  ```bash
  cd target/aarch64-apple-darwin/release
  tar czf atlas-v0.1.0-macos-aarch64.tar.gz atlas
  shasum -a 256 atlas-v0.1.0-macos-aarch64.tar.gz > atlas-v0.1.0-macos-aarch64.tar.gz.sha256
  ```

- [ ] **Windows x86_64**
  ```bash
  cd target/x86_64-pc-windows-msvc/release
  # Use 7z or PowerShell Compress-Archive
  7z a atlas-v0.1.0-windows-x86_64.zip atlas.exe
  certutil -hashfile atlas-v0.1.0-windows-x86_64.zip SHA256 > atlas-v0.1.0-windows-x86_64.zip.sha256
  ```

### Verify Archives

- [ ] All archives created successfully
- [ ] All SHA256 checksums generated
- [ ] Archives extract correctly
- [ ] Binaries in archives are executable
- [ ] Archive sizes reasonable (< 5MB compressed)

---

## Phase 7: Release Binary Testing

### Smoke Tests (Each Platform)

For each platform binary:

- [ ] Binary shows correct version
  ```bash
  ./atlas --version
  # Expected: Atlas 0.1.0
  ```

- [ ] Help command works
  ```bash
  ./atlas --help
  ```

- [ ] REPL starts and accepts input
  ```bash
  ./atlas repl
  # > let x: number = 42;
  # > x;
  # Expected: 42
  ```

- [ ] Can execute simple program
  ```bash
  echo 'let x: number = 42; x;' | ./atlas run -
  # Expected: 42
  ```

- [ ] Can execute file
  ```bash
  echo 'let x: number = 42; x;' > test.atl
  ./atlas run test.atl
  # Expected: 42
  ```

- [ ] Error handling works
  ```bash
  echo 'let x: number = "wrong";' | ./atlas run -
  # Expected: Type error AT1012
  ```

---

## Phase 8: Create GitHub Release

### Git Tagging

- [ ] Create annotated tag
  ```bash
  git tag -a v0.1.0 -m "Atlas v0.1.0 - Initial Release"
  ```

- [ ] Push tag to GitHub
  ```bash
  git push origin v0.1.0
  ```

### GitHub Release Page

- [ ] Create new release on GitHub
- [ ] Tag: `v0.1.0`
- [ ] Release title: `Atlas v0.1.0 - Initial Release`
- [ ] Upload all platform binaries:
  - [ ] `atlas-v0.1.0-linux-x86_64.tar.gz`
  - [ ] `atlas-v0.1.0-macos-x86_64.tar.gz`
  - [ ] `atlas-v0.1.0-macos-aarch64.tar.gz`
  - [ ] `atlas-v0.1.0-windows-x86_64.zip`
- [ ] Upload all SHA256 checksums:
  - [ ] `atlas-v0.1.0-linux-x86_64.tar.gz.sha256`
  - [ ] `atlas-v0.1.0-macos-x86_64.tar.gz.sha256`
  - [ ] `atlas-v0.1.0-macos-aarch64.tar.gz.sha256`
  - [ ] `atlas-v0.1.0-windows-x86_64.zip.sha256`

### Release Notes

Create comprehensive release notes including:

- [ ] Overview of Atlas v0.1.0
- [ ] Key features
- [ ] Installation instructions
- [ ] Quick start guide
- [ ] Known limitations
- [ ] Supported platforms
- [ ] Link to documentation
- [ ] SHA256 checksums in release notes
- [ ] Credits and acknowledgments

---

## Phase 9: Documentation Updates

### Update README.md

- [ ] Add installation section with download links
- [ ] Link to GitHub releases page
- [ ] Add quick start example
- [ ] Ensure version numbers are current

### Update Website/Docs (if applicable)

- [ ] Update any external documentation
- [ ] Update installation guides
- [ ] Publish API documentation
- [ ] Update compatibility matrix

---

## Phase 10: Post-Release Verification

### Download Verification

- [ ] Download each binary from GitHub releases
- [ ] Verify SHA256 checksums match
- [ ] Verify binaries execute correctly
- [ ] Test on fresh machine (no dev environment)

### Issue Monitoring

- [ ] Monitor GitHub issues for release problems
- [ ] Respond to installation issues within 24 hours
- [ ] Document any platform-specific problems
- [ ] Prepare hotfix if critical bugs found

### Community Communication

- [ ] Announce release (if applicable)
  - [ ] GitHub Discussions
  - [ ] Social media
  - [ ] Relevant forums/communities
- [ ] Share release notes
- [ ] Provide support channels

---

## Phase 11: Cleanup & Next Steps

### Post-Release Tasks

- [ ] Archive release logs and artifacts
- [ ] Update project roadmap
- [ ] Plan v0.2.0 features
- [ ] Document lessons learned
- [ ] Update STATUS.md for next development cycle

### Known Issues Documentation

- [ ] Document any known issues in GitHub Issues
- [ ] Label issues by severity
- [ ] Assign to milestones (v0.1.1, v0.2.0, etc.)
- [ ] Update KNOWN_ISSUES.md if it exists

---

## Rollback Plan

If critical issues are found after release:

1. **Immediate Actions:**
   - [ ] Remove binary downloads from GitHub releases
   - [ ] Add warning to release notes
   - [ ] Create GitHub issue documenting the problem

2. **Hotfix Process:**
   - [ ] Create hotfix branch from v0.1.0 tag
   - [ ] Fix critical issue
   - [ ] Run full test suite
   - [ ] Build new binaries
   - [ ] Release as v0.1.1

3. **Communication:**
   - [ ] Notify users of the issue
   - [ ] Provide workaround if available
   - [ ] Announce hotfix release

---

## Release Metrics

### Target Metrics for v0.1.0

- **Binary Size:** < 10MB uncompressed, < 5MB compressed
- **Test Coverage:** > 90% (estimated from 1,391 passing tests)
- **Build Time:** < 5 minutes (release builds)
- **Test Time:** < 2 minutes (full suite)
- **Documentation:** 100% of public APIs documented
- **Platform Coverage:** 4 Tier 1 platforms (Linux x86_64, macOS x86_64, macOS aarch64, Windows x86_64)

### Actual Metrics (To be filled at release)

- **Binary Size:**
  - Linux x86_64: ___ MB (compressed: ___ MB)
  - macOS x86_64: ___ MB (compressed: ___ MB)
  - macOS aarch64: ___ MB (compressed: ___ MB)
  - Windows x86_64: ___ MB (compressed: ___ MB)
- **Test Coverage:** ___% (1,391 tests passing)
- **Build Time:** ___ minutes
- **Test Time:** ___ minutes
- **Total Downloads (first week):** ___

---

## Approval Sign-Off

### Required Approvals

- [ ] **Technical Lead:** All tests passing, code quality verified
- [ ] **Documentation Lead:** All docs reviewed and current
- [ ] **Release Manager:** Binaries built and tested on all platforms
- [ ] **Security Reviewer:** Security audit passed, no critical vulnerabilities

### Final Go/No-Go Decision

- [ ] **GO for v0.1.0 Release** _(date: ___)_
- [ ] All critical checklist items complete
- [ ] No known release-blocking bugs
- [ ] Platform coverage meets requirements
- [ ] Release notes ready

**Release Manager Signature:** _______________
**Date:** _______________

---

## Quick Reference Commands

### Pre-Release Checks
```bash
# Run all tests
cargo test --workspace --all-features

# Check code quality
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check

# Security audit
cargo audit
cargo deny check

# Verify versions
grep "version" Cargo.toml
```

### Build All Platforms
```bash
# Script to build all Tier 1 platforms
for target in x86_64-unknown-linux-gnu x86_64-apple-darwin aarch64-apple-darwin x86_64-pc-windows-msvc; do
  echo "Building $target..."
  cargo build --release --target $target
done
```

### Verify Binary
```bash
# Quick smoke test
./atlas --version
./atlas repl <<< 'let x: number = 42; x;'
echo 'let x: number = 42; x;' | ./atlas run -
```

---

**Checklist Version:** 1.0
**Created:** 2026-02-13
**Last Updated:** 2026-02-13
**For Release:** v0.1.0
