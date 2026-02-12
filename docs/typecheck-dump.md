# Atlas Typecheck Dump (AI-Friendly)

## Purpose
Provide a stable JSON representation of inferred types and symbol bindings for AI agents.

## Requirements
- Deterministic field ordering.
- Include spans for every symbol and type.
- Output format must be stable across versions.

## CLI
- `atlas typecheck path/to/file.atl --json`

## Schema (v0.1)
- Root: `{ "typecheck_version": 1, "symbols": [...], "types": [...] }`
- Symbols include name, kind, span, and inferred/declared type.

## Versioning
- JSON output includes `typecheck_version: 1`.
