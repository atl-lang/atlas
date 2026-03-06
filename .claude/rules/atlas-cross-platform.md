---
paths:
  - "crates/**/tests/**"
  - "crates/**/src/**/*.rs"
---

# Cross-Platform Testing Rules

Auto-loaded when editing Rust source or test files.

- Use `std::path::Path` APIs, not string manipulation for paths.
- Use `Path::is_absolute()`, not `starts_with('/')`.
- Normalize separators in test assertions: `path.replace('\\', "/")`.
- Platform-specific test paths: use `#[cfg(unix)]` / `#[cfg(windows)]` helpers.
