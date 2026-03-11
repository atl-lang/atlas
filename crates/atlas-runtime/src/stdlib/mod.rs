//! Standard library functions

pub mod array;
#[cfg(feature = "http")]
pub mod async_io;
pub mod async_primitives;
pub mod collections;
pub mod compression;
pub mod console;
pub mod datetime;
pub mod fs;
pub mod future;
#[cfg(feature = "http")]
pub mod http;
pub mod io;
pub mod json;
pub mod math;
pub mod path;
pub mod process;
pub mod reflect;
pub mod regex;
pub mod string;
pub mod test;
pub mod types;

// Systems-level stdlib modules
pub mod crypto;
pub mod encoding;
#[cfg(feature = "http")]
pub mod net;
pub mod sync;
#[cfg(feature = "http")]
pub mod websocket;

use crate::security::SecurityContext;
use crate::value::{RuntimeError, Value};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

/// Shared, thread-safe output writer.
/// Default implementation writes to stdout.
pub type OutputWriter = Arc<Mutex<Box<dyn Write + Send>>>;

/// Construct a writer that goes to real stdout (the default).
pub fn stdout_writer() -> OutputWriter {
    Arc::new(Mutex::new(Box::new(std::io::stdout())))
}

/// A builtin dispatch function: takes args, span, security, output → Result<Value, RuntimeError>
type BuiltinFn =
    fn(&[Value], crate::span::Span, &SecurityContext, &OutputWriter) -> Result<Value, RuntimeError>;

/// Look up the documented signature for a stdlib function.
/// Returns `None` for functions without a registered signature.
fn stdlib_signature(func_name: &str) -> Option<&'static str> {
    match func_name {
        // Console namespace
        "consoleLog" => Some("console.log(...args: any) -> void"),
        "consolePrintln" => Some("console.println(...args: any) -> void"),
        "consolePrint" => Some("console.print(...args: any) -> void"),
        "consoleError" => Some("console.error(...args: any) -> void"),
        "consoleWarn" => Some("console.warn(...args: any) -> void"),
        "consoleDebug" => Some("console.debug(...args: any) -> void"),
        // Core builtins
        "len" => Some("len(value: string | []any) -> number"),
        "str" => Some("str(value: any) -> string"),
        "num" => Some("num(value: string) -> number"),
        "bool" => Some("bool(value: any) -> bool"),
        "type" => Some("type(value: any) -> string"),
        "assert" => Some("assert(condition: bool, message: string) -> void"),
        "assertEq" => Some("assertEq(actual: any, expected: any) -> void"),
        "panic" => Some("panic(message: string) -> void"),
        // String functions
        "split" => Some("split(str: string, sep: string) -> []string"),
        "join" => Some("join(arr: []string, sep: string) -> string"),
        "trim" => Some("trim(str: string) -> string"),
        "trimStart" => Some("trimStart(str: string) -> string"),
        "trimEnd" => Some("trimEnd(str: string) -> string"),
        "indexOf" => Some("indexOf(str: string, sub: string) -> number"),
        "lastIndexOf" => Some("lastIndexOf(str: string, sub: string) -> number"),
        "includes" => Some("includes(str: string, sub: string) -> bool"),
        "startsWith" => Some("startsWith(str: string, prefix: string) -> bool"),
        "endsWith" => Some("endsWith(str: string, suffix: string) -> bool"),
        "repeat" => Some("repeat(str: string, count: number) -> string"),
        "replace" => Some("replace(str: string, old: string, new: string) -> string"),
        "toUpperCase" => Some("toUpperCase(str: string) -> string"),
        "toLowerCase" => Some("toLowerCase(str: string) -> string"),
        "padStart" => Some("padStart(str: string, len: number, pad: string) -> string"),
        "padEnd" => Some("padEnd(str: string, len: number, pad: string) -> string"),
        "substring" => Some("substring(str: string, start: number, end: number) -> string"),
        "charCodeAt" => Some("charCodeAt(str: string, index: number) -> number"),
        "fromCharCode" => Some("fromCharCode(code: number) -> string"),
        // Array functions
        "arrayPush" => Some("arrayPush(arr: T[], value: T) -> T[]"),
        "arrayPop" => Some("arrayPop(arr: T[]) -> T[]"),
        "arrayShift" => Some("arrayShift(arr: T[]) -> T[]"),
        "arrayUnshift" => Some("arrayUnshift(arr: T[], value: T) -> T[]"),
        "arraySlice" => Some("arraySlice(arr: T[], start: number, end: number) -> T[]"),
        "arrayConcat" => Some("arrayConcat(a: T[], b: T[]) -> T[]"),
        "arrayIncludes" => Some("arrayIncludes(arr: T[], value: T) -> bool"),
        "arrayIndexOf" => Some("arrayIndexOf(arr: T[], value: T) -> number"),
        "arrayReverse" => Some("arrayReverse(arr: T[]) -> T[]"),
        "arraySort" => Some("arraySort(arr: T[]) -> T[]"),
        "arrayEnumerate" => Some("arrayEnumerate(arr: T[]) -> (number, T)[]"),
        "arrayFlat" => Some("arrayFlat(arr: T[][]) -> T[]"),
        "arrayFlatMap" => Some("arrayFlatMap(arr: T[], fn: (T) -> U[]) -> U[]"),
        "arrayFill" => Some("arrayFill(arr: T[], value: T, start: number, end: number) -> T[]"),
        // Math functions (B22: registered under "math*" keys, accessed via Math.* namespace)
        "mathAbs" => Some("Math.abs(n: number) -> number"),
        "mathCeil" => Some("Math.ceil(n: number) -> number"),
        "mathFloor" => Some("Math.floor(n: number) -> number"),
        "mathRound" => Some("Math.round(n: number) -> number"),
        "mathSqrt" => Some("Math.sqrt(n: number) -> Result<number, string>"),
        "mathPow" => Some("Math.pow(base: number, exp: number) -> number"),
        "mathMin" => Some("Math.min(a: number, b: number) -> number"),
        "mathMax" => Some("Math.max(a: number, b: number) -> number"),
        "mathRandom" => Some("Math.random() -> number"),
        "mathLog" => Some("Math.log(n: number) -> Result<number, string>"),
        "mathLog2" => Some("Math.log2(n: number) -> Result<number, string>"),
        "mathLog10" => Some("Math.log10(n: number) -> Result<number, string>"),
        "mathSin" => Some("Math.sin(n: number) -> number"),
        "mathCos" => Some("Math.cos(n: number) -> number"),
        "mathTan" => Some("Math.tan(n: number) -> number"),
        "mathAsin" => Some("Math.asin(n: number) -> Result<number, string>"),
        "mathAcos" => Some("Math.acos(n: number) -> Result<number, string>"),
        "mathAtan" => Some("Math.atan(n: number) -> number"),
        "mathAtan2" => Some("Math.atan2(y: number, x: number) -> number"),
        "mathTrunc" => Some("Math.trunc(n: number) -> number"),
        "mathExp" => Some("Math.exp(n: number) -> number"),
        "mathCbrt" => Some("Math.cbrt(n: number) -> number"),
        "mathHypot" => Some("Math.hypot(x: number, y: number) -> number"),
        "mathClamp" => {
            Some("Math.clamp(n: number, min: number, max: number) -> Result<number, string>")
        }
        "mathSign" => Some("Math.sign(n: number) -> number"),
        "mathPI" => Some("Math.PI -> number"),
        "mathE" => Some("Math.E -> number"),
        "mathSQRT2" => Some("Math.SQRT2 -> number"),
        "mathLN2" => Some("Math.LN2 -> number"),
        "mathLN10" => Some("Math.LN10 -> number"),
        // JSON — B23: bare globals removed. Use Json.* namespace.
        "formatJSON" => Some("formatJSON(value: any) -> string"),
        // Collections
        "hashMapNew" => Some("hashMapNew() -> HashMap<string, any>"),
        "hashMapGet" => Some("hashMapGet(map: HashMap<K,V>, key: K) -> V | null"),
        "hashMapSet" => Some("hashMapSet(map: HashMap<K,V>, key: K, value: V) -> HashMap<K,V>"),
        "hashMapHas" => Some("hashMapHas(map: HashMap<K,V>, key: K) -> bool"),
        "hashMapDelete" => Some("hashMapDelete(map: HashMap<K,V>, key: K) -> HashMap<K,V>"),
        "hashMapKeys" => Some("hashMapKeys(map: HashMap<K,V>) -> K[]"),
        "hashMapValues" => Some("hashMapValues(map: HashMap<K,V>) -> V[]"),
        "hashMapEntries" => Some("hashMapEntries(map: HashMap<K,V>) -> [][K, V]"),
        "hashSetNew" => Some("hashSetNew() -> HashSet<any>"),
        "hashSetAdd" => Some("hashSetAdd(set: HashSet<T>, value: T) -> HashSet<T>"),
        "hashSetHas" => Some("hashSetHas(set: HashSet<T>, value: T) -> bool"),
        "hashSetRemove" => Some("hashSetRemove(set: HashSet<T>, value: T) -> HashSet<T>"),
        _ => None,
    }
}

/// Construct an InvalidStdlibArgument error with context.
/// Automatically appends the function signature when registered.
pub fn stdlib_arg_error(
    func_name: &str,
    expected: &str,
    actual: &Value,
    span: crate::span::Span,
) -> RuntimeError {
    let sig = stdlib_signature(func_name);
    let msg = if let Some(sig) = sig {
        format!(
            "{}(): expected {}, got {}\n  Signature: {}",
            func_name,
            expected,
            actual.type_name(),
            sig
        )
    } else {
        format!(
            "{}(): expected {}, got {}",
            func_name,
            expected,
            actual.type_name()
        )
    };
    RuntimeError::InvalidStdlibArgument { msg, span }
}

/// Construct an arity error for stdlib functions.
/// Automatically appends the function signature when registered.
pub fn stdlib_arity_error(
    func_name: &str,
    expected: usize,
    actual: usize,
    span: crate::span::Span,
) -> RuntimeError {
    let sig = stdlib_signature(func_name);
    let msg = if let Some(sig) = sig {
        format!(
            "{}(): expected {} argument{}, got {}\n  Signature: {}",
            func_name,
            expected,
            if expected == 1 { "" } else { "s" },
            actual,
            sig
        )
    } else {
        format!(
            "{}(): expected {} argument{}, got {}",
            func_name,
            expected,
            if expected == 1 { "" } else { "s" },
            actual
        )
    };
    RuntimeError::InvalidStdlibArgument { msg, span }
}

static BUILTIN_REGISTRY: OnceLock<HashMap<&'static str, BuiltinFn>> = OnceLock::new();

