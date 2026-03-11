//! Comprehensive error code registry — all AT/AW diagnostic descriptors.
//!
//! Every `pub const` is a `DiagnosticDescriptor` with code, level, title,
//! message_template, static_help, static_note, and domain fully populated.
//!
//! ## Code ranges
//! - AT0xxx: Runtime errors (type, undefined, bounds, etc.)
//! - AT01xx: Stdlib errors
//! - AT03xx: Permission errors
//! - AT04xx: I/O errors
//! - AT1xxx: Syntax/lexer errors
//! - AT2xxx: Warnings (unused, unreachable, etc.) — level: Warning
//! - AT3xxx: Semantic/type checking errors
//! - AT4xxx: Async/await errors
//! - AT5xxx: Module system errors
//! - AT9xxx: Internal errors
//! - AW3xxx/AW9xxx: Runtime/internal warnings

use crate::diagnostic::{
    descriptor::{DiagnosticDescriptor, DiagnosticDomain},
    DiagnosticLevel,
};

// ── AT0xxx: Runtime Errors ─────────────────────────────────────────────────────

pub const TYPE_MISMATCH: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0001",
    level: DiagnosticLevel::Error,
    title: "Type mismatch",
    message_template: "type mismatch: expected {expected}, found {found}",
    static_help: Some("ensure the value type matches the expected type; use explicit conversion functions if needed"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const UNDEFINED_SYMBOL: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0002",
    level: DiagnosticLevel::Error,
    title: "Undefined symbol",
    message_template: "`{name}` is not defined",
    static_help: Some(
        "check the spelling and make sure the variable or function is declared before use",
    ),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const INVALID_ARITY: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0003",
    level: DiagnosticLevel::Error,
    title: "Arity mismatch",
    message_template: "function expects {expected} argument(s), found {found}",
    static_help: Some("check the function signature for the correct number of arguments"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const INVALID_OPERATION: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0004",
    level: DiagnosticLevel::Error,
    title: "Invalid operation",
    message_template: "invalid operation: {detail}",
    static_help: Some("this operation is not supported for the given type(s)"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const DIVIDE_BY_ZERO: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0005",
    level: DiagnosticLevel::Error,
    title: "Division by zero",
    message_template: "division by zero",
    static_help: Some("check that the divisor is not zero before dividing"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const ARRAY_OUT_OF_BOUNDS: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0006",
    level: DiagnosticLevel::Error,
    title: "Array index out of bounds",
    message_template: "index {index} is out of bounds for array of length {length}",
    static_help: Some("check the array length with `len()` before accessing elements"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const INVALID_NUMERIC_RESULT: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0007",
    level: DiagnosticLevel::Error,
    title: "Invalid numeric result",
    message_template: "numeric operation produced an invalid result ({detail})",
    static_help: Some("ensure inputs to math operations are finite and in range"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

// ── AT01xx: Stdlib Errors ──────────────────────────────────────────────────────

pub const STDLIB_ARG_ERROR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0102",
    level: DiagnosticLevel::Error,
    title: "Invalid stdlib argument",
    message_template: "invalid argument for `{function}`: {detail}",
    static_help: Some("check the function documentation for valid argument types and ranges"),
    static_note: None,
    domain: DiagnosticDomain::Stdlib,
};

pub const STDLIB_VALUE_ERROR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0103",
    level: DiagnosticLevel::Error,
    title: "Invalid stdlib value",
    message_template: "invalid value for `{function}`: {detail}",
    static_help: Some("the provided value is outside the expected range or type for this function"),
    static_note: None,
    domain: DiagnosticDomain::Stdlib,
};

pub const UNHASHABLE_TYPE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0140",
    level: DiagnosticLevel::Error,
    title: "Unhashable type",
    message_template: "type `{type_name}` cannot be used as a HashMap key",
    static_help: Some("only `number`, `string`, `bool`, and `null` are hashable — convert your value to one of these types first"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

// ── AT03xx: Permission Errors ──────────────────────────────────────────────────

pub const FILESYSTEM_PERMISSION_DENIED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0300",
    level: DiagnosticLevel::Error,
    title: "Filesystem permission denied",
    message_template: "filesystem access denied: {detail}",
    static_help: Some(
        "enable file permissions with `--allow-file` or adjust the security settings",
    ),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const NETWORK_PERMISSION_DENIED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0301",
    level: DiagnosticLevel::Error,
    title: "Network permission denied",
    message_template: "network access denied: {detail}",
    static_help: Some(
        "enable network permissions with `--allow-network` or adjust the security settings",
    ),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const PROCESS_PERMISSION_DENIED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0302",
    level: DiagnosticLevel::Error,
    title: "Process permission denied",
    message_template: "process access denied: {detail}",
    static_help: Some(
        "enable process permissions with `--allow-process` or adjust the security settings",
    ),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const ENVIRONMENT_PERMISSION_DENIED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0303",
    level: DiagnosticLevel::Error,
    title: "Environment variable permission denied",
    message_template: "environment variable access denied: {detail}",
    static_help: Some(
        "enable environment permissions with `--allow-env` or adjust the security settings",
    ),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

/// New in B17: FFI/native call permission denied.
pub const FFI_PERMISSION_DENIED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0304",
    level: DiagnosticLevel::Error,
    title: "FFI permission denied",
    message_template: "native FFI call denied: {detail}",
    static_help: Some(
        "enable FFI/native call permissions with `--allow-ffi` or adjust the security settings",
    ),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

// ── AT04xx: I/O Errors ─────────────────────────────────────────────────────────

pub const IO_ERROR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0400",
    level: DiagnosticLevel::Error,
    title: "I/O error",
    message_template: "I/O error: {detail}",
    static_help: Some("check file paths, permissions, and that the file system is accessible"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

// ── AT05xx: Execution Limits ───────────────────────────────────────────────────

/// New in B17: execution timed out (watchdog or user-configured limit).
pub const EXECUTION_TIMEOUT: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0500",
    level: DiagnosticLevel::Error,
    title: "Execution timeout",
    message_template: "execution exceeded the time limit of {limit}",
    static_help: Some("reduce computation or raise the timeout limit with `--timeout <seconds>`"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

/// New in B17: heap allocation exceeded the configured memory limit.
pub const MEMORY_LIMIT_EXCEEDED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT0501",
    level: DiagnosticLevel::Error,
    title: "Memory limit exceeded",
    message_template: "heap allocation exceeded the memory limit of {limit}",
    static_help: Some("reduce allocations or raise the limit with `--memory-limit <bytes>`"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

// ── AT1xxx: Syntax / Lexer Errors ─────────────────────────────────────────────

pub const SYNTAX_ERROR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1000",
    level: DiagnosticLevel::Error,
    title: "Syntax error",
    message_template: "syntax error: {detail}",
    static_help: Some("check the syntax near the indicated location"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const UNEXPECTED_TOKEN: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1001",
    level: DiagnosticLevel::Error,
    title: "Unexpected token",
    message_template: "unexpected token `{token}`",
    static_help: Some("check for missing semicolons, brackets, or operators near this location"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const UNTERMINATED_STRING: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1002",
    level: DiagnosticLevel::Error,
    title: "Unterminated string literal",
    message_template: "unterminated string literal",
    static_help: Some("add the closing quote `\"` to complete the string"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const INVALID_ESCAPE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1003",
    level: DiagnosticLevel::Error,
    title: "Invalid escape sequence",
    message_template: "invalid escape sequence `{sequence}`",
    static_help: Some("valid escape sequences: `\\n`, `\\t`, `\\r`, `\\\\`, `\\\"`, `\\0`, `\\xHH` (hex), `\\uHHHH` (unicode)"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const UNTERMINATED_COMMENT: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1004",
    level: DiagnosticLevel::Error,
    title: "Unterminated block comment",
    message_template: "unterminated block comment",
    static_help: Some("add `*/` to close the block comment"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const INVALID_NUMBER: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1005",
    level: DiagnosticLevel::Error,
    title: "Invalid number literal",
    message_template: "invalid number literal `{literal}`",
    static_help: Some(
        "numbers must be valid decimal or floating-point literals, e.g. `42`, `3.14`",
    ),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const UNEXPECTED_EOF: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1006",
    level: DiagnosticLevel::Error,
    title: "Unexpected end of file",
    message_template: "unexpected end of file",
    static_help: Some(
        "the file ended unexpectedly — check for missing closing brackets, braces, or semicolons",
    ),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// Parse error: function parameter is missing an ownership annotation.
pub const MISSING_OWNERSHIP_ANNOTATION: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1007",
    level: DiagnosticLevel::Error,
    title: "Missing ownership annotation",
    message_template: "parameter `{name}` is missing an ownership annotation",
    static_help: Some("use `own`, `borrow`, or `share` before each parameter name — own=move, borrow=read-only, share=shared-ref"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// `echo` is not Atlas syntax. Use `print(expr)`.
pub const FOREIGN_SYNTAX_ECHO: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1008",
    level: DiagnosticLevel::Error,
    title: "Foreign syntax: `echo`",
    message_template: "`echo` is not valid Atlas syntax",
    static_help: Some("use `print(expr)` instead — example: `print(\"hello, world\")`"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// `var` is not Atlas syntax. Use `let` or `let mut`.
pub const FOREIGN_SYNTAX_VAR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1009",
    level: DiagnosticLevel::Error,
    title: "Foreign syntax: `var`",
    message_template: "`var` is not valid Atlas syntax",
    static_help: Some("use `let name = value` (immutable) or `let mut name = value` (mutable)"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// `function` keyword is not Atlas syntax. Use `fn`.
pub const FOREIGN_SYNTAX_FUNCTION_KW: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1010",
    level: DiagnosticLevel::Error,
    title: "Foreign syntax: `function`",
    message_template: "`function` is not valid Atlas syntax",
    static_help: Some("use `fn name(own param: Type) -> ReturnType { body }` — example: `fn add(own a: number, own b: number) -> number { a + b }`"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// `class` is not Atlas syntax. Use `struct`.
pub const FOREIGN_SYNTAX_CLASS: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1011",
    level: DiagnosticLevel::Error,
    title: "Foreign syntax: `class`",
    message_template: "`class` is not valid Atlas syntax",
    static_help: Some(
        "use `struct Name { field: Type }` — example: `struct Point { x: number, y: number }`",
    ),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const SHADOWING_PRELUDE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1012",
    level: DiagnosticLevel::Error,
    title: "Cannot shadow prelude builtin",
    message_template: "cannot shadow prelude builtin `{name}` at the top level",
    static_help: Some("prelude builtins cannot be redefined at top level — use a different name or shadow inside a nested scope"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// `console.log` is not Atlas syntax. Use `print(...)`.
pub const FOREIGN_SYNTAX_CONSOLE_LOG: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1013",
    level: DiagnosticLevel::Error,
    title: "Foreign syntax: `console.log`",
    message_template: "`console.log` is not valid Atlas syntax",
    static_help: Some("use `print(expr)` instead — example: `print(\"value: \" + str(x))`"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// `x++` / `x--` are not Atlas syntax.
pub const FOREIGN_SYNTAX_INCREMENT: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1014",
    level: DiagnosticLevel::Error,
    title: "Foreign syntax: `++` / `--`",
    message_template: "`{op}` increment/decrement operator is not valid Atlas syntax",
    static_help: Some(
        "use `x = x + 1` (increment) or `x = x - 1` (decrement) — or `x += 1` / `x -= 1`",
    ),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// `import X from` is not Atlas module syntax.
pub const FOREIGN_SYNTAX_IMPORT_FROM: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1015",
    level: DiagnosticLevel::Error,
    title: "Foreign syntax: `import X from`",
    message_template: "`import X from \"module\"` is not valid Atlas syntax",
    static_help: Some("use `import { name } from \"./module\"` — see docs/language/modules.md"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// Assignment target is an index expression containing a range.
pub const INVALID_ASSIGN_TARGET_RANGE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1016",
    level: DiagnosticLevel::Error,
    title: "Invalid assignment target: range index",
    message_template: "cannot assign to a range index expression",
    static_help: Some(
        "array slice assignments are not supported — assign to a specific index: `arr[0] = value`",
    ),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// Assignment to a method call result.
pub const INVALID_ASSIGN_TARGET_CALL: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1017",
    level: DiagnosticLevel::Error,
    title: "Invalid assignment target: method call result",
    message_template: "cannot assign to a method call result",
    static_help: Some("method call results are not addressable — assign to a variable first: `let mut result = obj.method();`"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// Assignment to a member of a non-addressable expression.
pub const INVALID_ASSIGN_TARGET_MEMBER: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1018",
    level: DiagnosticLevel::Error,
    title: "Invalid assignment target: member of non-addressable expression",
    message_template: "cannot assign to a member of a non-addressable expression",
    static_help: Some("valid targets: variables (`x = v`), array indices (`arr[i] = v`), struct fields on variables (`obj.field = v`)"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// Expression is not a valid assignment target.
pub const INVALID_ASSIGN_TARGET: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1019",
    level: DiagnosticLevel::Error,
    title: "Invalid assignment target",
    message_template: "expression is not a valid assignment target",
    static_help: Some("valid assignment targets: variables, array indices, and struct fields"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// Parse error: statement is missing a terminating semicolon.
pub const MISSING_SEMICOLON: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1020",
    level: DiagnosticLevel::Error,
    title: "Missing semicolon",
    message_template: "missing `;` after statement",
    static_help: Some("add `;` to terminate the statement"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// Parse error: missing closing delimiter (brace, bracket, or parenthesis).
pub const MISSING_CLOSING_DELIMITER: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1021",
    level: DiagnosticLevel::Error,
    title: "Missing closing delimiter",
    message_template: "missing closing `{delimiter}`",
    static_help: Some(
        "add the matching closing delimiter to complete the block, list, or expression",
    ),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// Parse error: reserved keyword used as an identifier.
pub const RESERVED_KEYWORD_AS_IDENTIFIER: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1022",
    level: DiagnosticLevel::Error,
    title: "Reserved keyword used as identifier",
    message_template: "`{keyword}` is a reserved keyword and cannot be used as an identifier",
    static_help: Some("choose a different name that does not conflict with an Atlas keyword"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

// ── AT2xxx: Warnings ───────────────────────────────────────────────────────────

pub const UNUSED_VARIABLE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2001",
    level: DiagnosticLevel::Warning,
    title: "Unused variable or parameter",
    message_template: "`{name}` is declared but never used",
    static_help: Some(
        "remove the unused binding or prefix the name with `_` to silence this warning: `_name`",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const UNREACHABLE_CODE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2002",
    level: DiagnosticLevel::Warning,
    title: "Unreachable code",
    message_template: "this code is unreachable",
    static_help: Some("remove this code or restructure the control flow"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const DUPLICATE_DECLARATION: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2003",
    level: DiagnosticLevel::Warning,
    title: "Duplicate declaration",
    message_template: "`{name}` is declared more than once",
    static_help: Some("remove the duplicate or rename one of the declarations"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const UNUSED_FUNCTION: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2004",
    level: DiagnosticLevel::Warning,
    title: "Unused function",
    message_template: "function `{name}` is never called",
    static_help: Some("remove the unused function or prefix the name with `_` to silence: `_name`"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const VARIABLE_SHADOWING: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2005",
    level: DiagnosticLevel::Warning,
    title: "Variable shadowing",
    message_template: "`{name}` shadows a variable from an outer scope",
    static_help: Some("use a different name if shadowing is unintentional"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const CONSTANT_CONDITION: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2006",
    level: DiagnosticLevel::Warning,
    title: "Constant condition",
    message_template: "condition is always `{value}`",
    static_help: Some("simplify the expression — this condition never changes"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const UNNECESSARY_ANNOTATION: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2007",
    level: DiagnosticLevel::Warning,
    title: "Unnecessary type annotation",
    message_template: "type annotation `{annotation}` is redundant — the type can be inferred",
    static_help: Some("consider removing the explicit type annotation"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const UNUSED_IMPORT: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2008",
    level: DiagnosticLevel::Warning,
    title: "Unused import",
    message_template: "import `{name}` is never used",
    static_help: Some("remove the unused import statement"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const DEPRECATED_TYPE_ALIAS: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2009",
    level: DiagnosticLevel::Warning,
    title: "Deprecated type alias",
    message_template: "type alias `{alias}` is deprecated",
    static_help: Some("use the recommended replacement type instead of this deprecated alias"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const OWN_ON_PRIMITIVE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2010",
    level: DiagnosticLevel::Warning,
    title: "`own` annotation on primitive type has no effect",
    message_template: "`own` on parameter `{name}: {type_name}` has no effect",
    static_help: Some("primitive types (`number`, `bool`, `string`) are always copied — the `own` annotation is ignored"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const BORROW_ON_SHARED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2011",
    level: DiagnosticLevel::Warning,
    title: "`borrow` annotation on `share<T>` type is redundant",
    message_template: "`borrow` on parameter `{name}: share<{inner}>` is redundant",
    static_help: Some("`share<T>` already has reference semantics — the `borrow` annotation has no additional effect"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const BORROW_TO_OWN: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2012",
    level: DiagnosticLevel::Warning,
    title: "Borrowed value passed to `own` parameter",
    message_template:
        "passing a borrowed value to `own` parameter `{name}` — ownership cannot transfer",
    static_help: Some(
        "a `borrow` parameter cannot give up ownership — pass an owned value instead",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Warning: a non-Copy type is passed without an ownership annotation.
pub const MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2013",
    level: DiagnosticLevel::Warning,
    title: "Non-Copy type passed without ownership annotation",
    message_template: "parameter `{name}` has a non-Copy type but no ownership annotation",
    static_help: Some("annotate the parameter with `own` or `borrow` to clarify ownership intent"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Warning: `var` keyword is deprecated.
pub const DEPRECATED_VAR_KEYWORD: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2014",
    level: DiagnosticLevel::Warning,
    title: "`var` keyword is deprecated",
    message_template: "the `var` keyword is deprecated",
    static_help: Some(
        "use `let mut` for mutable bindings — `var` will be removed in a future version",
    ),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

/// Warning: CoW mutation result discarded — collection unchanged.
/// Emitted when arrayPush/hashMapPut/etc. is called as a statement and the returned
/// new collection is not assigned back. Atlas collections are Copy-on-Write; mutation
/// methods do not modify the original — they return a new collection that must be
/// rebound: `arr = arr.push(x)`.
pub const DISCARDED_COW_RESULT: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2015",
    level: DiagnosticLevel::Warning,
    title: "CoW mutation result discarded",
    message_template:
        "result of `{method}` is discarded — `{collection}` is unchanged after this call",
    static_help: Some(
        "Atlas collections are Copy-on-Write. Rebind the result: `collection = collection.method(args)`",
    ),
    static_note: Some("CoW methods return a new collection; the original is never mutated in-place"),
    domain: DiagnosticDomain::Typechecker,
};

/// Emitted by `atlas build` when the `atlas-launcher` binary cannot be found.
/// The launcher is installed alongside the `atlas` CLI and is required to produce
/// native OS executables. Missing launcher = `atlas build` cannot produce a binary.
pub const LAUNCHER_NOT_FOUND: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2020",
    level: DiagnosticLevel::Error,
    title: "Atlas launcher binary not found",
    message_template: "atlas-launcher binary not found alongside the atlas CLI or on PATH",
    static_help: Some(
        "Reinstall Atlas to restore atlas-launcher: `cargo install atlas-cli`. \
         The launcher must be in the same directory as the atlas binary or on your PATH.",
    ),
    static_note: Some(
        "atlas-launcher is a small binary that is embedded into every Atlas native executable \
         produced by `atlas build`",
    ),
    domain: DiagnosticDomain::Typechecker,
};

/// Emitted by `atlas build` when an I/O error occurs while writing the native binary.
pub const BINARY_EMIT_FAILED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT2021",
    level: DiagnosticLevel::Error,
    title: "Failed to emit native binary",
    message_template: "failed to write native binary to `{path}`: {reason}",
    static_help: Some(
        "Check that the target directory is writable and that you have sufficient disk space.",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

// ── AT3xxx: Semantic / Type Checking Errors ────────────────────────────────────

pub const TYPE_ERROR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3001",
    level: DiagnosticLevel::Error,
    title: "Type error",
    message_template: "type error: {detail}",
    static_help: Some("check that the expression types are compatible"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const BINARY_OP_TYPE_ERROR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3002",
    level: DiagnosticLevel::Error,
    title: "Binary operation type error",
    message_template: "operator `{op}` cannot be applied to `{left}` and `{right}`",
    static_help: Some("ensure both operands have compatible types for this operator"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const IMMUTABLE_ASSIGNMENT: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3003",
    level: DiagnosticLevel::Error,
    title: "Assignment to immutable variable",
    message_template: "cannot assign to `{name}` — it is not declared `let mut`",
    static_help: Some("use `let mut {name} = ...` to declare a mutable variable"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const MISSING_RETURN: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3004",
    level: DiagnosticLevel::Error,
    title: "Missing return value",
    message_template: "function `{name}` is missing a return value on some code paths",
    static_help: Some(
        "ensure all code paths return a value of the declared return type `{return_type}`",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const ARITY_MISMATCH: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3005",
    level: DiagnosticLevel::Error,
    title: "Function arity mismatch",
    message_template: "function `{name}` expects {expected} argument(s), found {found}",
    static_help: Some("check the function signature for the correct number of arguments"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const NOT_CALLABLE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3006",
    level: DiagnosticLevel::Error,
    title: "Expression is not callable",
    message_template: "`{expr}` is of type `{type_name}` and cannot be called as a function",
    static_help: Some("only functions can be called — check the type of this expression"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const INVALID_INDEX_TYPE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3010",
    level: DiagnosticLevel::Error,
    title: "Invalid index type",
    message_template: "cannot index with `{index_type}` — {detail}",
    static_help: Some(
        "array indices must be `number`; HashMap keys must match the declared key type",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const NOT_INDEXABLE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3011",
    level: DiagnosticLevel::Error,
    title: "Type is not indexable",
    message_template: "type `{type_name}` cannot be indexed",
    static_help: Some(
        "only arrays (`T[]`) and hashmaps (`HashMap<K,V>`) support index expressions",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const MATCH_EMPTY: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3020",
    level: DiagnosticLevel::Error,
    title: "Empty match expression",
    message_template: "match expression has no arms",
    static_help: Some("add at least one arm to the match expression"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const MATCH_ARM_TYPE_MISMATCH: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3021",
    level: DiagnosticLevel::Error,
    title: "Match arm type mismatch",
    message_template: "match arm returns `{found}`, expected `{expected}`",
    static_help: Some("all match arms must return the same type"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const PATTERN_TYPE_MISMATCH: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3022",
    level: DiagnosticLevel::Error,
    title: "Pattern type mismatch",
    message_template:
        "pattern type `{pattern_type}` is incompatible with value type `{value_type}`",
    static_help: Some("the pattern type must be compatible with the matched value"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const CONSTRUCTOR_ARITY: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3023",
    level: DiagnosticLevel::Error,
    title: "Constructor arity mismatch",
    message_template: "constructor for `{type_name}` expects {expected} field(s), found {found}",
    static_help: Some("check the struct definition for the correct number of fields"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const UNKNOWN_CONSTRUCTOR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3024",
    level: DiagnosticLevel::Error,
    title: "Unknown constructor",
    message_template: "constructor `{name}` is not defined",
    static_help: Some(
        "check the type definition — the constructor may be misspelled or not yet declared",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const UNSUPPORTED_PATTERN_TYPE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3025",
    level: DiagnosticLevel::Error,
    title: "Unsupported pattern type",
    message_template: "pattern type `{pattern_type}` is not supported in this context",
    static_help: Some("supported patterns: literals, identifiers, struct destructuring, array patterns, wildcards"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const ARRAY_PATTERN_TYPE_MISMATCH: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3026",
    level: DiagnosticLevel::Error,
    title: "Array pattern type mismatch",
    message_template: "array pattern expects element type `{expected}`, found `{found}`",
    static_help: Some("the array pattern element types must match the array element type"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const NON_EXHAUSTIVE_MATCH: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3027",
    level: DiagnosticLevel::Error,
    title: "Non-exhaustive match",
    message_template: "match is not exhaustive — missing case(s): {missing}",
    static_help: Some("add a wildcard arm `_ => { ... }` or cover all possible cases"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const NON_SHARED_TO_SHARED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3028",
    level: DiagnosticLevel::Error,
    title: "Non-`share<T>` value passed to `share` parameter",
    message_template:
        "cannot pass `{type_name}` to a `share` parameter — value is not wrapped in `share<T>`",
    static_help: Some(
        "wrap the value in a shared reference before passing it to a `share` parameter",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when an `impl Trait for Type` already exists.
pub const IMPL_ALREADY_EXISTS: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3029",
    level: DiagnosticLevel::Error,
    title: "Duplicate impl block",
    message_template: "`{type_name}` already implements `{trait_name}`",
    static_help: Some(
        "a type can only implement a given trait once — remove or merge the duplicate impl block",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when a `trait` attempts to redefine a built-in trait.
pub const TRAIT_REDEFINES_BUILTIN: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3030",
    level: DiagnosticLevel::Error,
    title: "Cannot redefine built-in trait",
    message_template: "cannot redefine built-in trait `{name}`",
    static_help: Some("built-in traits (`Copy`, `Move`, `Drop`, `Display`, `Debug`) are provided by the runtime and cannot be redeclared"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when a `trait` with the same name is declared more than once.
pub const TRAIT_ALREADY_DEFINED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3031",
    level: DiagnosticLevel::Error,
    title: "Trait already defined",
    message_template: "trait `{name}` is already declared in this scope",
    static_help: Some("trait names must be unique — rename or remove the duplicate declaration"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when an `impl` references a trait that has not been declared.
pub const TRAIT_NOT_FOUND: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3032",
    level: DiagnosticLevel::Error,
    title: "Trait not found",
    message_template: "trait `{name}` is not declared",
    static_help: Some(
        "declare the trait with `trait {name} { ... }` before using it in an impl block",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when an `impl` block is missing a required trait method.
pub const IMPL_METHOD_MISSING: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3033",
    level: DiagnosticLevel::Error,
    title: "impl block missing required method",
    message_template: "impl of `{trait_name}` for `{type_name}` is missing method `{method}`",
    static_help: Some("implement all methods declared in the trait — add the missing `fn {method}(...)` to the impl block"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when an impl method signature does not match the trait.
pub const IMPL_METHOD_SIGNATURE_MISMATCH: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3034",
    level: DiagnosticLevel::Error,
    title: "impl method signature mismatch",
    message_template: "method `{method}` signature in impl of `{trait_name}` for `{type_name}` does not match the trait declaration",
    static_help: Some("the method's parameter types and return type must exactly match the trait definition"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when a type does not implement a required trait.
pub const TYPE_DOES_NOT_IMPLEMENT_TRAIT: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3035",
    level: DiagnosticLevel::Error,
    title: "Type does not implement required trait",
    message_template: "`{type_name}` does not implement `{trait_name}`",
    static_help: Some(
        "add `impl {trait_name} for {type_name} { ... }` to satisfy the trait requirement",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when a Copy type is required but a non-Copy type is provided.
pub const COPY_TYPE_REQUIRED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3036",
    level: DiagnosticLevel::Error,
    title: "Copy type required",
    message_template: "type `{type_name}` is not Copy — this operation requires a Copy type",
    static_help: Some(
        "primitive types (`number`, `string`, `bool`) are Copy; user-defined types default to Move",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when a generic type argument does not satisfy a trait bound.
pub const TRAIT_BOUND_NOT_SATISFIED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3037",
    level: DiagnosticLevel::Error,
    title: "Trait bound not satisfied",
    message_template: "type argument `{type_name}` does not satisfy bound `{bound}`",
    static_help: Some(
        "implement the required trait for this type, or use a type that already implements it",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

// ── AT3040+: Closure Errors ────────────────────────────────────────────────────

pub const CLOSURE_CAPTURES_BORROW: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3040",
    level: DiagnosticLevel::Error,
    title: "Closure captures borrow",
    message_template: "closure captures `{name}` by borrow, but borrows cannot outlive their scope",
    static_help: Some(
        "capture by copy or use `own` ownership — borrows cannot escape their enclosing scope",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

// ── AT3050+: Type Inference Errors ─────────────────────────────────────────────

/// Fired when return type inference fails due to inconsistent return types.
pub const CANNOT_INFER_RETURN_TYPE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3050",
    level: DiagnosticLevel::Error,
    title: "Cannot infer return type",
    message_template: "cannot infer return type of `{name}` — branches return different types",
    static_help: Some("add an explicit return type annotation: `fn {name}(...) -> T`"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when a generic call cannot infer a type argument.
pub const CANNOT_INFER_TYPE_ARG: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3051",
    level: DiagnosticLevel::Error,
    title: "Cannot infer type argument",
    message_template:
        "cannot infer type argument for `{name}` — type parameter only appears in the return type",
    static_help: Some("provide an explicit type argument: `{name}::<Type>(args)`"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when an inferred type is incompatible with actual usage.
pub const INFERRED_TYPE_INCOMPATIBLE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3052",
    level: DiagnosticLevel::Error,
    title: "Inferred type incompatible with usage",
    message_template:
        "inferred type `{inferred}` is incompatible with usage expecting `{expected}`",
    static_help: Some(
        "add an explicit type annotation or fix the usage to match the inferred type",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when a variable is used after it has been moved via `own`.
pub const USE_AFTER_OWN: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3053",
    level: DiagnosticLevel::Error,
    title: "Use of moved value",
    message_template: "`{name}` was moved into an `own` parameter and cannot be used again",
    static_help: Some("use the value before the call, or pass a copy — once moved, the caller's binding is invalid"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when a `borrow` parameter escapes its scope.
pub const BORROW_ESCAPE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3054",
    level: DiagnosticLevel::Error,
    title: "`borrow` parameter escapes its scope",
    message_template: "`borrow` parameter `{name}` cannot escape the function body",
    static_help: Some("`borrow` parameters are read-only and cannot be returned, stored in a binding, or used as struct field values — copy the value instead"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when a `share` parameter is mutated or ownership-transferred.
pub const SHARE_VIOLATION: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3055",
    level: DiagnosticLevel::Error,
    title: "`share` parameter mutated or ownership-transferred",
    message_template: "cannot {action} `share` parameter `{name}`",
    static_help: Some("`share` parameters are immutable — you cannot assign to them or pass them to an `own` parameter"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when an inherent impl block names an unknown type.
pub const INHERENT_IMPL_UNKNOWN_TYPE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3056",
    level: DiagnosticLevel::Error,
    title: "Inherent impl for unknown type",
    message_template:
        "type `{type_name}` in `impl {type_name} {{ ... }}` is not declared in this file",
    static_help: Some("declare a struct or type alias named `{type_name}` before the impl block"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when the same method appears twice in an inherent impl block.
pub const INHERENT_METHOD_DUPLICATE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3057",
    level: DiagnosticLevel::Error,
    title: "Duplicate method in inherent impl",
    message_template: "method `{method}` is defined more than once in `impl {type_name}`",
    static_help: Some("each method name may appear only once per inherent impl block — rename or remove the duplicate"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Fired when `self` is not the first parameter in an inherent impl method.
pub const INHERENT_SELF_NOT_FIRST: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3058",
    level: DiagnosticLevel::Error,
    title: "`self` receiver is not the first parameter",
    message_template:
        "`self` receiver must be the first parameter of `{method}` in `impl {type_name}`",
    static_help: Some(
        "move the `self` parameter (e.g. `borrow self`) to the front of the parameter list",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const UNKNOWN_TYPE_NAME: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3060",
    level: DiagnosticLevel::Error,
    title: "Unknown type name",
    message_template: "unknown type `{type_name}`",
    static_help: Some("Atlas types: `number` (not int/float), `string` (not str/String), `bool` (not boolean), `T[]` for arrays, `HashMap<K,V>` for maps — define structs for custom types"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const NAMESPACE_METHOD_NO_RETURN_TYPE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT3061",
    level: DiagnosticLevel::Error,
    title: "Namespace method missing return type in typechecker",
    message_template: "namespace method `{namespace}.{method}` has no return type registered in the typechecker",
    static_help: Some("add a return type entry for this method in `resolve_namespace_return_type()` in `typechecker/expr.rs` — `Type::Unknown` is never valid"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

// ── AW3059/AW3060: Inherent + Ownership Warnings ──────────────────────────────

/// Warning: inherent method shadows a trait method of the same name.
pub const INHERENT_SHADOWS_TRAIT_METHOD: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AW3059",
    level: DiagnosticLevel::Warning,
    title: "Inherent method shadows trait method",
    message_template: "inherent method `{method}` on `{type_name}` shadows the `{trait_name}` trait method",
    static_help: Some("this is expected behaviour — inherent methods take precedence over trait methods; suppress with `@allow(inherent_shadow)` if intentional"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

/// Warning: a `share<T>` value passed to an `own` or `borrow` parameter.
pub const SHARE_PASSED_TO_NON_SHARE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AW3060",
    level: DiagnosticLevel::Warning,
    title: "`share<T>` passed to non-`share` parameter",
    message_template: "`share<{inner}>` value passed to `{annotation}` parameter `{name}`",
    static_help: Some("change the parameter annotation to `share` to accept shared references, or unwrap the value before passing"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

// ── AT4xxx: Async / Await Errors ───────────────────────────────────────────────

pub const AWAIT_OUTSIDE_ASYNC: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT4001",
    level: DiagnosticLevel::Error,
    title: "`await` outside async context",
    message_template: "`await` used outside of an `async fn` or top-level scope",
    static_help: Some("move this `await` into an `async fn`, or use it at the top level where the Atlas runtime is active"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const AWAIT_NON_FUTURE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT4002",
    level: DiagnosticLevel::Error,
    title: "`await` applied to a non-Future",
    message_template: "`await` applied to `{type_name}` — only `Future<T>` values can be awaited",
    static_help: Some("check that the expression returns a `Future<T>` — async functions return futures automatically"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const ASYNC_RETURN_TYPE_MISMATCH: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT4003",
    level: DiagnosticLevel::Error,
    title: "Async fn return type mismatch",
    message_template:
        "async fn `{name}` body returns `{found}`, declared return type is `{expected}`",
    static_help: Some("fix the returned value or update the declared return type annotation"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const ASYNC_FN_AS_SYNC_ARG: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT4004",
    level: DiagnosticLevel::Error,
    title: "Async fn passed as sync argument",
    message_template:
        "async fn `{name}` returns `Future<{inner}>`, not `{inner}` — cannot pass as sync argument",
    static_help: Some(
        "wrap the call in an `await` expression or change the parameter type to accept `Future<T>`",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const FUTURE_USED_WITHOUT_AWAIT: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT4005",
    level: DiagnosticLevel::Error,
    title: "Future used without `await`",
    message_template: "`Future<{inner}>` value used as `{inner}` — did you forget `await`?",
    static_help: Some("add `await` to resolve the future: `let result = expr await;`"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const ASYNC_MAIN_FORBIDDEN: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT4006",
    level: DiagnosticLevel::Error,
    title: "`main` cannot be async",
    message_template: "`main` function cannot be declared `async`",
    static_help: Some("use top-level `await` instead — the Atlas runtime wraps the entire script in `block_on` automatically"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const SPAWN_IN_SYNC_CONTEXT: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT4007",
    level: DiagnosticLevel::Error,
    title: "`spawn` in sync context",
    message_template: "`spawn` called in a sync context with no active async runtime",
    static_help: Some(
        "call `spawn` inside an `async fn` or at the top level where the Tokio runtime is active",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const FUTURE_TYPE_MISMATCH: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT4008",
    level: DiagnosticLevel::Error,
    title: "Future type mismatch",
    message_template: "`Future<{found}>` is not compatible with `Future<{expected}>`",
    static_help: Some("check that the future resolves to the expected type"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const ASYNC_CLOSURE_UNSUPPORTED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT4009",
    level: DiagnosticLevel::Error,
    title: "Async closures not supported",
    message_template: "async anonymous functions (async closures) are not yet supported",
    static_help: Some("use a named `async fn` declaration instead of an async closure"),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

pub const AWAIT_IN_SYNC_LOOP: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT4010",
    level: DiagnosticLevel::Error,
    title: "`await` inside sync loop",
    message_template: "`await` inside a sync for-loop body creates ambiguous evaluation order",
    static_help: Some(
        "move the loop into an `async fn` or restructure to avoid awaiting inside a sync loop",
    ),
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

// ── AT5xxx: Module System Errors ───────────────────────────────────────────────

pub const INVALID_MODULE_PATH: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT5001",
    level: DiagnosticLevel::Error,
    title: "Invalid module path",
    message_template: "invalid module path `{path}`",
    static_help: Some("module paths must be valid file paths relative to the project root"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const MODULE_NOT_FOUND: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT5002",
    level: DiagnosticLevel::Error,
    title: "Module not found",
    message_template: "module `{path}` not found",
    static_help: Some("check the module path and ensure the `.atl` file exists"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const CIRCULAR_DEPENDENCY: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT5003",
    level: DiagnosticLevel::Error,
    title: "Circular dependency",
    message_template: "circular module dependency detected: {cycle}",
    static_help: Some("reorganize modules to break the circular import chain"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const EXPORT_NOT_FOUND: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT5004",
    level: DiagnosticLevel::Error,
    title: "Export not found",
    message_template: "`{name}` is not exported by module `{module}`",
    static_help: Some(
        "check the module's exports — the symbol may not be exported or may be misspelled",
    ),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const IMPORT_RESOLUTION_FAILED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT5005",
    level: DiagnosticLevel::Error,
    title: "Import resolution failed",
    message_template: "failed to resolve import `{path}`: {detail}",
    static_help: Some("check the import path and module structure"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const MODULE_NOT_EXPORTED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT5006",
    level: DiagnosticLevel::Error,
    title: "Module does not export this symbol",
    message_template: "`{name}` is defined in `{module}` but not exported",
    static_help: Some("add `export` to the symbol declaration in the source module"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const NAMESPACE_IMPORT_UNSUPPORTED: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT5007",
    level: DiagnosticLevel::Error,
    title: "Namespace import not supported",
    message_template: "namespace imports (`import * as X from ...`) are not supported",
    static_help: Some("use named imports instead: `import { name } from \"module\"`"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

pub const DUPLICATE_EXPORT: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT5008",
    level: DiagnosticLevel::Error,
    title: "Duplicate export",
    message_template: "`{name}` is exported more than once from this module",
    static_help: Some(
        "each symbol can only be exported once per module — remove the duplicate export",
    ),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

// ── AT9xxx: Internal ───────────────────────────────────────────────────────────
// AT9000 (DEPRECATED_STDLIB_GLOBAL) removed — bare globals deleted, no backward compat (B35).

pub const INTERNAL_ERROR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT9995",
    level: DiagnosticLevel::Error,
    title: "Internal error",
    message_template: "internal error: {detail}",
    static_help: Some("this is a bug in the Atlas compiler or runtime — please report it"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const STACK_UNDERFLOW: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT9997",
    level: DiagnosticLevel::Error,
    title: "Stack underflow",
    message_template: "VM stack underflow — attempted to pop from an empty stack",
    static_help: Some("this is a VM internal error — please report it"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const UNKNOWN_OPCODE: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT9998",
    level: DiagnosticLevel::Error,
    title: "Unknown bytecode opcode",
    message_template: "unknown bytecode opcode `{opcode}`",
    static_help: Some("this is a VM internal error — please report it"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const GENERIC_ERROR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT9999",
    level: DiagnosticLevel::Error,
    title: "Error",
    message_template: "{detail}",
    static_help: Some("see the error message for details"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

pub const GENERIC_WARNING: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AW9999",
    level: DiagnosticLevel::Warning,
    title: "Warning",
    message_template: "{detail}",
    static_help: Some("see the warning message for details"),
    static_note: None,
    domain: DiagnosticDomain::Runtime,
};

// ── Descriptor Registry ────────────────────────────────────────────────────────

/// Lookup a descriptor by error code.  O(n) scan — only used by `atlas explain`
/// and error-quality tooling, not in hot paths.
pub fn lookup(code: &str) -> Option<&'static DiagnosticDescriptor> {
    DESCRIPTOR_REGISTRY.iter().copied().find(|d| d.code == code)
}

/// Get the static help text for an error code.
pub fn help_for(code: &str) -> Option<&'static str> {
    lookup(code).and_then(|d| d.static_help)
}

/// Get the title for an error code.
pub fn title_for(code: &str) -> Option<&'static str> {
    lookup(code).map(|d| d.title)
}

/// Get the description (title) for an error code.
pub fn description_for(code: &str) -> Option<&'static str> {
    title_for(code)
}

/// All registered descriptors.  Referenced by `atlas explain` and the coverage test.
pub static DESCRIPTOR_REGISTRY: &[&DiagnosticDescriptor] = &[
    &TYPE_MISMATCH,
    &UNDEFINED_SYMBOL,
    &INVALID_ARITY,
    &INVALID_OPERATION,
    &DIVIDE_BY_ZERO,
    &ARRAY_OUT_OF_BOUNDS,
    &INVALID_NUMERIC_RESULT,
    &STDLIB_ARG_ERROR,
    &STDLIB_VALUE_ERROR,
    &UNHASHABLE_TYPE,
    &FILESYSTEM_PERMISSION_DENIED,
    &NETWORK_PERMISSION_DENIED,
    &PROCESS_PERMISSION_DENIED,
    &ENVIRONMENT_PERMISSION_DENIED,
    &FFI_PERMISSION_DENIED,
    &IO_ERROR,
    &EXECUTION_TIMEOUT,
    &MEMORY_LIMIT_EXCEEDED,
    &SYNTAX_ERROR,
    &UNEXPECTED_TOKEN,
    &UNTERMINATED_STRING,
    &INVALID_ESCAPE,
    &UNTERMINATED_COMMENT,
    &INVALID_NUMBER,
    &UNEXPECTED_EOF,
    &MISSING_OWNERSHIP_ANNOTATION,
    &FOREIGN_SYNTAX_ECHO,
    &FOREIGN_SYNTAX_VAR,
    &FOREIGN_SYNTAX_FUNCTION_KW,
    &FOREIGN_SYNTAX_CLASS,
    &SHADOWING_PRELUDE,
    &FOREIGN_SYNTAX_CONSOLE_LOG,
    &FOREIGN_SYNTAX_INCREMENT,
    &FOREIGN_SYNTAX_IMPORT_FROM,
    &INVALID_ASSIGN_TARGET_RANGE,
    &INVALID_ASSIGN_TARGET_CALL,
    &INVALID_ASSIGN_TARGET_MEMBER,
    &INVALID_ASSIGN_TARGET,
    &MISSING_SEMICOLON,
    &MISSING_CLOSING_DELIMITER,
    &RESERVED_KEYWORD_AS_IDENTIFIER,
    &UNUSED_VARIABLE,
    &UNREACHABLE_CODE,
    &DUPLICATE_DECLARATION,
    &UNUSED_FUNCTION,
    &VARIABLE_SHADOWING,
    &CONSTANT_CONDITION,
    &UNNECESSARY_ANNOTATION,
    &UNUSED_IMPORT,
    &DEPRECATED_TYPE_ALIAS,
    &OWN_ON_PRIMITIVE,
    &BORROW_ON_SHARED,
    &BORROW_TO_OWN,
    &MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION,
    &DEPRECATED_VAR_KEYWORD,
    &DISCARDED_COW_RESULT,
    &TYPE_ERROR,
    &BINARY_OP_TYPE_ERROR,
    &IMMUTABLE_ASSIGNMENT,
    &MISSING_RETURN,
    &ARITY_MISMATCH,
    &NOT_CALLABLE,
    &INVALID_INDEX_TYPE,
    &NOT_INDEXABLE,
    &MATCH_EMPTY,
    &MATCH_ARM_TYPE_MISMATCH,
    &PATTERN_TYPE_MISMATCH,
    &CONSTRUCTOR_ARITY,
    &UNKNOWN_CONSTRUCTOR,
    &UNSUPPORTED_PATTERN_TYPE,
    &ARRAY_PATTERN_TYPE_MISMATCH,
    &NON_EXHAUSTIVE_MATCH,
    &NON_SHARED_TO_SHARED,
    &IMPL_ALREADY_EXISTS,
    &TRAIT_REDEFINES_BUILTIN,
    &TRAIT_ALREADY_DEFINED,
    &TRAIT_NOT_FOUND,
    &IMPL_METHOD_MISSING,
    &IMPL_METHOD_SIGNATURE_MISMATCH,
    &TYPE_DOES_NOT_IMPLEMENT_TRAIT,
    &COPY_TYPE_REQUIRED,
    &TRAIT_BOUND_NOT_SATISFIED,
    &CLOSURE_CAPTURES_BORROW,
    &CANNOT_INFER_RETURN_TYPE,
    &CANNOT_INFER_TYPE_ARG,
    &INFERRED_TYPE_INCOMPATIBLE,
    &USE_AFTER_OWN,
    &BORROW_ESCAPE,
    &SHARE_VIOLATION,
    &INHERENT_IMPL_UNKNOWN_TYPE,
    &INHERENT_METHOD_DUPLICATE,
    &INHERENT_SELF_NOT_FIRST,
    &UNKNOWN_TYPE_NAME,
    &NAMESPACE_METHOD_NO_RETURN_TYPE,
    &INHERENT_SHADOWS_TRAIT_METHOD,
    &SHARE_PASSED_TO_NON_SHARE,
    &AWAIT_OUTSIDE_ASYNC,
    &AWAIT_NON_FUTURE,
    &ASYNC_RETURN_TYPE_MISMATCH,
    &ASYNC_FN_AS_SYNC_ARG,
    &FUTURE_USED_WITHOUT_AWAIT,
    &ASYNC_MAIN_FORBIDDEN,
    &SPAWN_IN_SYNC_CONTEXT,
    &FUTURE_TYPE_MISMATCH,
    &ASYNC_CLOSURE_UNSUPPORTED,
    &AWAIT_IN_SYNC_LOOP,
    &INVALID_MODULE_PATH,
    &MODULE_NOT_FOUND,
    &CIRCULAR_DEPENDENCY,
    &EXPORT_NOT_FOUND,
    &IMPORT_RESOLUTION_FAILED,
    &MODULE_NOT_EXPORTED,
    &NAMESPACE_IMPORT_UNSUPPORTED,
    &DUPLICATE_EXPORT,
    &INTERNAL_ERROR,
    &STACK_UNDERFLOW,
    &UNKNOWN_OPCODE,
    &GENERIC_ERROR,
    &GENERIC_WARNING,
];
