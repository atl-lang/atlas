//! Type system representation

use serde::{Deserialize, Serialize};

/// Type representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Integer type
    Int,
    /// Float type
    Float,
    /// String type
    String,
    /// Boolean type
    Bool,
    /// Null type
    Null,
    /// Array type
    Array(Box<Type>),
    /// Function type
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    /// Unknown type (for error recovery)
    Unknown,
}

impl Type {
    /// Check if this type is compatible with another type
    pub fn is_assignable_to(&self, _other: &Type) -> bool {
        // Placeholder implementation
        true
    }

    /// Get a human-readable name for this type
    pub fn display_name(&self) -> String {
        match self {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::String => "string".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Null => "null".to_string(),
            Type::Array(inner) => format!("{}[]", inner.display_name()),
            Type::Function { .. } => "function".to_string(),
            Type::Unknown => "?".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        assert_eq!(Type::Int.display_name(), "int");
        assert_eq!(Type::String.display_name(), "string");
    }

    #[test]
    fn test_array_type() {
        let arr_type = Type::Array(Box::new(Type::Int));
        assert_eq!(arr_type.display_name(), "int[]");
    }
}