fn builtin_registry() -> &'static HashMap<&'static str, BuiltinFn> {
    BUILTIN_REGISTRY.get_or_init(|| {
        let mut m: HashMap<&'static str, BuiltinFn> = HashMap::with_capacity(300);

        // ====================================================================
        // Core
        // ====================================================================
        // ====================================================================
        // Console namespace (console.log, console.error, etc.)
        // ====================================================================
        m.insert("consoleLog", |args, span, _, output| {
            console::console_log(args, span, output)
        });
        m.insert("consolePrintln", |args, span, _, output| {
            console::console_println(args, span, output)
        });
        m.insert("consolePrint", |args, span, _, output| {
            console::console_print(args, span, output)
        });
        m.insert("consoleError", |args, span, _, output| {
            console::console_error(args, span, output)
        });
        m.insert("consoleWarn", |args, span, _, output| {
            console::console_warn(args, span, output)
        });
        m.insert("consoleDebug", |args, span, _, output| {
            console::console_debug(args, span, output)
        });
        m.insert("len", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("len", 1, args.len(), span));
            }
            Ok(Value::Number(len(&args[0], span)?))
        });
        m.insert("str", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("str", 1, args.len(), span));
            }
            let s = str(&args[0], span)?;
            Ok(Value::string(s))
        });

        // ====================================================================
        // String functions
        // ====================================================================
        m.insert("split", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("split", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "split", span)?;
            let sep = extract_string(&args[1], "split", span)?;
            string::split(s, sep, span)
        });
        m.insert("join", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("join", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "join", span)?;
            let sep = extract_string(&args[1], "join", span)?;
            let result = string::join(&arr, sep, span)?;
            Ok(Value::string(result))
        });
        m.insert("trim", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("trim", 1, args.len(), span));
            }
            let s = extract_string(&args[0], "trim", span)?;
            Ok(Value::string(string::trim(s)))
        });
        m.insert("trimStart", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("trimStart", 1, args.len(), span));
            }
            let s = extract_string(&args[0], "trimStart", span)?;
            Ok(Value::string(string::trim_start(s)))
        });
        m.insert("trimEnd", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("trimEnd", 1, args.len(), span));
            }
            let s = extract_string(&args[0], "trimEnd", span)?;
            Ok(Value::string(string::trim_end(s)))
        });
        m.insert("indexOf", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("indexOf", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "indexOf", span)?;
            let search = extract_string(&args[1], "indexOf", span)?;
            match string::index_of(s, search) {
                Some(idx) => Ok(Value::Option(Some(Box::new(Value::Number(idx))))),
                None => Ok(Value::Option(None)),
            }
        });
        m.insert("lastIndexOf", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("lastIndexOf", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "lastIndexOf", span)?;
            let search = extract_string(&args[1], "lastIndexOf", span)?;
            match string::last_index_of(s, search) {
                Some(idx) => Ok(Value::Option(Some(Box::new(Value::Number(idx))))),
                None => Ok(Value::Option(None)),
            }
        });
        m.insert("includes", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("includes", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "includes", span)?;
            let search = extract_string(&args[1], "includes", span)?;
            Ok(Value::Bool(string::includes(s, search)))
        });
        m.insert("toUpperCase", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("toUpperCase", 1, args.len(), span));
            }
            let s = extract_string(&args[0], "toUpperCase", span)?;
            Ok(Value::string(string::to_upper_case(s)))
        });
        m.insert("toLowerCase", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("toLowerCase", 1, args.len(), span));
            }
            let s = extract_string(&args[0], "toLowerCase", span)?;
            Ok(Value::string(string::to_lower_case(s)))
        });
        m.insert("substring", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("substring", 3, args.len(), span));
            }
            let s = extract_string(&args[0], "substring", span)?;
            let start = extract_number(&args[1], "substring", span)?;
            let end = extract_number(&args[2], "substring", span)?;
            let result = string::substring(s, start, end, span)?;
            Ok(Value::string(result))
        });
        m.insert("charAt", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("charAt", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "charAt", span)?;
            let index = extract_number(&args[1], "charAt", span)?;
            match string::char_at(s, index, span)? {
                Some(c) => Ok(Value::Option(Some(Box::new(Value::string(c))))),
                None => Ok(Value::Option(None)),
            }
        });
        m.insert("repeat", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("repeat", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "repeat", span)?;
            let count = extract_number(&args[1], "repeat", span)?;
            let result = string::repeat(s, count, span)?;
            Ok(Value::string(result))
        });
        m.insert("replace", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("replace", 3, args.len(), span));
            }
            let s = extract_string(&args[0], "replace", span)?;
            let search = extract_string(&args[1], "replace", span)?;
            let replacement = extract_string(&args[2], "replace", span)?;
            Ok(Value::string(string::replace(s, search, replacement)))
        });
        m.insert("replaceAll", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("replaceAll", 3, args.len(), span));
            }
            let s = extract_string(&args[0], "replaceAll", span)?;
            let search = extract_string(&args[1], "replaceAll", span)?;
            let replacement = extract_string(&args[2], "replaceAll", span)?;
            Ok(Value::string(string::replace_all(s, search, replacement)))
        });
        m.insert("padStart", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("padStart", 3, args.len(), span));
            }
            let s = extract_string(&args[0], "padStart", span)?;
            let length = extract_number(&args[1], "padStart", span)?;
            let fill = extract_string(&args[2], "padStart", span)?;
            let result = string::pad_start(s, length, fill, span)?;
            Ok(Value::string(result))
        });
        m.insert("padEnd", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("padEnd", 3, args.len(), span));
            }
            let s = extract_string(&args[0], "padEnd", span)?;
            let length = extract_number(&args[1], "padEnd", span)?;
            let fill = extract_string(&args[2], "padEnd", span)?;
            let result = string::pad_end(s, length, fill, span)?;
            Ok(Value::string(result))
        });
        m.insert("fromCharCode", |args, span, _, _| {
            if args.is_empty() {
                return Err(stdlib_arity_error("fromCharCode", 1, args.len(), span));
            }
            let mut result = String::new();
            for arg in args {
                let code = extract_number(arg, "fromCharCode", span)? as u32;
                let ch = char::from_u32(code).ok_or_else(|| RuntimeError::TypeError {
                    msg: format!("fromCharCode: {} is not a valid Unicode code point", code),
                    span,
                })?;
                result.push(ch);
            }
            Ok(Value::string(result))
        });
        m.insert("charCodeAt", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("charCodeAt", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "charCodeAt", span)?;
            let idx = extract_number(&args[1], "charCodeAt", span)? as usize;
            let ch = s.chars().nth(idx).ok_or_else(|| RuntimeError::TypeError {
                msg: format!(
                    "charCodeAt: index {} out of bounds for string of length {}",
                    idx,
                    s.chars().count()
                ),
                span,
            })?;
            Ok(Value::Number(ch as u32 as f64))
        });
        m.insert("startsWith", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("startsWith", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "startsWith", span)?;
            let prefix = extract_string(&args[1], "startsWith", span)?;
            Ok(Value::Bool(string::starts_with(s, prefix)))
        });
        m.insert("endsWith", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("endsWith", 2, args.len(), span));
            }
            let s = extract_string(&args[0], "endsWith", span)?;
            let suffix = extract_string(&args[1], "endsWith", span)?;
            Ok(Value::Bool(string::ends_with(s, suffix)))
        });

        // ====================================================================
        // Array functions
        // ====================================================================
        // Method-call variants (prefixed with "array") — used by arr.method() syntax
        m.insert("arrayPush", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("arrayPush", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayPush", span)?;
            Ok(array::push(&arr, args[1].clone()))
        });
        m.insert("arrayPop", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("arrayPop", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayPop", span)?;
            array::pop(&arr, span)
        });
        m.insert("arrayShift", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("arrayShift", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayShift", span)?;
            array::shift(&arr, span)
        });
        m.insert("arrayUnshift", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("arrayUnshift", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayUnshift", span)?;
            Ok(array::unshift(&arr, args[1].clone()))
        });
        m.insert("arrayReverse", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("arrayReverse", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayReverse", span)?;
            Ok(array::reverse(&arr))
        });
        m.insert("arraySort", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("arraySort", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "arraySort", span)?;
            Ok(array::sort_natural(&arr))
        });
        m.insert("arrayEnumerate", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("arrayEnumerate", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayEnumerate", span)?;
            Ok(array::enumerate(&arr))
        });
        // Free-function variants (legacy names)
        m.insert("pop", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pop", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "pop", span)?;
            array::pop(&arr, span)
        });
        m.insert("shift", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("shift", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "shift", span)?;
            array::shift(&arr, span)
        });
        m.insert("unshift", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("unshift", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "unshift", span)?;
            Ok(array::unshift(&arr, args[1].clone()))
        });
        m.insert("reverse", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("reverse", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "reverse", span)?;
            Ok(array::reverse(&arr))
        });
        m.insert("concat", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("concat", 2, args.len(), span));
            }
            let arr1 = extract_array(&args[0], "concat", span)?;
            let arr2 = extract_array(&args[1], "concat", span)?;
            Ok(array::concat(&arr1, &arr2))
        });
        m.insert("flatten", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("flatten", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "flatten", span)?;
            array::flatten(&arr, span)
        });
        m.insert("arrayIndexOf", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("arrayIndexOf", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayIndexOf", span)?;
            match array::index_of(&arr, &args[1]) {
                Some(idx) => Ok(Value::Option(Some(Box::new(Value::Number(idx))))),
                None => Ok(Value::Option(None)),
            }
        });
        m.insert("arrayLastIndexOf", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("arrayLastIndexOf", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayLastIndexOf", span)?;
            match array::last_index_of(&arr, &args[1]) {
                Some(idx) => Ok(Value::Option(Some(Box::new(Value::Number(idx))))),
                None => Ok(Value::Option(None)),
            }
        });
        m.insert("arrayIncludes", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("arrayIncludes", 2, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayIncludes", span)?;
            Ok(Value::Bool(array::includes(&arr, &args[1])))
        });
        m.insert("arrayIsEmpty", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("arrayIsEmpty", 1, args.len(), span));
            }
            let arr = extract_array(&args[0], "arrayIsEmpty", span)?;
            Ok(Value::Bool(arr.is_empty()))
        });
        m.insert("slice", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("slice", 3, args.len(), span));
            }
            let arr = extract_array(&args[0], "slice", span)?;
            let start = extract_number(&args[1], "slice", span)?;
            let end = extract_number(&args[2], "slice", span)?;
            array::slice(&arr, start, end, span)
        });

        // ====================================================================
        // Math namespace functions — registered under "math*" keys.
        // B22: Bare globals removed. Math.* is the only call syntax.
        // Dispatch: Math.abs → resolve_math_ns_method("abs") → "mathAbs" → call_builtin.
        // ====================================================================
        m.insert("mathAbs", |a, s, _, _| math::abs(a, s));
        m.insert("mathFloor", |a, s, _, _| math::floor(a, s));
        m.insert("mathCeil", |a, s, _, _| math::ceil(a, s));
        m.insert("mathRound", |a, s, _, _| math::round(a, s));
        m.insert("mathMin", |a, s, _, _| math::min(a, s));
        m.insert("mathMax", |a, s, _, _| math::max(a, s));
        m.insert("mathSqrt", |a, s, _, _| math::sqrt(a, s));
        m.insert("mathPow", |a, s, _, _| math::pow(a, s));
        m.insert("mathLog", |a, s, _, _| math::log(a, s));
        m.insert("mathSin", |a, s, _, _| math::sin(a, s));
        m.insert("mathCos", |a, s, _, _| math::cos(a, s));
        m.insert("mathTan", |a, s, _, _| math::tan(a, s));
        m.insert("mathAsin", |a, s, _, _| math::asin(a, s));
        m.insert("mathAcos", |a, s, _, _| math::acos(a, s));
        m.insert("mathAtan", |a, s, _, _| math::atan(a, s));
        m.insert("mathAtan2", |a, s, _, _| math::atan2(a, s));
        m.insert("mathTrunc", |a, s, _, _| math::trunc(a, s));
        m.insert("mathLog2", |a, s, _, _| math::log2(a, s));
        m.insert("mathLog10", |a, s, _, _| math::log10(a, s));
        m.insert("mathExp", |a, s, _, _| math::exp(a, s));
        m.insert("mathCbrt", |a, s, _, _| math::cbrt(a, s));
        m.insert("mathHypot", |a, s, _, _| math::hypot(a, s));
        m.insert("mathClamp", |a, s, _, _| math::clamp(a, s));
        m.insert("mathSign", |a, s, _, _| math::sign(a, s));
        m.insert("mathRandom", |a, s, _, _| math::random(a, s));
        // Math namespace constant accessors
        m.insert("mathPI", |a, s, _, _| math::math_pi(a, s));
        m.insert("mathE", |a, s, _, _| math::math_e(a, s));
        m.insert("mathSQRT2", |a, s, _, _| math::math_sqrt2(a, s));
        m.insert("mathLN2", |a, s, _, _| math::math_ln2(a, s));
        m.insert("mathLN10", |a, s, _, _| math::math_ln10(a, s));

        // ====================================================================
        // JSON functions
        // ====================================================================
        // B23: bare globals removed — all Json.* calls route through jsonNs* keys.
        m.insert("jsonNsParse", |a, s, _, _| json::parse_json(a, s));
        m.insert("jsonNsStringify", |a, s, _, _| json::to_json(a, s));
        m.insert("jsonNsIsValid", |a, s, _, _| json::is_valid_json(a, s));
        m.insert("jsonNsPrettify", |a, s, _, _| json::prettify_json(a, s));
        m.insert("jsonAsString", |a, s, _, _| json::json_as_string(a, s));
        m.insert("jsonAsNumber", |a, s, _, _| json::json_as_number(a, s));
        m.insert("jsonAsBool", |a, s, _, _| json::json_as_bool(a, s));
        m.insert("jsonGetString", |a, s, _, _| json::json_get_string(a, s));
        m.insert("jsonGetNumber", |a, s, _, _| json::json_get_number(a, s));
        m.insert("jsonGetBool", |a, s, _, _| json::json_get_bool(a, s));
        m.insert("jsonGetArray", |a, s, _, _| json::json_get_array(a, s));
        m.insert("jsonGetObject", |a, s, _, _| json::json_get_object(a, s));
        m.insert("jsonIsNull", |a, s, _, _| json::json_is_null(a, s));
        // Json namespace string-based methods (B23) — used by Json.* dispatch
        m.insert("jsonNsMinify", |a, s, _, _| json::json_ns_minify(a, s));
        m.insert("jsonNsKeys", |a, s, _, _| json::json_ns_keys(a, s));
        m.insert("jsonNsGetString", |a, s, _, _| {
            json::json_ns_get_string(a, s)
        });
        m.insert("jsonNsGetNumber", |a, s, _, _| {
            json::json_ns_get_number(a, s)
        });
        m.insert("jsonNsGetBool", |a, s, _, _| json::json_ns_get_bool(a, s));
        m.insert("jsonNsGetArray", |a, s, _, _| json::json_ns_get_array(a, s));
        m.insert("jsonNsGetObject", |a, s, _, _| {
            json::json_ns_get_object(a, s)
        });
        m.insert("jsonNsIsNull", |a, s, _, _| json::json_ns_is_null(a, s));

        // ====================================================================
        // Type checking functions
        // ====================================================================
        m.insert("type_of", |a, s, _, _| types::type_of(a, s));
        m.insert("typeof", |a, s, _, _| types::typeof_fn(a, s));
        m.insert("isString", |a, s, _, _| types::is_string(a, s));
        m.insert("isNumber", |a, s, _, _| types::is_number(a, s));
        m.insert("isBool", |a, s, _, _| types::is_bool(a, s));
        m.insert("isNull", |a, s, _, _| types::is_null(a, s));
        m.insert("isArray", |a, s, _, _| types::is_array(a, s));
        m.insert("isFunction", |a, s, _, _| types::is_function(a, s));
        m.insert("isObject", |a, s, _, _| types::is_object(a, s));
        m.insert("isType", |a, s, _, _| types::is_type(a, s));
        m.insert("hasField", |a, s, _, _| types::has_field(a, s));
        m.insert("hasMethod", |a, s, _, _| types::has_method(a, s));
        m.insert("hasTag", |a, s, _, _| types::has_tag(a, s));

        // ====================================================================
        // Type conversion functions
        // ====================================================================
        m.insert("toString", |a, s, _, _| types::to_string(a, s));
        m.insert("toNumber", |a, s, _, _| types::to_number(a, s));
        m.insert("toBool", |a, s, _, _| types::to_bool(a, s));
        m.insert("parseInt", |a, s, _, _| types::parse_int(a, s));
        m.insert("parseFloat", |a, s, _, _| types::parse_float(a, s));

        // ====================================================================
        // Option<T> constructors and helpers
        // ====================================================================
        m.insert("Some", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("Some", 1, args.len(), span));
            }
            Ok(types::some(args[0].clone()))
        });
        m.insert("None", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("None", 0, args.len(), span));
            }
            Ok(types::none())
        });
        m.insert("is_some", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("is_some", 1, args.len(), span));
            }
            Ok(Value::Bool(types::is_some(&args[0], span)?))
        });
        m.insert("is_none", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("is_none", 1, args.len(), span));
            }
            Ok(Value::Bool(types::is_none(&args[0], span)?))
        });

        // ====================================================================
        // Result<T,E> constructors and helpers
        // ====================================================================
        m.insert("Ok", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("Ok", 1, args.len(), span));
            }
            Ok(types::ok(args[0].clone()))
        });
        m.insert("Err", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("Err", 1, args.len(), span));
            }
            Ok(types::err(args[0].clone()))
        });
        m.insert("is_ok", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("is_ok", 1, args.len(), span));
            }
            Ok(Value::Bool(types::is_ok(&args[0], span)?))
        });
        m.insert("is_err", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("is_err", 1, args.len(), span));
            }
            Ok(Value::Bool(types::is_err(&args[0], span)?))
        });

        // ====================================================================
        // Generic unwrap functions (Option + Result)
        // ====================================================================
        m.insert("unwrap", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("unwrap", 1, args.len(), span));
            }
            match &args[0] {
                Value::Option(_) => types::unwrap_option(&args[0], span),
                Value::Result(_) => types::unwrap_result(&args[0], span),
                _ => Err(RuntimeError::TypeError {
                    msg: "unwrap() requires Option or Result value".to_string(),
                    span,
                }),
            }
        });
        m.insert("unwrap_or", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("unwrap_or", 2, args.len(), span));
            }
            match &args[0] {
                Value::Option(_) => types::unwrap_or_option(&args[0], args[1].clone(), span),
                Value::Result(_) => types::unwrap_or_result(&args[0], args[1].clone(), span),
                _ => Err(RuntimeError::TypeError {
                    msg: "unwrap_or() requires Option or Result value".to_string(),
                    span,
                }),
            }
        });
        m.insert("unwrapOr", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("unwrapOr", 2, args.len(), span));
            }
            match &args[0] {
                Value::Option(_) => types::unwrap_or_option(&args[0], args[1].clone(), span),
                Value::Result(_) => types::unwrap_or_result(&args[0], args[1].clone(), span),
                _ => Err(RuntimeError::TypeError {
                    msg: "unwrapOr() requires Option or Result value".to_string(),
                    span,
                }),
            }
        });
        m.insert("isSome", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("isSome", 1, args.len(), span));
            }
            match &args[0] {
                Value::Option(opt) => Ok(Value::Bool(opt.is_some())),
                _ => Err(RuntimeError::TypeError {
                    msg: "isSome() requires Option value".to_string(),
                    span,
                }),
            }
        });
        m.insert("isNone", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("isNone", 1, args.len(), span));
            }
            match &args[0] {
                Value::Option(opt) => Ok(Value::Bool(opt.is_none())),
                _ => Err(RuntimeError::TypeError {
                    msg: "isNone() requires Option value".to_string(),
                    span,
                }),
            }
        });
        m.insert("isOk", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("isOk", 1, args.len(), span));
            }
            match &args[0] {
                Value::Result(res) => Ok(Value::Bool(res.is_ok())),
                _ => Err(RuntimeError::TypeError {
                    msg: "isOk() requires Result value".to_string(),
                    span,
                }),
            }
        });
        m.insert("isErr", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("isErr", 1, args.len(), span));
            }
            match &args[0] {
                Value::Result(res) => Ok(Value::Bool(res.is_err())),
                _ => Err(RuntimeError::TypeError {
                    msg: "isErr() requires Result value".to_string(),
                    span,
                }),
            }
        });
        m.insert("expect", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("expect", 2, args.len(), span));
            }
            let message = extract_string(&args[1], "expect", span)?;
            match &args[0] {
                Value::Option(_) => types::expect_option(&args[0], message, span),
                Value::Result(_) => types::expect_result(&args[0], message, span),
                _ => Err(RuntimeError::TypeError {
                    msg: "expect() requires Option or Result value".to_string(),
                    span,
                }),
            }
        });
        m.insert("result_ok", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("result_ok", 1, args.len(), span));
            }
            types::result_ok(&args[0], span)
        });
        m.insert("result_err", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("result_err", 1, args.len(), span));
            }
            types::result_err(&args[0], span)
        });

        // ====================================================================
        // File I/O functions
        // ====================================================================
        // B26: bare ioReadLine/ioReadLinePrompt removed — all io.* calls route through ioNs* keys.
        m.insert("ioNsReadLine", |a, s, sc, _| io::io_read_line(a, s, sc));
        m.insert("ioNsReadLinePrompt", |a, s, sc, _| {
            io::io_read_line_prompt(a, s, sc)
        });
        m.insert("ioNsWrite", |a, s, sc, _| io::io_write(a, s, sc));
        m.insert("ioNsWriteLine", |a, s, sc, _| io::io_write_line(a, s, sc));
        m.insert("ioNsReadAll", |a, s, sc, _| io::io_read_all(a, s, sc));
        m.insert("ioNsFlush", |a, s, sc, _| io::io_flush(a, s, sc));
        // B24: bare globals removed — all file.* calls route through fileNs* keys.
        m.insert("fileNsRead", |a, s, sc, _| io::read_file(a, s, sc));
        m.insert("fileNsWrite", |a, s, sc, _| io::write_file(a, s, sc));
        m.insert("fileNsAppend", |a, s, sc, _| io::append_file(a, s, sc));
        m.insert("fileNsExists", |a, s, sc, _| io::file_exists(a, s, sc));
        m.insert("fileNsRemove", |a, s, sc, _| io::remove_file(a, s, sc));
        m.insert("fileNsCreateDir", |a, s, sc, _| io::create_dir(a, s, sc));
        m.insert("fileNsRemoveDir", |a, s, sc, _| io::remove_dir(a, s, sc));
        m.insert("fileInfo", |a, s, sc, _| io::file_info(a, s, sc));
        m.insert("pathJoin", |a, s, sc, _| io::path_join(a, s, sc));

        // ====================================================================
        // Reflection functions
        // ====================================================================
        m.insert("reflect_typeof", |a, s, _, _| reflect::typeof_fn(a, s));
        m.insert("reflect_is_callable", |a, s, _, _| {
            reflect::is_callable_fn(a, s)
        });
        m.insert("reflect_is_primitive", |a, s, _, _| {
            reflect::is_primitive_fn(a, s)
        });
        m.insert("reflect_same_type", |a, s, _, _| {
            reflect::same_type_fn(a, s)
        });
        m.insert("reflect_get_length", |a, s, _, _| {
            reflect::get_length_fn(a, s)
        });
        m.insert("reflect_is_empty", |a, s, _, _| reflect::is_empty_fn(a, s));
        m.insert("reflect_type_describe", |a, s, _, _| {
            reflect::type_describe_fn(a, s)
        });
        m.insert("reflect_clone", |a, s, _, _| reflect::clone_fn(a, s));
        m.insert("value_to_string", |a, s, _, _| {
            reflect::value_to_string_fn(a, s)
        });
        m.insert("reflect_value_to_string", |a, s, _, _| {
            reflect::value_to_string_fn(a, s)
        });
        m.insert("reflect_deep_equals", |a, s, _, _| {
            reflect::deep_equals_fn(a, s)
        });
        m.insert("reflect_get_function_name", |a, s, _, _| {
            reflect::get_function_name_fn(a, s)
        });
        m.insert("reflect_get_function_arity", |a, s, _, _| {
            reflect::get_function_arity_fn(a, s)
        });

        // ====================================================================
        // HashMap functions
        // ====================================================================
        m.insert("hashMapNew", |a, s, _, _| {
            collections::hashmap::new_map(a, s)
        });
        m.insert("hashMapFromEntries", |a, s, _, _| {
            collections::hashmap::from_entries(a, s)
        });
        m.insert("hashMapPut", |a, s, _, _| collections::hashmap::put(a, s));
        m.insert("hashMapCopy", |a, s, _, _| collections::hashmap::copy(a, s));
        m.insert("hashMapGet", |a, s, _, _| collections::hashmap::get(a, s));
        m.insert("hashMapRemove", |a, s, _, _| {
            collections::hashmap::remove(a, s)
        });
        m.insert("hashMapHas", |a, s, _, _| collections::hashmap::has(a, s));
        m.insert("hashMapSize", |a, s, _, _| collections::hashmap::size(a, s));
        m.insert("hashMapIsEmpty", |a, s, _, _| {
            collections::hashmap::is_empty(a, s)
        });
        m.insert("hashMapClear", |a, s, _, _| {
            collections::hashmap::clear(a, s)
        });
        m.insert("hashMapKeys", |a, s, _, _| collections::hashmap::keys(a, s));
        m.insert("hashMapValues", |a, s, _, _| {
            collections::hashmap::values(a, s)
        });
        m.insert("hashMapEntries", |a, s, _, _| {
            collections::hashmap::entries(a, s)
        });

        // ====================================================================
        // HashSet functions
        // ====================================================================
        m.insert("hashSetNew", |a, s, _, _| {
            collections::hashset::new_set(a, s)
        });
        m.insert("hashSetFromArray", |a, s, _, _| {
            collections::hashset::from_array(a, s)
        });
        m.insert("hashSetAdd", |a, s, _, _| collections::hashset::add(a, s));
        m.insert("hashSetRemove", |a, s, _, _| {
            collections::hashset::remove(a, s)
        });
        m.insert("hashSetHas", |a, s, _, _| collections::hashset::has(a, s));
        m.insert("hashSetSize", |a, s, _, _| collections::hashset::size(a, s));
        m.insert("hashSetIsEmpty", |a, s, _, _| {
            collections::hashset::is_empty(a, s)
        });
        m.insert("hashSetClear", |a, s, _, _| {
            collections::hashset::clear(a, s)
        });
        m.insert("hashSetUnion", |a, s, _, _| {
            collections::hashset::union(a, s)
        });
        m.insert("hashSetIntersection", |a, s, _, _| {
            collections::hashset::intersection(a, s)
        });
        m.insert("hashSetDifference", |a, s, _, _| {
            collections::hashset::difference(a, s)
        });
        m.insert("hashSetSymmetricDifference", |a, s, _, _| {
            collections::hashset::symmetric_difference(a, s)
        });
        m.insert("hashSetIsSubset", |a, s, _, _| {
            collections::hashset::is_subset(a, s)
        });
        m.insert("hashSetIsSuperset", |a, s, _, _| {
            collections::hashset::is_superset(a, s)
        });
        m.insert("hashSetToArray", |a, s, _, _| {
            collections::hashset::to_array(a, s)
        });

        // ====================================================================
        // Queue functions
        // ====================================================================
        m.insert("queueNew", |a, s, _, _| collections::queue::new_queue(a, s));
        m.insert("queueEnqueue", |a, s, _, _| {
            collections::queue::enqueue(a, s)
        });
        m.insert("queueDequeue", |a, s, _, _| {
            collections::queue::dequeue(a, s)
        });
        m.insert("queuePeek", |a, s, _, _| collections::queue::peek(a, s));
        m.insert("queueSize", |a, s, _, _| collections::queue::size(a, s));
        m.insert("queueIsEmpty", |a, s, _, _| {
            collections::queue::is_empty(a, s)
        });
        m.insert("queueClear", |a, s, _, _| collections::queue::clear(a, s));
        m.insert("queueToArray", |a, s, _, _| {
            collections::queue::to_array(a, s)
        });

        // ====================================================================
        // Stack functions
        // ====================================================================
        m.insert("stackNew", |a, s, _, _| collections::stack::new_stack(a, s));
        m.insert("stackPush", |a, s, _, _| collections::stack::push(a, s));
        m.insert("stackPop", |a, s, _, _| collections::stack::pop(a, s));
        m.insert("stackPeek", |a, s, _, _| collections::stack::peek(a, s));
        m.insert("stackSize", |a, s, _, _| collections::stack::size(a, s));
        m.insert("stackIsEmpty", |a, s, _, _| {
            collections::stack::is_empty(a, s)
        });
        m.insert("stackClear", |a, s, _, _| collections::stack::clear(a, s));
        m.insert("stackToArray", |a, s, _, _| {
            collections::stack::to_array(a, s)
        });

        // ====================================================================
        // Regex functions
        // ====================================================================
        m.insert("regexNew", |a, s, _, _| regex::regex_new(a, s));
        m.insert("regexNewWithFlags", |a, s, _, _| {
            regex::regex_new_with_flags(a, s)
        });
        m.insert("regexEscape", |a, s, _, _| regex::regex_escape(a, s));
        m.insert("regexIsMatch", |a, s, _, _| regex::regex_is_match(a, s));
        m.insert("regexFind", |a, s, _, _| regex::regex_find(a, s));
        m.insert("regexFindAll", |a, s, _, _| regex::regex_find_all(a, s));
        m.insert("regexCaptures", |a, s, _, _| regex::regex_captures(a, s));
        m.insert("regexCapturesNamed", |a, s, _, _| {
            regex::regex_captures_named(a, s)
        });
        m.insert("regexReplace", |a, s, _, _| regex::regex_replace(a, s));
        m.insert("regexReplaceAll", |a, s, _, _| {
            regex::regex_replace_all(a, s)
        });
        m.insert("regexSplit", |a, s, _, _| regex::regex_split(a, s));
        m.insert("regexSplitN", |a, s, _, _| regex::regex_split_n(a, s));
        m.insert("regexMatchIndices", |a, s, _, _| {
            regex::regex_match_indices(a, s)
        });
        m.insert("regexTest", |a, s, _, _| regex::regex_test(a, s));

        // ====================================================================
        // DateTime functions
        // ====================================================================
        m.insert("dateTimeNow", |a, s, _, _| datetime::date_time_now(a, s));
        m.insert("dateTimeFromTimestamp", |a, s, _, _| {
            datetime::date_time_from_timestamp(a, s)
        });
        m.insert("dateTimeFromComponents", |a, s, _, _| {
            datetime::date_time_from_components(a, s)
        });
        m.insert("dateTimeParseIso", |a, s, _, _| {
            datetime::date_time_parse_iso(a, s)
        });
        m.insert("dateTimeUtc", |a, s, _, _| datetime::date_time_utc(a, s));
        m.insert("dateTimeYear", |a, s, _, _| datetime::date_time_year(a, s));
        m.insert("dateTimeMonth", |a, s, _, _| {
            datetime::date_time_month(a, s)
        });
        m.insert("dateTimeDay", |a, s, _, _| datetime::date_time_day(a, s));
        m.insert("dateTimeHour", |a, s, _, _| datetime::date_time_hour(a, s));
        m.insert("dateTimeMinute", |a, s, _, _| {
            datetime::date_time_minute(a, s)
        });
        m.insert("dateTimeSecond", |a, s, _, _| {
            datetime::date_time_second(a, s)
        });
        m.insert("dateTimeWeekday", |a, s, _, _| {
            datetime::date_time_weekday(a, s)
        });
        m.insert("dateTimeDayOfYear", |a, s, _, _| {
            datetime::date_time_day_of_year(a, s)
        });
        m.insert("dateTimeAddSeconds", |a, s, _, _| {
            datetime::date_time_add_seconds(a, s)
        });
        m.insert("dateTimeAddMinutes", |a, s, _, _| {
            datetime::date_time_add_minutes(a, s)
        });
        m.insert("dateTimeAddHours", |a, s, _, _| {
            datetime::date_time_add_hours(a, s)
        });
        m.insert("dateTimeAddDays", |a, s, _, _| {
            datetime::date_time_add_days(a, s)
        });
        m.insert("dateTimeDiff", |a, s, _, _| datetime::date_time_diff(a, s));
        m.insert("dateTimeCompare", |a, s, _, _| {
            datetime::date_time_compare(a, s)
        });
        m.insert("dateTimeToTimestamp", |a, s, _, _| {
            datetime::date_time_to_timestamp(a, s)
        });
        m.insert("dateTimeToIso", |a, s, _, _| {
            datetime::date_time_to_iso(a, s)
        });
        m.insert("dateTimeFormat", |a, s, _, _| {
            datetime::date_time_format(a, s)
        });
        m.insert("dateTimeToRfc3339", |a, s, _, _| {
            datetime::date_time_to_rfc3339(a, s)
        });
        m.insert("dateTimeToRfc2822", |a, s, _, _| {
            datetime::date_time_to_rfc2822(a, s)
        });
        m.insert("dateTimeToCustom", |a, s, _, _| {
            datetime::date_time_to_custom(a, s)
        });
        m.insert("dateTimeParse", |a, s, _, _| {
            datetime::date_time_parse(a, s)
        });
        m.insert("dateTimeParseRfc3339", |a, s, _, _| {
            datetime::date_time_parse_rfc3339(a, s)
        });
        m.insert("dateTimeParseRfc2822", |a, s, _, _| {
            datetime::date_time_parse_rfc2822(a, s)
        });
        m.insert("dateTimeTryParse", |a, s, _, _| {
            datetime::date_time_try_parse(a, s)
        });
        m.insert("dateTimeToUtc", |a, s, _, _| {
            datetime::date_time_to_utc(a, s)
        });
        m.insert("dateTimeToLocal", |a, s, _, _| {
            datetime::date_time_to_local(a, s)
        });
        m.insert("dateTimeToTimezone", |a, s, _, _| {
            datetime::date_time_to_timezone(a, s)
        });
        m.insert("dateTimeGetTimezone", |a, s, _, _| {
            datetime::date_time_get_timezone(a, s)
        });
        m.insert("dateTimeGetOffset", |a, s, _, _| {
            datetime::date_time_get_offset(a, s)
        });
        m.insert("dateTimeInTimezone", |a, s, _, _| {
            datetime::date_time_in_timezone(a, s)
        });
        m.insert("durationFromSeconds", |a, s, _, _| {
            datetime::duration_from_seconds(a, s)
        });
        m.insert("durationFromMinutes", |a, s, _, _| {
            datetime::duration_from_minutes(a, s)
        });
        m.insert("durationFromHours", |a, s, _, _| {
            datetime::duration_from_hours(a, s)
        });
        m.insert("durationFromDays", |a, s, _, _| {
            datetime::duration_from_days(a, s)
        });
        m.insert("durationFormat", |a, s, _, _| {
            datetime::duration_format(a, s)
        });

        // ====================================================================
        // HTTP functions (feature = "http") — gated: pulls reqwest + aws-lc-sys
        // ====================================================================
        #[cfg(feature = "http")]
        {
            m.insert("httpRequest", |a, s, _, _| http::http_request(a, s));
            m.insert("httpRequestGet", |a, s, _, _| http::http_request_get(a, s));
            m.insert("httpRequestPost", |a, s, _, _| {
                http::http_request_post(a, s)
            });
            m.insert("httpRequestPut", |a, s, _, _| http::http_request_put(a, s));
            m.insert("httpRequestDelete", |a, s, _, _| {
                http::http_request_delete(a, s)
            });
            m.insert("httpRequestPatch", |a, s, _, _| {
                http::http_request_patch(a, s)
            });
            m.insert("httpSetHeader", |a, s, _, _| http::http_set_header(a, s));
            m.insert("httpSetBody", |a, s, _, _| http::http_set_body(a, s));
            m.insert("httpSetTimeout", |a, s, _, _| http::http_set_timeout(a, s));
            m.insert("httpSetQuery", |a, s, _, _| http::http_set_query(a, s));
            m.insert("httpSetFollowRedirects", |a, s, _, _| {
                http::http_set_follow_redirects(a, s)
            });
            m.insert("httpSetMaxRedirects", |a, s, _, _| {
                http::http_set_max_redirects(a, s)
            });
            m.insert("httpSetUserAgent", |a, s, _, _| {
                http::http_set_user_agent(a, s)
            });
            m.insert("httpSetAuth", |a, s, _, _| http::http_set_auth(a, s));
            m.insert("httpStatus", |a, s, _, _| http::http_status(a, s));
            m.insert("httpBody", |a, s, _, _| http::http_body(a, s));
            m.insert("httpHeader", |a, s, _, _| http::http_header(a, s));
            m.insert("httpHeaders", |a, s, _, _| http::http_headers(a, s));
            m.insert("httpUrl", |a, s, _, _| http::http_url(a, s));
            m.insert("httpIsSuccess", |a, s, _, _| http::http_is_success(a, s));
            m.insert("httpStatusText", |a, s, _, _| http::http_status_text(a, s));
            m.insert("httpContentType", |a, s, _, _| {
                http::http_content_type(a, s)
            });
            m.insert("httpContentLength", |a, s, _, _| {
                http::http_content_length(a, s)
            });
            m.insert("httpIsRedirect", |a, s, _, _| http::http_is_redirect(a, s));
            m.insert("httpIsClientError", |a, s, _, _| {
                http::http_is_client_error(a, s)
            });
            m.insert("httpIsServerError", |a, s, _, _| {
                http::http_is_server_error(a, s)
            });
            m.insert("httpSend", |a, s, sec, _| http::http_send(a, s, sec));
            m.insert("httpGet", |a, s, sec, _| http::http_get(a, s, sec));
            m.insert("httpPost", |a, s, sec, _| http::http_post(a, s, sec));
            m.insert("httpPut", |a, s, sec, _| http::http_put(a, s, sec));
            m.insert("httpDelete", |a, s, sec, _| http::http_delete(a, s, sec));
            m.insert("httpPatch", |a, s, sec, _| http::http_patch(a, s, sec));
            m.insert("httpPostJson", |a, s, sec, _| {
                http::http_post_json(a, s, sec)
            });
            m.insert("httpParseJson", |a, s, _, _| http::http_parse_json(a, s));
            m.insert("httpGetJson", |a, s, sec, _| http::http_get_json(a, s, sec));
            m.insert("httpCheckPermission", |a, s, sec, _| {
                http::http_check_permission(a, s, sec)
            });
        }

        // ====================================================================
        // Future/async functions
        // ====================================================================
        m.insert("futureResolve", |a, s, _, _| future::future_resolve(a, s));
        m.insert("futureReject", |a, s, _, _| future::future_reject(a, s));
        m.insert("futureNew", |a, s, _, _| future::future_new(a, s));
        m.insert("futureIsPending", |a, s, _, _| {
            future::future_is_pending(a, s)
        });
        m.insert("futureIsResolved", |a, s, _, _| {
            future::future_is_resolved(a, s)
        });
        m.insert("futureIsRejected", |a, s, _, _| {
            future::future_is_rejected(a, s)
        });
        m.insert("futureThen", |a, s, _, _| future::future_then(a, s));
        m.insert("futureCatch", |a, s, _, _| future::future_catch(a, s));
        m.insert("futureAll", |a, s, _, _| future::future_all_fn(a, s));
        m.insert("futureRace", |a, s, _, _| future::future_race_fn(a, s));

        // ====================================================================
        // Async I/O functions (feature = "http") — HTTP async ops need reqwest
        // ====================================================================
        #[cfg(feature = "http")]
        {
            m.insert("readFileAsync", |a, s, sc, _| {
                async_io::read_file_async(a, s, sc)
            });
            m.insert("writeFileAsync", |a, s, sc, _| {
                async_io::write_file_async(a, s, sc)
            });
            m.insert("appendFileAsync", |a, s, sc, _| {
                async_io::append_file_async(a, s, sc)
            });
            m.insert("httpSendAsync", |a, s, _, _| {
                async_io::http_send_async(a, s)
            });
            m.insert("httpGetAsync", |a, s, _, _| async_io::http_get_async(a, s));
            m.insert("httpPostAsync", |a, s, _, _| {
                async_io::http_post_async(a, s)
            });
            m.insert("httpPutAsync", |a, s, _, _| async_io::http_put_async(a, s));
            m.insert("httpDeleteAsync", |a, s, _, _| {
                async_io::http_delete_async(a, s)
            });
            m.insert("await", |a, s, _, _| async_io::await_future(a, s));
        }

        // ====================================================================
        // Async primitives - tasks
        // ====================================================================
        m.insert("spawn", |a, s, _, _| async_primitives::spawn(a, s));
        m.insert("taskJoin", |a, s, _, _| async_primitives::task_join(a, s));
        m.insert("taskStatus", |a, s, _, _| {
            async_primitives::task_status(a, s)
        });
        m.insert("taskCancel", |a, s, _, _| {
            async_primitives::task_cancel(a, s)
        });
        m.insert("taskId", |a, s, _, _| async_primitives::task_id(a, s));
        m.insert("taskName", |a, s, _, _| async_primitives::task_name(a, s));
        m.insert("joinAll", |a, s, _, _| async_primitives::join_all(a, s));

        // Async primitives - channels
        m.insert("channelBounded", |a, s, _, _| {
            async_primitives::channel_bounded(a, s)
        });
        m.insert("channelUnbounded", |a, s, _, _| {
            async_primitives::channel_unbounded(a, s)
        });
        m.insert("channelSend", |a, s, _, _| {
            async_primitives::channel_send(a, s)
        });
        m.insert("channelReceive", |a, s, _, _| {
            async_primitives::channel_receive(a, s)
        });
        m.insert("channelSelect", |a, s, _, _| {
            async_primitives::channel_select(a, s)
        });
        m.insert("channelIsClosed", |a, s, _, _| {
            async_primitives::channel_is_closed(a, s)
        });

        // Async primitives - sleep/timers
        m.insert("sleep", |a, s, _, _| async_primitives::sleep_fn(a, s));
        m.insert("timer", |a, s, _, _| async_primitives::timer_fn(a, s));
        m.insert("interval", |a, s, _, _| async_primitives::interval_fn(a, s));

        // Async primitives - timeout
        m.insert("timeout", |a, s, _, _| async_primitives::timeout_fn(a, s));

        // Async primitives - mutex
        m.insert("asyncMutex", |a, s, _, _| {
            async_primitives::async_mutex_new(a, s)
        });
        m.insert("asyncMutexGet", |a, s, _, _| {
            async_primitives::async_mutex_get(a, s)
        });
        m.insert("asyncMutexSet", |a, s, _, _| {
            async_primitives::async_mutex_set(a, s)
        });

        // ====================================================================
        // Process management
        // ====================================================================
        m.insert("exec", |a, s, sc, _| process::exec(a, s, sc));
        m.insert("shell", |a, s, sc, _| process::shell(a, s, sc));
        m.insert("shellOut", |a, s, sc, _| process::shell_out(a, s, sc));
        m.insert("getEnv", |a, s, sc, _| process::get_env(a, s, sc));
        m.insert("setEnv", |a, s, sc, _| process::set_env(a, s, sc));
        m.insert("unsetEnv", |a, s, sc, _| process::unset_env(a, s, sc));
        m.insert("listEnv", |a, s, sc, _| process::list_env(a, s, sc));
        m.insert("getCwd", |a, s, sc, _| process::get_cwd(a, s, sc));
        m.insert("getPid", |a, s, sc, _| process::get_pid(a, s, sc));
        m.insert("getProcessArgs", |a, s, sc, _| {
            process::get_process_args(a, s, sc)
        });
        m.insert("processRun", |a, s, sc, _| process::process_run(a, s, sc));
        m.insert("processOutputStdout", |a, s, sc, _| {
            process::process_output_stdout(a, s, sc)
        });
        m.insert("processOutputStderr", |a, s, sc, _| {
            process::process_output_stderr(a, s, sc)
        });
        m.insert("processOutputExitCode", |a, s, sc, _| {
            process::process_output_exit_code(a, s, sc)
        });
        m.insert("processOutputSuccess", |a, s, sc, _| {
            process::process_output_success(a, s, sc)
        });
        // B25: bare process globals removed — all process.* calls route through processNs* keys.
        m.insert("processNsSpawn", |a, s, sc, _| {
            process::spawn_process(a, s, sc)
        });
        m.insert("processNsWaitFor", |a, s, sc, _| {
            process::process_wait(a, s, sc)
        });
        m.insert("processNsKill", |a, s, sc, _| {
            process::process_kill(a, s, sc)
        });
        m.insert("processNsIsRunning", |a, s, sc, _| {
            process::process_is_running(a, s, sc)
        });
        m.insert("processNsStdin", |a, s, sc, _| {
            process::process_stdin(a, s, sc)
        });
        m.insert("processNsStdout", |a, s, sc, _| {
            process::process_stdout(a, s, sc)
        });
        m.insert("processNsStderr", |a, s, sc, _| {
            process::process_stderr(a, s, sc)
        });
        m.insert("processNsOutput", |a, s, sc, _| {
            process::process_output(a, s, sc)
        });

        // ====================================================================
        // Path manipulation
        // ====================================================================
        m.insert("pathJoinArray", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathJoinArray", 1, args.len(), span));
            }
            let segments = extract_array(&args[0], "pathJoinArray", span)?;
            let result = path::path_join(&segments, span)?;
            Ok(Value::string(result))
        });
        m.insert("pathParse", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathParse", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathParse", span)?;
            path::path_parse(path_str, span)
        });
        m.insert("pathNormalize", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathNormalize", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathNormalize", span)?;
            Ok(Value::string(path::path_normalize(path_str, span)?))
        });
        m.insert("pathAbsolute", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathAbsolute", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathAbsolute", span)?;
            Ok(Value::string(path::path_absolute(path_str, span)?))
        });
        m.insert("pathRelative", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("pathRelative", 2, args.len(), span));
            }
            let from = extract_string(&args[0], "pathRelative", span)?;
            let to = extract_string(&args[1], "pathRelative", span)?;
            Ok(Value::string(path::path_relative(from, to, span)?))
        });
        m.insert("pathParent", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathParent", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathParent", span)?;
            Ok(Value::string(path::path_parent(path_str, span)?))
        });
        m.insert("pathBasename", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathBasename", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathBasename", span)?;
            Ok(Value::string(path::path_basename(path_str, span)?))
        });
        m.insert("pathDirname", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathDirname", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathDirname", span)?;
            Ok(Value::string(path::path_dirname(path_str, span)?))
        });
        m.insert("pathExtension", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathExtension", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathExtension", span)?;
            Ok(Value::string(path::path_extension(path_str, span)?))
        });
        m.insert("pathIsAbsolute", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathIsAbsolute", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathIsAbsolute", span)?;
            Ok(Value::Bool(path::path_is_absolute(path_str, span)?))
        });
        m.insert("pathIsRelative", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathIsRelative", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathIsRelative", span)?;
            Ok(Value::Bool(path::path_is_relative(path_str, span)?))
        });
        m.insert("pathExists", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathExists", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathExists", span)?;
            Ok(Value::Bool(path::path_exists(path_str, span)?))
        });
        m.insert("pathCanonical", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathCanonical", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathCanonical", span)?;
            Ok(Value::string(path::path_canonical(path_str, span)?))
        });
        m.insert("pathEquals", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("pathEquals", 2, args.len(), span));
            }
            let path1 = extract_string(&args[0], "pathEquals", span)?;
            let path2 = extract_string(&args[1], "pathEquals", span)?;
            Ok(Value::Bool(path::path_equals(path1, path2, span)?))
        });
        m.insert("pathHomedir", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathHomedir", 0, args.len(), span));
            }
            Ok(Value::string(path::path_homedir(span)?))
        });
        m.insert("pathCwd", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathCwd", 0, args.len(), span));
            }
            Ok(Value::string(path::path_cwd(span)?))
        });
        m.insert("pathTempdir", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathTempdir", 0, args.len(), span));
            }
            Ok(Value::string(path::path_tempdir(span)?))
        });
        m.insert("pathSeparator", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathSeparator", 0, args.len(), span));
            }
            Ok(Value::string(path::path_separator(span)?))
        });
        m.insert("pathDelimiter", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathDelimiter", 0, args.len(), span));
            }
            Ok(Value::string(path::path_delimiter(span)?))
        });
        m.insert("pathExtSeparator", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("pathExtSeparator", 0, args.len(), span));
            }
            Ok(Value::string(path::path_ext_separator(span)?))
        });
        m.insert("pathDrive", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathDrive", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathDrive", span)?;
            Ok(Value::string(path::path_drive(path_str, span)?))
        });
        m.insert("pathToPlatform", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathToPlatform", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathToPlatform", span)?;
            Ok(Value::string(path::path_to_platform(path_str, span)?))
        });
        m.insert("pathToPosix", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathToPosix", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathToPosix", span)?;
            Ok(Value::string(path::path_to_posix(path_str, span)?))
        });
        m.insert("pathToWindows", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("pathToWindows", 1, args.len(), span));
            }
            let path_str = extract_string(&args[0], "pathToWindows", span)?;
            Ok(Value::string(path::path_to_windows(path_str, span)?))
        });

        // ====================================================================
        // File system operations — B24: all registered under fileNs* keys.
        // Bare fs* globals removed; file.* dispatch routes here.
        // ====================================================================
        m.insert("fileNsMkdir", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsMkdir", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsMkdir", span)?;
            fs::mkdir(path, span)
        });
        m.insert("fileNsMkdirp", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsMkdirp", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsMkdirp", span)?;
            fs::mkdirp(path, span)
        });
        m.insert("fileNsRmdir", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsRmdir", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsRmdir", span)?;
            fs::rmdir(path, span)
        });
        m.insert("fileNsRmdirRecursive", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error(
                    "fileNsRmdirRecursive",
                    1,
                    args.len(),
                    span,
                ));
            }
            let path = extract_string(&args[0], "fileNsRmdirRecursive", span)?;
            fs::rmdir_recursive(path, span)
        });
        m.insert("fileNsReadDir", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsReadDir", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsReadDir", span)?;
            fs::readdir(path, span)
        });
        m.insert("fileNsWalk", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsWalk", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsWalk", span)?;
            fs::walk(path, span)
        });
        m.insert("fileNsFilterEntries", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error(
                    "fileNsFilterEntries",
                    2,
                    args.len(),
                    span,
                ));
            }
            let entries = extract_array(&args[0], "fileNsFilterEntries", span)?;
            let pattern = extract_string(&args[1], "fileNsFilterEntries", span)?;
            fs::filter_entries(&entries, pattern, span)
        });
        m.insert("fileNsSortEntries", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsSortEntries", 1, args.len(), span));
            }
            let entries = extract_array(&args[0], "fileNsSortEntries", span)?;
            fs::sort_entries(&entries, span)
        });

        // File system operations - metadata
        m.insert("fileNsSize", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsSize", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsSize", span)?;
            fs::size(path, span)
        });
        m.insert("fileNsMtime", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsMtime", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsMtime", span)?;
            fs::mtime(path, span)
        });
        m.insert("fileNsCtime", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsCtime", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsCtime", span)?;
            fs::ctime(path, span)
        });
        m.insert("fileNsAtime", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsAtime", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsAtime", span)?;
            fs::atime(path, span)
        });
        m.insert("fileNsPermissions", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsPermissions", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsPermissions", span)?;
            fs::permissions(path, span)
        });
        m.insert("fileNsIsDir", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsIsDir", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsIsDir", span)?;
            fs::is_dir(path, span)
        });
        m.insert("fileNsIsFile", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsIsFile", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsIsFile", span)?;
            fs::is_file(path, span)
        });
        m.insert("fileNsIsSymlink", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsIsSymlink", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsIsSymlink", span)?;
            fs::is_symlink(path, span)
        });
        m.insert("fileNsInode", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsInode", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsInode", span)?;
            fs::inode(path, span)
        });

        // File system operations - temporary files
        m.insert("fileNsTempFile", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("fileNsTempFile", 0, args.len(), span));
            }
            fs::tmpfile(span)
        });
        m.insert("fileNsTempDir", |args, span, _, _| {
            if !args.is_empty() {
                return Err(stdlib_arity_error("fileNsTempDir", 0, args.len(), span));
            }
            fs::tmpdir(span)
        });

        // File system operations - symlinks
        m.insert("fileNsSymlink", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("fileNsSymlink", 2, args.len(), span));
            }
            let target = extract_string(&args[0], "fileNsSymlink", span)?;
            let link = extract_string(&args[1], "fileNsSymlink", span)?;
            fs::symlink(target, link, span)
        });
        m.insert("fileNsReadLink", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsReadLink", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsReadLink", span)?;
            fs::readlink(path, span)
        });
        m.insert("fileNsWatch", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsWatch", 1, args.len(), span));
            }
            let path = extract_string(&args[0], "fileNsWatch", span)?;
            fs::watch(path, span)
        });
        m.insert("fileNsWatchNext", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("fileNsWatchNext", 1, args.len(), span));
            }
            fs::watch_next(&args[0], span)
        });

        // ====================================================================
        // Compression - gzip
        // ====================================================================
        m.insert("gzipCompress", |args, span, _, _| {
            if args.is_empty() || args.len() > 2 {
                return Err(stdlib_arity_error("gzipCompress", 1, args.len(), span));
            }
            let level_opt = args.get(1);
            compression::gzip::gzip_compress(&args[0], level_opt, span)
        });
        m.insert("gzipDecompress", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("gzipDecompress", 1, args.len(), span));
            }
            compression::gzip::gzip_decompress(&args[0], span)
        });
        m.insert("gzipDecompressString", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error(
                    "gzipDecompressString",
                    1,
                    args.len(),
                    span,
                ));
            }
            compression::gzip::gzip_decompress_string(&args[0], span)
        });
        m.insert("gzipIsGzip", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("gzipIsGzip", 1, args.len(), span));
            }
            compression::gzip::gzip_is_gzip(&args[0], span)
        });
        m.insert("gzipCompressionRatio", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error(
                    "gzipCompressionRatio",
                    2,
                    args.len(),
                    span,
                ));
            }
            compression::gzip::gzip_compression_ratio(&args[0], &args[1], span)
        });

        // Compression - tar
        m.insert("tarCreate", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("tarCreate", 2, args.len(), span));
            }
            compression::tar::tar_create(&args[0], &args[1], span)
        });
        m.insert("tarCreateGz", |args, span, _, _| {
            if args.len() < 2 || args.len() > 3 {
                return Err(stdlib_arity_error("tarCreateGz", 2, args.len(), span));
            }
            let level_opt = args.get(2);
            compression::tar::tar_create_gz(&args[0], &args[1], level_opt, span)
        });
        m.insert("tarExtract", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("tarExtract", 2, args.len(), span));
            }
            compression::tar::tar_extract(&args[0], &args[1], span)
        });
        m.insert("tarExtractGz", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("tarExtractGz", 2, args.len(), span));
            }
            compression::tar::tar_extract_gz(&args[0], &args[1], span)
        });
        m.insert("tarList", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("tarList", 1, args.len(), span));
            }
            compression::tar::tar_list(&args[0], span)
        });
        m.insert("tarContains", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("tarContains", 2, args.len(), span));
            }
            compression::tar::tar_contains_file(&args[0], &args[1], span)
        });

        // Compression - zip
        m.insert("zipCreate", |args, span, _, _| {
            if args.is_empty() || args.len() > 3 {
                return Err(stdlib_arity_error("zipCreate", 2, args.len(), span));
            }
            let level_opt = args.get(2);
            compression::zip::zip_create(&args[0], &args[1], level_opt, span)
        });
        m.insert("zipCreateWithComment", |args, span, _, _| {
            if args.len() < 3 || args.len() > 4 {
                return Err(stdlib_arity_error(
                    "zipCreateWithComment",
                    3,
                    args.len(),
                    span,
                ));
            }
            let level_opt = args.get(3);
            compression::zip::zip_create_with_comment(&args[0], &args[1], &args[2], level_opt, span)
        });
        m.insert("zipExtract", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("zipExtract", 2, args.len(), span));
            }
            compression::zip::zip_extract(&args[0], &args[1], span)
        });
        m.insert("zipExtractFiles", |args, span, _, _| {
            if args.len() != 3 {
                return Err(stdlib_arity_error("zipExtractFiles", 3, args.len(), span));
            }
            compression::zip::zip_extract_files(&args[0], &args[1], &args[2], span)
        });
        m.insert("zipList", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("zipList", 1, args.len(), span));
            }
            compression::zip::zip_list(&args[0], span)
        });
        m.insert("zipContains", |args, span, _, _| {
            if args.len() != 2 {
                return Err(stdlib_arity_error("zipContains", 2, args.len(), span));
            }
            compression::zip::zip_contains_file(&args[0], &args[1], span)
        });
        m.insert("zipCompressionRatio", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error(
                    "zipCompressionRatio",
                    1,
                    args.len(),
                    span,
                ));
            }
            compression::zip::zip_compression_ratio(&args[0], span)
        });
        m.insert("zipAddFile", |args, span, _, _| {
            if args.len() < 2 || args.len() > 4 {
                return Err(stdlib_arity_error("zipAddFile", 2, args.len(), span));
            }
            let entry_name_opt = args.get(2);
            let level_opt = args.get(3);
            compression::zip::zip_add_file_fn(&args[0], &args[1], entry_name_opt, level_opt, span)
        });
        m.insert("zipValidate", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("zipValidate", 1, args.len(), span));
            }
            compression::zip::zip_validate_fn(&args[0], span)
        });
        m.insert("zipComment", |args, span, _, _| {
            if args.len() != 1 {
                return Err(stdlib_arity_error("zipComment", 1, args.len(), span));
            }
            compression::zip::zip_comment_fn(&args[0], span)
        });

        // ====================================================================
        // Testing primitives (assertions)
        // ====================================================================
        m.insert("assert", |a, s, _, _| test::assert(a, s));
        m.insert("assertFalse", |a, s, _, _| test::assert_false(a, s));
        m.insert("assertEqual", |a, s, _, _| test::assert_equal(a, s));
        m.insert("assertNotEqual", |a, s, _, _| test::assert_not_equal(a, s));
        m.insert("assertOk", |a, s, _, _| test::assert_ok(a, s));
        m.insert("assertErr", |a, s, _, _| test::assert_err(a, s));
        m.insert("assertSome", |a, s, _, _| test::assert_some(a, s));
        m.insert("assertNone", |a, s, _, _| test::assert_none(a, s));
        m.insert("assertContains", |a, s, _, _| test::assert_contains(a, s));
        m.insert("assertEmpty", |a, s, _, _| test::assert_empty(a, s));
        m.insert("assertLength", |a, s, _, _| test::assert_length(a, s));
        m.insert("assertThrows", |a, s, _, _| test::assert_throws(a, s));
        m.insert("assertNoThrow", |a, s, _, _| test::assert_no_throw(a, s));

        // ====================================================================
        // Crypto
        // ====================================================================
        m.insert("sha256", |a, s, _, _| crypto::sha256(a, s));
        m.insert("sha512", |a, s, _, _| crypto::sha512(a, s));
        m.insert("blake3Hash", |a, s, _, _| crypto::blake3_hash(a, s));
        m.insert("hmacSha256", |a, s, _, _| crypto::hmac_sha256(a, s));
        m.insert("hmacSha256Verify", |a, s, _, _| {
            crypto::hmac_sha256_verify(a, s)
        });
        m.insert("aesGcmEncrypt", |a, s, _, _| crypto::aes_gcm_encrypt(a, s));
        m.insert("aesGcmDecrypt", |a, s, _, _| crypto::aes_gcm_decrypt(a, s));
        m.insert("aesGcmGenerateKey", |a, s, _, _| {
            crypto::aes_gcm_generate_key(a, s)
        });

        // ====================================================================
        // Encoding
        // ====================================================================
        m.insert("base64Encode", |a, s, _, _| encoding::base64_encode(a, s));
        m.insert("base64Decode", |a, s, _, _| encoding::base64_decode(a, s));
        m.insert("base64UrlEncode", |a, s, _, _| {
            encoding::base64_url_encode(a, s)
        });
        m.insert("base64UrlDecode", |a, s, _, _| {
            encoding::base64_url_decode(a, s)
        });
        m.insert("hexEncode", |a, s, _, _| encoding::hex_encode(a, s));
        m.insert("hexDecode", |a, s, _, _| encoding::hex_decode(a, s));
        #[cfg(feature = "http")]
        m.insert("urlEncode", |a, s, _, _| encoding::url_encode(a, s));
        #[cfg(feature = "http")]
        m.insert("urlDecode", |a, s, _, _| encoding::url_decode(a, s));

        // ====================================================================
        // Networking TCP/UDP/TLS (feature = "http") — TLS needs rustls/aws-lc-sys
        // ====================================================================
        #[cfg(feature = "http")]
        {
            m.insert("tcpConnect", |a, s, sec, _| net::tcp_connect(a, s, sec));
            m.insert("tcpWrite", |a, s, _, _| net::tcp_write(a, s));
            m.insert("tcpRead", |a, s, _, _| net::tcp_read(a, s));
            m.insert("tcpReadBytes", |a, s, _, _| net::tcp_read_bytes(a, s));
            m.insert("tcpClose", |a, s, _, _| net::tcp_close(a, s));
            m.insert("tcpSetTimeout", |a, s, _, _| net::tcp_set_timeout(a, s));
            m.insert("tcpSetNodelay", |a, s, _, _| net::tcp_set_nodelay(a, s));
            m.insert("tcpLocalAddr", |a, s, _, _| net::tcp_local_addr(a, s));
            m.insert("tcpRemoteAddr", |a, s, _, _| net::tcp_remote_addr(a, s));
            m.insert("tcpListen", |a, s, sec, _| net::tcp_listen(a, s, sec));
            m.insert("tcpAccept", |a, s, _, _| net::tcp_accept(a, s));
            m.insert("tcpListenerAddr", |a, s, _, _| net::tcp_listener_addr(a, s));
            m.insert("tcpListenerClose", |a, s, _, _| {
                net::tcp_listener_close(a, s)
            });
            m.insert("udpBind", |a, s, sec, _| net::udp_bind(a, s, sec));
            m.insert("udpSend", |a, s, _, _| net::udp_send(a, s));
            m.insert("udpReceive", |a, s, _, _| net::udp_receive(a, s));
            m.insert("udpSetTimeout", |a, s, _, _| net::udp_set_timeout(a, s));
            m.insert("udpClose", |a, s, _, _| net::udp_close(a, s));
            m.insert("udpLocalAddr", |a, s, _, _| net::udp_local_addr(a, s));
            m.insert("tlsConnect", |a, s, sec, _| net::tls_connect(a, s, sec));
            m.insert("tlsWrite", |a, s, _, _| net::tls_write(a, s));
            m.insert("tlsRead", |a, s, _, _| net::tls_read(a, s));
            m.insert("tlsClose", |a, s, _, _| net::tls_close(a, s));
        }

        // ====================================================================
        // Synchronization primitives
        // ====================================================================
        m.insert("rwLockNew", |a, s, _, _| sync::rwlock_new(a, s));
        m.insert("rwLockRead", |a, s, _, _| sync::rwlock_read(a, s));
        m.insert("rwLockWrite", |a, s, _, _| sync::rwlock_write(a, s));
        m.insert("rwLockTryRead", |a, s, _, _| sync::rwlock_try_read(a, s));
        m.insert("rwLockTryWrite", |a, s, _, _| sync::rwlock_try_write(a, s));
        m.insert("semaphoreNew", |a, s, _, _| sync::semaphore_new(a, s));
        m.insert("semaphoreAcquire", |a, s, _, _| {
            sync::semaphore_acquire(a, s)
        });
        m.insert("semaphoreTryAcquire", |a, s, _, _| {
            sync::semaphore_try_acquire(a, s)
        });
        m.insert("semaphoreRelease", |a, s, _, _| {
            sync::semaphore_release(a, s)
        });
        m.insert("semaphoreAvailable", |a, s, _, _| {
            sync::semaphore_available(a, s)
        });
        m.insert("atomicNew", |a, s, _, _| sync::atomic_new(a, s));
        m.insert("atomicLoad", |a, s, _, _| sync::atomic_load(a, s));
        m.insert("atomicStore", |a, s, _, _| sync::atomic_store(a, s));
        m.insert("atomicAdd", |a, s, _, _| sync::atomic_add(a, s));
        m.insert("atomicSub", |a, s, _, _| sync::atomic_sub(a, s));
        m.insert("atomicCompareExchange", |a, s, _, _| {
            sync::atomic_compare_exchange(a, s)
        });

        // ====================================================================
        // WebSocket (feature = "http") — needs tungstenite + TLS
        // ====================================================================
        #[cfg(feature = "http")]
        {
            m.insert("wsConnect", |a, s, sec, _| websocket::ws_connect(a, s, sec));
            m.insert("wsSend", |a, s, _, _| websocket::ws_send(a, s));
            m.insert("wsSendBinary", |a, s, _, _| websocket::ws_send_binary(a, s));
            m.insert("wsReceive", |a, s, _, _| websocket::ws_receive(a, s));
            m.insert("wsPing", |a, s, _, _| websocket::ws_ping(a, s));
            m.insert("wsClose", |a, s, _, _| websocket::ws_close(a, s));
        }

        // ====================================================================
        // snake_case aliases (H-082: canonical names)
        // All camelCase names above are deprecated but still functional.
        // ====================================================================
        let snake_case_aliases: &[(&str, &str)] = &[
            // String functions
            ("join", "str_join"),
            ("trimStart", "trim_start"),
            ("trimEnd", "trim_end"),
            ("indexOf", "index_of"),
            ("indexOf", "str_index_of"),
            ("includes", "str_includes"),
            ("lastIndexOf", "last_index_of"),
            ("lastIndexOf", "str_last_index_of"),
            ("toUpperCase", "to_upper_case"),
            ("toLowerCase", "to_lower_case"),
            ("charAt", "char_at"),
            ("padStart", "pad_start"),
            ("padEnd", "pad_end"),
            ("startsWith", "starts_with"),
            ("endsWith", "ends_with"),
            // Array functions
            ("arrayPush", "array_push"),
            ("arrayPop", "array_pop"),
            ("arrayShift", "array_shift"),
            ("arrayUnshift", "array_unshift"),
            ("arrayReverse", "array_reverse"),
            ("arraySort", "array_sort"),
            ("arrayEnumerate", "array_enumerate"),
            ("arrayIndexOf", "array_index_of"),
            ("arrayLastIndexOf", "array_last_index_of"),
            ("arrayIncludes", "array_includes"),
            ("arrayIsEmpty", "array_is_empty"),
            // JSON instance methods — still available as snake_case (JsonValue instance dispatch)
            ("jsonAsString", "json_as_string"),
            ("jsonAsNumber", "json_as_number"),
            ("jsonAsBool", "json_as_bool"),
            ("jsonGetString", "json_get_string"),
            ("jsonGetNumber", "json_get_number"),
            ("jsonGetBool", "json_get_bool"),
            ("jsonGetArray", "json_get_array"),
            ("jsonGetObject", "json_get_object"),
            ("jsonIsNull", "json_is_null"),
            // Type functions
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
            // File I/O — B26: bare ioReadLine/ioReadLinePrompt removed, ioNs* keys only
            ("ioNsReadLine", "io_read_line"),
            ("ioNsReadLinePrompt", "io_read_line_prompt"),
            ("ioNsWrite", "io_write"),
            ("ioNsWriteLine", "io_write_line"),
            ("ioNsReadAll", "io_read_all"),
            ("ioNsFlush", "io_flush"),
            // B24: bare globals removed — fileNs* keys only
            ("fileNsRead", "file_ns_read"),
            ("fileNsWrite", "file_ns_write"),
            ("fileNsAppend", "file_ns_append"),
            ("fileNsExists", "file_ns_exists"),
            ("fileNsRemove", "file_ns_remove"),
            ("fileNsCreateDir", "file_ns_create_dir"),
            ("fileNsRemoveDir", "file_ns_remove_dir"),
            ("fileInfo", "file_info"),
            ("pathJoin", "path_join"),
            // HashMap
            ("hashMapNew", "hash_map_new"),
            ("hashMapFromEntries", "hash_map_from_entries"),
            ("hashMapPut", "hash_map_put"),
            ("hashMapCopy", "hash_map_copy"),
            ("hashMapGet", "hash_map_get"),
            ("hashMapRemove", "hash_map_remove"),
            ("hashMapHas", "hash_map_has"),
            ("hashMapSize", "hash_map_size"),
            ("hashMapIsEmpty", "hash_map_is_empty"),
            ("hashMapClear", "hash_map_clear"),
            ("hashMapKeys", "hash_map_keys"),
            ("hashMapValues", "hash_map_values"),
            ("hashMapEntries", "hash_map_entries"),
            // HashSet
            ("hashSetNew", "hash_set_new"),
            ("hashSetFromArray", "hash_set_from_array"),
            ("hashSetAdd", "hash_set_add"),
            ("hashSetRemove", "hash_set_remove"),
            ("hashSetHas", "hash_set_has"),
            ("hashSetSize", "hash_set_size"),
            ("hashSetIsEmpty", "hash_set_is_empty"),
            ("hashSetClear", "hash_set_clear"),
            ("hashSetUnion", "hash_set_union"),
            ("hashSetIntersection", "hash_set_intersection"),
            ("hashSetDifference", "hash_set_difference"),
            (
                "hashSetSymmetricDifference",
                "hash_set_symmetric_difference",
            ),
            ("hashSetIsSubset", "hash_set_is_subset"),
            ("hashSetIsSuperset", "hash_set_is_superset"),
            ("hashSetToArray", "hash_set_to_array"),
            // Queue
            ("queueNew", "queue_new"),
            ("queueEnqueue", "queue_enqueue"),
            ("queueDequeue", "queue_dequeue"),
            ("queuePeek", "queue_peek"),
            ("queueSize", "queue_size"),
            ("queueIsEmpty", "queue_is_empty"),
            ("queueClear", "queue_clear"),
            ("queueToArray", "queue_to_array"),
            // Stack
            ("stackNew", "stack_new"),
            ("stackPush", "stack_push"),
            ("stackPop", "stack_pop"),
            ("stackPeek", "stack_peek"),
            ("stackSize", "stack_size"),
            ("stackIsEmpty", "stack_is_empty"),
            ("stackClear", "stack_clear"),
            ("stackToArray", "stack_to_array"),
            // Regex
            ("regexNew", "regex_new"),
            ("regexNewWithFlags", "regex_new_with_flags"),
            ("regexEscape", "regex_escape"),
            ("regexIsMatch", "regex_is_match"),
            ("regexFind", "regex_find"),
            ("regexFindAll", "regex_find_all"),
            ("regexCaptures", "regex_captures"),
            ("regexCapturesNamed", "regex_captures_named"),
            ("regexReplace", "regex_replace"),
            ("regexReplaceAll", "regex_replace_all"),
            ("regexSplit", "regex_split"),
            ("regexSplitN", "regex_split_n"),
            ("regexMatchIndices", "regex_match_indices"),
            ("regexTest", "regex_test"),
            // DateTime
            ("dateTimeNow", "date_time_now"),
            ("dateTimeFromTimestamp", "date_time_from_timestamp"),
            ("dateTimeFromComponents", "date_time_from_components"),
            ("dateTimeParseIso", "date_time_parse_iso"),
            ("dateTimeUtc", "date_time_utc"),
            ("dateTimeYear", "date_time_year"),
            ("dateTimeMonth", "date_time_month"),
            ("dateTimeDay", "date_time_day"),
            ("dateTimeHour", "date_time_hour"),
            ("dateTimeMinute", "date_time_minute"),
            ("dateTimeSecond", "date_time_second"),
            ("dateTimeWeekday", "date_time_weekday"),
            ("dateTimeDayOfYear", "date_time_day_of_year"),
            ("dateTimeAddSeconds", "date_time_add_seconds"),
            ("dateTimeAddMinutes", "date_time_add_minutes"),
            ("dateTimeAddHours", "date_time_add_hours"),
            ("dateTimeAddDays", "date_time_add_days"),
            ("dateTimeDiff", "date_time_diff"),
            ("dateTimeCompare", "date_time_compare"),
            ("dateTimeToTimestamp", "date_time_to_timestamp"),
            ("dateTimeToIso", "date_time_to_iso"),
            ("dateTimeFormat", "date_time_format"),
            ("dateTimeToRfc3339", "date_time_to_rfc3339"),
            ("dateTimeToRfc2822", "date_time_to_rfc2822"),
            ("dateTimeToCustom", "date_time_to_custom"),
            ("dateTimeParse", "date_time_parse"),
            ("dateTimeParseRfc3339", "date_time_parse_rfc3339"),
            ("dateTimeParseRfc2822", "date_time_parse_rfc2822"),
            ("dateTimeTryParse", "date_time_try_parse"),
            ("dateTimeToUtc", "date_time_to_utc"),
            ("dateTimeToLocal", "date_time_to_local"),
            ("dateTimeToTimezone", "date_time_to_timezone"),
            ("dateTimeGetTimezone", "date_time_get_timezone"),
            ("dateTimeGetOffset", "date_time_get_offset"),
            ("dateTimeInTimezone", "date_time_in_timezone"),
            // Duration
            ("durationFromSeconds", "duration_from_seconds"),
            ("durationFromMinutes", "duration_from_minutes"),
            ("durationFromHours", "duration_from_hours"),
            ("durationFromDays", "duration_from_days"),
            ("durationFormat", "duration_format"),
            // HTTP
            ("httpRequest", "http_request"),
            ("httpRequestGet", "http_request_get"),
            ("httpRequestPost", "http_request_post"),
            ("httpRequestPut", "http_request_put"),
            ("httpRequestDelete", "http_request_delete"),
            ("httpRequestPatch", "http_request_patch"),
            ("httpSetHeader", "http_set_header"),
            ("httpSetBody", "http_set_body"),
            ("httpSetTimeout", "http_set_timeout"),
            ("httpSetQuery", "http_set_query"),
            ("httpSetFollowRedirects", "http_set_follow_redirects"),
            ("httpSetMaxRedirects", "http_set_max_redirects"),
            ("httpSetUserAgent", "http_set_user_agent"),
            ("httpSetAuth", "http_set_auth"),
            ("httpStatus", "http_status"),
            ("httpBody", "http_body"),
            ("httpHeader", "http_header"),
            ("httpHeaders", "http_headers"),
            ("httpUrl", "http_url"),
            ("httpIsSuccess", "http_is_success"),
            ("httpStatusText", "http_status_text"),
            ("httpContentType", "http_content_type"),
            ("httpContentLength", "http_content_length"),
            ("httpIsRedirect", "http_is_redirect"),
            ("httpIsClientError", "http_is_client_error"),
            ("httpIsServerError", "http_is_server_error"),
            ("httpSend", "http_send"),
            ("httpGet", "http_get"),
            ("httpPost", "http_post"),
            ("httpPut", "http_put"),
            ("httpDelete", "http_delete"),
            ("httpPatch", "http_patch"),
            ("httpPostJson", "http_post_json"),
            ("httpParseJson", "http_parse_json"),
            ("httpGetJson", "http_get_json"),
            ("httpCheckPermission", "http_check_permission"),
            // Future/Async
            ("futureResolve", "future_resolve"),
            ("futureReject", "future_reject"),
            ("futureNew", "future_new"),
            ("futureIsPending", "future_is_pending"),
            ("futureIsResolved", "future_is_resolved"),
            ("futureIsRejected", "future_is_rejected"),
            ("futureThen", "future_then"),
            ("futureCatch", "future_catch"),
            ("futureAll", "future_all"),
            ("futureRace", "future_race"),
            ("readFileAsync", "read_file_async"),
            ("writeFileAsync", "write_file_async"),
            ("appendFileAsync", "append_file_async"),
            ("httpSendAsync", "http_send_async"),
            ("httpGetAsync", "http_get_async"),
            ("httpPostAsync", "http_post_async"),
            ("httpPutAsync", "http_put_async"),
            ("httpDeleteAsync", "http_delete_async"),
            // Task/Channel
            ("taskJoin", "task_join"),
            ("taskStatus", "task_status"),
            ("taskCancel", "task_cancel"),
            ("taskId", "task_id"),
            ("taskName", "task_name"),
            ("joinAll", "join_all"),
            ("channelBounded", "channel_bounded"),
            ("channelUnbounded", "channel_unbounded"),
            ("channelSend", "channel_send"),
            ("channelReceive", "channel_receive"),
            ("channelSelect", "channel_select"),
            ("channelIsClosed", "channel_is_closed"),
            ("asyncMutex", "async_mutex"),
            ("asyncMutexGet", "async_mutex_get"),
            ("asyncMutexSet", "async_mutex_set"),
            // Process/Env
            ("getEnv", "get_env"),
            ("setEnv", "set_env"),
            ("unsetEnv", "unset_env"),
            ("listEnv", "list_env"),
            ("getCwd", "get_cwd"),
            ("getPid", "get_pid"),
            ("processNsSpawn", "spawn_process"),
            ("processNsStdin", "process_stdin"),
            ("processNsStdout", "process_stdout"),
            ("processNsStderr", "process_stderr"),
            ("processNsWaitFor", "process_wait"),
            ("processNsKill", "process_kill"),
            ("processNsIsRunning", "process_is_running"),
            ("processNsOutput", "process_output"),
            // Path
            ("pathJoinArray", "path_join_array"),
            ("pathParse", "path_parse"),
            ("pathNormalize", "path_normalize"),
            ("pathAbsolute", "path_absolute"),
            ("pathRelative", "path_relative"),
            ("pathParent", "path_parent"),
            ("pathBasename", "path_basename"),
            ("pathDirname", "path_dirname"),
            ("pathExtension", "path_extension"),
            ("pathIsAbsolute", "path_is_absolute"),
            ("pathIsRelative", "path_is_relative"),
            ("pathExists", "path_exists"),
            ("pathCanonical", "path_canonical"),
            ("pathEquals", "path_equals"),
            ("pathHomedir", "path_homedir"),
            ("pathCwd", "path_cwd"),
            ("pathTempdir", "path_tempdir"),
            ("pathSeparator", "path_separator"),
            ("pathDelimiter", "path_delimiter"),
            ("pathExtSeparator", "path_ext_separator"),
            ("pathDrive", "path_drive"),
            ("pathToPlatform", "path_to_platform"),
            ("pathToPosix", "path_to_posix"),
            ("pathToWindows", "path_to_windows"),
            // Filesystem — B24: all under fileNs* keys
            ("fileNsMkdir", "file_ns_mkdir"),
            ("fileNsMkdirp", "file_ns_mkdirp"),
            ("fileNsRmdir", "file_ns_rmdir"),
            ("fileNsRmdirRecursive", "file_ns_rmdir_recursive"),
            ("fileNsReadDir", "file_ns_read_dir"),
            ("fileNsWalk", "file_ns_walk"),
            ("fileNsFilterEntries", "file_ns_filter_entries"),
            ("fileNsSortEntries", "file_ns_sort_entries"),
            ("fileNsSize", "file_ns_size"),
            ("fileNsMtime", "file_ns_mtime"),
            ("fileNsCtime", "file_ns_ctime"),
            ("fileNsAtime", "file_ns_atime"),
            ("fileNsPermissions", "file_ns_permissions"),
            ("fileNsIsDir", "file_ns_is_dir"),
            ("fileNsIsFile", "file_ns_is_file"),
            ("fileNsIsSymlink", "file_ns_is_symlink"),
            ("fileNsInode", "file_ns_inode"),
            ("fileNsTempFile", "file_ns_temp_file"),
            ("fileNsTempDir", "file_ns_temp_dir"),
            ("fileNsSymlink", "file_ns_symlink"),
            ("fileNsReadLink", "file_ns_read_link"),
            ("fileNsWatch", "file_ns_watch"),
            ("fileNsWatchNext", "file_ns_watch_next"),
            // Compression
            ("gzipCompress", "gzip_compress"),
            ("gzipDecompress", "gzip_decompress"),
            ("gzipDecompressString", "gzip_decompress_string"),
            ("gzipIsGzip", "gzip_is_gzip"),
            ("gzipCompressionRatio", "gzip_compression_ratio"),
            ("tarCreate", "tar_create"),
            ("tarCreateGz", "tar_create_gz"),
            ("tarExtract", "tar_extract"),
            ("tarExtractGz", "tar_extract_gz"),
            ("tarList", "tar_list"),
            ("tarContains", "tar_contains"),
            ("zipCreate", "zip_create"),
            ("zipCreateWithComment", "zip_create_with_comment"),
            ("zipExtract", "zip_extract"),
            ("zipExtractFiles", "zip_extract_files"),
            ("zipList", "zip_list"),
            ("zipContains", "zip_contains"),
            ("zipCompressionRatio", "zip_compression_ratio"),
            ("zipAddFile", "zip_add_file"),
            ("zipValidate", "zip_validate"),
            ("zipComment", "zip_comment"),
            // Assertions
            ("assertFalse", "assert_false"),
            ("assertEqual", "assert_equal"),
            ("assertNotEqual", "assert_not_equal"),
            ("assertOk", "assert_ok"),
            ("assertErr", "assert_err"),
            ("assertSome", "assert_some"),
            ("assertNone", "assert_none"),
            ("assertContains", "assert_contains"),
            ("assertEmpty", "assert_empty"),
            ("assertLength", "assert_length"),
            ("assertThrows", "assert_throws"),
            ("assertNoThrow", "assert_no_throw"),
            // Crypto
            ("blake3Hash", "blake3_hash"),
            ("hmacSha256", "hmac_sha256"),
            ("hmacSha256Verify", "hmac_sha256_verify"),
            ("aesGcmEncrypt", "aes_gcm_encrypt"),
            ("aesGcmDecrypt", "aes_gcm_decrypt"),
            ("aesGcmGenerateKey", "aes_gcm_generate_key"),
            // Encoding
            ("base64Encode", "base64_encode"),
            ("base64Decode", "base64_decode"),
            ("base64UrlEncode", "base64_url_encode"),
            ("base64UrlDecode", "base64_url_decode"),
            ("hexEncode", "hex_encode"),
            ("hexDecode", "hex_decode"),
            ("urlEncode", "url_encode"),
            ("urlDecode", "url_decode"),
            // Networking
            ("tcpConnect", "tcp_connect"),
            ("tcpWrite", "tcp_write"),
            ("tcpRead", "tcp_read"),
            ("tcpReadBytes", "tcp_read_bytes"),
            ("tcpClose", "tcp_close"),
            ("tcpSetTimeout", "tcp_set_timeout"),
            ("tcpSetNodelay", "tcp_set_nodelay"),
            ("tcpLocalAddr", "tcp_local_addr"),
            ("tcpRemoteAddr", "tcp_remote_addr"),
            ("tcpListen", "tcp_listen"),
            ("tcpAccept", "tcp_accept"),
            ("tcpListenerAddr", "tcp_listener_addr"),
            ("tcpListenerClose", "tcp_listener_close"),
            ("udpBind", "udp_bind"),
            ("udpSend", "udp_send"),
            ("udpReceive", "udp_receive"),
            ("udpSetTimeout", "udp_set_timeout"),
            ("udpClose", "udp_close"),
            ("udpLocalAddr", "udp_local_addr"),
            ("tlsConnect", "tls_connect"),
            ("tlsWrite", "tls_write"),
            ("tlsRead", "tls_read"),
            ("tlsClose", "tls_close"),
            // Synchronization
            ("rwLockNew", "rw_lock_new"),
            ("rwLockRead", "rw_lock_read"),
            ("rwLockWrite", "rw_lock_write"),
            ("rwLockTryRead", "rw_lock_try_read"),
            ("rwLockTryWrite", "rw_lock_try_write"),
            ("semaphoreNew", "semaphore_new"),
            ("semaphoreAcquire", "semaphore_acquire"),
            ("semaphoreTryAcquire", "semaphore_try_acquire"),
            ("semaphoreRelease", "semaphore_release"),
            ("semaphoreAvailable", "semaphore_available"),
            ("atomicNew", "atomic_new"),
            ("atomicLoad", "atomic_load"),
            ("atomicStore", "atomic_store"),
            ("atomicAdd", "atomic_add"),
            ("atomicSub", "atomic_sub"),
            ("atomicCompareExchange", "atomic_compare_exchange"),
            // WebSocket
            ("wsConnect", "ws_connect"),
            ("wsSend", "ws_send"),
            ("wsSendBinary", "ws_send_binary"),
            ("wsReceive", "ws_receive"),
            ("wsPing", "ws_ping"),
            ("wsClose", "ws_close"),
        ];
        for &(camel, snake) in snake_case_aliases {
            if let Some(&func) = m.get(camel) {
                m.insert(snake, func);
            }
        }

        m
    })
}

