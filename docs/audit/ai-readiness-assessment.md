# Atlas AI-Readiness Assessment — Brutal Audit

**Date:** 2026-03-08
**Auditor:** Claude Sonnet 4.6 (adversarial critic mode)
**Sources read:** All `docs/language/`, all `docs/stdlib/`, `ast.rs`, `token.rs`, `stdlib/mod.rs`

---

## Executive Summary

Atlas v0.3 has a solid core — one numeric type, clean match expressions, unambiguous keywords — but it is actively sabotaging AI generation through three categories of failure: documentation that shows syntax that doesn't exist in the parser, a mutation model that is functionally invisible to AI, and a stdlib surface that is mid-transition between two APIs with neither one yet fully documented as the canonical form. Haiku will generate plausible-looking Atlas code that silently discards all collection mutations, calls methods that do not exist yet, and uses `|x|` closure syntax that the parser rejects. The north star goal — zero diagnostics on first generation — is not achievable in the current state.

---

## Score: 47/100 — "Looks clean, runs broken"

---

## 1. Ambiguity Problems (things that have multiple valid forms)

### 1.1 Three ways to construct a HashMap
```atlas
hashMapNew()                        // old global, deprecated (AT9000 warning)
HashMap.new()                       // new static constructor (B10)
record { key: value }               // parses to ObjectLiteral / HashMap at runtime
```
An AI asked to "create a map" has three valid choices. The `record` keyword is an especially confusing third option — it parses differently, produces a different AST node (`ObjectLiteral` vs `HashMap`), but arrives at the same runtime type. No doc explains when to use which. The grammar lists `record_literal` as a distinct form; the stdlib docs describe `HashMap.new()`. Haiku will pick one at random.

### 1.2 Two closure syntaxes — one of them doesn't exist
The grammar defines closures as:
```atlas
fn(x: number) -> number { x + 1 }
```
`METHOD-CONVENTIONS.md` (the AI-facing stdlib doc) shows:
```atlas
arr.map(|x| x * 2)
arr.filter(|x| x > 0)
```
`|x|` is Rust syntax. It does not exist in the token list. It does not appear in the grammar. Yet it appears in the document that is explicitly titled as the model for AI generation. Every AI that reads METHOD-CONVENTIONS.md will generate `|x|` closures. They will all fail.

### 1.3 Pipeline operator `|>` appears in spec, absent from implementation
`docs/language/async.md` contains this example as functional working code:
```atlas
let timed_out = sleep(secs) |> fn(_) -> Result<string, string> { return Err("timeout"); };
```
`|>` is not in `token.rs`. It is not in `grammar.md`. It is not in the parser. It is a phantom operator shown in a spec example. AI reading the async doc will try to use `|>` and get a parse error with no diagnostic explaining why.

### 1.4 `None` vs `None()`
`docs/language/types.md` states: "Constructors are functions: `Some(value)`, `None()`, `Ok(value)`, `Err(value)`."
`docs/language/control-flow.md` pattern match example uses: `None => 0` (in pattern position, correct) and `return None();` (in expression position).
`docs/language/structs-enums.md` match example uses: `None => 0` and notes `None` as a constructor pattern.

So: is `None` a value or `None()` a call? Both appear in official examples in different positions. AI will produce both forms. Only one works in expression position. This is unresolved in the docs and will cause failures.

### 1.5 Two valid `if` condition styles, one undocumented implication
```atlas
if condition { }      // no parens — Atlas style
if (condition) { }    // parens allowed
```
This is explicitly documented as intentional. The problem: it creates ambiguity with block expression syntax. An AI generating `if (x > 0) { ... }` is writing valid Atlas, but the AI reading the docs will correctly learn "no parens needed" and then see other examples with parens and be unsure which is canonical. The docs should call one preferred and one tolerated — they don't.

### 1.6 `Array<T>` vs `[]T` — two equivalent type syntaxes
Both forms are valid, both documented. There is no stated preference. `Array<string>` and `[]string` are interchangeable. Every AI generation prompt may produce either form, making AI-generated code inconsistent across a codebase.

