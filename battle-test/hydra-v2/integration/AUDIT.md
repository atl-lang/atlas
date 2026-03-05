# Component Audit: Integration

**Grammar Friction Points**
- Imports do not work in practice (CLI test runner only recognizes `.atlas` and cannot resolve imports).
- `hashMapPut` is non-mutating; integration glue must reassign maps explicitly.
- `if` requires parentheses; missing parens cause confusing parse failures.

**Missing Features (Atlas should have)**
- Module-level type exports with clear import semantics.
- Package manager or local module discovery (currently manual relative paths).
- Multi-value returns to avoid API reshaping when maps are immutable.

**Syntax Quality Rating:** 7/10

**AI Generation Experience**
- Integration tests were simple once components exported functions.
- Import ergonomics and type sharing are still underdocumented.
