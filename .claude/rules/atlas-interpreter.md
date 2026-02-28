---
globs: ["crates/atlas-runtime/src/interpreter/**"]
---

# Atlas Interpreter Quick-Ref

**Verified against:** `interpreter/mod.rs`, `value.rs`
**Update trigger:** Any phase adding Value variants or interpreter methods — update at GATE 7.

---

## Interpreter Struct Fields (interpreter/mod.rs:42)

```rust
pub struct Interpreter {
    pub(super) globals: HashMap<String, (Value, bool)>,        // (value, mutable)
    pub(super) locals: Vec<HashMap<String, (Value, bool)>>,
    pub(super) consumed_locals: Vec<HashSet<String>>,
    pub(super) function_bodies: HashMap<String, UserFunction>,
    pub(super) control_flow: ControlFlow,
    pub(super) monomorphizer: Monomorphizer,
    pub(super) current_security: Option<Arc<SecurityContext>>,
    pub(super) output_writer: OutputWriter,
    next_func_id: usize,
    library_loader: LibraryLoader,
    extern_functions: HashMap<String, ExternFunction>,
    callbacks: Vec<CallbackHandle>,
    current_module_path: Option<PathBuf>,
    module_exports_cache: HashMap<PathBuf, HashMap<String, Value>>,
    lookup_cache: cache::InterpreterCache,
}
```

## ControlFlow Enum (interpreter/mod.rs:25)

```rust
pub enum ControlFlow { None, Break, Continue, Return(Value) }
```

## Value Enum (value.rs) — All Variants

```rust
pub enum Value {
    // Primitives
    Number(f64),
    Bool(bool),
    Null,
    // CoW heap types (Arc — clone is cheap, mutate via Arc::make_mut)
    String(Arc<String>),
    Array(ValueArray),        // Arc<Vec<Value>>
    HashMap(ValueHashMap),    // Arc<AtlasHashMap>
    HashSet(ValueHashSet),    // Arc<AtlasHashSet>
    Queue(ValueQueue),        // Arc<VecDeque<Value>>
    Stack(ValueStack),        // Arc<Vec<Value>>
    // Explicit shared reference (Arc<Mutex<Value>>)
    SharedValue(Arc<Mutex<Value>>),
    // Functions
    Function(FunctionRef),
    Closure(ClosureRef),
    NativeFunction(...),
    Builtin(Arc<str>),
    // Async / runtime
    Future(...),
    TaskHandle(...),
    ChannelSender(...),
    ChannelReceiver(...),
    AsyncMutex(...),
    // Other
    JsonValue(serde_json::Value),
}
```

## CoW Write-Back Pattern (CRITICAL)

Collection mutation builtins return a NEW collection. The interpreter writes it back:

```rust
// In interpreter — apply_cow_writeback() writes result to the binding
// Both `let` and `var` bindings support content mutation (not rebinding)
// VM equivalent: emit_cow_writeback_if_needed()
```

**Never mutate a collection in-place.** Always: call builtin → get new Value → write back.

## Key Interpreter Methods

```rust
interpreter.get_binding(name: &str) -> Option<Value>  // locals then globals
interpreter.eval_expr(&expr) -> Result<Value, RuntimeError>
interpreter.eval_stmt(&stmt) -> Result<(), RuntimeError>
```

## Zero-Allocation Goal (DR-B04-01)

The interpreter eval loop should not allocate on the happy path. This is a goal, not a CI gate.

**Check at block completion:** `grep -n "Vec::new\|String::new\|format!" interpreter/expr.rs | wc -l`

Does NOT apply to error paths, stdlib functions, or diagnostics.
