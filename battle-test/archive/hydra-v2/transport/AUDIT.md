# Component Audit: Transport

**Grammar Friction Points**
- No byte/stream types; transport logic had to be string-only.
- No `indexOf`/`join` in stdlib; LSP parsing required split + manual rebuild.
- Assignment targets cannot be `obj.field`, so mutable state must be rebuilt.
- `if` requires parentheses; missing parens trigger confusing parser errors.

**Missing Features (Atlas should have)**
- Byte buffer and stream IO primitives (read/peek with timeout).
- Standard string search utilities (`indexOf`, `find`, `join`).
- Native framing helpers for NDJSON and LSP.

**Syntax Quality Rating:** 6/10

**AI Generation Experience**
- Straightforward for simple helpers, but real protocol parsing hit missing stdlib pieces quickly.
- The v0.3 grammar (record literals, no C-for) is consistent, but docs still mislead.
