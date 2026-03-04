# LSP Patterns

*DRs 001–006 archived to `archive/2026-02-decisions-lsp-v1.md`.*

## P-LSP-007: Workspace Symbol Performance Optimizations

**Context:** Phase 05C - large workspace performance

**Decision:** Three-tier optimization strategy:
1. **LRU Query Cache** - Cache search results (key: query + kind + limit as strings)
2. **Memory Bounds** - Max 100k symbols, evict oldest document when exceeded
3. **Batch Indexing** - Single cache invalidation for multiple documents

**Rationale:**
- SymbolKind doesn't impl Hash → use format!("{:?}") for cache keys
- Program doesn't impl Sync → batch indexing sequential, not parallel
- Prevents OOM on large workspaces, 10-100x speedup on cached queries

**Status:** Implemented with 21 tests (11 workspace search + 10 performance)

---

## P-B03-03: Partial AST on Parse Errors

**Decision:** `document.rs` stores the partial AST even when parse errors occur.
Previously: returned early with `ast = None` on any parse error.
Now: stores the AST before returning on parse errors, enabling hover/completion for partial code
(critical for `impl |` completions).

---

## Notes

**Testing Pattern:** LSP tests use inline server creation (see testing-patterns.md - lifetime issues prevent helper functions)

**Cross-File Support:** Phase 05 added workspace-wide symbol search, references, call hierarchy.

**Type Integration:** Refactorings don't use type information yet. Future enhancement could enable type-aware refactorings.
