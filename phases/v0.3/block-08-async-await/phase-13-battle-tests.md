# Phase 13: Battle Tests

## Dependencies

**Required:** Phase 12 (parity sweep complete)

**Verification:**
```bash
ls battle-tests/async-*/
atlas-track issues | grep battle
```

---

## Objective

Validate async/await against real-world Atlas programs: concurrent HTTP, file I/O pipelines, producer/consumer patterns, error recovery, and stress tests. Catch what unit tests miss.

---

## Files

**Create:** `battle-tests/async-basic/` — simple async programs
**Create:** `battle-tests/async-concurrency/` — concurrent task patterns
**Create:** `battle-tests/async-error-recovery/` — error handling under async
**Create:** `battle-tests/async-stdlib/` — full stdlib async integration

**Total new code:** ~500 lines Atlas test programs, ~80 lines Rust test harness

---

## Battle Test Programs

### async-basic/
```atlas
// 01_simple_fetch.atl — await a simulated fetch
async fn fetch_data(url: string) -> string {
    await sleep(10)
    return "data from " + url
}
let result = await fetch_data("https://api.example.com")
print(result)
// Expected: "data from https://api.example.com"
```

```atlas
// 02_sequential_awaits.atl — multiple sequential awaits
async fn step1() -> number { await sleep(0); return 1 }
async fn step2(n: number) -> number { await sleep(0); return n + 1 }
async fn step3(n: number) -> number { await sleep(0); return n * 2 }

let a = await step1()
let b = await step2(a)
let c = await step3(b)
print(c)
// Expected: 4
```

### async-concurrency/
```atlas
// 01_parallel_tasks.atl — spawn multiple concurrent tasks
async fn compute(n: number) -> number {
    await sleep(0)
    return n * n
}

let f1 = spawn(compute(3))
let f2 = spawn(compute(4))
let f3 = spawn(compute(5))

let results = await all([f1, f2, f3])
print(results)
// Expected: [9, 16, 25]
```

```atlas
// 02_race.atl — first future wins
async fn slow() -> string { await sleep(1000); return "slow" }
async fn fast() -> string { await sleep(1);    return "fast" }

let winner = await race([slow(), fast()])
print(winner)
// Expected: "fast"
```

### async-error-recovery/
```atlas
// 01_error_propagation.atl
async fn might_fail(n: number) -> Result<number, string> {
    if n < 0 { return Err("negative input") }
    return Ok(n * 2)
}

let result = await might_fail(-1)
match result {
    Ok(v) => print(v),
    Err(e) => print("Error: " + e)
}
// Expected: "Error: negative input"
```

### async-stdlib/
```atlas
// 01_file_pipeline.atl — read → transform → write
let content = await read_file_async("input.txt")
let transformed = content.to_upper()
await write_file_async("output.txt", transformed)
print("done")
// Expected: "done"
```

---

## Acceptance Criteria

- ✅ 10+ battle test programs authored
- ✅ All programs run correctly in interpreter and VM (parity)
- ✅ Concurrent programs complete without deadlock
- ✅ Error recovery programs produce correct error output
- ✅ File I/O battle test passes
- ✅ `atlas run` executes all battle test files without crash

---

## References

**Decision Logs:** D-030 (multi-thread — concurrent battle tests validate this)
**Auto-memory:** `compiler-quality/battle-testing.md`
**Related phases:** Phase 12 (parity baseline), Phase 14 (LSP), Phase 15 (AC verification)
