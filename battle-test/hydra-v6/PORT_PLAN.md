# Hydra → Atlas Port Plan - PROPER AUDIT

## Pre-flight Verification
- [x] Read METHOD-CONVENTIONS.md
- [x] Tested actual API (methods work, HashMap.new() not yet)
- [x] Understand what's real vs docs

## Port Strategy (Systematic)
1. Port each domain with REAL stdlib
2. VERIFY each compiles and runs
3. Document ONLY verified issues
4. Test every claim before writing it

## Domains to Port (Order: Simple → Complex)
1. [ ] config - Data structures only
2. [ ] metrics/types - Structs, traits, no state
3. [ ] transport - Enums, protocols
4. [ ] sanitizer - String operations, JSON validation
5. [ ] statestore - HashMap usage, file I/O
6. [ ] watcher - File watching
7. [ ] proxy - State machine (will hit trait blockers)
8. [ ] supervisor - Process lifecycle (will hit trait blockers)
9. [ ] injectable - Tool definitions
10. [ ] recorder - Traffic recording
11. [ ] security - Redaction patterns
12. [ ] adaptive - Learning system
13. [ ] bootstrap - Config generation
14. [ ] cli - Command handling
15. [ ] main.go - Entry point

## Verification Rules
- ✅ Every file MUST compile
- ✅ Every file MUST run (even if stubs)
- ✅ Every "missing" claim MUST be tested
- ✅ Every friction point MUST have code example
- ❌ NO assumptions without verification

## Known REAL Issues (Verified)
1. No self field access in trait impls (VERIFIED in supervisor.atlas)
2. No mutable self in traits (VERIFIED in supervisor.atlas)
3. No inherent impl blocks (VERIFIED - all need traits)
4. Empty array type inference (VERIFIED - needs annotations)
5. HashMap.new() not implemented yet (hashMapNew() still required)

## Stdlib Status (VERIFIED)
- ✅ String methods: `s.trim()`, `s.split()`, `s.includes()`
- ✅ Array methods: `arr.push()`, `arr.map()`, `arr.filter()`, `arr.reduce()`
- ✅ JSON static: `Json.parse()`, `Json.stringify()`, `Json.isValid()`
- ✅ File static: `File.read()`, `File.write()`, `File.exists()`
- ✅ Process static: `Process.spawn()`, `Process.exit()`
- ⚠️ HashMap constructor: Still `hashMapNew()` (not `HashMap.new()` yet)
- ⚠️ HashSet constructor: Still `hashSetNew()` (not `HashSet.new()` yet)
