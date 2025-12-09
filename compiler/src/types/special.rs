//! # Special Types
//!
//! This module implements special types that are built into Vela:
//! - `Option<T>`: Optional values (Some/None)
//! - `Result<T,E>`: Success/Error values (Ok/Err)
//! - `List<T>`: Dynamic arrays
//! - `Dict<K,V>`: Key-value mappings
//! - `Set<T>`: Unique value collections
//! - `Promise<T>`: Asynchronous computations
//! - `Stream<T>`: Asynchronous sequences

use std::fmt;

/// Option type variants
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptionVariant {
    /// Some value
    Some(Box<super::Type>),
    /// No value
    None,
}

impl fmt::Display for OptionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionVariant::Some(ty) => write!(f, "Some<{}>", ty),
            OptionVariant::None => write!(f, "None"),
        }
    }
}

/// Result type variants
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResultVariant {
    /// Success value
    Ok(Box<super::Type>),
    /// Error value
    Err(Box<super::Type>),
}

impl fmt::Display for ResultVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResultVariant::Ok(ty) => write!(f, "Ok<{}>", ty),
            ResultVariant::Err(ty) => write!(f, "Err<{}>", ty),
        }
    }
}

/// List type with element type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListType {
    pub element_type: Box<super::Type>,
}

impl ListType {
    pub fn new(element_type: super::Type) -> Self {
        Self {
            element_type: Box::new(element_type),
        }
    }
}

impl fmt::Display for ListType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "List<{}>", self.element_type)
    }
}

/// Dictionary type with key and value types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DictType {
    pub key_type: Box<super::Type>,
    pub value_type: Box<super::Type>,
}

impl DictType {
    pub fn new(key_type: super::Type, value_type: super::Type) -> Self {
        Self {
            key_type: Box::new(key_type),
            value_type: Box::new(value_type),
        }
    }
}

impl fmt::Display for DictType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dict<{}, {}>", self.key_type, self.value_type)
    }
}

/// Set type with element type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetType {
    pub element_type: Box<super::Type>,
}

impl SetType {
    pub fn new(element_type: super::Type) -> Self {
        Self {
            element_type: Box::new(element_type),
        }
    }
}

impl fmt::Display for SetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Set<{}>", self.element_type)
    }
}

/// Promise type for asynchronous computations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PromiseType {
    pub result_type: Box<super::Type>,
}

impl PromiseType {
    pub fn new(result_type: super::Type) -> Self {
        Self {
            result_type: Box::new(result_type),
        }
    }
}

impl fmt::Display for PromiseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Promise<{}>", self.result_type)
    }
}

/// Stream type for asynchronous sequences
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamType {
    pub element_type: Box<super::Type>,
}

impl StreamType {
    pub fn new(element_type: super::Type) -> Self {
        Self {
            element_type: Box::new(element_type),
        }
    }
}

impl fmt::Display for StreamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stream<{}>", self.element_type)
    }
}

/// Tuple type with multiple element types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TupleType {
    pub elements: Vec<super::Type>,
}

impl TupleType {
    pub fn new(elements: Vec<super::Type>) -> Self {
        Self { elements }
    }

    pub fn unit() -> Self {
        Self::new(vec![])
    }

    pub fn pair(first: super::Type, second: super::Type) -> Self {
        Self::new(vec![first, second])
    }

    pub fn triple(first: super::Type, second: super::Type, third: super::Type) -> Self {
        Self::new(vec![first, second, third])
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_unit(&self) -> bool {
        self.elements.is_empty()
    }
}

impl fmt::Display for TupleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.elements.is_empty() {
            write!(f, "()")
        } else {
            let elements_str = self.elements.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            write!(f, "({})", elements_str)
        }
    }
}

/// Union type (sum type)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionType {
    pub variants: Vec<super::Type>,
}

impl UnionType {
    pub fn new(variants: Vec<super::Type>) -> Self {
        Self { variants }
    }

    pub fn binary(left: super::Type, right: super::Type) -> Self {
        Self::new(vec![left, right])
    }

    pub fn len(&self) -> usize {
        self.variants.len()
    }
}

impl fmt::Display for UnionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variants_str = self.variants.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        write!(f, "{}", variants_str)
    }
}

/// Intersection type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntersectionType {
    pub types: Vec<super::Type>,
}

impl IntersectionType {
    pub fn new(types: Vec<super::Type>) -> Self {
        Self { types }
    }

