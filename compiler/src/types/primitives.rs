//! # Primitive Types
//!
//! This module defines the primitive types in Vela:
//! - `Number`: 64-bit integers and floats
//! - `String`: UTF-8 encoded strings
//! - `Bool`: Boolean values (true/false)
//! - `Void`: Absence of value (for functions with no return)

use std::fmt;

/// Primitive types in Vela
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    /// 64-bit numeric type (integer or float depending on context)
    Number,

    /// UTF-8 encoded string
    String,

    /// Boolean value (true or false)
    Bool,

    /// Absence of value (used for functions that don't return anything)
    Void,
}

impl PrimitiveType {
    /// Get the size in bytes of this primitive type
    pub fn size(&self) -> usize {
        match self {
            PrimitiveType::Number => 8,  // 64 bits
            PrimitiveType::String => 16, // pointer (8) + length (8)
            PrimitiveType::Bool => 1,    // 1 byte
            PrimitiveType::Void => 0,    // no size
        }
    }

    /// Check if this type can be used in arithmetic operations
    pub fn is_numeric(&self) -> bool {
        matches!(self, PrimitiveType::Number)
    }

    /// Check if this type can be compared for equality
    pub fn is_comparable(&self) -> bool {
        !matches!(self, PrimitiveType::Void)
    }

    /// Check if this type can be ordered (<, >, <=, >=)
    pub fn is_ordered(&self) -> bool {
        matches!(self, PrimitiveType::Number | PrimitiveType::String)
    }

    /// Get the default value for this type
    pub fn default_value(&self) -> Option<String> {
        match self {
            PrimitiveType::Number => Some("0".to_string()),
            PrimitiveType::String => Some("\"\"".to_string()),
            PrimitiveType::Bool => Some("false".to_string()),
            PrimitiveType::Void => None,
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PrimitiveType::Number => "number",
            PrimitiveType::String => "string",
            PrimitiveType::Bool => "bool",
            PrimitiveType::Void => "void",
        };
        write!(f, "{}", name)
    }
}

/// Type information for primitive operations
#[derive(Debug, Clone)]
pub struct PrimitiveOperation {
    pub name: &'static str,
    pub operand_types: Vec<PrimitiveType>,
    pub result_type: PrimitiveType,
}

