# Atlas AI Battle-Test Framework

**Purpose**: Production-ready AI guide for building Atlas projects
**Status**: CORRECTED based on 2026-03-03 codebase audit
**Atlas Version**: v0.2 (439 stdlib functions)

---

## CRITICAL UPDATE

Previous documentation was **severely outdated**. After thorough audit:

| Previously Claimed | Reality |
|-------------------|---------|
| No JSON stringify | `toJSON()` works |
| No exec() | `exec()`, `shell()`, `spawn()` work |
| No mkdir | `createDir()` works |
| Limited strings | 20+ string methods |
| No HashMap/HashSet | Fully implemented |
| No HTTP | Full client, 35+ tests |
| No crypto | SHA, HMAC, AES-GCM, Base64 |
| No networking | TCP, UDP, TLS, WebSocket |

**Only actual gaps**: User-defined struct/enum (v0.4), object literal syntax

---

## Quick Start

### 1. Essential Reading
- **[Stdlib Reality (CORRECTED)](./quick-start/stdlib-reality.md)** - 439 functions available
- **[Grammar Reference](./reference/GRAMMAR.md)** - Complete syntax
- **[Critical Gaps (CORRECTED)](./language-reality/critical-gaps.md)** - Only 2 gaps

### 2. Syntax Cheatsheet
- **[Syntax Cheatsheet](./quick-start/syntax-cheatsheet.md)** - Copy-paste patterns

### 3. Gotchas
- **[Match Expressions](./gotchas/match-expressions.md)** - #1 syntax trap

---

## What Atlas v0.2 Can Do

### Core Language
- Full type system with generics
- Pattern matching (match expressions)
- Result/Option with `?` operator
- Closures and HOFs
- CoW semantics

### Stdlib Categories (439 functions)
| Category | Functions | Status |
|----------|-----------|--------|
| JSON | parse, serialize | Works |
| File I/O | read, write, mkdir, exists, remove, list | Works |
| Process | exec, shell, spawn | Works |
| Strings | 20+ methods | Works |
| Arrays | 20+ methods + HOFs | Works |
| HashMap | 10 methods | Works |
| HashSet | 7 methods | Works |
| DateTime | now, sleep, arithmetic | Works |
| HTTP | GET/POST/PUT/DELETE/PATCH | Works |
| Regex | 8 functions | Works |
| Crypto | SHA, HMAC, AES-GCM, Base64 | Works |
| Networking | TCP, UDP, TLS, WebSocket | Works |
| Math | Full suite | Works |
| Type checks | typeof, isNumber, etc. | Works |

### What's NOT Available (v0.4)
- User-defined `struct` types
- User-defined `enum` types
- Object literal syntax `{key: value}`

**Workarounds**: Use `hashMapNew()` or `parseJSON()`

---

## Framework Structure

```
.atlas-ai/
├── quick-start/
│   ├── README.md              # 5-minute guide
│   ├── syntax-cheatsheet.md   # Copy-paste patterns
│   └── stdlib-reality.md      # CORRECTED - what actually works
├── language-reality/
│   └── critical-gaps.md       # CORRECTED - only 2 gaps
├── reference/
│   └── GRAMMAR.md             # Complete syntax reference
├── gotchas/
│   └── match-expressions.md   # Common syntax traps
└── battle-test-framework/
    └── methodology.md         # Testing approach
```

---

## Quick Test

```atlas
// test.atl - Verify stdlib works
fn main() -> void {
    // JSON
    let data = parseJSON("{\"name\": \"test\"}");
    print(toJSON(data));

    // File I/O
    let result = writeFile("test.txt", "hello");
    match result {
        Ok(_) => print("Written!"),
        Err(e) => print("Error: " + e),
    }

    // HashMap
    let map = hashMapNew();
    hashMapPut(map, "key", "value");
    print(hashMapGet(map, "key"));

    // Process
    let cmd = exec(["echo", "hello"]);
    match cmd {
        Ok(r) => print(r.stdout),
        Err(e) => print("Error: " + e),
    }
}

main();
```

Run: `atlas run test.atl`

---

## Battle Test Results: Hydra Atlas

**Project**: Port Hydra (MCP supervisor) from Go to Atlas
**Original**: ~4,000 LOC Go, 8 components

### Previous Assessment (WRONG)
| Component | Status | Reason |
|-----------|--------|--------|
| Transport | Done | - |
| Sanitizer | Done | - |
| StateStore | Partial | "No stringify" |
| Supervisor | Blocked | "No exec" |
| Watcher | Blocked | "No file ops" |
| Others | Blocked | - |

### Corrected Assessment
| Component | Status | Notes |
|-----------|--------|-------|
| Transport | Done | Works |
| Sanitizer | Done | Works |
| StateStore | **Feasible** | toJSON + createDir exist |
| Supervisor | **Feasible** | exec() works |
| Watcher | **Feasible** | fileInfo + readDir exist |
| Proxy | **Feasible** | Depends on Supervisor |
| Config | **Feasible** | JSON serialization works |

**Conclusion**: Most "blocked" components should now work.

---

## For Other Projects

This framework is designed to be copied:

```bash
cp -r .atlas-ai /your/project/
```

Then:
1. Read `quick-start/stdlib-reality.md`
2. Reference `reference/GRAMMAR.md`
3. Check `gotchas/` for common traps
4. Use `battle-test-framework/methodology.md` for testing

---

## Official Atlas Docs

**Location**: `~/dev/projects/atlas/docs/`

| File | Purpose |
|------|---------|
| `specification/syntax.md` | Grammar |
| `specification/types.md` | Type system |
| `specification/stdlib.md` | Stdlib spec |

---

## Contributing

If you discover:
- New patterns
- Workarounds
- Issues

Document them and share with the Atlas community.

---

**Last Updated**: 2026-03-03 (post-codebase audit)
**Framework Version**: 2.0 (major corrections)
**Atlas Version**: v0.2
