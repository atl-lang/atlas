# Atlas Formatter (atlas-formatter)

The Atlas formatter (`atlas fmt`) produces canonical formatted output from Atlas source code.
It is implemented in the `atlas-formatter` crate and integrated into the LSP server for
on-save and on-demand formatting.

---

## Running the Formatter

```bash
# Format files in place
atlas fmt

# Format a specific file
atlas fmt src/main.atlas

# Check if files are already formatted (exits non-zero if not)
atlas fmt --check

# Format from stdin
atlas fmt -
```

The formatter is also invoked by the LSP server in response to `textDocument/formatting` and
`textDocument/rangeFormatting` requests from the editor.

---

## Format Pipeline

The formatter follows a strict pipeline — it never calls the VM:

```
Source
  └─> Lexer (with comments)
        └─> CommentCollector          # preserves all comments by span
              └─> Lexer (clean)
                    └─> Parser
                          └─> AST
                                └─> FormatVisitor    # walks AST, re-emits text
                                      └─> Formatted output
```

1. The source is tokenized twice: once with comments (for `CommentCollector`) and once without
   (for the parser, which does not accept comment tokens).
2. `CommentCollector` captures every `//`, `/* */`, and `///` comment with its span.
3. The parser produces an AST from the clean token stream.
4. `FormatVisitor` walks the AST and re-emits formatted text, inserting collected comments at
   the correct positions based on span proximity.
5. The output is guaranteed to end with exactly one newline.

---

## Formatting Rules

### Indentation

- 4 spaces per level (default)
- No tabs
- Block bodies (function bodies, if/else branches, loop bodies, trait/impl blocks) are indented
  one level deeper than their containing construct

### Line length

- Maximum 100 characters per line (default)
- The formatter breaks multi-line constructs when they exceed this limit

### Semicolons

- Semicolons are always inserted after statements
- No ASI (automatic semicolon insertion) — every statement gets an explicit `;`

### Trailing commas

- Trailing commas are added in multi-line constructs (default: `true`)
- Single-line constructs do not have trailing commas

### Spacing

- Single space after keywords: `if (`, `while (`, `for (`, `fn name(`
- Single space around binary operators: `a + b`, `x = y`
- No space before `:` in type annotations, single space after: `name: Type`
- No space before `;`
- Function parameters: `fn name(param: Type, param2: Type)`
- Function call arguments: `foo(a, b, c)`
- Arrow in return types: ` -> Type`

### Comments

All comment types are preserved:

| Comment style | Preservation |
|---|---|
| `// line comment` | Preserved on same line or as preceding line |
| `/* block comment */` | Preserved inline |
| `/// doc comment` | Preserved immediately before declaration |

Doc comments (`///`) are kept attached to the declaration they precede. The formatter does
not strip, reorder, or reformat comment content.

### Idempotency

The formatter is idempotent: `format(format(x)) == format(x)`. Running `atlas fmt` twice
on the same file produces no further changes.

---

## Configuration

The formatter is configured via `FormatConfig`. Configuration can be set in `atlas.toml`
under a `[fmt]` section (when that support is wired up in the CLI) or passed directly
via the Rust API.

| Field | Type | Default | Description |
|---|---|---|---|
| `indent_size` | integer | `4` | Spaces per indentation level |
| `max_width` | integer | `100` | Maximum line width before breaking |
| `trailing_commas` | bool | `true` | Add trailing commas in multi-line constructs |
| `semicolon_style` | enum | `"always"` | Semicolon style — currently only `"always"` is supported |

### atlas.toml (future)

```toml
[fmt]
indent_size = 2
max_width = 80
trailing_commas = true
```

### Builder API (Rust)

```rust
use atlas_formatter::FormatConfig;

let config = FormatConfig::default()
    .with_indent_size(2)
    .with_max_width(80)
    .with_trailing_commas(false);
```

---

## Rust API

```rust
use atlas_formatter::{format_source, format_source_with_config, check_formatted, FormatConfig, FormatResult};

// Format with defaults
match format_source(source) {
    FormatResult::Ok(formatted) => println!("{}", formatted),
    FormatResult::ParseError(errors) => eprintln!("Parse errors: {:?}", errors),
}

// Format with custom config
let config = FormatConfig::default().with_indent_size(2);
let result = format_source_with_config(source, &config);

// Check without modifying (for CI)
if !check_formatted(source) {
    eprintln!("File is not formatted");
    std::process::exit(1);
}
```

### `FormatResult`

| Variant | Description |
|---|---|
| `Ok(String)` | Successfully formatted source code |
| `ParseError(Vec<String>)` | Source has parse errors; original is unchanged |

The formatter never modifies source with parse errors — it returns the error messages instead.

---

## LSP Integration

The LSP server delegates `textDocument/formatting` and `textDocument/rangeFormatting` to the
formatter:

- **Full document formatting**: calls `format_document(text)` which uses default config
- **Range formatting**: calls `format_range(text, range)` which formats only the selected range

The LSP server returns the result as a `Vec<TextEdit>` — a single edit replacing the entire
document (or range) with the formatted output.

---

## Examples

### Before

```atlas
fn greet(   name:string,   count:number ) ->string{
let msg="Hello "+name;
for(let i=0;i<count;i++){
print(msg)
}
return msg
}
```

### After `atlas fmt`

```atlas
fn greet(name: string, count: number) -> string {
    let msg = "Hello " + name;
    for (let i = 0; i < count; i++) {
        print(msg);
    }
    return msg;
}
```

---

## Exit Codes

| Exit code | Meaning |
|---|---|
| `0` | Success (or `--check` found all files formatted) |
| `1` | `--check` found unformatted files |
| `2` | Parse error in one or more files |
