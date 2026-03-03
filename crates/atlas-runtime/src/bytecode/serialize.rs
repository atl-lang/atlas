//! Bytecode serialization and deserialization
//!
//! Defines the binary encoding for Atlas values in the constant pool.
//! All multi-byte integers use big-endian encoding.
//!
//! ## Type Tags (1 byte)
//! - 0x00: Null
//! - 0x01: Bool
//! - 0x02: Number (f64)
//! - 0x03: String
//! - 0x04: Function
//! - 0x05: Builtin
//! - 0x06: Array
//! - 0x07: Option
//! - 0x08: Result
//! - 0x09: HashMap
//! - 0x0A: HashSet
//! - 0x0B: Queue
//! - 0x0C: Stack
//! - 0x0D: Regex
//! - 0x0E: DateTime
//!
//! Runtime-only types (panic on serialize): NativeFunction, JsonValue,
//! HttpRequest, HttpResponse, Future, TaskHandle, Channel*, AsyncMutex,
//! Closure, SharedValue.

use crate::span::Span;
use crate::stdlib::collections::hash::HashKey;
use crate::stdlib::collections::hashmap::AtlasHashMap;
use crate::stdlib::collections::hashset::AtlasHashSet;
use crate::stdlib::collections::queue::AtlasQueue;
use crate::stdlib::collections::stack::AtlasStack;
use crate::value::{Value, ValueArray, ValueHashMap, ValueHashSet, ValueQueue, ValueStack};

/// Value type tags for serialization
mod tags {
    pub const NULL: u8 = 0x00;
    pub const BOOL: u8 = 0x01;
    pub const NUMBER: u8 = 0x02;
    pub const STRING: u8 = 0x03;
    pub const FUNCTION: u8 = 0x04;
    pub const BUILTIN: u8 = 0x05;
    pub const ARRAY: u8 = 0x06;
    pub const OPTION: u8 = 0x07;
    pub const RESULT: u8 = 0x08;
    pub const HASHMAP: u8 = 0x09;
    pub const HASHSET: u8 = 0x0A;
    pub const QUEUE: u8 = 0x0B;
    pub const STACK: u8 = 0x0C;
    pub const REGEX: u8 = 0x0D;
    pub const DATETIME: u8 = 0x0E;
}

