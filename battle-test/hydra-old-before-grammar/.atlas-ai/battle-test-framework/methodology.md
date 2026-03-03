# Atlas Battle Test Methodology

**Systematic approach to testing Atlas with real-world projects.**

Reusable framework proven on Hydra Atlas (MCP supervisor port).

---

## Philosophy

**Battle testing** = Building real software to discover language capabilities and limitations.

**Goals**:
1. Find what works (patterns, stdlib, syntax)
2. Find what doesn't (gaps, bugs, limitations)
3. Document workarounds
4. Measure feasibility for project types

**Not goals**:
- Benchmarking performance
- Finding compiler bugs (that's separate)
- Comparing to other languages

---

## Project Selection Criteria

### Good Battle Test Projects

**Characteristics**:
- Real-world utility (not toy examples)
- Well-defined scope (2000-5000 LOC target)
- Diverse feature usage (files, strings, data structures, etc.)
- Existing reference implementation (to compare against)
- Clear success criteria

**Examples**:
- CLI tools (parsers, formatters, build tools)
- Data processors (log analyzers, file transformers)
- Simple servers (if sockets available)
- Code generators
- Configuration managers

### Poor Battle Test Projects

**Avoid**:
- Trivial examples (hello world, calculators)
- Huge projects (>10K LOC)
- Highly specialized (graphics, ML, embedded)
- No clear completion criteria
- Requires features you know are missing

---

## Phase 1: Pre-Flight (Before Writing Code)

### Step 1.1: Audit Project Requirements

List ALL features needed:
- String operations (which ones?)
- File I/O (read/write/create/delete?)
- Data structures (arrays/maps/sets?)
- Process execution?
- Networking?
- JSON handling?
- Date/time?

**Output**: Requirements checklist

### Step 1.2: Check Against Atlas Reality

For EACH requirement, check:
- Is function in `stdlib-reality.md`?
- Has it been battle-tested?
- Are there known workarounds?

**Output**: Feasibility assessment (GO / CAUTION / NO-GO)

### Step 1.3: Identify Risk Areas

Flag requirements that:
- Use unconfirmed functions
- Need complex objects
- Require exec/networking
- Depend on missing features

**Output**: Risk register

### Step 1.4: Plan Component Order

Build components from least to most risky:
1. Data structures (primitives only)
2. Pure functions (algorithms)
3. File I/O (if needed)
4. Risky features (exec, objects)

**Output**: Build order

---

## Phase 2: Foundation (First 20% of Code)

### Step 2.1: Test Basic Syntax

Write minimal test for core patterns:
- Match expressions
- Result handling
- For loops
- String operations

**Goal**: Prove you understand Atlas syntax before building

```atlas
// test_foundation.atl
fn test_basics() -> void {
    // Strings
    let text: string = trim("  test  ");
    let parts: array = split(text, ",");

    // Match
    let result: Result<string, string> = Ok("success");
    let value: string = match result {
        Ok(v) => v,
        Err(_) => "default"
    };

    print("Foundation test: " + value);
}

test_basics();
```

**Output**: Passing foundation tests

### Step 2.2: Build First Component

Choose simplest component, implement fully:
- Write types/functions
- Add tests
- Verify it works

**Success criteria**: `atlas run` succeeds, outputs correct results

### Step 2.3: Document Discoveries

As you build, document:
- Functions that work differently than expected
- Syntax patterns you figured out
- Error messages and solutions

**Output**: Project-specific discoveries file

---

## Phase 3: Iteration (Middle 60% of Code)

### Step 3.1: Component-by-Component

For each component:

1. **Design** - Plan functions, types, interfaces
2. **Implement** - Write Atlas code
3. **Test** - Run and verify
4. **Debug** - Fix syntax errors (expect match issues!)
5. **Document** - Record patterns and gotchas

**Pattern**: Keep components small (300-500 LOC max)

### Step 3.2: Handle Blockers

When you hit a blocker:

1. **Confirm** - Is function really missing? Check docs, try simple test
2. **Workaround** - Can you solve it another way?
3. **Document** - Add to blockers list with details
4. **Continue or Stop** - Is this component essential?

**Decision tree**:
- If workaround exists → Use it, document, continue
- If no workaround, component non-critical → Skip, document, continue
- If no workaround, component critical → STOP, report blocker

### Step 3.3: Track Progress

Update progress file after each component:
```markdown
## Component Status
- [x] Transport - 100% complete
- [x] Sanitizer - 100% complete
- [ ] StateStore - 60% (blocked: no stringify)
- [ ] Supervisor - 20% (blocked: exec broken)
```

---

## Phase 4: Integration (Final 20%)

### Step 4.1: Connect Components

Wire components together:
- Import/export between modules
- Pass data structures
- Handle errors across boundaries

**Watch for**: Type mismatches, Result chaining

### Step 4.2: End-to-End Tests

Test complete workflows:
- Real input files
- Multi-component pipelines
- Error scenarios

**Goal**: Prove the system works as a whole

### Step 4.3: Measure Success

Calculate metrics:
```
Components attempted:   8
Components complete:    2
Components partial:     2
Components blocked:     4

Completion rate:        25%
Blocker rate:           50%
Workaround success:     50%
```

---

## Phase 5: Documentation (Final Step)

### Step 5.1: Write Battle Test Report

Document:
1. **Project overview** - What you built, why
2. **What worked** - Components, patterns, stdlib
3. **What blocked** - Missing features, bugs
4. **Workarounds** - Solutions you found
5. **Conclusion** - Is Atlas ready for this project type?

**Template**: See `project-template.md`

### Step 5.2: Extract Learnings

Create reusable docs:
- New syntax patterns discovered
- Working code snippets
- Stdlib functions verified
- Gotchas encountered

Add to `.atlas-ai/` for next battle test

### Step 5.3: Update Reality Check

Update `stdlib-reality.md` with:
- Functions confirmed working
- Functions confirmed missing
- New workarounds discovered

---

## Metrics to Track

### Quantitative
- Components attempted / completed / blocked
- Lines of code written
- Test pass rate
- Time spent debugging syntax vs logic
- Number of blockers encountered

### Qualitative
- Code readability (1-5)
- Workaround difficulty (easy/medium/hard)
- Documentation accuracy (matches reality?)
- Development experience (smooth/rough?)

---

## Success Criteria

### Complete Success
- ✅ All components implemented
- ✅ All tests passing
- ✅ Matches reference implementation behavior
- ✅ Clean, maintainable code

### Partial Success
- ✅ Core components working
- ⚠️ Some components blocked (workarounds exist)
- ✅ Tests pass for implemented parts
- ⚠️ Viable for limited use cases

### Failure
- ❌ Most components blocked
- ❌ No viable workarounds
- ❌ Cannot achieve project goals
- ❌ More limitations than capabilities

---

## Hydra Atlas Example

### Pre-Flight
- **Requirements**: File I/O, JSON, process exec, state management
- **Assessment**: ⚠️ CAUTION - exec unclear, JSON stringify missing
- **Risk**: HIGH for Supervisor, MEDIUM for StateStore

### Foundation
- ✅ Transport component (pure string processing) - SUCCESS
- ✅ Basic patterns verified

### Iteration
- ✅ Sanitizer (string filtering) - SUCCESS
- ⚠️ StateStore (JSON workaround) - PARTIAL
- ❌ Supervisor (exec blocked) - FAILED

### Integration
- ⚠️ PARTIAL - Data components work, process management doesn't

### Result
- **Completion**: 25% (2/8 components)
- **Verdict**: Atlas suitable for data processing, not process management
- **Workarounds**: 50% success rate
- **Experience**: Match syntax was 90% of errors

---

## Tips for AI Agents

### DO
- Start with simplest components
- Test incrementally
- Document everything
- Expect match expression errors
- Build workarounds library
- Track time spent on syntax vs logic

### DON'T
- Assume stdlib functions exist
- Use complex objects
- Try to implement everything if blocked
- Ignore early warnings
- Skip documentation

### When Blocked
1. Check `stdlib-reality.md`
2. Check `gotchas/`
3. Try simple test case
4. Search for workaround
5. Document blocker
6. Continue or pivot

---

## Reusing This Methodology

### For Your Project

1. **Copy framework**: `cp -r .atlas-ai /your/project/`
2. **Read quick start**: Essential patterns
3. **Follow phases**: Pre-flight → Foundation → Iteration → Integration → Docs
4. **Track metrics**: Use same format
5. **Share learnings**: Contribute back

### Adapt for Project Type

**CLI tools**: Focus on string/file processing
**Data processors**: Focus on algorithms and data structures
**Network services**: Expect blockers, plan workarounds
**System tools**: High risk, verify capabilities first

---

**This methodology achieves**:
- Systematic discovery
- Reusable knowledge
- Clear success/failure criteria
- Valuable documentation for community

**Proven on Hydra Atlas. Ready for your project.**
