//! Tests unitarios para gRPC Decorators
//!
//! Implementación de: TASK-113CA
//! Historia: VELA-1080
//! Fecha: 2025-12-30

#[cfg(test)]
use super::*;
use crate::ast::*;
use crate::grpc_decorators::*;

// Helper function to create string literal expressions
fn string_expr(s: &str) -> Expression {
    let range = Range::new(Position::new(1, 1), Position::new(1, 1));
    Expression::Literal(Literal::new(range, serde_json::Value::String(s.to_string()), "string".to_string()))
}

// Helper function to create a simple range
fn simple_range() -> Range {
    Range::new(Position::new(1, 1), Position::new(1, 1))
}

// Helper function to create a simple block statement
fn empty_block() -> BlockStatement {
    BlockStatement::new(simple_range(), vec![])
}

// Helper function to create a named type annotation
fn named_type(name: &str) -> TypeAnnotation {
    TypeAnnotation::Named(NamedType {
        node: ASTNode::new(simple_range()),
        name: name.to_string(),
    })
}

// Helper function to create a parameter
fn create_parameter(name: &str, type_name: &str) -> Parameter {
    Parameter::new(
        name.to_string(),
        Some(named_type(type_name)),
        None,
        simple_range(),
    )
}

// Helper function to create a function declaration
fn create_function_declaration(
    name: &str,
    decorators: Vec<Decorator>,
    parameters: Vec<Parameter>,
    return_type: Option<TypeAnnotation>,
    is_async: bool,
) -> FunctionDeclaration {
    FunctionDeclaration::new(
        simple_range(),
        true,
        name.to_string(),
        decorators,
        parameters,
        return_type,
        empty_block(),
        is_async,
        vec![],
    )
}

#[test]
fn test_grpc_service_decorator_parsing() {
    let mut processor = GrpcDecoratorProcessor::new();

    // Crear un class declaration con decorador @grpc.service
    let decorator = Decorator {
        name: "grpc.service".to_string(),
        arguments: vec![
            string_expr("UserService"),
            string_expr("vela.user.v1"),
        ],
        range: simple_range(),
    };

    let class = ClassDeclaration {
        node: ASTNode::new(simple_range()),
        is_public: false,
        name: "UserService".to_string(),
        decorators: vec![decorator],
        constructor: None,
        fields: vec![],
        methods: vec![],
        extends: None,
        implements: vec![],
        generic_params: vec![],
    };

    // Procesar el decorador
    let result = processor.process_class_decorators(&class);
    assert!(result.is_ok());

    // Verificar que el servicio fue registrado
    assert!(processor.services.contains_key("UserService"));
    let service = &processor.services["UserService"];
    assert_eq!(service.name, "UserService");
    assert_eq!(service.package, "vela.user.v1");

    // Procesar la clase con el decorador
    processor.process_class_decorators(&class).unwrap();

    // Validar que el servicio se haya registrado correctamente
    let service_info = processor.services.get("UserService").unwrap();
    assert_eq!(service_info.name, "UserService");
    assert_eq!(service_info.package, "vela.user.v1");
    assert!(service_info.methods.is_empty());
}

#[test]
fn test_grpc_method_decorator_unary() {
    let mut processor = GrpcDecoratorProcessor::new();

    // Primero registrar un servicio
    let service_decorator = Decorator {
        name: "grpc.service".to_string(),
        arguments: vec![
            string_expr("UserService"),
            string_expr("vela.user.v1"),
        ],
        range: simple_range(),
    };

    let class = ClassDeclaration {
        node: ASTNode::new(simple_range()),
        is_public: false,
        name: "UserService".to_string(),
        decorators: vec![service_decorator],
        constructor: None,
        fields: vec![],
        methods: vec![],
        extends: None,
        implements: vec![],
        generic_params: vec![],
    };

    processor.process_class_decorators(&class).unwrap();

    // Ahora agregar un método unary
    let method_decorator = Decorator {
        name: "grpc.method".to_string(),
        arguments: vec![
            string_expr("GetUser"),
            string_expr("unary"),
        ],
        range: simple_range(),
    };

    let method = create_function_declaration(
        "getUser",
        vec![method_decorator],
        vec![create_parameter("request", "GetUserRequest")],
        Some(named_type("Result<User, Error>")),
        true,
    );

    // Procesar el método
    let result = processor.process_method_decorators(&method);
    assert!(result.is_ok());

    // Verificar que el método fue registrado
    assert!(processor.methods.contains_key("getUser"));
    let method_info = &processor.methods["getUser"];
    assert_eq!(method_info.method_name, "GetUser");
    assert_eq!(method_info.streaming_type, GrpcStreamingType::Unary);
}