impl PrimitiveOperation {
    /// Get all valid operations for a given operand type
    pub fn operations_for(operand: PrimitiveType) -> Vec<PrimitiveOperation> {
        match operand {
            PrimitiveType::Number => vec![
                PrimitiveOperation {
                    name: "+",
                    operand_types: vec![PrimitiveType::Number, PrimitiveType::Number],
                    result_type: PrimitiveType::Number,
                },
                PrimitiveOperation {
                    name: "-",
                    operand_types: vec![PrimitiveType::Number, PrimitiveType::Number],
                    result_type: PrimitiveType::Number,
                },
                PrimitiveOperation {
                    name: "*",
                    operand_types: vec![PrimitiveType::Number, PrimitiveType::Number],
                    result_type: PrimitiveType::Number,
                },
                PrimitiveOperation {
                    name: "/",
                    operand_types: vec![PrimitiveType::Number, PrimitiveType::Number],
                    result_type: PrimitiveType::Number,
                },
                PrimitiveOperation {
                    name: "%",
                    operand_types: vec![PrimitiveType::Number, PrimitiveType::Number],
                    result_type: PrimitiveType::Number,
                },
                PrimitiveOperation {
                    name: "==",
                    operand_types: vec![PrimitiveType::Number, PrimitiveType::Number],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: "!=",
                    operand_types: vec![PrimitiveType::Number, PrimitiveType::Number],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: "<",
                    operand_types: vec![PrimitiveType::Number, PrimitiveType::Number],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: "<=",
                    operand_types: vec![PrimitiveType::Number, PrimitiveType::Number],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: ">",
                    operand_types: vec![PrimitiveType::Number, PrimitiveType::Number],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: ">=",
                    operand_types: vec![PrimitiveType::Number, PrimitiveType::Number],
                    result_type: PrimitiveType::Bool,
                },
            ],
            PrimitiveType::String => vec![
                PrimitiveOperation {
                    name: "+",
                    operand_types: vec![PrimitiveType::String, PrimitiveType::String],
                    result_type: PrimitiveType::String,
                },
                PrimitiveOperation {
                    name: "==",
                    operand_types: vec![PrimitiveType::String, PrimitiveType::String],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: "!=",
                    operand_types: vec![PrimitiveType::String, PrimitiveType::String],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: "<",
                    operand_types: vec![PrimitiveType::String, PrimitiveType::String],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: "<=",
                    operand_types: vec![PrimitiveType::String, PrimitiveType::String],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: ">",
                    operand_types: vec![PrimitiveType::String, PrimitiveType::String],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: ">=",
                    operand_types: vec![PrimitiveType::String, PrimitiveType::String],
                    result_type: PrimitiveType::Bool,
                },
            ],
            PrimitiveType::Bool => vec![
                PrimitiveOperation {
                    name: "&&",
                    operand_types: vec![PrimitiveType::Bool, PrimitiveType::Bool],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: "||",
                    operand_types: vec![PrimitiveType::Bool, PrimitiveType::Bool],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: "!",
                    operand_types: vec![PrimitiveType::Bool],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: "==",
                    operand_types: vec![PrimitiveType::Bool, PrimitiveType::Bool],
                    result_type: PrimitiveType::Bool,
                },
                PrimitiveOperation {
                    name: "!=",
                    operand_types: vec![PrimitiveType::Bool, PrimitiveType::Bool],
                    result_type: PrimitiveType::Bool,
                },
            ],
            PrimitiveType::Void => vec![], // No operations on void
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_sizes() {
        assert_eq!(PrimitiveType::Number.size(), 8);
        assert_eq!(PrimitiveType::String.size(), 16);
        assert_eq!(PrimitiveType::Bool.size(), 1);
        assert_eq!(PrimitiveType::Void.size(), 0);
    }

    #[test]
    fn test_primitive_properties() {
        assert!(PrimitiveType::Number.is_numeric());
        assert!(!PrimitiveType::String.is_numeric());
        assert!(!PrimitiveType::Bool.is_numeric());
        assert!(!PrimitiveType::Void.is_numeric());

        assert!(PrimitiveType::Number.is_comparable());
        assert!(PrimitiveType::String.is_comparable());
        assert!(PrimitiveType::Bool.is_comparable());
        assert!(!PrimitiveType::Void.is_comparable());

        assert!(PrimitiveType::Number.is_ordered());
        assert!(PrimitiveType::String.is_ordered());
        assert!(!PrimitiveType::Bool.is_ordered());
        assert!(!PrimitiveType::Void.is_ordered());
    }

    #[test]
    fn test_default_values() {
        assert_eq!(PrimitiveType::Number.default_value(), Some("0".to_string()));
        assert_eq!(PrimitiveType::String.default_value(), Some("\"\"".to_string()));
        assert_eq!(PrimitiveType::Bool.default_value(), Some("false".to_string()));
        assert_eq!(PrimitiveType::Void.default_value(), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", PrimitiveType::Number), "number");
        assert_eq!(format!("{}", PrimitiveType::String), "string");
        assert_eq!(format!("{}", PrimitiveType::Bool), "bool");
        assert_eq!(format!("{}", PrimitiveType::Void), "void");
    }

    #[test]
    fn test_number_operations() {
        let ops = PrimitiveOperation::operations_for(PrimitiveType::Number);
        assert!(ops.len() > 0);

        // Check that + operation exists
        let add_op = ops.iter().find(|op| op.name == "+").unwrap();
        assert_eq!(add_op.operand_types, vec![PrimitiveType::Number, PrimitiveType::Number]);
        assert_eq!(add_op.result_type, PrimitiveType::Number);
    }

    #[test]
    fn test_bool_operations() {
        let ops = PrimitiveOperation::operations_for(PrimitiveType::Bool);
        assert!(ops.len() > 0);

        // Check that && operation exists
        let and_op = ops.iter().find(|op| op.name == "&&").unwrap();
        assert_eq!(and_op.operand_types, vec![PrimitiveType::Bool, PrimitiveType::Bool]);
        assert_eq!(and_op.result_type, PrimitiveType::Bool);
    }

    #[test]
    fn test_void_operations() {
        let ops = PrimitiveOperation::operations_for(PrimitiveType::Void);
        assert_eq!(ops.len(), 0);
    }
}