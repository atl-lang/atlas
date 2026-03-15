//! Expression type checking

use crate::ast::*;
use crate::diagnostic::error_codes;
use crate::span::Span;
use crate::typechecker::suggestions;
use crate::typechecker::TypeChecker;
use crate::types::{StructuralMemberType, Type, TypeParamDef, ANY_TYPE_PARAM};
use std::collections::{HashMap, HashSet};

/// Resolve the expected parameter types for a static namespace method call.
/// Returns `Some(params)` when we have a known signature, `None` when variadic or untracked.
/// An empty `Some(vec![])` means zero-arg method.
fn resolve_namespace_param_types(ns: &str, method: &str) -> Option<Vec<Type>> {
    let str = Type::String;
    let num = Type::Number;
    let _json = Type::JsonValue;
    let str_arr = Type::Array(Box::new(Type::String));
    // Normalize namespace to lowercase for case-insensitive matching (AI-friendly)
    let ns_lower = ns.to_lowercase();
    match (ns_lower.as_str(), method) {
        // Json namespace
        ("json", "parse" | "isValid" | "minify" | "keys") => Some(vec![str]),
        ("json", "prettify") => Some(vec![str.clone(), num.clone()]),
        ("json", "stringify") => Some(vec![Type::any_placeholder()]),
        ("json", "getString" | "getNumber" | "getBool" | "getArray" | "getObject" | "isNull") => {
            Some(vec![str.clone(), str])
        }
        // Math namespace
        ("math", "abs" | "floor" | "ceil" | "round" | "sign") => Some(vec![num.clone()]),
        ("math", "sqrt" | "log" | "sin" | "cos" | "tan") => Some(vec![num.clone()]),
        ("math", "asin" | "acos" | "atan" | "trunc" | "log2" | "log10" | "exp" | "cbrt") => {
            Some(vec![num.clone()])
        }
        ("math", "min" | "max" | "pow") => Some(vec![num.clone(), num.clone()]),
        ("math", "atan2" | "hypot") => Some(vec![num.clone(), num.clone()]),
        ("math", "clamp") => Some(vec![num.clone(), num.clone(), num.clone()]),
        ("math", "random" | "PI" | "E" | "SQRT2" | "LN2" | "LN10") => Some(vec![]),
        // Env namespace
        ("env", "get" | "unset") => Some(vec![str.clone()]),
        ("env", "set") => Some(vec![str.clone(), str.clone()]),
        ("env", "list") => Some(vec![]),
        // File namespace — B24: full method set
        (
            "file",
            "read" | "exists" | "remove" | "createDir" | "removeDir" | "mkdir" | "mkdirp" | "rmdir"
            | "rmdirRecursive" | "readDir" | "walk" | "size" | "mtime" | "ctime" | "atime"
            | "permissions" | "inode" | "isDir" | "isFile" | "isSymlink" | "readLink",
        ) => Some(vec![str.clone()]),
        ("file", "write" | "append") => Some(vec![str.clone(), str.clone()]),
        ("file", "symlink") => Some(vec![str.clone(), str.clone()]),
        ("file", "sortEntries") => None, // variadic array arg
        ("file", "filterEntries") => Some(vec![str.clone(), str.clone()]), // entries array + pattern
        ("file", "tempFile" | "tempDir") => Some(vec![]),
        ("file", "watch") => Some(vec![str.clone()]),
        ("file", "watchNext") => None, // handle arg (Value)
        // Process namespace
        ("process", "cwd" | "pid" | "args" | "getProcessArgs" | "platform" | "arch") => {
            Some(vec![])
        }
        ("process", "exit") => Some(vec![num.clone()]), // H-266
        ("process", "run") => Some(vec![str.clone(), str_arr]),
        ("process", "shellOut") => Some(vec![str.clone()]),
        ("process", "exec") => None, // accepts string or string[] - runtime validates
        ("process", "shell") => Some(vec![str.clone()]),
        ("process", "spawn") => None, // array of strings arg
        ("process", "waitFor" | "isRunning" | "stdout" | "stderr") => None, // handle arg
        ("process", "kill") => None,  // handle + optional signal
        ("process", "stdin") => None, // handle + data arg
        // Path namespace (extname is Node.js alias for extension)
        (
            "path",
            "dirname" | "basename" | "extension" | "extname" | "normalize" | "absolute" | "parent"
            | "canonical" | "exists" | "isAbsolute" | "isRelative",
        ) => Some(vec![str.clone()]),
        ("path", "join") => None, // variadic: accepts 1+ string args
        ("path", "homedir" | "cwd" | "tempdir" | "separator") => Some(vec![]),
        // DateTime namespace
        ("datetime", "now" | "utc") => Some(vec![]),
        ("datetime", "fromTimestamp") => Some(vec![num.clone()]),
        ("datetime", "parseIso" | "parseRfc3339" | "parseRfc2822") => Some(vec![str.clone()]),
        // parse(text, format) — 2 args
        ("datetime", "parse") => Some(vec![str.clone(), str.clone()]),
        // datetime.tryParse(text, formats[]) — variadic; skip arity check
        ("datetime", "tryParse") => None,
        // datetime.fromComponents — variadic (year,month,day,hour,min,sec) → skip arity
        ("datetime", "fromComponents") => None,
        // Regex namespace
        ("regex", "new") => Some(vec![str.clone()]),
        ("regex", "test" | "isMatch") => None, // regex value + string arg; skip arity check
        ("regex", "escape") => Some(vec![str.clone()]),
        // Crypto namespace
        ("crypto", "sha256" | "sha512") => Some(vec![str.clone()]),
        ("crypto", "blake3") => Some(vec![str.clone()]),
        ("crypto", "hmac") => Some(vec![str.clone(), str.clone(), str.clone()]),
        ("crypto", "hmacVerify") => Some(vec![str.clone(), str.clone(), str.clone(), str.clone()]),
        // Encoding namespace — all methods take one string, return one string
        (
            "encoding",
            "base64Encode" | "base64Decode" | "base64UrlEncode" | "base64UrlDecode" | "hexEncode"
            | "hexDecode" | "urlEncode" | "urlDecode",
        ) => Some(vec![str.clone()]),
        // Http namespace — options-object API (B28). All accept optional map as last arg.
        // Use None (skip arity) so optional body/options args are not rejected.
        ("http", "get" | "post" | "put" | "delete" | "patch") => None,
        ("http", "checkPermission") => Some(vec![Type::String]),
        // Net namespace — variadic / complex → skip arity check
        (
            "net",
            "tcpConnect" | "tcpListen" | "tcpWrite" | "tcpRead" | "tcpReadBytes" | "tcpClose"
            | "tcpSetTimeout" | "tcpSetNodelay" | "tcpLocalAddr" | "tcpRemoteAddr" | "tcpAccept"
            | "tcpListenerAddr" | "tcpListenerClose" | "udpBind" | "udpSend" | "udpReceive"
            | "udpClose" | "udpLocalAddr" | "udpSetTimeout" | "tlsConnect" | "tlsRead" | "tlsWrite"
            | "tlsClose" | "wsConnect" | "wsSend" | "wsSendBinary" | "wsReceive" | "wsClose"
            | "wsPing",
        ) => None,
        // Io namespace
        ("io", "readLine") => Some(vec![]),
        ("io", "readLinePrompt") => Some(vec![str.clone()]),
        // Console namespace — variadic, skip arity check
        ("console", "log" | "println" | "print" | "error" | "warn" | "debug") => None,
        // Reflect namespace (B40-P03)
        (
            "reflect",
            "typeOf" | "fields" | "isCallable" | "isPrimitive" | "getLength" | "isEmpty"
            | "typeDescribe" | "clone" | "valueToString" | "getFunctionName" | "getFunctionArity",
        ) => Some(vec![Type::any_placeholder()]),
        ("reflect", "hasMethod" | "sameType" | "deepEquals") => {
            Some(vec![Type::any_placeholder(), Type::any_placeholder()])
        }
        // SQLite namespace (B40-P05)
        ("sqlite", "open") => Some(vec![str.clone()]),
        // future namespace (B33)
        ("future", "resolve" | "reject") => Some(vec![Type::any_placeholder()]),
        ("future", "all" | "race" | "allSettled" | "any") => None, // array arg — skip arity check
        ("future", "never") => Some(vec![]),
        ("future", "delay") => Some(vec![num.clone()]),
        // task namespace (B31)
        ("task", "sleep" | "interval") => Some(vec![num.clone()]),
        ("task", "spawn") => None, // Future arg — variadic
        ("task", "join" | "cancel" | "status" | "id") => None, // TaskHandle arg
        ("task", "joinAll") => None, // []TaskHandle arg
        ("task", "timeout") => None, // Future + ms args
        // test namespace — optional msg arg, skip strict arity for those with msg?
        ("test", "assert") => None,             // 1 or 2 args
        ("test", "equal" | "notEqual") => None, // 2 or 3 args
        ("test", "throws" | "noThrow") => None, // function arg
        ("test", "ok" | "err") => Some(vec![Type::any_placeholder()]),
        ("test", "contains") => None, // array + value
        ("test", "empty") => Some(vec![Type::Array(Box::new(Type::any_placeholder()))]),
        ("test", "approx") => Some(vec![num.clone(), num.clone(), num.clone()]),
        // Unknown combination
        _ => None,
    }
}

/// Resolve the return type for a static namespace method call (Json.parse, Math.sqrt, etc.)
fn resolve_namespace_return_type(ns: &str, method: &str) -> Type {
    // Normalize namespace to lowercase for case-insensitive matching (AI-friendly)
    let ns_lower = ns.to_lowercase();
    match (ns_lower.as_str(), method) {
        // Array namespace (D-062: Array.isArray(x) mirrors TypeScript — AI models know it cold)
        ("array", "isArray") => Type::Bool,
        // Json namespace
        ("json", "parse") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::JsonValue, Type::String],
        },
        ("json", "stringify") => Type::String,
        ("json", "isValid") => Type::Bool,
        ("json", "prettify") => Type::String,
        // B23: new Json namespace methods
        ("json", "minify") => Type::String,
        ("json", "keys") => Type::Array(Box::new(Type::String)),
        ("json", "getString" | "getArray" | "getObject") => Type::String,
        ("json", "getNumber") => Type::Number,
        ("json", "getBool" | "isNull") => Type::Bool,
        // Math namespace
        (
            "math",
            "abs" | "floor" | "ceil" | "round" | "min" | "max" | "pow" | "sign" | "random" | "atan"
            | "sin" | "cos" | "tan" | "trunc" | "exp" | "cbrt" | "hypot" | "atan2" | "PI" | "E"
            | "SQRT2" | "LN2" | "LN10",
        ) => Type::Number,
        ("math", "sqrt" | "clamp" | "log" | "asin" | "acos" | "log2" | "log10") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::Number, Type::String],
        },
        // Env namespace
        ("env", "get") => Type::Generic {
            name: "Option".to_string(),
            type_args: vec![Type::String],
        },
        ("env", "set" | "unset") => Type::Null,
        ("env", "list") => Type::JsonValue,
        // File namespace — B24: full return type coverage
        ("file", "read") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::String, Type::String],
        },
        (
            "file",
            "write" | "append" | "remove" | "rename" | "copy" | "createDir" | "removeDir" | "mkdir"
            | "mkdirp" | "rmdir" | "rmdirRecursive" | "symlink",
        ) => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::Null, Type::String],
        },
        ("file", "exists" | "isDir" | "isFile" | "isSymlink") => Type::Bool,
        ("file", "readDir" | "walk" | "sortEntries") => Type::Array(Box::new(Type::String)),
        ("file", "filterEntries") => Type::Array(Box::new(Type::String)),
        ("file", "info") => Type::JsonValue,
        ("file", "size" | "inode") => Type::Number,
        ("file", "mtime" | "ctime" | "atime" | "permissions" | "readLink") => Type::String,
        ("file", "tempFile" | "tempDir") => Type::String,
        ("file", "watch") => Type::JsonValue, // watcher handle
        ("file", "watchNext") => Type::String,
        // B40-P07: Async file operations — return Future<Result<T, string>>
        ("file", "readAsync") => Type::Generic {
            name: "Future".to_string(),
            type_args: vec![Type::String],
        },
        ("file", "writeAsync" | "appendAsync" | "renameAsync" | "copyAsync") => Type::Generic {
            name: "Future".to_string(),
            type_args: vec![Type::Null],
        },
        // Process namespace
        ("process", "cwd") => Type::String,
        ("process", "pid") => Type::Number,
        // H-275: process.platform() / process.arch() — OS and CPU info
        ("process", "platform" | "arch") => Type::String,
        // H-266: process.exit(code) — terminates process, returns never
        ("process", "exit") => Type::Never,
        // H-213: process.args() / process.getProcessArgs() — CLI argv access
        ("process", "args" | "getProcessArgs") => Type::Array(Box::new(Type::String)),
        // H-212: process.run(program, args) — direct exec returns Result<string, string>
        ("process", "run") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::String, Type::String],
        },
        // B18: process.exec(cmd) / process.shell(cmd) — returns Result<ProcessOutput, string>
        ("process", "exec" | "shell") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![
                Type::Generic {
                    name: "ProcessOutput".to_string(),
                    type_args: vec![],
                },
                Type::String,
            ],
        },
        ("process", "shellOut") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::String, Type::String],
        },
        // B25: process handle methods
        ("process", "spawn") => Type::Number, // returns handle (pid)
        ("process", "waitFor") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::Number, Type::String],
        },
        ("process", "kill") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::Null, Type::String],
        },
        ("process", "isRunning") => Type::Bool,
        // All three return IO handles [string, number]
        ("process", "stdin" | "stdout" | "stderr") => Type::Tuple(vec![Type::String, Type::Number]),
        // H-276: process.output returns the full output of a spawned process
        ("process", "output") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::String, Type::String],
        },
        // Path namespace (extname is Node.js alias for extension)
        (
            "path",
            "join" | "dirname" | "basename" | "extension" | "extname" | "normalize" | "absolute"
            | "resolve" | "canonical" | "homedir" | "cwd" | "tempdir" | "separator",
        ) => Type::String,
        ("path", "parent") => Type::Generic {
            name: "Option".to_string(),
            type_args: vec![Type::String],
        },
        ("path", "exists" | "isAbsolute" | "isRelative") => Type::Bool,
        // DateTime namespace — returns DateTime value (H-231)
        ("datetime", "now" | "fromTimestamp" | "fromComponents" | "utc") => Type::Generic {
            name: "DateTime".to_string(),
            type_args: vec![],
        },
        // All parse methods panic on failure (RuntimeError), return DateTime directly on success
        ("datetime", "parseIso" | "parse" | "parseRfc3339" | "parseRfc2822" | "tryParse") => {
            Type::Generic {
                name: "DateTime".to_string(),
                type_args: vec![],
            }
        }
        // Regex namespace (H-231): regex.new returns Result<Regex, string>
        ("regex", "new") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![
                Type::Generic {
                    name: "Regex".to_string(),
                    type_args: vec![],
                },
                Type::String,
            ],
        },
        // regex.test / regex.isMatch as namespace methods (also available as instance methods)
        ("regex", "test" | "isMatch") => Type::Bool,
        // regex.captures returns Option<string[]> — None when no match, Some(groups) when matched
        ("regex", "captures") => Type::Generic {
            name: "Option".to_string(),
            type_args: vec![Type::Array(Box::new(Type::String))],
        },
        // regex.capturesNamed returns Option<Map<string, string>> for named capture groups
        ("regex", "capturesNamed") => Type::Generic {
            name: "Option".to_string(),
            type_args: vec![Type::Generic {
                name: "Map".to_string(),
                type_args: vec![Type::String, Type::String],
            }],
        },
        // regex.escape escapes special chars in a string for use in a regex
        ("regex", "escape") => Type::String,
        // Note: find/findAll/replace/replaceAll/split are instance methods on Regex
        // values (dispatched via TypeTag::RegexValue), not namespace methods.
        // Crypto namespace
        ("crypto", "sha256" | "sha512" | "blake3") => Type::String,
        ("crypto", "hmac") => Type::String,
        ("crypto", "hmacVerify") => Type::Bool,
        // Encoding namespace — all methods return string
        (
            "encoding",
            "base64Encode" | "base64Decode" | "base64UrlEncode" | "base64UrlDecode" | "hexEncode"
            | "hexDecode" | "urlEncode" | "urlDecode",
        ) => Type::String,
        ("http", "checkPermission") => Type::Bool,
        // Http namespace — returns Result<HttpResponse, string> (B28 options-object API)
        ("http", "get" | "post" | "put" | "delete" | "patch") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![
                Type::Generic {
                    name: "HttpResponse".to_string(),
                    type_args: vec![],
                },
                Type::String,
            ],
        },
        // Net namespace — connection/bind methods return Result<Unknown, String>
        ("net", "tcpConnect" | "tcpListen" | "udpBind" | "tlsConnect" | "wsConnect") => {
            Type::Generic {
                name: "Result".to_string(),
                type_args: vec![Type::Unknown, Type::String],
            }
        }
        // Net namespace — read methods return String
        ("net", "tcpRead" | "tcpReadBytes" | "tlsRead" | "wsReceive" | "udpReceive") => {
            Type::String
        }
        // Net namespace — addr methods return String
        ("net", "tcpLocalAddr" | "tcpRemoteAddr" | "tcpListenerAddr" | "udpLocalAddr") => {
            Type::String
        }
        // Net namespace — void methods (write, close, set*)
        (
            "net",
            "tcpWrite" | "tcpClose" | "tcpSetTimeout" | "tcpSetNodelay" | "tcpListenerClose"
            | "tcpAccept" | "udpSend" | "udpClose" | "udpSetTimeout" | "tlsWrite" | "tlsClose"
            | "wsSend" | "wsSendBinary" | "wsClose" | "wsPing",
        ) => Type::Null,
        // Io namespace — returns Option<string> (None on EOF)
        ("io", "readLine" | "readLinePrompt") => Type::Generic {
            name: "Option".to_string(),
            type_args: vec![Type::String],
        },
        // Console namespace — all methods return void (Null)
        ("console", "log" | "println" | "print" | "error" | "warn" | "debug") => Type::Null,
        // Reflect namespace (B40-P03)
        ("reflect", "typeOf") => Type::String,
        ("reflect", "fields") => Type::Array(Box::new(Type::String)),
        ("reflect", "hasMethod") => Type::Bool,
        ("reflect", "isCallable") => Type::Bool,
        ("reflect", "isPrimitive") => Type::Bool,
        ("reflect", "sameType") => Type::Bool,
        ("reflect", "getLength") => Type::Number,
        ("reflect", "isEmpty") => Type::Bool,
        ("reflect", "typeDescribe") => Type::String,
        ("reflect", "clone") => Type::any_placeholder(),
        ("reflect", "valueToString") => Type::String,
        ("reflect", "deepEquals") => Type::Bool,
        ("reflect", "getFunctionName") => Type::String,
        ("reflect", "getFunctionArity") => Type::Number,
        // SQLite namespace (B40-P05)
        ("sqlite", "open") => Type::Generic {
            name: "SqliteConnection".to_string(),
            type_args: vec![],
        },
        // Gzip namespace
        ("gzip", "compress" | "decompress") => Type::Array(Box::new(Type::Number)),
        ("gzip", "decompressString") => Type::String,
        ("gzip", "isGzip") => Type::Bool,
        ("gzip", "compressionRatio") => Type::Number,
        // Tar namespace
        ("tar", "create" | "createGz") => Type::Null,
        ("tar", "extract" | "extractGz" | "list") => Type::Array(Box::new(Type::Unknown)),
        ("tar", "contains") => Type::Bool,
        // Zip namespace
        ("zip", "create" | "createWithComment" | "addFile") => Type::Null,
        ("zip", "extract" | "extractFiles" | "list") => Type::Array(Box::new(Type::Unknown)),
        ("zip", "contains" | "validate") => Type::Bool,
        ("zip", "compressionRatio") => Type::Number,
        ("zip", "comment") => Type::String,
        // future namespace (B33)
        (
            "future",
            "resolve" | "reject" | "all" | "race" | "allSettled" | "any" | "never" | "delay",
        ) => Type::Generic {
            name: "Future".to_string(),
            type_args: vec![],
        },
        // task namespace (B31)
        ("task", "sleep" | "interval") => Type::Generic {
            name: "Future".to_string(),
            type_args: vec![Type::Null],
        },
        ("task", "cancel") => Type::Null,
        ("task", "status") => Type::String,
        ("task", "id") => Type::Number,
        ("task", "spawn") => Type::Generic {
            name: "TaskHandle".to_string(),
            type_args: vec![],
        },
        ("task", "join") => Type::Generic {
            name: "Future".to_string(),
            type_args: vec![],
        },
        ("task", "joinAll") => Type::Generic {
            name: "Future".to_string(),
            type_args: vec![],
        },
        ("task", "timeout") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![
                Type::Generic {
                    name: "Future".to_string(),
                    type_args: vec![],
                },
                Type::String,
            ],
        },
        // sync namespace — factory functions return typed handles for method dispatch
        ("sync", "atomic") => Type::Generic {
            name: "AtomicValue".to_string(),
            type_args: vec![],
        },
        ("sync", "rwLock") => Type::Generic {
            name: "RwLockValue".to_string(),
            type_args: vec![],
        },
        ("sync", "semaphore") => Type::Generic {
            name: "SemaphoreValue".to_string(),
            type_args: vec![],
        },
        // test namespace — all assertion methods return void (Null)
        (
            "test",
            "assert" | "equal" | "notEqual" | "throws" | "noThrow" | "ok" | "err" | "contains"
            | "empty" | "approx",
        ) => Type::Null,
        // Default: unknown for unrecognized combinations
        _ => Type::Unknown,
    }
}