#[test]
fn test_grpc_method_decorator_server_streaming() {
    let mut processor = GrpcDecoratorProcessor::new();

    // Registrar servicio
    let service_decorator = Decorator {
        name: "grpc.service".to_string(),
        arguments: vec![
            string_expr("UserService"),
            string_expr("vela.user.v1"),
        ],
        range: simple_range(),
    };

    let class = ClassDeclaration {
        node: ASTNode::new(simple_range()),
        is_public: false,
        name: "UserService".to_string(),
        decorators: vec![service_decorator],
        constructor: None,
        fields: vec![],
        methods: vec![],
        extends: None,
        implements: vec![],
        generic_params: vec![],
    };

    processor.process_class_decorators(&class).unwrap();

    // Agregar método server streaming
    let method_decorator = Decorator {
        name: "grpc.method".to_string(),
        arguments: vec![
            string_expr("ListUsers"),
            string_expr("server_streaming"),
        ],
        range: simple_range(),
    };

    let method = create_function_declaration(
        "listUsers",
        vec![method_decorator],
        vec![create_parameter("request", "ListUsersRequest")],
        Some(named_type("Stream<User>")),
        true,
    );

    processor.process_method_decorators(&method).unwrap();

    let method_info = &processor.methods["listUsers"];
    assert_eq!(method_info.method_name, "ListUsers");
    assert_eq!(method_info.streaming_type, GrpcStreamingType::ServerStreaming);
}

#[test]
fn test_grpc_method_without_service_fails() {
    let mut processor = GrpcDecoratorProcessor::new();

    let method_decorator = Decorator {
        name: "grpc.method".to_string(),
        arguments: vec![
            string_expr("GetUser"),
            string_expr("unary"),
        ],
        range: simple_range(),
    };

    let method = create_function_declaration(
        "getUser",
        vec![method_decorator],
        vec![create_parameter("request", "GetUserRequest")],
        Some(named_type("User")),
        true,
    );

    // Procesar el método sin servicio registrado debería fallar
    let result = processor.process_method_decorators(&method);
    assert!(result.is_err());
}

#[test]
fn test_invalid_streaming_type_fails() {
    let mut processor = GrpcDecoratorProcessor::new();

    // Registrar servicio
    let service_decorator = Decorator {
        name: "grpc.service".to_string(),
        arguments: vec![
            string_expr("UserService"),
            string_expr("vela.user.v1"),
        ],
        range: simple_range(),
    };

    let class = ClassDeclaration {
        node: ASTNode::new(simple_range()),
        is_public: false,
        name: "UserService".to_string(),
        decorators: vec![service_decorator],
        constructor: None,
        fields: vec![],
        methods: vec![],
        extends: None,
        implements: vec![],
        generic_params: vec![],
    };

    processor.process_class_decorators(&class).unwrap();

    // Intentar método con streaming inválido
    let method_decorator = Decorator {
        name: "grpc.method".to_string(),
        arguments: vec![
            string_expr("GetUser"),
            string_expr("invalid_type"),
        ],
        range: simple_range(),
    };

    let method = create_function_declaration(
        "getUser",
        vec![method_decorator],
        vec![create_parameter("request", "GetUserRequest")],
        Some(named_type("User")),
        true,
    );

    let result = processor.process_method_decorators(&method);
    assert!(result.is_err());
}

#[test]
fn test_protobuf_generation() {
    let mut processor = GrpcDecoratorProcessor::new();

    // Registrar servicio con métodos
    let service_decorator = Decorator {
        name: "grpc.service".to_string(),
        arguments: vec![
            string_expr("UserService"),
            string_expr("vela.user.v1"),
        ],
        range: simple_range(),
    };

    let class = ClassDeclaration {
        node: ASTNode::new(simple_range()),
        is_public: false,
        name: "UserService".to_string(),
        decorators: vec![service_decorator],
        constructor: None,
        fields: vec![],
        methods: vec![],
        extends: None,
        implements: vec![],
        generic_params: vec![],
    };

    processor.process_class_decorators(&class).unwrap();

    // Agregar métodos
    let unary_method_decorator = Decorator {
        name: "grpc.method".to_string(),
        arguments: vec![
            string_expr("GetUser"),
            string_expr("unary"),
        ],
        range: simple_range(),
    };

    let unary_method = create_function_declaration(
        "getUser",
        vec![unary_method_decorator],
        vec![create_parameter("request", "GetUserRequest")],
        Some(named_type("User")),
        true,
    );

    let streaming_method_decorator = Decorator {
        name: "grpc.method".to_string(),
        arguments: vec![
            string_expr("ListUsers"),
            string_expr("server_streaming"),
        ],
        range: simple_range(),
    };

    let streaming_method = create_function_declaration(
        "listUsers",
        vec![streaming_method_decorator],
        vec![create_parameter("request", "ListUsersRequest")],
        Some(named_type("Stream<User>")),
        true,
    );

    processor.process_method_decorators(&unary_method).unwrap();
    processor.process_method_decorators(&streaming_method).unwrap();

    // Generar protobuf
    let proto = processor.generate_protobuf();

    // Verificar contenido
    assert!(proto.contains("syntax = \"proto3\";"));
    assert!(proto.contains("package vela.user.v1;"));
    assert!(proto.contains("service UserService {"));
    assert!(proto.contains("rpc GetUser(GetUserRequest) returns (User);"));
    assert!(proto.contains("rpc ListUsers(ListUsersRequest) returns (stream User);"));
}

