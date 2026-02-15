# DR-004: Collection Value Representation

**Date:** 2026-02-15
**Status:** Accepted
**Component:** Standard Library - Collections

## Context
Collections (HashMap, HashSet, Queue, Stack) need representation in Atlas's Value enum and memory management strategy.

## Decision
**Add four new Value variants using Rc<RefCell<>> for shared mutable ownership:**

```rust
// In crates/atlas-runtime/src/value.rs
pub enum Value {
    // Existing variants...
    Number(f64),
    String(Rc<String>),
    Bool(bool),
    Null,
    Array(Rc<RefCell<Vec<Value>>>),
    Function(FunctionRef),
    NativeFunction(NativeFn),
    JsonValue(Rc<JsonValue>),
    Option(Option<Box<Value>>),
    Result(Result<Box<Value>, Box<Value>>),

    // New collection variants
    HashMap(Rc<RefCell<AtlasHashMap>>),
    HashSet(Rc<RefCell<AtlasHashSet>>),
    Queue(Rc<RefCell<AtlasQueue>>),
    Stack(Rc<RefCell<AtlasStack>>),
}
```

**Internal collection types:**
```rust
// Wrapper types for type safety
struct AtlasHashMap {
    inner: HashMap<HashKey, Value>,
    // Future: insertion order tracking for iteration
}

struct AtlasHashSet {
    inner: HashSet<HashKey>,
    // Backed by HashMap internally
}

struct AtlasQueue {
    inner: VecDeque<Value>,
    // Circular buffer for O(1) enqueue/dequeue
}

struct AtlasStack {
    inner: Vec<Value>,
    // Simple Vec with push/pop
}
```

## Rationale
**Consistent with Array memory model:** Arrays use `Rc<RefCell<Vec<Value>>>` - collections follow same pattern for:
- Shared ownership (multiple references to same collection)
- Interior mutability (modify through shared reference)
- Reference semantics (assignment copies reference, not data)

**Separate wrapper types (not direct HashMap/HashSet):**
- Allows future enhancements (insertion order, metadata)
- Type safety at Value level
- Cleaner API surface

**Industry precedent:**
- **Rust:** Reference-counted types (Rc) for shared ownership, RefCell for runtime borrow checking
- **JavaScript:** All objects (including Map/Set) have reference semantics
- **Python:** dict/set are reference types
- **Go:** map/slice are reference types

## Alternatives Considered
- **Generic Collection<T> type:** Rejected - each collection has different semantics, better to be explicit
- **Box instead of Rc:** Rejected - need shared ownership for collection aliasing
- **No RefCell (immutable):** Rejected - collections need mutation (insert, remove)
- **Arc<Mutex<>> for thread safety:** Rejected - Atlas is single-threaded in v0.2

## Consequences
- ✅ **Benefits:** Consistent with existing Array pattern (familiar)
- ✅ **Benefits:** Reference semantics enable efficient passing without cloning
- ✅ **Benefits:** Interior mutability enables natural mutation patterns
- ✅ **Benefits:** Wrapper types allow future enhancements without breaking changes
- ⚠️  **Trade-offs:** RefCell has runtime borrow checking cost (small, acceptable)
- ⚠️  **Trade-offs:** Reference semantics can surprise users (but documented, consistent with JS/Python)
- ❌ **Costs:** Four new Value variants increase enum size (negligible on modern hardware)

## Implementation Notes
**Phase:** `stdlib/phase-07a-hash-infrastructure-hashmap.md` (HashMap/HashSet)
**Phase:** `stdlib/phase-07c-queue-stack.md` (Queue/Stack)

**File locations:**
- `crates/atlas-runtime/src/value.rs` - Value enum additions
- `crates/atlas-runtime/src/stdlib/collections/hashmap.rs` - AtlasHashMap
- `crates/atlas-runtime/src/stdlib/collections/hashset.rs` - AtlasHashSet
- `crates/atlas-runtime/src/stdlib/collections/queue.rs` - AtlasQueue
- `crates/atlas-runtime/src/stdlib/collections/stack.rs` - AtlasStack

**Type guard additions:**
```atlas
isHashMap(value: any) -> bool
isHashSet(value: any) -> bool
isQueue(value: any) -> bool
isStack(value: any) -> bool
```

## References
- Spec: `docs/specification/runtime.md` (Value representation section)
- API: `docs/api/stdlib.md` (Collections section)
- Related: DR-001 (JSON Value representation), DR-003 (Hash function design)