/// Serialize a Value to bytes
pub(super) fn serialize_value(value: &Value, bytes: &mut Vec<u8>) {
    match value {
        Value::Null => {
            bytes.push(tags::NULL);
        }
        Value::Bool(b) => {
            bytes.push(tags::BOOL);
            bytes.push(if *b { 1 } else { 0 });
        }
        Value::Number(n) => {
            bytes.push(tags::NUMBER);
            bytes.extend_from_slice(&n.to_be_bytes());
        }
        Value::String(s) => {
            bytes.push(tags::STRING);
            serialize_string(s, bytes);
        }
        Value::Function(func) => {
            bytes.push(tags::FUNCTION);
            // Serialize function name
            serialize_string(&func.name, bytes);
            // Serialize arity
            bytes.push(func.arity as u8);
            // Serialize bytecode offset
            bytes.extend_from_slice(&(func.bytecode_offset as u32).to_be_bytes());
            // Serialize local_count (u16 - needed for VM frame allocation)
            bytes.extend_from_slice(&(func.local_count as u16).to_be_bytes());
            // Serialize param_ownership: count (1 byte) + each annotation (1 byte)
            bytes.push(func.param_ownership.len() as u8);
            for ann in &func.param_ownership {
                bytes.push(serialize_ownership(ann));
            }
            // Serialize param_names: count (1 byte) + each name as (len u16 + bytes)
            bytes.push(func.param_names.len() as u8);
            for pname in &func.param_names {
                let pname_bytes = pname.as_bytes();
                bytes.extend_from_slice(&(pname_bytes.len() as u16).to_be_bytes());
                bytes.extend_from_slice(pname_bytes);
            }
            // Serialize return_ownership (1 byte)
            bytes.push(serialize_ownership(&func.return_ownership));
        }
        Value::Builtin(name) => {
            bytes.push(tags::BUILTIN);
            serialize_string(name, bytes);
        }
        Value::Array(arr) => {
            bytes.push(tags::ARRAY);
            let slice = arr.as_slice();
            bytes.extend_from_slice(&(slice.len() as u32).to_be_bytes());
            for elem in slice {
                serialize_value(elem, bytes);
            }
        }
        Value::Option(opt) => {
            bytes.push(tags::OPTION);
            match opt {
                None => bytes.push(0),
                Some(inner) => {
                    bytes.push(1);
                    serialize_value(inner, bytes);
                }
            }
        }
        Value::Result(res) => {
            bytes.push(tags::RESULT);
            match res {
                Ok(v) => {
                    bytes.push(0); // Ok tag
                    serialize_value(v, bytes);
                }
                Err(e) => {
                    bytes.push(1); // Err tag
                    serialize_value(e, bytes);
                }
            }
        }
        Value::HashMap(map) => {
            bytes.push(tags::HASHMAP);
            let entries = map.inner().entries();
            bytes.extend_from_slice(&(entries.len() as u32).to_be_bytes());
            for (k, v) in entries {
                serialize_hashkey(&k, bytes);
                serialize_value(&v, bytes);
            }
        }
        Value::HashSet(set) => {
            bytes.push(tags::HASHSET);
            let elements = set.inner().to_vec();
            bytes.extend_from_slice(&(elements.len() as u32).to_be_bytes());
            for elem in elements {
                serialize_hashkey(&elem, bytes);
            }
        }
        Value::Queue(queue) => {
            bytes.push(tags::QUEUE);
            let elements = queue.inner().to_vec();
            bytes.extend_from_slice(&(elements.len() as u32).to_be_bytes());
            for elem in elements {
                serialize_value(&elem, bytes);
            }
        }
        Value::Stack(stack) => {
            bytes.push(tags::STACK);
            let elements = stack.inner().to_vec();
            bytes.extend_from_slice(&(elements.len() as u32).to_be_bytes());
            for elem in elements {
                serialize_value(&elem, bytes);
            }
        }
        Value::Regex(re) => {
            bytes.push(tags::REGEX);
            serialize_string(re.as_str(), bytes);
        }
        Value::DateTime(dt) => {
            bytes.push(tags::DATETIME);
            // Serialize as RFC3339 string for human-readability and timezone preservation
            let s = dt.to_rfc3339();
            serialize_string(&s, bytes);
        }
        // Runtime-only types cannot be serialized
        Value::NativeFunction(_) => {
            panic!("Cannot serialize native functions in bytecode constants");
        }
        Value::JsonValue(_) => {
            panic!("Cannot serialize JSON values in bytecode constants");
        }
        Value::HttpRequest(_) => {
            panic!("Cannot serialize HttpRequest values in bytecode constants");
        }
        Value::HttpResponse(_) => {
            panic!("Cannot serialize HttpResponse values in bytecode constants");
        }
        Value::Future(_) => {
            panic!("Cannot serialize Future values in bytecode constants");
        }
        Value::TaskHandle(_) => {
            panic!("Cannot serialize TaskHandle values in bytecode constants");
        }
        Value::ChannelSender(_) => {
            panic!("Cannot serialize ChannelSender values in bytecode constants");
        }
        Value::ChannelReceiver(_) => {
            panic!("Cannot serialize ChannelReceiver values in bytecode constants");
        }
        Value::AsyncMutex(_) => {
            panic!("Cannot serialize AsyncMutex values in bytecode constants");
        }
        Value::Closure(_) => {
            panic!("Cannot serialize Closure values in bytecode constants");
        }
        Value::SharedValue(_) => {
            panic!("Cannot serialize SharedValue in bytecode constants");
        }
    }
}

/// Serialize ownership annotation
fn serialize_ownership(ann: &Option<crate::ast::OwnershipAnnotation>) -> u8 {
    match ann {
        None => 0,
        Some(crate::ast::OwnershipAnnotation::Own) => 1,
        Some(crate::ast::OwnershipAnnotation::Borrow) => 2,
        Some(crate::ast::OwnershipAnnotation::Shared) => 3,
    }
}

