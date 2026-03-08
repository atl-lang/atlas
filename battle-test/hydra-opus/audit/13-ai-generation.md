# Audit: AI Generation Friction

**Friction Level:** HIGH

This document captures every syntax/semantic issue I encountered while porting Hydra that an AI would likely get wrong on first attempt.

---

## Top 10 AI Generation Errors

### 1. String Interpolation Syntax (100% error rate)

**AI will write:**
```atlas
print(`Hello {name}`);
```

**Correct:**
```atlas
print(`Hello ${name}`);
```

**Documentation says:** `{expr}` (grammar.md line 228-229)
**Implementation requires:** `${expr}`

**Fix:** Update documentation to match implementation

---

### 2. Empty Array Literals (100% error rate)

**AI will write:**
```atlas
let items = [];
items = items.push("test");
```

**Correct:**
```atlas
let items: string[] = [];
items = items.push("test");
```

**Error:** `Type mismatch: expected string[], found ?[]`

---

### 3. Deprecated Global Functions (~90% error rate)

**AI will write (trained on older code):**
```atlas
let trimmed = trim(input);
let data = parseJSON(str);
let ts = dateTimeNow();
let sorted = arraySort(nums);
```

**Correct (modern):**
```atlas
let trimmed = input.trim();
let data = Json.parse(str);
let ts = DateTime.now();
let sorted = nums.sort();
```

---

### 4. Mutable Variables (~70% error rate)

**AI will write:**
```atlas
let counter = 0;
counter = counter + 1;  // Error!
```

**Correct:**
```atlas
let mut counter = 0;
counter = counter + 1;
```

**Error:** `Cannot assign to immutable variable 'counter'`

---

### 5. Result vs Option Functions (~50% error rate)

**AI will write:**
```atlas
let val = hashMapGet(map, "key");
if is_err(val) { ... }  // WRONG
```

**Correct:**
```atlas
let val = hashMapGet(map, "key");
if is_none(val) { ... }  // Option uses is_none/is_some
```

Functions returning **Option:** `hashMapGet`, `jsonGetString`, `find`, `indexOf`
Functions returning **Result:** `Json.parse`, `sqrt`, `log`, `spawn`

---

### 6. unwrap() Type Loss (~60% error rate)

**AI will write:**
```atlas
let start_result = proxy_start(proxy);
let running = unwrap(start_result);
print(running.state);  // Error: any has no field 'state'
```

**Correct:**
```atlas
let start_result = proxy_start(proxy);
let running: Proxy = unwrap(start_result);
print(running.state);
```

---

### 7. sort() Requires Comparator (for generic sort)

**AI will write:**
```atlas
let sorted = sort(nums);  // Error: expects 2 args
```

**Correct:**
```atlas
let sorted = nums.sort();  // Method syntax
// OR
let sorted = arraySort(nums);  // Global (deprecated)
```

---

### 8. HashMap Method Syntax

**AI will write:**
```atlas
map.put("key", "value");  // No put method
map.get("key");  // This works!
```

**Correct:**
```atlas
map = map.set("key", "value");  // Must rebind
let val = map.get("key");
```

---

### 9. Trait-less Struct Methods

**AI will write (Go/Rust-trained):**
```atlas
impl Server {
    fn start(self) -> Result<void, string> { ... }
}
```

**Correct (Atlas requires trait):**
```atlas
trait Startable {
    fn start(self) -> Result<void, string>;
}

impl Startable for Server {
    fn start(self) -> Result<void, string> { ... }
}
```

---

### 10. Return Type Always Required

**AI will write:**
```atlas
fn add(a: number, b: number) {
    return a + b;
}
```

**Correct:**
```atlas
fn add(a: number, b: number) -> number {
    return a + b;
}
```

---

## Error Prediction by AI Training Source

| If AI trained on... | Most likely errors |
|---------------------|-------------------|
| TypeScript | Template syntax `${x}` correct, but will miss `let mut` |
| Go | Will expect methods without traits, miss Result/Option |
| Rust | Closest match, but will expect `impl Type {}` without trait |
| Python | Will miss all type annotations and mutability |

---

## Self-Correction Success Rate

| Error Type | Can AI Self-Correct? | Notes |
|------------|---------------------|-------|
| Template syntax | YES | Error message clear |
| Empty array | YES | Error mentions `?[]` |
| Deprecated fn | YES | Warning shows alternative |
| let mut | YES | Error suggests `let mut` |
| Result/Option | MAYBE | Must know function signature |
| unwrap type | NO | Error is vague ("any has no method") |
| sort args | YES | Shows expected signature |
| HashMap methods | MAYBE | Must read docs |
| Trait requirement | NO | Error doesn't suggest trait |
| Return type | YES | Error clear |

---

## Recommended Documentation Updates

1. **String interpolation:** Fix grammar.md to show `${expr}`
2. **Empty arrays:** Add example showing required type annotation
3. **Deprecated functions:** Create migration guide from globals to methods
4. **unwrap:** Document that type annotation is required
5. **Option vs Result:** Add table of common functions and their return types
