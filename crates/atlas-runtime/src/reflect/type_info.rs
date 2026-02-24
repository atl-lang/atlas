//! Type information for reflection
//!
//! TypeInfo provides runtime introspection of Atlas types, exposing structural
//! details like field names, function signatures, and array element types.

use crate::types::Type;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Kind of type for categorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeKind {
    /// Primitive number type
    Number,
    /// Primitive string type
    String,
    /// Primitive boolean type
    Bool,
    /// Null type
    Null,
    /// Void type (functions returning nothing)
    Void,
    /// Never type (no values)
    Never,
    /// Array type (has element_type)
    Array,
    /// Function type (has parameters and return_type)
    Function,
    /// JSON dynamic type
    JsonValue,
    /// Generic type (has type arguments)
    Generic,
    /// Type alias (has alias target)
    Alias,
    /// Type parameter (unresolved)
    TypeParameter,
    /// Unknown type (error recovery)
    Unknown,
    /// Extern type (FFI)
    Extern,
    /// Structural type (member requirements)
    Structural,
    /// Union type
    Union,
    /// Intersection type
    Intersection,
    /// Option type
    Option,
    /// Result type
    Result,
}

/// Field information for struct types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldInfo {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: TypeInfo,
}

/// Complete type information for runtime introspection
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Type name (e.g., "number", "string[]", "function")
    pub name: String,

    /// Type kind for categorization
    pub kind: TypeKind,

    /// For struct types: field names and types
    /// Empty for non-struct types
    pub fields: Vec<FieldInfo>,

    /// For function types: parameter types
    /// Empty for non-function types
    pub parameters: Vec<TypeInfo>,

    /// For function types: return type
    /// None for non-function types
    pub return_type: Option<Box<TypeInfo>>,

    /// For array types: element type
    /// None for non-array types
    pub element_type: Option<Box<TypeInfo>>,

    /// For generic types: type arguments
    /// Empty for non-generic types
    pub type_args: Vec<TypeInfo>,

    /// For alias types: underlying type information
    /// None for non-alias types
    pub alias_target: Option<Box<TypeInfo>>,
}