/// Deserialize ownership annotation
fn deserialize_ownership(b: u8) -> Option<crate::ast::OwnershipAnnotation> {
    match b {
        1 => Some(crate::ast::OwnershipAnnotation::Own),
        2 => Some(crate::ast::OwnershipAnnotation::Borrow),
        3 => Some(crate::ast::OwnershipAnnotation::Shared),
        _ => None,
    }
}

/// Helper to serialize a string with u32 length prefix
fn serialize_string(s: &str, bytes: &mut Vec<u8>) {
    let s_bytes = s.as_bytes();
    bytes.extend_from_slice(&(s_bytes.len() as u32).to_be_bytes());
    bytes.extend_from_slice(s_bytes);
}

/// Helper to deserialize a string with u32 length prefix
fn deserialize_string(bytes: &[u8]) -> Result<(String, usize), String> {
    if bytes.len() < 4 {
        return Err("Truncated string length".to_string());
    }
    let len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    if bytes.len() < 4 + len {
        return Err("Truncated string data".to_string());
    }
    let s = String::from_utf8(bytes[4..4 + len].to_vec())
        .map_err(|e| format!("Invalid UTF-8 in string: {}", e))?;
    Ok((s, 4 + len))
}

/// Serialize a HashKey to bytes
fn serialize_hashkey(key: &HashKey, bytes: &mut Vec<u8>) {
    match key {
        HashKey::Null => bytes.push(0),
        HashKey::Bool(b) => {
            bytes.push(1);
            bytes.push(if *b { 1 } else { 0 });
        }
        HashKey::Number(n) => {
            bytes.push(2);
            bytes.extend_from_slice(&n.0.to_be_bytes());
        }
        HashKey::String(s) => {
            bytes.push(3);
            serialize_string(s, bytes);
        }
    }
}

/// Deserialize a HashKey from bytes
fn deserialize_hashkey(bytes: &[u8]) -> Result<(HashKey, usize), String> {
    if bytes.is_empty() {
        return Err("Truncated hashkey".to_string());
    }
    match bytes[0] {
        0 => Ok((HashKey::Null, 1)),
        1 => {
            if bytes.len() < 2 {
                return Err("Truncated hashkey bool".to_string());
            }
            Ok((HashKey::Bool(bytes[1] != 0), 2))
        }
        2 => {
            if bytes.len() < 9 {
                return Err("Truncated hashkey number".to_string());
            }
            let num_bytes: [u8; 8] = bytes[1..9].try_into().unwrap();
            Ok((
                HashKey::Number(ordered_float::OrderedFloat(f64::from_be_bytes(num_bytes))),
                9,
            ))
        }
        3 => {
            let (s, consumed) = deserialize_string(&bytes[1..])?;
            Ok((HashKey::String(std::sync::Arc::new(s)), 1 + consumed))
        }
        t => Err(format!("Invalid hashkey tag: {}", t)),
    }
}

