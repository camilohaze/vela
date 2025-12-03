/*
Tests unitarios para el módulo AST

Jira: TASK-RUST-102
Historia: US-RUST-02 (Compiler Foundation)
Fecha: 2025-12-01

Cobertura de tests: AST nodes básicos
*/

use vela_compiler::ast::*;
use vela_compiler::error::{CompileError, CompileResult};

#[cfg(test)]
mod tests {
    use super::*;

    // ===================================================================
    // UTILITY FUNCTIONS TESTS
    // ===================================================================

    #[test]
    fn test_create_position() {
        let pos = create_position(1, 5);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 5);
    }

    #[test]
    fn test_create_range() {
        let range = create_range(1, 1, 1, 10);
        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.column, 1);
        assert_eq!(range.end.line, 1);
        assert_eq!(range.end.column, 10);
    }

    #[test]
    fn test_position_display() {
        let pos = create_position(5, 10);
        assert_eq!(format!("{}", pos), "5:10");
    }

    #[test]
    fn test_range_display() {
        let range = create_range(1, 1, 2, 5);
        assert_eq!(format!("{}", range), "1:1 - 2:5");
    }

    // ===================================================================
    // AST NODE TESTS
    // ===================================================================

    #[test]
    fn test_ast_node_creation() {
        let range = create_range(1, 1, 1, 10);
        let node = ASTNode::new(range.clone());
        assert_eq!(node.range, range);
    }

    // ===================================================================
    // PROGRAM TESTS
    // ===================================================================

    #[test]
    fn test_program_creation() {
        let range = create_range(1, 1, 10, 1);
        let program = Program::new(range, vec![], vec![]);
        assert_eq!(program.imports.len(), 0);
        assert_eq!(program.declarations.len(), 0);
    }

    #[test]
    fn test_program_with_imports_and_declarations() {
        let range = create_range(1, 1, 10, 1);

        // Create an import
        let import_range = create_range(1, 1, 1, 25);
        let import = ImportDeclaration::new(
            import_range,
            ImportKind::Package,
            "http".to_string(),
            None,
            None,
            None,
        );

        // Create a function declaration
        let func_range = create_range(3, 1, 5, 2);
        let body = BlockStatement::new(create_range(5, 1, 5, 2), vec![]);
        let func = FunctionDeclaration::new(
            func_range,
            true,
            "main".to_string(),
            vec![],
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(3, 15, 3, 19),
                "void".to_string(),
            ))),
            body,
            false,
            vec![],
        );

        let program = Program::new(
            range,
            vec![import],
            vec![Declaration::Function(func)],
        );

        assert_eq!(program.imports.len(), 1);
        assert_eq!(program.declarations.len(), 1);
    }

    // ===================================================================
    // IMPORT TESTS
    // ===================================================================

    #[test]
    fn test_import_declaration_creation() {
        let range = create_range(1, 1, 1, 25);
        let import = ImportDeclaration::new(
            range,
            ImportKind::Package,
            "http".to_string(),
            Some("net".to_string()),
            Some(vec!["Client".to_string()]),
            Some(vec!["Server".to_string()]),
        );

        assert_eq!(import.kind, ImportKind::Package);
        assert_eq!(import.path, "http");
        assert_eq!(import.alias, Some("net".to_string()));
        assert_eq!(import.show, Some(vec!["Client".to_string()]));
        assert_eq!(import.hide, Some(vec!["Server".to_string()]));
    }

    #[test]
    fn test_import_kinds() {
        assert_eq!(ImportKind::System, ImportKind::System);
        assert_eq!(ImportKind::Package, ImportKind::Package);
        assert_eq!(ImportKind::Module, ImportKind::Module);
        assert_eq!(ImportKind::Library, ImportKind::Library);
        assert_eq!(ImportKind::Extension, ImportKind::Extension);
        assert_eq!(ImportKind::Assets, ImportKind::Assets);
    }

    // ===================================================================
    // FUNCTION DECLARATION TESTS
    // ===================================================================

    #[test]
    fn test_function_declaration_creation() {
        let range = create_range(1, 1, 3, 2);
        let body = BlockStatement::new(create_range(3, 1, 3, 2), vec![]);
        let func = FunctionDeclaration::new(
            range,
            true,
            "add".to_string(),
            vec![],
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 20, 1, 26),
                "Number".to_string(),
            ))),
            body,
            false,
            vec![],
        );

        assert_eq!(func.name, "add");
        assert!(func.is_public);
        assert!(!func.is_async);
        assert_eq!(func.parameters.len(), 0);
        assert!(func.generic_params.is_empty());
    }

    #[test]
    fn test_async_function_declaration() {
        let range = create_range(1, 1, 3, 2);
        let body = BlockStatement::new(create_range(3, 1, 3, 2), vec![]);
        let func = FunctionDeclaration::new(
            range,
            false,
            "fetchData".to_string(),
            vec![],
            Some(TypeAnnotation::Generic(GenericType::new(
                create_range(1, 25, 1, 35),
                "Result".to_string(),
                vec![TypeAnnotation::Primitive(PrimitiveType::new(
                    create_range(1, 32, 1, 38),
                    "String".to_string(),
                ))],
            ))),
            body,
            true, // async
            vec![],
        );

        assert_eq!(func.name, "fetchData");
        assert!(!func.is_public);
        assert!(func.is_async);
    }

    #[test]
    fn test_function_with_parameters() {
        let param_range = create_range(1, 11, 1, 17);
        let param = Parameter::new(
            "a".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 11, 1, 17),
                "Number".to_string(),
            ))),
            None,
            param_range,
        );

        let func_range = create_range(1, 1, 3, 2);
        let body = BlockStatement::new(create_range(3, 1, 3, 2), vec![]);
        let func = FunctionDeclaration::new(
            func_range,
            true,
            "add".to_string(),
            vec![param],
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 30, 1, 36),
                "Number".to_string(),
            ))),
            body,
            false,
            vec![],
        );

        assert_eq!(func.parameters.len(), 1);
        assert_eq!(func.parameters[0].name, "a");
    }

    // ===================================================================
    // PARAMETER TESTS
    // ===================================================================

    #[test]
    fn test_parameter_creation() {
        let range = create_range(1, 11, 1, 17);
        let param = Parameter::new(
            "value".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 11, 1, 17),
                "String".to_string(),
            ))),
            Some(Expression::Literal(Literal::new(
                create_range(1, 25, 1, 31),
                serde_json::json!("default"),
                "string".to_string(),
            ))),
            range,
        );

        assert_eq!(param.name, "value");
        assert!(param.type_annotation.is_some());
        assert!(param.default_value.is_some());
    }

    // ===================================================================
    // STRUCT DECLARATION TESTS
    // ===================================================================

    #[test]
    fn test_struct_declaration_creation() {
        let field = StructField::new(
            "id".to_string(),
            TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(2, 9, 2, 15),
                "Number".to_string(),
            )),
            true,
            create_range(2, 5, 2, 15),
        );

        let range = create_range(1, 1, 4, 2);
        let struct_decl = StructDeclaration::new(
            range,
            true,
            "User".to_string(),
            vec![field],
            vec![],
        );

        assert_eq!(struct_decl.name, "User");
        assert!(struct_decl.is_public);
        assert_eq!(struct_decl.fields.len(), 1);
        assert_eq!(struct_decl.fields[0].name, "id");
    }

    // ===================================================================
    // ENUM DECLARATION TESTS
    // ===================================================================

    #[test]
    fn test_enum_declaration_creation() {
        let variant = EnumVariant::new(
            "Active".to_string(),
            None,
            create_range(2, 5, 2, 11),
        );

        let range = create_range(1, 1, 4, 2);
        let enum_decl = EnumDeclaration::new(
            range,
            true,
            "Status".to_string(),
            vec![variant],
            vec![],
        );

        assert_eq!(enum_decl.name, "Status");
        assert!(enum_decl.is_public);
        assert_eq!(enum_decl.variants.len(), 1);
        assert_eq!(enum_decl.variants[0].name, "Active");
    }

    // ===================================================================
    // TYPE ALIAS TESTS
    // ===================================================================

    #[test]
    fn test_type_alias_declaration() {
        let range = create_range(1, 1, 1, 20);
        let type_alias = TypeAliasDeclaration::new(
            range,
            true,
            "UserId".to_string(),
            TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 15, 1, 21),
                "Number".to_string(),
            )),
        );

        assert_eq!(type_alias.name, "UserId");
        assert!(type_alias.is_public);
    }

    // ===================================================================
    // STATEMENTS TESTS
    // ===================================================================

    #[test]
    fn test_block_statement_creation() {
        let range = create_range(1, 1, 3, 2);
        let block = BlockStatement::new(range.clone(), vec![]);
        assert_eq!(block.statements.len(), 0);
        assert_eq!(block.node.range, range);
    }

    #[test]
    fn test_expression_statement_creation() {
        let range = create_range(1, 1, 1, 10);
        let expr = Expression::Literal(Literal::new(
            range.clone(),
            serde_json::json!(42),
            "number".to_string(),
        ));
        let stmt = ExpressionStatement::new(range, expr);
        assert!(matches!(stmt.expression, Expression::Literal(_)));
    }

    #[test]
    fn test_variable_declaration_statement() {
        let range = create_range(1, 1, 1, 15);
        let initializer = Expression::Literal(Literal::new(
            create_range(1, 11, 1, 15),
            serde_json::json!("hello"),
            "string".to_string(),
        ));
        let var_decl = VariableDeclaration::new(
            range,
            "name".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 6, 1, 12),
                "String".to_string(),
            ))),
            Some(initializer),
            false, // not state
        );

        assert_eq!(var_decl.name, "name");
        assert!(!var_decl.is_state);
        assert!(var_decl.initializer.is_some());
    }

    #[test]
    fn test_state_variable_declaration() {
        let range = create_range(1, 1, 1, 10);
        let state_var = VariableDeclaration::new(
            range,
            "count".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 11, 1, 17),
                "Number".to_string(),
            ))),
            None,
            true, // is state
        );

        assert_eq!(state_var.name, "count");
        assert!(state_var.is_state);
        assert!(state_var.initializer.is_none());
    }

    #[test]
    fn test_return_statement() {
        let range = create_range(1, 1, 1, 15);
        let value = Expression::Literal(Literal::new(
            create_range(1, 8, 1, 15),
            serde_json::json!(42),
            "number".to_string(),
        ));
        let return_stmt = ReturnStatement::new(range, Some(value));

        assert!(return_stmt.value.is_some());
    }

    #[test]
    fn test_if_statement() {
        let range = create_range(1, 1, 3, 2);
        let condition = Expression::Literal(Literal::new(
            create_range(1, 5, 1, 9),
            serde_json::json!(true),
            "bool".to_string(),
        ));
        let then_branch = Statement::Block(BlockStatement::new(
            create_range(1, 11, 1, 13),
            vec![],
        ));
        let else_branch = Some(Statement::Block(BlockStatement::new(
            create_range(2, 1, 2, 3),
            vec![],
        )));

        let if_stmt = IfStatement::new(range, condition, then_branch, else_branch);

        assert!(if_stmt.else_branch.is_some());
    }

    // ===================================================================
    // EXPRESSIONS TESTS
    // ===================================================================

    #[test]
    fn test_literal_expression() {
        let range = create_range(1, 1, 1, 4);
        let literal = Literal::new(range, serde_json::json!(42), "number".to_string());
        assert_eq!(literal.kind, "number");
        assert_eq!(literal.value, serde_json::json!(42));
    }

    #[test]
    fn test_identifier_expression() {
        let range = create_range(1, 1, 1, 3);
        let ident = Identifier::new(range.clone(), "var".to_string());
        assert_eq!(ident.name, "var");
        assert_eq!(ident.node.range, range);
    }

    #[test]
    fn test_binary_expression() {
        let range = create_range(1, 1, 1, 5);
        let left = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 1), "a".to_string()));
        let right = Expression::Identifier(Identifier::new(create_range(1, 5, 1, 5), "b".to_string()));
        let binary = BinaryExpression::new(range.clone(), left, "+".to_string(), right);
        assert_eq!(binary.operator, "+");
        assert_eq!(binary.node.range, range);
    }

    #[test]
    fn test_unary_expression() {
        let range = create_range(1, 1, 1, 3);
        let operand = Expression::Identifier(Identifier::new(create_range(1, 2, 1, 3), "x".to_string()));
        let unary = UnaryExpression::new(range, "-".to_string(), operand);
        assert_eq!(unary.operator, "-");
    }

    #[test]
    fn test_call_expression() {
        let range = create_range(1, 1, 1, 8);
        let callee = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 3), "add".to_string()));
        let args = vec![
            Expression::Literal(Literal::new(create_range(1, 5, 1, 5), serde_json::json!(2), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 7, 1, 7), serde_json::json!(3), "number".to_string())),
        ];
        let call = CallExpression::new(range, callee, args);
        assert_eq!(call.arguments.len(), 2);
    }

    #[test]
    fn test_member_access_expression() {
        let range = create_range(1, 1, 1, 6);
        let object = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 4), "user".to_string()));
        let member_access = MemberAccessExpression::new(range, object, "name".to_string(), false);
        assert_eq!(member_access.member, "name");
        assert!(!member_access.is_optional);
    }

    #[test]
    fn test_optional_member_access() {
        let range = create_range(1, 1, 1, 7);
        let object = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 4), "user".to_string()));
        let member_access = MemberAccessExpression::new(range, object, "name".to_string(), true);
        assert_eq!(member_access.member, "name");
        assert!(member_access.is_optional);
    }

    #[test]
    fn test_index_access_expression() {
        let range = create_range(1, 1, 1, 6);
        let object = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 5), "array".to_string()));
        let index = Expression::Literal(Literal::new(create_range(1, 6, 1, 6), serde_json::json!(0), "number".to_string()));
        let index_access = IndexAccessExpression::new(range, object, index);
        assert!(matches!(index_access.index.as_ref(), Expression::Literal(_)));
    }

    #[test]
    fn test_array_literal() {
        let range = create_range(1, 1, 1, 7);
        let elements = vec![
            Expression::Literal(Literal::new(create_range(1, 2, 1, 2), serde_json::json!(1), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 4, 1, 4), serde_json::json!(2), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 6, 1, 6), serde_json::json!(3), "number".to_string())),
        ];
        let array = ArrayLiteral::new(range, elements);
        assert_eq!(array.elements.len(), 3);
    }

    #[test]
    fn test_tuple_literal() {
        let range = create_range(1, 1, 1, 9);
        let elements = vec![
            Expression::Literal(Literal::new(create_range(1, 2, 1, 2), serde_json::json!(1), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 4, 1, 6), serde_json::json!("hello"), "string".to_string())),
        ];
        let tuple = TupleLiteral::new(range, elements);
        assert_eq!(tuple.elements.len(), 2);
    }

    #[test]
    fn test_struct_literal() {
        let range = create_range(1, 1, 1, 25);
        let fields = vec![
            StructLiteralField::new(
                "id".to_string(),
                Expression::Literal(Literal::new(create_range(1, 8, 1, 8), serde_json::json!(1), "number".to_string())),
                create_range(1, 6, 1, 8),
            ),
            StructLiteralField::new(
                "name".to_string(),
                Expression::Literal(Literal::new(create_range(1, 18, 1, 24), serde_json::json!("Alice"), "string".to_string())),
                create_range(1, 11, 1, 24),
            ),
        ];
        let struct_lit = StructLiteral::new(range, "User".to_string(), fields);
        assert_eq!(struct_lit.struct_name, "User");
        assert_eq!(struct_lit.fields.len(), 2);
    }

    #[test]
    fn test_lambda_expression() {
        let range = create_range(1, 1, 1, 12);
        let parameters = vec![
            Parameter::new(
                "x".to_string(),
                Some(TypeAnnotation::Primitive(PrimitiveType::new(
                    create_range(1, 2, 1, 8),
                    "Number".to_string(),
                ))),
                None,
                create_range(1, 2, 1, 8),
            ),
        ];
        let body = LambdaBody::Expression(Box::new(Expression::Binary(BinaryExpression::new(
            create_range(1, 12, 1, 15),
            Expression::Identifier(Identifier::new(create_range(1, 12, 1, 13), "x".to_string())),
            "*".to_string(),
            Expression::Literal(Literal::new(create_range(1, 15, 1, 15), serde_json::json!(2), "number".to_string())),
        ))));

        let lambda = LambdaExpression::new(range, parameters, body);
        assert_eq!(lambda.parameters.len(), 1);
        assert!(matches!(lambda.body, LambdaBody::Expression(_)));
    }

    #[test]
    fn test_if_expression() {
        let range = create_range(1, 1, 1, 20);
        let condition = Expression::Literal(Literal::new(
            create_range(1, 5, 1, 9),
            serde_json::json!(true),
            "bool".to_string(),
        ));
        let then_branch = Expression::Literal(Literal::new(
            create_range(1, 11, 1, 15),
            serde_json::json!("yes"),
            "string".to_string(),
        ));
        let else_branch = Expression::Literal(Literal::new(
            create_range(1, 17, 1, 20),
            serde_json::json!("no"),
            "string".to_string(),
        ));

        let if_expr = IfExpression::new(range, condition, then_branch, else_branch);
        assert!(matches!(if_expr.condition.as_ref(), Expression::Literal(_)));
    }

    #[test]
    fn test_string_interpolation() {
        let range = create_range(1, 1, 1, 15);
        let parts = vec![
            StringInterpolationPart::String("Hello, ".to_string()),
            StringInterpolationPart::Expression(Box::new(Expression::Identifier(
                Identifier::new(create_range(1, 9, 1, 12), "name".to_string()),
            ))),
            StringInterpolationPart::String("!".to_string()),
        ];
        let interpolation = StringInterpolation::new(range, parts);
        assert_eq!(interpolation.parts.len(), 3);
    }

    #[test]
    fn test_await_expression() {
        let range = create_range(1, 1, 1, 12);
        let inner_expr = Expression::Call(CallExpression::new(
            create_range(1, 7, 1, 12),
            Expression::Identifier(Identifier::new(create_range(1, 7, 1, 10), "get".to_string())),
            vec![],
        ));
        let await_expr = AwaitExpression::new(range.clone(), inner_expr);
        assert!(matches!(await_expr.expression.as_ref(), Expression::Call(_)));
        assert_eq!(await_expr.node.range, range);
    }

    // ===================================================================
    // PATTERNS TESTS
    // ===================================================================

    #[test]
    fn test_literal_pattern() {
        let range = create_range(1, 1, 1, 3);
        let pattern = LiteralPattern::new(range, serde_json::json!(42));
        assert_eq!(pattern.value, serde_json::json!(42));
    }

    #[test]
    fn test_identifier_pattern() {
        let range = create_range(1, 1, 1, 3);
        let pattern = IdentifierPattern::new(range, "value".to_string());
        assert_eq!(pattern.name, "value");
    }

    #[test]
    fn test_tuple_pattern() {
        let range = create_range(1, 1, 1, 7);
        let elements = vec![
            Pattern::Identifier(IdentifierPattern::new(create_range(1, 2, 1, 2), "x".to_string())),
            Pattern::Identifier(IdentifierPattern::new(create_range(1, 4, 1, 4), "y".to_string())),
        ];
        let tuple_pattern = TuplePattern::new(range, elements);
        assert_eq!(tuple_pattern.elements.len(), 2);
    }

    #[test]
    fn test_struct_pattern() {
        let range = create_range(1, 1, 1, 12);
        let fields = vec![
            StructPatternField::new(
                "id".to_string(),
                Pattern::Identifier(IdentifierPattern::new(create_range(1, 6, 1, 6), "id".to_string())),
                create_range(1, 6, 1, 6),
            ),
        ];
        let struct_pattern = StructPattern::new(range, "User".to_string(), fields);
        assert_eq!(struct_pattern.struct_name, "User");
        assert_eq!(struct_pattern.fields.len(), 1);
    }

    #[test]
    fn test_enum_pattern() {
        let range = create_range(1, 1, 1, 8);
        let inner_patterns = Some(vec![
            Pattern::Identifier(IdentifierPattern::new(create_range(1, 6, 1, 7), "x".to_string())),
        ]);
        let enum_pattern = EnumPattern::new(range, "Some".to_string(), inner_patterns);
        assert_eq!(enum_pattern.variant_name, "Some");
        assert!(enum_pattern.inner_patterns.is_some());
    }

    #[test]
    fn test_wildcard_pattern() {
        let range = create_range(1, 1, 1, 1);
        let wildcard = WildcardPattern::new(range.clone());
        // Wildcard pattern has no additional fields to test
        assert_eq!(wildcard.node.range, range);
    }

    // ===================================================================
    // TYPE ANNOTATIONS TESTS
    // ===================================================================

    #[test]
    fn test_primitive_type() {
        let range = create_range(1, 1, 1, 6);
        let primitive = PrimitiveType::new(range, "Number".to_string());
        assert_eq!(primitive.name, "Number");
    }

    #[test]
    fn test_array_type() {
        let range = create_range(1, 1, 1, 8);
        let element_type = TypeAnnotation::Primitive(PrimitiveType::new(
            create_range(1, 1, 1, 6),
            "String".to_string(),
        ));
        let array_type = ArrayType::new(range.clone(), element_type);
        assert!(matches!(array_type.element_type.as_ref(), TypeAnnotation::Primitive(_)));
        assert_eq!(array_type.node.range, range);
    }

    #[test]
    fn test_tuple_type() {
        let range = create_range(1, 1, 1, 15);
        let element_types = vec![
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 2, 1, 8), "Number".to_string())),
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 10, 1, 16), "String".to_string())),
        ];
        let tuple_type = TupleType::new(range, element_types);
        assert_eq!(tuple_type.element_types.len(), 2);
    }

    #[test]
    fn test_function_type() {
        let range = create_range(1, 1, 1, 18);
        let parameter_types = vec![
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 2, 1, 8), "Number".to_string())),
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 10, 1, 16), "Number".to_string())),
        ];
        let return_type = TypeAnnotation::Primitive(PrimitiveType::new(
            create_range(1, 20, 1, 26),
            "Number".to_string(),
        ));
        let function_type = FunctionType::new(range, parameter_types, return_type);
        assert_eq!(function_type.parameter_types.len(), 2);
    }

    #[test]
    fn test_generic_type() {
        let range = create_range(1, 1, 1, 12);
        let type_args = vec![
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 7, 1, 13), "String".to_string())),
        ];
        let generic_type = GenericType::new(range, "Option".to_string(), type_args);
        assert_eq!(generic_type.base_name, "Option");
        assert_eq!(generic_type.type_arguments.len(), 1);
    }

    #[test]
    fn test_union_type() {
        let range = create_range(1, 1, 1, 15);
        let types = vec![
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 1, 1, 7), "active".to_string())),
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 10, 1, 15), "inactive".to_string())),
        ];
        let union_type = UnionType::new(range, types);
        assert_eq!(union_type.types.len(), 2);
    }

    #[test]
    fn test_named_type() {
        let range = create_range(1, 1, 1, 4);
        let named_type = NamedType::new(range, "User".to_string());
        assert_eq!(named_type.name, "User");
    }

    #[test]
    fn test_optional_type() {
        let range = create_range(1, 1, 1, 7);
        let inner_type = TypeAnnotation::Primitive(PrimitiveType::new(
            create_range(1, 1, 1, 6),
            "String".to_string(),
        ));
        let optional_type = OptionalType::new(range.clone(), inner_type);
        assert!(matches!(optional_type.inner_type.as_ref(), TypeAnnotation::Primitive(_)));
        assert_eq!(optional_type.node.range, range);
    }

    // ===================================================================
    // UTILITY FUNCTIONS TESTS
    // ===================================================================

    #[test]
    fn test_is_expression_statement_valid() {
        // Valid expressions for statements
        let call_expr = Expression::Call(CallExpression::new(
            create_range(1, 1, 1, 6),
            Expression::Identifier(Identifier::new(create_range(1, 1, 1, 4), "func".to_string())),
            vec![],
        ));
        assert!(is_expression_statement_valid(&call_expr));

        // Invalid expressions for statements
        let binary_expr = Expression::Binary(BinaryExpression::new(
            create_range(1, 1, 1, 5),
            Expression::Literal(Literal::new(create_range(1, 1, 1, 1), serde_json::json!(1), "number".to_string())),
            "+".to_string(),
            Expression::Literal(Literal::new(create_range(1, 5, 1, 5), serde_json::json!(2), "number".to_string())),
        ));
        assert!(!is_expression_statement_valid(&binary_expr));
    }

    // ===================================================================
    // COMPLEX AST CONSTRUCTION TESTS
    // ===================================================================

    #[test]
    fn test_complete_function_ast() {
        // Create a complete function AST similar to the Python example
        // fn add(a: Number, b: Number) -> Number { return a + b }

        let func_range = create_range(1, 1, 1, 50);

        // Parameters
        let param_a = Parameter::new(
            "a".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 11, 1, 17),
                "Number".to_string(),
            ))),
            None,
            create_range(1, 9, 1, 17),
        );

        let param_b = Parameter::new(
            "b".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 19, 1, 25),
                "Number".to_string(),
            ))),
            None,
            create_range(1, 19, 1, 25),
        );

        // Return statement: return a + b
        let return_stmt = ReturnStatement::new(
            create_range(1, 40, 1, 48),
            Some(Expression::Binary(BinaryExpression::new(
                create_range(1, 47, 1, 48),
                Expression::Identifier(Identifier::new(create_range(1, 47, 1, 47), "a".to_string())),
                "+".to_string(),
                Expression::Identifier(Identifier::new(create_range(1, 49, 1, 49), "b".to_string())),
            ))),
        );

        // Function body
        let body = BlockStatement::new(
            create_range(1, 38, 1, 50),
            vec![Statement::Return(return_stmt)],
        );

        // Complete function
        let function = FunctionDeclaration::new(
            func_range,
            true,
            "add".to_string(),
            vec![param_a, param_b],
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 30, 1, 36),
                "Number".to_string(),
            ))),
            body,
            false,
            vec![],
        );

        // Verify the structure
        assert_eq!(function.name, "add");
        assert!(function.is_public);
        assert_eq!(function.parameters.len(), 2);
        assert_eq!(function.parameters[0].name, "a");
        assert_eq!(function.parameters[1].name, "b");

        if let Statement::Return(ref return_stmt) = &function.body.statements[0] {
            assert!(return_stmt.value.is_some());
            if let Expression::Binary(ref binary) = return_stmt.value.as_ref().unwrap() {
                assert_eq!(binary.operator, "+");
            } else {
                panic!("Expected binary expression in return statement");
            }
        } else {
            panic!("Expected return statement in function body");
        }
    }

    #[test]
    fn test_program_with_multiple_declarations() {
        // Create a program with multiple declarations
        let program_range = create_range(1, 1, 20, 1);

        // Import declaration
        let import = ImportDeclaration::new(
            create_range(1, 1, 1, 20),
            ImportKind::Package,
            "std".to_string(),
            None,
            None,
            None,
        );

        // Type alias
        let type_alias = TypeAliasDeclaration::new(
            create_range(3, 1, 3, 15),
            true,
            "UserId".to_string(),
            TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(3, 12, 3, 18),
                "Number".to_string(),
            )),
        );

        // Struct declaration
        let struct_decl = StructDeclaration::new(
            create_range(5, 1, 8, 2),
            true,
            "User".to_string(),
            vec![
                StructField::new(
                    "id".to_string(),
                    TypeAnnotation::Named(NamedType::new(create_range(6, 9, 6, 14), "UserId".to_string())),
                    true,
                    create_range(6, 5, 6, 14),
                ),
                StructField::new(
                    "name".to_string(),
                    TypeAnnotation::Primitive(PrimitiveType::new(create_range(7, 10, 7, 16), "String".to_string())),
                    true,
                    create_range(7, 5, 7, 16),
                ),
            ],
            vec![],
        );

        // Function declaration
        let func_body = BlockStatement::new(create_range(12, 20, 12, 22), vec![]);
        let func = FunctionDeclaration::new(
            create_range(10, 1, 12, 22),
            true,
            "createUser".to_string(),
            vec![
                Parameter::new(
                    "id".to_string(),
                    Some(TypeAnnotation::Named(NamedType::new(create_range(10, 18, 10, 23), "UserId".to_string()))),
                    None,
                    create_range(10, 16, 10, 23),
                ),
                Parameter::new(
                    "name".to_string(),
                    Some(TypeAnnotation::Primitive(PrimitiveType::new(create_range(10, 30, 10, 36), "String".to_string()))),
                    None,
                    create_range(10, 25, 10, 36),
                ),
            ],
            Some(TypeAnnotation::Named(NamedType::new(create_range(10, 41, 10, 45), "User".to_string()))),
            func_body,
            false,
            vec![],
        );

        let program = Program::new(
            program_range,
            vec![import],
            vec![
                Declaration::TypeAlias(type_alias),
                Declaration::Struct(struct_decl),
                Declaration::Function(func),
            ],
        );

        assert_eq!(program.imports.len(), 1);
        assert_eq!(program.declarations.len(), 3);
    }
}