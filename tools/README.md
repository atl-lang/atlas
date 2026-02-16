# Atlas Development Tools

**Purpose:** Scripts and utilities for Atlas development workflow.

---

## Available Tools

### validate-phase.sh

**Purpose:** Validate phase files before execution to catch assumptions early.

**Usage:**
```bash
./tools/validate-phase.sh phases/stdlib/phase-07d-collection-integration.md
```

**What it checks:**
1. ✅ Phase file exists
2. ✅ Core runtime files exist (value.rs, interpreter, VM, stdlib)
3. ✅ Memory system exists (patterns, decisions, gates, testing)
4. ✅ Phase-specific file references are valid
5. ✅ `cargo check -p atlas-runtime` passes

**Output:**
```
✅ Validation PASSED - Phase file is accurate
```

or

```
❌ Validation FAILED - 3 error(s) found

Fix these issues before proceeding:
1. Ensure all referenced files exist
2. Update phase file to reference correct paths
3. Fix any build errors
```

**When to use:**
- Before starting phase execution (part of GATE -1)
- After updating phase files
- When phase file references seem suspicious

**Exit codes:**
- `0` - Validation passed
- `1` - Validation failed

---

## Future Tools (Planned)

### parity-check.sh

Automated parity verification between interpreter and VM.

### benchmark-runner.sh

Run collection benchmarks and generate reports.

### phase-generator.sh

Generate new phase files from template with validation.

---

## Contributing

When adding new tools:

1. Follow existing patterns (bash scripts, exit codes, clear output)
2. Add documentation to this README
3. Make executable: `chmod +x tools/your-tool.sh`
4. Test thoroughly before committing

---

## Notes

**Location:** All tools in `tools/` directory at project root.

**Requirements:**
- Bash shell (zsh compatible)
- Cargo (for build/test commands)
- Standard Unix utilities (grep, find, etc.)

**Compatibility:**
- macOS ✅
- Linux ✅
- Windows (WSL) ✅
