/**
 * Tests unitarios para el sistema de tipos
 *
 * Jira: VELA-1106
 * Historia: VELA-1106
 */

use vela_compiler::types::*;
use vela_compiler::types::primitives::PrimitiveType;
use vela_compiler::types::special::{AsyncIteratorType, helpers};

#[cfg(test)]
mod tests {
    use super::*;

    mod async_iterator_type_tests {
        use super::*;

        #[test]
        fn test_async_iterator_type_creation() {
            let element_type = Type::Primitive(PrimitiveType::Number);
            let async_iter = AsyncIteratorType::new(element_type.clone());

            assert_eq!(*async_iter.element_type, element_type);
        }

        #[test]
        fn test_async_iterator_type_display() {
            let element_type = Type::Primitive(PrimitiveType::String);
            let async_iter = AsyncIteratorType::new(element_type);

            assert_eq!(format!("{}", async_iter), "AsyncIterator<string>");
        }

        #[test]
        fn test_async_iterator_type_with_complex_element() {
            // Crear un tipo complejo: List<Number>
            let number_type = Type::Primitive(PrimitiveType::Number);
            let list_type = helpers::list_type(number_type);
            let async_iter = AsyncIteratorType::new(list_type);

            assert_eq!(format!("{}", async_iter), "AsyncIterator<List<number>>");
        }

        #[test]
        fn test_async_iterator_helper_function() {
            let element_type = Type::Primitive(PrimitiveType::Bool);
            let async_iter_type = helpers::async_iterator_type(element_type);

            match async_iter_type {
                Type::AsyncIterator(async_iter) => {
                    assert_eq!(*async_iter.element_type, Type::Primitive(PrimitiveType::Bool));
                }
                _ => panic!("Expected AsyncIterator type"),
            }
        }

        #[test]
        fn test_async_iterator_type_equality() {
            let type1 = Type::Primitive(PrimitiveType::Number);
            let type2 = Type::Primitive(PrimitiveType::Number);

            let async_iter1 = AsyncIteratorType::new(type1);
            let async_iter2 = AsyncIteratorType::new(type2);

            assert_eq!(async_iter1, async_iter2);
        }

        #[test]
        fn test_async_iterator_type_inequality() {
            let number_type = Type::Primitive(PrimitiveType::Number);
            let string_type = Type::Primitive(PrimitiveType::String);

            let async_iter1 = AsyncIteratorType::new(number_type);
            let async_iter2 = AsyncIteratorType::new(string_type);

            assert_ne!(async_iter1, async_iter2);
        }

        #[test]
        fn test_type_enum_async_iterator_variant() {
            let element_type = Type::Primitive(PrimitiveType::String);
            let async_iter_type = Type::AsyncIterator(AsyncIteratorType::new(element_type));

            match async_iter_type {
                Type::AsyncIterator(async_iter) => {
                    assert_eq!(*async_iter.element_type, Type::Primitive(PrimitiveType::String));
                }
                _ => panic!("Expected AsyncIterator variant"),
            }
        }

        #[test]
        fn test_nested_async_iterator() {
            // AsyncIterator<AsyncIterator<Number>>
            let number_type = Type::Primitive(PrimitiveType::Number);
            let inner_async_iter = helpers::async_iterator_type(number_type);
            let outer_async_iter = helpers::async_iterator_type(inner_async_iter);

            match outer_async_iter {
                Type::AsyncIterator(outer) => {
                    match *outer.element_type {
                        Type::AsyncIterator(inner) => {
                            assert_eq!(*inner.element_type, Type::Primitive(PrimitiveType::Number));
                        }
                        _ => panic!("Expected nested AsyncIterator"),
                    }
                }
                _ => panic!("Expected AsyncIterator"),
            }
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_async_iterator_in_function_return_type() {
            // Simular: fn getDataStream() -> AsyncIterator<Data>
            let data_type = Type::Simple("Data".to_string());
            let return_type = helpers::async_iterator_type(data_type);

            match return_type {
                Type::AsyncIterator(async_iter) => {
                    match *async_iter.element_type {
                        Type::Simple(name) => assert_eq!(name, "Data"),
                        _ => panic!("Expected Data type"),
                    }
                }
                _ => panic!("Expected AsyncIterator return type"),
            }
        }

        #[test]
        fn test_async_iterator_with_result_element() {
            // AsyncIterator<Result<Data, Error>>
            let data_type = Type::Simple("Data".to_string());
            let error_type = Type::Simple("Error".to_string());
            let result_type = helpers::result_type(data_type, error_type);
            let async_iter_type = helpers::async_iterator_type(result_type);

            match async_iter_type {
                Type::AsyncIterator(async_iter) => {
                    match *async_iter.element_type {
                        Type::Constructor(ctor) => {
                            assert_eq!(ctor.name, "Result");
                            assert_eq!(ctor.args.len(), 2);
                        }
                        _ => panic!("Expected Result constructor"),
                    }
                }
                _ => panic!("Expected AsyncIterator"),
            }
        }
    }
}