impl TypeInfo {
    /// Create TypeInfo from a Type
    pub fn from_type(ty: &Type) -> Self {
        match ty {
            Type::Number => TypeInfo {
                name: "number".to_string(),
                kind: TypeKind::Number,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: None,
            },

            Type::String => TypeInfo {
                name: "string".to_string(),
                kind: TypeKind::String,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: None,
            },

            Type::Bool => TypeInfo {
                name: "bool".to_string(),
                kind: TypeKind::Bool,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: None,
            },

            Type::Null => TypeInfo {
                name: "null".to_string(),
                kind: TypeKind::Null,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: None,
            },

            Type::Void => TypeInfo {
                name: "void".to_string(),
                kind: TypeKind::Void,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: None,
            },
            Type::Never => TypeInfo {
                name: "never".to_string(),
                kind: TypeKind::Never,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: None,
            },

            Type::Array(inner) => {
                let element = TypeInfo::from_type(inner);
                TypeInfo {
                    name: format!("{}[]", element.name),
                    kind: TypeKind::Array,
                    fields: vec![],
                    parameters: vec![],
                    return_type: None,
                    element_type: Some(Box::new(element)),
                    type_args: vec![],
                    alias_target: None,
                }
            }

            Type::Function {
                params,
                return_type,
                ..
            } => {
                let param_infos: Vec<TypeInfo> = params.iter().map(TypeInfo::from_type).collect();

                let return_info = TypeInfo::from_type(return_type);

                TypeInfo {
                    name: "function".to_string(),
                    kind: TypeKind::Function,
                    fields: vec![],
                    parameters: param_infos,
                    return_type: Some(Box::new(return_info)),
                    element_type: None,
                    type_args: vec![],
                    alias_target: None,
                }
            }

            Type::JsonValue => TypeInfo {
                name: "json".to_string(),
                kind: TypeKind::JsonValue,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: None,
            },

            Type::Generic { name, type_args } => {
                let arg_infos: Vec<TypeInfo> = type_args.iter().map(TypeInfo::from_type).collect();

                let args_str = arg_infos
                    .iter()
                    .map(|t| t.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");

                TypeInfo {
                    name: format!("{}<{}>", name, args_str),
                    kind: TypeKind::Generic,
                    fields: vec![],
                    parameters: vec![],
                    return_type: None,
                    element_type: None,
                    type_args: arg_infos,
                    alias_target: None,
                }
            }

            Type::TypeParameter { name } => TypeInfo {
                name: name.clone(),
                kind: TypeKind::TypeParameter,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: None,
            },

            Type::Unknown => TypeInfo {
                name: "unknown".to_string(),
                kind: TypeKind::Unknown,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: None,
            },

            Type::Extern(extern_type) => TypeInfo {
                name: format!("extern:{}", extern_type.display_name()),
                kind: TypeKind::Extern,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: None,
            },
            Type::Structural { members } => TypeInfo {
                name: ty.display_name(),
                kind: TypeKind::Structural,
                fields: members
                    .iter()
                    .map(|member| FieldInfo {
                        name: member.name.clone(),
                        field_type: TypeInfo::from_type(&member.ty),
                    })
                    .collect(),
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: None,
            },
            Type::Union(members) => TypeInfo {
                name: "union".to_string(),
                kind: TypeKind::Union,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: members.iter().map(TypeInfo::from_type).collect(),
                alias_target: None,
            },
            Type::Intersection(members) => TypeInfo {
                name: "intersection".to_string(),
                kind: TypeKind::Intersection,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: members.iter().map(TypeInfo::from_type).collect(),
                alias_target: None,
            },

            Type::Alias { name, target, .. } => TypeInfo {
                name: name.clone(),
                kind: TypeKind::Alias,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
                alias_target: Some(Box::new(TypeInfo::from_type(target))),
            },
        }
    }

    /// Check if this is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(
            self.kind,
            TypeKind::Number | TypeKind::String | TypeKind::Bool | TypeKind::Null
        )
    }

    /// Check if this is a function type
    pub fn is_function(&self) -> bool {
        matches!(self.kind, TypeKind::Function)
    }

    /// Check if this is an array type
    pub fn is_array(&self) -> bool {
        matches!(self.kind, TypeKind::Array)
    }

    /// Check if this is a generic type
    pub fn is_generic(&self) -> bool {
        matches!(self.kind, TypeKind::Generic)
    }

    /// Get function signature as a string (for function types only)
    pub fn function_signature(&self) -> Option<String> {
        if !self.is_function() {
            return None;
        }

        let params = self
            .parameters
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>()
            .join(", ");

        let return_name = self
            .return_type
            .as_ref()
            .map(|r| r.name.clone())
            .unwrap_or_else(|| "void".to_string());

        Some(format!("({}) -> {}", params, return_name))
    }

    /// Get a detailed description of this type
    pub fn describe(&self) -> String {
        match self.kind {
            TypeKind::Number => "primitive number type".to_string(),
            TypeKind::String => "primitive string type".to_string(),
            TypeKind::Bool => "primitive boolean type".to_string(),
            TypeKind::Null => "null type".to_string(),
            TypeKind::Void => "void type (no value)".to_string(),
            TypeKind::Never => "never type (no values)".to_string(),

            TypeKind::Array => {
                if let Some(elem) = &self.element_type {
                    format!("array of {}", elem.name)
                } else {
                    "array type".to_string()
                }
            }

            TypeKind::Function => {
                if let Some(sig) = self.function_signature() {
                    format!("function {}", sig)
                } else {
                    "function type".to_string()
                }
            }

            TypeKind::JsonValue => "dynamic JSON value type".to_string(),
            TypeKind::Structural => "structural type".to_string(),
            TypeKind::Union => "union type".to_string(),
            TypeKind::Intersection => "intersection type".to_string(),

            TypeKind::Generic => {
                format!(
                    "generic type {} with {} type argument(s)",
                    self.name,
                    self.type_args.len()
                )
            }

            TypeKind::Alias => {
                if let Some(target) = &self.alias_target {
                    format!("alias {} for {}", self.name, target.name)
                } else {
                    format!("alias {}", self.name)
                }
            }

            TypeKind::TypeParameter => {
                format!("type parameter {}", self.name)
            }

            TypeKind::Unknown => "unknown type (error recovery)".to_string(),
            TypeKind::Extern => format!("extern type {}", self.name),
            TypeKind::Option => "Option type".to_string(),
            TypeKind::Result => "Result type".to_string(),
        }
    }
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