### 1.7 Implicit return vs explicit `return` — two ways to return values
Both tail-expression (no semicolon) and `return expr;` are valid. The grammar documents both. This is correct behavior but it means AI will produce mixed styles, and the codebase will look inconsistent without a stated convention.

---

## 2. AI Generation Traps (things Haiku will get wrong, guaranteed)

### 2.1 CoW rebind — the silent killer
This is the single most dangerous trap in Atlas. Collection mutation returns a new value. Discarding the return value is silent data loss:
```atlas
arr.push(x);              // ❌ silently discards the result, arr unchanged
arr = arr.push(x);        // ✅ correct
```
Every AI trained on TypeScript, JavaScript, Python, Go, or Rust will generate the first form. None of those languages require rebinding on push. The rebind requirement is documented in `docs/language/types.md` and `docs/stdlib/array.md`, but the new `METHOD-CONVENTIONS.md` doc — the primary AI reference — shows `arr.push(x)` with no rebind, no note, no warning. This will silently corrupt every collection in every AI-generated Atlas program.

**Severity: Maximum.** Silent failure, no diagnostic, wrong result every time.

### 2.2 `|x|` closure syntax that was never implemented
See §1.2. The primary stdlib API reference shows `arr.map(|x| x * 2)`. The parser requires `arr.map(fn(x) { x * 2 })`. This is not a minor syntax variant — it is a completely different token sequence. Haiku will generate `|x|` syntax confidently because the official documentation shows it.

### 2.3 `sqrt(x)` returns `Result<number, string>`, not `number`
Every AI will generate `let root: number = sqrt(x)`. This is a type error. `sqrt` returns `Result<number, string>`. The docs warn about this in `AI-GENERATION-NOTES.md`, but that file is not prominently linked — it is listed as an index entry in `docs/stdlib/index.md`. The actual `docs/stdlib/math.md` shows `sqrt` returning `number` in the signature block with no warning. The AI-GENERATION-NOTES warning is in a separate file that an AI reading only the math docs will never see.

### 2.4 `indexOf`, `charAt`, `lastIndexOf` return `Option<T>`, not bare values
```atlas
let i: number = s.indexOf("x");  // ❌ type error, returns Option<number>
```
Every language Haiku knows — Python, JS, Go, Rust — returns a number or -1 from indexOf. Atlas returns `Option<number>`. The AI-GENERATION-NOTES file documents this, but the `string.md` doc buries the warning inline. Given how many AI-generated programs use string search, this will cause failures in the majority of string-heavy programs.

### 2.5 `async fn main()` is forbidden
Every AI trained on modern async patterns will write `async fn main()`. This is a diagnostic error in Atlas (AT4006). The spec requires top-level `await` instead. While this is documented in the async spec, it is so counter to universal convention that every first-attempt generation will produce `async fn main()`.

### 2.6 `return` inside `match` arms is a syntax error (H-114, resolved)
H-114 is marked resolved, but the pattern is still a trap: AI will write `return` inside match arms by instinct from Rust. Whether the fix is fully propagated to all production code paths should be verified with real programs.

### 2.7 Struct variant construction `Enum::Variant { field: value }` is not supported
The docs explicitly state: "Struct variants are parsed in declarations but are not currently constructible with `Enum::Variant { ... }` syntax." Haiku will write it anyway — it is the obvious syntax for named enum variants. Silent failure or a confusing parse error awaits.

### 2.8 No `impl` blocks on plain structs — trait required
The docs show:
```atlas
impl Greetable for Person { fn greet(self) -> string { ... } }
```
There is no `impl Person { fn greet(self) -> string { ... } }` syntax without a trait. Every AI trained on Rust, Go, or TypeScript will try to add methods directly to a struct without inventing a trait. H-144 confirms this is an open gap. AI-generated struct-heavy code will be broken.

### 2.9 `math.md` doc inconsistency between global and dot syntax
`Math.sqrt(x)` is the new canonical form. But `sqrt(x)` (bare) is also valid (no AT9000). The `math.md` doc still documents bare globals as the primary form. AI reading `math.md` will use bare globals; AI reading `METHOD-CONVENTIONS.md` will use `Math.sqrt`. Neither is wrong, but the inconsistency means AI-generated code will be inconsistent and the docs give no canonical guidance.

