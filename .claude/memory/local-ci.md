# Local CI Tracking

## Last Full Check
- **Timestamp:** 2026-03-03T00:45:00Z
- **Agent:** opus
- **Result:** pending (workflow just established)

## Batch Policy
- **Trigger:** 5 commits OR 24 hours (whichever comes first)
- **On trigger:** Haiku agent runs `act` + `coderabbit review`

## Pending Since Last Check
- Commits: 0
- Files changed: 0

## Quick Local Checks (run after every fix)
```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo nextest run -p atlas-runtime
```

## Full Local CI (Haiku agent, batched)
```bash
# 1. CodeRabbit review
coderabbit review --base main --plain

# 2. CI checks (same as GitHub Actions, no Docker needed)
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo build --workspace
cargo nextest run --workspace

# Optional: act (requires Docker/Colima/OrbStack)
# act -j Build -j Clippy -j Format --container-architecture linux/amd64
```

## Remote Push Policy
- **When:** After full local CI passes + batch threshold met
- **How:** Single PR with all accumulated fixes
- **Auto-merge:** Yes, CI already validated locally
