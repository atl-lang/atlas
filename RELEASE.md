# Atlas Release Process

This document defines the packaging and distribution strategy for Atlas releases.

## Version Scheme

Atlas follows [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH
```

- **MAJOR** - Breaking changes to syntax, semantics, or diagnostics
- **MINOR** - New features (backward compatible)
- **PATCH** - Bug fixes only

See `docs/versioning.md` for complete versioning policy.

## Release Artifact Naming

### Format

```
atlas-v{VERSION}-{OS}-{ARCH}[.ext]
```

### Examples

```
atlas-v0.1.0-linux-x86_64
atlas-v0.1.0-linux-aarch64
atlas-v0.1.0-macos-x86_64
atlas-v0.1.0-macos-aarch64
atlas-v0.1.0-windows-x86_64.exe
```

### Components

- **atlas** - Project name
- **v{VERSION}** - Full semantic version with 'v' prefix (e.g., v0.1.0)
- **{OS}** - Operating system: `linux`, `macos`, `windows`
- **{ARCH}** - Architecture: `x86_64`, `aarch64`
- **[.ext]** - Extension: `.exe` for Windows, none for Unix-like

## Supported Platforms

### Tier 1 (Guaranteed Support)

| OS | Architecture | Target Triple |
|----|--------------|---------------|
| Linux | x86_64 | x86_64-unknown-linux-gnu |
| macOS | x86_64 | x86_64-apple-darwin |
| macOS | aarch64 | aarch64-apple-darwin |
| Windows | x86_64 | x86_64-pc-windows-msvc |

### Tier 2 (Best Effort)

| OS | Architecture | Target Triple |
|----|--------------|---------------|
| Linux | aarch64 | aarch64-unknown-linux-gnu |

### Requirements

- **Rust:** 1.70 or later
- **MSRV Policy:** Minimum Supported Rust Version bumps require MINOR version increment
- **Libc:** glibc 2.17+ on Linux (for broad compatibility)

## Release Checklist

### Pre-Release

- [ ] All tests pass on all Tier 1 platforms
  ```bash
  cargo test --all-targets --all-features
  ```

- [ ] Clippy passes with no warnings
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```

- [ ] Code is formatted
  ```bash
  cargo fmt --check
  ```

- [ ] Security audit passes
  ```bash
  cargo audit
  cargo deny check
  ```

- [ ] Version number updated in:
  - [ ] `Cargo.toml` (workspace)
  - [ ] `CHANGELOG.md` (if exists)
  - [ ] Git tag created

- [ ] Documentation is current
  - [ ] README.md reflects new features
  - [ ] DEPENDENCIES.md is up to date
  - [ ] Implementation guides updated

### Build Release Binaries

For each Tier 1 platform:

```bash
# Set target
export TARGET=x86_64-unknown-linux-gnu

# Install target if needed
rustup target add $TARGET

# Build release binary
cargo build --release --target $TARGET

# Binary location
ls target/$TARGET/release/atlas
```

### Platform-Specific Build Commands

**Linux (x86_64):**
```bash
cargo build --release --target x86_64-unknown-linux-gnu
strip target/x86_64-unknown-linux-gnu/release/atlas
```

**macOS (x86_64):**
```bash
cargo build --release --target x86_64-apple-darwin
strip target/x86_64-apple-darwin/release/atlas
```

**macOS (aarch64):**
```bash
cargo build --release --target aarch64-apple-darwin
strip target/aarch64-apple-darwin/release/atlas
```

**Windows (x86_64):**
```bash
cargo build --release --target x86_64-pc-windows-msvc
# No strip on Windows - handled by rustc
```

### Package Artifacts

```bash
# Linux/macOS
cd target/x86_64-unknown-linux-gnu/release
tar czf atlas-v0.1.0-linux-x86_64.tar.gz atlas
shasum -a 256 atlas-v0.1.0-linux-x86_64.tar.gz > atlas-v0.1.0-linux-x86_64.tar.gz.sha256

# Windows
cd target/x86_64-pc-windows-msvc/release
7z a atlas-v0.1.0-windows-x86_64.zip atlas.exe
# Generate SHA256 checksum
certutil -hashfile atlas-v0.1.0-windows-x86_64.zip SHA256
```

### Testing Release Binaries

For each platform binary:

- [ ] Binary runs and shows version
  ```bash
  ./atlas --version
  ```

- [ ] REPL starts without errors
  ```bash
  ./atlas repl
  ```

- [ ] Can execute simple Atlas programs
  ```bash
  echo 'let x: number = 42; x;' | ./atlas run -
  ```

- [ ] Binary size is reasonable (< 10MB uncompressed)

### Publishing

- [ ] Create GitHub Release
  - [ ] Tag: `v{VERSION}`
  - [ ] Title: `Atlas v{VERSION}`
  - [ ] Release notes from CHANGELOG
  - [ ] Upload all platform binaries
  - [ ] Upload SHA256 checksums

- [ ] Update documentation
  - [ ] Installation instructions
  - [ ] Binary download links
  - [ ] Platform compatibility matrix

### Post-Release

- [ ] Verify GitHub release page
- [ ] Test download links
- [ ] Announce release (if applicable)
- [ ] Monitor for issues

## Binary Size Optimization

Current strategies:
- `strip` symbols on Unix platforms
- `opt-level = "z"` in release profile (optional)
- LTO enabled for smaller binaries
- Panic = abort to reduce unwinding code

**Target Size:** < 5MB compressed, < 10MB uncompressed for v0.1

## Code Signing (Future)

Not implemented in v0.1, but planned:

- **macOS:** Sign with Apple Developer ID
- **Windows:** Sign with Authenticode certificate
- **Linux:** GPG signature of artifacts

## Future Automation

CI automation planned for:
- Cross-compilation on GitHub Actions
- Automatic artifact generation
- SHA256 checksum generation
- Draft release creation
- Binary upload

See `.github/workflows/release.yml` (to be created) for automation strategy.

## Distribution Channels

### v0.1 (Current)

- GitHub Releases (primary)
- Manual download and installation

### Future

- Homebrew formula (macOS/Linux)
- Cargo install (from crates.io)
- Scoop bucket (Windows)
- APT/RPM repositories (Linux)
- Docker images

## Minimum Viable Release

For v0.1, the following is sufficient:

1. Build binaries for Tier 1 platforms
2. Create GitHub release with binaries
3. Include SHA256 checksums
4. Update README with download links

Automation and additional distribution channels can be added incrementally.

---

**Document Version:** 1.0
**Last Updated:** 2026-02-12
**Next Review:** Before v0.1.0 release