### 2.10 C-style `for` loop exists in AST but is undocumented
`ast.rs` contains `Stmt::For(ForStmt)` with `init`, `cond`, `step` fields — a classic C-style `for(;;)` loop. This node exists in the AST but is completely absent from `grammar.md` and `control-flow.md`. An AI could potentially trigger this parse path or, more likely, be confused when it encounters Atlas code that uses it. Either way, having undocumented AST nodes is a trust violation.

### 2.11 `++` and `--` exist in the AST but are undocumented
`IncrementStmt` and `DecrementStmt` are in `ast.rs`. They are absent from `grammar.md`. These are either unimplemented (parser scaffolding) or implemented but deliberately hidden. If they work, AI should know. If they don't, they should be removed from the AST.

---

## 3. Consistency Failures

### 3.1 Stdlib naming is mid-transition — two canonical forms exist simultaneously
The stdlib is caught between the old global API (`arrayPush`, `hashMapGet`, `regexTest`) and the new method API (`arr.push()`, `map.get()`, `Regex.test()`). Both work. Neither is wrong. But every doc file still documents the old form as the primary API — `array.md`, `string.md`, `collections.md`, `math.md` all lead with old-style globals. Only `METHOD-CONVENTIONS.md` shows the new API. An AI reading any individual stdlib doc will use the old API and get AT9000 warnings.

### 3.2 `len()` vs `.len()` — inconsistent length access
`core.md` documents `fn len(value: []T) -> number` as a global. `METHOD-CONVENTIONS.md` shows `arr.len()` as a method. `str.len()` is the method form. No doc states which is preferred for which type. AI will use both interchangeably and be confused by inconsistency.

### 3.3 Type check functions: `isString` (camelCase) but `is_some`, `is_ok` (snake_case)
From `docs/stdlib/types.md`:
- Type predicates: `isString`, `isNumber`, `isBool`, `isArray` — camelCase
- Option/Result checks: `is_some`, `is_none`, `is_ok`, `is_err` — snake_case

These are in the same module, doing the same category of work (runtime type/state checking), and they use different naming conventions. This is a direct inheritance from "did it as we went" development. AI will either camelCase everything or snake_case everything and get half of them wrong.

### 3.4 `map.size()` vs `arr.len()` — different methods for the same concept
Arrays use `.len()`. Maps and sets use `.size()`. Haiku will consistently call `arr.size()` and `map.len()`. This is the TypeScript `Map.size` vs `Array.length` mistake replicated in Atlas, except Atlas also chose to use `.len()` for arrays (Rust), making it inconsistent in two directions at once.

### 3.5 `HashMap.new()` vs `HashSet.new()` but no `Array.new()` or `Queue.new()`
Static constructors exist for HashMap and HashSet. Arrays use literal syntax `[]`. Queue and Stack have `queueNew()` (old) or no documented static constructor in METHOD-CONVENTIONS. The pattern is inconsistent — some types have `Type.new()`, others use literals or globals.

### 3.6 `record { }` vs `{ }` for anonymous structs — two syntaxes for "object literal"
`grammar.md` distinguishes:
- `record_literal`: `record { key: value }` — maps to HashMap
- `anon_struct_literal`: `{ key: value }` — anonymous struct

These look nearly identical to an AI. The presence/absence of the `record` keyword changes the semantics entirely (HashMap vs structural type). An AI generating ad-hoc data will guess wrong roughly 50% of the time.

---

## 4. Inherited Mistakes from Go/Rust/TypeScript

### 4.1 Ownership annotations (`own`, `borrow`, `shared`) — Rust's complexity without Rust's benefit
Atlas has `own param: T`, `borrow param: T`, `shared param: T` as parameter annotations. These are optional but exist in the grammar and AST. For AI generation, these annotations are pure noise: they add cognitive load with no benefit because Atlas does not enforce ownership semantics at the typechecker level (there is no borrow checker). An AI generating Atlas code must decide whether to include ownership annotations on every function parameter, with no rule for when they are required vs optional. This is Rust's learning tax imported wholesale. The north star is "clean to generate" — optional annotations that do nothing are the opposite.