#[test]
fn test_runtime_code_generation() {
    let mut processor = GrpcDecoratorProcessor::new();

    // Registrar servicio simple
    let service_decorator = Decorator {
        name: "grpc.service".to_string(),
        arguments: vec![
            string_expr("TestService"),
            string_expr("vela.test.v1"),
        ],
        range: simple_range(),
    };

    let class = ClassDeclaration {
        node: ASTNode::new(simple_range()),
        is_public: false,
        name: "TestService".to_string(),
        decorators: vec![service_decorator],
        constructor: None,
        fields: vec![],
        methods: vec![],
        extends: None,
        implements: vec![],
        generic_params: vec![],
    };

    processor.process_class_decorators(&class).unwrap();

    // Agregar método
    let method_decorator = Decorator {
        name: "grpc.method".to_string(),
        arguments: vec![
            string_expr("TestMethod"),
            string_expr("unary"),
        ],
        range: simple_range(),
    };

    let method = create_function_declaration(
        "testMethod",
        vec![method_decorator],
        vec![create_parameter("request", "TestRequest")],
        Some(named_type("TestResponse")),
        true,
    );

    processor.process_method_decorators(&method).unwrap();

    // Generar código runtime
    let code = processor.generate_runtime_code();

    // Verificar contenido
    assert!(code.contains("// Código generado automáticamente para servicios gRPC"));
    assert!(code.contains("pub mod testservice"));
    assert!(code.contains("#[tonic::async_trait]"));
    assert!(code.contains("pub trait TestService"));
    assert!(code.contains("async fn test_method"));
    assert!(code.contains("unimplemented!()"));
}

    #[test]
    fn test_codegen_from_proto_stub() {
        // Simula la generación de código Rust desde un archivo .proto
        let result = GrpcDecoratorProcessor::generate_rust_from_proto("test.proto", "./out");
        assert!(result.is_ok());
    }

#[test]
fn test_grpc_method_decorator_client_streaming() {
    let mut processor = GrpcDecoratorProcessor::new();

    // Registrar servicio
    let service_decorator = Decorator {
        name: "grpc.service".to_string(),
        arguments: vec![
            string_expr("StreamService"),
            string_expr("vela.stream.v1"),
        ],
        range: simple_range(),
    };

    let class = ClassDeclaration {
        node: ASTNode::new(simple_range()),
        is_public: false,
        name: "StreamService".to_string(),
        decorators: vec![service_decorator],
        constructor: None,
        fields: vec![],
        methods: vec![],
        extends: None,
        implements: vec![],
        generic_params: vec![],
    };

    processor.process_class_decorators(&class).unwrap();

    // Agregar método client streaming
    let method_decorator = Decorator {
        name: "grpc.method".to_string(),
        arguments: vec![
            string_expr("UploadData"),
            string_expr("client_streaming"),
        ],
        range: simple_range(),
    };

    let method = create_function_declaration(
        "uploadData",
        vec![method_decorator],
        vec![create_parameter("stream", "Stream<DataChunk>")],
        Some(named_type("UploadResponse")),
        true,
    );

    let result = processor.process_method_decorators(&method);
    assert!(result.is_ok());
    let method_info = &processor.methods["uploadData"];
    assert_eq!(method_info.method_name, "UploadData");
    assert_eq!(method_info.streaming_type, GrpcStreamingType::ClientStreaming);
}

#[test]
fn test_grpc_method_decorator_bidirectional_streaming() {
    let mut processor = GrpcDecoratorProcessor::new();

    // Registrar servicio
    let service_decorator = Decorator {
        name: "grpc.service".to_string(),
        arguments: vec![
            string_expr("ChatService"),
            string_expr("vela.chat.v1"),
        ],
        range: simple_range(),
    };

    let class = ClassDeclaration {
        node: ASTNode::new(simple_range()),
        is_public: false,
        name: "ChatService".to_string(),
        decorators: vec![service_decorator],
        constructor: None,
        fields: vec![],
        methods: vec![],
        extends: None,
        implements: vec![],
        generic_params: vec![],
    };

    processor.process_class_decorators(&class).unwrap();

    // Agregar método bidirectional streaming
    let method_decorator = Decorator {
        name: "grpc.method".to_string(),
        arguments: vec![
            string_expr("Chat"),
            string_expr("bidirectional_streaming"),
        ],
        range: simple_range(),
    };

    let method = create_function_declaration(
        "chat",
        vec![method_decorator],
        vec![create_parameter("stream", "Stream<ChatMessage>")],
        Some(named_type("Stream<ChatMessage>")),
        true,
    );

    let result = processor.process_method_decorators(&method);
    assert!(result.is_ok());
    let method_info = &processor.methods["chat"];
    assert_eq!(method_info.method_name, "Chat");
    assert_eq!(method_info.streaming_type, GrpcStreamingType::BidirectionalStreaming);
}