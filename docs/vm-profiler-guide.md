# Atlas VM Profiler Guide

**Version:** v0.2 | **Status:** Production Ready

The Atlas profiler collects runtime performance data to help identify bottlenecks and guide optimization efforts.

---

## Overview

The profiler instruments VM execution to collect:
- **Call counts** — how many times each function is called
- **Execution time** — total and average time spent in each function
- **Hot spots** — instruction-level frequency data
- **Allocation data** — object and array creation counts

Profiler overhead is under 10% in standard profiling mode.

---

## Using the Profiler

### Via CLI

```bash
atlas run main.atl --profile                    # enable profiling, print report after
atlas run main.atl --profile --profile-out=profile.json  # save to JSON file
atlas bench main.atl                            # benchmarks include profiler data
```

### Profile Output Format

```
Atlas Profile Report
====================
Total execution time: 1.234s

Function Profiles (sorted by total time):
─────────────────────────────────────────────────────────────────
Function              Calls    Total Time    Avg Time    % Total
─────────────────────────────────────────────────────────────────
compute_matrix        1        823ms         823ms       66.7%
multiply_row          100      412ms         4.12ms      33.4%
dot_product           10000    380ms         38μs        30.8%
format_output         1        11ms          11ms         0.9%
─────────────────────────────────────────────────────────────────
```

### Via Embedding API

```rust
use atlas_runtime::api::{Runtime, ExecutionMode};
use atlas_runtime::profiler::ProfileData;

let mut runtime = Runtime::new(ExecutionMode::VM);
runtime.enable_profiling(true);

runtime.eval_file("main.atl").unwrap();

let profile = runtime.get_profile();
for entry in &profile.functions {
    println!("{}: {}ms ({} calls)", entry.name, entry.total_ms, entry.call_count);
}
```

---

## Interpreting Profile Data

### Finding Bottlenecks

Focus on functions with high **% Total** time:

```
1. If one function dominates (>50%): optimize that function first
2. If many small functions: look for algorithmic improvements
3. If stdlib calls dominate: check for redundant calls
```

### Call Count Analysis

```
High call count + low total time = efficient hot path (good)
High call count + high total time = critical optimization target
Low call count + high total time = expensive single operation
```

### Reading the JSON Output

```json
{
  "total_duration_ms": 1234,
  "functions": [
    {
      "name": "compute_matrix",
      "call_count": 1,
      "total_ms": 823.4,
      "avg_ms": 823.4,
      "min_ms": 823.4,
      "max_ms": 823.4,
      "percent_total": 66.7
    }
  ],
  "stdlib_calls": {
    "sqrt": { "count": 10000, "total_ms": 12.3 },
    "len": { "count": 50000, "total_ms": 8.1 }
  }
}
```

---

## Profiling Workflows

### Workflow 1: Finding the Slowest Function

```bash
atlas run my_program.atl --profile 2>&1 | head -20
```

Look at the first entry — that's your biggest target.

### Workflow 2: Comparing Before/After

```bash
# Before optimization
atlas run main.atl --profile --profile-out=before.json

# Make changes
# ...

# After optimization
atlas run main.atl --profile --profile-out=after.json

# Compare (manual or use diff tools)
```

### Workflow 3: Profiling a Specific Scenario

```atlas
// Use sleep to separate phases in profile
fn phase_one() -> void { /* ... */ }
fn phase_two() -> void { /* ... */ }

phase_one();
sleep(0);    // visual separator in timeline
phase_two();
```

### Workflow 4: Identifying Stdlib Overuse

If `len`, `split`, or `parseJSON` appear many times in hot loops, consider:
- Caching the result outside the loop
- Restructuring to avoid repeated computation

```atlas
// Bad: len() called every iteration
for i in 0..len(arr) { ... }

// Better: cache the length
let n = len(arr);
for i in 0..n { ... }
```

---

## Profiling Atlas Code

### Example: Profiling a Sort Algorithm

```atlas
fn bubble_sort(arr: array) -> array {
    let n = len(arr);
    let result = arr;
    for i in 0..n {
        for j in 0..(n - i - 1) {
            if result[j] > result[j + 1] {
                let tmp = result[j];
                result[j] = result[j + 1];
                result[j + 1] = tmp;
            }
        }
    }
    return result;
}

// Generate test data
let data = [];
for i in 0..1000 {
    data = data + [random_int(0, 10000)];
}

// Profile the sort
let sorted = bubble_sort(data);
```

Run with:
```bash
atlas run sort_test.atl --profile
```

The profile will show `bubble_sort` dominating, confirming it's the hot path.

---

## Profiler Overhead

The Atlas profiler is designed for production-safe profiling:

| Mode | Overhead |
|------|---------|
| Standard profiling | < 10% |
| Disabled (default) | 0% |

The profiler uses sampling-based timing at function boundaries, not instruction-level instrumentation, keeping overhead low.

---

## Best Practices

1. **Profile before optimizing** — Don't guess where the bottleneck is
2. **Profile with realistic data** — Small inputs may not reveal true bottlenecks
3. **Focus on the hot path** — 80% of time is usually in 20% of functions
4. **Measure after each change** — Verify optimizations actually help
5. **Disable for production** — Profiling is for development only

---

## Integration with Optimizer

Use profiler data to guide optimizer settings:

```bash
# Profile to find hot functions
atlas run main.atl --profile

# Run with aggressive optimization for hot code paths
atlas run main.atl --optimize=aggressive
```

---

*See also: [VM Optimizer Guide](vm-optimizer-guide.md) | [VM Debugger Guide](vm-debugger-guide.md) | [CLI Reference](cli-reference.md)*
