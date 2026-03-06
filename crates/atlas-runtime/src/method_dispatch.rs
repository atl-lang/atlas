//! Shared method dispatch table for interpreter/VM parity.
//!
//! Both the interpreter and compiler consult this module to map
//! (TypeTag, method_name) → stdlib function name.

use serde::{Deserialize, Serialize};

/// Runtime-stable type tag for method dispatch.
/// Mirrors the types that support method call syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeTag {
    JsonValue,
    Array,
    HttpResponse,
    String,
    HashMap,
    Option,
    Result,
}

/// Resolve a method call to its stdlib function name.
/// Returns None if the type/method combination is not registered.
pub fn resolve_method(type_tag: TypeTag, method_name: &str) -> Option<String> {
    match type_tag {
        TypeTag::JsonValue => Some(format!("json{}", capitalize_first(method_name))),
        TypeTag::Array => resolve_array_method(method_name),
        TypeTag::HttpResponse => resolve_http_response_method(method_name),
        TypeTag::String => resolve_string_method(method_name),
        TypeTag::HashMap => resolve_hashmap_method(method_name),
        TypeTag::Option => resolve_option_method(method_name),
        TypeTag::Result => resolve_result_method(method_name),
    }
}

/// Resolve an HttpResponse method call to its stdlib function name.
fn resolve_http_response_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "status" => "httpStatus",
        "body" => "httpBody",
        "headers" => "httpHeaders",
        "header" => "httpHeader",
        "url" => "httpUrl",
        "isSuccess" => "httpIsSuccess",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve a string method call to its stdlib function name.
fn resolve_string_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        // Core methods
        "length" => "len",
        "charAt" => "charAt",
        "substring" => "substring",
        // Search methods
        "indexOf" => "indexOf",
        "lastIndexOf" => "lastIndexOf",
        "includes" => "includes",
        "startsWith" => "startsWith",
        "endsWith" => "endsWith",
        // Transform methods
        "toUpperCase" => "toUpperCase",
        "toLowerCase" => "toLowerCase",
        "trim" => "trim",
        "trimStart" => "trimStart",
        "trimEnd" => "trimEnd",
        "repeat" => "repeat",
        "replace" => "replace",
        "split" => "split",
        // Padding methods
        "padStart" => "padStart",
        "padEnd" => "padEnd",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve an array method call to its stdlib function name.
fn resolve_array_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        // Mutating methods — write back to receiver
        "push" => "arrayPush",
        "pop" => "arrayPop",
        "shift" => "arrayShift",
        "unshift" => "arrayUnshift",
        "reverse" => "arrayReverse",
        // Non-mutating — return new value, receiver unchanged
        "sort" => "arraySort",
        "len" | "length" => "len",
        "includes" => "arrayIncludes",
        "indexOf" => "arrayIndexOf",
        "lastIndexOf" => "arrayLastIndexOf",
        "slice" => "slice",
        "concat" => "concat",
        "flatten" => "flatten",
        "join" => "join",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve a HashMap method call to its stdlib function name.
/// HashMap mutates in-place via Arc<Mutex> — no CoW write-back needed.
fn resolve_hashmap_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        // Read methods
        "get" => "hashMapGet",
        "has" | "containsKey" => "hashMapHas",
        "size" | "len" => "hashMapSize",
        "isEmpty" => "hashMapIsEmpty",
        "keys" => "hashMapKeys",
        "values" => "hashMapValues",
        "entries" => "hashMapEntries",
        // Mutating methods (in-place via Arc<Mutex> — no write-back needed)
        "set" | "put" => "hashMapPut",
        "remove" | "delete" => "hashMapRemove",
        "clear" => "hashMapClear",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve an Option<T> method call to its stdlib function name.
fn resolve_option_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "unwrap" => "unwrap",
        "unwrapOr" => "unwrapOr",
        "isSome" => "isSome",
        "isNone" => "isNone",
        "map" => "optionMap",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve a Result<T,E> method call to its stdlib function name.
fn resolve_result_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "unwrap" => "unwrap",
        "unwrapOr" => "unwrapOr",
        "isOk" => "isOk",
        "isErr" => "isErr",
        "map" => "resultMap",
        "mapErr" => "resultMapErr",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Returns true if a stdlib function name is a mutating array method (returns modified collection).
pub fn is_array_mutating_collection(func_name: &str) -> bool {
    matches!(func_name, "arrayPush" | "arrayUnshift" | "arrayReverse")
}

/// Returns true if a stdlib function name is a mutating array method that returns a pair
/// `[extracted_value, new_array]` (pop/shift pattern).
pub fn is_array_mutating_pair(func_name: &str) -> bool {
    matches!(func_name, "arrayPop" | "arrayShift")
}

/// Capitalize first letter of each snake_case segment and join.
///
/// "as_string" → "AsString"
/// "is_null" → "IsNull"
fn capitalize_first(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}
