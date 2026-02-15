# Phase 07c: Queue + Stack - FIFO and LIFO Collections

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Basic stdlib infrastructure must exist. HashMap/HashSet are NOT dependencies.

**Verification:**
```bash
# Verify stdlib infrastructure
ls crates/atlas-runtime/src/stdlib/mod.rs
ls crates/atlas-runtime/src/stdlib/prelude.rs

# Verify Value enum and generic types
grep -n "enum Value" crates/atlas-runtime/src/value.rs
grep -n "Option<" crates/atlas-runtime/src/value.rs

# Clean build check
cargo clean && cargo check -p atlas-runtime

# Verify tests pass
cargo test -p atlas-runtime stdlib -- --nocapture
```

**What's needed:**
- Stdlib infrastructure from v0.1
- Value enum (existing)
- Option<T> type (phase 06a)
- Test infrastructure (rstest)

**If missing:** Basic infrastructure should exist - this is independent of HashMap/HashSet

**Note:** This phase does NOT depend on phase 07a or 07b. Can be implemented independently.

---

## Objective

Implement Queue (FIFO) and Stack (LIFO) collections for fundamental data structure operations. Queue uses circular buffer (VecDeque) for O(1) enqueue/dequeue. Stack uses Vec for O(1) push/pop. Provide complete APIs for both structures with proper error handling, Option-based returns, and world-class performance.

## Files

**Create:** `crates/atlas-runtime/src/stdlib/collections/queue.rs` (~250 lines)
**Create:** `crates/atlas-runtime/src/stdlib/collections/stack.rs` (~200 lines)
**Update:** `crates/atlas-runtime/src/stdlib/collections/mod.rs` (~30 lines - add queue/stack modules)
**Update:** `crates/atlas-runtime/src/value.rs` (~60 lines - Queue + Stack variants)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (~150 lines - Queue + Stack functions)
**Update:** `docs/api/stdlib.md` (~300 lines - Queue + Stack documentation)
**Tests:** `crates/atlas-runtime/tests/queue_tests.rs` (~400 lines)
**Tests:** `crates/atlas-runtime/tests/stack_tests.rs` (~350 lines)

**Total new code:** ~690 lines
**Total tests:** ~750 lines (30+ test cases)

## Dependencies

- Value enum (existing)
- Option<T> type (phase 06a)
- Stdlib infrastructure (prelude, native functions)
- Rust VecDeque (for Queue)
- Rust Vec (for Stack)

## Implementation

### 1. Queue Implementation (`collections/queue.rs`)

**AtlasQueue using VecDeque for O(1) operations:**
```rust
use crate::value::Value;
use std::collections::VecDeque;

/// Atlas Queue - FIFO collection with O(1) enqueue/dequeue
/// Backed by VecDeque (circular buffer)
pub struct AtlasQueue {
    inner: VecDeque<Value>,
}

impl AtlasQueue {
    /// Create new empty queue
    pub fn new() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }

    /// Create queue with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
        }
    }

    /// Add element to back of queue (FIFO)
    pub fn enqueue(&mut self, value: Value) {
        self.inner.push_back(value);
    }

    /// Remove and return element from front of queue
    /// Returns None if empty
    pub fn dequeue(&mut self) -> Option<Value> {
        self.inner.pop_front()
    }

    /// View front element without removing
    /// Returns None if empty
    pub fn peek(&self) -> Option<&Value> {
        self.inner.front()
    }

    /// Get number of elements
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all elements
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Convert to array (preserves FIFO order)
    pub fn to_vec(&self) -> Vec<Value> {
        self.inner.iter().cloned().collect()
    }
}

impl Default for AtlasQueue {
    fn default() -> Self {
        Self::new()
    }
}
```

### 2. Stack Implementation (`collections/stack.rs`)

**AtlasStack using Vec for O(1) operations:**
```rust
use crate::value::Value;

/// Atlas Stack - LIFO collection with O(1) push/pop
/// Backed by Vec for maximum performance
pub struct AtlasStack {
    inner: Vec<Value>,
}

impl AtlasStack {
    /// Create new empty stack
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
        }
    }

    /// Create stack with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Push element onto top of stack (LIFO)
    pub fn push(&mut self, value: Value) {
        self.inner.push(value);
    }

    /// Pop element from top of stack
    /// Returns None if empty
    pub fn pop(&mut self) -> Option<Value> {
        self.inner.pop()
    }

    /// View top element without removing
    /// Returns None if empty
    pub fn peek(&self) -> Option<&Value> {
        self.inner.last()
    }

    /// Get number of elements
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all elements
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Convert to array (bottom to top order)
    pub fn to_vec(&self) -> Vec<Value> {
        self.inner.clone()
    }
}

impl Default for AtlasStack {
    fn default() -> Self {
        Self::new()
    }
}
```

