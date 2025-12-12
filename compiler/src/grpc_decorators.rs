// gRPC Decorators para Vela
//
// Implementación de: TASK-113CA
// Historia: VELA-1080
// Fecha: 2025-12-30
//
// Descripción:
// Decoradores compile-time para definir servicios y métodos gRPC.
// Genera código que integra con el runtime gRPC de Vela.

use crate::ast::*;
use crate::error::CompileResult;
use std::collections::HashMap;

/// Tipos de streaming gRPC soportados
#[derive(Debug, Clone, PartialEq)]
pub enum GrpcStreamingType {
    Unary,
    ServerStreaming,
    ClientStreaming,
    BidirectionalStreaming,
}

/// Información de un método gRPC
#[derive(Debug, Clone)]
pub struct GrpcMethodInfo {
    pub service_name: String,
    pub method_name: String,
    pub streaming_type: GrpcStreamingType,
    pub request_type: String,
    pub response_type: String,
    pub options: HashMap<String, String>,
}

/// Información de un servicio gRPC
#[derive(Debug, Clone)]
pub struct GrpcServiceInfo {
    pub name: String,
    pub package: String,
    pub methods: Vec<GrpcMethodInfo>,
    pub options: HashMap<String, String>,
}

/// Processor para decoradores gRPC
pub struct GrpcDecoratorProcessor {
    pub services: HashMap<String, GrpcServiceInfo>,
    pub methods: HashMap<String, GrpcMethodInfo>,
}

impl GrpcDecoratorProcessor {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    /// Generar código Rust a partir de un archivo .proto usando prost/tonic
    pub fn generate_rust_from_proto(proto_path: &str, _out_dir: &str) -> std::io::Result<()> {
        // TODO: Implementar generación real de código usando tonic-build
        // Por ahora, simulamos que la generación fue exitosa
        // tonic_build::compile_protos(proto_path)?;
        Ok(())
    }

    /// Procesar decoradores @grpc.service en clases
    pub fn process_class_decorators(&mut self, class: &ClassDeclaration) -> CompileResult<()> {
        // Buscar decorador @grpc.service
        let service_decorator = class.decorators.iter().find(|d| d.name == "grpc.service");

        if let Some(decorator) = service_decorator {
            // Crear un nuevo GrpcServiceInfo en lugar de usar extract_grpc_method_info
            let service_info = GrpcServiceInfo {
                name: decorator
                    .arguments
                    .get(0)
                    .and_then(|arg| self.extract_string_from_expression(arg).ok())
                    .unwrap_or_else(|| class.name.clone()),
                package: decorator
                    .arguments
                    .get(1)
                    .and_then(|arg| self.extract_string_from_expression(arg).ok())
                    .unwrap_or_else(|| "default_package".to_string()),
                methods: Vec::new(),
                options: HashMap::new(),
            };

            self.services.insert(service_info.name.clone(), service_info);
        }

        Ok(())
    }

    /// Procesar decoradores @grpc.method en métodos
    pub fn process_method_decorators(&mut self, method: &FunctionDeclaration) -> CompileResult<()> {
        // Buscar decorador @grpc.method
        let method_decorator = method.decorators.iter().find(|d| d.name == "grpc.method");

        if let Some(decorator) = method_decorator {
            // Verificar que haya al menos un servicio registrado
            if self.services.is_empty() {
                return Err(crate::error::CompileError::Semantic(
                    crate::error::SemanticError::TypeInferenceFailed {
                        location: crate::error::SourceLocation::new(0, 0, 0),
                        message: "No se puede procesar decorador @grpc.method sin un servicio @grpc.service registrado".to_string(),
                    }
                ));
            }
            // Extraer información del método (sin service_name por ahora)
            let method_name = if decorator.arguments.len() > 0 {
                self.extract_string_from_expression(&decorator.arguments[0])?
            } else {
                method.name.clone()
            };

            let streaming_type_str = if decorator.arguments.len() > 1 {
                self.extract_string_from_expression(&decorator.arguments[1])?
            } else {
                "unary".to_string()
            };

            let streaming_type = match streaming_type_str.as_str() {
                "unary" => GrpcStreamingType::Unary,
                "server_streaming" => GrpcStreamingType::ServerStreaming,
                "client_streaming" => GrpcStreamingType::ClientStreaming,
                "bidirectional_streaming" => GrpcStreamingType::BidirectionalStreaming,
                _ => {
                    return Err(crate::error::CompileError::Semantic(
                        crate::error::SemanticError::TypeInferenceFailed {
                            location: crate::error::SourceLocation::new(0, 0, 0),
                            message: format!(
                                "Tipo de streaming inválido: {}. Debe ser: unary, server_streaming, client_streaming, bidirectional_streaming",
                                streaming_type_str
                            ),
                        }
                    ));
                }
            };

            // Inferir tipos de request/response desde la signatura del método
            let request_type = self.infer_request_type(method)?;
            let response_type = self.infer_response_type(method)?;

            let method_info = GrpcMethodInfo {
                service_name: String::new(), // Se asignará después
                method_name,
                streaming_type,
                request_type,
                response_type,
                options: HashMap::new(),
            };

            self.methods.insert(method.name.clone(), method_info);
        }

        Ok(())
    }