    pub fn binary(left: super::Type, right: super::Type) -> Self {
        Self::new(vec![left, right])
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }
}

impl fmt::Display for IntersectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let types_str = self.types.iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(" & ");
        write!(f, "{}", types_str)
    }
}

/// Range type for inclusive/exclusive ranges
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RangeType {
    pub element_type: Box<super::Type>,
    pub inclusive: bool,
}

impl RangeType {
    pub fn inclusive(element_type: super::Type) -> Self {
        Self {
            element_type: Box::new(element_type),
            inclusive: true,
        }
    }

    pub fn exclusive(element_type: super::Type) -> Self {
        Self {
            element_type: Box::new(element_type),
            inclusive: false,
        }
    }
}

impl fmt::Display for RangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let range_op = if self.inclusive { "..=" } else { ".." };
        write!(f, "{} {} {}", self.element_type, range_op, self.element_type)
    }
}

/// Iterator type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IteratorType {
    pub element_type: Box<super::Type>,
}

impl IteratorType {
    pub fn new(element_type: super::Type) -> Self {
        Self {
            element_type: Box::new(element_type),
        }
    }
}

impl fmt::Display for IteratorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Iterator<{}>", self.element_type)
    }
}

/// Helper functions for creating common special types
pub mod helpers {
    use super::*;

    /// Create Option<T> type
    pub fn option_type(inner: super::Type) -> super::Type {
        super::Type::Constructor(super::generics::TypeConstructor::unary(
            "Option".to_string(),
            inner
        ))
    }

    /// Create Result<T,E> type
    pub fn result_type(ok: super::Type, err: super::Type) -> super::Type {
        super::Type::Constructor(super::generics::TypeConstructor::binary(
            "Result".to_string(),
            ok,
            err
        ))
    }

    /// Create List<T> type
    pub fn list_type(element: super::Type) -> super::Type {
        super::Type::Constructor(super::generics::TypeConstructor::unary(
            "List".to_string(),
            element
        ))
    }

    /// Create Dict<K,V> type
    pub fn dict_type(key: super::Type, value: super::Type) -> super::Type {
        super::Type::Constructor(super::generics::TypeConstructor::binary(
            "Dict".to_string(),
            key,
            value
        ))
    }

    /// Create Set<T> type
    pub fn set_type(element: super::Type) -> super::Type {
        super::Type::Constructor(super::generics::TypeConstructor::unary(
            "Set".to_string(),
            element
        ))
    }

    /// Create Promise<T> type
    pub fn promise_type(result: super::Type) -> super::Type {
        super::Type::Constructor(super::generics::TypeConstructor::unary(
            "Promise".to_string(),
            result
        ))
    }

    /// Create Stream<T> type
    pub fn stream_type(element: super::Type) -> super::Type {
        super::Type::Constructor(super::generics::TypeConstructor::unary(
            "Stream".to_string(),
            element
        ))
    }

    /// Create Iterator<T> type
    pub fn iterator_type(element: super::Type) -> super::Type {
        super::Type::Constructor(super::generics::TypeConstructor::unary(
            "Iterator".to_string(),
            element
        ))
    }

    /// Create unit tuple type ()
    pub fn unit_type() -> super::Type {
        super::Type::Tuple(TupleType::unit())
    }

    /// Create pair tuple type (A, B)
    pub fn pair_type(first: super::Type, second: super::Type) -> super::Type {
        super::Type::Tuple(TupleType::pair(first, second))
    }

    /// Create triple tuple type (A, B, C)
    pub fn triple_type(first: super::Type, second: super::Type, third: super::Type) -> super::Type {
        super::Type::Tuple(TupleType::triple(first, second, third))
    }

    /// Create union type A | B
    pub fn union_type(left: super::Type, right: super::Type) -> super::Type {
        super::Type::Union(UnionType::binary(left, right))
    }

    /// Create intersection type A & B
    pub fn intersection_type(left: super::Type, right: super::Type) -> super::Type {
        super::Type::Intersection(IntersectionType::binary(left, right))
    }

    /// Create inclusive range type
    pub fn range_inclusive_type(element: super::Type) -> super::Type {
        super::Type::Range(RangeType::inclusive(element))
    }

    /// Create exclusive range type
    pub fn range_exclusive_type(element: super::Type) -> super::Type {
        super::Type::Range(RangeType::exclusive(element))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Type, primitives::PrimitiveType};

    #[test]
    fn test_list_type() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let list = ListType::new(int_type);

