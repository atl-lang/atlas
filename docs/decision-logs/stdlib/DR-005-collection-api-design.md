# DR-005: Collection API Design and Iteration

**Date:** 2026-02-15
**Status:** Accepted
**Component:** Standard Library - Collections

## Context
Collections need API design for creation, manipulation, and iteration. Atlas must balance between function-based stdlib and method syntax capabilities.

## Decision
**Use function-based API (not methods) with explicit collection types:**

**HashMap API (15 functions):**
```atlas
// Creation
hashMapNew() -> HashMap
hashMapFromEntries(entries: [key, value][]) -> HashMap

// Mutation
hashMapPut(map: HashMap, key: hashable, value: any) -> void
hashMapRemove(map: HashMap, key: hashable) -> Option<any>
hashMapClear(map: HashMap) -> void

// Query
hashMapGet(map: HashMap, key: hashable) -> Option<any>
hashMapHas(map: HashMap, key: hashable) -> bool
hashMapSize(map: HashMap) -> number
hashMapIsEmpty(map: HashMap) -> bool

// Access
hashMapKeys(map: HashMap) -> hashable[]
hashMapValues(map: HashMap) -> any[]
hashMapEntries(map: HashMap) -> [hashable, any][]

// Iteration (callback-based)
hashMapForEach(map: HashMap, fn: (value: any, key: hashable) -> void) -> void

// Transformation
hashMapMap(map: HashMap, fn: (value: any, key: hashable) -> any) -> HashMap
hashMapFilter(map: HashMap, fn: (value: any, key: hashable) -> bool) -> HashMap
```

**HashSet API (13 functions):**
```atlas
// Creation
hashSetNew() -> HashSet
hashSetFromArray(elements: hashable[]) -> HashSet

// Mutation
hashSetAdd(set: HashSet, element: hashable) -> void
hashSetRemove(set: HashSet, element: hashable) -> bool
hashSetClear(set: HashSet) -> void

// Query
hashSetHas(set: HashSet, element: hashable) -> bool
hashSetSize(set: HashSet) -> number
hashSetIsEmpty(set: HashSet) -> bool

// Set operations
hashSetUnion(a: HashSet, b: HashSet) -> HashSet
hashSetIntersection(a: HashSet, b: HashSet) -> HashSet
hashSetDifference(a: HashSet, b: HashSet) -> HashSet
hashSetSymmetricDifference(a: HashSet, b: HashSet) -> HashSet
hashSetIsSubset(a: HashSet, b: HashSet) -> bool
hashSetIsSuperset(a: HashSet, b: HashSet) -> bool

// Access
hashSetToArray(set: HashSet) -> hashable[]

// Iteration
hashSetForEach(set: HashSet, fn: (element: hashable) -> void) -> void
hashSetFilter(set: HashSet, fn: (element: hashable) -> bool) -> HashSet
```

**Queue API (8 functions):**
```atlas
queueNew() -> Queue
queueEnqueue(queue: Queue, element: any) -> void
queueDequeue(queue: Queue) -> Option<any>
queuePeek(queue: Queue) -> Option<any>
queueSize(queue: Queue) -> number
queueIsEmpty(queue: Queue) -> bool
queueClear(queue: Queue) -> void
queueToArray(queue: Queue) -> any[]
```

**Stack API (8 functions):**
```atlas
stackNew() -> Stack
stackPush(stack: Stack, element: any) -> void
stackPop(stack: Stack) -> Option<any>
stackPeek(stack: Stack) -> Option<any>
stackSize(stack: Stack) -> number
stackIsEmpty(stack: Stack) -> bool
stackClear(stack: Stack) -> void
stackToArray(stack: Stack) -> any[]
```

**Iteration strategy:** Callback-based (like array intrinsics), NO full iterator protocol in v0.2.

## Rationale
**Function-based API (not methods):**
- Consistent with current stdlib (all functions: `split`, `map`, `filter`, etc.)
- Method syntax infrastructure exists but not required for v0.2
- Can add methods later without breaking functions
- Simpler for AI code generation

**Explicit type names in function names:**
- `hashMapPut` vs `put` - clear which collection type
- Prevents naming conflicts
- Better discoverability for AI/autocomplete
- Matches pattern: `jsonParse`, `jsonStringify` (type prefix)

**Callback iteration (not Iterator protocol):**
- Reuses array intrinsic pattern (map, filter, forEach already implemented)
- No need for complex Iterator trait/protocol in v0.2
- Simpler implementation, easier parity maintenance
- Can add full iterator protocol in v0.3 if needed

**Industry patterns:**
- **Rust:** Methods on types (`.insert()`, `.get()`) - but Rust has different constraints
- **Go:** Built-in syntax (`m[key] = value`) - Atlas doesn't have syntax sugar yet
- **JavaScript:** Methods (`.set()`, `.get()`) - but JS has prototype system
- **Python:** Mix of syntax and functions (`d[k]`, `d.get(k)`) - Atlas uses functions

**AI-first optimization:**
- Clear function names (no ambiguity)
- Explicit types in signatures
- Callback pattern already familiar from arrays

## Alternatives Considered
- **Method-based API (`.put()`, `.get()`):** Rejected - requires teaching AI about method dispatch, less explicit
- **Full Iterator protocol:** Rejected - too complex for v0.2, defer to v0.3
- **Generic functions (`collectionSize()`):** Rejected - loses type safety, unclear
- **Operator overloading (`map[key]`):** Rejected - no syntax support in v0.2

## Consequences
- ✅ **Benefits:** Consistent with stdlib pattern (familiar)
- ✅ **Benefits:** Clear, explicit function names (AI-friendly)
- ✅ **Benefits:** No method dispatch complexity
- ✅ **Benefits:** Type safety at call site
- ⚠️  **Trade-offs:** Longer function names (but clearer)
- ⚠️  **Trade-offs:** No syntax sugar (acceptable for v0.2)
- ❌ **Costs:** More functions to implement (but explicit is better than implicit)

## Implementation Notes
**Phases:**
- `stdlib/phase-07a-hash-infrastructure-hashmap.md` - HashMap API
- `stdlib/phase-07b-hashset.md` - HashSet API
- `stdlib/phase-07c-queue-stack.md` - Queue/Stack APIs
- `stdlib/phase-07d-collection-integration.md` - Unified iteration, integration tests

**Function locations:**
- `crates/atlas-runtime/src/stdlib/collections/hashmap.rs` - HashMap functions
- `crates/atlas-runtime/src/stdlib/collections/hashset.rs` - HashSet functions
- `crates/atlas-runtime/src/stdlib/collections/queue.rs` - Queue functions
- `crates/atlas-runtime/src/stdlib/collections/stack.rs` - Stack functions

**Callback intrinsics (interpreter/VM direct implementation):**
- `hashMapForEach`, `hashMapMap`, `hashMapFilter`
- `hashSetForEach`, `hashSetFilter`

**Pure stdlib functions (no callback):**
- All creation, mutation, query, access functions
- Set operations (union, intersection, etc.)

## References
- Spec: `docs/api/stdlib.md` (Collections section)
- Related: DR-002 (Array Intrinsics - callback pattern)
- Related: DR-003 (Hash function design)
- Related: DR-004 (Collection value representation)
