# Atlas Language Server (atlas-lsp)

The Atlas LSP server provides IDE integration for `.atlas` files over the Language Server Protocol.
It is implemented in the `atlas-lsp` crate using `tower-lsp` and runs as a standalone process
communicating over stdio.

## Capabilities

The server advertises the following capabilities on initialization:

| Capability | Detail |
|---|---|
| Text document sync | Full (entire document re-sent on each change) |
| Diagnostics | Per-document, pushed on open/change; identifier: `"atlas"` |
| Document symbols | Nested hierarchy |
| Workspace symbols | Up to 100 results per query |
| Hover | Markdown, with identifier range highlight |
| Go-to-definition | Single location (scalar response) |
| Find references | Cross-document via symbol index |
| Completion | Triggered by `.`; context-sensitive |
| Document formatting | Full and range |
| Code actions | Quick-fix, refactor, extract, inline, rewrite, source, organize-imports |
| Semantic tokens | Full and range |
| Inlay hints | Type hints and parameter hints |
| Call hierarchy | Prepare, incoming calls, outgoing calls |
| Folding ranges | Block-level folding |

---

## Completion

Trigger character: `.`

`generate_completions(text, position, ast, symbols)` is the entry point. It assembles completion
items from several context-sensitive sources:

### Always included

**Keywords** — snippets with tab-stop placeholders:

| Label | Insert text |
|---|---|
| `let` | `let ${1:name}: ${2:type} = ${3:value};` |
| `let mut` | `let mut ${1:name}: ${2:type} = ${3:value};` |
| `fn` | `fn ${1:name}(${2:params}) -> ${3:type} {\n\t${4}\n}` |
| `if` | `if (${1:condition}) {\n\t${2}\n}` |
| `while` | `while (${1:condition}) {\n\t${2}\n}` |
| `for` | `for (${1:init}; ${2:condition}; ${3:update}) {\n\t${4}\n}` |
| `return` | `return ${1:value};` |
| `break` | (plain) |
| `continue` | (plain) |
| `true`, `false`, `null` | (plain) |

**Primitive type keywords**: `number`, `string`, `bool`

**Built-in functions**: `print`, `len`, `push`, `pop` — with signature detail and snippet.

### Context-sensitive

**Ownership annotations** — only when cursor is inside a function parameter list (detected by
walking backwards to find `fn name(`):

| Label | Insert text | Semantics |
|---|---|---|
| `own` | `own ${1:name}: ${2:Type}` | Move: caller's binding is invalidated after call |
| `borrow` | `borrow ${1:name}: ${2:Type}` | Immutable borrow: caller retains ownership |
| `shared` | `shared ${1:name}: ${2:Type}` | Shared reference: Arc<T> semantics |

**Trait names after `impl`** — when text immediately before cursor ends with `impl`:
- Built-in traits: `Copy`, `Move`, `Drop`, `Display`, `Debug` (with documentation strings)
- User-defined traits from the current document's AST

**Method stubs inside impl body** — when cursor is after `fn` inside an `impl TraitName for`
block, completion suggests required method signatures as snippets, one per trait method.

**Document symbols**:
- Top-level functions with full signature detail: `fn name(params) -> ReturnType`
- Top-level variable declarations with type

---

## Hover

`generate_hover(text, position, ast, symbols)` returns Markdown with the identifier range
highlighted. Hover content is formatted as fenced `atlas` code blocks.

### What gets shown

| Hover target | Content |
|---|---|
| Function declaration | Doc comment (if present, from `///` lines) + `fn name(params) -> return_type` |
| Variable declaration | `let [mut] name: Type` — explicit type, or inferred anonymous function type, or type from symbol table |
| Type alias | `type Name = ...` |
| Function parameter | `(own\|borrow\|share parameter) name: Type` |
| Trait declaration | `**(trait)** Name` + method signatures in code block |
| `impl` block | `**(impl)** TypeName implements TraitName` + method signatures |
| Symbol table entry | Variable, function, parameter, const, builtin — with resolved type |
| Built-in function | Full signature + description + `*builtin function*` marker |
| Keyword | One-line description |

