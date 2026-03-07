# Tracking Archive

## Context

On 2026-03-07 the project migrated from `atlas-track` (project-local CLI) to `pt` (project-tracker — centralized multi-project CLI).

## Files

- `atlas-track-legacy-YYYYMMDD.db` — Final state of the atlas-track SQLite DB before migration. All issues, decisions, sessions, and blocks through S-116 / B9 complete.
- `pt-snapshot-YYYYMMDD.db` — Snapshot of the pt DB at migration time. This is the same data migrated into the centralized `~/.project-tracker/atlas/tracking.db`.

## Why pt?

- Centralized across all projects (`atlas`, `dashhq`, `ainanny`, `project-tracker`)
- Same CLI commands — `atlas-track X` → `pt X` (exact same names)
- GUI monitoring across all projects from one dashboard
- `~/.project-tracker/handoffs/<slug>-handoff.md` — per-project handoffs in one place

## Live DB

The active tracking DB is at: `~/.project-tracker/atlas/tracking.db`
The CLI is `~/bin/pt` (or `pt` if `~/bin` is on PATH).

## If Something Goes Wrong

1. The legacy atlas-track DB is complete and can be re-read with `sqlite3 atlas-track-legacy-*.db`
2. The atlas-track CLI still exists at `~/.local/bin/atlas-track` (not removed)
3. All data was preserved in pt — nothing was deleted
