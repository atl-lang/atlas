//! Comprehensive error code registry with descriptions and help text
//!
//! Error codes follow the ATxxxx scheme for errors and AWxxxx for warnings.
//! Ranges:
//! - AT0xxx: Runtime errors (type, undefined, bounds, etc.)
//! - AT01xx: Stdlib errors
//! - AT03xx: Permission errors
//! - AT04xx: I/O errors
//! - AT1xxx: Syntax/lexer errors
//! - AT2xxx: Warnings (unused, unreachable, etc.)
//! - AT3xxx: Semantic/type checking errors
//! - AT4xxx: Async/await errors
//! - AT5xxx: Module system errors
//! - AT9xxx: Internal errors

// === Error Code Constants ===

// AT0xxx - Type and Runtime Errors
pub const TYPE_MISMATCH: &str = "AT0001";
pub const UNDEFINED_SYMBOL: &str = "AT0002";
pub const INVALID_ARITY: &str = "AT0003";
pub const INVALID_OPERATION: &str = "AT0004";
pub const DIVIDE_BY_ZERO: &str = "AT0005";
pub const ARRAY_OUT_OF_BOUNDS: &str = "AT0006";
pub const INVALID_NUMERIC_RESULT: &str = "AT0007";
pub const STDLIB_ARG_ERROR: &str = "AT0102";
pub const STDLIB_VALUE_ERROR: &str = "AT0103";
pub const UNHASHABLE_TYPE: &str = "AT0140";

// AT03xx - Permission Errors
pub const FILESYSTEM_PERMISSION_DENIED: &str = "AT0300";
pub const NETWORK_PERMISSION_DENIED: &str = "AT0301";
pub const PROCESS_PERMISSION_DENIED: &str = "AT0302";
pub const ENVIRONMENT_PERMISSION_DENIED: &str = "AT0303";

// AT04xx - I/O Errors
pub const IO_ERROR: &str = "AT0400";

// AT1xxx - Syntax Errors
pub const SYNTAX_ERROR: &str = "AT1000";
pub const UNEXPECTED_TOKEN: &str = "AT1001";
pub const UNTERMINATED_STRING: &str = "AT1002";
pub const INVALID_ESCAPE: &str = "AT1003";
pub const UNTERMINATED_COMMENT: &str = "AT1004";
pub const INVALID_NUMBER: &str = "AT1005";
pub const UNEXPECTED_EOF: &str = "AT1006";
/// Parse error: function parameter is missing an ownership annotation (`own`, `borrow`, or `share`).
pub const MISSING_OWNERSHIP_ANNOTATION: &str = "AT1007";

// AT1008–AT1015: Cross-language pattern errors (foreign syntax detected at parser level)
/// `echo` is not Atlas syntax. Use `print(expr)`.
pub const FOREIGN_SYNTAX_ECHO: &str = "AT1008";
/// `var` is not Atlas syntax. Use `let` or `let mut`.
pub const FOREIGN_SYNTAX_VAR: &str = "AT1009";
/// `function` keyword is not Atlas syntax. Use `fn`.
pub const FOREIGN_SYNTAX_FUNCTION_KW: &str = "AT1010";
/// `class` is not Atlas syntax. Use `struct`.
pub const FOREIGN_SYNTAX_CLASS: &str = "AT1011";
/// `console.log` is not Atlas syntax. Use `print(...)`.
pub const FOREIGN_SYNTAX_CONSOLE_LOG: &str = "AT1013";
/// `x++` / `x--` are not Atlas syntax. Use `x = x + 1` / `x = x - 1`.
pub const FOREIGN_SYNTAX_INCREMENT: &str = "AT1014";
/// `import X from` is not Atlas module syntax.
pub const FOREIGN_SYNTAX_IMPORT_FROM: &str = "AT1015";

// AT1016–AT1019: Specific invalid assignment target codes (replacing the 4 duplicate generic errors)
/// Assignment target is an index expression containing a range — not valid.
pub const INVALID_ASSIGN_TARGET_RANGE: &str = "AT1016";
/// Assignment to a method call result — not a valid lvalue.
pub const INVALID_ASSIGN_TARGET_CALL: &str = "AT1017";
/// Assignment to a member of a non-addressable expression — not valid.
pub const INVALID_ASSIGN_TARGET_MEMBER: &str = "AT1018";
/// Expression is not a valid assignment target.
pub const INVALID_ASSIGN_TARGET: &str = "AT1019";

pub const SHADOWING_PRELUDE: &str = "AT1012";