**Verdict: This is the wrong call for an AI-first language.** If Atlas doesn't enforce ownership, these keywords should not exist in the user-facing language.

### 4.2 `extends` AND `:` for trait bounds — two syntaxes for the same thing
```atlas
fn identity<T: Copy>(value: T) -> T { ... }          // Rust-style
fn identity<T extends number>(value: T) -> T { ... } // TypeScript-style
```
Both are valid per the docs. AI trained on TypeScript will use `extends`. AI trained on Rust will use `:`. Both work. This is a direct copy of TypeScript's `extends` keyword appearing alongside Rust's `:` syntax. Pick one. Kill the other.

### 4.3 `Future<T>` written explicitly vs inferred — Go channel complexity
The async docs say: "Write the inner return type — `-> string`, not `-> Future<string>`. The compiler automatically wraps it in `Future<T>`. Writing `-> Future<T>` explicitly is also accepted and means the same thing."

This is two valid forms for the same declaration. AI will produce both. The rule "write `string`, not `Future<string>`, except both work" is not a rule — it is ambiguity. TypeScript did the same thing with `Promise<T>` vs just the return type in async functions, and TypeScript developers are still confused by it.

### 4.4 `impl TraitName for TypeName` with no bare `impl TypeName` — Rust without the motivation
Rust requires `impl Trait for Type` when implementing a trait interface, but also has bare `impl Type { fn method() }` for inherent methods. Atlas has only the trait form. This means every struct method requires the developer to invent a trait, name it, declare it, then implement it. The cognitive overhead is identical to Rust but the reward (trait objects, bounds, polymorphism) requires actually understanding the trait system. For AI generation, this means every "add a method to a struct" task requires generating 3 declarations instead of 1. H-144 is already filed, but the impact on AI generation is severe — it is the most common struct task and it is broken.

### 4.5 The `json` type — TypeScript's `any` mistake renamed
Atlas has a special `json` type that is "only assignable to `json`". This is TypeScript's `any` with a JSON-specific alias. The problem: AI generating code that processes JSON must navigate `json` → typed value conversions using `jsonGetString`, `jsonGetNumber`, etc., which are low-level accessor globals with no ergonomic path. TypeScript at least has structural typing that makes JSON objects usable. Atlas's `json` type is a walled garden. Real programs that process JSON require substantial boilerplate.

---

## 5. Missing Fundamentals

### 5.1 No string-to-number or number-to-string conversion that is obvious
`str(x)` converts to string. `toNumber(x)` converts to number. But these are global functions — `parseInt`, `parseFloat`, `toString` are also listed. An AI will reach for `x.toString()` (TypeScript) or `x.to_string()` (Rust) and get a method-not-found error. The correct form `str(x)` is not obvious to any AI trained on existing languages.

### 5.2 No range-based for loop with index (enumerate)
```atlas
for i in 0..10 { }    // range — exists (H-116 resolved)
for i, v in arr { }   // enumerate — not documented, likely not implemented
```
Iterating an array with access to both the index and value requires a manual counter or `arrayIndexOf`. Python's `enumerate`, Rust's `.enumerate()`, TypeScript's `.entries()` all exist in v1.0. This is a fundamental missing primitive.

### 5.3 Trait inheritance (`trait A: B`) — not implemented (H-076)
Filed, known. But the impact: every AI that reaches for trait composition (extending a base trait) will fail silently or get a parse error. This blocks realistic interface hierarchies.

### 5.4 Generic traits (`trait Container<T>`) — not implemented (H-077)
Filed, known. The impact: building generic abstractions is blocked. Every AI-generated `trait Serializable<T>` fails. Given that Atlas targets systems-level code, this is a v1.0 requirement.

### 5.5 No tuple type
Atlas has no tuple syntax. `[T, U]` is an array. There is no `(T, U)` tuple. Functions returning multiple values must use arrays (losing type safety per-element) or anonymous structs. Python, Rust, TypeScript all have tuples in v1.0. Every AI will try `let (a, b) = some_function()` and fail.