        assert_eq!(format!("{}", list), "List<number>");
    }

    #[test]
    fn test_dict_type() {
        let key_type = Type::Primitive(PrimitiveType::String);
        let value_type = Type::Primitive(PrimitiveType::Number);
        let dict = DictType::new(key_type, value_type);

        assert_eq!(format!("{}", dict), "Dict<string, number>");
    }

    #[test]
    fn test_tuple_type() {
        // Unit tuple
        let unit = TupleType::unit();
        assert_eq!(unit.len(), 0);
        assert!(unit.is_unit());
        assert_eq!(format!("{}", unit), "()");

        // Pair tuple
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);
        let pair = TupleType::pair(int_type.clone(), string_type.clone());
        assert_eq!(pair.len(), 2);
        assert_eq!(format!("{}", pair), "(number, string)");

        // Triple tuple
        let bool_type = Type::Primitive(PrimitiveType::Bool);
        let triple = TupleType::triple(int_type, string_type, bool_type);
        assert_eq!(triple.len(), 3);
        assert_eq!(format!("{}", triple), "(number, string, bool)");
    }

    #[test]
    fn test_union_type() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);

        let union = UnionType::binary(int_type, string_type);
        assert_eq!(union.len(), 2);
        assert_eq!(format!("{}", union), "number | string");
    }

    #[test]
    fn test_intersection_type() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);

        let intersection = IntersectionType::binary(int_type, string_type);
        assert_eq!(intersection.len(), 2);
        assert_eq!(format!("{}", intersection), "number & string");
    }

    #[test]
    fn test_range_type() {
        let int_type = Type::Primitive(PrimitiveType::Number);

        let inclusive = RangeType::inclusive(int_type.clone());
        assert!(inclusive.inclusive);
        assert_eq!(format!("{}", inclusive), "number ..= number");

        let exclusive = RangeType::exclusive(int_type);
        assert!(!exclusive.inclusive);
        assert_eq!(format!("{}", exclusive), "number .. number");
    }

    #[test]
    fn test_promise_type() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let promise = PromiseType::new(int_type);

        assert_eq!(format!("{}", promise), "Promise<number>");
    }

    #[test]
    fn test_stream_type() {
        let string_type = Type::Primitive(PrimitiveType::String);
        let stream = StreamType::new(string_type);

        assert_eq!(format!("{}", stream), "Stream<string>");
    }

    #[test]
    fn test_iterator_type() {
        let bool_type = Type::Primitive(PrimitiveType::Bool);
        let iter = IteratorType::new(bool_type);

        assert_eq!(format!("{}", iter), "Iterator<bool>");
    }

    #[test]
    fn test_helper_functions() {
        let int_type = Type::Primitive(PrimitiveType::Number);
        let string_type = Type::Primitive(PrimitiveType::String);
        let bool_type = Type::Primitive(PrimitiveType::Bool);

        // Test option
        let option = helpers::option_type(int_type.clone());
        match option {
            Type::Constructor(tc) => {
                assert_eq!(tc.name, "Option");
                assert_eq!(tc.params.len(), 1);
            }
            _ => panic!("Expected constructor type"),
        }

        // Test result
        let result = helpers::result_type(int_type.clone(), string_type.clone());
        match result {
            Type::Constructor(tc) => {
                assert_eq!(tc.name, "Result");
                assert_eq!(tc.params.len(), 2);
            }
            _ => panic!("Expected constructor type"),
        }

        // Test list
        let list = helpers::list_type(int_type.clone());
        match list {
            Type::Constructor(tc) => {
                assert_eq!(tc.name, "List");
                assert_eq!(tc.params.len(), 1);
            }
            _ => panic!("Expected constructor type"),
        }

        // Test dict
        let dict = helpers::dict_type(string_type.clone(), int_type.clone());
        match dict {
            Type::Constructor(tc) => {
                assert_eq!(tc.name, "Dict");
                assert_eq!(tc.params.len(), 2);
            }
            _ => panic!("Expected constructor type"),
        }

        // Test tuple helpers
        let unit = helpers::unit_type();
        assert_eq!(format!("{}", unit), "()");

        let pair = helpers::pair_type(int_type.clone(), string_type.clone());
        assert_eq!(format!("{}", pair), "(number, string)");

        let triple = helpers::triple_type(int_type, string_type, bool_type);
        assert_eq!(format!("{}", triple), "(number, string, bool)");
    }
}