/// Deserialize a Value from bytes, returns (Value, bytes_consumed)
pub(super) fn deserialize_value(bytes: &[u8]) -> Result<(Value, usize), String> {
    if bytes.is_empty() {
        return Err("Unexpected end of data while reading value".to_string());
    }

    let tag = bytes[0];
    let rest = &bytes[1..];

    match tag {
        tags::NULL => Ok((Value::Null, 1)),

        tags::BOOL => {
            if rest.is_empty() {
                return Err("Truncated bool value".to_string());
            }
            Ok((Value::Bool(rest[0] != 0), 2))
        }

        tags::NUMBER => {
            if rest.len() < 8 {
                return Err("Truncated number value".to_string());
            }
            let num_bytes: [u8; 8] = rest[0..8].try_into().unwrap();
            Ok((Value::Number(f64::from_be_bytes(num_bytes)), 9))
        }

        tags::STRING => {
            let (s, consumed) = deserialize_string(rest)?;
            Ok((Value::string(&s), 1 + consumed))
        }

        tags::FUNCTION => {
            let (name, mut cursor) = deserialize_string(rest)?;
            if rest.len() < cursor + 1 + 4 + 2 + 1 {
                return Err("Truncated function data".to_string());
            }
            let arity = rest[cursor] as usize;
            cursor += 1;
            let offset = u32::from_be_bytes([
                rest[cursor],
                rest[cursor + 1],
                rest[cursor + 2],
                rest[cursor + 3],
            ]) as usize;
            cursor += 4;
            let local_count = u16::from_be_bytes([rest[cursor], rest[cursor + 1]]) as usize;
            cursor += 2;

            // Deserialize param_ownership
            let param_count = rest[cursor] as usize;
            cursor += 1;
            if rest.len() < cursor + param_count {
                return Err("Truncated function ownership data".to_string());
            }
            let param_ownership: Vec<Option<crate::ast::OwnershipAnnotation>> = rest
                [cursor..cursor + param_count]
                .iter()
                .map(|&b| deserialize_ownership(b))
                .collect();
            cursor += param_count;

            // Deserialize param_names
            if rest.len() < cursor + 1 {
                return Err("Truncated param_names count".to_string());
            }
            let names_count = rest[cursor] as usize;
            cursor += 1;
            let mut param_names: Vec<String> = Vec::with_capacity(names_count);
            for _ in 0..names_count {
                if rest.len() < cursor + 2 {
                    return Err("Truncated param name length".to_string());
                }
                let plen = u16::from_be_bytes([rest[cursor], rest[cursor + 1]]) as usize;
                cursor += 2;
                if rest.len() < cursor + plen {
                    return Err("Truncated param name bytes".to_string());
                }
                let pname = String::from_utf8(rest[cursor..cursor + plen].to_vec())
                    .map_err(|e| format!("Invalid UTF-8 in param name: {}", e))?;
                param_names.push(pname);
                cursor += plen;
            }

            // Deserialize return_ownership
            if rest.len() < cursor + 1 {
                return Err("Truncated return_ownership".to_string());
            }
            let return_ownership = deserialize_ownership(rest[cursor]);
            cursor += 1;

            Ok((
                Value::Function(crate::value::FunctionRef {
                    name,
                    arity,
                    bytecode_offset: offset,
                    local_count,
                    param_ownership,
                    param_names,
                    return_ownership,
                }),
                1 + cursor,
            ))
        }

        tags::BUILTIN => {
            let (name, consumed) = deserialize_string(rest)?;
            Ok((
                Value::Builtin(std::sync::Arc::from(name.as_str())),
                1 + consumed,
            ))
        }

        tags::ARRAY => {
            if rest.len() < 4 {
                return Err("Truncated array length".to_string());
            }
            let count = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
            let mut cursor = 4;
            let mut elements = Vec::with_capacity(count);
            for _ in 0..count {
                let (elem, consumed) = deserialize_value(&rest[cursor..])?;
                elements.push(elem);
                cursor += consumed;
            }
            Ok((Value::Array(ValueArray::from_vec(elements)), 1 + cursor))
        }

        tags::OPTION => {
            if rest.is_empty() {
                return Err("Truncated option tag".to_string());
            }
            match rest[0] {
                0 => Ok((Value::Option(None), 2)),
                1 => {
                    let (inner, consumed) = deserialize_value(&rest[1..])?;
                    Ok((Value::Option(Some(Box::new(inner))), 2 + consumed))
                }
                t => Err(format!("Invalid option tag: {}", t)),
            }
        }

        tags::RESULT => {
            if rest.is_empty() {
                return Err("Truncated result tag".to_string());
            }
            match rest[0] {
                0 => {
                    let (inner, consumed) = deserialize_value(&rest[1..])?;
                    Ok((Value::Result(Ok(Box::new(inner))), 2 + consumed))
                }
                1 => {
                    let (inner, consumed) = deserialize_value(&rest[1..])?;
                    Ok((Value::Result(Err(Box::new(inner))), 2 + consumed))
                }
                t => Err(format!("Invalid result tag: {}", t)),
            }
        }

        tags::HASHMAP => {
            if rest.len() < 4 {
                return Err("Truncated hashmap length".to_string());
            }
            let count = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
            let mut cursor = 4;
            let mut map = AtlasHashMap::new();
            for _ in 0..count {
                let (key, key_consumed) = deserialize_hashkey(&rest[cursor..])?;
                cursor += key_consumed;
                let (val, val_consumed) = deserialize_value(&rest[cursor..])?;
                cursor += val_consumed;
                map.insert(key, val);
            }
            Ok((Value::HashMap(ValueHashMap::from_atlas(map)), 1 + cursor))
        }

        tags::HASHSET => {
            if rest.len() < 4 {
                return Err("Truncated hashset length".to_string());
            }
            let count = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
            let mut cursor = 4;
            let mut set = AtlasHashSet::new();
            for _ in 0..count {
                let (elem, consumed) = deserialize_hashkey(&rest[cursor..])?;
                cursor += consumed;
                set.insert(elem);
            }
            Ok((Value::HashSet(ValueHashSet::from_atlas(set)), 1 + cursor))
        }

        tags::QUEUE => {
            if rest.len() < 4 {
                return Err("Truncated queue length".to_string());
            }
            let count = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
            let mut cursor = 4;
            let mut queue = AtlasQueue::new();
            for _ in 0..count {
                let (elem, consumed) = deserialize_value(&rest[cursor..])?;
                cursor += consumed;
                queue.enqueue(elem);
            }
            Ok((Value::Queue(ValueQueue::from_atlas(queue)), 1 + cursor))
        }

        tags::STACK => {
            if rest.len() < 4 {
                return Err("Truncated stack length".to_string());
            }
            let count = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
            let mut cursor = 4;
            let mut stack = AtlasStack::new();
            for _ in 0..count {
                let (elem, consumed) = deserialize_value(&rest[cursor..])?;
                cursor += consumed;
                stack.push(elem);
            }
            Ok((Value::Stack(ValueStack::from_atlas(stack)), 1 + cursor))
        }

        tags::REGEX => {
            let (pattern, consumed) = deserialize_string(rest)?;
            let re =
                regex::Regex::new(&pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;
            Ok((Value::Regex(std::sync::Arc::new(re)), 1 + consumed))
        }

        tags::DATETIME => {
            let (s, consumed) = deserialize_string(rest)?;
            let dt = chrono::DateTime::parse_from_rfc3339(&s)
                .map_err(|e| format!("Invalid datetime: {}", e))?
                .with_timezone(&chrono::Utc);
            Ok((Value::DateTime(std::sync::Arc::new(dt)), 1 + consumed))
        }

        _ => Err(format!("Unknown value type tag: {:#x}", tag)),
    }
}

/// Serialize a Span to bytes
pub(super) fn serialize_span(span: &Span, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&(span.start as u32).to_be_bytes());
    bytes.extend_from_slice(&(span.end as u32).to_be_bytes());
}