### 5.6 No destructuring assignment
No `let { x, y } = point` syntax. No `let [a, b] = pair`. Struct field access requires dot notation. Array destructuring in let is not documented. TypeScript, Python, and modern JS all support destructuring as a primary pattern. AI will attempt it constantly.

### 5.7 Mutable `self` in trait methods is broken (H-147, open)
Any stateful object in Atlas — anything that needs to update internal state through a method — requires mutable self. H-147 is open. This means Atlas cannot express the most common OOP pattern: an object that mutates itself. Every AI-generated stateful class equivalent fails.

### 5.8 Cannot read struct fields in trait method implementations (H-148, open)
H-148 is open. If you implement a trait for a struct, you cannot access the struct's fields inside the trait method body. This is the most basic operation in OOP — `return self.name`. It is broken. Combined with H-147, the entire trait-based OOP system is non-functional for real programs.

### 5.9 No `switch` statement — only `match`
`match` is more powerful, but AI trained on Go/TypeScript/C will write `switch` first. Atlas has no `switch` keyword. The docs should explicitly state this and show the `match` equivalent. Currently, the docs document `match` without noting that `switch` does not exist.

### 5.10 `if/else` as expression — documented as not producing a value, then contradicted
`control-flow.md` says: "`if` parses in expression position but does not produce a value." Yet `match` produces values and H-115 (if/else as expression returns `?` type) is resolved, implying it should work. This needs a clear, unambiguous statement. AI will try to write `let x = if flag { 1 } else { 2 }` and get inconsistent results.

---

## 6. Per-Domain Scores

| Domain | Score | Key failure |
|--------|-------|-------------|
| **Structs** | 45/100 | No `impl Struct` syntax; no destructuring; struct enum variant construction broken |
| **Enums** | 60/100 | Core match works; struct variant construction unimplemented; H-110/H-111 resolved but trust is damaged |
| **Functions** | 72/100 | Return type required is clean; ownership annotations add noise; two bound syntaxes |
| **Error handling** | 55/100 | `?` operator is good; `None` vs `None()` is ambiguous; `Result`-returning stdlib surprises (sqrt, indexOf) cause silent failures |
| **Collections** | 40/100 | CoW rebind is invisible; three ways to create a map; dot method syntax shows `|x|` closures that don't parse |
| **Async** | 50/100 | `async fn main()` trap; `|>` operator in docs but not implemented; `Future<T>` explicit vs inferred ambiguity |
| **Stdlib** | 38/100 | Mid-transition between two APIs; wrong return types on math/string functions; `isString`/`is_some` naming mismatch |
| **Types** | 65/100 | Single number type is excellent; `T?` optional syntax rejected (must use `Option<T>`); `json` type is a trap |
| **Traits** | 20/100 | H-147 and H-148 are open: cannot read struct fields or have mutable self — the system is non-functional for real programs |
| **Modules** | 75/100 | Import/export syntax is clear and clean; closest to the north star |

**Overall: 47/100**

---

## 7. Top 10 Concrete Recommendations (ranked by impact)

### R1. Fix METHOD-CONVENTIONS.md closure syntax immediately
Replace `arr.map(|x| x * 2)` with `arr.map(fn(x) { x * 2 })` everywhere. This is a one-file fix that removes a trap from the primary AI reference document. The `|x|` syntax has never existed in Atlas. Its presence in the docs is a documentation bug that causes 100% first-attempt failure rate on any collection operation involving a callback.

**Impact:** Every collection + callback operation currently fails. Fix is 10 minutes.

### R2. Add explicit rebind requirement to METHOD-CONVENTIONS.md
Every `arr.push(x)` example must be `arr = arr.push(x)` or annotated with "returns new array — rebind required." The current doc presents mutation methods with no rebind note. Silent data loss on every collection write is the worst possible first impression.

**Impact:** Every collection mutation currently silently fails. Fix is adding 10 lines to METHOD-CONVENTIONS.md.

