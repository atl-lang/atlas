# Migration: C-style for → for-in / while

**Decision:** D-007 (Remove C-style for and ++/--)
**Effort:** Requires restructuring

---

## Rules

### Increment/Decrement
```
x++   →  x += 1
++x   →  x += 1
x--   →  x -= 1
--x   →  x -= 1
```

### C-style for → while
```atlas
// Before
for (init; condition; step) { body }

// After
init;
while condition {
    body
    step;
}
```

### Iteration over indices → for-in or while
```atlas
// Before
for (var i = 0; i < len(arr); i++) {
    process(arr[i]);
}

// After (preferred - direct iteration)
for item in arr {
    process(item);
}

// After (if index needed)
let mut i = 0;
while i < len(arr) {
    process(arr[i]);
    i += 1;
}
```

---

## Examples

### Simple counting loop
```atlas
// Before
for (var i = 0; i < 10; i++) {
    print(str(i));
}

// After
let mut i = 0;
while i < 10 {
    print(str(i));
    i += 1;
}
```

### Iterating array elements
```atlas
// Before
for (var i = 0; i < len(items); i++) {
    let item = items[i];
    process(item);
}

// After (better)
for item in items {
    process(item);
}
```

### Reverse iteration
```atlas
// Before
for (var i = len(arr) - 1; i >= 0; i--) {
    process(arr[i]);
}

// After
let mut i = len(arr) - 1;
while i >= 0 {
    process(arr[i]);
    i -= 1;
}

// After (when reverse() available)
for item in reverse(arr) {
    process(item);
}
```

### Step by 2
```atlas
// Before
for (var i = 0; i < 10; i += 2) {
    print(str(i));
}

// After
let mut i = 0;
while i < 10 {
    print(str(i));
    i += 2;
}
```

### Infinite loop with break
```atlas
// Before
for (;;) {
    if (done) { break; }
    doWork();
}

// After
while true {
    if done { break; }
    doWork();
}
```

---

## Increment in expressions (not supported)

v0.2 allowed `++`/`--` as statements only. If your code somehow has:
```atlas
// This was never valid, but just in case:
let y = x++;  // ERROR in both versions
```

Use explicit assignment:
```atlas
let y = x;
x += 1;
```

---

## Verification

After migration:
```bash
grep -rE '\+\+|--' --include="*.atl" .
grep -rE 'for\s*\(' --include="*.atl" .
```

Both should return no matches.
