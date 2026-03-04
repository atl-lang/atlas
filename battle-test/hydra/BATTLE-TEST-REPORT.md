# Hydra Battle Test Report (Atlas)

## Executive Summary
- **Completion:** 8/8 components implemented at functional stub level, with tests covering core logic.
- **Quality Rating (overall):** 6/10
- **Key Finding:** Atlas v0.3 grammar is workable, but documentation drift, non-mutating map ops, and missing/runtime-blocked primitives create major friction for real-world systems code.

## Friction Points

**P0 Blockers (cannot fully implement Hydra parity)**
- No streaming process I/O (stdin/stdout pipes) from `spawnProcess`.
- No filesystem watcher API; must degrade to polling/snapshot diffs.
- Record field assignment is not supported; state updates require full-record rebuilds.
- `syntax.md` and `grammar-conformance.md` contradict actual parser behavior (v0.3 grammar vs v0.2 docs).

**P1 Major Friction**
- JSON isolation + unsafe extraction (`jsonAsString` throws) makes defensive parsing brittle.
- No standard string search helpers (`indexOf`, `join`), increasing parsing boilerplate.
- Type alias and structural type syntax are not documented in the main syntax spec.
- `hashMapPut` returns a new map (no in-place mutation), forcing pervasive reassignments and API redesign.
- `if` requires parentheses; missing parens produce non-obvious parse errors.
- Empty array literals require awkward workarounds (`slice([""],0,0)`); `let x: string[] = []` still fails.
- Array concatenation with `+` is rejected; must use `arrayPush`/`concat`.
- `match` arms require commas and a trailing `;` in statement position.

**P2 Minor Friction**
- Import ergonomics are noisy (no namespace imports for local modules).
- Range tokens exist but only for slicing; examples using `0..n` are misleading.
- Runtime security blocks `getEnv` and `spawnProcess` in tests, forcing faked paths.

## AI Confusion Patterns
- **Doc drift:** AI will follow `syntax.md` and generate `var`, arrow functions, and C-style for loops, all of which now fail parsing.
- **Object vs record:** `record {}` is required but most examples show `{}`.
- **Structural type annotations:** AI guesses `object` or `map` types because the official spec doesn’t mention `{ field: type }` annotations.
- **Control flow parens:** AI frequently emits `if cond {}` which fails; parser requires `if (cond)`.
- **Map mutation:** AI assumes `hashMapPut` mutates; it actually returns a new map.
- **Empty arrays:** AI emits `[]` without context; typechecker rejects it even with `let x: string[] = []`.

## What Worked Well
- The v0.3 grammar (let/let mut, for-in, record literals, fn-only lambdas) is consistent and predictable.
- Built-in JSON parsing and hash maps are sufficient for stateful components.
- Async primitives exist, but they lack ergonomic language syntax.

## Recommendations for Atlas
- Align `docs/specification/syntax.md` and `grammar-conformance.md` with current parser behavior.
- Add record field assignment or a `mut record` update syntax.
- Provide safe JSON accessors returning `Option<T>`.
- Add string search/join helpers and framing helpers for protocols.
- Expose process stdin/stdout streams and a real filesystem watcher API.
- Provide a mutating map API or explicit `put!` syntax to avoid silent no-ops.
- Make empty array literals type-check with explicit annotation.
- Document required `if` parentheses and `match` punctuation in the main syntax guide.

## Code Samples

**AI-Friendly (v0.3 correct)**
```atlas
let mut total = 0;
for item in items {
    total = total + item;
}
let cfg = record { host: "localhost", port: 8080 };
let f = fn(x: number) -> number { return x * 2; };
if (total > 0) { print("ok"); }
let mut xs: string[] = slice([""], 0, 0);
xs = arrayPush(xs, "a");
```

**AI-Confusing (old docs, now invalid)**
```atlas
var total = 0;                 // removed
for (var i = 0; i < 10; i++) { } // removed
let cfg = { host: "localhost" }; // ambiguous block vs record
let f = (x) => x * 2;            // arrow removed
if total > 0 { }                 // missing parens
let xs: string[] = [];           // rejected by typechecker
hashMapPut(map, "k", "v");       // no in-place mutation
```

## Brutally Honest Conclusion
Atlas v0.3 has a cleaner grammar than v0.2, but the **documentation is out of sync with reality**. This alone is enough to cause AI-generated code to fail consistently. The language also lacks several systems-level primitives (process streaming I/O, filesystem watching) that are required to rebuild Hydra with parity. As a result, the port is functional but not faithful: core ideas are implemented, yet real runtime behavior is still blocked by missing APIs.