**Doc comment extraction**: `///` lines immediately preceding the definition are collected and
prepended to the hover block. Empty lines between doc comments and the definition stop collection.

**Inferred types**: When a function has no explicit return type annotation, the hover falls back
to the symbol table's inferred return type (skipping `unknown`, `?`, `void`). Same for variable
types.

### Built-in functions with hover

All standard library built-ins have full hover signatures:

- I/O: `print`, `println`, `input`
- Type conversion: `string`, `number`, `bool`, `int`
- Type checking: `typeof`, `is_number`, `is_string`, `is_bool`, `is_null`, `is_array`, `is_function`
- Array: `len`, `push`, `pop`, `shift`, `unshift`, `slice`, `concat`, `reverse`, `sort`, `map`, `filter`, `reduce`, `find`, `every`, `some`, `includes`, `index_of`, `join`, `flat`
- String: `split`, `trim`, `to_upper`, `to_lower`, `starts_with`, `ends_with`, `contains`, `replace`, `char_at`, `substring`, `pad_start`, `pad_end`, `repeat`
- Math: `abs`, `floor`, `ceil`, `round`, `sqrt`, `pow`, `min`, `max`, `sin`, `cos`, `tan`, `log`, `log10`, `exp`, `random`, `random_range`
- HashMap: `keys`, `values`, `entries`, `has_key`, `remove`
- Assertions: `assert`, `assert_eq`, `assert_ne`
- Time: `time`, `sleep`
- Error handling: `error`, `try_catch`

---

## Inlay Hints

`generate_inlay_hints(text, range, ast, symbols, config)` produces two kinds of hints.

### Type hints

Shown after variable names that have **no explicit type annotation**, when the initializer
is not an obvious type (literal or array literal). Rendered as `: Type` inline after the
variable name. Kind: `TYPE`.

For functions with **no explicit return type annotation**, a `→ ReturnType` hint appears
at the opening brace, using `infer_return_type` on the function body directly.

Type strings are truncated to 25 characters by default (shown as `Type...`); full type
appears in the tooltip.

### Parameter name hints

Shown before function call arguments. Rendered as `paramName:` before the argument.
Kind: `PARAMETER`. Skipped when:
- The argument is an identifier with the same name as the parameter (case-insensitive)
- The argument is a literal

### Configuration (`InlayHintConfig`)

| Field | Default | Description |
|---|---|---|
| `show_type_hints` | `true` | Show inferred types on unannotated variables |
| `show_parameter_hints` | `true` | Show parameter names at call sites |
| `show_inferred_return` | `true` | Show inferred return type on unannotated functions |
| `max_type_length` | `25` | Maximum characters before truncating type string |
| `skip_obvious_types` | `true` | Skip hints when type is obvious from initializer |

---

## Go-to-Definition

`find_definition(symbol_index, uri, text, position)` resolves in two steps:

1. Checks `SymbolIndex::find_definition_at(uri, position)` — position-keyed lookup
2. Falls back to text-based identifier extraction, then `find_definitions(name)` — prefers
   same-file definitions, falls back to any file in the workspace

---

## Find References

`find_all_references(uri, text, position, ast, symbols, symbol_index, include_declaration)`
walks the full AST collecting all identifier usages by name. Cross-document references are
resolved via the workspace `SymbolIndex`.

---

## Semantic Tokens

Full and range semantic token responses. Tokens are classified by the lexer output plus AST
context.

### Token types (22 total)

`namespace`, `type`, `class`, `enum`, `interface`, `struct`, `typeParameter`, `parameter`,
`variable`, `property`, `enumMember`, `event`, `function`, `method`, `macro`, `keyword`,
`modifier`, `comment`, `string`, `number`, `regexp`, `operator`

### Token modifiers (10 total)

`declaration`, `definition`, `readonly`, `static`, `deprecated`, `abstract`, `async`,
`modification`, `documentation`, `defaultLibrary`

### Classification rules

