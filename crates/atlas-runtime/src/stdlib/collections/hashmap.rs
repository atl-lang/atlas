//! HashMap collection implementation

use super::hash::HashKey;
use crate::span::Span;
use crate::stdlib::{stdlib_arg_error, stdlib_arity_error};
use crate::value::{RuntimeError, Value, ValueArray, ValueHashMap};
use std::collections::HashMap;

/// Atlas HashMap - key-value collection with O(1) average operations
///
/// Uses Rust's standard HashMap internally with deterministic hashing.
/// Only hashable types (number, string, bool, null) can be used as keys.
#[derive(Debug, Clone, PartialEq)]
pub struct AtlasHashMap {
    inner: HashMap<HashKey, Value>,
}

impl AtlasHashMap {
    /// Create new empty HashMap with default capacity
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Create HashMap with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Insert key-value pair, returns previous value if existed
    pub fn insert(&mut self, key: HashKey, value: Value) -> Option<Value> {
        self.inner.insert(key, value)
    }

    /// Get value by key (returns None if not found)
    pub fn get(&self, key: &HashKey) -> Option<&Value> {
        self.inner.get(key)
    }

    /// Remove key-value pair, returns value if existed
    pub fn remove(&mut self, key: &HashKey) -> Option<Value> {
        self.inner.remove(key)
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &HashKey) -> bool {
        self.inner.contains_key(key)
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all entries
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get all keys as vector
    pub fn keys(&self) -> Vec<HashKey> {
        self.inner.keys().cloned().collect()
    }

    /// Get all values as vector
    pub fn values(&self) -> Vec<Value> {
        self.inner.values().cloned().collect()
    }

    /// Get all entries as vector of (key, value) pairs
    pub fn entries(&self) -> Vec<(HashKey, Value)> {
        self.inner
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl Default for AtlasHashMap {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn extract_array_ref<'a>(
    func_name: &str,
    value: &'a Value,
    span: Span,
) -> Result<&'a ValueArray, RuntimeError> {
    match value {
        Value::Array(arr) => Ok(arr),
        _ => Err(stdlib_arg_error(func_name, "array", value, span)),
    }
}

fn extract_hashmap_ref(value: &Value, span: Span) -> Result<&ValueHashMap, RuntimeError> {
    match value {
        Value::HashMap(map) => Ok(map),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected HashMap, got {}", value.type_name()),
            span,
        }),
    }
}

// ============================================================================
// Public stdlib functions
// ============================================================================

/// Create new empty HashMap
pub fn new_map(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("HashMap.new", 0, args.len(), span));
    }
    Ok(Value::HashMap(ValueHashMap::new()))
}

/// Create HashMap from array of [key, value] entries
pub fn from_entries(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error(
            "HashMap.fromEntries",
            1,
            args.len(),
            span,
        ));
    }

    let entries_array = extract_array_ref("HashMap.fromEntries", &args[0], span)?;
    let mut map = AtlasHashMap::new();

    for entry in entries_array.as_slice() {
        let pair = extract_array_ref("HashMap.fromEntries", entry, span)?;
        let pair_slice = pair.as_slice();

        if pair_slice.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "Entry must be [key, value] array with exactly 2 elements".to_string(),
                span,
            });
        }

        let key = HashKey::from_value(&pair_slice[0], span)?;
        let value = pair_slice[1].clone();
        map.insert(key, value);
    }

    Ok(Value::HashMap(ValueHashMap::from_atlas(map)))
}

/// Insert or update key-value pair. Mutates in place.
pub fn put(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(stdlib_arity_error("HashMap.put", 3, args.len(), span));
    }

    let key = HashKey::from_value(&args[1], span)?;
    let value = args[2].clone();

    let map = extract_hashmap_ref(&args[0], span)?;
    map.with_mut(|inner| {
        inner.insert(key, value);
    });

    Ok(args[0].clone())
}

/// Get value by key
pub fn get(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("HashMap.get", 2, args.len(), span));
    }

    let map = extract_hashmap_ref(&args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;

    let value = map.with(|inner| inner.get(&key).cloned());
    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

/// Remove key-value pair. Returns [Option<Value>, modified HashMap].
pub fn remove(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("HashMap.remove", 2, args.len(), span));
    }

    let key = HashKey::from_value(&args[1], span)?;

    let map = extract_hashmap_ref(&args[0], span)?;
    let removed = map.with_mut(|inner| inner.remove(&key));

    let removed_opt = match removed {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    };
    Ok(Value::array(vec![removed_opt, args[0].clone()]))
}

/// Check if key exists
pub fn has(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("HashMap.has", 2, args.len(), span));
    }

    let map = extract_hashmap_ref(&args[0], span)?;
    let key = HashKey::from_value(&args[1], span)?;

    Ok(Value::Bool(map.with(|inner| inner.contains_key(&key))))
}

/// Get number of entries
pub fn size(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashMap.size", 1, args.len(), span));
    }

    let map = extract_hashmap_ref(&args[0], span)?;
    let len = map.with(|inner| inner.len());
    Ok(Value::Number(len as f64))
}

/// Check if HashMap is empty
pub fn is_empty(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashMap.isEmpty", 1, args.len(), span));
    }

    let map = extract_hashmap_ref(&args[0], span)?;
    let empty = map.with(|inner| inner.is_empty());
    Ok(Value::Bool(empty))
}

/// Remove all entries. Mutates in place and returns the map.
pub fn clear(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashMap.clear", 1, args.len(), span));
    }

    let map = extract_hashmap_ref(&args[0], span)?;
    map.with_mut(|inner| inner.clear());
    Ok(args[0].clone())
}

/// Get all keys as array
pub fn keys(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashMap.keys", 1, args.len(), span));
    }

    let map = extract_hashmap_ref(&args[0], span)?;
    let keys: Vec<Value> =
        map.with(|inner| inner.keys().into_iter().map(|k| k.to_value()).collect());
    Ok(Value::array(keys))
}

/// Get all values as array
pub fn values(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashMap.values", 1, args.len(), span));
    }

    let map = extract_hashmap_ref(&args[0], span)?;
    let vals = map.with(|inner| inner.values());
    Ok(Value::array(vals))
}

/// Get all entries as array of [key, value] pairs
pub fn entries(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashMap.entries", 1, args.len(), span));
    }

    let map = extract_hashmap_ref(&args[0], span)?;
    let entries: Vec<Value> = map.with(|inner| {
        inner
            .entries()
            .into_iter()
            .map(|(k, v)| Value::array(vec![k.to_value(), v]))
            .collect()
    });
    Ok(Value::array(entries))
}

/// Create a deep copy of a HashMap.
pub fn copy(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("HashMap.copy", 1, args.len(), span));
    }

    let map = extract_hashmap_ref(&args[0], span)?;
    let copied = map.with(|inner| inner.clone());
    Ok(Value::HashMap(ValueHashMap::from_atlas(copied)))
}