// AT2xxx - Warnings
pub const UNUSED_VARIABLE: &str = "AT2001";
pub const UNREACHABLE_CODE: &str = "AT2002";
pub const DUPLICATE_DECLARATION: &str = "AT2003";
pub const UNUSED_FUNCTION: &str = "AT2004";
pub const VARIABLE_SHADOWING: &str = "AT2005";
pub const CONSTANT_CONDITION: &str = "AT2006";
pub const UNNECESSARY_ANNOTATION: &str = "AT2007";
pub const UNUSED_IMPORT: &str = "AT2008";
pub const DEPRECATED_TYPE_ALIAS: &str = "AT2009";
pub const OWN_ON_PRIMITIVE: &str = "AT2010";
pub const BORROW_ON_SHARED: &str = "AT2011";
pub const BORROW_TO_OWN: &str = "AT2012";
/// Warning: a non-Copy (Move) type is passed to a parameter without an ownership annotation.
/// Add `own` or `borrow` to the parameter to clarify ownership transfer semantics.
pub const MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION: &str = "AT2013";

/// Warning: `var` keyword is deprecated. Use `let mut` for mutable bindings.
pub const DEPRECATED_VAR_KEYWORD: &str = "AT2014";

// AT3040+: Closure errors
pub const CLOSURE_CAPTURES_BORROW: &str = "AT3040";

// AT3050+: Type inference errors
/// Fired when return type inference fails due to inconsistent return types across branches.
/// Add an explicit `-> T` annotation to resolve the ambiguity.
pub const CANNOT_INFER_RETURN_TYPE: &str = "AT3050";

/// Fired when a generic call cannot infer a type argument because the type parameter
/// only appears in the return type (not in any function parameter).
/// Provide an explicit type argument: `func::<Type>(args)`.
pub const CANNOT_INFER_TYPE_ARG: &str = "AT3051";

/// Fired when a type that was inferred (or computed) for a variable or expression is
/// incompatible with how it is used at the call or operator site.
pub const INFERRED_TYPE_INCOMPATIBLE: &str = "AT3052";

/// Fired when a variable that was moved into an `own` parameter is used again after the call.
/// The caller's binding is invalidated after an `own` transfer.
pub const USE_AFTER_OWN: &str = "AT3053";

/// Fired when a `borrow` parameter escapes its scope: returned from a function,
/// stored in a let binding, or used as a struct literal field value.
/// Borrows are read-only within the function body and cannot outlive the call.
pub const BORROW_ESCAPE: &str = "AT3054";

/// Fired when a `share` parameter is mutated (assigned to) or transferred via `own`
/// to another function. Share params are read-only from the callee's perspective —
/// neither side may mutate through the shared reference.
pub const SHARE_VIOLATION: &str = "AT3055";

// AT3xxx - Semantic and Type Checking Errors
pub const TYPE_ERROR: &str = "AT3001";
pub const BINARY_OP_TYPE_ERROR: &str = "AT3002";
pub const IMMUTABLE_ASSIGNMENT: &str = "AT3003";
pub const MISSING_RETURN: &str = "AT3004";
pub const ARITY_MISMATCH: &str = "AT3005";
pub const NOT_CALLABLE: &str = "AT3006";
pub const INVALID_INDEX_TYPE: &str = "AT3010";
pub const NOT_INDEXABLE: &str = "AT3011";
pub const MATCH_EMPTY: &str = "AT3020";
pub const MATCH_ARM_TYPE_MISMATCH: &str = "AT3021";
pub const PATTERN_TYPE_MISMATCH: &str = "AT3022";
pub const CONSTRUCTOR_ARITY: &str = "AT3023";
pub const UNKNOWN_CONSTRUCTOR: &str = "AT3024";
pub const UNSUPPORTED_PATTERN_TYPE: &str = "AT3025";
pub const ARRAY_PATTERN_TYPE_MISMATCH: &str = "AT3026";
pub const NON_EXHAUSTIVE_MATCH: &str = "AT3027";
pub const NON_SHARED_TO_SHARED: &str = "AT3028";

/// Fired when an `impl Trait for Type` already exists for the same `(Type, Trait)` pair.
/// Each type may only have one impl per trait. Remove or merge duplicate impls.
pub const IMPL_ALREADY_EXISTS: &str = "AT3029";

/// Fired when a `trait` declaration attempts to redefine a built-in trait (Copy, Move, Drop,
/// Display, Debug). Built-in traits are provided by the runtime and cannot be redeclared.
pub const TRAIT_REDEFINES_BUILTIN: &str = "AT3030";

