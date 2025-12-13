//! Tests para Gateway Decorators
//!
//! Tests unitarios para el procesador de decoradores @gateway

use crate::ast::*;
use crate::gateway_decorators::{GatewayDecoratorProcessor, GatewayEndpointInfo};

#[test]
fn test_basic_gateway_decorator() {
    let mut processor = GatewayDecoratorProcessor::new();

    // Crear un decorador @gateway básico
    let decorator_args = StructLiteral::new(
        Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 30 } },
        "".to_string(),
        vec![
            StructLiteralField::new(
                "method".to_string(),
                Expression::Literal(Literal::new(
                    Range { start: Position { line: 0, column: 10 }, end: Position { line: 0, column: 15 } },
                    serde_json::Value::String("GET".to_string()),
                    "string".to_string()
                )),
                Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 15 } }
            ),
            StructLiteralField::new(
                "path".to_string(),
                Expression::Literal(Literal::new(
                    Range { start: Position { line: 0, column: 20 }, end: Position { line: 0, column: 30 } },
                    serde_json::Value::String("/api/users".to_string()),
                    "string".to_string()
                )),
                Range { start: Position { line: 0, column: 16 }, end: Position { line: 0, column: 30 } }
            ),
        ]
    );

    let decorator = Decorator {
        name: "gateway".to_string(),
        arguments: vec![Expression::StructLiteral(decorator_args)],
        range: Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 30 } },
    };

    let method = FunctionDeclaration {
        node: ASTNode::new(Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 100 } }),
        is_public: true,
        name: "getUsers".to_string(),
        decorators: vec![decorator],
        parameters: vec![],
        return_type: Some(TypeAnnotation::Named(NamedType::new(
            Range { start: Position { line: 0, column: 60 }, end: Position { line: 0, column: 66 } },
            "Result".to_string()
        ))),
        body: BlockStatement::new(
            Range { start: Position { line: 0, column: 68 }, end: Position { line: 0, column: 70 } },
            vec![]
        ),
        is_async: true,
        is_generator: false,
        generic_params: vec![],
    };

    // Procesar el método
    processor.process_method_decorators("UserController", &method).unwrap();

    // Verificar que se registró el endpoint
    let key = "UserController::getUsers";
    assert!(processor.get_endpoints().contains_key(key));

    let endpoint = processor.get_endpoints().get(key).unwrap();
    assert_eq!(endpoint.http_method, "GET");
    assert_eq!(endpoint.path, "/api/users");
    assert_eq!(endpoint.auth_required, false); // default
    assert_eq!(endpoint.cors_enabled, true); // default
}