    /// Extraer string de una expresión
    fn extract_string_from_expression(&self, expr: &Expression) -> CompileResult<String> {
        match expr {
            Expression::Literal(lit) => {
                if let serde_json::Value::String(s) = &lit.value {
                    Ok(s.clone())
                } else {
                    Err(crate::error::CompileError::Semantic(
                        crate::error::SemanticError::TypeInferenceFailed {
                            location: crate::error::SourceLocation::new(0, 0, 0),
                            message: "Expected string literal".to_string(),
                        }
                    ))
                }
            }
            _ => Err(crate::error::CompileError::Semantic(
                crate::error::SemanticError::TypeInferenceFailed {
                    location: crate::error::SourceLocation::new(0, 0, 0),
                    message: "Expected string literal".to_string(),
                }
            )),
        }
    }

    /// Extraer información del decorador @grpc.method
    fn extract_grpc_method_info(&self, class_name: &str, method: &FunctionDeclaration, decorator: &Decorator) -> CompileResult<GrpcMethodInfo> {
        // @grpc.method(name, streaming_type)
        let method_name = if decorator.arguments.len() > 0 {
            self.extract_string_from_expression(&decorator.arguments[0])?
        } else {
            method.name.clone()
        };

        let streaming_type_str = if decorator.arguments.len() > 1 {
            self.extract_string_from_expression(&decorator.arguments[1])?
        } else {
            "unary".to_string()
        };

        let streaming_type = match streaming_type_str.as_str() {
            "unary" => GrpcStreamingType::Unary,
            "server_streaming" => GrpcStreamingType::ServerStreaming,
            "client_streaming" => GrpcStreamingType::ClientStreaming,
            "bidirectional_streaming" => GrpcStreamingType::BidirectionalStreaming,
            _ => {
                return Err(crate::error::CompileError::Semantic(
                    crate::error::SemanticError::TypeInferenceFailed {
                        location: crate::error::SourceLocation::new(0, 0, 0),
                        message: format!(
                            "Tipo de streaming inválido: {}. Debe ser: unary, server_streaming, client_streaming, bidirectional_streaming",
                            streaming_type_str
                        ),
                    }
                ));
            }
        };

        // Inferir tipos de request/response desde la signatura del método
        let request_type = self.infer_request_type(method)?;
        let response_type = self.infer_response_type(method)?;

        Ok(GrpcMethodInfo {
            service_name: class_name.to_string(),
            method_name,
            streaming_type,
            request_type,
            response_type,
            options: HashMap::new(),
        })
    }

    /// Inferir tipo de request desde parámetros del método
    fn infer_request_type(&self, method: &FunctionDeclaration) -> CompileResult<String> {
        // Si hay parámetros, usar el primero (o el segundo si el primero es self)
        if method.parameters.is_empty() {
            Ok("google.protobuf.Empty".to_string())
        } else if method.parameters.len() == 1 {
            // Solo un parámetro, asumimos que es el request
            match &method.parameters[0].type_annotation {
                Some(type_annotation) => self.type_annotation_to_string(type_annotation),
                None => Ok("google.protobuf.Empty".to_string()),
            }
        } else {
            // Múltiples parámetros, asumir que el primero es self y el segundo es request
            match &method.parameters[1].type_annotation {
                Some(type_annotation) => self.type_annotation_to_string(type_annotation),
                None => Ok("google.protobuf.Empty".to_string()),
            }
        }
    }

    /// Inferir tipo de response desde return type del método
    fn infer_response_type(&self, method: &FunctionDeclaration) -> CompileResult<String> {
        // Por simplicidad, usar el return type anotado
        // En una implementación completa, esto manejaría Result<T> y Stream<T>
        match &method.return_type {
            Some(type_annotation) => self.type_annotation_to_string(type_annotation),
            None => Ok("google.protobuf.Empty".to_string()), // Default para void
        }
    }