/// Fired when a `trait` with the same name is declared more than once in the same scope.
/// Trait names must be unique. Rename or remove the duplicate declaration.
pub const TRAIT_ALREADY_DEFINED: &str = "AT3031";

/// Fired when an `impl` block references a trait that has not been declared.
/// Ensure the trait is declared with `trait TraitName { ... }` before the impl.
pub const TRAIT_NOT_FOUND: &str = "AT3032";

/// Fired when an `impl` block is missing a method required by the trait.
/// Every method listed in the trait declaration must be implemented.
pub const IMPL_METHOD_MISSING: &str = "AT3033";

/// Fired when an `impl` block's method signature does not match the trait's declaration.
/// Parameter types and return type must match exactly (excluding the `self` parameter type).
pub const IMPL_METHOD_SIGNATURE_MISMATCH: &str = "AT3034";

/// Fired when a method is called on a type that does not implement the required trait.
/// Implement the trait for the type with `impl TraitName for TypeName { ... }`.
pub const TYPE_DOES_NOT_IMPLEMENT_TRAIT: &str = "AT3035";

/// Fired when a context requires a Copy type but a non-Copy type is provided.
/// Primitive types (number, string, bool) are Copy. User-defined types default to Move.
pub const COPY_TYPE_REQUIRED: &str = "AT3036";

/// Fired when a generic type argument does not satisfy a trait bound.
/// For example, `fn f<T: Display>(x: T)` requires `T` to implement `Display`.
pub const TRAIT_BOUND_NOT_SATISFIED: &str = "AT3037";

// AT4xxx - Async/Await Errors
pub const AWAIT_OUTSIDE_ASYNC: &str = "AT4001";
pub const AWAIT_NON_FUTURE: &str = "AT4002";
pub const ASYNC_RETURN_TYPE_MISMATCH: &str = "AT4003";
pub const ASYNC_FN_AS_SYNC_ARG: &str = "AT4004";
pub const FUTURE_USED_WITHOUT_AWAIT: &str = "AT4005";
pub const ASYNC_MAIN_FORBIDDEN: &str = "AT4006";
pub const SPAWN_IN_SYNC_CONTEXT: &str = "AT4007";
pub const FUTURE_TYPE_MISMATCH: &str = "AT4008";
pub const ASYNC_CLOSURE_UNSUPPORTED: &str = "AT4009";
pub const AWAIT_IN_SYNC_LOOP: &str = "AT4010";

// AT5xxx - Module System Errors
pub const INVALID_MODULE_PATH: &str = "AT5001";
pub const MODULE_NOT_FOUND: &str = "AT5002";
pub const CIRCULAR_DEPENDENCY: &str = "AT5003";
pub const EXPORT_NOT_FOUND: &str = "AT5004";
pub const IMPORT_RESOLUTION_FAILED: &str = "AT5005";
pub const MODULE_NOT_EXPORTED: &str = "AT5006";
pub const NAMESPACE_IMPORT_UNSUPPORTED: &str = "AT5007";
pub const DUPLICATE_EXPORT: &str = "AT5008";

// AT9xxx - Internal Errors

/// Warning: a deprecated stdlib global name was called (e.g. `arrayPush`, `hashMapGet`, `readFile`).
/// These names continue to work but will be removed in a future version.
/// Use the method syntax or namespace form instead: `arr.push(x)`, `map.get(k)`, `File.read(path)`.
/// See docs/stdlib/METHOD-CONVENTIONS.md for the full mapping.
pub const DEPRECATED_STDLIB_GLOBAL: &str = "AT9000";

pub const INTERNAL_ERROR: &str = "AT9995";
pub const STACK_UNDERFLOW: &str = "AT9997";
pub const UNKNOWN_OPCODE: &str = "AT9998";
pub const GENERIC_ERROR: &str = "AT9999";
pub const GENERIC_WARNING: &str = "AW9999";

// === Error Code Info Registry ===

/// Error code descriptor with code, description, and optional help text
#[derive(Debug, Clone)]
pub struct ErrorCodeInfo {
    /// The error code string (e.g., "AT0001")
    pub code: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Optional contextual help text
    pub help: Option<&'static str>,
}

/// Get info for an error code, if known
pub fn lookup(code: &str) -> Option<ErrorCodeInfo> {
    ERROR_CODES.iter().find(|e| e.code == code).cloned()
}

/// Get help text for an error code
pub fn help_for(code: &str) -> Option<&'static str> {
    lookup(code).and_then(|e| e.help)
}