### 3. Value Enum Updates (`value.rs`)

Add Queue and Stack variants:

```rust
use crate::stdlib::collections::queue::AtlasQueue;
use crate::stdlib::collections::stack::AtlasStack;

pub enum Value {
    // Existing variants...
    HashMap(Rc<RefCell<AtlasHashMap>>),
    HashSet(Rc<RefCell<AtlasHashSet>>),

    // New: Queue and Stack collections
    Queue(Rc<RefCell<AtlasQueue>>),
    Stack(Rc<RefCell<AtlasStack>>),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            // Existing cases...
            Value::Queue(_) => "queue",
            Value::Stack(_) => "stack",
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Existing cases...
            (Value::Queue(_), Value::Queue(_)) => false, // Reference type
            (Value::Stack(_), Value::Stack(_)) => false, // Reference type
            _ => false,
        }
    }
}
```

### 4. Stdlib Functions (`stdlib/prelude.rs`)

**Queue Functions (8 functions):**
```rust
fn queue_new(_args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    Ok(Value::Queue(Rc::new(RefCell::new(AtlasQueue::new()))))
}

fn queue_enqueue(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    // args[0]: Queue, args[1]: element
    let queue = expect_queue(&args[0], "queueEnqueue")?;
    let element = args[1].clone();

    queue.borrow_mut().enqueue(element);
    Ok(Value::Null)
}

fn queue_dequeue(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let queue = expect_queue(&args[0], "queueDequeue")?;

    let value = queue.borrow_mut().dequeue();
    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

fn queue_peek(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let queue = expect_queue(&args[0], "queuePeek")?;

    let value = queue.borrow().peek().cloned();
    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

fn queue_size(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let queue = expect_queue(&args[0], "queueSize")?;
    Ok(Value::Number(queue.borrow().len() as f64))
}

fn queue_is_empty(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let queue = expect_queue(&args[0], "queueIsEmpty")?;
    Ok(Value::Bool(queue.borrow().is_empty()))
}

fn queue_clear(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let queue = expect_queue(&args[0], "queueClear")?;
    queue.borrow_mut().clear();
    Ok(Value::Null)
}

fn queue_to_array(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let queue = expect_queue(&args[0], "queueToArray")?;
    let elements = queue.borrow().to_vec();
    Ok(Value::Array(Rc::new(RefCell::new(elements))))
}

fn expect_queue(value: &Value, func_name: &str) -> Result<Rc<RefCell<AtlasQueue>>, RuntimeError> {
    match value {
        Value::Queue(q) => Ok(Rc::clone(q)),
        _ => Err(RuntimeError::new(
            ErrorCode::AT0102,
            format!("{} expects Queue, got {}", func_name, value.type_name())
        ))
    }
}
```

**Stack Functions (8 functions):**
```rust
fn stack_new(_args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    Ok(Value::Stack(Rc::new(RefCell::new(AtlasStack::new()))))
}

fn stack_push(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    // args[0]: Stack, args[1]: element
    let stack = expect_stack(&args[0], "stackPush")?;
    let element = args[1].clone();

    stack.borrow_mut().push(element);
    Ok(Value::Null)
}

fn stack_pop(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let stack = expect_stack(&args[0], "stackPop")?;

    let value = stack.borrow_mut().pop();
    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

fn stack_peek(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let stack = expect_stack(&args[0], "stackPeek")?;

    let value = stack.borrow().peek().cloned();
    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

fn stack_size(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let stack = expect_stack(&args[0], "stackSize")?;
    Ok(Value::Number(stack.borrow().len() as f64))
}

fn stack_is_empty(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let stack = expect_stack(&args[0], "stackIsEmpty")?;
    Ok(Value::Bool(stack.borrow().is_empty()))
}

fn stack_clear(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let stack = expect_stack(&args[0], "stackClear")?;
    stack.borrow_mut().clear();
    Ok(Value::Null)
}

fn stack_to_array(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let stack = expect_stack(&args[0], "stackToArray")?;
    let elements = stack.borrow().to_vec();
    Ok(Value::Array(Rc::new(RefCell::new(elements))))
}

fn expect_stack(value: &Value, func_name: &str) -> Result<Rc<RefCell<AtlasStack>>, RuntimeError> {
    match value {
        Value::Stack(s) => Ok(Rc::clone(s)),
        _ => Err(RuntimeError::new(
            ErrorCode::AT0102,
            format!("{} expects Stack, got {}", func_name, value.type_name())
        ))
    }
}
```