/// Check if a function name is a builtin (stdlib function, not intrinsic)
pub fn is_builtin(name: &str) -> bool {
    builtin_registry().contains_key(name)
}

/// Check if a function name is an array intrinsic (handled in interpreter/VM)
pub fn is_array_intrinsic(name: &str) -> bool {
    matches!(
        name,
        "map"
            | "filter"
            | "reduce"
            | "forEach" | "for_each"
            | "find"
            | "findIndex" | "find_index"
            | "flatMap" | "flat_map"
            | "some"
            | "every"
            | "sort"
            | "sortBy" | "sort_by"
            // Result intrinsics (callback-based)
            | "result_map"
            | "result_map_err"
            | "result_and_then"
            | "result_or_else"
            // HashMap intrinsics (callback-based)
            | "hashMapForEach" | "hash_map_for_each"
            | "hashMapMap" | "hash_map_map"
            | "hashMapFilter" | "hash_map_filter"
            // HashSet intrinsics (callback-based)
            | "hashSetForEach" | "hash_set_for_each"
            | "hashSetMap" | "hash_set_map"
            | "hashSetFilter" | "hash_set_filter"
            // Regex intrinsics (callback-based)
            | "regexReplaceWith" | "regex_replace_with"
            | "regexReplaceAllWith" | "regex_replace_all_with"
            // Test intrinsics (callback-based)
            | "assertThrows" | "assert_throws"
            | "assertNoThrow" | "assert_no_throw"
    )
}

