---
globs: [".github/workflows/**", "codecov.yml", "deny.toml"]
---

# Atlas CI Standards

Auto-loaded when touching CI configuration. Read before modifying any workflow file.

## Architecture

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | PR + main push | Core gate: fmt, clippy, supply-chain, test, coverage |
| `security.yml` | Cargo changes + weekly | cargo audit for CVEs |
| `bench.yml` | Runtime changes + main push | Criterion benchmarks, regression detection |

## ci.yml Job Dependency Chain

```
changes (path filter)
  └─> fmt
  └─> clippy
  └─> supply-chain (cargo deny check)
  └─> test (ubuntu + windows, parallel)
        └─> test-matrix (macos + beta, main push only)
        └─> coverage (PR + main push)
  └─> ci-success (gates on ALL of the above)
```

`ci-success` is the branch protection required check. If you add a new required job,
you MUST add it to `ci-success`'s needs array and env loop. Otherwise it can fail silently.

## Path Filter (docs-only fast path)

The `changes` job uses an exclusion model — everything except docs/images is `code`:

```yaml
- '**'
- '!**/*.md'
- '!docs/**'
- '!phases/**'
- '!**/*.png'
- '!**/*.jpg'
- '!**/*.svg'
```

**Never switch back to an explicit allowlist.** New config files (deny.toml, .cargo/config.toml,
build.rs, bench configs) would silently skip CI if missed from the list. The exclusion model
is safe by default — non-doc changes always run CI.

## Coverage Policy

Coverage is enforced by Codecov via `codecov.yml` in the repo root.

**Per-crate floors (enforced in GATE 6):**

| Crate | Floor |
|-------|-------|
| `atlas-runtime` | 70% |
| `atlas-cli` | 50% |
| `atlas-formatter` | 60% |
| `atlas-lsp` | 40% |
| `atlas-jit` | 25% |
| `atlas-config` | 60% |
| `atlas-build` | 40% |
| `atlas-package` | 40% |

**Patch coverage:** 80% of new lines must be covered.
**Project drop:** CI fails if total coverage drops more than 2% in a single PR.

Coverage runs on **main push only** — NOT on PRs. Tarpaulin is slow (~5 min); running it
on every PR would kill the hyper-paced block workflow. Codecov uses `carryforward: true`
to apply the last baseline to PR patch checks — PRs still get a Codecov status check,
just without re-running tarpaulin.
Tarpaulin excludes: `target/`, `*/tests/`, `*/benches/`, `*/fuzz/`.

## Benchmark Regression Policy

`bench.yml` uses `benchmark-action/github-action-benchmark` with:
- `alert-threshold: '115%'` — 15% regression from baseline = CI fails hard
- `fail-on-alert: true` — not a warning, a failure
- `comment-on-alert: true` — posts on the commit with details
- Baseline stored in `gh-pages` branch, auto-updated on each clean main push

**If you see a benchmark failure:** the regression is real. Investigate before merging.
Valid reasons to override: the benchmark itself was wrong. Performance regressions in
core paths are never acceptable without an architectural justification.

## Supply Chain Policy (`deny.toml`)

`[advisories]` field value types:
- `yanked` → `"deny" | "warn" | "allow"`
- `unsound` → `"all" | "workspace" | "transitive" | "none"` (scope, not severity)
- `unmaintained` → same scope values as unsound

Current policy: `yanked = "deny"`, `unsound = "all"`, `unmaintained = "none"`.
The audit job in security.yml handles CVE severity via `--deny unsound --deny yanked`.

## Verify Checklist (Before Merging Any CI Change)

- [ ] All required checks present: fmt, clippy, supply-chain, test, coverage
- [ ] `ci-success` needs array includes every required job
- [ ] Path filter uses exclusion model (not allowlist)
- [ ] No job accidentally skips on non-docs changes
- [ ] bench.yml has `fail-on-alert: true`
- [ ] Coverage job runs on PRs (not just main push)
- [ ] `CODECOV_TOKEN` secret is set in repo settings
