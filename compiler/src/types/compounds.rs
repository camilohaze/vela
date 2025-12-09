//! # Compound Types
//!
//! This module defines compound types in Vela:
//! - `Struct`: Product types with named fields
//! - `Enum`: Sum types with variants
//! - `Union`: Anonymous sum types

use std::collections::HashMap;
use std::fmt;
use crate::types::Type;
use crate::types::generics;
use crate::types::primitives;

/// Struct type definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructType {
    /// Name of the struct
    pub name: String,

    /// Fields with their types
    pub fields: HashMap<String, super::Type>,

    /// Methods (functions associated with the struct)
    pub methods: HashMap<String, super::functions::FunctionType>,
}

impl StructType {
    /// Create a new struct type
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    /// Add a field to the struct
    pub fn add_field(&mut self, name: String, ty: super::Type) {
        self.fields.insert(name, ty);
    }

    /// Add a method to the struct
    pub fn add_method(&mut self, name: String, method: super::functions::FunctionType) {
        self.methods.insert(name, method);
    }

    /// Get a field type by name
    pub fn get_field(&self, name: &str) -> Option<&super::Type> {
        self.fields.get(name)
    }

    /// Check if the struct has a field
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    /// Get all field names
    pub fn field_names(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }
}

impl fmt::Display for StructType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "struct {}", self.name)
    }
}

/// Enum variant definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumVariant {
    /// Unit variant (no data)
    Unit(String),

    /// Tuple variant (unnamed fields)
    Tuple(String, Vec<super::Type>),

    /// Struct variant (named fields)
    Struct(String, HashMap<String, super::Type>),
}

impl EnumVariant {
    /// Get the name of this variant
    pub fn name(&self) -> &str {
        match self {
            EnumVariant::Unit(name) => name,
            EnumVariant::Tuple(name, _) => name,
            EnumVariant::Struct(name, _) => name,
        }
    }

    /// Check if this variant has data
    pub fn has_data(&self) -> bool {
        !matches!(self, EnumVariant::Unit(_))
    }

    /// Get all free type variables in this variant
    pub fn free_vars(&self) -> std::collections::HashSet<generics::TypeVar> {
        match self {
            EnumVariant::Unit(_) => std::collections::HashSet::new(),
            EnumVariant::Tuple(_, types) => {
                types.iter().flat_map(|t| t.free_vars()).collect()
            }
            EnumVariant::Struct(_, fields) => {
                fields.values().flat_map(|t| t.free_vars()).collect()
            }
        }
    }

    /// Apply a substitution to this variant
    pub fn apply_substitution(&self, subst: &super::unification::Substitution) -> Self {
        match self {
            EnumVariant::Unit(name) => EnumVariant::Unit(name.clone()),
            EnumVariant::Tuple(name, types) => {
                let new_types = types.iter()
                    .map(|t| t.apply_substitution(subst))
                    .collect();
                EnumVariant::Tuple(name.clone(), new_types)
            }
            EnumVariant::Struct(name, fields) => {
                let new_fields = fields.iter()
                    .map(|(k, v)| (k.clone(), v.apply_substitution(subst)))
                    .collect();
                EnumVariant::Struct(name.clone(), new_fields)
            }
        }
    }
}

impl fmt::Display for EnumVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnumVariant::Unit(name) => write!(f, "{}", name),
            EnumVariant::Tuple(name, types) => {
                let types_str = types.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{}({})", name, types_str)
            }
            EnumVariant::Struct(name, fields) => {
                let fields_str = fields.iter()
                    .map(|(name, ty)| format!("{}: {}", name, ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{}{{{}}}", name, fields_str)
            }
        }
    }
}

/// Enum type definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumType {
    /// Name of the enum
    pub name: String,

    /// Variants of the enum
    pub variants: Vec<EnumVariant>,
}

impl EnumType {
    /// Create a new enum type
    pub fn new(name: String) -> Self {
        Self {
            name,
            variants: Vec::new(),
        }
    }

    /// Add a unit variant
    pub fn add_unit_variant(&mut self, name: String) {
        self.variants.push(EnumVariant::Unit(name));
    }

    /// Add a tuple variant
    pub fn add_tuple_variant(&mut self, name: String, types: Vec<super::Type>) {
        self.variants.push(EnumVariant::Tuple(name, types));
    }

    /// Add a struct variant
    pub fn add_struct_variant(&mut self, name: String, fields: HashMap<String, super::Type>) {
        self.variants.push(EnumVariant::Struct(name, fields));
    }

    /// Get a variant by name
    pub fn get_variant(&self, name: &str) -> Option<&EnumVariant> {
        self.variants.iter().find(|v| v.name() == name)
    }

    /// Check if the enum has a variant
    pub fn has_variant(&self, name: &str) -> bool {
        self.variants.iter().any(|v| v.name() == name)
    }

    /// Get all variant names
    pub fn variant_names(&self) -> Vec<String> {
        self.variants.iter().map(|v| v.name().to_string()).collect()
    }
}

impl fmt::Display for EnumType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "enum {}", self.name)
    }
}

/// Union type (anonymous sum type)
pub type UnionType = Vec<super::Type>;

/// Helper functions for working with compound types
pub mod helpers {
    use super::*;

    /// Create a common struct type (Person)
    pub fn create_person_struct() -> StructType {
        let mut person = StructType::new("Person".to_string());
        person.add_field("name".to_string(), super::Type::Primitive(primitives::PrimitiveType::String));
        person.add_field("age".to_string(), super::Type::Primitive(primitives::PrimitiveType::Number));
        person
    }

