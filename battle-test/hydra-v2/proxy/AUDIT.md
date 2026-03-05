# Component Audit: Proxy

**Grammar Friction Points**
- No member assignment; queue and proxy state require full-record rebuilds.
- Function-typed fields in records are possible but noisy to type and pass around.
- `if` requires parentheses; missing parens fail parsing with non-obvious errors.
- `hashMapPut` returns a new map; “mutation” must reassign, which breaks intuitive proxy APIs.
- `[]` empty arrays fail without awkward workarounds (`slice([""],0,0)`).
- Array concat with `+` rejected; must use `arrayPush`/`concat`.

**Missing Features (Atlas should have)**
- Native concurrent read loops or ergonomic async/await syntax.
- Select-style channel primitives integrated into language (not just stdlib).
- Structured JSON-RPC helpers (parse/validate).
- First-class tuple/multi-return to avoid awkward API rewrites for immutable maps.

**Syntax Quality Rating:** 6/10

**AI Generation Experience**
- Core routing logic is straightforward, but concurrency patterns are hard to express cleanly.
- Lack of safe JSON field extraction forces brittle assumptions.
