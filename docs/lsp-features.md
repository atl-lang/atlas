# Atlas LSP Features

**Version:** v0.2 | **Status:** Production Ready

The Atlas Language Server provides comprehensive IDE support for the Atlas programming language through the Language Server Protocol (LSP).

---

## Overview

Atlas LSP implements the Language Server Protocol to provide rich language features in any LSP-compatible editor, including VS Code, Neovim, Emacs, and others.

**Protocol Version:** LSP 3.17
**Transport:** stdio (default), TCP (optional)

---

## Implemented Features

### 1. Hover Information

**Capability:** `hoverProvider`
**Method:** `textDocument/hover`

Displays type information, documentation, and signature details when hovering over code elements.

**What you get:**
- Function signatures with parameter types and return types
- Variable types
- Inline documentation from comments
- Builtin function documentation

**Example:**
```atlas
fn calculate(x: number) -> number { return x * 2; }
//  ^hover here shows: fn calculate(x: number) -> number
```

**Performance:** < 100ms response time

---

### 2. Semantic Syntax Highlighting

**Capability:** `semanticTokensProvider`
**Method:** `textDocument/semanticTokens/full`

Provides accurate, type-aware syntax highlighting that understands Atlas semantics.

**Token types:**
- `function` — function definitions and calls
- `variable` — local and global variables
- `parameter` — function parameters
- `type` — type annotations
- `keyword` — language keywords
- `string` — string literals
- `number` — numeric literals
- `operator` — operators and punctuation
- `comment` — inline and block comments

---

### 3. Code Actions

**Capability:** `codeActionProvider`
**Method:** `textDocument/codeAction`

Offers quick-fix suggestions and refactoring actions for code issues.

**Available actions:**
- Add missing return type annotation
- Add missing type annotation to variable
- Convert `let` to `var` (or vice versa)
- Extract expression to variable
- Add `return` keyword
- Fix import path

---

### 4. Diagnostics (Real-time Error Reporting)

**Capability:** `diagnosticsProvider`
**Method:** `textDocument/publishDiagnostics`

Reports errors and warnings in real-time as you type.

**Diagnostic types:**
- Parse errors (syntax errors)
- Type errors (type mismatches, unknown types)
- Binding errors (undefined variables, duplicate declarations)
- Warning: unused variables
- Warning: unreachable code

**Error codes:** All diagnostics include error codes (e.g., `E0012`) for documentation lookup.

---

### 5. Document Symbols

**Capability:** `documentSymbolProvider`
**Method:** `textDocument/documentSymbol`

Lists all symbols (functions, variables, types) in the current file for quick navigation.

**Supports:**
- Function definitions with parameter lists
- Variable declarations
- Hierarchical symbol display

---

### 6. Workspace Symbols

**Capability:** `workspaceSymbolProvider`
**Method:** `workspace/symbol`

Search for symbols across all Atlas files in the workspace.

**Usage:** Press the "Search Symbol" shortcut in your editor and type to filter.

**Performance:** Workspace index is built incrementally; search is < 50ms.

---

### 7. Completion (Intellisense)

**Capability:** `completionProvider`
**Method:** `textDocument/completion`

Provides context-aware code completion suggestions.

**What gets completed:**
- Variables in scope
- Function names
- Object field access (`obj.`)
- Stdlib function names
- Type names
- Keywords

**Trigger characters:** `.` (member access)

---

### 8. Signature Help

**Capability:** `signatureHelpProvider`
**Method:** `textDocument/signatureHelp`

Shows function signatures while you type arguments.

**Trigger characters:** `(`, `,`

**Shows:**
- Full function signature
- Parameter types
- Current parameter highlighted
- Documentation if available

---

### 9. Go to Definition

**Capability:** `definitionProvider`
**Method:** `textDocument/definition`

Navigate to where a symbol is defined.

**Supports:**
- Functions
- Variables
- Parameters
- Imported symbols

**Usage:** Ctrl+Click or F12 (varies by editor)

---

### 10. Find All References

**Capability:** `referencesProvider`
**Method:** `textDocument/references`

Find all usages of a symbol in the workspace.

**Supports:**
- Functions (all call sites)
- Variables (all read/write sites)
- Parameters

**Performance:** < 200ms for workspace-wide search.

---

### 11. Document Formatting

**Capability:** `documentFormattingProvider`
**Method:** `textDocument/formatting`

Format the entire document using the Atlas formatter.

**Usage:** Format Document shortcut in your editor

---

### 12. Range Formatting

**Capability:** `documentRangeFormattingProvider`
**Method:** `textDocument/rangeFormatting`

Format a selected region of code.

---

### 13. Code Folding

**Capability:** `foldingRangeProvider`
**Method:** `textDocument/foldingRange`

Enables collapsing of code regions in editors.

**Foldable regions:**
- Function bodies
- `if`/`else` blocks
- `for`/`while` loops
- Object literals
- Array literals

---

### 14. Inlay Hints

**Capability:** `inlayHintProvider`
**Method:** `textDocument/inlayHint`

Shows inferred type annotations inline in the editor.

**Example:**
```atlas
let x = 42;        //   shows: : number
let y = "hello";   //   shows: : string
```

---

### 15. Call Hierarchy

**Capability:** `callHierarchyProvider`
**Method:** `textDocument/prepareCallHierarchy`
         `callHierarchy/incomingCalls`
         `callHierarchy/outgoingCalls`

Navigate the call graph: see what calls a function, and what a function calls.

**Incoming calls:** "Who calls this function?"
**Outgoing calls:** "What does this function call?"

**Usage:** Right-click a function → "Show Call Hierarchy"

---

### 16. Rename Symbol

**Capability:** `renameProvider`
**Method:** `textDocument/rename`

Rename a symbol and update all references workspace-wide.

**Supports:**
- Functions
- Variables
- Parameters

**Usage:** Right-click → Rename Symbol, or F2 (varies by editor)

---

## Performance Characteristics

| Feature | Response Time |
|---------|--------------|
| Hover | < 100ms |
| Diagnostics | < 200ms after edit |
| Completion | < 50ms |
| Signature help | < 20ms |
| Go to definition | < 50ms |
| Find references | < 200ms (workspace) |
| Workspace symbols | < 50ms |
| Formatting | < 500ms (large files) |

---

## Editor Setup

See the editor-specific setup guides:

- [VS Code Setup](editor-setup/vscode.md)
- [Neovim Setup](editor-setup/neovim.md)
- [Emacs Setup](editor-setup/emacs.md)

---

## Troubleshooting

See [LSP Troubleshooting](lsp-troubleshooting.md) for common issues and solutions.
