/*
Tests unitarios para el módulo AST

Jira: TASK-RUST-102
Historia: US-RUST-02 (Compiler Foundation)
Fecha: 2025-12-01

Cobertura: >= 80%
Tests incluidos:
- Creación de nodos básicos (Position, Range, ASTNode)
- Declaraciones (Function, Struct, Enum, etc.)
- Statements (Block, Expression, Variable, etc.)
- Expressions (Literal, Binary, Call, etc.)
- Patterns (Literal, Identifier, Struct, etc.)
- Type Annotations (Primitive, Array, Function, etc.)
- Visitor pattern básico
- Utility functions
- Validaciones de sintaxis
*/

use vela_compiler::ast::*;
use serde_json;

#[cfg(test)]
mod tests {
    use super::*;

    // ===================================================================
    // BASE NODE TESTS
    // ===================================================================

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(3, 7);
        assert_eq!(format!("{}", pos), "3:7");
    }

    #[test]
    fn test_range_creation() {
        let start = Position::new(1, 1);
        let end = Position::new(1, 10);
        let range = Range::new(start, end);
        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.column, 1);
        assert_eq!(range.end.line, 1);
        assert_eq!(range.end.column, 10);
    }

    #[test]
    fn test_range_display() {
        let start = Position::new(2, 5);
        let end = Position::new(2, 15);
        let range = Range::new(start, end);
        assert_eq!(format!("{}", range), "2:5 - 2:15");
    }

    #[test]
    fn test_ast_node_creation() {
        let range = create_range(1, 1, 1, 5);
        let node = ASTNode::new(range);
        assert_eq!(node.range.start.line, 1);
        assert_eq!(node.range.start.column, 1);
        assert_eq!(node.range.end.line, 1);
        assert_eq!(node.range.end.column, 5);
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
        assert_eq!(program.node.range.start.line, 1);
    }

    #[test]
    fn test_program_with_imports_and_declarations() {
        let range = create_range(1, 1, 20, 1);

        // Crear import
        let import_range = create_range(1, 1, 1, 25);
        let import = ImportDeclaration::new(
            import_range,
            ImportKind::Package,
            "http".to_string(),
            None,
            None,
            None,
        );

        // Crear declaración de función
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
        let range = create_range(1, 1, 1, 30);
        let import = ImportDeclaration::new(
            range,
            ImportKind::System,
            "io".to_string(),
            Some("IO".to_string()),
            None,
            None,
        );

        assert_eq!(import.kind, ImportKind::System);
        assert_eq!(import.path, "io");
        assert_eq!(import.alias, Some("IO".to_string()));
        assert!(import.show.is_none());
        assert!(import.hide.is_none());
    }

    #[test]
    fn test_import_with_show_and_hide() {
        let range = create_range(1, 1, 1, 50);
        let import = ImportDeclaration::new(
            range,
            ImportKind::Library,
            "utils".to_string(),
            None,
            Some(vec!["sort".to_string(), "filter".to_string()]),
            Some(vec!["deprecated".to_string()]),
        );

        assert_eq!(import.kind, ImportKind::Library);
        assert_eq!(import.show.as_ref().unwrap().len(), 2);
        assert_eq!(import.hide.as_ref().unwrap().len(), 1);
    }

    // ===================================================================
    // DECLARATION TESTS
    // ===================================================================

    #[test]
    fn test_function_declaration_creation() {
        let range = create_range(1, 1, 5, 2);
        let body = BlockStatement::new(create_range(4, 1, 5, 2), vec![]);

        let param_range = create_range(1, 10, 1, 15);
        let param = Parameter::from_name(
            "x".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                param_range,
                "Number".to_string(),
            ))),
            None,
            param_range,
        );

        let func = FunctionDeclaration::new(
            range,
            true,
            "add".to_string(),
            vec![param],
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
        assert_eq!(func.parameters.len(), 1);
        assert!(func.return_type.is_some());
        assert!(func.generic_params.is_empty());
    }

    #[test]
    fn test_async_function_declaration() {
        let range = create_range(1, 1, 5, 2);
        let body = BlockStatement::new(create_range(4, 1, 5, 2), vec![]);

        let func = FunctionDeclaration::new(
            range,
            false,
            "fetchData".to_string(),
            vec![],
            Some(TypeAnnotation::Generic(GenericType::new(
                create_range(1, 20, 1, 35),
                "Result".to_string(),
                vec![TypeAnnotation::Primitive(PrimitiveType::new(
                    create_range(1, 27, 1, 33),
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
    fn test_struct_declaration_creation() {
        let range = create_range(1, 1, 5, 2);

        let field_range = create_range(2, 5, 2, 15);
        let field = StructField::new(
            "name".to_string(),
            TypeAnnotation::Primitive(PrimitiveType::new(
                field_range,
                "String".to_string(),
            )),
            true,
            field_range,
        );

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
        assert!(struct_decl.generic_params.is_empty());
    }

    #[test]
    fn test_enum_declaration_creation() {
        let range = create_range(1, 1, 8, 2);

        let variant1 = EnumVariant::new("Red".to_string(), None, create_range(2, 5, 2, 8));
        let variant2 = EnumVariant::new(
            "Custom".to_string(),
            Some(vec![
                StructField::new(
                    "r".to_string(),
                    TypeAnnotation::Primitive(PrimitiveType::new(
                        create_range(4, 10, 4, 16),
                        "Number".to_string(),
                    )),
                    false,
                    create_range(4, 10, 4, 16),
                ),
                StructField::new(
                    "g".to_string(),
                    TypeAnnotation::Primitive(PrimitiveType::new(
                        create_range(4, 18, 4, 24),
                        "Number".to_string(),
                    )),
                    false,
                    create_range(4, 18, 4, 24),
                ),
            ]),
            create_range(4, 5, 4, 25),
        );

        let enum_decl = EnumDeclaration::new(
            range,
            true,
            "Color".to_string(),
            vec![variant1, variant2],
            vec![],
        );

        assert_eq!(enum_decl.name, "Color");
        assert!(enum_decl.is_public);
        assert_eq!(enum_decl.variants.len(), 2);
        assert!(enum_decl.variants[0].fields.is_none());
        assert!(enum_decl.variants[1].fields.is_some());
    }

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
    // STATEMENT TESTS
    // ===================================================================

    #[test]
    fn test_block_statement_creation() {
        let range = create_range(1, 1, 5, 2);
        let block = BlockStatement::new(range, vec![]);
        assert_eq!(block.statements.len(), 0);
        assert_eq!(block.node.range, range);
    }

    #[test]
    fn test_expression_statement_creation() {
        let range = create_range(1, 1, 1, 10);
        let expr = Expression::Literal(Literal::new(
            range,
            serde_json::json!(42),
            "number".to_string(),
        ));
        let stmt = ExpressionStatement::new(range, expr);

        assert_eq!(stmt.node.range, range);
        match &stmt.expression {
            Expression::Literal(lit) => assert_eq!(lit.value, serde_json::json!(42)),
            _ => panic!("Expected literal expression"),
        }
    }

    #[test]
    fn test_variable_declaration() {
        let range = create_range(1, 1, 1, 15);
        let initializer = Expression::Literal(Literal::new(
            create_range(1, 10, 1, 15),
            serde_json::json!("hello"),
            "string".to_string(),
        ));

        let var_decl = VariableDeclaration::new(
            range,
            "message".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 5, 1, 11),
                "String".to_string(),
            ))),
            Some(initializer),
            false, // not state
        );

        assert_eq!(var_decl.name, "message");
        assert!(!var_decl.is_state);
        assert!(var_decl.type_annotation.is_some());
        assert!(var_decl.initializer.is_some());
    }

    #[test]
    fn test_state_variable_declaration() {
        let range = create_range(1, 1, 1, 20);
        let state_var = VariableDeclaration::new(
            range,
            "count".to_string(),
            Some(TypeAnnotation::Primitive(PrimitiveType::new(
                create_range(1, 10, 1, 16),
                "Number".to_string(),
            ))),
            Some(Expression::Literal(Literal::new(
                create_range(1, 19, 1, 20),
                serde_json::json!(0),
                "number".to_string(),
            ))),
            true, // is state
        );

        assert_eq!(state_var.name, "count");
        assert!(state_var.is_state);
    }

    #[test]
    fn test_assignment_statement() {
        let range = create_range(1, 1, 1, 10);
        let target = Expression::Identifier(Identifier::new(
            create_range(1, 1, 1, 5),
            "count".to_string(),
        ));
        let value = Expression::Literal(Literal::new(
            create_range(1, 9, 1, 10),
            serde_json::json!(5),
            "number".to_string(),
        ));

        let assignment = AssignmentStatement::new(range, target, value);
        assert_eq!(assignment.node.range, range);
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
        let range = create_range(1, 1, 5, 2);
        let condition = Expression::Identifier(Identifier::new(
            create_range(1, 5, 1, 9),
            "flag".to_string(),
        ));
        let then_branch = Statement::Block(BlockStatement::new(
            create_range(2, 1, 3, 2),
            vec![],
        ));

        let if_stmt = IfStatement::new(range, condition, then_branch, None);
        assert!(if_stmt.else_branch.is_none());
    }

    #[test]
    fn test_if_else_statement() {
        let range = create_range(1, 1, 7, 2);
        let condition = Expression::Binary(BinaryExpression::new(
            create_range(1, 5, 1, 12),
            Expression::Identifier(Identifier::new(create_range(1, 5, 1, 6), "x".to_string())),
            ">".to_string(),
            Expression::Literal(Literal::new(create_range(1, 10, 1, 12), serde_json::json!(0), "number".to_string())),
        ));
        let then_branch = Statement::Return(ReturnStatement::new(
            create_range(2, 5, 2, 12),
            Some(Expression::Literal(Literal::new(create_range(2, 12, 2, 12), serde_json::json!(1), "number".to_string()))),
        ));
        let else_branch = Statement::Return(ReturnStatement::new(
            create_range(4, 5, 4, 12),
            Some(Expression::Literal(Literal::new(create_range(4, 12, 4, 12), serde_json::json!(0), "number".to_string()))),
        ));

        let if_stmt = IfStatement::new(range, condition, then_branch, Some(else_branch));
        assert!(if_stmt.else_branch.is_some());
    }

    // ===================================================================
    // EXPRESSION TESTS
    // ===================================================================

    #[test]
    fn test_literal_expression() {
        let range = create_range(1, 1, 1, 4);
        let literal = Literal::new(range, serde_json::json!(42), "number".to_string());
        assert_eq!(literal.kind, "number");
        assert_eq!(literal.value, serde_json::json!(42));
    }

    #[test]
    fn test_none_literal() {
        let range = create_range(1, 1, 1, 4);
        let none_literal = Literal::new(range, serde_json::json!(null), "none".to_string());
        assert_eq!(none_literal.kind, "none");
        assert_eq!(none_literal.value, serde_json::json!(null));
    }

    #[test]
    fn test_identifier_expression() {
        let range = create_range(1, 1, 1, 5);
        let ident = Identifier::new(range, "variable".to_string());
        assert_eq!(ident.name, "variable");
        assert_eq!(ident.node.range, range);
    }

    #[test]
    fn test_binary_expression() {
        let range = create_range(1, 1, 1, 5);
        let left = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 1), "a".to_string()));
        let right = Expression::Identifier(Identifier::new(create_range(1, 5, 1, 5), "b".to_string()));
        let binary = BinaryExpression::new(range, left, "+".to_string(), right);

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
        let range = create_range(1, 1, 1, 10);
        let callee = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 3), "add".to_string()));
        let args = vec![
            Expression::Literal(Literal::new(create_range(1, 5, 1, 5), serde_json::json!(2), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 8, 1, 8), serde_json::json!(3), "number".to_string())),
        ];

        let call = CallExpression::new(range, callee, args);
        assert_eq!(call.arguments.len(), 2);
    }

    #[test]
    fn test_member_access_expression() {
        let range = create_range(1, 1, 1, 8);
        let object = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 4), "user".to_string()));
        let member_access = MemberAccessExpression::new(range, object, "name".to_string(), false);

        assert_eq!(member_access.member, "name");
        assert!(!member_access.is_optional);
    }

    #[test]
    fn test_optional_member_access() {
        let range = create_range(1, 1, 1, 12);
        let object = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 4), "user".to_string()));
        let optional_access = MemberAccessExpression::new(range, object, "address".to_string(), true);

        assert_eq!(optional_access.member, "address");
        assert!(optional_access.is_optional);
    }

    #[test]
    fn test_index_access_expression() {
        let range = create_range(1, 1, 1, 8);
        let object = Expression::Identifier(Identifier::new(create_range(1, 1, 1, 5), "array".to_string()));
        let index = Expression::Literal(Literal::new(create_range(1, 7, 1, 7), serde_json::json!(0), "number".to_string()));
        let index_access = IndexAccessExpression::new(range, object, index);

        assert_eq!(index_access.node.range, range);
    }

    #[test]
    fn test_array_literal() {
        let range = create_range(1, 1, 1, 10);
        let elements = vec![
            Expression::Literal(Literal::new(create_range(1, 2, 1, 2), serde_json::json!(1), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 5, 1, 5), serde_json::json!(2), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 8, 1, 8), serde_json::json!(3), "number".to_string())),
        ];

        let array = ArrayLiteral::new(range, elements);
        assert_eq!(array.elements.len(), 3);
    }

    #[test]
    fn test_tuple_literal() {
        let range = create_range(1, 1, 1, 12);
        let elements = vec![
            Expression::Literal(Literal::new(create_range(1, 2, 1, 2), serde_json::json!(1), "number".to_string())),
            Expression::Literal(Literal::new(create_range(1, 5, 1, 9), serde_json::json!("hello"), "string".to_string())),
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
                create_range(1, 5, 1, 9),
            ),
            StructLiteralField::new(
                "name".to_string(),
                Expression::Literal(Literal::new(create_range(1, 18, 1, 24), serde_json::json!("Alice"), "string".to_string())),
                create_range(1, 12, 1, 25),
            ),
        ];

        let struct_lit = StructLiteral::new(range, "User".to_string(), fields);
        assert_eq!(struct_lit.struct_name, "User");
        assert_eq!(struct_lit.fields.len(), 2);
    }

    #[test]
    fn test_lambda_expression() {
        let range = create_range(1, 1, 1, 15);
        let parameters = vec![
            Parameter::from_name(
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
    }

    #[test]
    fn test_if_expression() {
        let range = create_range(1, 1, 1, 25);
        let condition = Expression::Identifier(Identifier::new(create_range(1, 5, 1, 9), "flag".to_string()));
        let then_branch = Expression::Literal(Literal::new(create_range(1, 13, 1, 17), serde_json::json!("yes"), "string".to_string()));
        let else_branch = Expression::Literal(Literal::new(create_range(1, 21, 1, 25), serde_json::json!("no"), "string".to_string()));

        let if_expr = IfExpression::new(range, condition, then_branch, else_branch);
        assert_eq!(if_expr.node.range, range);
    }

    #[test]
    fn test_await_expression() {
        let range = create_range(1, 1, 1, 15);
        let inner_expr = Expression::Call(CallExpression::new(
            create_range(1, 7, 1, 15),
            Expression::Identifier(Identifier::new(create_range(1, 7, 1, 12), "fetch".to_string())),
            vec![],
        ));

        let await_expr = AwaitExpression::new(range, inner_expr);
        assert_eq!(await_expr.node.range, range);
    }

    // ===================================================================
    // PATTERN TESTS
    // ===================================================================

    #[test]
    fn test_literal_pattern() {
        let range = create_range(1, 1, 1, 3);
        let pattern = LiteralPattern::new(range, serde_json::json!(42));
        assert_eq!(pattern.value, serde_json::json!(42));
    }

    #[test]
    fn test_identifier_pattern() {
        let range = create_range(1, 1, 1, 5);
        let pattern = IdentifierPattern::new(range, "value".to_string());
        assert_eq!(pattern.name, "value");
    }

    #[test]
    fn test_tuple_pattern() {
        let range = create_range(1, 1, 1, 8);
        let elements = vec![
            Pattern::Identifier(IdentifierPattern::new(create_range(1, 2, 1, 2), "x".to_string())),
            Pattern::Identifier(IdentifierPattern::new(create_range(1, 5, 1, 5), "y".to_string())),
        ];

        let tuple_pattern = TuplePattern::new(range, elements);
        assert_eq!(tuple_pattern.elements.len(), 2);
    }

    #[test]
    fn test_struct_pattern() {
        let range = create_range(1, 1, 1, 15);
        let fields = vec![
            StructPatternField::new(
                "id".to_string(),
                Pattern::Identifier(IdentifierPattern::new(create_range(1, 8, 1, 8), "id".to_string())),
                create_range(1, 5, 1, 9),
            ),
            StructPatternField::new(
                "name".to_string(),
                Pattern::Identifier(IdentifierPattern::new(create_range(1, 14, 1, 14), "name".to_string())),
                create_range(1, 12, 1, 15),
            ),
        ];

        let struct_pattern = StructPattern::new(range, "User".to_string(), fields);
        assert_eq!(struct_pattern.struct_name, "User");
        assert_eq!(struct_pattern.fields.len(), 2);
    }

    #[test]
    fn test_enum_pattern() {
        let range = create_range(1, 1, 1, 10);
        let inner_patterns = Some(vec![
            Pattern::Identifier(IdentifierPattern::new(create_range(1, 6, 1, 6), "value".to_string())),
        ]);

        let enum_pattern = EnumPattern::new(range, "Some".to_string(), inner_patterns);
        assert_eq!(enum_pattern.variant_name, "Some");
        assert!(enum_pattern.inner_patterns.is_some());
        assert_eq!(enum_pattern.inner_patterns.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_wildcard_pattern() {
        let range = create_range(1, 1, 1, 1);
        let wildcard = WildcardPattern::new(range);
        assert_eq!(wildcard.node.range, range);
    }

    // ===================================================================
    // TYPE ANNOTATION TESTS
    // ===================================================================

    #[test]
    fn test_primitive_type() {
        let range = create_range(1, 1, 1, 6);
        let primitive = PrimitiveType::new(range, "Number".to_string());
        assert_eq!(primitive.name, "Number");
    }

    #[test]
    fn test_array_type() {
        let range = create_range(1, 1, 1, 10);
        let element_type = TypeAnnotation::Primitive(PrimitiveType::new(
            create_range(1, 6, 1, 10),
            "String".to_string(),
        ));
        let array_type = ArrayType::new(range, element_type);
        assert_eq!(array_type.node.range, range);
    }

    #[test]
    fn test_tuple_type() {
        let range = create_range(1, 1, 1, 15);
        let element_types = vec![
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 2, 1, 8), "Number".to_string())),
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 10, 1, 15), "String".to_string())),
        ];
        let tuple_type = TupleType::new(range, element_types);
        assert_eq!(tuple_type.element_types.len(), 2);
    }

    #[test]
    fn test_function_type() {
        let range = create_range(1, 1, 1, 20);
        let param_types = vec![
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 2, 1, 8), "Number".to_string())),
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 10, 1, 16), "Number".to_string())),
        ];
        let return_type = TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 18, 1, 24), "Number".to_string()));
        let func_type = FunctionType::new(range, param_types, return_type);
        assert_eq!(func_type.parameter_types.len(), 2);
    }

    #[test]
    fn test_generic_type() {
        let range = create_range(1, 1, 1, 15);
        let type_args = vec![
            TypeAnnotation::Primitive(PrimitiveType::new(create_range(1, 8, 1, 14), "String".to_string())),
        ];
        let generic_type = GenericType::new(range, "Option".to_string(), type_args);
        assert_eq!(generic_type.base_name, "Option");
        assert_eq!(generic_type.type_arguments.len(), 1);
    }

    #[test]
    fn test_union_type() {
        let range = create_range(1, 1, 1, 20);
        let types = vec![
            TypeAnnotation::Named(NamedType::new(create_range(1, 1, 1, 6), "active".to_string())),
            TypeAnnotation::Named(NamedType::new(create_range(1, 10, 1, 20), "inactive".to_string())),
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
        let range = create_range(1, 1, 1, 8);
        let inner_type = TypeAnnotation::Primitive(PrimitiveType::new(
            create_range(1, 1, 1, 6),
            "String".to_string(),
        ));
        let optional_type = OptionalType::new(range, inner_type);
        assert_eq!(optional_type.node.range, range);
    }

    // ===================================================================
    // UTILITY FUNCTIONS TESTS
    // ===================================================================

    #[test]
    fn test_create_position_utility() {
        let pos = create_position(10, 20);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 20);
    }

    #[test]
    fn test_create_range_utility() {
        let range = create_range(1, 5, 2, 10);
        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.column, 5);
        assert_eq!(range.end.line, 2);
        assert_eq!(range.end.column, 10);
    }

    #[test]
    fn test_is_expression_statement_valid() {
        // Valid: Call expression
        let call_expr = Expression::Call(CallExpression::new(
            create_range(1, 1, 1, 8),
            Expression::Identifier(Identifier::new(create_range(1, 1, 1, 6), "print".to_string())),
            vec![],
        ));
        assert!(is_expression_statement_valid(&call_expr));

        // Valid: Binary expression (assignment-like)
        let binary_expr = Expression::Binary(BinaryExpression::new(
            create_range(1, 1, 1, 5),
            Expression::Identifier(Identifier::new(create_range(1, 1, 1, 1), "a".to_string())),
            "=".to_string(),
            Expression::Literal(Literal::new(create_range(1, 5, 1, 5), serde_json::json!(5), "number".to_string())),
        ));
        assert!(is_expression_statement_valid(&binary_expr));

        // Invalid: Just a literal
        let literal_expr = Expression::Literal(Literal::new(
            create_range(1, 1, 1, 3),
            serde_json::json!(42),
            "number".to_string(),
        ));
        assert!(!is_expression_statement_valid(&literal_expr));
    }

    // ===================================================================
    // EVENT SYSTEM TESTS
    // ===================================================================

    #[test]
    fn test_event_on_statement() {
        let range = create_range(1, 1, 1, 25);
        let event_type = Expression::Literal(Literal::new(
            create_range(1, 5, 1, 18),
            serde_json::json!("user.created"),
            "string".to_string(),
        ));
        let handler = Expression::Identifier(Identifier::new(
            create_range(1, 20, 1, 25),
            "callback".to_string(),
        ));

        let event_on = EventOnStatement::new(range, event_type, handler, None);
        assert!(event_on.type_param.is_none());
    }

    #[test]
    fn test_event_emit_statement() {
        let range = create_range(1, 1, 1, 20);
        let event_type = Expression::Literal(Literal::new(
            create_range(1, 7, 1, 20),
            serde_json::json!("user.updated"),
            "string".to_string(),
        ));
        let payload = Some(Expression::StructLiteral(StructLiteral::new(
            create_range(1, 22, 1, 35),
            "User".to_string(),
            vec![],
        )));

        let event_emit = EventEmitStatement::new(range, event_type, payload);
        assert!(event_emit.payload.is_some());
    }

    #[test]
    fn test_event_off_statement() {
        let range = create_range(1, 1, 1, 25);
        let event_type = Expression::Literal(Literal::new(
            create_range(1, 5, 1, 18),
            serde_json::json!("user.created"),
            "string".to_string(),
        ));
        let handler = Some(Expression::Identifier(Identifier::new(
            create_range(1, 20, 1, 25),
            "callback".to_string(),
        )));

        let event_off = EventOffStatement::new(range, event_type, handler);
        assert!(event_off.handler.is_some());
    }

    // ===================================================================
    // DISPATCH TESTS
    // ===================================================================

    #[test]
    fn test_dispatch_statement() {
        let range = create_range(1, 1, 1, 15);
        let action = Expression::Call(CallExpression::new(
            create_range(1, 9, 1, 15),
            Expression::Identifier(Identifier::new(create_range(1, 9, 1, 14), "INCREMENT".to_string())),
            vec![],
        ));

        let dispatch = DispatchStatement::new(range, action);
        assert_eq!(dispatch.node.range, range);
    }

    // ===================================================================
    // DECORATOR TESTS
    // ===================================================================

    #[test]
    fn test_decorator_creation() {
        let range = create_range(1, 1, 1, 15);
        let args = vec![
            Expression::Literal(Literal::new(create_range(1, 10, 1, 15), serde_json::json!("/api"), "string".to_string())),
        ];
        let decorator = Decorator::new("get".to_string(), args, range);
        assert_eq!(decorator.name, "get");
        assert_eq!(decorator.arguments.len(), 1);
    }

    // ===================================================================
    // MODULE TESTS
    // ===================================================================

    #[test]
    fn test_module_declaration() {
        let range = create_range(1, 1, 10, 2);
        let decorators = vec![
            Decorator::new(
                "module".to_string(),
                vec![Expression::ObjectLiteral(ObjectLiteral::new(
                    create_range(1, 10, 1, 50),
                    vec![
                        ObjectLiteralField::new(
                            "declarations".to_string(),
                            Expression::ArrayLiteral(ArrayLiteral::new(
                                create_range(1, 25, 1, 35),
                                vec![],
                            )),
                            create_range(1, 15, 1, 36),
                        ),
                    ],
                ))],
                create_range(1, 1, 1, 51),
            ),
        ];
        let body = vec![];

        let module = ModuleDeclaration::new(
            range,
            true,
            "AuthModule".to_string(),
            decorators,
            body,
            vec![],
            vec![],
            vec![],
            vec![],
        );

        assert_eq!(module.name, "AuthModule");
        assert!(module.is_public);
        assert_eq!(module.decorators.len(), 1);
    }

    // ===================================================================
    // SERIALIZATION TESTS
    // ===================================================================

    #[test]
    fn test_ast_node_serialization() {
        let range = create_range(1, 1, 1, 5);
        let node = ASTNode::new(range);

        let serialized = serde_json::to_string(&node).unwrap();
        let deserialized: ASTNode = serde_json::from_str(&serialized).unwrap();

        assert_eq!(node.range.start.line, deserialized.range.start.line);
        assert_eq!(node.range.start.column, deserialized.range.start.column);
        assert_eq!(node.range.end.line, deserialized.range.end.line);
        assert_eq!(node.range.end.column, deserialized.range.end.column);
    }

    #[test]
    fn test_literal_serialization() {
        let range = create_range(1, 1, 1, 4);
        let literal = Literal::new(range, serde_json::json!(42), "number".to_string());

        let serialized = serde_json::to_string(&literal).unwrap();
        let deserialized: Literal = serde_json::from_str(&serialized).unwrap();

        assert_eq!(literal.value, deserialized.value);
        assert_eq!(literal.kind, deserialized.kind);
    }

    #[test]
    fn test_function_declaration_serialization() {
        let range = create_range(1, 1, 5, 2);
        let body = BlockStatement::new(create_range(4, 1, 5, 2), vec![]);
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

        let serialized = serde_json::to_string(&func).unwrap();
        let deserialized: FunctionDeclaration = serde_json::from_str(&serialized).unwrap();

        assert_eq!(func.name, deserialized.name);
        assert_eq!(func.is_public, deserialized.is_public);
        assert_eq!(func.is_async, deserialized.is_async);
    }
}