# Atlas AI Design Principles — The Forward Strategy

> **Decision:** D-060 (CORE) | **Status:** Locked
> **For AI agents:** Read this before making ANY syntax, grammar, or API design decision.

---

## The Core Rule

**Systems-level power lives at the EDGES. The TypeScript surface stays clean.**

Atlas is TypeScript's front end on top of Rust's runtime model. An AI generating Atlas code should be able to produce correct everyday code from TypeScript muscle memory alone, with zero Atlas-specific training. Systems features appear only where they matter — at explicit system boundaries.

---

## The Three Tiers

### Tier 1 — Everyday Code (Zero AI Friction)
Identical to TypeScript. AI generates this correctly, cold, with no Atlas training.

```atlas
// Variables
let x = 42;
let mut count = 0;

// Functions — colon return type (TypeScript style)
fn add(a: number, b: number): number {
    a + b
}

// Collections — TypeScript names, TypeScript constructors
let m = new Map<string, number>();
m.set("key", 42);
let v = m.get("key");

let s = new Set<string>();
s.add("hello");

// Arrays — TypeScript postfix syntax
let arr: number[] = [1, 2, 3];
arr.push(4);

// Control flow
if condition { }
while running { }
for item in collection { }
match value { pattern => result }

// Error handling — Rust Result, TypeScript ? operator
fn divide(a: number, b: number): Result<number, string> {
    if b == 0 { return Err("division by zero"); }
    Ok(a / b)
}
let result = divide(10, 2)?;

// Structs + impl
struct Point { x: number, y: number }
impl Point {
    fn distance(self): number { Math.sqrt(self.x * self.x + self.y * self.y) }
}

// Namespaces — lowercase (TypeScript module style)
console.log("hello");
Math.sqrt(16.0);
file.read("path.txt");
```

### Tier 2 — Systems Boundary (Explicit Only When It Matters)

Ownership annotations appear ONLY on function parameters that cross ownership boundaries. **`borrow` is the implicit default — write nothing for the common case.** (D-040)

```atlas
// Common case: borrow is implicit. Write TypeScript-style, it works.
fn process(data: string): string {
    data.toUpperCase()     // borrow implicit — caller keeps ownership
}

// Explicit own: caller's binding becomes invalid after call (like Rust move)
fn consume(own data: Buffer): void {
    data.flush();
    // data is gone from caller's scope
}

// Explicit share: concurrent read access (like Rust Arc<T>)
fn cache(share handle: Connection): void {
    store(handle);    // caller and callee both hold valid references
}

// CoW is INVISIBLE — just works, no annotation needed
let a = [1, 2, 3];
let b = a;          // O(1) clone — value semantics, no syntax
b.push(4);          // a is still [1, 2, 3] — CoW fires transparently
```

**The rule:** If a dev can write the function without thinking about ownership, they should write it without ownership annotations. Annotations are for when ownership semantics ARE the point.

### Tier 3 — Atlas-Native (When TypeScript Has No Answer)

For systems features TypeScript cannot reach, design Atlas-native syntax. Never copy Rust or Go surface syntax — ask "what would TypeScript/Atlas do here?"

```atlas
// Async — TypeScript model (same syntax, Rust tokio under the hood)
async fn fetchUser(id: string): Result<User, string> {
    let response = await http.get("/users/" + id);
    Json.parse<User>(response.body())
}

// Channels — Go-inspired semantics, Atlas surface syntax (future)
// Concurrency — tokio work-stealing (D-057), transparent to user
```

---

## The AI-Friction Decision Filter

Before adding any syntax or renaming anything, apply this filter in order:

1. **TypeScript has it?** → Use TypeScript's exact form. `new Map<K,V>()`, `interface`, `extends`, `T[]`, `?` operator.
2. **TypeScript has no answer?** → Design Atlas-native. Minimal tokens. Unambiguous. AI-generatable from a single example.
3. **Rust/Go has it?** → Irrelevant. Never copy Rust or Go surface syntax. (Exception: `match`, `impl`, `trait` — AI knows these from Rust training and they have no TypeScript equivalent.)

**Token cost matters.** Every extra token is AI friction. `Map` beats `HashMap` (4 tokens saved). `new Map()` beats `hashMapNew()` (AI knows one cold, had to learn the other).

---

## What NOT to Change

These are correct and locked. Do not revisit without architect approval:

| Feature | Form | Why |
|---------|------|-----|
| CoW memory model | Invisible (D-029) | Runtime — AI doesn't see it |
| `borrow` default | Implicit (D-040) | Zero annotation for common case |
| `fn` keyword | `fn` not `function` | Fewer tokens |
| `number` type | Single unified type | AI never picks wrong numeric type |
| `T[]` arrays | Postfix (D-041) | TypeScript style |
| `match` expression | `match expr { pat => result }` | AI knows from Rust, TS getting it |
| `impl Foo { }` | Rust-style (D-036) | AI knows from Rust, acceptable divergence |
| `trait extends` | TypeScript style (D-026) | |
| `Result<T,E>` / `Option<T>` | Rust names (AI knows both) | |

---

## What Needs to Be Fixed (Open Issues)

| Issue | Current | Fix | Decision |
|-------|---------|-----|----------|
| H-373 | `HashMap` name | `Map` | D-060 |
| H-374 | `HashSet` name | `Set` | D-060 |
| H-375 | No constructor syntax | `new Map<K,V>()` | D-033, D-060 |
| H-376 | `record {}` ambiguous | struct literal only, map via `new Map()` | D-014 |

---

## For AI Agents — Quick Reference

**When you see a syntax decision to make:**
- Open this doc
- Run `pt decisions all` — CORE decisions appear first
- Apply the filter above
- If you decide something new: `pt add-decision` it immediately

**Commands you need:**
```bash
pt decision D-XXX          # Full decision detail (NOT 'pt issue D-XXX')
pt decisions all            # All decisions — CORE first
pt decisions [component]    # Filter by: parser|typechecker|runtime|stdlib|infra
pt check-decision "keyword" # Search before implementing
```

**The old DB confusion:** There is an old `tracking/atlas.db` in the repo. Ignore it. The real database is at `~/.project-tracker/atlas/tracking.db` and is accessed via `pt` commands. Never query `tracking/atlas.db` directly.