    /// Create a common enum type (Result)
    pub fn create_result_enum() -> EnumType {
        let mut result = EnumType::new("Result".to_string());
        result.add_struct_variant("Ok".to_string(),
            [("value".to_string(), Type::Variable(generics::TypeVar { id: 1, name: Some("T".to_string()) }))]
                .into_iter().collect()
        );
        result.add_struct_variant("Err".to_string(),
            [("error".to_string(), Type::Variable(generics::TypeVar { id: 2, name: Some("E".to_string()) }))]
                .into_iter().collect()
        );
        result
    }

    /// Create an Option enum type
    pub fn create_option_enum() -> EnumType {
        let mut option = EnumType::new("Option".to_string());
        option.add_struct_variant("Some".to_string(),
            [("value".to_string(), Type::Variable(generics::TypeVar { id: 1, name: Some("T".to_string()) }))]
                .into_iter().collect()
        );
        option.add_unit_variant("None".to_string());
        option
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Type, primitives::PrimitiveType, generics::TypeVar};

    #[test]
    fn test_struct_creation() {
        let mut person = StructType::new("Person".to_string());
        person.add_field("name".to_string(), Type::Primitive(PrimitiveType::String));
        person.add_field("age".to_string(), Type::Primitive(PrimitiveType::Number));

        assert_eq!(person.name, "Person");
        assert!(person.has_field("name"));
        assert!(person.has_field("age"));
        assert!(!person.has_field("email"));

        let name_field = person.get_field("name").unwrap();
        assert_eq!(*name_field, Type::Primitive(PrimitiveType::String));

        let fields = person.field_names();
        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"age".to_string()));
    }

    #[test]
    fn test_enum_unit_variant() {
        let mut color = EnumType::new("Color".to_string());
        color.add_unit_variant("Red".to_string());
        color.add_unit_variant("Green".to_string());
        color.add_unit_variant("Blue".to_string());

        assert_eq!(color.name, "Color");
        assert!(color.has_variant("Red"));
        assert!(color.has_variant("Green"));
        assert!(color.has_variant("Blue"));
        assert!(!color.has_variant("Yellow"));

        let red_variant = color.get_variant("Red").unwrap();
        assert_eq!(red_variant.name(), "Red");
        assert!(!red_variant.has_data());

        let variants = color.variant_names();
        assert_eq!(variants.len(), 3);
        assert!(variants.contains(&"Red".to_string()));
    }

    #[test]
    fn test_enum_tuple_variant() {
        let mut point = EnumType::new("Point".to_string());
        point.add_tuple_variant("Coord".to_string(),
            vec![Type::Primitive(PrimitiveType::Number), Type::Primitive(PrimitiveType::Number)]
        );

        let coord_variant = point.get_variant("Coord").unwrap();
        assert_eq!(coord_variant.name(), "Coord");
        assert!(coord_variant.has_data());

        match coord_variant {
            EnumVariant::Tuple(name, types) => {
                assert_eq!(name, "Coord");
                assert_eq!(types.len(), 2);
                assert_eq!(types[0], Type::Primitive(PrimitiveType::Number));
                assert_eq!(types[1], Type::Primitive(PrimitiveType::Number));
            }
            _ => panic!("Expected tuple variant"),
        }
    }

    #[test]
    fn test_enum_struct_variant() {
        let mut shape = EnumType::new("Shape".to_string());
        let mut circle_fields = HashMap::new();
        circle_fields.insert("radius".to_string(), Type::Primitive(PrimitiveType::Number));
        shape.add_struct_variant("Circle".to_string(), circle_fields);

        let circle_variant = shape.get_variant("Circle").unwrap();
        assert_eq!(circle_variant.name(), "Circle");
        assert!(circle_variant.has_data());

        match circle_variant {
            EnumVariant::Struct(name, fields) => {
                assert_eq!(name, "Circle");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields["radius"], Type::Primitive(PrimitiveType::Number));
            }
            _ => panic!("Expected struct variant"),
        }
    }

    #[test]
    fn test_display_formatting() {
        let mut person = StructType::new("Person".to_string());
        person.add_field("name".to_string(), Type::Primitive(PrimitiveType::String));
        assert_eq!(format!("{}", person), "struct Person");

        let mut color = EnumType::new("Color".to_string());
        color.add_unit_variant("Red".to_string());
        assert_eq!(format!("{}", color), "enum Color");

        let unit_variant = EnumVariant::Unit("Red".to_string());
        assert_eq!(format!("{}", unit_variant), "Red");

        let tuple_variant = EnumVariant::Tuple("Point".to_string(),
            vec![Type::Primitive(PrimitiveType::Number), Type::Primitive(PrimitiveType::Number)]
        );
        assert_eq!(format!("{}", tuple_variant), "Point(number, number)");
    }

    #[test]
    fn test_helper_functions() {
        let person = helpers::create_person_struct();
        assert_eq!(person.name, "Person");
        assert!(person.has_field("name"));
        assert!(person.has_field("age"));

        let result = helpers::create_result_enum();
        assert_eq!(result.name, "Result");
        assert!(result.has_variant("Ok"));
        assert!(result.has_variant("Err"));

        let option = helpers::create_option_enum();
        assert_eq!(option.name, "Option");
        assert!(option.has_variant("Some"));
        assert!(option.has_variant("None"));
    }
}