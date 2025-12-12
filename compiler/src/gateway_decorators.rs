//! Gateway Decorators para Vela
//!
//! Implementación de: VELA-611
//! Historia: VELA-611
//! Fecha: 2025-12-11
//!
//! Descripción:
//! Decoradores compile-time para definir endpoints del API Gateway.
//! Genera código que integra con el sistema de routing y middleware.

use crate::ast::*;
use crate::error::CompileResult;
use std::collections::HashMap;

/// Processor para decoradores de gateway
pub struct GatewayDecoratorProcessor {
    pub gateway_endpoints: HashMap<String, GatewayEndpointInfo>,
}

#[derive(Debug, Clone)]
pub struct GatewayEndpointInfo {
    pub class_name: String,
    pub method_name: String,
    pub http_method: String,
    pub path: String,
    pub middlewares: Vec<String>,
    pub auth_required: bool,
    pub rate_limit: Option<String>,
    pub cors_enabled: bool,
}

impl GatewayDecoratorProcessor {
    pub fn new() -> Self {
        Self {
            gateway_endpoints: HashMap::new(),
        }
    }

    /// Procesar decoradores @gateway en métodos
    pub fn process_method_decorators(&mut self, class_name: &str, method: &FunctionDeclaration) -> CompileResult<()> {
        // Buscar decorador @gateway
        let gateway_decorator = method.decorators.iter().find(|d| d.name == "gateway");

        if let Some(decorator) = gateway_decorator {
            let endpoint_info = self.extract_gateway_endpoint_info(class_name, method, decorator)?;
            let key = format!("{}::{}", class_name, method.name);
            self.gateway_endpoints.insert(key, endpoint_info);
        }

        Ok(())
    }

