# Component Audit: Watcher

**Grammar Friction Points**
- No recursive directory traversal utilities; `readDir` returns filenames only.
- No file watch API; implementation devolves to snapshot diffing.
- `hashMapPut` is not mutating; snapshot updates must reassign every time.
- `if` requires parentheses; missing parens are a frequent AI mistake.

**Missing Features (Atlas should have)**
- Native filesystem watcher with debounce and ignore support.
- `readDirRecursive` and globbing utilities.
- Path helpers beyond `pathJoin`/`pathBasename` (e.g., `pathRel`).

**Syntax Quality Rating:** 5/10

**AI Generation Experience**
- The lack of watchers forces a different architecture; this is a functional gap, not syntax.
- Even a simple poller needs extensive glue code and IO adapters.
