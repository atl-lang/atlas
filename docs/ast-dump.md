# Atlas AST Dump (AI-Friendly)

## Purpose
Provide a stable JSON representation of the AST for AI agents and tooling.

## Requirements
- Deterministic field ordering.
- Include spans for every node.
- Output format must be stable across versions.

## CLI
- `atlas ast path/to/file.atl --json`

## Schema (v0.1)
- Root: `{ "kind": "Program", "items": [...] }`
- Each node: `{ "kind": "NodeType", "span": {...}, ... }`

## Versioning
- JSON output includes `ast_version: 1`.
