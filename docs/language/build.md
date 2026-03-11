# Building Atlas Programs

## Overview

`atlas build` compiles your Atlas project into a native OS executable — a real
Mach-O (macOS), ELF (Linux), or PE (Windows) binary that runs directly on the OS
without requiring Atlas or Rust to be installed.

## Quick Start

```toml
# atlas.toml
[package]
name = "myapp"
version = "0.1.0"

[[bin]]
name = "myapp"
path = "src/main.atl"
```

```sh
atlas build           # → target/debug/myapp
atlas build --release # → target/release/myapp
./target/debug/myapp  # run directly — no atlas CLI needed
```

## How It Works

Atlas uses the **self-appending launcher** pattern (D-048):

```
[atlas-launcher binary] + [module archive] + [ATLAS_BC_MAGIC] + [payload_len]
= ./myapp  (real OS executable)
```

1. `atlas build` compiles each source file to Atlas bytecode
2. All module bytecodes are packaged as a multi-module archive
3. The `atlas-launcher` binary (the VM runtime) is prepended
4. The result is a native OS binary that you can `scp`, distribute, or install

The binary is self-contained — the Atlas VM is inside it. No runtime dependency.

## atlas.toml Reference

### Binary target (`[[bin]]`)

```toml
[[bin]]
name = "myapp"      # output binary name (target/debug/myapp on Unix)
path = "src/main.atl"  # entry point (must contain main function)
```

### Library target (`[lib]`)

```toml
[lib]
name = "mylib"
path = "src/lib.atl"
```

Libraries produce `target/debug/lib/mylib.atl.bc` — bytecode archives for use
as dependencies, not standalone executables.

## Build Profiles

| Profile | Command | Optimizations |
|---------|---------|---------------|
| dev | `atlas build` | None (fast compile) |
| release | `atlas build --release` | O2 (slower compile, faster binary) |

## Requirements

- `atlas-launcher` must be installed alongside the `atlas` binary
- Running `cargo install atlas-cli` installs both automatically
- If `atlas build` reports **AT2020 LAUNCHER_NOT_FOUND**, reinstall: `cargo install atlas-cli`

## Distribution

The produced binary is fully self-contained:

```sh
# Build release binary
atlas build --release

# Distribute it — no Atlas required on the target machine
cp target/release/myapp /usr/local/bin/
scp target/release/myapp user@server:/usr/local/bin/
```

The binary includes the Atlas VM. End users never need to install anything.
