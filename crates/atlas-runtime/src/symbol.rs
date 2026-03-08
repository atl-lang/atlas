//! Symbol table and name binding

use crate::ast::TypeAliasDecl;
use crate::span::Span;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol type
    pub ty: Type,
    /// Whether the symbol is mutable
    pub mutable: bool,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Declaration location
    pub span: Span,
    /// Whether this symbol is exported (for module system)
    pub exported: bool,
}

/// Symbol classification
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    /// Variable binding
    Variable,
    /// Function binding
    Function,
    /// Parameter binding
    Parameter,
    /// Builtin function
    Builtin,
}

/// Symbol table for name resolution
#[derive(Clone, Debug)]
pub struct SymbolTable {
    /// Stack of scopes (innermost last)
    scopes: Vec<HashMap<String, Symbol>>,
    /// Top-level hoisted functions
    functions: HashMap<String, Symbol>,
    /// Type alias declarations (name -> alias)
    type_aliases: HashMap<String, TypeAliasDecl>,
    /// Exported type alias names
    type_alias_exports: HashSet<String>,
}

impl SymbolTable {
    /// Create a new symbol table with builtins
    pub fn new() -> Self {
        let mut table = Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            type_aliases: HashMap::new(),
            type_alias_exports: HashSet::new(),
        };