/// Extract string from value
fn extract_string<'a>(
    value: &'a Value,
    func_name: &str,
    span: crate::span::Span,
) -> Result<&'a str, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_ref()),
        _ => Err(stdlib_arg_error(func_name, "string", value, span)),
    }
}

/// Extract number from value
fn extract_number(
    value: &Value,
    func_name: &str,
    span: crate::span::Span,
) -> Result<f64, RuntimeError> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(stdlib_arg_error(func_name, "number", value, span)),
    }
}

/// Extract array from value (clones elements from the mutex-guarded vec)
fn extract_array(
    value: &Value,
    func_name: &str,
    span: crate::span::Span,
) -> Result<Vec<Value>, RuntimeError> {
    match value {
        Value::Array(arr) => Ok(arr.as_slice().to_vec()),
        _ => Err(stdlib_arg_error(func_name, "array", value, span)),
    }
}

/// Call a builtin function by name
pub fn call_builtin(
    name: &str,
    args: &[Value],
    call_span: crate::span::Span,
    security: &SecurityContext,
    output: &OutputWriter,
) -> Result<Value, RuntimeError> {
    match builtin_registry().get(name) {
        Some(dispatch_fn) => dispatch_fn(args, call_span, security, output),
        None => Err(RuntimeError::UnknownFunction {
            name: name.to_string(),
            span: call_span,
        }),
    }
}

/// Print a value to the configured output writer.
///
/// Only accepts string, number, bool, or null per stdlib specification.
pub fn print(
    value: &Value,
    span: crate::span::Span,
    output: &OutputWriter,
) -> Result<(), RuntimeError> {
    let mut w = output.lock().map_err(|_| RuntimeError::TypeError {
        msg: "output lock poisoned".into(),
        span,
    })?;
    writeln!(w, "{}", value.to_display_string()).map_err(|_| RuntimeError::TypeError {
        msg: "write failed".into(),
        span,
    })?;
    Ok(())
}

/// Get the length of a string or array
///
/// For strings, returns Unicode scalar count (not byte length).
/// For arrays, returns element count.
pub fn len(value: &Value, span: crate::span::Span) -> Result<f64, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.chars().count() as f64), // Unicode scalar count
        Value::Array(arr) => Ok(arr.len() as f64),
        _ => Err(stdlib_arg_error("len", "string or array", value, span)),
    }
}

/// Convert a value to a string
///
/// Only accepts number, bool, or null per stdlib specification.
pub fn str(value: &Value, _span: crate::span::Span) -> Result<String, RuntimeError> {
    Ok(value.to_display_string())
}