    /// Extraer información del endpoint del decorador @gateway
    fn extract_gateway_endpoint_info(&self, class_name: &str, method: &FunctionDeclaration, decorator: &Decorator) -> CompileResult<GatewayEndpointInfo> {
        let mut http_method = "GET".to_string(); // Default
        let mut path = format!("/{}", method.name); // Default path
        let mut middlewares = Vec::new();
        let mut auth_required = false;
        let mut rate_limit = None;
        let mut cors_enabled = true; // Default enabled

        // Procesar argumentos del decorador (StructLiteral con campos nombrados)
        if !decorator.arguments.is_empty() {
            if let Expression::StructLiteral(struct_lit) = &decorator.arguments[0] {
                for field in &struct_lit.fields {
                    match field.name.as_str() {
                        "method" => {
                            if let Expression::Literal(lit) = &field.value {
                                if lit.kind == "string" {
                                    if let serde_json::Value::String(method_str) = &lit.value {
                                        http_method = method_str.clone();
                                    }
                                }
                            }
                        }
                        "path" => {
                            if let Expression::Literal(lit) = &field.value {
                                if lit.kind == "string" {
                                    if let serde_json::Value::String(path_str) = &lit.value {
                                        path = path_str.clone();
                                    }
                                }
                            }
                        }
                        "middlewares" => {
                            if let Expression::ArrayLiteral(array_lit) = &field.value {
                                for element in &array_lit.elements {
                                    if let Expression::Literal(lit) = element {
                                        if lit.kind == "string" {
                                            if let serde_json::Value::String(mw) = &lit.value {
                                                middlewares.push(mw.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "auth" => {
                            if let Expression::Literal(lit) = &field.value {
                                if lit.kind == "bool" {
                                    if let serde_json::Value::Bool(auth) = &lit.value {
                                        auth_required = *auth;
                                    }
                                }
                            }
                        }
                        "rateLimit" => {
                            if let Expression::Literal(lit) = &field.value {
                                if lit.kind == "string" {
                                    if let serde_json::Value::String(limit) = &lit.value {
                                        rate_limit = Some(limit.clone());
                                    }
                                }
                            }
                        }
                        "cors" => {
                            if let Expression::Literal(lit) = &field.value {
                                if lit.kind == "bool" {
                                    if let serde_json::Value::Bool(cors) = &lit.value {
                                        cors_enabled = *cors;
                                    }
                                }
                            }
                        }
                        _ => {} // Ignorar campos desconocidos
                    }
                }
            }
        }

        Ok(GatewayEndpointInfo {
            class_name: class_name.to_string(),
            method_name: method.name.clone(),
            http_method,
            path,
            middlewares,
            auth_required,
            rate_limit,
            cors_enabled,
        })
    }

    /// Generar código de integración con el API Gateway
    pub fn generate_gateway_code(&self) -> String {
        let mut code = String::new();

        code.push_str("// Generated Gateway Integration Code\n");
        code.push_str("// This code integrates decorated methods with the API Gateway\n\n");

        for (key, endpoint) in &self.gateway_endpoints {
            code.push_str(&format!("// Endpoint: {} -> {} {}\n",
                key, endpoint.http_method, endpoint.path));

            // Generate route registration
            code.push_str(&format!("gateway.register_route(\n"));
            code.push_str(&format!("    \"{}\", \"{}\",\n", endpoint.http_method, endpoint.path));
            code.push_str(&format!("    {}::{},\n", endpoint.class_name, endpoint.method_name));

            // Generate middleware configuration
            if !endpoint.middlewares.is_empty() {
                code.push_str("    &[");
                for (i, mw) in endpoint.middlewares.iter().enumerate() {
                    if i > 0 { code.push_str(", "); }
                    code.push_str(&format!("\"{}\"", mw));
                }
                code.push_str("],\n");
            } else {
                code.push_str("    &[],\n");
            }

            // Generate auth configuration
            code.push_str(&format!("    {},\n", endpoint.auth_required));

            // Generate rate limit configuration
            if let Some(limit) = &endpoint.rate_limit {
                code.push_str(&format!("    Some(\"{}\"),\n", limit));
            } else {
                code.push_str("    None,\n");
            }

            // Generate CORS configuration
            code.push_str(&format!("    {}\n", endpoint.cors_enabled));
            code.push_str(");\n\n");
        }

        code
    }

    /// Obtener todos los endpoints registrados
    pub fn get_endpoints(&self) -> &HashMap<String, GatewayEndpointInfo> {
        &self.gateway_endpoints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_decorator_processing() {
        let mut processor = GatewayDecoratorProcessor::new();

        // Create a mock method with @gateway decorator
        let decorator_args = StructLiteral::new(
            Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 50 } },
            "".to_string(), // No struct name for decorator args
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
            ]
        );

        let decorator = Decorator {
            name: "gateway".to_string(),
            arguments: vec![Expression::StructLiteral(decorator_args)],
            range: Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 50 } },
        };

        let method = FunctionDeclaration {
            node: ASTNode::new(Range { start: Position { line: 0, column: 0 }, end: Position { line: 0, column: 100 } }),
            is_public: true,
            name: "createUser".to_string(),
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
            is_async: false,
            generic_params: vec![],
        };

        // Process the method
        processor.process_method_decorators("UserController", &method).unwrap();

        // Verify the endpoint was registered
        let key = "UserController::createUser";
        assert!(processor.get_endpoints().contains_key(key));

        let endpoint = processor.get_endpoints().get(key).unwrap();
        assert_eq!(endpoint.http_method, "POST");
        assert_eq!(endpoint.path, "/api/users");
        assert_eq!(endpoint.auth_required, true);
    }

    #[test]
    fn test_gateway_code_generation() {
        let mut processor = GatewayDecoratorProcessor::new();

        // Add a test endpoint
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

        // Generate code
        let code = processor.generate_gateway_code();

        // Verify code contains expected elements
        assert!(code.contains("GET"));
        assert!(code.contains("/test"));
        assert!(code.contains("logging"));
        assert!(code.contains("cors"));
        assert!(code.contains("100/min"));
    }
}