/// Get description for an error code
pub fn description_for(code: &str) -> Option<&'static str> {
    lookup(code).map(|e| e.description)
}

/// All known error codes with descriptions and help
pub static ERROR_CODES: &[ErrorCodeInfo] = &[
    // === AT0xxx: Runtime Errors ===
    ErrorCodeInfo {
        code: "AT0001",
        description: "Type mismatch",
        help: Some("Ensure the types match. Use explicit type conversions if needed."),
    },
    ErrorCodeInfo {
        code: "AT0002",
        description: "Undefined symbol",
        help: Some("Check spelling. The variable or function may not be in scope."),
    },
    ErrorCodeInfo {
        code: "AT0003",
        description: "Arity mismatch",
        help: Some("Check the function signature for the correct number of arguments."),
    },
    ErrorCodeInfo {
        code: "AT0004",
        description: "Invalid operation",
        help: Some("This operation is not supported for the given types."),
    },
    ErrorCodeInfo {
        code: "AT0005",
        description: "Division by zero",
        help: Some("Check that the divisor is not zero before dividing."),
    },
    ErrorCodeInfo {
        code: "AT0006",
        description: "Array index out of bounds",
        help: Some("Check array length with len() before accessing elements."),
    },
    ErrorCodeInfo {
        code: "AT0007",
        description: "Invalid numeric result (NaN or Infinity)",
        help: Some("Ensure the number is finite. Check inputs to math operations."),
    },
    // AT01xx: Stdlib errors
    ErrorCodeInfo {
        code: "AT0102",
        description: "Invalid stdlib argument",
        help: Some("Check the function documentation for valid argument types and ranges."),
    },
    ErrorCodeInfo {
        code: "AT0103",
        description: "Invalid value for stdlib operation",
        help: Some("The provided value is outside the expected range or type."),
    },
    ErrorCodeInfo {
        code: "AT0140",
        description: "Unhashable type",
        help: Some("Only number, string, bool, and null are hashable. Convert your value first."),
    },
    // AT03xx: Permission errors
    ErrorCodeInfo {
        code: "AT0300",
        description: "Filesystem permission denied",
        help: Some("Enable file permissions with --allow-file or adjust security settings."),
    },
    ErrorCodeInfo {
        code: "AT0301",
        description: "Network permission denied",
        help: Some("Enable network permissions with --allow-network or adjust security settings."),
    },
    ErrorCodeInfo {
        code: "AT0302",
        description: "Process permission denied",
        help: Some("Enable process permissions with --allow-process or adjust security settings."),
    },
    ErrorCodeInfo {
        code: "AT0303",
        description: "Environment variable permission denied",
        help: Some("Enable environment permissions with --allow-env or adjust security settings."),
    },
    // AT04xx: I/O errors
    ErrorCodeInfo {
        code: "AT0400",
        description: "I/O error",
        help: Some("Check file paths, permissions, and that the file system is accessible."),
    },
    // === AT1xxx: Syntax/Lexer Errors ===
    ErrorCodeInfo {
        code: "AT1000",
        description: "Syntax error",
        help: Some("Check the syntax near the indicated location."),
    },
    ErrorCodeInfo {
        code: "AT1001",
        description: "Unexpected token",
        help: Some("The parser encountered a token it didn't expect. Check for missing semicolons, brackets, or operators."),
    },
    ErrorCodeInfo {
        code: "AT1002",
        description: "Unterminated string literal",
        help: Some("Add the closing quote to complete the string."),
    },
    ErrorCodeInfo {
        code: "AT1003",
        description: "Invalid escape sequence",
        help: Some("Valid escapes: \\n, \\t, \\r, \\\\, \\\", \\0. Use \\\\ for a literal backslash."),
    },
    ErrorCodeInfo {
        code: "AT1004",
        description: "Unterminated block comment",
        help: Some("Add */ to close the block comment."),
    },
    ErrorCodeInfo {
        code: "AT1005",
        description: "Invalid number literal",
        help: Some("Check the number format. Numbers must be valid decimal or floating-point."),
    },
    ErrorCodeInfo {
        code: "AT1006",
        description: "Unexpected end of file",
        help: Some("The file ended unexpectedly. Check for missing closing brackets or semicolons."),
    },
    ErrorCodeInfo {
        code: "AT1007",
        description: "Missing ownership annotation on function parameter",
        help: Some("Every function parameter requires exactly one of: `own`, `borrow`, or `share`.\n  own x: T    — caller's binding is moved; callee owns the lifetime\n  borrow x: T — read-only; caller retains ownership after the call\n  share x: T  — both hold valid references simultaneously"),
    },
    ErrorCodeInfo {
        code: "AT1012",
        description: "Cannot shadow prelude builtin at top level",
        help: Some("Prelude builtins cannot be redefined at the top level. Use a different name or shadow in a nested scope."),
    },
    // AT1008–AT1019: Cross-language patterns and specific assignment errors
    ErrorCodeInfo {
        code: "AT1008",
        description: "Foreign syntax: `echo` is not valid in Atlas",
        help: Some("`echo` is not an Atlas keyword.\n  Use: print(expr)\n  Example: print(\"hello, world\")"),
    },
    ErrorCodeInfo {
        code: "AT1009",
        description: "Foreign syntax: `var` is not valid in Atlas",
        help: Some("`var` is not an Atlas keyword.\n  Use: let name = value         (immutable)\n       let mut name = value     (mutable)\n  Example: let x = 42  |  let mut count = 0"),
    },
    ErrorCodeInfo {
        code: "AT1010",
        description: "Foreign syntax: `function` keyword is not valid in Atlas",
        help: Some("`function` is not an Atlas keyword.\n  Use: fn name(own param: Type) -> ReturnType { body }\n  Example: fn add(own a: number, own b: number) -> number { a + b }"),
    },
    ErrorCodeInfo {
        code: "AT1011",
        description: "Foreign syntax: `class` is not valid in Atlas",
        help: Some("`class` is not an Atlas keyword.\n  Use: struct Name { field: Type }\n  Example:\n    struct Point { x: number, y: number }\n    let p = Point { x: 1, y: 2 };"),
    },
    ErrorCodeInfo {
        code: "AT1013",
        description: "Foreign syntax: `console.log` is not valid in Atlas",
        help: Some("`console.log` is not Atlas syntax.\n  Use: print(expr)\n  Example: print(\"value: \" + str(x))"),
    },
    ErrorCodeInfo {
        code: "AT1014",
        description: "Foreign syntax: `++` / `--` increment/decrement operators are not valid in Atlas",
        help: Some("`++` and `--` do not exist in Atlas.\n  Use: x = x + 1   (increment)\n       x = x - 1   (decrement)\n  Or:  x += 1  |  x -= 1"),
    },
    ErrorCodeInfo {
        code: "AT1015",
        description: "Foreign syntax: `import X from` is not Atlas module syntax",
        help: Some("`import X from \"module\"` is not Atlas syntax.\n  Use: import { name } from \"./module\"\n  Or for external modules: see docs/language/modules.md"),
    },
    ErrorCodeInfo {
        code: "AT1016",
        description: "Invalid assignment target: cannot assign to a range index",
        help: Some("Array slice assignments are not supported. Assign to a specific index:\n  arr[0] = value   ✓\n  arr[0..3] = ...  ✗"),
    },
    ErrorCodeInfo {
        code: "AT1017",
        description: "Invalid assignment target: cannot assign to a method call result",
        help: Some("Method call results are not addressable. Assign to a variable first:\n  let result = obj.method();\n  result = newValue;   — only if result is let mut"),
    },
    ErrorCodeInfo {
        code: "AT1018",
        description: "Invalid assignment target: member access on a non-addressable expression",
        help: Some("Only variable, index, and member expressions are valid assignment targets:\n  x = value          ✓  (variable)\n  arr[0] = value     ✓  (index)\n  obj.field = value  ✓  (member of variable)\n  f().field = value  ✗  (member of call result)"),
    },
    ErrorCodeInfo {
        code: "AT1019",
        description: "Invalid assignment target",
        help: Some("Valid assignment targets: variables, array indices, and struct fields.\n  x = value          ✓\n  arr[i] = value     ✓\n  obj.field = value  ✓"),
    },
    // === AT2xxx: Warnings ===
    ErrorCodeInfo {
        code: "AT2001",
        description: "Unused variable or parameter",
        help: Some("Remove the unused binding or prefix with underscore: _name"),
    },
    ErrorCodeInfo {
        code: "AT2002",
        description: "Unreachable code",
        help: Some("Remove this code or restructure your control flow."),
    },
    ErrorCodeInfo {
        code: "AT2003",
        description: "Duplicate declaration",
        help: Some("Remove the duplicate or rename one of the declarations."),
    },
    ErrorCodeInfo {
        code: "AT2004",
        description: "Unused function",
        help: Some("Remove the unused function or prefix with underscore: _name"),
    },
    ErrorCodeInfo {
        code: "AT2005",
        description: "Variable shadowing",
        help: Some("This variable shadows a variable from an outer scope. Use a different name if unintentional."),
    },
    ErrorCodeInfo {
        code: "AT2006",
        description: "Constant condition",
        help: Some("This condition is always true or always false. Simplify the expression."),
    },
    ErrorCodeInfo {
        code: "AT2007",
        description: "Unnecessary type annotation",
        help: Some("The type can be inferred. Consider removing the explicit annotation."),
    },
    ErrorCodeInfo {
        code: "AT2008",
        description: "Unused import",
        help: Some("Remove the unused import statement."),
    },
    ErrorCodeInfo {
        code: "AT2009",
        description: "Deprecated type alias",
        help: Some("Use the recommended replacement instead of the deprecated alias."),
    },
    ErrorCodeInfo {
        code: "AT2010",
        description: "`own` annotation on primitive type has no effect",
        help: Some("Primitive types (number, bool, string) are always copied. The `own` annotation is ignored."),
    },
    ErrorCodeInfo {
        code: "AT2011",
        description: "`borrow` annotation on `share<T>` type is redundant",
        help: Some("`share<T>` already has reference semantics. The `borrow` annotation has no additional effect."),
    },
    ErrorCodeInfo {
        code: "AT2012",
        description: "Passing borrowed value to `own` parameter — ownership cannot transfer",
        help: Some("A `borrow` parameter cannot give up ownership. Pass an owned value instead."),
    },
    ErrorCodeInfo {
        code: "AT2013",
        description: "Non-Copy type passed without ownership annotation",
        help: Some("This type is not Copy. Annotate the parameter with `own` or `borrow` to clarify ownership intent."),
    },
    ErrorCodeInfo {
        code: "AT2014",
        description: "The `var` keyword is deprecated",
        help: Some("Use `let mut` for mutable bindings. The `var` keyword will be removed in a future version."),
    },
    // === AT3xxx: Semantic/Type Checking Errors ===
    ErrorCodeInfo {
        code: "AT3001",
        description: "Type error in expression",
        help: Some("Check that the expression types are compatible."),
    },
    ErrorCodeInfo {
        code: "AT3002",
        description: "Binary operation type error",
        help: Some("Ensure both operands have compatible types for this operator."),
    },
    ErrorCodeInfo {
        code: "AT3003",
        description: "Assignment to immutable variable",
        help: Some("Use 'let mut' to declare a mutable variable."),
    },
    ErrorCodeInfo {
        code: "AT3004",
        description: "Missing return value",
        help: Some("Ensure all code paths return a value of the declared return type."),
    },
    ErrorCodeInfo {
        code: "AT3005",
        description: "Function arity mismatch",
        help: Some("Check the function signature for the correct number of arguments."),
    },
    ErrorCodeInfo {
        code: "AT3006",
        description: "Expression is not callable",
        help: Some("Only functions can be called. Check the type of this expression."),
    },
    ErrorCodeInfo {
        code: "AT3010",
        description: "Invalid index type",
        help: Some("Array indices must be numbers. HashMap keys must match the key type."),
    },
    ErrorCodeInfo {
        code: "AT3011",
        description: "Type is not indexable",
        help: Some("Only arrays and hashmaps can be indexed."),
    },
    ErrorCodeInfo {
        code: "AT3020",
        description: "Empty match expression",
        help: Some("Add at least one arm to the match expression."),
    },
    ErrorCodeInfo {
        code: "AT3021",
        description: "Match arm type mismatch",
        help: Some("All match arms must return the same type."),
    },
    ErrorCodeInfo {
        code: "AT3022",
        description: "Pattern type mismatch",
        help: Some("The pattern type must be compatible with the matched value."),
    },
    ErrorCodeInfo {
        code: "AT3023",
        description: "Constructor arity mismatch",
        help: Some("Check the constructor for the correct number of fields."),
    },
    ErrorCodeInfo {
        code: "AT3024",
        description: "Unknown constructor",
        help: Some("This constructor is not defined. Check the type definition."),
    },
    ErrorCodeInfo {
        code: "AT3025",
        description: "Unsupported pattern type",
        help: Some("This pattern form is not supported in this context."),
    },
    ErrorCodeInfo {
        code: "AT3026",
        description: "Array pattern type mismatch",
        help: Some("The array pattern must match the array element type."),
    },
    ErrorCodeInfo {
        code: "AT3027",
        description: "Non-exhaustive match",
        help: Some("Add a wildcard arm (_) or cover all possible cases."),
    },
    ErrorCodeInfo {
        code: "AT3028",
        description: "Passing non-`share<T>` value to `share` parameter",
        help: Some("Wrap the value in a shared reference before passing it to a `share` parameter."),
    },
    ErrorCodeInfo {
        code: "AT3029",
        description: "Duplicate impl block",
        help: Some("A type can only implement a given trait once. Remove the duplicate impl block."),
    },
    // === AT3030+: Trait System Errors ===
    ErrorCodeInfo {
        code: "AT3030",
        description: "Cannot redefine built-in trait",
        help: Some("Built-in traits (Copy, Move, Drop, Display, Debug) cannot be redeclared by user code."),
    },
    ErrorCodeInfo {
        code: "AT3031",
        description: "Trait already defined",
        help: Some("A trait with this name is already declared in scope. Use a different name."),
    },
    ErrorCodeInfo {
        code: "AT3032",
        description: "Trait not found",
        help: Some("The trait name was not declared. Declare it with `trait Name { ... }` before using it."),
    },
    ErrorCodeInfo {
        code: "AT3033",
        description: "impl block is missing required method",
        help: Some("The impl block must implement all methods declared in the trait."),
    },
    ErrorCodeInfo {
        code: "AT3034",
        description: "impl method signature does not match trait declaration",
        help: Some("The method's parameter types and return type must exactly match the trait definition."),
    },
    ErrorCodeInfo {
        code: "AT3035",
        description: "Type does not implement required trait",
        help: Some("Add an `impl TraitName for TypeName { ... }` block to satisfy the trait requirement."),
    },
    ErrorCodeInfo {
        code: "AT3036",
        description: "Copy type required",
        help: Some("This operation requires a Copy type. Implement the Copy trait or use a value type."),
    },
    ErrorCodeInfo {
        code: "AT3037",
        description: "Trait bound not satisfied",
        help: Some("The type argument does not satisfy the required trait bound on this type parameter."),
    },
    // === AT3040+: Closure Errors ===
    ErrorCodeInfo {
        code: "AT3040",
        description: "Cannot capture borrow in closure",
        help: Some("Borrows cannot outlive their scope. Capture by copy or use `own` ownership instead."),
    },
    // === AT3050+: Type Inference Errors ===
    ErrorCodeInfo {
        code: "AT3050",
        description: "Cannot infer return type",
        help: Some("Add an explicit return type annotation: `fn name(...) -> T`. This error fires when branches return different types and inference cannot resolve a unique return type."),
    },
    ErrorCodeInfo {
        code: "AT3051",
        description: "Cannot infer type argument",
        help: Some("The type parameter only appears in the return type or is unconstrained. Provide an explicit type argument: `func::<Type>(args)`."),
    },
    ErrorCodeInfo {
        code: "AT3052",
        description: "Inferred type incompatible with usage",
        help: Some("The type inferred for this expression is incompatible with how it is used at this site. Add an explicit type annotation or change the usage."),
    },
    ErrorCodeInfo {
        code: "AT3053",
        description: "Use of moved value after `own` transfer",
        help: Some("When a variable is passed to an `own` parameter, ownership transfers to the callee and the caller's binding becomes invalid. Use the value before the call, or pass a copy."),
    },
    ErrorCodeInfo {
        code: "AT3054",
        description: "`borrow` parameter escapes its scope",
        help: Some("`borrow` parameters are read-only within the function body. They cannot be returned, stored in a let binding, or used as struct field values. Read the value or copy primitives instead."),
    },
    ErrorCodeInfo {
        code: "AT3055",
        description: "`share` parameter mutated or ownership-transferred",
        help: Some("`share` parameters are immutable from the callee's perspective. You cannot assign to a `share` param or pass it to an `own` parameter (which would transfer ownership of something you do not own)."),
    },
    ErrorCodeInfo {
        code: "AT3061",
        description: "Namespace method has no return type in typechecker — D-010 violation",
        help: Some("Add a return type entry for this namespace method in resolve_namespace_return_type() in typechecker/expr.rs. Type::Unknown is never a valid return type."),
    },
    ErrorCodeInfo {
        code: "AT3060",
        description: "Unknown type name — likely a type from another language",
        help: Some("Atlas types: `number` (not int/float), `string` (not str/String), `bool` (not boolean), `[]T` for arrays, `HashMap<K,V>` for maps. Define structs for custom types."),
    },
    // === AT4xxx: Async/Await Errors ===
    ErrorCodeInfo {
        code: "AT4001",
        description: "`await` used outside of an async function or top-level scope",
        help: Some("Move this `await` into an `async fn`, or restructure so the `await` appears at the top level of the script."),
    },
    ErrorCodeInfo {
        code: "AT4002",
        description: "`await` applied to a non-Future value",
        help: Some("Only values of type `Future<T>` can be awaited. Check that the expression returns a Future."),
    },
    ErrorCodeInfo {
        code: "AT4003",
        description: "async fn body return type incompatible with declared return type",
        help: Some("The value returned from the async fn body does not match the declared return type. Add an explicit annotation or fix the returned value."),
    },
    ErrorCodeInfo {
        code: "AT4004",
        description: "async fn passed where a sync fn parameter is expected",
        help: Some("An `async fn` returns `Future<T>`, not `T` directly. Wrap it or change the parameter type to accept `Future<T>`."),
    },
    ErrorCodeInfo {
        code: "AT4005",
        description: "Future value used as its resolved inner type without `await`",
        help: Some("This `Future<T>` value is being used as `T`. Did you forget to `await` it?"),
    },
    ErrorCodeInfo {
        code: "AT4006",
        description: "`main` function cannot be declared `async`",
        help: Some("Use top-level `await` instead — the Atlas runtime wraps the entire script in `block_on` automatically."),
    },
    ErrorCodeInfo {
        code: "AT4007",
        description: "`spawn` called in a sync context with no active runtime",
        help: Some("Call `spawn` inside an `async fn` or at the top level where the Tokio runtime is active."),
    },
    ErrorCodeInfo {
        code: "AT4008",
        description: "Future type parameter mismatch",
        help: Some("`Future<T>` is not compatible with `Future<U>` when `T ≠ U`. Check that the future resolves to the expected type."),
    },
    ErrorCodeInfo {
        code: "AT4009",
        description: "async anonymous functions (async closures) are not yet supported",
        help: Some("Use a named `async fn` declaration instead of an async closure for now."),
    },
    ErrorCodeInfo {
        code: "AT4010",
        description: "`await` inside a sync for-loop body — ambiguous evaluation order",
        help: Some("Move the loop into an `async fn` or restructure to avoid awaiting inside a sync loop."),
    },
    // === AT5xxx: Module System Errors ===
    ErrorCodeInfo {
        code: "AT5001",
        description: "Invalid module path",
        help: Some("Module paths must be valid file paths relative to the project root."),
    },
    ErrorCodeInfo {
        code: "AT5002",
        description: "Module not found",
        help: Some("Check the module path and ensure the file exists."),
    },
    ErrorCodeInfo {
        code: "AT5003",
        description: "Circular dependency detected",
        help: Some("Reorganize modules to break the circular import chain."),
    },
    ErrorCodeInfo {
        code: "AT5004",
        description: "Export not found in module",
        help: Some("Check the module's exports. The symbol may not be exported."),
    },
    ErrorCodeInfo {
        code: "AT5005",
        description: "Import resolution failed",
        help: Some("Check the import path and module structure."),
    },
    ErrorCodeInfo {
        code: "AT5006",
        description: "Module does not export this symbol",
        help: Some("Add 'export' to the symbol declaration in the source module."),
    },
    ErrorCodeInfo {
        code: "AT5007",
        description: "Namespace import not supported",
        help: Some("Use named imports: import { name } from \"module\""),
    },
    ErrorCodeInfo {
        code: "AT5008",
        description: "Duplicate export",
        help: Some("Each symbol can only be exported once per module."),
    },
    // === AT9xxx: Internal Errors ===
    ErrorCodeInfo {
        code: "AT9000",
        description: "Deprecated stdlib global name",
        help: Some("Use method syntax or static namespace instead. See docs/stdlib/METHOD-CONVENTIONS.md for the full mapping. Example: `arrayPush(arr, x)` → `arr.push(x)`, `readFile(path)` → `File.read(path)`."),
    },
    ErrorCodeInfo {
        code: "AT9995",
        description: "Internal error",
        help: Some("This is a bug in the runtime or compiler. Please report it."),
    },
    ErrorCodeInfo {
        code: "AT9997",
        description: "Stack underflow",
        help: Some("This is a VM internal error. Please report it."),
    },
    ErrorCodeInfo {
        code: "AT9998",
        description: "Unknown bytecode opcode",
        help: Some("This is a VM internal error. Please report it."),
    },
    ErrorCodeInfo {
        code: "AT9999",
        description: "Generic error",
        help: None,
    },
    ErrorCodeInfo {
        code: "AW9999",
        description: "Generic warning",
        help: None,
    },
];