**Register functions:**
```rust
// In create_prelude()

// Queue functions
register_native!(prelude, "queueNew", queue_new, 0);
register_native!(prelude, "queueEnqueue", queue_enqueue, 2);
register_native!(prelude, "queueDequeue", queue_dequeue, 1);
register_native!(prelude, "queuePeek", queue_peek, 1);
register_native!(prelude, "queueSize", queue_size, 1);
register_native!(prelude, "queueIsEmpty", queue_is_empty, 1);
register_native!(prelude, "queueClear", queue_clear, 1);
register_native!(prelude, "queueToArray", queue_to_array, 1);

// Stack functions
register_native!(prelude, "stackNew", stack_new, 0);
register_native!(prelude, "stackPush", stack_push, 2);
register_native!(prelude, "stackPop", stack_pop, 1);
register_native!(prelude, "stackPeek", stack_peek, 1);
register_native!(prelude, "stackSize", stack_size, 1);
register_native!(prelude, "stackIsEmpty", stack_is_empty, 1);
register_native!(prelude, "stackClear", stack_clear, 1);
register_native!(prelude, "stackToArray", stack_to_array, 1);
```

### 5. Documentation (`docs/api/stdlib.md`)

Add Queue and Stack sections:

```markdown
### Queue Functions

Queue provides FIFO (First-In-First-Out) collection using circular buffer. O(1) enqueue and dequeue operations.

#### queueNew
**Signature:** `queueNew() -> Queue`
**Behavior:** Creates new empty queue
**Example:** `let q = queueNew()`

#### queueEnqueue
**Signature:** `queueEnqueue(queue: Queue, element: any) -> void`
**Behavior:** Adds element to back of queue
**Example:** `queueEnqueue(q, "first"); queueEnqueue(q, "second")`

#### queueDequeue
**Signature:** `queueDequeue(queue: Queue) -> Option<any>`
**Behavior:** Removes and returns front element, None if empty
**Example:** `let first = queueDequeue(q)` → Some("first")

[... continue for all 8 queue functions ...]

### Stack Functions

Stack provides LIFO (Last-In-First-Out) collection. O(1) push and pop operations.

#### stackNew
**Signature:** `stackNew() -> Stack`
**Behavior:** Creates new empty stack
**Example:** `let s = stackNew()`

#### stackPush
**Signature:** `stackPush(stack: Stack, element: any) -> void`
**Behavior:** Pushes element onto top of stack
**Example:** `stackPush(s, 1); stackPush(s, 2)`

#### stackPop
**Signature:** `stackPop(stack: Stack) -> Option<any>`
**Behavior:** Pops and returns top element, None if empty
**Example:** `let top = stackPop(s)` → Some(2)

[... continue for all 8 stack functions ...]

**Examples:**
```atlas
// Queue example (FIFO)
let q = queueNew()
queueEnqueue(q, "first")
queueEnqueue(q, "second")
queueEnqueue(q, "third")

queueDequeue(q)  // Some("first")
queueDequeue(q)  // Some("second")
queueDequeue(q)  // Some("third")
queueDequeue(q)  // None

// Stack example (LIFO)
let s = stackNew()
stackPush(s, 1)
stackPush(s, 2)
stackPush(s, 3)

stackPop(s)  // Some(3)
stackPop(s)  // Some(2)
stackPop(s)  // Some(1)
stackPop(s)  // None
```
```

## Tests (TDD - Use rstest)

### Queue Tests (`tests/queue_tests.rs`) - 18 tests

**Creation:**
1. Create empty queue with queueNew()
2. New queue has size 0 and isEmpty true