#[test]
fn test_gateway_decorator_with_auth_and_middlewares() {
    let mut processor = GatewayDecoratorProcessor::new();

    // Crear decorador con autenticación y middlewares
    let middleware_array = ArrayLiteral::new(
        Range { start: Position { line: 0, column: 50 }, end: Position { line: 0, column: 75 } },
        vec![
            Expression::Literal(Literal::new(
                Range { start: Position { line: 0, column: 52 }, end: Position { line: 0, column: 60 } },
                serde_json::Value::String("logging".to_string()),
                "string".to_string()
            )),
            Expression::Literal(Literal::new(
                Range { start: Position { line: 0, column: 62 }, end: Position { line: 0, column: 73 } },
                serde_json::Value::String("validation".to_string()),
                "string".to_string()
            )),
        ]
    );

    let decorator_args = StructLiteral::new(
        Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 90 } },
        "".to_string(),
        vec![
            StructLiteralField::new(
                "method".to_string(),
                Expression::Literal(Literal::new(
                    Range { start: Position { line: 0, column: 10 }, end: Position { line: 0, column: 16 } },
                    serde_json::Value::String("POST".to_string()),
                    "string".to_string()
                )),
                Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 16 } }
            ),
            StructLiteralField::new(
                "path".to_string(),
                Expression::Literal(Literal::new(
                    Range { start: Position { line: 0, column: 20 }, end: Position { line: 0, column: 32 } },
                    serde_json::Value::String("/api/users".to_string()),
                    "string".to_string()
                )),
                Range { start: Position { line: 0, column: 17 }, end: Position { line: 0, column: 32 } }
            ),
            StructLiteralField::new(
                "auth".to_string(),
                Expression::Literal(Literal::new(
                    Range { start: Position { line: 0, column: 40 }, end: Position { line: 0, column: 44 } },
                    serde_json::Value::Bool(true),
                    "bool".to_string()
                )),
                Range { start: Position { line: 0, column: 33 }, end: Position { line: 0, column: 44 } }
            ),
            StructLiteralField::new(
                "middlewares".to_string(),
                Expression::ArrayLiteral(middleware_array),
                Range { start: Position { line: 0, column: 45 }, end: Position { line: 0, column: 75 } }
            ),
            StructLiteralField::new(
                "rateLimit".to_string(),
                Expression::Literal(Literal::new(
                    Range { start: Position { line: 0, column: 80 }, end: Position { line: 0, column: 90 } },
                    serde_json::Value::String("100/min".to_string()),
                    "string".to_string()
                )),
                Range { start: Position { line: 0, column: 76 }, end: Position { line: 0, column: 90 } }
            ),
        ]
    );

    let decorator = Decorator {
        name: "gateway".to_string(),
        arguments: vec![Expression::StructLiteral(decorator_args)],
        range: Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 90 } },
    };

    let method = FunctionDeclaration {
        node: ASTNode::new(Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 150 } }),
        is_public: true,
        name: "createUser".to_string(),
        decorators: vec![decorator],
        parameters: vec![Parameter::from_name(
                "userData".to_string(),
                Some(TypeAnnotation::Named(NamedType::new(
                    Range { start: Position { line: 0, column: 110 }, end: Position { line: 0, column: 117 } },
                    "UserDTO".to_string()
                ))),
                None,
                Range { start: Position { line: 0, column: 100 }, end: Position { line: 0, column: 120 } },
            )],
        return_type: Some(TypeAnnotation::Named(NamedType::new(
            Range { start: Position { line: 0, column: 125 }, end: Position { line: 0, column: 131 } },
            "Result".to_string()
        ))),
        body: BlockStatement::new(
            Range { start: Position { line: 0, column: 133 }, end: Position { line: 0, column: 135 } },
            vec![]
        ),
        is_async: true,
        is_generator: false,
        generic_params: vec![],
    };

    // Procesar el método
    processor.process_method_decorators("UserController", &method).unwrap();

    // Verificar que se registró el endpoint
    let key = "UserController::createUser";
    assert!(processor.get_endpoints().contains_key(key));

    let endpoint = processor.get_endpoints().get(key).unwrap();
    assert_eq!(endpoint.http_method, "POST");
    assert_eq!(endpoint.path, "/api/users");
    assert_eq!(endpoint.auth_required, true);
    assert_eq!(endpoint.middlewares, vec!["logging", "validation"]);
    assert_eq!(endpoint.rate_limit, Some("100/min".to_string()));
    assert_eq!(endpoint.cors_enabled, true);
}

#[test]
fn test_multiple_endpoints() {
    let mut processor = GatewayDecoratorProcessor::new();

    // Endpoint 1: GET /api/users
    let decorator1_args = StructLiteral::new(
        Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 25 } },
        "".to_string(),
        vec![
            StructLiteralField::new(
                "method".to_string(),
                Expression::Literal(Literal::new(
                    Range { start: Position { line: 0, column: 10 }, end: Position { line: 0, column: 15 } },
                    serde_json::Value::String("GET".to_string()),
                    "string".to_string()
                )),
                Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 15 } }
            ),
            StructLiteralField::new(
                "path".to_string(),
                Expression::Literal(Literal::new(
                    Range { start: Position { line: 0, column: 20 }, end: Position { line: 0, column: 25 } },
                    serde_json::Value::String("/api/users".to_string()),
                    "string".to_string()
                )),
                Range { start: Position { line: 0, column: 16 }, end: Position { line: 0, column: 25 } }
            ),
        ]
    );

    let decorator1 = Decorator {
        name: "gateway".to_string(),
        arguments: vec![Expression::StructLiteral(decorator1_args)],
        range: Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 25 } },
    };

    let method1 = FunctionDeclaration {
        node: ASTNode::new(Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 80 } }),
        is_public: true,
        name: "getUsers".to_string(),
        decorators: vec![decorator1],
        parameters: vec![],
        return_type: Some(TypeAnnotation::Named(NamedType::new(
            Range { start: Position { line: 0, column: 50 }, end: Position { line: 0, column: 56 } },
            "Result".to_string()
        ))),
        body: BlockStatement::new(
            Range { start: Position { line: 0, column: 58 }, end: Position { line: 0, column: 60 } },
            vec![]
        ),
        is_async: true,
        is_generator: false,
        generic_params: vec![],
    };

    // Endpoint 2: POST /api/users
    let decorator2_args = StructLiteral::new(
        Range { start: Position { line: 1, column: 0 }, end: Position { line: 1, column: 30 } },
        "".to_string(),
        vec![
            StructLiteralField::new(
                "method".to_string(),
                Expression::Literal(Literal::new(
                    Range { start: Position { line: 1, column: 10 }, end: Position { line: 1, column: 16 } },
                    serde_json::Value::String("POST".to_string()),
                    "string".to_string()
                )),
                Range { start: Position { line: 1, column: 0 }, end: Position { line: 1, column: 16 } }
            ),
            StructLiteralField::new(
                "path".to_string(),
                Expression::Literal(Literal::new(
                    Range { start: Position { line: 1, column: 20 }, end: Position { line: 1, column: 30 } },
                    serde_json::Value::String("/api/users".to_string()),
                    "string".to_string()
                )),
                Range { start: Position { line: 1, column: 17 }, end: Position { line: 1, column: 30 } }
            ),
        ]
    );

    let decorator2 = Decorator {
        name: "gateway".to_string(),
        arguments: vec![Expression::StructLiteral(decorator2_args)],
        range: Range { start: Position { line: 1, column: 0 }, end: Position { line: 1, column: 30 } },
    };

    let method2 = FunctionDeclaration {
        node: ASTNode::new(Range { start: Position { line: 1, column: 0 }, end: Position { line: 1, column: 85 } }),
        is_public: true,
        name: "createUser".to_string(),
        decorators: vec![decorator2],
        parameters: vec![],
        return_type: Some(TypeAnnotation::Named(NamedType::new(
            Range { start: Position { line: 1, column: 55 }, end: Position { line: 1, column: 61 } },
            "Result".to_string()
        ))),
        body: BlockStatement::new(
            Range { start: Position { line: 1, column: 63 }, end: Position { line: 1, column: 65 } },
            vec![]
        ),
        is_async: true,
        is_generator: false,
        generic_params: vec![],
    };

    // Procesar ambos métodos
    processor.process_method_decorators("UserController", &method1).unwrap();
    processor.process_method_decorators("UserController", &method2).unwrap();

    // Verificar que se registraron ambos endpoints
    assert_eq!(processor.get_endpoints().len(), 2);
    assert!(processor.get_endpoints().contains_key("UserController::getUsers"));
    assert!(processor.get_endpoints().contains_key("UserController::createUser"));

    let endpoint1 = processor.get_endpoints().get("UserController::getUsers").unwrap();
    assert_eq!(endpoint1.http_method, "GET");

    let endpoint2 = processor.get_endpoints().get("UserController::createUser").unwrap();
    assert_eq!(endpoint2.http_method, "POST");
}

