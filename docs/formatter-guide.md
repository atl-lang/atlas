# Atlas Formatter Guide

**Version:** v0.2 | **Status:** Production Ready

The Atlas formatter (`atlas fmt`) automatically formats Atlas source code to a consistent, canonical style.

---

## Overview

The Atlas formatter:
- Enforces consistent indentation (4 spaces by default)
- Normalizes whitespace around operators and punctuation
- Standardizes brace placement
- Sorts imports (when applicable)
- Is idempotent — formatting twice produces the same output

---

## Usage

### Format a Single File

```bash
atlas fmt main.atl          # format and overwrite in place
atlas fmt main.atl --check  # check if file is formatted (exit 1 if not)
atlas fmt main.atl --dry-run  # print formatted output without modifying
```

### Format Multiple Files

```bash
atlas fmt src/              # format all .atl files recursively
atlas fmt *.atl             # format all .atl files in current directory
atlas fmt --all             # format entire project (uses atlas.toml to find files)
```

### Check Mode (for CI)

```bash
atlas fmt --check           # exit 0 = formatted, exit 1 = needs formatting
atlas fmt --check src/      # check directory
```

---

## Formatting Rules

### Indentation

4 spaces per level (no tabs):

```atlas
// Before
fn greet(name: string) -> string {
  return concat("Hello, ", name);
}

// After
fn greet(name: string) -> string {
    return concat("Hello, ", name);
}
```

### Brace Placement

Opening braces on the same line (K&R style):

```atlas
// Before
fn add(a: number, b: number) -> number
{
    return a + b;
}

// After
fn add(a: number, b: number) -> number {
    return a + b;
}
```

### Operator Spacing

Single space around binary operators:

```atlas
// Before
let x=1+2;
let y = a*b + c*d;

// After
let x = 1 + 2;
let y = a * b + c * d;
```

### Function Call Spacing

No space between function name and arguments:

```atlas
// Before
print ("hello");
len (arr);

// After
print("hello");
len(arr);
```

### Array and Object Literals

```atlas
// Short arrays: single line
let nums = [1, 2, 3];

// Long arrays: multi-line with trailing comma
let names = [
    "Alice",
    "Bob",
    "Charlie",
];

// Short objects: single line
let point = { x: 1, y: 2 };

// Long objects: multi-line
let config = {
    host: "localhost",
    port: 8080,
    debug: true,
};
```

### if/else Formatting

```atlas
// Before
if (x > 0) {
  print("positive");
}
else {
  print("non-positive");
}

// After
if x > 0 {
    print("positive");
} else {
    print("non-positive");
}
```

Note: The formatter removes redundant parentheses around `if` conditions.

### Trailing Commas

The formatter adds trailing commas in multi-line collections:

```atlas
let config = {
    host: "localhost",
    port: 8080,    // trailing comma added
};
```

### Blank Lines

- One blank line between top-level declarations
- Two blank lines before function definitions (except first)
- No trailing blank lines in blocks

---

## Configuration

Configure the formatter in `atlas.toml`:

```toml
[formatter]
indent_width = 4           # spaces per indent level (default: 4)
max_line_width = 100       # soft line width limit (default: 100)
trailing_commas = true     # add trailing commas in multi-line (default: true)
quote_style = "double"     # "double" or "single" (default: "double")
```

---

## Editor Integration

### VS Code

Install the Atlas extension. The formatter runs automatically on save when configured:

```json
// .vscode/settings.json
{
    "[atlas]": {
        "editor.formatOnSave": true,
        "editor.defaultFormatter": "atl-lang.atlas"
    }
}
```

### Neovim (conform.nvim)

```lua
require("conform").setup({
    formatters_by_ft = {
        atlas = { "atlas_fmt" },
    },
    format_on_save = true,
})

require("conform").formatters.atlas_fmt = {
    command = "atlas",
    args = { "fmt", "--dry-run", "$FILENAME" },
    stdin = false,
}
```

### Emacs

```elisp
(defun atlas-format-buffer ()
  "Format current Atlas buffer using atlas fmt."
  (interactive)
  (when (eq major-mode 'atlas-mode)
    (shell-command-to-string
     (format "atlas fmt %s" (buffer-file-name)))
    (revert-buffer t t)))

(add-hook 'atlas-mode-hook
          (lambda ()
            (add-hook 'before-save-hook #'atlas-format-buffer nil t)))
```

---

## CI Integration

Add a formatting check to your CI pipeline:

```yaml
# GitHub Actions
- name: Check formatting
  run: atlas fmt --check src/
```

```bash
# Pre-commit hook (.git/hooks/pre-commit)
#!/bin/bash
atlas fmt --check $(git diff --cached --name-only --diff-filter=ACM | grep '\.atl$')
if [ $? -ne 0 ]; then
    echo "Format check failed. Run 'atlas fmt' to fix."
    exit 1
fi
```

---

## Formatting Behavior with Malformed Code

The formatter requires syntactically valid Atlas code. If the code has parse errors:

```bash
$ atlas fmt broken.atl
error: cannot format file with syntax errors
  --> broken.atl:5:10
  |
5 |     let x = ;
  |             ^ expected expression
```

Fix syntax errors before formatting.

---

## Disabling Formatting

Use `// fmt: off` and `// fmt: on` to disable formatting for a region:

```atlas
// fmt: off
let matrix = [
    1, 0, 0,
    0, 1, 0,
    0, 0, 1,
];
// fmt: on
```

---

## Examples: Before and After

### Complex Function

```atlas
// Before (messy formatting)
fn calculate_statistics(data:array)->object{
let sum=0;
for n in data{sum=sum+n;}
let mean=sum/len(data);
let variance=0;
for n in data{
let diff=n-mean;
variance=variance+(diff*diff);}
variance=variance/len(data);
return {mean:mean,variance:variance,std_dev:sqrt(variance)};}

// After (formatter applied)
fn calculate_statistics(data: array) -> object {
    let sum = 0;
    for n in data {
        sum = sum + n;
    }
    let mean = sum / len(data);
    let variance = 0;
    for n in data {
        let diff = n - mean;
        variance = variance + (diff * diff);
    }
    variance = variance / len(data);
    return {
        mean: mean,
        variance: variance,
        std_dev: sqrt(variance),
    };
}
```

---

*See also: [CLI Reference](cli-reference.md) | [Editor Setup](editor-setup/vscode.md)*