/// Deserialize a Span from bytes, returns (Span, bytes_consumed)
pub(super) fn deserialize_span(bytes: &[u8]) -> Result<(Span, usize), String> {
    if bytes.len() < 8 {
        return Err("Truncated span data".to_string());
    }
    let start = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let end = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    Ok((Span { start, end }, 8))
}

/// Compute CRC32 checksum for bytecode integrity verification
pub(super) fn compute_checksum(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_null() {
        let mut bytes = Vec::new();
        serialize_value(&Value::Null, &mut bytes);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(val, Value::Null);
    }

    #[test]
    fn test_roundtrip_bool() {
        for b in [true, false] {
            let mut bytes = Vec::new();
            serialize_value(&Value::Bool(b), &mut bytes);
            let (val, consumed) = deserialize_value(&bytes).unwrap();
            assert_eq!(consumed, bytes.len());
            assert_eq!(val, Value::Bool(b));
        }
    }

    #[test]
    fn test_roundtrip_number() {
        for n in [0.0, 1.0, -1.0, 3.14159, f64::MAX, f64::MIN] {
            let mut bytes = Vec::new();
            serialize_value(&Value::Number(n), &mut bytes);
            let (val, consumed) = deserialize_value(&bytes).unwrap();
            assert_eq!(consumed, bytes.len());
            assert_eq!(val, Value::Number(n));
        }
    }

    #[test]
    fn test_roundtrip_string() {
        for s in ["", "hello", "hello world", "emoji: 🎉", "multi\nline"] {
            let mut bytes = Vec::new();
            serialize_value(&Value::string(s), &mut bytes);
            let (val, consumed) = deserialize_value(&bytes).unwrap();
            assert_eq!(consumed, bytes.len());
            assert_eq!(val, Value::string(s));
        }
    }

    #[test]
    fn test_roundtrip_array() {
        let arr = Value::Array(ValueArray::from_vec(vec![
            Value::Number(1.0),
            Value::string("two"),
            Value::Bool(true),
        ]));
        let mut bytes = Vec::new();
        serialize_value(&arr, &mut bytes);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(val, arr);
    }

    #[test]
    fn test_roundtrip_nested_array() {
        let inner = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
        let outer = Value::Array(ValueArray::from_vec(vec![inner.clone(), Value::Null]));
        let mut bytes = Vec::new();
        serialize_value(&outer, &mut bytes);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(val, outer);
    }

    #[test]
    fn test_roundtrip_option() {
        // None
        let none = Value::Option(None);
        let mut bytes = Vec::new();
        serialize_value(&none, &mut bytes);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(val, none);

        // Some
        let some = Value::Option(Some(Box::new(Value::Number(42.0))));
        let mut bytes = Vec::new();
        serialize_value(&some, &mut bytes);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(val, some);
    }

    #[test]
    fn test_roundtrip_result() {
        // Ok
        let ok = Value::Result(Ok(Box::new(Value::string("success"))));
        let mut bytes = Vec::new();
        serialize_value(&ok, &mut bytes);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(val, ok);

        // Err
        let err = Value::Result(Err(Box::new(Value::string("error"))));
        let mut bytes = Vec::new();
        serialize_value(&err, &mut bytes);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(val, err);
    }

    #[test]
    fn test_roundtrip_hashmap() {
        let mut map = AtlasHashMap::new();
        map.insert(
            HashKey::String(std::sync::Arc::new("a".to_string())),
            Value::Number(1.0),
        );
        map.insert(
            HashKey::String(std::sync::Arc::new("b".to_string())),
            Value::string("two"),
        );
        let val = Value::HashMap(ValueHashMap::from_atlas(map));

        let mut bytes = Vec::new();
        serialize_value(&val, &mut bytes);
        let (result, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        if let Value::HashMap(result_map) = result {
            assert_eq!(result_map.inner().len(), 2);
            assert_eq!(
                result_map
                    .inner()
                    .get(&HashKey::String(std::sync::Arc::new("a".to_string()))),
                Some(&Value::Number(1.0))
            );
        } else {
            panic!("Expected HashMap");
        }
    }

    #[test]
    fn test_roundtrip_regex() {
        let re = Value::Regex(std::sync::Arc::new(regex::Regex::new(r"\d+").unwrap()));
        let mut bytes = Vec::new();
        serialize_value(&re, &mut bytes);
        let (val, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        if let Value::Regex(result_re) = val {
            assert_eq!(result_re.as_str(), r"\d+");
        } else {
            panic!("Expected Regex");
        }
    }

    #[test]
    fn test_roundtrip_datetime() {
        use chrono::TimeZone;
        let dt = chrono::Utc.with_ymd_and_hms(2026, 3, 2, 12, 0, 0).unwrap();
        let val = Value::DateTime(std::sync::Arc::new(dt));
        let mut bytes = Vec::new();
        serialize_value(&val, &mut bytes);
        let (result, consumed) = deserialize_value(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        if let Value::DateTime(result_dt) = result {
            assert_eq!(*result_dt, dt);
        } else {
            panic!("Expected DateTime");
        }
    }

    #[test]
    fn test_checksum() {
        let data = b"hello world";
        let checksum = compute_checksum(data);
        assert_eq!(checksum, compute_checksum(data)); // deterministic
        assert_ne!(checksum, compute_checksum(b"hello world!")); // different data
    }
}