#[test]
fn test_code_generation() {
    let mut processor = GatewayDecoratorProcessor::new();

    // Agregar un endpoint de prueba
    let endpoint = GatewayEndpointInfo {
        class_name: "TestController".to_string(),
        method_name: "testMethod".to_string(),
        http_method: "GET".to_string(),
        path: "/test".to_string(),
        middlewares: vec!["logging".to_string(), "cors".to_string()],
        auth_required: true,
        rate_limit: Some("100/min".to_string()),
        cors_enabled: true,
    };

    processor.gateway_endpoints.insert("TestController::testMethod".to_string(), endpoint);

    // Generar código
    let code = processor.generate_gateway_code();

    // Verificar que el código contiene los elementos esperados
    assert!(code.contains("GET"));
    assert!(code.contains("/test"));
    assert!(code.contains("logging"));
    assert!(code.contains("cors"));
    assert!(code.contains("100/min"));
    assert!(code.contains("TestController::testMethod"));
}

#[test]
fn test_default_values() {
    let mut processor = GatewayDecoratorProcessor::new();

    // Decorador sin argumentos (usar valores por defecto)
    let decorator = Decorator {
        name: "gateway".to_string(),
        arguments: vec![], // Sin argumentos
        range: Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 10 } },
    };

    let method = FunctionDeclaration {
        node: ASTNode::new(Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 60 } }),
        is_public: true,
        name: "defaultMethod".to_string(),
        decorators: vec![decorator],
        parameters: vec![],
        return_type: Some(TypeAnnotation::Named(NamedType::new(
            Range { start: Position { line: 0, column: 40 }, end: Position { line: 0, column: 46 } },
            "String".to_string()
        ))),
        body: BlockStatement::new(
            Range { start: Position { line: 0, column: 48 }, end: Position { line: 0, column: 50 } },
            vec![]
        ),
        is_async: false,
        is_generator: false,
        generic_params: vec![],
    };

    // Procesar el método
    processor.process_method_decorators("TestController", &method).unwrap();

    // Verificar valores por defecto
    let key = "TestController::defaultMethod";
    assert!(processor.get_endpoints().contains_key(key));

    let endpoint = processor.get_endpoints().get(key).unwrap();
    assert_eq!(endpoint.http_method, "GET"); // default
    assert_eq!(endpoint.path, "/defaultMethod"); // default basado en nombre del método
    assert_eq!(endpoint.auth_required, false); // default
    assert_eq!(endpoint.cors_enabled, true); // default
    assert!(endpoint.middlewares.is_empty()); // default
    assert!(endpoint.rate_limit.is_none()); // default
}