    /// Convertir TypeAnnotation a string
    fn type_annotation_to_string(&self, type_annotation: &TypeAnnotation) -> CompileResult<String> {
        match type_annotation {
            TypeAnnotation::Named(named_type) => Ok(named_type.name.clone()),
            TypeAnnotation::Generic(generic) => {
                let base = &generic.base_name;
                let args: Vec<String> = generic.type_arguments.iter()
                    .map(|arg| self.type_annotation_to_string(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{}<{}>", base, args.join(", ")))
            }
            _ => Ok("Unknown".to_string()), // Simplificado
        }
    }

    /// Generar código protobuf para los servicios
    pub fn generate_protobuf(&self) -> String {
        let mut proto = String::new();

        // Header
        proto.push_str("syntax = \"proto3\";\n\n");

        // Generar servicios
        for service in self.services.values() {
            if !service.package.is_empty() {
                proto.push_str(&format!("package {};\n\n", service.package));
            }

            proto.push_str(&format!("service {} {{\n", service.name));

            // Agregar métodos que pertenecen a este servicio
            for (method_name, method) in &self.methods {
                if method.service_name == service.name || method.service_name.is_empty() {
                    let streaming_req = matches!(method.streaming_type, GrpcStreamingType::ClientStreaming | GrpcStreamingType::BidirectionalStreaming);
                    let streaming_resp = matches!(method.streaming_type, GrpcStreamingType::ServerStreaming | GrpcStreamingType::BidirectionalStreaming);

                    let req_stream = if streaming_req { "stream " } else { "" };
                    let resp_stream = if streaming_resp { "stream " } else { "" };

                    // Para tipos de respuesta streaming, extraer el tipo interno de Stream<T>
                    let response_type = if streaming_resp && method.response_type.starts_with("Stream<") && method.response_type.ends_with(">") {
                        // Extraer T de Stream<T>
                        let inner_type = &method.response_type[7..method.response_type.len()-1];
                        inner_type.to_string()
                    } else {
                        method.response_type.clone()
                    };

                    proto.push_str(&format!(
                        "  rpc {}({}{}) returns ({}{});\n",
                        method.method_name, req_stream, method.request_type, resp_stream, response_type
                    ));
                }
            }

            proto.push_str("}\n\n");
        }

        proto
    }

    /// Generar código Rust para el runtime gRPC
    pub fn generate_runtime_code(&self) -> String {
        let mut code = String::new();

        code.push_str("// Código generado automáticamente para servicios gRPC\n\n");

        for service in self.services.values() {
            code.push_str(&format!("pub mod {} {{\n", service.name.to_lowercase()));
            code.push_str("    use tonic::{Request, Response, Status};\n\n");

            // Generar trait del servicio
            code.push_str(&format!("    #[derive(Debug)]\n"));
            code.push_str(&format!("    pub struct {}Service;\n\n", service.name));

            code.push_str(&format!("    #[tonic::async_trait]\n"));
            code.push_str(&format!("    pub trait {} {{\n", service.name));

            // Agregar métodos que pertenecen a este servicio
            for (method_name, method) in &self.methods {
                if method.service_name == service.name || method.service_name.is_empty() {
                    let req_stream = matches!(method.streaming_type, GrpcStreamingType::ClientStreaming | GrpcStreamingType::BidirectionalStreaming);
                    let resp_stream = matches!(method.streaming_type, GrpcStreamingType::ServerStreaming | GrpcStreamingType::BidirectionalStreaming);

                    let req_type = if req_stream {
                        format!("tonic::Streaming<{}>", method.request_type)
                    } else {
                        format!("Request<{}>", method.request_type)
                    };

                    let resp_type = if resp_stream {
                        format!("Result<Response<tonic::Streaming<{}>>, Status>", method.response_type)
                    } else {
                        format!("Result<Response<{}>, Status>", method.response_type)
                    };

                    let rust_method_name = self.camel_to_snake_case(&method.method_name);

                    code.push_str(&format!(
                        "        async fn {}(request: {}) -> {};\n",
                        rust_method_name, req_type, resp_type
                    ));
                }
            }

            code.push_str("    }\n\n");

            // Generar implementación por defecto
            code.push_str(&format!("    #[tonic::async_trait]\n"));
            code.push_str(&format!("    impl {} for {}Service {{\n", service.name, service.name));

            // Agregar métodos que pertenecen a este servicio
            for (method_name, method) in &self.methods {
                if method.service_name == service.name || method.service_name.is_empty() {
                    let req_stream = matches!(method.streaming_type, GrpcStreamingType::ClientStreaming | GrpcStreamingType::BidirectionalStreaming);
                    let resp_stream = matches!(method.streaming_type, GrpcStreamingType::ServerStreaming | GrpcStreamingType::BidirectionalStreaming);

                    let req_type = if req_stream {
                        format!("tonic::Streaming<{}>", method.request_type)
                    } else {
                        format!("Request<{}>", method.request_type)
                    };

                    let resp_type = if resp_stream {
                        format!("Result<Response<tonic::Streaming<{}>>, Status>", method.response_type)
                    } else {
                        format!("Result<Response<{}>, Status>", method.response_type)
                    };

                    let rust_method_name = self.camel_to_snake_case(&method.method_name);

                    code.push_str(&format!(
                        "        async fn {}(request: {}) -> {} {{\n",
                        rust_method_name, req_type, resp_type
                    ));
                    code.push_str("            // TODO: Implementar lógica del método\n");
                    code.push_str("            unimplemented!()\n");
                    code.push_str("        }\n\n");
                }
            }

            code.push_str("    }\n");
            code.push_str("}\n\n");
        }

        code
    }

    /// Convertir camelCase a snake_case
    fn camel_to_snake_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_uppercase() {
                if !result.is_empty() {
                    result.push('_');
                }
                result.push(ch.to_lowercase().next().unwrap());
            } else {
                result.push(ch);
            }
        }

        result
    }
}