**Enqueue and Dequeue:**
3. Enqueue element increases size
4. Dequeue returns elements in FIFO order
5. Enqueue multiple, dequeue multiple preserves order
6. Dequeue from empty queue returns None
7. Enqueue after dequeue works correctly
8. Queue accepts any value type (numbers, strings, arrays, etc.)

**Peek:**
9. Peek returns front element without removing
10. Peek on empty queue returns None
11. Peek doesn't change size

**Size and IsEmpty:**
12. Size reflects current element count
13. IsEmpty true for new queue
14. IsEmpty false after enqueue
15. IsEmpty true after dequeuing all elements

**Clear:**
16. Clear removes all elements
17. Clear on empty queue is safe

**ToArray:**
18. ToArray returns elements in FIFO order
19. ToArray doesn't modify queue
20. ToArray on empty queue returns empty array

**Integration:**
21. Multiple queues are independent
22. Queue reference semantics (assignment shares reference)
23. Large queue (1000+ elements) performance test

**Minimum: 18 tests**

### Stack Tests (`tests/stack_tests.rs`) - 18 tests

**Creation:**
1. Create empty stack with stackNew()
2. New stack has size 0 and isEmpty true

**Push and Pop:**
3. Push element increases size
4. Pop returns elements in LIFO order
5. Push multiple, pop multiple preserves reverse order
6. Pop from empty stack returns None
7. Push after pop works correctly
8. Stack accepts any value type

**Peek:**
9. Peek returns top element without removing
10. Peek on empty stack returns None
11. Peek doesn't change size

**Size and IsEmpty:**
12. Size reflects current element count
13. IsEmpty true for new stack
14. IsEmpty false after push
15. IsEmpty true after popping all elements

**Clear:**
16. Clear removes all elements
17. Clear on empty stack is safe

**ToArray:**
18. ToArray returns elements bottom to top
19. ToArray doesn't modify stack
20. ToArray on empty stack returns empty array

**Integration:**
21. Multiple stacks are independent
22. Stack reference semantics (assignment shares reference)
23. Large stack (1000+ elements) performance test

**Minimum: 18 tests**

**Total minimum test count:** 36 tests (18 queue + 18 stack)

**Parity requirement:** All tests run in both interpreter and VM with identical results.

## Integration Points

- Uses: Value enum for storage
- Uses: Option<T> for optional returns
- Uses: Stdlib infrastructure (prelude, native functions)
- Uses: Rust VecDeque (Queue) and Vec (Stack)
- Creates: Queue collection type
- Creates: Stack collection type
- Updates: Value enum with Queue and Stack variants
- Output: 16 functions (8 queue + 8 stack) in stdlib

## Acceptance

- ✅ Queue variant added to Value enum
- ✅ Stack variant added to Value enum
- ✅ AtlasQueue struct implemented with VecDeque
- ✅ AtlasStack struct implemented with Vec
- ✅ All 8 Queue functions implemented and registered
- ✅ All 8 Stack functions implemented and registered
- ✅ queueNew, queueEnqueue, queueDequeue work correctly
- ✅ queuePeek, queueSize, queueIsEmpty, queueClear work correctly
- ✅ queueToArray works correctly
- ✅ stackNew, stackPush, stackPop work correctly
- ✅ stackPeek, stackSize, stackIsEmpty, stackClear work correctly
- ✅ stackToArray works correctly
- ✅ Queue maintains FIFO order
- ✅ Stack maintains LIFO order
- ✅ Empty queue/stack operations return None (Option)
- ✅ 36+ tests pass (18 queue + 18 stack)
- ✅ 100% interpreter/VM parity verified
- ✅ Documentation complete in docs/api/stdlib.md
- ✅ No clippy warnings
- ✅ cargo test -p atlas-runtime passes
- ✅ Decision logs DR-004, DR-005 referenced

## References

**Decision Logs:**
- DR-004: Collection value representation
- DR-005: Collection API design

**Specifications:**
- `docs/specification/runtime.md` - Value representation
- `docs/api/stdlib.md` - Queue and Stack APIs

**Implementation:**
- Rust VecDeque: https://doc.rust-lang.org/std/collections/struct.VecDeque.html
- Rust Vec: https://doc.rust-lang.org/std/vec/struct.Vec.html

**Next Phase:** `phase-07d-collection-integration.md` (unified iteration + integration tests)