        // Add prelude builtins
        table.define_builtin(
            "print",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()], // Accepts any type
                return_type: Box::new(Type::Void),
            },
        );
        table.define_builtin(
            "len",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()], // String or Array
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "str",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()], // Converts any type to string
                return_type: Box::new(Type::String),
            },
        );

        // String functions - Core Operations
        table.define_builtin(
            "split",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Array(Box::new(Type::String))),
            },
        );
        table.define_builtin(
            "join",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::String)), Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "trim",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "trimStart",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "trimEnd",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );

        // String functions - Search Operations
        table.define_builtin(
            "indexOf",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Generic {
                    name: "Option".to_string(),
                    type_args: vec![Type::Number],
                }),
            },
        );
        table.define_builtin(
            "lastIndexOf",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Generic {
                    name: "Option".to_string(),
                    type_args: vec![Type::Number],
                }),
            },
        );
        table.define_builtin(
            "includes",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Bool),
            },
        );

        // String functions - Transformation
        table.define_builtin(
            "toUpperCase",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "toLowerCase",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "substring",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number, Type::Number],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "charAt",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "repeat",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "replace",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String, Type::String],
                return_type: Box::new(Type::String),
            },
        );

        // String functions - Formatting
        table.define_builtin(
            "padStart",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number, Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "padEnd",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number, Type::String],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "startsWith",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "endsWith",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::Bool),
            },
        );

        // Array functions - Use Unknown for array element types to support any array type
        // This allows string[], number[], etc. to work with these functions
        table.define_builtin(
            "pop",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::any_placeholder()))],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );
        table.define_builtin(
            "shift",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::any_placeholder()))],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );
        table.define_builtin(
            "unshift",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::any_placeholder(),
                ],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );
        table.define_builtin(
            "reverse",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::any_placeholder()))],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );
        table.define_builtin(
            "concat",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Array(Box::new(Type::any_placeholder())),
                ],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );
        table.define_builtin(
            "flatten",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::Array(Box::new(
                    Type::any_placeholder(),
                ))))],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );
        table.define_builtin(
            "arrayIndexOf",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::any_placeholder(),
                ],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "arrayLastIndexOf",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::any_placeholder(),
                ],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "arrayIncludes",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::any_placeholder(),
                ],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "arrayIsEmpty",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Array(Box::new(Type::any_placeholder()))],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "slice",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Number,
                    Type::Number,
                ],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );

        // Array intrinsics (callback-based) - use Unknown for generic array support
        table.define_builtin(
            "map",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::any_placeholder()],
                        return_type: Box::new(Type::any_placeholder()),
                    },
                ],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );
        table.define_builtin(
            "filter",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::any_placeholder()],
                        return_type: Box::new(Type::Bool),
                    },
                ],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );
        table.define_builtin(
            "reduce",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::any_placeholder(), Type::any_placeholder()],
                        return_type: Box::new(Type::any_placeholder()),
                    },
                    Type::any_placeholder(),
                ],
                return_type: Box::new(Type::any_placeholder()),
            },
        );
        table.define_builtin(
            "forEach",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::any_placeholder()],
                        return_type: Box::new(Type::Void),
                    },
                ],
                return_type: Box::new(Type::Null),
            },
        );
        table.define_builtin(
            "find",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::any_placeholder()],
                        return_type: Box::new(Type::Bool),
                    },
                ],
                return_type: Box::new(Type::Generic {
                    name: "Option".to_string(),
                    type_args: vec![Type::any_placeholder()],
                }),
            },
        );
        table.define_builtin(
            "findIndex",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::any_placeholder()],
                        return_type: Box::new(Type::Bool),
                    },
                ],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "flatMap",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::any_placeholder()],
                        return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
                    },
                ],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );
        table.define_builtin(
            "some",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::any_placeholder()],
                        return_type: Box::new(Type::Bool),
                    },
                ],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "every",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::any_placeholder()],
                        return_type: Box::new(Type::Bool),
                    },
                ],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "sort",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::any_placeholder(), Type::any_placeholder()],
                        return_type: Box::new(Type::Number),
                    },
                ],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );
        table.define_builtin(
            "sortBy",
            Type::Function {
                type_params: vec![],
                params: vec![
                    Type::Array(Box::new(Type::any_placeholder())),
                    Type::Function {
                        type_params: vec![],
                        params: vec![Type::any_placeholder()],
                        return_type: Box::new(Type::Number),
                    },
                ],
                return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
            },
        );

        // Math functions - Basic Operations
        table.define_builtin(
            "abs",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "floor",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "ceil",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "round",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "min",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number, Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "max",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number, Type::Number],
                return_type: Box::new(Type::Number),
            },
        );

        // Math functions - Exponential/Power
        table.define_builtin(
            "sqrt",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::Number, Type::String],
                }),
            },
        );
        table.define_builtin(
            "pow",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number, Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "log",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::Number, Type::String],
                }),
            },
        );

        // Math functions - Trigonometry
        table.define_builtin(
            "sin",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "cos",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "tan",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "asin",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::Number, Type::String],
                }),
            },
        );
        table.define_builtin(
            "acos",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::Number, Type::String],
                }),
            },
        );
        table.define_builtin(
            "atan",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );

        // Math functions - Utilities
        table.define_builtin(
            "clamp",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number, Type::Number, Type::Number],
                return_type: Box::new(Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::Number, Type::String],
                }),
            },
        );
        table.define_builtin(
            "sign",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(Type::Number),
            },
        );
        table.define_builtin(
            "random",
            Type::Function {
                type_params: vec![],
                params: vec![],
                return_type: Box::new(Type::Number),
            },
        );

        // Math constants (registered as variables, not functions)
        table
            .define(Symbol {
                name: "PI".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            })
            .ok(); // Ignore if already defined

        table
            .define(Symbol {
                name: "E".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            })
            .ok();

        table
            .define(Symbol {
                name: "SQRT2".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            })
            .ok();

        table
            .define(Symbol {
                name: "LN2".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            })
            .ok();

        table
            .define(Symbol {
                name: "LN10".to_string(),
                ty: Type::Number,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            })
            .ok();

        // JSON functions
        table.define_builtin(
            "parseJSON",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::JsonValue, Type::String],
                }),
            },
        );
        table.define_builtin(
            "jsonIsNull",
            Type::Function {
                type_params: vec![],
                params: vec![Type::JsonValue],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "toString",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "value_to_string",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "toNumber",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::Number, Type::String],
                }),
            },
        );
        table.define_builtin(
            "toBool",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "parseInt",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number],
                return_type: Box::new(Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::Number, Type::String],
                }),
            },
        );
        table.define_builtin(
            "parseFloat",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::Number, Type::String],
                }),
            },
        );
        table.define_builtin(
            "isString",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "isNumber",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "isBool",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "isNull",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "isArray",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "isFunction",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "isObject",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "isType",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder(), Type::String],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "hasField",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder(), Type::String],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "hasMethod",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder(), Type::String],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "hasTag",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder(), Type::String],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "is_some",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Generic {
                    name: "Option".to_string(),
                    type_args: vec![Type::any_placeholder()],
                }],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "is_none",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Generic {
                    name: "Option".to_string(),
                    type_args: vec![Type::any_placeholder()],
                }],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "is_ok",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::any_placeholder(), Type::any_placeholder()],
                }],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "is_err",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::any_placeholder(), Type::any_placeholder()],
                }],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "toJSON",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()], // Accepts any serializable value
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "isValidJSON",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "prettifyJSON",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::Number], // JSON string, indent size
                return_type: Box::new(Type::String),
            },
        );
        table.define_builtin(
            "minifyJSON",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );

        register_process_functions(&mut table);

        // ── Async / Future stdlib (Phase 11) ──────────────────────────────
        // These functions return Value::Future at runtime, so the typechecker
        // must know their return type is Future<T> for `await` to be valid.
        let future_any = Type::Generic {
            name: "Future".to_string(),
            type_args: vec![Type::any_placeholder()],
        };
        let future_null = Type::Generic {
            name: "Future".to_string(),
            type_args: vec![Type::Null],
        };
        let future_string = Type::Generic {
            name: "Future".to_string(),
            type_args: vec![Type::String],
        };
        let future_array = Type::Generic {
            name: "Future".to_string(),
            type_args: vec![Type::Array(Box::new(Type::any_placeholder()))],
        };

        // Timers
        table.define_builtin(
            "sleep",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(future_null.clone()),
            },
        );
        table.define_builtin(
            "interval",
            Type::Function {
                type_params: vec![],
                params: vec![Type::Number],
                return_type: Box::new(future_null.clone()),
            },
        );
        table.define_builtin(
            "timeout",
            Type::Function {
                type_params: vec![],
                params: vec![future_any.clone(), Type::Number],
                return_type: Box::new(future_any.clone()),
            },
        );

        // Task spawning
        table.define_builtin(
            "spawn",
            Type::Function {
                type_params: vec![],
                params: vec![future_any.clone(), Type::any_placeholder()],
                return_type: Box::new(future_any.clone()),
            },
        );
        table.define_builtin(
            "taskJoin",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(future_any.clone()),
            },
        );

        // Future combinators
        table.define_builtin(
            "futureAll",
            Type::Function {
                type_params: vec![],
                // Accept any[] so empty array literals [] pass type-checking
                params: vec![Type::Array(Box::new(Type::any_placeholder()))],
                return_type: Box::new(future_array),
            },
        );
        table.define_builtin(
            "futureRace",
            Type::Function {
                type_params: vec![],
                // Accept any[] so mixed/empty arrays pass type-checking
                params: vec![Type::Array(Box::new(Type::any_placeholder()))],
                return_type: Box::new(future_any.clone()),
            },
        );

        // Future constructors
        table.define_builtin(
            "futureResolve",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(future_any.clone()),
            },
        );
        table.define_builtin(
            "futureReject",
            Type::Function {
                type_params: vec![],
                params: vec![Type::any_placeholder()],
                return_type: Box::new(future_any.clone()),
            },
        );
        table.define_builtin(
            "futureNew",
            Type::Function {
                type_params: vec![],
                params: vec![],
                return_type: Box::new(future_any.clone()),
            },
        );

        // Future introspection
        table.define_builtin(
            "futureIsResolved",
            Type::Function {
                type_params: vec![],
                params: vec![future_any.clone()],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "futureIsRejected",
            Type::Function {
                type_params: vec![],
                params: vec![future_any.clone()],
                return_type: Box::new(Type::Bool),
            },
        );
        table.define_builtin(
            "futureIsPending",
            Type::Function {
                type_params: vec![],
                params: vec![future_any.clone()],
                return_type: Box::new(Type::Bool),
            },
        );

        // Async I/O
        table.define_builtin(
            "readFileAsync",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String],
                return_type: Box::new(future_string),
            },
        );
        table.define_builtin(
            "writeFileAsync",
            Type::Function {
                type_params: vec![],
                params: vec![Type::String, Type::String],
                return_type: Box::new(future_null),
            },
        );

        table
    }

    /// Define a type alias in the current module
    pub fn define_type_alias(
        &mut self,
        alias: TypeAliasDecl,
    ) -> Result<(), Box<(String, Option<TypeAliasDecl>)>> {
        if let Some(existing) = self.type_aliases.get(&alias.name.name) {
            return Err(Box::new((
                format!("Type alias '{}' already defined", alias.name.name),
                Some(existing.clone()),
            )));
        }
        self.type_aliases.insert(alias.name.name.clone(), alias);
        Ok(())
    }

    /// Look up a type alias by name
    pub fn get_type_alias(&self, name: &str) -> Option<&TypeAliasDecl> {
        self.type_aliases.get(name)
    }

    /// Get all type aliases
    pub fn type_aliases(&self) -> &HashMap<String, TypeAliasDecl> {
        &self.type_aliases
    }

    /// Mark a type alias as exported
    pub fn mark_type_alias_exported(&mut self, name: &str) -> bool {
        if self.type_aliases.contains_key(name) {
            self.type_alias_exports.insert(name.to_string());
            true
        } else {
            false
        }
    }

    /// Get exported type aliases
    pub fn get_type_alias_exports(&self) -> HashMap<String, TypeAliasDecl> {
        self.type_alias_exports
            .iter()
            .filter_map(|name| {
                self.type_aliases
                    .get(name)
                    .cloned()
                    .map(|alias| (name.clone(), alias))
            })
            .collect()
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a symbol in the current scope
    /// Returns Err with existing symbol if symbol already exists in current scope
    pub fn define(&mut self, symbol: Symbol) -> Result<(), Box<(String, Option<Symbol>)>> {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(existing) = scope.get(&symbol.name) {
                return Err(Box::new((
                    format!("Symbol '{}' is already defined in this scope", symbol.name),
                    Some(existing.clone()),
                )));
            }
            scope.insert(symbol.name.clone(), symbol);
            Ok(())
        } else {
            Err(Box::new(("No scope to define symbol in".to_string(), None)))
        }
    }

    /// Define a top-level function (hoisted)
    /// Returns Err with existing symbol if function already exists
    pub fn define_function(&mut self, symbol: Symbol) -> Result<(), Box<(String, Option<Symbol>)>> {
        if let Some(existing) = self.functions.get(&symbol.name) {
            return Err(Box::new((
                format!("Function '{}' is already defined", symbol.name),
                Some(existing.clone()),
            )));
        }
        self.functions.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Define a scoped function (nested function, not hoisted)
    ///
    /// This defines a function in the current scope on the stack, rather than
    /// in the global functions table. Nested functions are not hoisted and
    /// follow normal lexical scoping rules.
    ///
    /// Returns Err with existing symbol if name already exists in current scope
    pub fn define_scoped_function(
        &mut self,
        symbol: Symbol,
    ) -> Result<(), Box<(String, Option<Symbol>)>> {
        // Define in current scope (not global functions HashMap)
        // This allows nested functions to shadow outer functions and follow
        // lexical scoping rules
        self.define(symbol)
    }

    /// Look up a symbol in all scopes (innermost first, then functions)
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Check local scopes first (innermost to outermost)
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }

        // Check top-level functions (hoisted)
        self.functions.get(name)
    }

    /// Look up a symbol mutably in all scopes (innermost first, then functions)
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        // Check local scopes first (innermost to outermost)
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                return scope.get_mut(name);
            }
        }

        // Check top-level functions (hoisted)
        self.functions.get_mut(name)
    }

    /// Look up a symbol mutably in the current (innermost) scope only.
    /// Returns `None` if the symbol exists only in an outer scope or not at all.
    pub fn lookup_current_scope_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.scopes.last_mut()?.get_mut(name)
    }

    /// Returns true if the symbol is defined in the current (innermost) scope.
    pub fn is_defined_in_current_scope(&self, name: &str) -> bool {
        self.scopes.last().is_some_and(|s| s.contains_key(name))
    }

    /// Define a builtin function
    fn define_builtin(&mut self, name: &str, ty: Type) {
        self.functions.insert(
            name.to_string(),
            Symbol {
                name: name.to_string(),
                ty,
                mutable: false,
                kind: SymbolKind::Builtin,
                span: Span::dummy(),
                exported: false,
            },
        );
    }

    /// Check if a name is a prelude builtin
    pub fn is_prelude_builtin(&self, name: &str) -> bool {
        if let Some(symbol) = self.functions.get(name) {
            symbol.kind == SymbolKind::Builtin
        } else {
            false
        }
    }

    /// Check if we're currently in the global scope
    pub fn is_global_scope(&self) -> bool {
        self.scopes.len() == 1
    }

    /// Get all symbols from all scopes and functions
    /// Returns a vector of all symbols in the table
    pub fn all_symbols(&self) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        // Collect from all scopes
        for scope in &self.scopes {
            for symbol in scope.values() {
                symbols.push(symbol.clone());
            }
        }

        // Collect from functions (excluding builtins for cleaner output)
        for symbol in self.functions.values() {
            if symbol.kind != SymbolKind::Builtin {
                symbols.push(symbol.clone());
            }
        }

        symbols
    }

    /// Merge another symbol table into this one (for REPL state persistence)
    ///
    /// Adds new symbols from the other table to the top-level scope.
    /// Overwrites existing symbols with the same name.
    /// Does not merge nested scopes (only top-level scope and functions).
    pub fn merge(&mut self, other: SymbolTable) {
        // Merge top-level scope (index 0)
        if let Some(other_top_scope) = other.scopes.first() {
            if let Some(self_top_scope) = self.scopes.first_mut() {
                for (name, symbol) in other_top_scope {
                    self_top_scope.insert(name.clone(), symbol.clone());
                }
            }
        }

        // Merge functions (overwrite existing)
        for (name, symbol) in other.functions {
            // Don't overwrite builtins
            if symbol.kind != SymbolKind::Builtin {
                self.functions.insert(name, symbol);
            }
        }
    }

    /// Get all exported symbols from this symbol table
    ///
    /// Returns symbols marked as exported (for module system)
    pub fn get_exports(&self) -> HashMap<String, Symbol> {
        let mut exports = HashMap::new();

        // Check top-level scope for exported symbols
        if let Some(top_scope) = self.scopes.first() {
            for (name, symbol) in top_scope {
                if symbol.exported {
                    exports.insert(name.clone(), symbol.clone());
                }
            }
        }

        // Check top-level functions for exported symbols
        for (name, symbol) in &self.functions {
            if symbol.exported && symbol.kind != SymbolKind::Builtin {
                exports.insert(name.clone(), symbol.clone());
            }
        }

        exports
    }

    /// Mark a symbol as exported
    ///
    /// Used by binder when processing export declarations
    pub fn mark_exported(&mut self, name: &str) -> bool {
        // Check top-level scope first
        if let Some(top_scope) = self.scopes.first_mut() {
            if let Some(symbol) = top_scope.get_mut(name) {
                symbol.exported = true;
                return true;
            }
        }

        // Check top-level functions
        if let Some(symbol) = self.functions.get_mut(name) {
            symbol.exported = true;
            return true;
        }

        false
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

fn register_process_functions(table: &mut SymbolTable) {
    table.define_builtin(
        "spawnProcess",
        Type::Function {
            type_params: vec![],
            params: vec![Type::Array(Box::new(Type::String))],
            return_type: Box::new(Type::Number),
        },
    );
    table.define_builtin(
        "processStdin",
        Type::Function {
            type_params: vec![],
            params: vec![Type::Number],
            return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
        },
    );
    table.define_builtin(
        "processStdout",
        Type::Function {
            type_params: vec![],
            params: vec![Type::Number],
            return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
        },
    );
    table.define_builtin(
        "processStderr",
        Type::Function {
            type_params: vec![],
            params: vec![Type::Number],
            return_type: Box::new(Type::Array(Box::new(Type::any_placeholder()))),
        },
    );
    table.define_builtin(
        "processWait",
        Type::Function {
            type_params: vec![],
            params: vec![Type::Number],
            return_type: Box::new(Type::Generic {
                name: "Result".to_string(),
                type_args: vec![Type::Number, Type::String],
            }),
        },
    );
    table.define_builtin(
        "processKill",
        Type::Function {
            type_params: vec![],
            params: vec![Type::Number, Type::Number],
            return_type: Box::new(Type::Generic {
                name: "Result".to_string(),
                type_args: vec![Type::Null, Type::String],
            }),
        },
    );
    table.define_builtin(
        "processIsRunning",
        Type::Function {
            type_params: vec![],
            params: vec![Type::Number],
            return_type: Box::new(Type::Bool),
        },
    );
    table.define_builtin(
        "processOutput",
        Type::Function {
            type_params: vec![],
            params: vec![Type::Number],
            return_type: Box::new(Type::Generic {
                name: "Result".to_string(),
                type_args: vec![Type::String, Type::String],
            }),
        },
    );

    // H-082: Register snake_case aliases for all camelCase builtins
    let aliases: &[(&str, &str)] = &[
        ("forEach", "for_each"),
        ("findIndex", "find_index"),
        ("flatMap", "flat_map"),
        ("sortBy", "sort_by"),
        ("indexOf", "index_of"),
        ("lastIndexOf", "last_index_of"),
        ("toUpperCase", "to_upper_case"),
        ("toLowerCase", "to_lower_case"),
        ("charAt", "char_at"),
        ("padStart", "pad_start"),
        ("padEnd", "pad_end"),
        ("startsWith", "starts_with"),
        ("endsWith", "ends_with"),
        ("trimStart", "trim_start"),
        ("trimEnd", "trim_end"),
        ("arrayIndexOf", "array_index_of"),
        ("arrayLastIndexOf", "array_last_index_of"),
        ("arrayIncludes", "array_includes"),
        ("parseJSON", "parse_json"),
        ("toJSON", "to_json"),
        ("isValidJSON", "is_valid_json"),
        ("prettifyJSON", "prettify_json"),
        ("minifyJSON", "minify_json"),
        ("jsonIsNull", "json_is_null"),
        ("isString", "is_string"),
        ("isNumber", "is_number"),
        ("isBool", "is_bool"),
        ("isNull", "is_null"),
        ("isArray", "is_array"),
        ("isFunction", "is_function"),
        ("isObject", "is_object"),
        ("isType", "is_type"),
        ("hasField", "has_field"),
        ("hasMethod", "has_method"),
        ("hasTag", "has_tag"),
        ("toString", "to_string_conv"),
        ("toNumber", "to_number"),
        ("toBool", "to_bool"),
        ("parseInt", "parse_int"),
        ("parseFloat", "parse_float"),
        ("spawnProcess", "spawn_process"),
        ("processStdin", "process_stdin"),
        ("processStdout", "process_stdout"),
        ("processStderr", "process_stderr"),
        ("processWait", "process_wait"),
        ("processKill", "process_kill"),
        ("processIsRunning", "process_is_running"),
        ("processOutput", "process_output"),
    ];
    for &(camel, snake) in aliases {
        if let Some(symbol) = table.functions.get(camel).cloned() {
            table.functions.insert(
                snake.to_string(),
                Symbol {
                    name: snake.to_string(),
                    ..symbol
                },
            );
        }
    }
}
