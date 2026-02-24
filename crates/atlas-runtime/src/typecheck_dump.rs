//! Typecheck dump for AI-friendly JSON output
//!
//! Provides a stable JSON representation of inferred types and symbol bindings
//! for AI agents to analyze and understand type checking results.

use crate::symbol::{SymbolKind, SymbolTable};
use crate::types::Type;
use serde::{Deserialize, Serialize};

/// Typecheck dump schema version
pub const TYPECHECK_VERSION: u32 = 1;

/// Symbol information for typecheck dump
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolInfo {
    /// Symbol name
    pub name: String,
    /// Symbol kind (variable, parameter, function)
    pub kind: String,
    /// Start position in source
    pub start: usize,
    /// End position in source
    pub end: usize,
    /// Inferred or declared type
    #[serde(rename = "type")]
    pub ty: String,
    /// Whether the symbol is mutable
    pub mutable: bool,
}

/// Type information for typecheck dump
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Type name/description
    pub name: String,
    /// Kind of type (primitive, array, function)
    pub kind: String,
    /// Additional type details (for arrays, functions, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Typecheck dump output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypecheckDump {
    /// Typecheck dump schema version
    pub typecheck_version: u32,
    /// Symbols in the program
    pub symbols: Vec<SymbolInfo>,
    /// Types encountered during type checking
    pub types: Vec<TypeInfo>,
}

impl TypecheckDump {
    /// Create a new typecheck dump
    pub fn new() -> Self {
        Self {
            typecheck_version: TYPECHECK_VERSION,
            symbols: Vec::new(),
            types: Vec::new(),
        }
    }

    /// Create a typecheck dump from a symbol table
    pub fn from_symbol_table(symbol_table: &SymbolTable) -> Self {
        let mut dump = Self::new();

        // Collect all symbols
        dump.symbols = symbol_table
            .all_symbols()
            .iter()
            .map(|symbol| SymbolInfo {
                name: symbol.name.clone(),
                kind: symbol_kind_to_string(&symbol.kind),
                start: symbol.span.start,
                end: symbol.span.end,
                ty: type_to_string(&symbol.ty),
                mutable: symbol.mutable,
            })
            .collect();

        // Sort symbols by position, then by name for deterministic output
        dump.symbols
            .sort_by(|a, b| a.start.cmp(&b.start).then(a.name.cmp(&b.name)));

        // Collect unique types
        let mut type_names = std::collections::HashSet::new();
        for symbol in symbol_table.all_symbols() {
            collect_types(&symbol.ty, &mut type_names);
        }

        dump.types = type_names
            .into_iter()
            .map(|type_name| {
                let (kind, details) = parse_type_info(&type_name);
                TypeInfo {
                    name: type_name,
                    kind,
                    details,
                }
            })
            .collect();

        // Sort types by name for deterministic output
        dump.types.sort_by(|a, b| a.name.cmp(&b.name));

        dump
    }

    /// Convert to JSON string (pretty-printed)
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert to compact JSON string
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl Default for TypecheckDump {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert symbol kind to string
fn symbol_kind_to_string(kind: &SymbolKind) -> String {
    match kind {
        SymbolKind::Variable => "variable".to_string(),
        SymbolKind::Parameter => "parameter".to_string(),
        SymbolKind::Function => "function".to_string(),
        SymbolKind::Builtin => "builtin".to_string(),
    }
}

/// Convert type to string representation
fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Never => "never".to_string(),
        Type::Number => "number".to_string(),
        Type::String => "string".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Void => "void".to_string(),
        Type::Null => "null".to_string(),
        Type::Array(elem) => format!("{}[]", type_to_string(elem)),
        Type::Function {
            params,
            return_type,
            ..
        } => {
            let param_types: Vec<String> = params.iter().map(type_to_string).collect();
            format!(
                "({}) -> {}",
                param_types.join(", "),
                type_to_string(return_type)
            )
        }
        Type::JsonValue => "json".to_string(),
        Type::Generic { name, type_args } => {
            let args = type_args
                .iter()
                .map(type_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", name, args)
        }
        Type::Alias {
            name, type_args, ..
        } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let args = type_args
                    .iter()
                    .map(type_to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, args)
            }
        }
        Type::TypeParameter { name } => name.clone(),
        Type::Unknown => "unknown".to_string(),
        Type::Extern(extern_type) => extern_type.display_name().to_string(),
        Type::Union(members) => members
            .iter()
            .map(type_to_string)
            .collect::<Vec<_>>()
            .join(" | "),
        Type::Intersection(members) => members
            .iter()
            .map(type_to_string)
            .collect::<Vec<_>>()
            .join(" & "),
        Type::Structural { members } => {
            let parts = members
                .iter()
                .map(|member| format!("{}: {}", member.name, type_to_string(&member.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ {} }}", parts)
        }
    }
}

/// Collect all types mentioned in a type (including nested types)
fn collect_types(ty: &Type, types: &mut std::collections::HashSet<String>) {
    let type_str = type_to_string(ty);
    types.insert(type_str);

    match ty {
        Type::Array(elem) => collect_types(elem, types),
        Type::Function {
            params,
            return_type,
            ..
        } => {
            for param in params {
                collect_types(param, types);
            }
            collect_types(return_type, types);
        }
        Type::Generic { type_args, .. } => {
            for arg in type_args {
                collect_types(arg, types);
            }
        }
        Type::Alias {
            type_args, target, ..
        } => {
            for arg in type_args {
                collect_types(arg, types);
            }
            collect_types(target, types);
        }
        Type::Extern(_) => {
            // Extern types are primitives, no nested types to collect
        }
        Type::Structural { members } => {
            for member in members {
                collect_types(&member.ty, types);
            }
        }
        _ => {}
    }
}

/// Parse type information into kind and details
fn parse_type_info(type_name: &str) -> (String, Option<String>) {
    if let Some(stripped) = type_name.strip_suffix("[]") {
        (
            "array".to_string(),
            Some(format!("element type: {}", stripped)),
        )
    } else if type_name.contains("->") {
        (
            "function".to_string(),
            Some(format!("signature: {}", type_name)),
        )
    } else {
        ("primitive".to_string(), None)
    }
}
