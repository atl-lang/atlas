# atlas-formatter/src/

Atlas source code formatter. Parses Atlas source, walks the AST via a visitor, and
re-emits canonical formatted output with comment preservation.

## Files

| File | Role |
|------|------|
| `lib.rs` | Public API: `format_source()`, `format_source_with_config()`, `check_formatted()` |
| `formatter.rs` | `Formatter` struct, `FormatConfig` (indent_size, max_width, trailing_commas, semicolon_style), `FormatResult` |
| `visitor.rs` | `FormatVisitor` — AST walker that produces formatted output; handles all `Stmt`/`Expr`/`Item` variants |
| `comments.rs` | `CommentCollector`, `Comment`, `CommentKind`, `CommentPosition` — preserves comments from source |

## Key Types

- `FormatConfig` — defaults: indent=4, max_width=100, trailing_commas=true, semicolon=Always
- `FormatResult` — wraps formatted `String` or parse error
- `FormatVisitor` — stateful visitor: `output: String`, `indent_level`, `comments: Vec<Comment>`

## Patterns

- Comments are collected from lexer output BEFORE formatting, then re-inserted by `FormatVisitor`
  during the walk based on span proximity.
- `FormatVisitor::into_output()` ensures file ends with exactly one newline.
- Format pipeline: `source → Lexer → Parser → CommentCollector → FormatVisitor → String`

## Tests

Tests live in `crates/atlas-formatter/tests/` (separate crate test directory).
No inline `#[cfg(test)]` blocks in `src/`.

## Critical Rules

- **No behavior changes.** The formatter must round-trip: `format(format(x)) == format(x)`.
- **Comment preservation is mandatory.** Stripping comments = formatter bug.
- Never call into the runtime interpreter or VM — formatter depends only on `atlas-runtime` for AST types.
