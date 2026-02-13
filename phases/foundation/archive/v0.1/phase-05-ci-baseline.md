# Phase 05 - CI Baseline

## Objective
Set up a minimal CI pipeline for tests and formatting.

## Inputs
- `docs/guides/code-quality-standards.md`

## Deliverables
- CI config (GitHub Actions or equivalent).
- Steps: `cargo fmt --check`, `cargo test`.

## Steps
- Add a CI workflow file.
- Ensure the workflow runs on pushes and PRs.

## Exit Criteria
- CI passes on main branch.