impl<'a> TypeChecker<'a> {
    /// Check an expression and return its type
    pub(super) fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Number(_) => Type::Number,
                Literal::String(_) => Type::String,
                Literal::Bool(_) => Type::Bool,
                Literal::Null => Type::Null,
            },
            Expr::TemplateString { parts, .. } => {
                for part in parts {
                    if let TemplatePart::Expression(expr) = part {
                        self.check_expr(expr);
                    }
                }
                Type::String
            }
            Expr::Identifier(id) => {
                // Track that this symbol was used
                self.used_symbols.insert(id.name.clone());

                // AT3053: use-after-own — variable was moved into an `own` call
                if self.moved_vars.contains(&id.name) {
                    self.diagnostics.push(
                        error_codes::USE_AFTER_OWN.emit(id.span)
                            .arg("name", &id.name)
                            .with_help(format!(
                                "after passing `{}` to an `own` parameter, the binding is invalidated.\n\
                                 To keep using the value, change the parameter annotation:\n\
                                 • `borrow {}` — read-only access, caller retains ownership\n\
                                 • `share {}`  — both hold valid refs simultaneously",
                                id.name, id.name, id.name
                            ))
                            .build()
                            .with_label("value already moved"),
                    );
                }

                if id.name == "None" {
                    return Type::Generic {
                        name: "Option".to_string(),
                        type_args: vec![Type::any_placeholder()],
                    };
                }

                if let Some(symbol) = self.symbol_table.lookup(&id.name) {
                    symbol.ty.clone()
                } else {
                    // Check if it's a user-defined enum variant (bare constructor like `Quit`
                    // or tuple variant like `Unknown` used as a function value).
                    // Skip stdlib constructors which are handled separately.
                    let is_stdlib_ctor = matches!(id.name.as_str(), "Ok" | "Err" | "Some" | "None");
                    if !is_stdlib_ctor {
                        // Clone data needed to avoid borrow conflict with resolve_type_ref.
                        let found: Option<(String, crate::ast::EnumVariant)> =
                            self.enum_decls.iter().find_map(|(enum_name, decl)| {
                                decl.variants
                                    .iter()
                                    .find(|v| v.name().name == id.name)
                                    .map(|v| (enum_name.clone(), v.clone()))
                            });
                        if let Some((enum_name, variant)) = found {
                            let enum_type = Type::Generic {
                                name: enum_name,
                                type_args: vec![],
                            };
                            return match variant {
                                crate::ast::EnumVariant::Unit { .. } => enum_type,
                                crate::ast::EnumVariant::Tuple { fields, .. } => {
                                    let params: Vec<Type> =
                                        fields.iter().map(|f| self.resolve_type_ref(f)).collect();
                                    Type::Function {
                                        type_params: vec![],
                                        params,
                                        return_type: Box::new(enum_type),
                                    }
                                }
                                crate::ast::EnumVariant::Struct { .. } => enum_type,
                            };
                        }
                    }
                    // Symbol not found - may be a builtin or undefined variable.
                    // Binder should have caught undefined variables, so this is likely a builtin.
                    Type::Unknown
                }
            }
            Expr::Binary(binary) => self.check_binary(binary),
            Expr::Unary(unary) => self.check_unary(unary),
            Expr::Call(call) => self.check_call(call),
            Expr::Index(index) => self.check_index(index),
            Expr::ArrayLiteral(arr) => self.check_array_literal(arr),
            Expr::Group(group) => self.check_expr(&group.expr),
            Expr::Match(match_expr) => self.check_match(match_expr),
            Expr::Member(member) => self.check_member(member),
            Expr::Try(try_expr) => self.check_try(try_expr),
            Expr::AnonFn {
                params,
                return_type,
                body,
                span,
            } => self.check_anon_fn(params, return_type.as_ref(), body, *span),
            Expr::Block(block) => {
                // H-115: if/else as expression — parser wraps `if cond { a } else { b }`
                // as Block { statements: [Stmt::If(...)], tail_expr: None }. Infer type from
                // both branches when both have tail expressions.
                if block.tail_expr.is_none()
                    && block.statements.len() == 1
                    && block.tail_expr.is_none()
                {
                    if let Stmt::If(if_stmt) = &block.statements[0] {
                        if let Some(else_block) = &if_stmt.else_block {
                            if let (Some(then_tail), Some(else_tail)) =
                                (&if_stmt.then_block.tail_expr, &else_block.tail_expr)
                            {
                                self.enter_scope();
                                let then_type = self.check_expr(then_tail);
                                self.exit_scope();
                                self.enter_scope();
                                let else_type = self.check_expr(else_tail);
                                self.exit_scope();
                                if then_type == else_type {
                                    return then_type;
                                }
                                return Type::union(vec![then_type, else_type]);
                            }
                        }
                    }
                }
                self.enter_scope();
                for stmt in &block.statements {
                    self.check_statement(stmt);
                }
                // If block has a tail expression (no semicolon), its type is that expr's type.
                // If the last statement is a return/break/continue, its type is Never.
                // An empty block or a block of pure statements (no tail expr) has type Void.
                let block_type = if let Some(tail) = &block.tail_expr {
                    self.check_expr(tail)
                } else {
                    match block.statements.last() {
                        Some(Stmt::Return(_)) => Type::Never,
                        _ => Type::Void,
                    }
                };
                self.exit_scope();
                block_type
            }
            Expr::ObjectLiteral(obj) => {
                let mut members = Vec::with_capacity(obj.entries.len());
                for entry in &obj.entries {
                    let value_type = self.check_expr(&entry.value);
                    members.push(StructuralMemberType {
                        name: entry.key.name.clone(),
                        ty: value_type,
                    });
                }
                Type::Structural { members }
            }
            Expr::StructExpr(struct_expr) => {
                let struct_name = struct_expr.name.name.as_str();
                let struct_type = self.resolve_struct_type(struct_name, struct_expr.name.span);

                if let Some(struct_type) = struct_type {
                    let members = match struct_type.normalized() {
                        Type::Structural { members } => members,
                        _ => Vec::new(),
                    };

                    let mut member_types: HashMap<String, Type> = HashMap::new();
                    for member in members {
                        member_types.insert(member.name.clone(), member.ty.clone());
                    }

                    let mut seen_fields: HashSet<String> = HashSet::new();
                    for field in &struct_expr.fields {
                        // AT3054: explicitly-annotated `borrow` param cannot escape into a struct
                        // field. Bare params (implicit borrow, D-040) are excluded — they are
                        // valid pass-throughs (same rule as the let-binding check above).
                        // H-267: primitives (number, bool, string) are always copied, so escape is moot.
                        if let Expr::Identifier(id) = &field.value {
                            let ownership = self
                                .current_fn_param_ownerships
                                .get(&id.name)
                                .cloned()
                                .flatten();
                            let is_explicit_borrow = ownership == Some(OwnershipAnnotation::Borrow)
                                && self.current_fn_explicit_borrow_params.contains(&id.name);
                            // Primitives are always copied — no escape semantics apply (H-267)
                            let param_type =
                                self.symbol_table.lookup(&id.name).map(|s| s.ty.clone());
                            let is_primitive = matches!(
                                param_type,
                                Some(Type::Number) | Some(Type::Bool) | Some(Type::String)
                            );
                            if is_explicit_borrow && !is_primitive {
                                self.diagnostics.push(
                                    error_codes::BORROW_ESCAPE.emit(field.span)
                                        .arg("name", &id.name)
                                        .with_help(
                                            "copy the value or use a computation result instead of \
                                             storing a `borrow` parameter directly in a struct",
                                        )
                                        .build()
                                        .with_label("borrow escapes into struct"),
                                );
                            }
                        }
                        let value_type = self.check_expr(&field.value);
                        if let Some(expected_type) = member_types.get(&field.name.name) {
                            if !seen_fields.insert(field.name.name.clone()) {
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR
                                        .emit(field.span)
                                        .arg(
                                            "detail",
                                            format!(
                                                "Duplicate field '{}' in struct '{}'",
                                                field.name.name, struct_expr.name.name
                                            ),
                                        )
                                        .with_help("each struct field can only be initialized once")
                                        .build()
                                        .with_label("duplicate field initializer"),
                                );
                            }
                            // H-162: empty array literal [] in struct field — skip mismatch
                            // if the declared field type is an array. The [] is typed as ?[]
                            // by check_array_literal but is valid when the field type is T[].
                            let is_empty_array = matches!(
                                &field.value,
                                Expr::ArrayLiteral(a) if a.elements.is_empty()
                            );
                            let expected_is_array =
                                matches!(expected_type.normalized(), Type::Array(_));
                            if is_empty_array && expected_is_array {
                                // Valid — empty literal assigned to typed array field
                            } else if !self.is_assignable_with_traits(&value_type, expected_type) {
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR
                                        .emit(field.span)
                                        .arg(
                                            "detail",
                                            format!(
                                                "Type mismatch: expected {}, found {}",
                                                expected_type.display_name(),
                                                value_type.display_name()
                                            ),
                                        )
                                        .with_help(format!(
                                            "field '{}' must be of type {}",
                                            field.name.name,
                                            expected_type.display_name()
                                        ))
                                        .build()
                                        .with_label(format!(
                                            "expected {}, found {}",
                                            expected_type.display_name(),
                                            value_type.display_name()
                                        )),
                                );
                            }
                        } else {
                            let close_match = suggestions::suggest_similar_name(
                                &field.name.name,
                                member_types.keys().map(|k| k.as_str()),
                            );
                            let valid_fields = {
                                let mut names: Vec<_> =
                                    member_types.keys().map(|k| format!("`{k}`")).collect();
                                names.sort();
                                names.join(", ")
                            };
                            let help = if let Some(ref suggestion) = close_match {
                                format!("{} — valid fields are: {}", suggestion, valid_fields)
                            } else {
                                format!(
                                    "valid fields for `{}` are: {}",
                                    struct_expr.name.name, valid_fields
                                )
                            };
                            self.diagnostics.push(
                                error_codes::INVALID_INDEX_TYPE
                                    .emit(field.span)
                                    .arg("index_type", &field.name.name)
                                    .arg(
                                        "detail",
                                        format!(
                                            "struct '{}' has no field named '{}'",
                                            struct_expr.name.name, field.name.name
                                        ),
                                    )
                                    .with_help(help)
                                    .build()
                                    .with_label("unknown field"),
                            );
                        }
                    }

                    for (field_name, _) in member_types {
                        if !seen_fields.contains(&field_name) {
                            self.diagnostics.push(
                                error_codes::TYPE_ERROR
                                    .emit(struct_expr.span)
                                    .arg(
                                        "detail",
                                        format!(
                                            "Missing field '{}' in struct '{}'",
                                            field_name, struct_expr.name.name
                                        ),
                                    )
                                    .with_help(format!(
                                        "provide a value for field '{}'",
                                        field_name
                                    ))
                                    .build()
                                    .with_label("missing field"),
                            );
                        }
                    }

                    struct_type
                } else {
                    let mut members = Vec::with_capacity(struct_expr.fields.len());
                    let mut seen = HashSet::new();
                    for field in &struct_expr.fields {
                        if !seen.insert(field.name.name.clone()) {
                            self.diagnostics.push(
                                error_codes::TYPE_ERROR
                                    .emit(field.span)
                                    .arg(
                                        "detail",
                                        format!(
                                            "Duplicate field '{}' in struct '{}'",
                                            field.name.name, struct_expr.name.name
                                        ),
                                    )
                                    .with_help("each struct field can only be initialized once")
                                    .build()
                                    .with_label("duplicate field initializer"),
                            );
                        }
                        let value_type = self.check_expr(&field.value);
                        members.push(StructuralMemberType {
                            name: field.name.name.clone(),
                            ty: value_type,
                        });
                    }
                    Type::Structural { members }
                }
            }
            Expr::Range {
                start,
                end,
                inclusive: _,
                span,
            } => self.check_range(start, end, *span),
            Expr::EnumVariant(ev) => {
                // H-229: look up declared variant fields and type-check args against them.
                // Clone the TypeRefs first to avoid borrow conflicts with self below.
                let declared_field_typerefs: Option<Vec<crate::ast::TypeRef>> = self
                    .enum_decls
                    .get(&ev.enum_name.name)
                    .and_then(|decl| {
                        decl.variants
                            .iter()
                            .find(|v| v.name().name == ev.variant_name.name)
                    })
                    .and_then(|variant| {
                        if let crate::ast::EnumVariant::Tuple { fields, .. } = variant {
                            Some(fields.clone())
                        } else {
                            None
                        }
                    });
                let declared_field_types: Option<Vec<Type>> = declared_field_typerefs
                    .map(|refs| refs.iter().map(|f| self.resolve_type_ref(f)).collect());

                if let Some(args) = &ev.args {
                    if let Some(ref field_types) = declared_field_types {
                        // Arity check
                        if args.len() != field_types.len() {
                            self.diagnostics.push(
                                error_codes::ARITY_MISMATCH
                                    .emit(ev.span)
                                    .arg("name", &ev.variant_name.name)
                                    .arg("expected", format!("{}", field_types.len()))
                                    .arg("found", format!("{}", args.len()))
                                    .with_help(format!(
                                        "variant '{}::{}' has {} field{}",
                                        ev.enum_name.name,
                                        ev.variant_name.name,
                                        field_types.len(),
                                        if field_types.len() == 1 { "" } else { "s" }
                                    ))
                                    .build()
                                    .with_label("argument count mismatch"),
                            );
                        } else {
                            // Type check each arg against its declared field type
                            for (i, (arg, expected)) in
                                args.iter().zip(field_types.iter()).enumerate()
                            {
                                let arg_type = self.check_expr(arg);
                                if expected.normalized() != Type::Unknown
                                    && !self.is_assignable_with_traits(&arg_type, expected)
                                {
                                    self.diagnostics.push(
                                        error_codes::TYPE_ERROR
                                            .emit(arg.span())
                                            .arg(
                                                "detail",
                                                format!(
                                                    "Argument {} has wrong type: expected {}, found {}",
                                                    i + 1,
                                                    expected.display_name(),
                                                    arg_type.display_name()
                                                ),
                                            )
                                            .with_help(format!(
                                                "field {} of '{}::{}' must be of type {}",
                                                i + 1,
                                                ev.enum_name.name,
                                                ev.variant_name.name,
                                                expected.display_name()
                                            ))
                                            .build()
                                            .with_label("type mismatch"),
                                    );
                                }
                            }
                        }
                    } else {
                        // No declared field types (unit variant or unknown enum) — just check exprs
                        for arg in args {
                            self.check_expr(arg);
                        }
                    }
                }

                // If the enum name is registered, return its named type so that variables
                // holding enum values don't resolve to Unknown (H-110, H-111).
                if self.enum_names.contains(&ev.enum_name.name) {
                    Type::Generic {
                        name: ev.enum_name.name.clone(),
                        type_args: vec![],
                    }
                } else {
                    Type::Unknown
                }
            }
            Expr::TupleLiteral { elements, .. } => {
                let elem_types: Vec<Type> = elements.iter().map(|e| self.check_expr(e)).collect();
                Type::Tuple(elem_types)
            }
            Expr::New {
                type_name,
                type_args,
                args,
                span: _,
            } => {
                // Type-check constructor arguments (usually empty for collections)
                for arg in args {
                    self.check_expr(arg);
                }

                // Resolve type arguments to Type
                let resolved_args: Vec<crate::types::Type> = type_args
                    .iter()
                    .map(|tr| self.resolve_type_ref(tr))
                    .collect();

                // Surface type names match internal names (D-060 + H-373)
                let type_name_str = type_name.name.as_str();

                if resolved_args.is_empty() {
                    // new Map() — no type args, return generic with no type params
                    crate::types::Type::Generic {
                        name: type_name_str.to_string(),
                        type_args: vec![],
                    }
                } else {
                    crate::types::Type::Generic {
                        name: type_name_str.to_string(),
                        type_args: resolved_args,
                    }
                }
            }

            Expr::Await { expr, span } => {
                // AT4001: await outside async context
                if !self.in_async_context {
                    self.diagnostics.push(
                        error_codes::AWAIT_OUTSIDE_ASYNC.emit(*span)
                            .with_help(
                                "move this into an `async fn`, or restructure so the `await` appears at the top level of the script",
                            )
                            .build()
                            .with_label("not inside an async fn or top-level"),
                    );
                    self.check_expr(expr);
                    return Type::Unknown;
                }
                let operand_ty = self.check_expr(expr);
                let operand_norm = operand_ty.normalized();
                // AT4002: await applied to non-Future value
                match operand_norm {
                    Type::Generic {
                        ref name,
                        ref type_args,
                    } if name == "Future" => type_args.first().cloned().unwrap_or(Type::Unknown),
                    Type::Unknown => Type::Unknown, // upstream error already reported
                    ref t if crate::types::Type::is_any_placeholder(t) => {
                        // any_placeholder — returned for stdlib bare-global calls whose return
                        // type is not in the typechecker table. Allow await on these so tests
                        // using e.g. `await sleep(10)` or `await futureResolve(42)` pass.
                        Type::Unknown
                    }
                    _ => {
                        self.diagnostics.push(
                            error_codes::AWAIT_NON_FUTURE
                                .emit(*span)
                                .arg("type_name", operand_norm.display_name())
                                .with_help("only values of type `Future<T>` can be awaited")
                                .build()
                                .with_label(format!(
                                    "type `{}` is not `Future<_>`",
                                    operand_norm.display_name()
                                )),
                        );
                        Type::Unknown
                    }
                }
            }
        }
    }

    fn check_range(
        &mut self,
        start: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
        _span: Span,
    ) -> Type {
        let check_bound = |this: &mut Self, bound: &Option<Box<Expr>>| {
            if let Some(expr) = bound {
                let bound_type = this.check_expr(expr);
                let bound_norm = bound_type.normalized();
                if bound_norm != Type::Number {
                    this.diagnostics.push(
                        error_codes::TYPE_ERROR
                            .emit(expr.span())
                            .arg(
                                "detail",
                                format!(
                                    "Range bound must be number, found {}",
                                    bound_type.display_name()
                                ),
                            )
                            .with_help("range bounds must be numbers")
                            .build()
                            .with_label("type mismatch"),
                    );
                }
            }
        };

        check_bound(self, start);
        check_bound(self, end);

        Type::Range
    }

    fn check_call_against_signature(
        &mut self,
        call: &CallExpr,
        callee_type: &Type,
        type_params: &[TypeParamDef],
        params: &[Type],
        return_type: &Type,
    ) -> Type {
        self.check_call_against_signature_inner(
            call,
            callee_type,
            type_params,
            params,
            return_type,
            &[],
        )
    }

    fn check_call_against_signature_with_types(
        &mut self,
        call: &CallExpr,
        callee_type: &Type,
        type_params: &[TypeParamDef],
        params: &[Type],
        return_type: &Type,
        pre_evaluated: &[Type],
    ) -> Type {
        self.check_call_against_signature_inner(
            call,
            callee_type,
            type_params,
            params,
            return_type,
            pre_evaluated,
        )
    }

    fn check_call_against_signature_inner(
        &mut self,
        call: &CallExpr,
        callee_type: &Type,
        type_params: &[TypeParamDef],
        params: &[Type],
        return_type: &Type,
        pre_evaluated: &[Type],
    ) -> Type {
        // Extract callee name for default param lookup (B39-P05)
        let callee_name = if let Expr::Identifier(id) = call.callee.as_ref() {
            Some(id.name.clone())
        } else {
            None
        };

        // Look up required arity for functions with defaults
        let required_arity = callee_name
            .as_ref()
            .and_then(|name| self.fn_required_arity.get(name).copied())
            .unwrap_or(params.len());

        // Look up rest param element type (B41-P04: variadic params)
        let rest_elem_type = callee_name
            .as_ref()
            .and_then(|name| self.fn_rest_param.get(name).cloned());
        let is_variadic = rest_elem_type.is_some();

        // Check argument count (B39-P05: default params; B41-P04: variadic params)
        let arg_count = call.args.len();
        let arity_ok = if is_variadic {
            arg_count >= required_arity
        } else {
            arg_count >= required_arity && arg_count <= params.len()
        };
        if !arity_ok {
            let expected = if is_variadic {
                format!("{}+", required_arity)
            } else if required_arity == params.len() {
                format!("{}", params.len())
            } else {
                format!("{}-{}", required_arity, params.len())
            };
            self.diagnostics.push(
                error_codes::ARITY_MISMATCH
                    .emit(call.span)
                    .arg("name", callee_type.display_name())
                    .arg("expected", expected.clone())
                    .arg("found", format!("{}", arg_count))
                    .with_help(if is_variadic {
                        format!(
                            "variadic function requires at least {} argument(s); you provided {}",
                            required_arity, arg_count
                        )
                    } else if required_arity < params.len() {
                        format!(
                            "function accepts {} to {} arguments; you provided {}",
                            required_arity,
                            params.len(),
                            arg_count
                        )
                    } else {
                        suggestions::suggest_arity_fix(params.len(), arg_count, callee_type)
                    })
                    .build()
                    .with_label("argument count mismatch"),
            );
        }

        // If function has type parameters, use type inference
        if !type_params.is_empty() {
            return self.check_call_with_inference(type_params, params, return_type, call);
        }

        // Non-generic function - check argument types.
        if is_variadic {
            // Check fixed args (all params except the rest param slot which is last)
            let fixed_param_count = params.len().saturating_sub(1);
            self.check_arg_types(call, &params[..fixed_param_count], pre_evaluated);
            // Check each variadic arg against the rest param element type
            if let Some(elem_ty) = rest_elem_type {
                for i in fixed_param_count..call.args.len() {
                    let arg_type = if let Some(t) = pre_evaluated.get(i) {
                        t.clone()
                    } else {
                        self.check_expr(&call.args[i])
                    };
                    if arg_type.normalized() != Type::Unknown
                        && elem_ty.normalized() != Type::Unknown
                        && !self.is_assignable_with_traits(&arg_type, &elem_ty)
                    {
                        self.diagnostics.push(
                            error_codes::TYPE_MISMATCH
                                .emit(call.args[i].span())
                                .arg("expected", elem_ty.display_name())
                                .arg("found", arg_type.display_name())
                                .with_help(format!(
                                    "rest argument {} must be of type {}",
                                    i + 1,
                                    elem_ty.display_name()
                                ))
                                .build()
                                .with_label("wrong type for rest argument"),
                        );
                    }
                }
            }
        } else {
            self.check_arg_types(call, params, pre_evaluated);
        }

        return_type.clone()
    }

    /// Check argument types against expected param types.
    /// `pre_evaluated` may be empty (args evaluated fresh) or contain pre-evaluated types
    /// (reuse to avoid double-evaluation and duplicate diagnostics).
    fn check_arg_types(&mut self, call: &CallExpr, params: &[Type], pre_evaluated: &[Type]) {
        for (i, arg) in call.args.iter().enumerate() {
            let arg_type = if let Some(t) = pre_evaluated.get(i) {
                t.clone()
            } else {
                self.check_expr(arg)
            };
            if let Some(expected_type) = params.get(i) {
                if self.is_hashmap_new_call(arg) && self.is_typed_hashmap(expected_type) {
                    continue;
                }
                if expected_type.normalized() == Type::Unknown {
                    continue;
                }
                // Skip type check when argument type is Unknown (e.g., returned from a
                // static namespace call like Json.parse() or Env.get() whose return type
                // isn't tracked by the typechecker yet).
                if arg_type.normalized() == Type::Unknown {
                    continue;
                }
                if !self.is_assignable_with_traits(&arg_type, expected_type) {
                    let help = suggestions::suggest_type_mismatch(expected_type, &arg_type)
                        .unwrap_or_else(|| {
                            format!(
                                "argument {} must be of type {}",
                                i + 1,
                                expected_type.display_name()
                            )
                        });
                    self.diagnostics.push(
                        error_codes::TYPE_ERROR
                            .emit(arg.span())
                            .arg(
                                "detail",
                                format!(
                                    "Argument {} type mismatch: expected {}, found {}",
                                    i + 1,
                                    expected_type.display_name(),
                                    arg_type.display_name()
                                ),
                            )
                            .with_help(help)
                            .build()
                            .with_label(format!(
                                "expected {}, found {}",
                                expected_type.display_name(),
                                arg_type.display_name()
                            )),
                    );
                }
            }
        }
    }

    /// Check ownership constraints at a call site.
    ///
    /// Accepts pre-evaluated argument types to avoid double-evaluating expressions.
    /// Validates:
    /// - `own` param: warn if argument is a `borrow`-annotated param of the caller
    /// - `shared` param: error if argument type is not `share<T>`
    /// - `borrow` param: always accepted, no diagnostic
    fn check_call_ownership(&mut self, call: &CallExpr, callee_name: &str, arg_types: &[Type]) {
        let ownerships = match self.fn_ownership_registry.get(callee_name) {
            Some(entry) => entry.0.clone(),
            None => return,
        };
        for (i, arg) in call.args.iter().enumerate() {
            let param_ownership = match ownerships.get(i) {
                Some(o) => o.clone(),
                None => continue,
            };
            match param_ownership {
                Some(OwnershipAnnotation::Own) => {
                    // Warn if argument is a `borrow`-annotated parameter of the enclosing function
                    if let Expr::Identifier(id) = arg {
                        let caller_ownership = self
                            .current_fn_param_ownerships
                            .get(&id.name)
                            .cloned()
                            .flatten();
                        if caller_ownership == Some(OwnershipAnnotation::Borrow) {
                            self.diagnostics.push(
                                error_codes::BORROW_TO_OWN
                                    .emit(arg.span())
                                    .arg("name", &id.name)
                                    .with_help(
                                        "pass an owned value instead of a `borrow` parameter",
                                    )
                                    .build(),
                            );
                        } else if caller_ownership == Some(OwnershipAnnotation::Share) {
                            // AT3055: share param passed to own — cannot transfer ownership of
                            // something that is shared (caller still holds a valid ref)
                            self.diagnostics.push(
                                error_codes::SHARE_VIOLATION
                                    .emit(arg.span())
                                    .arg("action", "pass to `own` parameter")
                                    .arg("name", &id.name)
                                    .with_help(
                                        "share params are held by both caller and callee — \
                                         ownership cannot be transferred to a third party",
                                    )
                                    .build(),
                            );
                        } else {
                            // Mark variable as moved — any subsequent use triggers AT3053
                            self.moved_vars.insert(id.name.clone());
                        }
                    }
                }
                Some(OwnershipAnnotation::Share) => {
                    // A share-annotated param of the enclosing function is implicitly a shared
                    // reference — allow passing it to another share param without AT3028.
                    let arg_is_share_param = if let Expr::Identifier(id) = arg {
                        self.current_fn_param_ownerships
                            .get(&id.name)
                            .cloned()
                            .flatten()
                            == Some(OwnershipAnnotation::Share)
                    } else {
                        false
                    };
                    // Error if argument type is not `share<T>` and not a share param
                    if !arg_is_share_param {
                        if let Some(arg_type) = arg_types.get(i) {
                            let is_shared = matches!(
                                arg_type,
                                Type::Generic { name, .. } if name == "share"
                            );
                            if !is_shared {
                                self.diagnostics.push(
                                    error_codes::NON_SHARED_TO_SHARED.emit(arg.span())
                                        .arg("type_name", arg_type.display_name())
                                        .with_help(
                                            "wrap the value in a shared reference before passing it",
                                        )
                                        .build(),
                                );
                            }
                        }
                    }
                }
                Some(OwnershipAnnotation::Borrow) => {
                    // borrow params accept any value — no diagnostic
                }
                None => {
                    // Unannotated param: warn if the argument type is non-Copy (Move type)
                    if let Some(arg_type) = arg_types.get(i) {
                        if matches!(arg_type.normalized(), Type::TypeParameter { .. }) {
                            continue;
                        }
                        if self.is_move_type(arg_type) {
                            self.diagnostics.push(
                                error_codes::MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION
                                    .emit(arg.span())
                                    .arg("name", arg_type.display_name())
                                    .with_help(
                                        "non-Copy types should use explicit 'own' or 'borrow' \
                                         ownership annotations",
                                    )
                                    .build(),
                            );
                        }
                    }
                }
            }
        }
    }

    /// Emit AT3052 if either operand of a binary expression is an identifier, providing
    /// context that the inferred type of that variable is incompatible at this use site.
    fn maybe_emit_at3052_for_binary(
        &mut self,
        binary: &BinaryExpr,
        left_type: &Type,
        right_type: &Type,
    ) {
        // Emit for the left side if it is an identifier
        if matches!(*binary.left, Expr::Identifier(_)) {
            self.diagnostics.push(
                error_codes::INFERRED_TYPE_INCOMPATIBLE
                    .emit(binary.left.span())
                    .arg("inferred", left_type.display_name())
                    .arg("expected", right_type.display_name())
                    .with_help("add an explicit type annotation to clarify the intended type")
                    .build()
                    .with_label(format!("has inferred type '{}'", left_type.display_name())),
            );
        } else if matches!(*binary.right, Expr::Identifier(_)) {
            // Emit for the right side
            self.diagnostics.push(
                error_codes::INFERRED_TYPE_INCOMPATIBLE
                    .emit(binary.right.span())
                    .arg("inferred", right_type.display_name())
                    .arg("expected", left_type.display_name())
                    .with_help("add an explicit type annotation to clarify the intended type")
                    .build()
                    .with_label(format!("has inferred type '{}'", right_type.display_name())),
            );
        }
    }

    /// Check a binary expression
    fn check_binary(&mut self, binary: &BinaryExpr) -> Type {
        let left_type = self.check_expr(&binary.left);
        let right_type = self.check_expr(&binary.right);
        let left_norm = left_type.normalized();
        let right_norm = right_type.normalized();

        let left_is_any =
            matches!(left_norm, Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM);
        let right_is_any =
            matches!(right_norm, Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM);
        if left_is_any || right_is_any {
            return match binary.op {
                BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge
                | BinaryOp::And
                | BinaryOp::Or => Type::Bool,
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                    Type::any_placeholder()
                }
            };
        }

        // Skip type checking if either side is Unknown (error recovery)
        if left_norm == Type::Unknown || right_norm == Type::Unknown {
            return Type::Unknown;
        }

        match binary.op {
            BinaryOp::Add => {
                if let Some(array_type) = self.array_concat_result(&left_norm, &right_norm) {
                    return array_type;
                }
                if self.all_union_pairs_valid(&left_norm, &right_norm, |a, b| {
                    (*a == Type::Number && *b == Type::Number)
                        || (*a == Type::String && *b == Type::String)
                }) {
                    if left_norm == Type::String || right_norm == Type::String {
                        Type::String
                    } else {
                        Type::Number
                    }
                } else {
                    let help = suggestions::suggest_binary_operator_fix("+", &left_type, &right_type)
                        .unwrap_or_else(|| "ensure both operands are numbers (for addition), strings (for concatenation), or arrays with compatible element types".to_string());
                    self.diagnostics.push(
                        error_codes::BINARY_OP_TYPE_ERROR
                            .emit(binary.span)
                            .arg("op", "+")
                            .arg("left", left_type.display_name())
                            .arg("right", right_type.display_name())
                            .with_help(help)
                            .build()
                            .with_label(format!(
                                "found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            )),
                    );
                    self.maybe_emit_at3052_for_binary(binary, &left_type, &right_type);
                    Type::Unknown
                }
            }
            BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if self.all_union_pairs_valid(&left_norm, &right_norm, |a, b| {
                    *a == Type::Number && *b == Type::Number
                }) {
                    Type::Number
                } else {
                    let op_str = match binary.op {
                        BinaryOp::Sub => "-",
                        BinaryOp::Mul => "*",
                        BinaryOp::Div => "/",
                        BinaryOp::Mod => "%",
                        _ => unreachable!(),
                    };
                    let help =
                        suggestions::suggest_binary_operator_fix(op_str, &left_type, &right_type)
                            .unwrap_or_else(|| {
                                format!(
                                    "'{op_str}' requires both operands to be numbers; found {} and {}. Use num() to convert strings.",
                                    left_type.display_name(),
                                    right_type.display_name()
                                )
                            });
                    self.diagnostics.push(
                        error_codes::BINARY_OP_TYPE_ERROR
                            .emit(binary.span)
                            .arg("op", op_str)
                            .arg("left", left_type.display_name())
                            .arg("right", right_type.display_name())
                            .with_help(help)
                            .build()
                            .with_label("type mismatch"),
                    );
                    Type::Unknown
                }
            }
            BinaryOp::Eq | BinaryOp::Ne => {
                // Equality requires same types
                if !self.types_overlap(&left_norm, &right_norm) {
                    self.diagnostics.push(
                        error_codes::BINARY_OP_TYPE_ERROR
                            .emit(binary.span)
                            .arg("op", "==")
                            .arg("left", left_type.display_name())
                            .arg("right", right_type.display_name())
                            .with_help(
                                "both operands must have the same type for equality comparison",
                            )
                            .build()
                            .with_label("type mismatch"),
                    );
                }
                Type::Bool
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                let left_cmp = self.comparable_operand_type(&left_norm);
                let right_cmp = self.comparable_operand_type(&right_norm);
                if self.all_union_pairs_valid(&left_cmp, &right_cmp, |a, b| {
                    *a == Type::Number && *b == Type::Number
                }) {
                    Type::Bool
                } else {
                    self.diagnostics.push(
                        error_codes::BINARY_OP_TYPE_ERROR
                            .emit(binary.span)
                            .arg("op", "<")
                            .arg("left", left_type.display_name())
                            .arg("right", right_type.display_name())
                            .with_help("comparison operators (<, <=, >, >=) only work with numbers")
                            .build()
                            .with_label("type mismatch"),
                    );
                    Type::Bool // Still return bool for error recovery
                }
            }
            BinaryOp::And | BinaryOp::Or => {
                if !self.all_union_pairs_valid(&left_norm, &right_norm, |a, b| {
                    *a == Type::Bool && *b == Type::Bool
                }) {
                    self.diagnostics.push(
                        error_codes::BINARY_OP_TYPE_ERROR
                            .emit(binary.span)
                            .arg("op", "and/or")
                            .arg("left", left_type.display_name())
                            .arg("right", right_type.display_name())
                            .with_help("logical operators (and, or) only work with bool values")
                            .build()
                            .with_label("type mismatch"),
                    );
                }
                Type::Bool
            }
        }
    }

    /// Check a unary expression
    fn check_unary(&mut self, unary: &UnaryExpr) -> Type {
        let expr_type = self.check_expr(&unary.expr);
        let expr_norm = expr_type.normalized();

        if matches!(expr_norm, Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM) {
            return Type::any_placeholder();
        }

        match unary.op {
            UnaryOp::Negate => {
                if expr_norm != Type::Number {
                    self.diagnostics.push(
                        error_codes::BINARY_OP_TYPE_ERROR
                            .emit(unary.span)
                            .arg("op", "unary -")
                            .arg("left", expr_type.display_name())
                            .arg("right", "number")
                            .with_help("negation (-) only works with numbers")
                            .build()
                            .with_label("type mismatch"),
                    );
                    Type::Unknown
                } else {
                    Type::Number
                }
            }
            UnaryOp::Not => {
                if expr_norm != Type::Bool {
                    self.diagnostics.push(
                        error_codes::BINARY_OP_TYPE_ERROR
                            .emit(unary.span)
                            .arg("op", "unary !")
                            .arg("left", expr_type.display_name())
                            .arg("right", "bool")
                            .with_help("logical not (!) only works with bool values")
                            .build()
                            .with_label("type mismatch"),
                    );
                    Type::Unknown
                } else {
                    Type::Bool
                }
            }
        }
    }

    /// Check a function call
    fn check_call(&mut self, call: &CallExpr) -> Type {
        // H-325: Constructor syntax — Foo(args) as sugar for Foo.new(args).
        // Must check BEFORE check_expr on callee to avoid "undefined variable" false errors
        // when the callee is a struct type name (not a variable).
        if let Expr::Identifier(id) = call.callee.as_ref() {
            if self.struct_decls.contains_key(id.name.as_str()) {
                if let Some(static_new) = self
                    .static_methods_registry
                    .get(&(id.name.clone(), "new".to_string()))
                    .cloned()
                {
                    let return_type = self.resolve_type_ref(&static_new.return_type);
                    let expected_count = static_new.params.len();
                    if call.args.len() != expected_count {
                        self.diagnostics.push(
                            error_codes::ARITY_MISMATCH
                                .emit(call.span)
                                .arg("name", format!("{}.new", id.name))
                                .arg("expected", format!("{}", expected_count))
                                .arg("found", format!("{}", call.args.len()))
                                .with_help(format!(
                                    "constructor `{}::new` requires {} argument{}",
                                    id.name,
                                    expected_count,
                                    if expected_count == 1 { "" } else { "s" }
                                ))
                                .build()
                                .with_label("argument count mismatch"),
                        );
                        return Type::Unknown;
                    }
                    for (i, arg) in call.args.iter().enumerate() {
                        let arg_type = self.check_expr(arg);
                        if let Some(param) = static_new.params.get(i) {
                            let expected_type = self.resolve_type_ref(&param.type_ref);
                            if !self.is_assignable_with_traits(&arg_type, &expected_type) {
                                self.diagnostics.push(
                                    error_codes::TYPE_MISMATCH
                                        .emit(arg.span())
                                        .arg("expected", expected_type.display_name())
                                        .arg("found", arg_type.display_name())
                                        .with_help(format!(
                                            "argument {} to `{}::new` has wrong type: expected {}, found {}",
                                            i + 1, id.name,
                                            expected_type.display_name(),
                                            arg_type.display_name()
                                        ))
                                        .build()
                                        .with_label("type mismatch"),
                                );
                            }
                        }
                    }
                    return return_type;
                } else {
                    self.diagnostics.push(
                        error_codes::NO_NEW_CONSTRUCTOR
                            .emit(call.span)
                            .arg("type_name", id.name.clone())
                            .with_help(format!(
                                "add a static `fn new(...): {}` method inside an inherent `impl {}` block",
                                id.name, id.name
                            ))
                            .build()
                            .with_label("no constructor"),
                    );
                    return Type::Unknown;
                }
            }
        }

        let callee_type = self.check_expr(&call.callee);
        let callee_norm = callee_type.normalized();

        // Extract callee name for ownership registry lookup (direct calls only)
        let callee_name = if let Expr::Identifier(id) = call.callee.as_ref() {
            Some(id.name.clone())
        } else {
            None
        };

        if let Some(ref name) = callee_name {
            match name.as_str() {
                "Some" => {
                    if call.args.len() != 1 {
                        self.diagnostics.push(
                            error_codes::CONSTRUCTOR_ARITY
                                .emit(call.span)
                                .arg("type_name", "Some")
                                .arg("expected", "1")
                                .arg("found", format!("{}", call.args.len()))
                                .with_help("Some requires exactly 1 argument: Some(value)")
                                .build()
                                .with_label("wrong arity"),
                        );
                        return Type::Unknown;
                    }
                    let arg_type = self.check_expr(&call.args[0]);
                    return Type::Generic {
                        name: "Option".to_string(),
                        type_args: vec![arg_type],
                    };
                }
                "None" => {
                    if !call.args.is_empty() {
                        self.diagnostics.push(
                            error_codes::CONSTRUCTOR_ARITY
                                .emit(call.span)
                                .arg("type_name", "None")
                                .arg("expected", "0")
                                .arg("found", format!("{}", call.args.len()))
                                .with_help("None requires no arguments: None")
                                .build()
                                .with_label("wrong arity"),
                        );
                        return Type::Unknown;
                    }
                    return Type::Generic {
                        name: "Option".to_string(),
                        type_args: vec![Type::any_placeholder()],
                    };
                }
                "Ok" => {
                    if call.args.len() != 1 {
                        self.diagnostics.push(
                            error_codes::CONSTRUCTOR_ARITY
                                .emit(call.span)
                                .arg("type_name", "Ok")
                                .arg("expected", "1")
                                .arg("found", format!("{}", call.args.len()))
                                .with_help("Ok requires exactly 1 argument: Ok(value)")
                                .build()
                                .with_label("wrong arity"),
                        );
                        return Type::Unknown;
                    }
                    let arg_type = self.check_expr(&call.args[0]);
                    return Type::Generic {
                        name: "Result".to_string(),
                        type_args: vec![arg_type, Type::any_placeholder()],
                    };
                }
                "Err" => {
                    if call.args.len() != 1 {
                        self.diagnostics.push(
                            error_codes::CONSTRUCTOR_ARITY
                                .emit(call.span)
                                .arg("type_name", "Err")
                                .arg("expected", "1")
                                .arg("found", format!("{}", call.args.len()))
                                .with_help("Err requires exactly 1 argument: Err(value)")
                                .build()
                                .with_label("wrong arity"),
                        );
                        return Type::Unknown;
                    }
                    let arg_type = self.check_expr(&call.args[0]);
                    return Type::Generic {
                        name: "Result".to_string(),
                        type_args: vec![Type::any_placeholder(), arg_type],
                    };
                }
                "mapNew" | "map_new" => {
                    if !call.args.is_empty() {
                        self.diagnostics.push(
                            error_codes::ARITY_MISMATCH
                                .emit(call.span)
                                .arg("name", "mapNew")
                                .arg("expected", "0")
                                .arg("found", format!("{}", call.args.len()))
                                .with_help("mapNew() takes no arguments")
                                .build()
                                .with_label("argument count mismatch"),
                        );
                        return Type::Unknown;
                    }
                    return Type::Generic {
                        name: "Map".to_string(),
                        type_args: vec![Type::any_placeholder(), Type::any_placeholder()],
                    };
                }
                "mapSet" | "map_set" => {
                    if call.args.len() != 3 {
                        self.diagnostics.push(
                            error_codes::ARITY_MISMATCH
                                .emit(call.span)
                                .arg("name", "mapSet")
                                .arg("expected", "3")
                                .arg("found", format!("{}", call.args.len()))
                                .with_help("mapSet(map, key, value) requires exactly 3 arguments")
                                .build()
                                .with_label("argument count mismatch"),
                        );
                        return Type::Unknown;
                    }

                    let map_type = self.check_expr(&call.args[0]);
                    let key_type = self.check_expr(&call.args[1]);
                    let value_type = self.check_expr(&call.args[2]);

                    let Some((expected_key, expected_value)) = self.hashmap_type_args(&map_type)
                    else {
                        let map_norm = map_type.normalized();
                        let map_is_any_or_unknown = matches!(map_norm, Type::Unknown)
                            || matches!(
                                map_norm,
                                Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM
                            );
                        let map_is_structural = matches!(map_norm, Type::Structural { .. });
                        if map_is_any_or_unknown || map_is_structural {
                            return map_type;
                        }
                        self.diagnostics.push(
                            error_codes::TYPE_ERROR
                                .emit(call.args[0].span())
                                .arg(
                                    "detail",
                                    format!(
                                        "mapSet expects Map for argument 1, found {}",
                                        map_type.display_name()
                                    ),
                                )
                                .with_help("argument 1 must be a HashMap")
                                .build()
                                .with_label("type mismatch"),
                        );
                        return Type::Unknown;
                    };

                    if !self.is_untyped_hashmap(&map_type) {
                        if !self.is_assignable_with_traits(&key_type, &expected_key) {
                            let help = suggestions::suggest_type_mismatch(&expected_key, &key_type)
                                .unwrap_or_else(|| {
                                    format!(
                                        "expected {}, found {}",
                                        expected_key.display_name(),
                                        key_type.display_name()
                                    )
                                });
                            self.diagnostics.push(
                                error_codes::TYPE_ERROR
                                    .emit(call.args[1].span())
                                    .arg(
                                        "detail",
                                        format!(
                                            "mapSet key type mismatch: expected {}, found {}",
                                            expected_key.display_name(),
                                            key_type.display_name()
                                        ),
                                    )
                                    .with_help(help)
                                    .build()
                                    .with_label("type mismatch"),
                            );
                        }
                        if !self.is_assignable_with_traits(&value_type, &expected_value) {
                            let help =
                                suggestions::suggest_type_mismatch(&expected_value, &value_type)
                                    .unwrap_or_else(|| {
                                        format!(
                                            "expected {}, found {}",
                                            expected_value.display_name(),
                                            value_type.display_name()
                                        )
                                    });
                            self.diagnostics.push(
                                error_codes::TYPE_ERROR
                                    .emit(call.args[2].span())
                                    .arg(
                                        "detail",
                                        format!(
                                            "mapSet value type mismatch: expected {}, found {}",
                                            expected_value.display_name(),
                                            value_type.display_name()
                                        ),
                                    )
                                    .with_help(help)
                                    .build()
                                    .with_label("type mismatch"),
                            );
                        }
                    }

                    return map_type;
                }
                "mapGet" | "map_get" => {
                    if call.args.len() != 2 {
                        self.diagnostics.push(
                            error_codes::ARITY_MISMATCH
                                .emit(call.span)
                                .arg("name", "mapGet")
                                .arg("expected", "2")
                                .arg("found", format!("{}", call.args.len()))
                                .with_help("mapGet(map, key) requires exactly 2 arguments")
                                .build()
                                .with_label("argument count mismatch"),
                        );
                        return Type::Unknown;
                    }

                    let map_type = self.check_expr(&call.args[0]);
                    let key_type = self.check_expr(&call.args[1]);

                    let Some((expected_key, expected_value)) = self.hashmap_type_args(&map_type)
                    else {
                        let map_norm = map_type.normalized();
                        let map_is_any_or_unknown = matches!(map_norm, Type::Unknown)
                            || matches!(
                                map_norm,
                                Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM
                            );
                        let map_is_structural = matches!(map_norm, Type::Structural { .. });
                        if map_is_any_or_unknown || map_is_structural {
                            return Type::Generic {
                                name: "Option".to_string(),
                                type_args: vec![Type::any_placeholder()],
                            };
                        }
                        self.diagnostics.push(
                            error_codes::TYPE_ERROR
                                .emit(call.args[0].span())
                                .arg(
                                    "detail",
                                    format!(
                                        "mapGet expects Map for argument 1, found {}",
                                        map_type.display_name()
                                    ),
                                )
                                .with_help("argument 1 must be a HashMap")
                                .build()
                                .with_label("type mismatch"),
                        );
                        return Type::Unknown;
                    };

                    if !self.is_untyped_hashmap(&map_type)
                        && !self.is_assignable_with_traits(&key_type, &expected_key)
                    {
                        let help = suggestions::suggest_type_mismatch(&expected_key, &key_type)
                            .unwrap_or_else(|| {
                                format!(
                                    "expected {}, found {}",
                                    expected_key.display_name(),
                                    key_type.display_name()
                                )
                            });
                        self.diagnostics.push(
                            error_codes::TYPE_ERROR
                                .emit(call.args[1].span())
                                .arg(
                                    "detail",
                                    format!(
                                        "mapGet key type mismatch: expected {}, found {}",
                                        expected_key.display_name(),
                                        key_type.display_name()
                                    ),
                                )
                                .with_help(help)
                                .build()
                                .with_label("type mismatch"),
                        );
                    }

                    if self.is_untyped_hashmap(&map_type) {
                        return Type::Generic {
                            name: "Option".to_string(),
                            type_args: vec![Type::any_placeholder()],
                        };
                    }

                    return Type::Generic {
                        name: "Option".to_string(),
                        type_args: vec![expected_value],
                    };
                }
                // H-236: Array element-type checking for mutation/concat calls
                "arrayPush" | "array_push" | "arrayUnshift" | "array_unshift" => {
                    if call.args.len() == 2 {
                        let arr_type = self.check_expr(&call.args[0]);
                        let elem_type = self.check_expr(&call.args[1]);
                        if let Some(expected_elem) = self.array_elem_type(&arr_type) {
                            if !Self::is_untyped_collection_elem(&expected_elem)
                                && !self.is_assignable_with_traits(&elem_type, &expected_elem)
                            {
                                let method = name.as_str();
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR
                                        .emit(call.args[1].span())
                                        .arg(
                                            "detail",
                                            format!(
                                                "{method} element type mismatch: expected {}, found {}",
                                                expected_elem.display_name(),
                                                elem_type.display_name()
                                            ),
                                        )
                                        .with_help(format!(
                                            "array element type is {} — cannot push {}",
                                            expected_elem.display_name(),
                                            elem_type.display_name()
                                        ))
                                        .build()
                                        .with_label("type mismatch"),
                                );
                            }
                        }
                        return arr_type;
                    }
                }
                "arrayConcat" => {
                    if call.args.len() == 2 {
                        let arr_type = self.check_expr(&call.args[0]);
                        let other_type = self.check_expr(&call.args[1]);
                        if let Some(expected_elem) = self.array_elem_type(&arr_type) {
                            if !Self::is_untyped_collection_elem(&expected_elem) {
                                if let Some(other_elem) = self.array_elem_type(&other_type) {
                                    if !self.is_assignable_with_traits(&other_elem, &expected_elem)
                                    {
                                        self.diagnostics.push(
                                            error_codes::TYPE_ERROR
                                                .emit(call.args[1].span())
                                                .arg(
                                                    "detail",
                                                    format!(
                                                        "arrayConcat element type mismatch: expected {}[], found {}[]",
                                                        expected_elem.display_name(),
                                                        other_elem.display_name()
                                                    ),
                                                )
                                                .with_help(format!(
                                                    "both arrays must have element type {} to concat",
                                                    expected_elem.display_name()
                                                ))
                                                .build()
                                                .with_label("type mismatch"),
                                        );
                                    }
                                }
                            }
                        }
                        return arr_type;
                    }
                }
                // H-236: HashSet element-type checking
                "setAdd" | "set_add" | "setRemove" | "set_remove" => {
                    if call.args.len() == 2 {
                        let set_type = self.check_expr(&call.args[0]);
                        let elem_type = self.check_expr(&call.args[1]);
                        if let Some(expected_elem) = self.hashset_elem_type(&set_type) {
                            if !Self::is_untyped_collection_elem(&expected_elem)
                                && !self.is_assignable_with_traits(&elem_type, &expected_elem)
                            {
                                let method = name.as_str();
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR
                                        .emit(call.args[1].span())
                                        .arg(
                                            "detail",
                                            format!(
                                                "{method} element type mismatch: expected {}, found {}",
                                                expected_elem.display_name(),
                                                elem_type.display_name()
                                            ),
                                        )
                                        .with_help(format!(
                                            "set element type is {} — cannot add/remove {}",
                                            expected_elem.display_name(),
                                            elem_type.display_name()
                                        ))
                                        .build()
                                        .with_label("type mismatch"),
                                );
                            }
                        }
                        return set_type;
                    }
                }
                // H-236: Queue element-type checking
                "queueEnqueue" | "queue_enqueue" => {
                    if call.args.len() == 2 {
                        let queue_type = self.check_expr(&call.args[0]);
                        let elem_type = self.check_expr(&call.args[1]);
                        if let Some(expected_elem) = self.queue_elem_type(&queue_type) {
                            if !Self::is_untyped_collection_elem(&expected_elem)
                                && !self.is_assignable_with_traits(&elem_type, &expected_elem)
                            {
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR
                                        .emit(call.args[1].span())
                                        .arg(
                                            "detail",
                                            format!(
                                                "queueEnqueue element type mismatch: expected {}, found {}",
                                                expected_elem.display_name(),
                                                elem_type.display_name()
                                            ),
                                        )
                                        .with_help(format!(
                                            "queue element type is {} — cannot enqueue {}",
                                            expected_elem.display_name(),
                                            elem_type.display_name()
                                        ))
                                        .build()
                                        .with_label("type mismatch"),
                                );
                            }
                        }
                        return queue_type;
                    }
                }
                // H-236: Stack element-type checking
                "stackPush" | "stack_push" => {
                    if call.args.len() == 2 {
                        let stack_type = self.check_expr(&call.args[0]);
                        let elem_type = self.check_expr(&call.args[1]);
                        if let Some(expected_elem) = self.stack_elem_type(&stack_type) {
                            if !Self::is_untyped_collection_elem(&expected_elem)
                                && !self.is_assignable_with_traits(&elem_type, &expected_elem)
                            {
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR
                                        .emit(call.args[1].span())
                                        .arg(
                                            "detail",
                                            format!(
                                                "stackPush element type mismatch: expected {}, found {}",
                                                expected_elem.display_name(),
                                                elem_type.display_name()
                                            ),
                                        )
                                        .with_help(format!(
                                            "stack element type is {} — cannot push {}",
                                            expected_elem.display_name(),
                                            elem_type.display_name()
                                        ))
                                        .build()
                                        .with_label("type mismatch"),
                                );
                            }
                        }
                        return stack_type;
                    }
                }
                // Duration bare globals return Map<string, number>
                "durationFromSeconds"
                | "durationFromMinutes"
                | "durationFromHours"
                | "durationFromDays" => {
                    for arg in &call.args {
                        let _ = self.check_expr(arg);
                    }
                    return Type::Generic {
                        name: "Map".to_string(),
                        type_args: vec![Type::String, Type::Number],
                    };
                }
                // H-112: hashMapHas / hashSetHas return bool
                "mapHas" | "map_has" | "setHas" | "set_has" => {
                    return Type::Bool;
                }
                // H-276: Result/Option predicates return bool
                "isOk" | "is_ok" | "isErr" | "is_err" | "isSome" | "is_some" | "isNone"
                | "is_none" => {
                    // Consume the argument for side effects but return bool
                    for arg in &call.args {
                        let _ = self.check_expr(arg);
                    }
                    return Type::Bool;
                }
                // H-276: len() returns number
                "len" => {
                    for arg in &call.args {
                        let _ = self.check_expr(arg);
                    }
                    return Type::Number;
                }
                // H-276: typeof() returns string
                "typeof" | "type_of" => {
                    for arg in &call.args {
                        let _ = self.check_expr(arg);
                    }
                    return Type::String;
                }
                // H-164: unwrap() returns the inner type T from Option<T> or Result<T, E>
                "unwrap" | "expect" => {
                    if call.args.len() == 1 {
                        let arg_type = self.check_expr(&call.args[0]);
                        let inner = match arg_type.normalized() {
                            Type::Generic { name, type_args }
                                if (name == "Option" || name == "Result")
                                    && !type_args.is_empty() =>
                            {
                                type_args[0].clone()
                            }
                            _ => arg_type,
                        };
                        return inner;
                    }
                }
                _ => {}
            }
        }

        // Pre-evaluate arg types for ownership checking (avoids double-evaluation in check_expr
        // for the `shared` param path). check_call_against_signature re-evaluates independently.
        let arg_types_for_ownership: Vec<Type> = if callee_name.is_some() {
            call.args.iter().map(|a| self.check_expr(a)).collect()
        } else {
            Vec::new()
        };

        match &callee_norm {
            Type::Function {
                type_params,
                params,
                return_type,
            } => {
                if let Some(ref name) = callee_name {
                    self.check_call_ownership(call, name, &arg_types_for_ownership);
                }
                self.check_call_against_signature_with_types(
                    call,
                    &callee_type,
                    type_params,
                    params,
                    return_type,
                    &arg_types_for_ownership,
                )
            }
            Type::Union(members) => {
                if members.is_empty() {
                    return Type::Unknown;
                }

                let mut signature: Option<Type> = None;
                for member in members {
                    match member {
                        Type::Function { .. } => {
                            if signature.is_none() {
                                signature = Some(member.clone());
                            } else if signature.as_ref() != Some(member) {
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR.emit(call.span)
                                        .arg("detail", "Cannot call union of incompatible function signatures")
                                        .with_help("ensure all union members share the same function signature")
                                        .build()
                                        .with_label("ambiguous call"),
                                );
                                return Type::Unknown;
                            }
                        }
                        _ => {
                            self.diagnostics.push(
                                error_codes::NOT_CALLABLE
                                    .emit(call.span)
                                    .arg("expr", member.display_name())
                                    .arg("type_name", member.display_name())
                                    .with_help(suggestions::suggest_not_callable(&callee_type))
                                    .build()
                                    .with_label("not callable"),
                            );
                            return Type::Unknown;
                        }
                    }
                }

                if let Some(Type::Function {
                    type_params,
                    params,
                    return_type,
                }) = signature
                {
                    return self.check_call_against_signature(
                        call,
                        &callee_type,
                        &type_params,
                        &params,
                        &return_type,
                    );
                }

                Type::Unknown
            }
            Type::Unknown => {
                // Error recovery: still check arguments for side effects (usage tracking)
                // This ensures parameters referenced in arguments are marked as used
                for arg in &call.args {
                    self.check_expr(arg);
                }
                Type::any_placeholder()
            }
            // any-typed value is callable — skip signature checking, return any.
            // This covers: `let f: any = ...`, type aliases of `any` (e.g. `type Handler = any`),
            // and array elements of type `any[]`.
            t if Type::is_any_placeholder(t) => {
                for arg in &call.args {
                    self.check_expr(arg);
                }
                Type::any_placeholder()
            }
            _ => {
                self.diagnostics.push(
                    error_codes::NOT_CALLABLE
                        .emit(call.span)
                        .arg("expr", callee_type.display_name())
                        .arg("type_name", callee_type.display_name())
                        .with_help(suggestions::suggest_not_callable(&callee_type))
                        .build()
                        .with_label("not callable"),
                );
                Type::Unknown
            }
        }
    }

    /// Check a generic function call with type inference
    fn check_call_with_inference(
        &mut self,
        type_params: &[TypeParamDef],
        params: &[Type],
        return_type: &Type,
        call: &CallExpr,
    ) -> Type {
        use crate::typechecker::generics::TypeInferer;

        let mut inferer = TypeInferer::new();

        // Check each argument and try to infer type parameters
        for (i, arg) in call.args.iter().enumerate() {
            let arg_type = self.check_expr(arg);

            if let Some(param_type) = params.get(i) {
                // Try to unify parameter type with argument type
                if let Err(e) = inferer.unify(param_type, &arg_type) {
                    // Inference failed - report error
                    self.diagnostics.push(
                        error_codes::TYPE_ERROR.emit(arg.span())
                            .arg("detail", format!(
                                "Type inference failed: cannot match argument {} of type {} with parameter of type {}",
                                i + 1,
                                arg_type.display_name(),
                                param_type.display_name()
                            ))
                            .with_help(format!("Inference error: {:?}", e))
                            .build()
                            .with_label("type mismatch"),
                    );
                    return Type::Unknown;
                }
            }
        }

        // Check if all type parameters were inferred
        if !inferer.all_inferred(type_params) {
            // Some type parameters couldn't be inferred
            let uninferred: Vec<String> = type_params
                .iter()
                .filter(|param| inferer.get_substitution(&param.name).is_none())
                .map(|param| param.name.clone())
                .collect();

            for param_name in &uninferred {
                self.diagnostics.push(
                    error_codes::CANNOT_INFER_TYPE_ARG.emit(call.span)
                        .arg("name", param_name)
                        .with_help("This type parameter only appears in the return type or is unconstrained. Provide an explicit type argument: `func::<Type>(args)`")
                        .build()
                        .with_label("type argument cannot be inferred from call arguments"),
                );
            }
            return Type::Unknown;
        }

        // Apply substitutions to return type
        let inferred_return = inferer.apply_substitutions(return_type);

        // Validate constraints
        if !self.check_constraints(type_params, &inferer, call.span) {
            return Type::Unknown;
        }

        inferred_return
    }

    fn array_concat_result(&self, left: &Type, right: &Type) -> Option<Type> {
        let left_elem = self.array_elem_type_if_all_arrays(left)?;
        let right_elem = self.array_elem_type_if_all_arrays(right)?;

        if self.is_assignable_with_traits(&left_elem, &right_elem) {
            Some(Type::Array(Box::new(right_elem)))
        } else if self.is_assignable_with_traits(&right_elem, &left_elem) {
            Some(Type::Array(Box::new(left_elem)))
        } else {
            None
        }
    }

    fn array_elem_type_if_all_arrays(&self, ty: &Type) -> Option<Type> {
        match ty.normalized() {
            Type::Array(elem) => Some(*elem),
            Type::Union(members) => {
                let mut element_types = Vec::with_capacity(members.len());
                for member in members {
                    match member.normalized() {
                        Type::Array(elem) => element_types.push(*elem),
                        _ => return None,
                    }
                }
                if element_types.is_empty() {
                    None
                } else {
                    Some(Type::union(element_types))
                }
            }
            _ => None,
        }
    }

    fn all_union_pairs_valid<F>(&self, left: &Type, right: &Type, mut predicate: F) -> bool
    where
        F: FnMut(&Type, &Type) -> bool,
    {
        let left_members = self.union_members(left);
        let right_members = self.union_members(right);

        for l in &left_members {
            for r in &right_members {
                if matches!(l.normalized(), Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM)
                    || matches!(r.normalized(), Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM)
                {
                    continue;
                }
                if l.normalized() == Type::Unknown || r.normalized() == Type::Unknown {
                    return false;
                }
                if !predicate(l, r) {
                    return false;
                }
            }
        }
        true
    }

    fn types_overlap(&self, left: &Type, right: &Type) -> bool {
        let left_members = self.union_members(left);
        let right_members = self.union_members(right);

        for l in &left_members {
            for r in &right_members {
                if matches!(l.normalized(), Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM)
                    || matches!(r.normalized(), Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM)
                {
                    return true;
                }
                if l.normalized() == Type::Unknown || r.normalized() == Type::Unknown {
                    return false;
                }
                if self.is_assignable_with_traits(l, r) || self.is_assignable_with_traits(r, l) {
                    return true;
                }
            }
        }
        false
    }

    fn union_members(&self, ty: &Type) -> Vec<Type> {
        match ty.normalized() {
            Type::Union(members) => members,
            other => vec![other],
        }
    }

    fn check_structural_method_call(
        &mut self,
        method_name: &str,
        members: &[StructuralMemberType],
        member: &MemberExpr,
    ) -> Option<Type> {
        let required = members.iter().find(|m| m.name == method_name)?;
        let Type::Function {
            params,
            return_type,
            ..
        } = &required.ty
        else {
            self.diagnostics.push(
                error_codes::INVALID_INDEX_TYPE.emit(member.member.span)
                    .arg("index_type", method_name)
                    .arg("detail", format!("type '{}' has no method '{}'", required.ty.display_name(), method_name))
                    .with_help(format!("type '{}' does not support method '{}'", required.ty.display_name(), method_name))
                    .with_note(format!(
                        "trait constraint `{}` requires this method — check that the correct trait is implemented for `{}`",
                        method_name,
                        required.ty.display_name()
                    ))
                    .build()
                    .with_label("method not found"),
            );
            return Some(Type::Unknown);
        };

        let provided_args = member.args.as_ref().map(|args| args.len()).unwrap_or(0);
        let expected_args = params.len();
        if provided_args != expected_args {
            self.diagnostics.push(
                error_codes::ARITY_MISMATCH
                    .emit(member.span)
                    .arg("name", method_name)
                    .arg("expected", format!("{}", expected_args))
                    .arg("found", format!("{}", provided_args))
                    .with_help(format!(
                        "method '{}' requires exactly {} argument{}",
                        method_name,
                        expected_args,
                        if expected_args == 1 { "" } else { "s" }
                    ))
                    .build()
                    .with_label("argument count mismatch"),
            );
        }

        if let Some(args) = &member.args {
            for (i, arg) in args.iter().enumerate() {
                let arg_type = self.check_expr(arg);
                if let Some(expected_type) = params.get(i) {
                    // Unknown expected type means the method accepts any argument
                    // (e.g. callback-based array methods: arr.map, arr.filter, etc.)
                    if expected_type.normalized() == crate::typechecker::Type::Unknown {
                        continue;
                    }
                    if !self.is_assignable_with_traits(&arg_type, expected_type) {
                        self.diagnostics.push(
                            error_codes::TYPE_ERROR
                                .emit(arg.span())
                                .arg(
                                    "detail",
                                    format!(
                                        "Argument {} has wrong type: expected {}, found {}",
                                        i + 1,
                                        expected_type.display_name(),
                                        arg_type.display_name()
                                    ),
                                )
                                .with_help(format!(
                                    "argument {} must be of type {}",
                                    i + 1,
                                    expected_type.display_name()
                                ))
                                .build()
                                .with_label("type mismatch"),
                        );
                    }
                }
            }
        }

        Some(*return_type.clone())
    }

    fn check_structural_property_access(
        &mut self,
        member_name: &str,
        members: &[StructuralMemberType],
        member: &MemberExpr,
    ) -> Option<Type> {
        let required = members.iter().find(|m| m.name == member_name);
        let Some(required) = required else {
            let available: Vec<&str> = members.iter().map(|m| m.name.as_str()).collect();
            let similar = crate::typechecker::suggestions::find_similar_name(
                member_name,
                available.iter().copied(),
            );
            let mut diag = error_codes::INVALID_INDEX_TYPE
                .emit(member.member.span)
                .arg("index_type", member_name)
                .arg("detail", format!("type has no member '{}'", member_name))
                .with_help(format!(
                    "check that '{}' exists on this record or namespace",
                    member_name
                ))
                .build()
                .with_label("member not found");
            if let Some(name) = similar {
                diag = diag.with_suggestion_rename_noted(
                    format!("did you mean `{}`?", name),
                    member_name,
                    name,
                    format!("`{}` exists on this namespace or record type", name),
                );
            }
            self.diagnostics.push(diag);
            return Some(Type::Unknown);
        };

        Some(required.ty.clone())
    }

    fn comparable_operand_type(&mut self, ty: &Type) -> Type {
        ty.normalized()
    }

    /// Check a member expression (method call)
    fn check_member(&mut self, member: &MemberExpr) -> Type {
        // Fast-path: static namespace identifiers (Json, Math, Env).
        // These are not registered in the symbol table — detect by identifier name.
        if let crate::ast::Expr::Identifier(id) = member.target.as_ref() {
            if let Some(ns_tag) = crate::method_dispatch::namespace_type_tag(&id.name) {
                member.type_tag.set(Some(ns_tag));

                // H-293: Json.parse<T>() returns Result<T, string> instead of Result<JsonValue, string>
                let return_type = if id.name.eq_ignore_ascii_case("json")
                    && member.member.name == "parse"
                    && !member.type_args.is_empty()
                {
                    // Resolve the type argument
                    let type_arg = self.resolve_type_ref(&member.type_args[0]);
                    // Validate that it's a struct type (structural type)
                    if !matches!(type_arg, Type::Structural { .. }) {
                        self.diagnostics.push(
                            error_codes::TYPE_ERROR
                                .emit(member.type_args[0].span())
                                .arg(
                                    "detail",
                                    format!(
                                        "Json.parse<T> requires T to be a struct type, found {}",
                                        type_arg.display_name()
                                    ),
                                )
                                .with_help("use a struct type: Json.parse<User>(str)")
                                .build()
                                .with_label("expected struct type"),
                        );
                    }
                    // Return Result<T, string>
                    Type::Generic {
                        name: "Result".to_string(),
                        type_args: vec![type_arg, Type::String],
                    }
                } else {
                    // Standard namespace return type
                    resolve_namespace_return_type(&id.name, &member.member.name)
                };

                // D-010: Type::Unknown is always an error state, never a silent wildcard.
                // If a namespace method has no type entry, emit a diagnostic immediately.
                if return_type == Type::Unknown {
                    self.diagnostics.push(
                        error_codes::NAMESPACE_METHOD_NO_RETURN_TYPE
                            .emit(member.span)
                            .arg("namespace", &id.name)
                            .arg("method", &member.member.name)
                            .build()
                            .with_label("untyped namespace method"),
                    );
                }

                // H-243: check arity and argument types for namespace method calls.
                // Previously, namespace early-return silently ignored all arguments.
                let ns_name = id.name.clone();
                let method_name_str = member.member.name.clone();
                if let Some(args) = &member.args {
                    if let Some(param_types) =
                        resolve_namespace_param_types(&ns_name, &method_name_str)
                    {
                        if args.len() != param_types.len() {
                            self.diagnostics.push(
                                error_codes::ARITY_MISMATCH
                                    .emit(member.span)
                                    .arg("name", format!("{}.{}", ns_name, method_name_str))
                                    .arg("expected", format!("{}", param_types.len()))
                                    .arg("found", format!("{}", args.len()))
                                    .with_help(format!(
                                        "{}.{}() requires exactly {} argument{}",
                                        ns_name,
                                        method_name_str,
                                        param_types.len(),
                                        if param_types.len() == 1 { "" } else { "s" }
                                    ))
                                    .build()
                                    .with_label("argument count mismatch"),
                            );
                        }
                        for (i, (arg, expected_ty)) in
                            args.iter().zip(param_types.iter()).enumerate()
                        {
                            let arg_ty = self.check_expr(arg);
                            if arg_ty.normalized() == Type::Unknown {
                                // Upstream error already reported — skip cascade.
                                continue;
                            }
                            if !self.is_assignable_with_traits(&arg_ty, expected_ty) {
                                let help = suggestions::suggest_type_mismatch(expected_ty, &arg_ty)
                                    .unwrap_or_else(|| {
                                        format!(
                                            "argument {} must be of type {}",
                                            i + 1,
                                            expected_ty.display_name()
                                        )
                                    });
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR
                                        .emit(arg.span())
                                        .arg(
                                            "detail",
                                            format!(
                                                "Argument {} type mismatch: expected {}, found {}",
                                                i + 1,
                                                expected_ty.display_name(),
                                                arg_ty.display_name()
                                            ),
                                        )
                                        .with_help(help)
                                        .build()
                                        .with_label("type mismatch"),
                                );
                            }
                        }
                    } else {
                        // No param types registered — evaluate args for side effects only
                        // (usage tracking, inner diagnostics).
                        for arg in args {
                            self.check_expr(arg);
                        }
                    }
                }

                return return_type;
            }

            // Check for static method calls: Type.staticMethod()
            // The identifier must be a known struct type with a static method of the given name.
            if self.struct_decls.contains_key(&id.name) {
                let type_name = id.name.clone();
                let method_name = member.member.name.clone();

                if let Some(static_method) = self
                    .static_methods_registry
                    .get(&(type_name.clone(), method_name.clone()))
                    .cloned()
                {
                    // Mark this as a static dispatch
                    *member.static_dispatch.borrow_mut() = Some(type_name.clone());

                    // Resolve return type
                    let return_type = self.resolve_type_ref(&static_method.return_type);

                    // Check argument count and types
                    if let Some(args) = &member.args {
                        let expected_count = static_method.params.len();
                        if args.len() != expected_count {
                            self.diagnostics.push(
                                error_codes::ARITY_MISMATCH
                                    .emit(member.span)
                                    .arg("name", format!("{}.{}", type_name, method_name))
                                    .arg("expected", format!("{}", expected_count))
                                    .arg("found", format!("{}", args.len()))
                                    .with_help(format!(
                                        "static method '{}.{}' requires {} argument{}",
                                        type_name,
                                        method_name,
                                        expected_count,
                                        if expected_count == 1 { "" } else { "s" }
                                    ))
                                    .build()
                                    .with_label("argument count mismatch"),
                            );
                        }

                        // Type-check each argument
                        for (i, arg) in args.iter().enumerate() {
                            let arg_type = self.check_expr(arg);
                            if let Some(param) = static_method.params.get(i) {
                                let expected_type = self.resolve_type_ref(&param.type_ref);
                                if !self.is_assignable_with_traits(&arg_type, &expected_type) {
                                    self.diagnostics.push(
                                        error_codes::TYPE_MISMATCH
                                            .emit(arg.span())
                                            .arg("expected", expected_type.display_name())
                                            .arg("found", arg_type.display_name())
                                            .with_help(format!(
                                                "argument {} has wrong type: expected {}, found {}",
                                                i + 1,
                                                expected_type.display_name(),
                                                arg_type.display_name()
                                            ))
                                            .build()
                                            .with_label("type mismatch"),
                                    );
                                }
                            }
                        }
                    } else if !static_method.params.is_empty() {
                        // No arguments provided but method expects some
                        self.diagnostics.push(
                            error_codes::ARITY_MISMATCH
                                .emit(member.span)
                                .arg("name", format!("{}.{}", type_name, method_name))
                                .arg("expected", format!("{}", static_method.params.len()))
                                .arg("found", "0")
                                .build()
                                .with_label("missing arguments"),
                        );
                    }

                    return return_type;
                }
            }
        }

        // Type-check the target expression
        let target_type = self.check_expr(&member.target);

        // Annotate MemberExpr with TypeTag for method dispatch parity
        let type_tag = match target_type.normalized() {
            Type::JsonValue => Some(crate::method_dispatch::TypeTag::JsonValue),
            Type::Array(_) => Some(crate::method_dispatch::TypeTag::Array),
            Type::String => Some(crate::method_dispatch::TypeTag::String),
            // H-260: primitive instance methods (D-021 TypeScript parity)
            Type::Number => Some(crate::method_dispatch::TypeTag::Number),
            Type::Bool => Some(crate::method_dispatch::TypeTag::Bool),
            Type::Generic { ref name, .. } if name == "Map" => {
                Some(crate::method_dispatch::TypeTag::Map)
            }
            Type::Generic { ref name, .. } if name == "Set" => {
                Some(crate::method_dispatch::TypeTag::Set)
            }
            Type::Generic { ref name, .. } if name == "Queue" => {
                Some(crate::method_dispatch::TypeTag::Queue)
            }
            Type::Generic { ref name, .. } if name == "Stack" => {
                Some(crate::method_dispatch::TypeTag::Stack)
            }
            Type::Generic { ref name, .. } if name == "Option" => {
                Some(crate::method_dispatch::TypeTag::Option)
            }
            Type::Generic { ref name, .. } if name == "Result" => {
                Some(crate::method_dispatch::TypeTag::Result)
            }
            // H-231: instance method dispatch for DateTime, Regex, HttpResponse
            Type::Generic { ref name, .. } if name == "DateTime" => {
                Some(crate::method_dispatch::TypeTag::DateTime)
            }
            Type::Generic { ref name, .. } if name == "Regex" => {
                Some(crate::method_dispatch::TypeTag::RegexValue)
            }
            Type::Generic { ref name, .. } if name == "HttpResponse" => {
                Some(crate::method_dispatch::TypeTag::HttpResponse)
            }
            Type::Generic { ref name, .. } if name == "ProcessOutput" => {
                Some(crate::method_dispatch::TypeTag::ProcessOutput)
            }
            Type::Generic { ref name, .. } if name == "SqliteConnection" => {
                Some(crate::method_dispatch::TypeTag::SqliteConnection)
            }
            Type::Generic { ref name, .. } if name == "Future" => {
                Some(crate::method_dispatch::TypeTag::FutureValue)
            }
            Type::Generic { ref name, .. } if name == "AtomicValue" => {
                Some(crate::method_dispatch::TypeTag::AtomicValue)
            }
            Type::Generic { ref name, .. } if name == "RwLockValue" => {
                Some(crate::method_dispatch::TypeTag::RwLockValue)
            }
            Type::Generic { ref name, .. } if name == "SemaphoreValue" => {
                Some(crate::method_dispatch::TypeTag::SemaphoreValue)
            }
            _ => None,
        };
        member.type_tag.set(type_tag);

        // Unknown is error-recovery: propagate Unknown.
        if target_type.normalized() == Type::Unknown {
            return Type::Unknown;
        }
        // any-typed target: field/method access on `any` returns `any`.
        if Type::is_any_placeholder(&target_type.normalized()) {
            return Type::any_placeholder();
        }

        // Look up the method in the method table and clone the signature to avoid borrow issues
        let method_name = &member.member.name;
        let target_norm = target_type.normalized();

        // TypeTag-based instance methods: AtomicValue, RwLockValue, SemaphoreValue
        // These use runtime TypeTag dispatch — resolve return types statically here.
        if let Type::Generic { ref name, .. } = target_norm {
            let maybe_return = match name.as_str() {
                "AtomicValue" => match method_name.as_str() {
                    "get" | "add" | "sub" => Some(Type::Number),
                    "set" => Some(Type::Null),
                    "compareSwap" => Some(Type::Bool),
                    _ => None,
                },
                "RwLockValue" => match method_name.as_str() {
                    "read" | "tryRead" => Some(Type::Unknown), // value type not tracked statically
                    "write" => Some(Type::Null),
                    "tryWrite" => Some(Type::Bool),
                    _ => None,
                },
                "SemaphoreValue" => match method_name.as_str() {
                    "available" => Some(Type::Number),
                    "tryAcquire" => Some(Type::Bool),
                    "acquire" | "release" => Some(Type::Null),
                    _ => None,
                },
                _ => None,
            };
            if let Some(ret) = maybe_return {
                // Check args for side effects
                if let Some(args) = &member.args {
                    for arg in args {
                        self.check_expr(arg);
                    }
                }
                return ret;
            }
        }

        // Tuple element access: t.0, t.1, ...
        if member.args.is_none() {
            if let Type::Tuple(ref elem_types) = target_norm {
                if let Ok(idx) = method_name.parse::<usize>() {
                    if let Some(elem_ty) = elem_types.get(idx) {
                        return elem_ty.clone();
                    } else {
                        self.diagnostics.push(
                            error_codes::TYPE_ERROR
                                .emit(member.member.span)
                                .arg(
                                    "detail",
                                    format!(
                                        "tuple index {} out of range: tuple has {} element{}",
                                        idx,
                                        elem_types.len(),
                                        if elem_types.len() == 1 { "" } else { "s" }
                                    ),
                                )
                                .with_help(format!(
                                    "valid indices are 0..{}",
                                    elem_types.len().saturating_sub(1)
                                ))
                                .build()
                                .with_label("index out of range"),
                        );
                        return Type::Unknown;
                    }
                } else {
                    // Non-numeric member on tuple
                    self.diagnostics.push(
                        error_codes::TYPE_ERROR
                            .emit(member.member.span)
                            .arg(
                                "detail",
                                format!(
                                    "tuple has no field '{}': use .0, .1, ... for element access",
                                    method_name
                                ),
                            )
                            .with_help("tuple elements are accessed by numeric index: t.0, t.1")
                            .build()
                            .with_label("invalid tuple field"),
                    );
                    return Type::Unknown;
                }
            }
        }

        if member.args.is_none() {
            if let Type::Structural { members } = &target_norm {
                if let Some(return_type) =
                    self.check_structural_property_access(method_name, members, member)
                {
                    return return_type;
                }
            }
        }

        if let Type::Union(members) = target_norm {
            let mut return_types = Vec::new();
            let mut signatures = Vec::new();

            for member_ty in &members {
                if let Some(sig) = self.method_table.lookup(member_ty, method_name) {
                    signatures.push(sig.clone());
                    return_types.push(sig.return_type);
                } else {
                    self.diagnostics.push(
                        error_codes::INVALID_INDEX_TYPE
                            .emit(member.member.span)
                            .arg("index_type", method_name.as_str())
                            .arg(
                                "detail",
                                format!(
                                    "type '{}' has no method '{}'",
                                    member_ty.display_name(),
                                    method_name
                                ),
                            )
                            .with_help(format!(
                                "method '{}' must exist on all union members",
                                method_name
                            ))
                            .build()
                            .with_label("method not found"),
                    );
                    return Type::Unknown;
                }
            }

            if let Some(first_sig) = signatures.first() {
                let expected_args = first_sig.arg_types.len();
                let provided_args = member.args.as_ref().map(|args| args.len()).unwrap_or(0);

                if provided_args != expected_args {
                    self.diagnostics.push(
                        error_codes::ARITY_MISMATCH
                            .emit(member.span)
                            .arg("name", method_name.as_str())
                            .arg("expected", format!("{}", expected_args))
                            .arg("found", format!("{}", provided_args))
                            .with_help(format!(
                                "method '{}' requires exactly {} argument{}",
                                method_name,
                                expected_args,
                                if expected_args == 1 { "" } else { "s" }
                            ))
                            .build()
                            .with_label("argument count mismatch"),
                    );
                }

                if let Some(args) = &member.args {
                    for (i, arg) in args.iter().enumerate() {
                        let arg_type = self.check_expr(arg);
                        for sig in &signatures {
                            if let Some(expected_type) = sig.arg_types.get(i) {
                                // Unknown expected type means the method accepts any argument
                                if expected_type.normalized() == crate::typechecker::Type::Unknown {
                                    continue;
                                }
                                if !self.is_assignable_with_traits(&arg_type, expected_type) {
                                    self.diagnostics.push(
                                        error_codes::TYPE_ERROR
                                            .emit(arg.span())
                                            .arg(
                                                "detail",
                                                format!(
                                                "Argument {} has wrong type: expected {}, found {}",
                                                i + 1,
                                                expected_type.display_name(),
                                                arg_type.display_name()
                                            ),
                                            )
                                            .with_help(format!(
                                                "argument {} must be of type {}",
                                                i + 1,
                                                expected_type.display_name()
                                            ))
                                            .build()
                                            .with_label("type mismatch"),
                                    );
                                    return Type::Unknown;
                                }
                            }
                        }
                    }
                }

                return Type::union(return_types);
            }

            // Empty union — degenerate state, emit diagnostic (D-010: Unknown is error state).
            self.diagnostics.push(
                error_codes::TYPE_ERROR
                    .emit(member.span)
                    .arg(
                        "detail",
                        format!("cannot call '{}' on empty union type", member.member.name),
                    )
                    .with_help(
                        "ensure the expression has a concrete type before calling methods on it",
                    )
                    .build()
                    .with_label("empty union type"),
            );
            return Type::Unknown;
        }

        if member.args.is_some() {
            if let Type::Structural { ref members } = target_norm {
                if let Some(return_type) =
                    self.check_structural_method_call(method_name, members, member)
                {
                    return return_type;
                }
            }
        }

        if let Type::TraitObject { name: trait_name } = &target_norm {
            if let Some(methods) = self.trait_registry.get_methods(trait_name) {
                if let Some(method_sig) = methods.iter().find(|m| m.name == *method_name).cloned() {
                    let param_types = method_sig.param_types.clone();
                    let return_type = method_sig.return_type.clone();
                    let expected_args = param_types.len();
                    let provided_args = member.args.as_ref().map(|args| args.len()).unwrap_or(0);
                    if provided_args != expected_args {
                        self.diagnostics.push(
                            error_codes::ARITY_MISMATCH
                                .emit(member.span)
                                .arg("name", method_name.as_str())
                                .arg("expected", format!("{}", expected_args))
                                .arg("found", format!("{}", provided_args))
                                .with_help(format!(
                                    "method '{}' requires exactly {} argument{}",
                                    method_name,
                                    expected_args,
                                    if expected_args == 1 { "" } else { "s" }
                                ))
                                .build()
                                .with_label("argument count mismatch"),
                        );
                    }

                    if let Some(args) = &member.args {
                        for (i, arg) in args.iter().enumerate() {
                            let arg_type = self.check_expr(arg);
                            if let Some(expected_type) = param_types.get(i) {
                                // Unknown expected type means the method accepts any argument
                                if expected_type.normalized() == crate::typechecker::Type::Unknown {
                                    continue;
                                }
                                if !self.is_assignable_with_traits(&arg_type, expected_type) {
                                    self.diagnostics.push(
                                        error_codes::TYPE_ERROR
                                            .emit(arg.span())
                                            .arg(
                                                "detail",
                                                format!(
                                                "Argument {} has wrong type: expected {}, found {}",
                                                i + 1,
                                                expected_type.display_name(),
                                                arg_type.display_name()
                                            ),
                                            )
                                            .with_help(format!(
                                                "argument {} must be of type {}",
                                                i + 1,
                                                expected_type.display_name()
                                            ))
                                            .build()
                                            .with_label("type mismatch"),
                                    );
                                    return Type::Unknown;
                                }
                            }
                        }
                    }

                    *member.trait_dispatch.borrow_mut() = Some((String::new(), trait_name.clone()));
                    return return_type;
                } else if member.args.is_some() {
                    self.diagnostics.push(
                        error_codes::INVALID_INDEX_TYPE.emit(member.member.span)
                            .arg("index_type", method_name.as_str())
                            .arg("detail", format!("trait '{}' has no method '{}'", trait_name, method_name))
                            .with_help(format!(
                                "trait `{trait_name}` does not define a method `{method_name}` — check the trait definition for the correct method name"
                            ))
                            .with_note("if you intended to call an inherent method, remove the trait annotation from the `impl` block")
                            .build()
                            .with_label("method not found"),
                    );
                    return Type::Unknown;
                }
            }
        }

        // H-312: Type parameter with trait bounds — look up method from bound traits.
        // e.g., `fn foo<T extends Printable>(x: T) { x.to_str() }` should resolve `to_str` from Printable.
        if let Type::TypeParameter { ref name } = target_norm {
            // Clone trait bounds upfront to avoid holding immutable borrow during check_expr calls
            let type_param_bounds: Option<Vec<String>> = self
                .active_type_params
                .iter()
                .find(|p| &p.name == name)
                .map(|p| {
                    p.trait_bounds
                        .iter()
                        .map(|b| b.trait_name.clone())
                        .collect()
                });

            if let Some(bounds) = type_param_bounds {
                for trait_name in &bounds {
                    // Clone method entry to release borrow on trait_registry
                    let method_entry =
                        self.trait_registry
                            .get_methods(trait_name)
                            .and_then(|methods| {
                                methods.iter().find(|m| m.name == *method_name).cloned()
                            });

                    if let Some(method_entry) = method_entry {
                        // Found the method in a bound trait — validate args and return
                        let param_types = method_entry.param_types.clone();
                        let return_type = method_entry.return_type.clone();
                        let expected_args = param_types.len();
                        let provided_args =
                            member.args.as_ref().map(|args| args.len()).unwrap_or(0);

                        if provided_args != expected_args {
                            self.diagnostics.push(
                                error_codes::ARITY_MISMATCH
                                    .emit(member.span)
                                    .arg("name", method_name.as_str())
                                    .arg("expected", format!("{}", expected_args))
                                    .arg("found", format!("{}", provided_args))
                                    .with_help(format!(
                                        "method '{}' requires exactly {} argument{}",
                                        method_name,
                                        expected_args,
                                        if expected_args == 1 { "" } else { "s" }
                                    ))
                                    .build()
                                    .with_label("argument count mismatch"),
                            );
                        }

                        if let Some(args) = &member.args {
                            for (i, arg) in args.iter().enumerate() {
                                let arg_type = self.check_expr(arg);
                                if let Some(expected_type) = param_types.get(i) {
                                    if expected_type.normalized()
                                        == crate::typechecker::Type::Unknown
                                    {
                                        continue;
                                    }
                                    if !self.is_assignable_with_traits(&arg_type, expected_type) {
                                        self.diagnostics.push(
                                            error_codes::TYPE_ERROR
                                                .emit(arg.span())
                                                .arg(
                                                    "detail",
                                                    format!(
                                                        "Argument {} has wrong type: expected {}, found {}",
                                                        i + 1,
                                                        expected_type.display_name(),
                                                        arg_type.display_name()
                                                    ),
                                                )
                                                .with_help(format!(
                                                    "argument {} must be of type {}",
                                                    i + 1,
                                                    expected_type.display_name()
                                                ))
                                                .build()
                                                .with_label("type mismatch"),
                                        );
                                        return Type::Unknown;
                                    }
                                }
                            }
                        }

                        // Set trait_dispatch with empty type_name to signal dynamic dispatch.
                        // Type parameters require runtime resolution because the concrete
                        // type is not known at compile time (just like TraitObject).
                        *member.trait_dispatch.borrow_mut() =
                            Some((String::new(), trait_name.clone()));
                        return return_type;
                    }
                }

                // Type param has bounds but method not found in any bound trait
                if member.args.is_some() && !bounds.is_empty() {
                    self.diagnostics.push(
                        error_codes::INVALID_INDEX_TYPE
                            .emit(member.member.span)
                            .arg("index_type", method_name.as_str())
                            .arg(
                                "detail",
                                format!(
                                    "type parameter '{}' with bounds [{}] has no method '{}'",
                                    name,
                                    bounds.join(", "),
                                    method_name
                                ),
                            )
                            .with_help(format!(
                                "add a trait bound that defines '{}', or use a different method",
                                method_name
                            ))
                            .build()
                            .with_label("method not found on bounded type parameter"),
                    );
                    return Type::Unknown;
                }
            }
        }

        let method_sig = self.method_table.lookup(&target_type, method_name);

        if let Some(method_sig) = method_sig {
            // Check argument count
            let provided_args = member.args.as_ref().map(|args| args.len()).unwrap_or(0);
            let expected_args = method_sig.arg_types.len();

            // Trailing Unknown-typed args are treated as optional — allow fewer args provided.
            let min_required = {
                let mut min = expected_args;
                for t in method_sig.arg_types.iter().rev() {
                    if t.normalized() == Type::Unknown {
                        min = min.saturating_sub(1);
                    } else {
                        break;
                    }
                }
                min
            };
            if provided_args < min_required || provided_args > expected_args {
                self.diagnostics.push(
                    error_codes::ARITY_MISMATCH
                        .emit(member.span)
                        .arg("name", method_name.as_str())
                        .arg("expected", format!("{}", expected_args))
                        .arg("found", format!("{}", provided_args))
                        .with_help(format!(
                            "method '{}' requires exactly {} argument{}",
                            method_name,
                            expected_args,
                            if expected_args == 1 { "" } else { "s" }
                        ))
                        .build()
                        .with_label("argument count mismatch"),
                );
            }

            // Check argument types if present
            if let Some(args) = &member.args {
                for (i, arg) in args.iter().enumerate() {
                    let arg_type = self.check_expr(arg);
                    if let Some(expected_type) = method_sig.arg_types.get(i) {
                        // Unknown expected type means the method accepts any argument
                        // (e.g. callback-based array methods: arr.map, arr.filter, etc.)
                        if expected_type.normalized() == crate::typechecker::Type::Unknown {
                            continue;
                        }
                        if !self.is_assignable_with_traits(&arg_type, expected_type) {
                            self.diagnostics.push(
                                error_codes::TYPE_ERROR
                                    .emit(arg.span())
                                    .arg(
                                        "detail",
                                        format!(
                                            "Argument {} has wrong type: expected {}, found {}",
                                            i + 1,
                                            expected_type.display_name(),
                                            arg_type.display_name()
                                        ),
                                    )
                                    .with_help(format!(
                                        "argument {} must be of type {}",
                                        i + 1,
                                        expected_type.display_name()
                                    ))
                                    .build()
                                    .with_label("type mismatch"),
                            );
                        }
                    }
                }
            }

            // For callback-based ARRAY methods whose return type depends on the callback's
            // return type (map, flatMap), infer the element type from the callback arg.
            // Only applies to Array targets — Option/Result also have "map" but different semantics.
            let return_type = if matches!(method_name.as_str(), "map" | "flatMap")
                && matches!(target_norm, Type::Array(_))
            {
                if let Some(args) = &member.args {
                    if let Some(callback_arg) = args.first() {
                        let cb_type = self.check_expr(callback_arg);
                        if let Type::Function {
                            return_type: cb_ret,
                            ..
                        } = cb_type.normalized()
                        {
                            if cb_ret.normalized() != Type::Unknown {
                                if method_name == "flatMap" {
                                    match cb_ret.normalized() {
                                        Type::Array(inner) => Type::Array(inner),
                                        other => Type::Array(Box::new(other)),
                                    }
                                } else {
                                    Type::Array(cb_ret)
                                }
                            } else {
                                method_sig.return_type
                            }
                        } else {
                            method_sig.return_type
                        }
                    } else {
                        method_sig.return_type
                    }
                } else {
                    method_sig.return_type
                }
            } else {
                method_sig.return_type
            };
            return_type
        } else if let Some((return_type, type_name)) =
            self.resolve_inherent_method_call(&target_type, method_name)
        {
            // Slot 2a: inherent method dispatch (D-037: inherent takes priority over trait)
            // trait_dispatch uses empty string for trait_name to signal inherent dispatch.
            *member.trait_dispatch.borrow_mut() = Some((type_name, String::new()));

            if let Some(args) = &member.args {
                for arg in args.iter() {
                    let _ = self.check_expr(arg);
                }
            }
            return_type
        } else if let Some((return_type, type_name, trait_name)) =
            self.resolve_trait_method_call_with_info(&target_type, method_name)
        {
            // Slot 2b: trait method dispatch — found a matching impl
            // Annotate MemberExpr with dispatch info for compiler/interpreter
            *member.trait_dispatch.borrow_mut() = Some((type_name, trait_name));

            // Check args if present (non-self params only)
            if let Some(args) = &member.args {
                for arg in args.iter() {
                    let _ = self.check_expr(arg);
                }
            }
            return_type
        } else {
            // Check if a declared trait has this method — if so, emit AT3035 (not implemented)
            // rather than generic AT3010 (not found).
            let trait_name_with_method = self
                .trait_registry
                .find_trait_with_method(method_name)
                .map(|s| s.to_owned());

            if let Some(trait_name) = trait_name_with_method {
                let type_display = self.nominal_display_name(&target_type);
                self.diagnostics.push(
                    error_codes::TYPE_DOES_NOT_IMPLEMENT_TRAIT
                        .emit(member.member.span)
                        .arg("type_name", &type_display)
                        .arg("trait_name", &trait_name)
                        .with_help(format!(
                            "implement '{}' for '{}' with: impl {} for {} {{ ... }}",
                            trait_name, type_display, trait_name, type_display
                        ))
                        .build()
                        .with_label(format!("trait '{}' not implemented", trait_name)),
                );
            } else {
                // Method not found for this type (not a trait method either)
                let similar = self.method_suggestion_for(&target_type, method_name);
                let help = format!(
                    "type '{}' does not support method '{}'",
                    target_type.display_name(),
                    method_name
                );
                let mut diag = error_codes::INVALID_INDEX_TYPE
                    .emit(member.member.span)
                    .arg("index_type", method_name.as_str())
                    .arg(
                        "detail",
                        format!(
                            "type '{}' has no method '{}'",
                            target_type.display_name(),
                            method_name
                        ),
                    )
                    .with_help(help)
                    .build()
                    .with_label("method not found");
                if let Some(name) = similar {
                    diag = diag.with_suggestion_rename_noted(
                        format!("did you mean `{}`?", name),
                        method_name,
                        &name,
                        format!("`{}` is a method available on this type", name),
                    );
                }
                self.diagnostics.push(diag);
            }
            Type::Unknown
        }
    }

    /// Check an index expression
    fn check_index(&mut self, index: &IndexExpr) -> Type {
        let target_type = self.check_expr(&index.target);
        let target_norm = target_type.normalized();

        if matches!(target_norm, Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM) {
            let IndexValue::Single(expr) = &index.index;
            self.check_expr(expr);
            return Type::any_placeholder();
        }

        let IndexValue::Single(index_expr) = &index.index;
        let index_type = self.check_expr(index_expr);
        let index_norm = index_type.normalized();

        let index_is_range = index_norm == Type::Range;

        match target_norm {
            Type::Array(elem_type) => {
                if index_is_range {
                    Type::Array(elem_type)
                } else {
                    if index_norm != Type::Number {
                        self.diagnostics.push(
                            error_codes::TYPE_ERROR
                                .emit(index_expr.span())
                                .arg(
                                    "detail",
                                    format!(
                                        "Array index must be number, found {}",
                                        index_type.display_name()
                                    ),
                                )
                                .with_help("array indices must be numbers")
                                .build()
                                .with_label("type mismatch"),
                        );
                    }
                    *elem_type
                }
            }
            Type::JsonValue => {
                if index_is_range {
                    self.diagnostics.push(
                        error_codes::TYPE_ERROR
                            .emit(index_expr.span())
                            .arg("detail", "Range indices are only valid for arrays")
                            .with_help("only arrays can be sliced with ranges")
                            .build()
                            .with_label("not indexable"),
                    );
                    Type::Unknown
                } else {
                    if index_norm != Type::String && index_norm != Type::Number {
                        self.diagnostics.push(
                            error_codes::TYPE_ERROR
                                .emit(index_expr.span())
                                .arg(
                                    "detail",
                                    format!(
                                        "JSON index must be string or number, found {}",
                                        index_type.display_name()
                                    ),
                                )
                                .with_help(
                                    "use a string key or numeric index to access JSON values",
                                )
                                .build()
                                .with_label("type mismatch"),
                        );
                    }
                    Type::JsonValue
                }
            }
            Type::String => {
                if index_is_range {
                    self.diagnostics.push(
                        error_codes::TYPE_ERROR
                            .emit(index_expr.span())
                            .arg("detail", "Range indices are only valid for arrays")
                            .with_help("only arrays can be sliced with ranges")
                            .build()
                            .with_label("not indexable"),
                    );
                    Type::Unknown
                } else {
                    if index_norm != Type::Number {
                        self.diagnostics.push(
                            error_codes::TYPE_ERROR
                                .emit(index_expr.span())
                                .arg(
                                    "detail",
                                    format!(
                                        "String index must be number, found {}",
                                        index_type.display_name()
                                    ),
                                )
                                .with_help("string indices must be numbers")
                                .build()
                                .with_label("type mismatch"),
                        );
                    }
                    Type::String
                }
            }
            Type::Union(members) => {
                let mut result_types = Vec::new();
                for member in members {
                    match member {
                        Type::Array(elem_type) => {
                            if index_is_range {
                                result_types.push(Type::Array(elem_type));
                            } else {
                                if index_norm != Type::Number {
                                    self.diagnostics.push(
                                        error_codes::TYPE_ERROR
                                            .emit(index_expr.span())
                                            .arg(
                                                "detail",
                                                format!(
                                                    "Array index must be number, found {}",
                                                    index_type.display_name()
                                                ),
                                            )
                                            .with_help("array indices must be numbers")
                                            .build()
                                            .with_label("type mismatch"),
                                    );
                                }
                                result_types.push(*elem_type);
                            }
                        }
                        Type::JsonValue => {
                            if index_is_range {
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR
                                        .emit(index_expr.span())
                                        .arg("detail", "Range indices are only valid for arrays")
                                        .with_help("only arrays can be sliced with ranges")
                                        .build()
                                        .with_label("not indexable"),
                                );
                                return Type::Unknown;
                            }
                            if index_norm != Type::String && index_norm != Type::Number {
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR.emit(index_expr.span())
                                        .arg("detail", format!("JSON index must be string or number, found {}", index_type.display_name()))
                                        .with_help("use a string key or numeric index to access JSON values")
                                        .build()
                                        .with_label("type mismatch"),
                                );
                            }
                            result_types.push(Type::JsonValue);
                        }
                        Type::String => {
                            if index_is_range {
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR
                                        .emit(index_expr.span())
                                        .arg("detail", "Range indices are only valid for arrays")
                                        .with_help("only arrays can be sliced with ranges")
                                        .build()
                                        .with_label("not indexable"),
                                );
                                return Type::Unknown;
                            }
                            if index_norm != Type::Number {
                                self.diagnostics.push(
                                    error_codes::TYPE_ERROR
                                        .emit(index_expr.span())
                                        .arg(
                                            "detail",
                                            format!(
                                                "String index must be number, found {}",
                                                index_type.display_name()
                                            ),
                                        )
                                        .with_help("string indices must be numbers")
                                        .build()
                                        .with_label("type mismatch"),
                                );
                            }
                            result_types.push(Type::String);
                        }
                        _ => {
                            self.diagnostics.push(
                                error_codes::TYPE_ERROR
                                    .emit(index.target.span())
                                    .arg(
                                        "detail",
                                        format!("Cannot index into type {}", member.display_name()),
                                    )
                                    .with_help(
                                        "only arrays, strings, and json values can be indexed",
                                    )
                                    .build()
                                    .with_label("not indexable"),
                            );
                            return Type::Unknown;
                        }
                    }
                }
                Type::union(result_types)
            }
            Type::Unknown => Type::Unknown,
            _ => {
                self.diagnostics.push(
                    error_codes::TYPE_ERROR
                        .emit(index.target.span())
                        .arg(
                            "detail",
                            format!("Cannot index into type {}", target_type.display_name()),
                        )
                        .with_help("only arrays, strings, and json values can be indexed")
                        .build()
                        .with_label("not indexable"),
                );
                Type::Unknown
            }
        }
    }

    /// Check an array literal
    fn check_array_literal(&mut self, arr: &ArrayLiteral) -> Type {
        if arr.elements.is_empty() {
            // Empty array - element type is unknown until constrained
            return Type::Array(Box::new(Type::Unknown));
        }

        // Check first element to determine array type
        let first_type = self.check_expr(&arr.elements[0]);

        // Check that all elements have the same type
        for (i, elem) in arr.elements.iter().enumerate().skip(1) {
            let elem_type = self.check_expr(elem);
            if !self.is_assignable_with_traits(&elem_type, &first_type) {
                self.diagnostics.push(
                    error_codes::TYPE_ERROR
                        .emit(elem.span())
                        .arg(
                            "detail",
                            format!(
                                "Array element {} has wrong type: expected {}, found {}",
                                i,
                                first_type.display_name(),
                                elem_type.display_name()
                            ),
                        )
                        .with_help(format!(
                            "all array elements must be type {} (inferred from first element)",
                            first_type.display_name()
                        ))
                        .build()
                        .with_label("type mismatch"),
                );
            }
        }

        Type::Array(Box::new(first_type))
    }

    /// Check a match expression
    fn check_match(&mut self, match_expr: &crate::ast::MatchExpr) -> Type {
        // 1. Check scrutinee type
        let scrutinee_type = self.check_expr(&match_expr.scrutinee);

        if scrutinee_type.normalized() == Type::Unknown {
            // Error in scrutinee, skip match checking
            return Type::Unknown;
        }

        // 2. Check each arm and collect result types
        let mut arm_types = Vec::new();

        for (arm_idx, arm) in match_expr.arms.iter().enumerate() {
            // Check pattern against scrutinee type
            let pattern_bindings = self.check_pattern(&arm.pattern, &scrutinee_type);

            // Enter a new scope for pattern bindings
            self.symbol_table.enter_scope();

            // Add pattern bindings to symbol table for this arm's scope
            for (var_name, var_type, var_span) in &pattern_bindings {
                let symbol = crate::symbol::Symbol {
                    name: var_name.clone(),
                    ty: var_type.clone(),
                    mutable: false, // Pattern bindings are immutable
                    kind: crate::symbol::SymbolKind::Variable,
                    span: *var_span,
                    exported: false,
                    visibility: crate::ast::Visibility::Private,
                };
                // Ignore if binding fails (duplicate names in pattern - will be caught separately)
                let _ = self.symbol_table.define(symbol);
            }

            // Check guard if present — must be bool (AT3029)
            if let Some(guard) = &arm.guard {
                let guard_type = self.check_expr(guard);
                if guard_type.normalized() != Type::Bool {
                    self.diagnostics.push(
                        error_codes::TYPE_ERROR
                            .emit(guard.span())
                            .arg(
                                "detail",
                                format!(
                                    "Guard expression must be bool, found {}",
                                    guard_type.display_name()
                                ),
                            )
                            .with_help("guard expressions must evaluate to a boolean value")
                            .build()
                            .with_label("must be bool"),
                    );
                }
            }

            // Check arm body with bindings in scope
            let arm_type = self.check_expr(&arm.body);
            arm_types.push((arm_type.clone(), arm.body.span(), arm_idx));

            // Exit scope (removes pattern bindings)
            self.symbol_table.exit_scope();
        }

        // 3. Ensure all arms return compatible types
        if arm_types.is_empty() {
            // Empty match (parser should prevent this, but handle gracefully)
            self.diagnostics.push(
                error_codes::MATCH_EMPTY
                    .emit(match_expr.span)
                    .build()
                    .with_label("empty match"),
            );
            return Type::Unknown;
        }

        let mut unified = arm_types[0].0.clone();
        for (arm_type, arm_span, _arm_idx) in &arm_types[1..] {
            if let Some(lub) = crate::typechecker::inference::least_upper_bound(&unified, arm_type)
            {
                unified = lub;
            } else {
                self.diagnostics.push(
                    error_codes::MATCH_ARM_TYPE_MISMATCH
                        .emit(*arm_span)
                        .arg("found", arm_type.display_name())
                        .arg("expected", unified.display_name())
                        .with_help(format!(
                            "all match arms must return compatible types (current: {})",
                            unified.display_name()
                        ))
                        .build()
                        .with_label("type mismatch"),
                );
            }
        }

        // 4. Check exhaustiveness
        self.check_exhaustiveness(&match_expr.arms, &scrutinee_type, match_expr.span);

        // 5. Return the unified type
        unified
    }

    /// Check if a pattern covers a given constructor name (including inside OR patterns)
    fn pattern_covers_constructor(pattern: &crate::ast::Pattern, ctor: &str) -> bool {
        use crate::ast::Pattern;
        match pattern {
            Pattern::Constructor { name, .. } => name.name == ctor,
            Pattern::Or(alternatives, _) => alternatives
                .iter()
                .any(|alt| Self::pattern_covers_constructor(alt, ctor)),
            _ => false,
        }
    }

    /// Check if a pattern covers a given bool literal (including inside OR patterns)
    fn pattern_covers_bool(pattern: &crate::ast::Pattern, val: bool) -> bool {
        use crate::ast::{Literal, Pattern};
        match pattern {
            Pattern::Literal(Literal::Bool(b), _) => *b == val,
            Pattern::Or(alternatives, _) => alternatives
                .iter()
                .any(|alt| Self::pattern_covers_bool(alt, val)),
            _ => false,
        }
    }

    /// Check if a pattern is a catch-all (wildcard or unguarded variable, including inside OR)
    fn pattern_is_catch_all(pattern: &crate::ast::Pattern) -> bool {
        use crate::ast::Pattern;
        match pattern {
            Pattern::Wildcard(_) | Pattern::Variable(_) => true,
            Pattern::Or(alternatives, _) => alternatives.iter().any(Self::pattern_is_catch_all),
            _ => false,
        }
    }

    /// Check exhaustiveness of match arms
    fn check_exhaustiveness(
        &mut self,
        arms: &[crate::ast::MatchArm],
        scrutinee_type: &Type,
        match_span: Span,
    ) {
        // Check if there's a catch-all pattern (wildcard or variable binding, unguarded)
        let has_catch_all = arms
            .iter()
            .any(|arm| arm.guard.is_none() && Self::pattern_is_catch_all(&arm.pattern));

        if has_catch_all {
            // Wildcard or variable catches everything - exhaustive
            return;
        }

        // Check exhaustiveness based on scrutinee type
        let scrutinee_norm = scrutinee_type.normalized();
        if let Type::Union(members) = scrutinee_norm {
            for member in members {
                self.check_exhaustiveness(arms, &member, match_span);
            }
            return;
        }

        match scrutinee_norm {
            Type::Generic { name, .. } if name == "Option" => {
                // Option<T> requires Some and None to be covered
                let has_some = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_constructor(&arm.pattern, "Some")
                });

                let has_none = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_constructor(&arm.pattern, "None")
                });

                if !has_some || !has_none {
                    let missing = if !has_some && !has_none {
                        "Some(_), None".to_string()
                    } else if !has_some {
                        "Some(_)".to_string()
                    } else {
                        "None".to_string()
                    };

                    self.diagnostics.push(
                        error_codes::NON_EXHAUSTIVE_MATCH
                            .emit(match_span)
                            .arg("missing", &missing)
                            .with_help(format!("Add arm: {} => ...", missing))
                            .build()
                            .with_label("non-exhaustive"),
                    );
                }
            }

            Type::Generic { name, .. } if name == "Result" => {
                // Result<T,E> requires Ok and Err to be covered
                let has_ok = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_constructor(&arm.pattern, "Ok")
                });

                let has_err = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_constructor(&arm.pattern, "Err")
                });

                if !has_ok || !has_err {
                    let missing = if !has_ok && !has_err {
                        "Ok(_), Err(_)".to_string()
                    } else if !has_ok {
                        "Ok(_)".to_string()
                    } else {
                        "Err(_)".to_string()
                    };

                    self.diagnostics.push(
                        error_codes::NON_EXHAUSTIVE_MATCH
                            .emit(match_span)
                            .arg("missing", &missing)
                            .with_help(format!("Add arm: {} => ...", missing))
                            .build()
                            .with_label("non-exhaustive"),
                    );
                }
            }

            Type::Bool => {
                // Bool requires true and false to be covered (or wildcard)
                let has_true = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_bool(&arm.pattern, true)
                });
                let has_false = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_bool(&arm.pattern, false)
                });

                if !has_true || !has_false {
                    let missing = if !has_true && !has_false {
                        "true, false".to_string()
                    } else if !has_true {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    };

                    self.diagnostics.push(
                        error_codes::NON_EXHAUSTIVE_MATCH
                            .emit(match_span)
                            .arg("missing", &missing)
                            .with_help(format!("Add arm: {} => ... or use wildcard _", missing))
                            .build()
                            .with_label("non-exhaustive"),
                    );
                }
            }

            Type::Number | Type::String | Type::Array(_) | Type::Null => {
                // These types have infinite values - require wildcard
                self.diagnostics.push(
                    error_codes::NON_EXHAUSTIVE_MATCH
                        .emit(match_span)
                        .arg(
                            "missing",
                            format!("wildcard for {}", scrutinee_type.display_name()),
                        )
                        .with_help("Add wildcard pattern: _ => ...")
                        .build()
                        .with_label("non-exhaustive"),
                );
            }

            // H-230: user-defined enum — check that every declared variant is covered
            Type::Generic { ref name, .. } => {
                // Clone to avoid borrow conflict with self below
                let variant_names: Option<Vec<String>> =
                    self.enum_decls.get(name.as_str()).map(|decl| {
                        decl.variants
                            .iter()
                            .map(|v| v.name().name.clone())
                            .collect()
                    });

                if let Some(variant_names) = variant_names {
                    let missing: Vec<String> = variant_names
                        .iter()
                        .filter(|vname| {
                            !arms.iter().any(|arm| {
                                arm.guard.is_none()
                                    && Self::pattern_covers_enum_variant(&arm.pattern, vname)
                            })
                        })
                        .cloned()
                        .collect();

                    if !missing.is_empty() {
                        let missing_str = missing.join(", ");
                        self.diagnostics.push(
                            error_codes::NON_EXHAUSTIVE_MATCH
                                .emit(match_span)
                                .arg("missing", &missing_str)
                                .with_help(format!(
                                    "add arm{} for: {}",
                                    if missing.len() == 1 { "" } else { "s" },
                                    missing
                                        .iter()
                                        .map(|v| format!("{} => ...", v))
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                ))
                                .build()
                                .with_label("non-exhaustive match"),
                        );
                    }
                }
                // If enum_name is not in enum_decls, we can't check — skip silently
            }

            _ => {
                // For other types, warn but don't error (conservative approach)
            }
        }
    }

    /// Check if a pattern covers a specific enum variant name (by variant name only, not enum name).
    /// Handles EnumVariant (qualified), BareVariant (bare), Constructor (legacy/builtin), and OR.
    fn pattern_covers_enum_variant(pattern: &crate::ast::Pattern, variant_name: &str) -> bool {
        use crate::ast::Pattern;
        match pattern {
            Pattern::EnumVariant {
                variant_name: vn, ..
            } => vn.name == variant_name,
            Pattern::BareVariant { name, .. } => name.name == variant_name,
            Pattern::Constructor { name, .. } => name.name == variant_name,
            Pattern::Or(alternatives, _) => alternatives
                .iter()
                .any(|alt| Self::pattern_covers_enum_variant(alt, variant_name)),
            _ => false,
        }
    }

    /// Check a pattern and return variable bindings (name, type, span)
    fn check_pattern(
        &mut self,
        pattern: &Pattern,
        expected_type: &Type,
    ) -> Vec<(String, Type, Span)> {
        let mut bindings = Vec::new();
        let expected_norm = expected_type.normalized();

        if let Type::Union(members) = expected_norm {
            match pattern {
                Pattern::Wildcard(_) => return bindings,
                Pattern::Variable(id) => {
                    bindings.push((id.name.clone(), Type::Union(members), id.span));
                    return bindings;
                }
                Pattern::Literal(lit, span) => {
                    let lit_type = match lit {
                        Literal::Number(_) => Type::Number,
                        Literal::String(_) => Type::String,
                        Literal::Bool(_) => Type::Bool,
                        Literal::Null => Type::Null,
                    };
                    if !members
                        .iter()
                        .any(|member| self.is_assignable_with_traits(&lit_type, member))
                    {
                        self.diagnostics.push(
                            error_codes::PATTERN_TYPE_MISMATCH
                                .emit(*span)
                                .arg("value_type", Type::Union(members).display_name())
                                .arg("pattern_type", lit_type.display_name())
                                .with_help("use a matching literal or wildcard pattern")
                                .build()
                                .with_label("type mismatch"),
                        );
                    }
                    return bindings;
                }
                Pattern::Constructor { name, args, span } => {
                    let ctor_name = name.name.as_str();
                    let target_member = members.iter().find(|member| match member.normalized() {
                        Type::Generic { name, .. }
                            if ctor_name == "Some" || ctor_name == "None" =>
                        {
                            name == "Option"
                        }
                        Type::Generic { name, .. } if ctor_name == "Ok" || ctor_name == "Err" => {
                            name == "Result"
                        }
                        _ => false,
                    });

                    if let Some(member) = target_member {
                        return self.check_constructor_pattern(name, args, member, *span);
                    }

                    self.diagnostics.push(
                        error_codes::PATTERN_TYPE_MISMATCH
                            .emit(*span)
                            .arg("value_type", Type::Union(members).display_name())
                            .arg("pattern_type", format!("constructor {}", name.name))
                            .with_help("use a matching constructor or wildcard pattern")
                            .build()
                            .with_label("type mismatch"),
                    );
                    return bindings;
                }
                Pattern::Array { elements, span } => {
                    for member in &members {
                        if matches!(member.normalized(), Type::Array(_)) {
                            return self.check_array_pattern(elements, member, *span);
                        }
                    }
                    self.diagnostics.push(
                        error_codes::PATTERN_TYPE_MISMATCH
                            .emit(*span)
                            .arg("value_type", Type::Union(members).display_name())
                            .arg("pattern_type", "array pattern")
                            .with_help("use a matching array pattern or wildcard")
                            .build()
                            .with_label("type mismatch"),
                    );
                    return bindings;
                }
                Pattern::Tuple { elements, span } => {
                    for pat in elements {
                        bindings.extend(self.check_pattern(pat, &Type::Unknown));
                    }
                    let _ = span;
                    return bindings;
                }
                Pattern::Or(alternatives, _) => {
                    // Check each sub-pattern independently; bindings from first sub-pattern used
                    for alt in alternatives {
                        let alt_bindings = self.check_pattern(alt, &Type::Union(members.clone()));
                        if bindings.is_empty() {
                            bindings = alt_bindings;
                        }
                    }
                    return bindings;
                }
                Pattern::EnumVariant {
                    enum_name,
                    variant_name,
                    args,
                    ..
                } => {
                    // H-120: look up variant field types so bindings get proper types
                    let field_types =
                        self.enum_variant_field_types(&enum_name.name, &variant_name.name);
                    for (i, arg) in args.iter().enumerate() {
                        let field_ty = field_types
                            .get(i)
                            .map(|tr| self.resolve_type_ref(tr))
                            .unwrap_or(Type::Unknown);
                        bindings.extend(self.check_pattern(arg, &field_ty));
                    }
                    return bindings;
                }
                Pattern::BareVariant { name, args, .. } => {
                    // H-223: bare variant — look up fields by variant name across all enums
                    let field_types = self.bare_variant_field_types(&name.name);
                    for (i, arg) in args.iter().enumerate() {
                        let field_ty = field_types
                            .get(i)
                            .map(|tr| self.resolve_type_ref(tr))
                            .unwrap_or(Type::Unknown);
                        bindings.extend(self.check_pattern(arg, &field_ty));
                    }
                    return bindings;
                }
                Pattern::Struct { fields, .. } => {
                    // Struct pattern inside union: bind each field as Unknown for now.
                    for field in fields {
                        match &field.pattern {
                            Some(sub) => bindings.extend(self.check_pattern(sub, &Type::Unknown)),
                            None => bindings.push((
                                field.name.name.clone(),
                                Type::Unknown,
                                field.name.span,
                            )),
                        }
                    }
                    return bindings;
                }
            }
        }

        match pattern {
            Pattern::Literal(lit, span) => {
                // Check literal type matches expected type
                let lit_type = match lit {
                    Literal::Number(_) => Type::Number,
                    Literal::String(_) => Type::String,
                    Literal::Bool(_) => Type::Bool,
                    Literal::Null => Type::Null,
                };

                if !self.is_assignable_with_traits(&lit_type, &expected_norm) {
                    self.diagnostics.push(
                        error_codes::PATTERN_TYPE_MISMATCH
                            .emit(*span)
                            .arg("value_type", expected_norm.display_name())
                            .arg("pattern_type", lit_type.display_name())
                            .with_help(format!(
                                "use a {} literal or wildcard pattern",
                                expected_norm.display_name()
                            ))
                            .build()
                            .with_label("type mismatch"),
                    );
                }
            }

            Pattern::Wildcard(_) => {
                // Wildcard matches anything, no bindings
            }

            Pattern::Variable(id) => {
                // Variable binding - binds the entire scrutinee value
                bindings.push((id.name.clone(), expected_norm.clone(), id.span));
            }

            Pattern::Constructor { name, args, span } => {
                // Check constructor pattern (Ok, Err, Some, None)
                bindings.extend(self.check_constructor_pattern(name, args, &expected_norm, *span));
            }

            Pattern::Array { elements, span } => {
                // Check array pattern
                bindings.extend(self.check_array_pattern(elements, &expected_norm, *span));
            }

            Pattern::Tuple { elements, span } => {
                let elem_types: Vec<Type> = match &expected_norm {
                    Type::Tuple(elems) => elems.clone(),
                    _ => (0..elements.len()).map(|_| Type::Unknown).collect(),
                };
                for (pat, ty) in elements
                    .iter()
                    .zip(elem_types.iter().chain(std::iter::repeat(&Type::Unknown)))
                {
                    bindings.extend(self.check_pattern(pat, ty));
                }
                let _ = span;
            }

            Pattern::Or(alternatives, _) => {
                // Check each sub-pattern independently; bindings from first sub-pattern used
                for alt in alternatives {
                    let alt_bindings = self.check_pattern(alt, expected_type);
                    if bindings.is_empty() {
                        bindings = alt_bindings;
                    }
                }
            }

            Pattern::EnumVariant {
                enum_name,
                variant_name,
                args,
                ..
            } => {
                // H-120: look up variant field types so bindings get proper types
                let field_types =
                    self.enum_variant_field_types(&enum_name.name, &variant_name.name);
                for (i, arg) in args.iter().enumerate() {
                    let field_ty = field_types
                        .get(i)
                        .map(|tr| self.resolve_type_ref(tr))
                        .unwrap_or(Type::Unknown);
                    bindings.extend(self.check_pattern(arg, &field_ty));
                }
            }

            Pattern::BareVariant { name, args, .. } => {
                // H-223: bare variant — look up fields by variant name across all enums
                let field_types = self.bare_variant_field_types(&name.name);
                for (i, arg) in args.iter().enumerate() {
                    let field_ty = field_types
                        .get(i)
                        .map(|tr| self.resolve_type_ref(tr))
                        .unwrap_or(Type::Unknown);
                    bindings.extend(self.check_pattern(arg, &field_ty));
                }
            }

            Pattern::Struct { fields, .. } => {
                // Bind each field. For shorthand `{ x }`, bind x with Unknown type;
                // for explicit `{ x: p }`, recurse into sub-pattern with Unknown type.
                // Full struct type resolution is left to a future typechecker pass.
                for field in fields {
                    match &field.pattern {
                        Some(sub) => bindings.extend(self.check_pattern(sub, &Type::Unknown)),
                        None => {
                            bindings.push((field.name.name.clone(), Type::Unknown, field.name.span))
                        }
                    }
                }
            }
        }

        bindings
    }

    /// Check constructor pattern (Ok, Err, Some, None)
    fn check_constructor_pattern(
        &mut self,
        name: &Identifier,
        args: &[Pattern],
        expected_type: &Type,
        span: Span,
    ) -> Vec<(String, Type, Span)> {
        let mut bindings = Vec::new();
        let expected_norm = expected_type.normalized();

        match expected_norm {
            Type::Generic {
                name: type_name,
                type_args,
            } => {
                match type_name.as_str() {
                    "Option" if type_args.len() == 1 => {
                        // Option<T> has constructors: Some(T), None
                        match name.name.as_str() {
                            "Some" => {
                                if args.len() != 1 {
                                    self.diagnostics.push(
                                        error_codes::CONSTRUCTOR_ARITY
                                            .emit(span)
                                            .arg("type_name", "Some")
                                            .arg("expected", "1")
                                            .arg("found", format!("{}", args.len()))
                                            .with_help(
                                                "Some requires exactly 1 argument: Some(value)",
                                            )
                                            .build()
                                            .with_label("wrong arity"),
                                    );
                                } else {
                                    // Check inner pattern against T
                                    bindings.extend(self.check_pattern(&args[0], &type_args[0]));
                                }
                            }
                            "None" => {
                                if !args.is_empty() {
                                    self.diagnostics.push(
                                        error_codes::CONSTRUCTOR_ARITY
                                            .emit(span)
                                            .arg("type_name", "None")
                                            .arg("expected", "0")
                                            .arg("found", format!("{}", args.len()))
                                            .with_help("None requires no arguments: None")
                                            .build()
                                            .with_label("wrong arity"),
                                    );
                                }
                            }
                            _ => {
                                self.diagnostics.push(
                                    error_codes::UNKNOWN_CONSTRUCTOR
                                        .emit(name.span)
                                        .arg("name", &name.name)
                                        .with_help(
                                            "Option only has constructors: Some(value) and None",
                                        )
                                        .build()
                                        .with_label("unknown constructor"),
                                );
                            }
                        }
                    }
                    "Result" if type_args.len() == 2 => {
                        // Result<T, E> has constructors: Ok(T), Err(E)
                        match name.name.as_str() {
                            "Ok" => {
                                if args.len() != 1 {
                                    self.diagnostics.push(
                                        error_codes::CONSTRUCTOR_ARITY
                                            .emit(span)
                                            .arg("type_name", "Ok")
                                            .arg("expected", "1")
                                            .arg("found", format!("{}", args.len()))
                                            .with_help("Ok requires exactly 1 argument: Ok(value)")
                                            .build()
                                            .with_label("wrong arity"),
                                    );
                                } else {
                                    // Check inner pattern against T
                                    bindings.extend(self.check_pattern(&args[0], &type_args[0]));
                                }
                            }
                            "Err" => {
                                if args.len() != 1 {
                                    self.diagnostics.push(
                                        error_codes::CONSTRUCTOR_ARITY
                                            .emit(span)
                                            .arg("type_name", "Err")
                                            .arg("expected", "1")
                                            .arg("found", format!("{}", args.len()))
                                            .with_help(
                                                "Err requires exactly 1 argument: Err(error)",
                                            )
                                            .build()
                                            .with_label("wrong arity"),
                                    );
                                } else {
                                    // Check inner pattern against E
                                    bindings.extend(self.check_pattern(&args[0], &type_args[1]));
                                }
                            }
                            _ => {
                                self.diagnostics.push(
                                    error_codes::UNKNOWN_CONSTRUCTOR.emit(name.span)
                                        .arg("name", &name.name)
                                        .with_help("Result only has constructors: Ok(value) and Err(error)")
                                        .build()
                                        .with_label("unknown constructor"),
                                );
                            }
                        }
                    }
                    _ if self.enum_names.contains(type_name.as_str()) => {
                        // User-defined enum: look up the variant and bind its payload args.
                        let enum_decl = self.enum_decls.get(type_name.as_str()).cloned();
                        if let Some(decl) = enum_decl {
                            let variant = decl
                                .variants
                                .iter()
                                .find(|v| v.name().name == name.name)
                                .cloned();
                            match variant {
                                Some(crate::ast::EnumVariant::Unit { .. }) => {
                                    // Unit variant: no payload
                                    if !args.is_empty() {
                                        self.diagnostics.push(
                                            error_codes::CONSTRUCTOR_ARITY
                                                .emit(span)
                                                .arg("type_name", &name.name)
                                                .arg("expected", "0")
                                                .arg("found", format!("{}", args.len()))
                                                .with_help(format!(
                                                    "{} is a unit variant and takes no arguments",
                                                    name.name
                                                ))
                                                .build()
                                                .with_label("wrong arity"),
                                        );
                                    }
                                }
                                Some(crate::ast::EnumVariant::Tuple { fields, .. }) => {
                                    // Tuple variant: bind each field
                                    if args.len() != fields.len() {
                                        self.diagnostics.push(
                                            error_codes::CONSTRUCTOR_ARITY
                                                .emit(span)
                                                .arg("type_name", &name.name)
                                                .arg("expected", format!("{}", fields.len()))
                                                .arg("found", format!("{}", args.len()))
                                                .with_help(format!(
                                                    "{} requires {} argument(s)",
                                                    name.name,
                                                    fields.len()
                                                ))
                                                .build()
                                                .with_label("wrong arity"),
                                        );
                                    } else {
                                        for (arg, field_type_ref) in args.iter().zip(fields.iter())
                                        {
                                            let field_ty = self.resolve_type_ref(field_type_ref);
                                            bindings.extend(self.check_pattern(arg, &field_ty));
                                        }
                                    }
                                }
                                Some(crate::ast::EnumVariant::Struct { .. }) => {
                                    // Struct variant: not yet supported in patterns
                                    // Accept without binding for now to avoid false positives
                                }
                                None => {
                                    self.diagnostics.push(
                                        error_codes::UNKNOWN_CONSTRUCTOR
                                            .emit(name.span)
                                            .arg("name", &name.name)
                                            .with_help(format!(
                                                "{} is not a variant of {}",
                                                name.name, type_name
                                            ))
                                            .build()
                                            .with_label("unknown variant"),
                                    );
                                }
                            }
                        }
                    }
                    _ => {
                        self.diagnostics.push(
                            error_codes::UNSUPPORTED_PATTERN_TYPE
                                .emit(span)
                                .arg("pattern_type", expected_type.display_name())
                                .with_help(
                                    "constructor patterns only work with Option and Result types",
                                )
                                .build()
                                .with_label("unsupported type"),
                        );
                    }
                }
            }
            _ => {
                self.diagnostics.push(
                    error_codes::UNSUPPORTED_PATTERN_TYPE
                        .emit(span)
                        .arg("pattern_type", expected_type.display_name())
                        .with_help("constructor patterns only work with Option and Result types")
                        .build()
                        .with_label("unsupported type"),
                );
            }
        }

        bindings
    }

    /// Check array pattern
    fn check_array_pattern(
        &mut self,
        elements: &[Pattern],
        expected_type: &Type,
        span: Span,
    ) -> Vec<(String, Type, Span)> {
        let mut bindings = Vec::new();
        let expected_norm = expected_type.normalized();

        match expected_norm {
            Type::Array(elem_type) => {
                // Check each pattern element against the array element type
                for pattern in elements {
                    bindings.extend(self.check_pattern(pattern, &elem_type));
                }
            }
            _ => {
                self.diagnostics.push(
                    error_codes::ARRAY_PATTERN_TYPE_MISMATCH
                        .emit(span)
                        .arg("expected", "array")
                        .arg("found", expected_type.display_name())
                        .with_help("array patterns can only match array types")
                        .build()
                        .with_label("type mismatch"),
                );
            }
        }

        bindings
    }

    /// Check try expression (error propagation operator ?)
    fn check_try(&mut self, try_expr: &TryExpr) -> Type {
        use crate::ast::TryTargetKind;

        // Type check the expression being tried
        let expr_type = self.check_expr(&try_expr.expr);
        let expr_norm = expr_type.normalized();

        // Skip if expression type is unknown (error already reported)
        if expr_norm == Type::Unknown {
            return Type::Unknown;
        }

        // Expression must be Result<T, E> or Option<T>
        enum TrySource {
            Result { ok_type: Type, err_type: Type },
            Option { inner_type: Type },
        }

        let source = match &expr_norm {
            Type::Generic { name, type_args } if name == "Result" && type_args.len() == 2 => {
                TrySource::Result {
                    ok_type: type_args[0].clone(),
                    err_type: type_args[1].clone(),
                }
            }
            Type::Generic { name, type_args } if name == "Option" && type_args.len() == 1 => {
                TrySource::Option {
                    inner_type: type_args[0].clone(),
                }
            }
            _ => {
                self.diagnostics.push(
                    error_codes::TYPE_ERROR.emit(try_expr.span)
                        .arg("detail", format!("? operator requires Result<T, E> or Option<T> type, found {}", expr_type.display_name()))
                        .with_help("the ? operator can only be applied to Result<T, E> or Option<T> values")
                        .build()
                        .with_label("not a Result or Option type"),
                );
                return Type::Unknown;
            }
        };

        // At top-level (script/REPL context), ? is allowed without a function.
        // Propagation semantics: Ok(v) → v, Err(e) → early eval termination.
        let is_top_level = self.current_function_return_type.is_none();

        match source {
            TrySource::Result { ok_type, err_type } => {
                // Annotate for compiler
                *try_expr.target_kind.borrow_mut() = Some(TryTargetKind::Result);

                if is_top_level {
                    return ok_type;
                }

                let function_return_type = self.current_function_return_type.clone().unwrap();
                let function_return_norm = function_return_type.normalized();

                // Function must return Result<T', E'>
                match &function_return_norm {
                    Type::Generic { name, type_args }
                        if name == "Result" && type_args.len() == 2 =>
                    {
                        let function_err_type = &type_args[1];

                        let err_norm = err_type.normalized();
                        let function_err_norm = function_err_type.normalized();
                        let err_is_any = matches!(
                            err_norm,
                            Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM
                        );
                        let function_err_is_any = matches!(
                            function_err_norm,
                            Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM
                        );

                        // Error types must be compatible (any-placeholder is always compatible)
                        if !err_is_any && !function_err_is_any && err_norm != function_err_norm {
                            self.diagnostics.push(
                                error_codes::TYPE_ERROR.emit(try_expr.span)
                                    .arg("detail", format!(
                                        "? operator error type mismatch: expression has error type {}, but function returns {}",
                                        err_type.display_name(),
                                        function_err_type.display_name()
                                    ))
                                    .with_help(format!(
                                        "convert the error type to {} or change the function's error type",
                                        function_err_type.display_name()
                                    ))
                                    .build()
                                    .with_label("error type mismatch"),
                            );
                        }

                        ok_type
                    }
                    _ => {
                        // Allow `?` as an unwrap in non-Result functions (runtime error on Err).
                        ok_type
                    }
                }
            }
            TrySource::Option { inner_type } => {
                // Annotate for compiler
                *try_expr.target_kind.borrow_mut() = Some(TryTargetKind::Option);

                if is_top_level {
                    return inner_type;
                }

                let function_return_type = self.current_function_return_type.clone().unwrap();
                let function_return_norm = function_return_type.normalized();

                // Function must return Option<T'>
                match &function_return_norm {
                    Type::Generic { name, type_args }
                        if name == "Option" && type_args.len() == 1 =>
                    {
                        inner_type
                    }
                    _ => {
                        // Allow `?` as an unwrap in non-Option functions (runtime error on None).
                        inner_type
                    }
                }
            }
        }
    }

    /// Typecheck an anonymous function expression.
    ///
    /// Resolves param types (any for untyped arrow-fn params — Block 5 infers them),
    /// checks the body in a new scope, validates capture semantics, and returns
    /// `Type::Function { params, return_type }`.
    pub(super) fn check_anon_fn(
        &mut self,
        params: &[crate::ast::Param],
        return_type_ref: Option<&crate::ast::TypeRef>,
        body: &Expr,
        span: Span,
    ) -> Type {
        // Resolve param types — None type_ref → any (inferred later in Block 5)
        let param_types: Vec<Type> = params
            .iter()
            .map(|p| self.resolve_type_ref(&p.type_ref))
            .collect();

        // Resolve declared return type (if present)
        let declared_return = return_type_ref.map(|t| self.resolve_type_ref(t));

        // Save and update function context so return statements inside the closure
        // are valid and checked against the declared return type.
        let prev_return_type = self.current_function_return_type.clone();
        let prev_function_info = self.current_function_info.clone();
        self.current_function_return_type =
            Some(declared_return.clone().unwrap_or(Type::any_placeholder()));
        self.current_function_info = Some(("<closure>".to_string(), span));

        // Enter a new scope for the closure body
        self.enter_scope();

        // Define params as locals in the closure scope
        for (param, ty) in params.iter().zip(param_types.iter()) {
            let symbol = crate::symbol::Symbol {
                name: param.name.name.clone(),
                ty: ty.clone(),
                mutable: false,
                kind: crate::symbol::SymbolKind::Parameter,
                span: param.name.span,
                exported: false,
                visibility: crate::ast::Visibility::Private,
            };
            let _ = self.symbol_table.define(symbol);
        }

        // Validate capture semantics for identifiers referenced in the body
        self.check_capture_semantics(body, span);

        // Check the body — for block bodies, extract the last expr type if present
        let body_type = match body {
            Expr::Block(block) => {
                for stmt in &block.statements {
                    self.check_statement(stmt);
                }
                // Infer type from the tail expression or last statement:
                // - Tail expr (bare expression at end of block, no semicolon) → its type
                // - Bare expr statement → use its type
                // - Return statement → use the returned expr type (check_statement already validated it)
                // - Anything else → Void
                if let Some(tail) = &block.tail_expr {
                    self.check_expr(tail)
                } else {
                    block
                        .statements
                        .last()
                        .and_then(|s| match s {
                            crate::ast::Stmt::Expr(e) => Some(self.check_expr(&e.expr)),
                            crate::ast::Stmt::Return(r) => r
                                .value
                                .as_ref()
                                .map(|e| self.check_expr(e))
                                .or(Some(Type::Void)),
                            _ => None,
                        })
                        .unwrap_or(Type::Void)
                }
            }
            _ => self.check_expr(body),
        };

        self.exit_scope();

        // Restore function context
        self.current_function_return_type = prev_return_type;
        self.current_function_info = prev_function_info;

        let return_type = match declared_return {
            Some(declared) => {
                if !self.is_assignable_with_traits(&body_type, &declared) {
                    self.diagnostics.push(
                        error_codes::TYPE_ERROR
                            .emit(span)
                            .arg(
                                "detail",
                                format!(
                                    "closure body returns {} but declared return type is {}",
                                    body_type.display_name(),
                                    declared.display_name()
                                ),
                            )
                            .build()
                            .with_label("return type mismatch"),
                    );
                }
                declared
            }
            None => body_type,
        };

        Type::Function {
            type_params: vec![],
            params: param_types,
            return_type: Box::new(return_type),
        }
    }

    /// Walk a closure body expression and emit diagnostics for invalid captures.
    ///
    /// Rules:
    /// - Copy types: captured by copy — always valid
    /// - Non-Copy types: captured by move — valid, caller loses ownership
    /// - `borrow`-annotated variables: **error** — borrows cannot outlive their scope
    fn check_capture_semantics(&mut self, expr: &Expr, closure_span: Span) {
        match expr {
            Expr::Identifier(id) => {
                // Check if this identifier is a borrow-annotated param in the enclosing fn
                if let Some(ownership) = self.current_fn_param_ownerships.get(&id.name) {
                    if matches!(ownership, Some(crate::ast::OwnershipAnnotation::Borrow)) {
                        self.diagnostics.push(
                            error_codes::CLOSURE_CAPTURES_BORROW.emit(closure_span)
                                .arg("name", &id.name)
                                .with_help(format!(
                                    "cannot capture `borrow` parameter `{}` in a closure — borrows cannot outlive their scope.\n\
                                     Fix: change the parameter annotation to `own` (moved in) or `share` (shared ref),\n\
                                     or pass a copy of the value into the closure explicitly.",
                                    id.name
                                ))
                                .build()
                                .with_label("borrow captured here"),
                        );
                    }
                }
            }
            Expr::Binary(b) => {
                self.check_capture_semantics(&b.left, closure_span);
                self.check_capture_semantics(&b.right, closure_span);
            }
            Expr::Unary(u) => self.check_capture_semantics(&u.expr, closure_span),
            Expr::Call(c) => {
                self.check_capture_semantics(&c.callee, closure_span);
                for arg in &c.args {
                    self.check_capture_semantics(arg, closure_span);
                }
            }
            Expr::TemplateString { parts, .. } => {
                for part in parts {
                    if let TemplatePart::Expression(expr) = part {
                        self.check_capture_semantics(expr, closure_span);
                    }
                }
            }
            Expr::Group(g) => self.check_capture_semantics(&g.expr, closure_span),
            Expr::Block(block) => {
                for stmt in &block.statements {
                    self.check_capture_semantics_stmt(stmt, closure_span);
                }
            }
            Expr::AnonFn { body, .. } => {
                self.check_capture_semantics(body, closure_span);
            }
            _ => {}
        }
    }

    fn check_capture_semantics_stmt(&mut self, stmt: &crate::ast::Stmt, closure_span: Span) {
        match stmt {
            crate::ast::Stmt::Expr(e) => {
                self.check_capture_semantics(&e.expr, closure_span);
            }
            crate::ast::Stmt::Return(r) => {
                if let Some(val) = &r.value {
                    self.check_capture_semantics(val, closure_span);
                }
            }
            crate::ast::Stmt::VarDecl(v) => {
                self.check_capture_semantics(&v.init, closure_span);
            }
            _ => {}
        }
    }
}