### R3. Remove `|>` pipeline operator from async.md or implement it
The async spec contains a working code example using `|>` which does not exist. Either implement the operator (would be a genuine AI-generation win — pipelines are extremely clean to generate) or replace the example with working code. A spec example that doesn't run is worse than no example.

**Impact:** Async error handling examples are non-functional. Anyone reading the spec is misled.

### R4. Resolve `None` vs `None()` definitively
Pick one. Document it everywhere. The correct form for expression position should be consistent. Having `None` in pattern context and `None()` in expression context is accurate but confusing, and the docs present both without explaining the distinction. Recommendation: `None` everywhere (value, not call), matching Rust convention.

**Impact:** Option-returning functions will have inconsistent generation. Moderate fix cost.

### R5. Add a prominent CoW/rebind cheat sheet to every collection doc page
Not buried in types.md — right at the top of `METHOD-CONVENTIONS.md`, `array.md`, and `collections.md`. One box that says: "IMPORTANT: All collection mutation methods return a new collection. You must assign the result." Every AI reading any collection doc must see this before the first example.

**Impact:** Prevents the most common silent failure in Atlas programs.

### R6. Implement bare `impl TypeName` blocks (H-144)
The inability to add methods to a struct without inventing a trait is the biggest structural gap in Atlas for practical programs. AI generates struct methods as the most common OOP task. Every single one fails. This is a P1 that should be P0 for AI-readiness.

**Impact:** Unlocks every struct-based program. Currently 0% of struct methods work without a trait.

### R7. Fix the math.md doc to show `Result<number, string>` return types
`sqrt`, `log`, `clamp`, `asin`, `acos` all return `Result<number, string>` but `math.md` shows them returning `number`. The AI-GENERATION-NOTES file has the warning but is not linked from `math.md`. Add inline warnings to the individual function docs and link from `math.md`'s header.

**Impact:** Every numerical computation involving these functions produces a type error.

### R8. Pick one trait bound syntax and kill the other
`T extends number` (TypeScript) and `T: Trait` (Rust) both work. Choose `:`. Document it. Remove `extends` from the grammar. The north star is "one way to do everything." Having two identical syntax forms is a direct violation.

**Impact:** Type-level code will be inconsistent across AI-generated Atlas. Easy to fix now, harder after adoption.

### R9. Remove or implement ownership annotations
`own`, `borrow`, `shared` are keywords in the language with no enforcement by the typechecker. They add noise to every function signature generation decision. Either enforce them (real ownership semantics, Rust complexity) or remove them from the user-facing language (parser still parses them for future use, but docs don't teach them). Documenting optional unenforced annotations makes every AI ask "do I need these?" on every function parameter.

**Impact:** Every function with resource parameters is noisier than it needs to be.

### R10. Audit and clean up undocumented AST nodes
`Stmt::For` (C-style for), `IncrementStmt`, `DecrementStmt` exist in the AST but are absent from docs and grammar. These are either implemented (document them) or they are dead scaffolding (mark them explicitly as unimplemented in the grammar and add a parse error if triggered). Undocumented features that partially work destroy trust in the entire language.

**Impact:** Predictability. If the grammar doc is complete and accurate, AI can rely on it. Currently it is not.

---

## Appendix: What Atlas Gets Right

These are genuine strengths. They should be protected.

- **Single numeric type.** `number` for everything. Zero `int` vs `float` ambiguity. This alone eliminates a category of AI generation errors present in Go, Rust, and TypeScript.
- **Required return types on named functions.** Forces explicit contracts. AI generates cleaner signatures.
- **`match` as an expression with exhaustiveness.** Well-designed. Clean to generate.
- **`?` error propagation operator.** Clear, unambiguous, one syntax.
- **Template strings with `{expr}`.** Clean interpolation. Better than JS backtick with `${expr}`.
- **Mandatory `let mut` for mutation.** AI can see which variables change. Reduces surprise.
- **No semicolons required on tail expressions.** Rust-style implicit return is clean.
- **`for item in iterable` (no parentheses default).** Closer to Python, cleaner than Go/C.
- **Module system.** `import { x } from "./path"` is exactly TypeScript and exactly what AI generates correctly on the first try.