| Token | Assigned type | Modifiers |
|---|---|---|
| Keywords (`let`, `fn`, `if`, `while`, `for`, `return`, etc.) | `keyword` | — |
| `own`, `borrow`, `share`, `trait`, `impl`, `async`, `await`, `struct`, `enum` | `keyword` | — |
| `true`, `false`, `null` | `keyword` | — |
| Number literals | `number` | — |
| String literals, template strings | `string` | — |
| `// line comments`, `/* block comments */` | `comment` | — |
| `///` doc comments | `comment` | `documentation` |
| Operators | `operator` | — |
| Built-in function identifiers | `function` | `defaultLibrary` |
| Function parameters | `parameter` | — |
| Symbol-table functions | `function` | `declaration` |
| Symbol-table immutable variables | `variable` | `readonly` |
| Symbol-table mutable variables | `variable` | — |
| Symbol-table constants | `variable` | `readonly` |
| PascalCase identifiers (heuristic) | `type` | — |
| Other identifiers | `variable` | — |
| Punctuation (parens, braces, brackets, etc.) | (skipped) | — |

---

## Document and Workspace Symbols

**Document symbols** (`textDocument/documentSymbol`): Nested response. Extracts functions,
top-level variables, and type aliases from the AST with their LSP `SymbolKind`.

**Workspace symbols** (`workspace/symbol`): Queries the `WorkspaceIndex` with a text query,
returning up to 100 matching symbols across all open documents.

---

## Code Actions

Available code action kinds: `quickfix`, `refactor`, `refactor.extract`, `refactor.inline`,
`refactor.rewrite`, `source`, `source.organizeImports`.

Actions are generated by `generate_code_actions(uri, range, context, text, ast, symbols,
diagnostics)` based on current diagnostics and selection range.

---

## Folding Ranges

`folding_range` response provides block-level folding for functions, control flow, and other
brace-delimited constructs.

---

## Call Hierarchy

`prepare_call_hierarchy`, `incoming_calls`, and `outgoing_calls` are all implemented.
Incoming and outgoing call resolution is cross-document: the server collects all open
document texts and ASTs before querying the `SymbolIndex`.

---

## Editor Setup

### VS Code

Install the Atlas extension (not yet published; connect manually for now):

1. Add to `settings.json`:

```json
{
  "atlas.lsp.serverPath": "/path/to/atlas-lsp"
}
```

2. Or use the generic `vscode-languageclient` with:

```json
{
  "serverOptions": {
    "command": "/path/to/target/release/atlas-lsp",
    "args": [],
    "transport": "stdio"
  },
  "clientOptions": {
    "documentSelector": [{ "scheme": "file", "language": "atlas" }]
  }
}
```

### Neovim (nvim-lspconfig)

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.atlas then
  configs.atlas = {
    default_config = {
      cmd = { '/path/to/target/release/atlas-lsp' },
      filetypes = { 'atlas' },
      root_dir = lspconfig.util.root_pattern('atlas.toml', '.git'),
      settings = {},
    },
  }
end

lspconfig.atlas.setup {}
```

Add filetype detection:

```lua
vim.filetype.add({ extension = { atlas = 'atlas' } })
```

### Helix

In `languages.toml`:

```toml
[[language]]
name = "atlas"
scope = "source.atlas"
file-types = ["atlas"]
language-servers = ["atlas-lsp"]

[language-server.atlas-lsp]
command = "/path/to/target/release/atlas-lsp"
```

---

## Internal Architecture

```
AtlasLspServer
├── documents: HashMap<Url, DocumentState>   # per-file: text, AST, diagnostics
├── workspace_index: WorkspaceIndex          # cross-file symbol search
├── symbol_index: SymbolIndex                # definition + reference tracking
└── inlay_config: InlayHintConfig            # hint display settings
```

Document state is updated on `textDocument/didOpen` and `textDocument/didChange` (full sync).
AST re-parse and workspace index update happen synchronously within the same lock acquisition.
Diagnostics are published to the client immediately after each update.

The server is built with `tower-lsp`. All LSP handlers are `async`. Document access uses
`Arc<Mutex<HashMap<Url, DocumentState>>>`.
