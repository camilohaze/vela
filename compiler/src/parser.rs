use crate::ast::*;
use crate::error::{CompileError, CompileResult, ParseError, SourceLocation};
use crate::lexer::{Token, TokenKind};

/// Parser para el lenguaje Vela.
/// Convierte tokens del lexer en un AST (Abstract Syntax Tree).
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// Crea un nuevo parser con los tokens dados.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parsea todos los tokens y retorna un programa completo.
    pub fn parse(&mut self) -> CompileResult<Program> {
        let mut imports = Vec::new();
        let mut declarations = Vec::new();

        // Parsear imports primero
        while self.check(TokenKind::Import) {
            imports.push(self.parse_import()?);
        }

        // Parsear declaraciones hasta el final
        while !self.is_at_end() {
            println!("🔧 Main parse loop, current token: {:?}", self.current_token());
            // Skip semicolons between declarations
            if self.check(TokenKind::Semicolon) {
                self.advance();
                continue;
            }
            declarations.push(self.parse_declaration()?);
        }

        let range = self.create_range_from_start();
        Ok(Program::new(range, imports, declarations))
    }

    /// Parsea una declaración de import.
    fn parse_import(&mut self) -> CompileResult<ImportDeclaration> {
        let start_pos = self.current_token().range.start.clone();

        self.consume(TokenKind::Import)?;
        let kind = self.parse_import_kind()?;
        let path = self.consume_string()?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(ImportDeclaration::new(range, kind, path, None, None, None))
    }

    /// Parsea el tipo de import.
    fn parse_import_kind(&mut self) -> CompileResult<ImportKind> {
        if self.check(TokenKind::Package) {
            self.advance();
            Ok(ImportKind::Package)
        } else if self.check(TokenKind::Library) {
            self.advance();
            Ok(ImportKind::Library)
        } else if self.check(TokenKind::Extension) {
            self.advance();
            Ok(ImportKind::Extension)
        } else {
            Ok(ImportKind::Package) // default
        }
    }

    /// Parsea una declaración.
    fn parse_declaration(&mut self) -> CompileResult<Declaration> {
        println!("🔧 parse_declaration called, current token: {:?}", self.current_token());

        let is_public = self.check(TokenKind::Public);
        if is_public {
            self.advance();
        }

        // Parse decorators before any declaration
        let decorators = self.parse_decorators()?;

        // Base language declarations
        if self.check(TokenKind::Fn) {
            println!("🔧 Found fn token, parsing function declaration");
            self.parse_function_declaration(is_public, decorators)
        } else if self.check(TokenKind::Struct) {
            self.parse_struct_declaration(is_public, decorators)
        } else if self.check(TokenKind::Enum) {
            self.parse_enum_declaration(is_public)
        } else if self.check(TokenKind::Type) {
            self.parse_type_alias_declaration(is_public)
        } else if self.check(TokenKind::State) {
            let var_decl = self.parse_variable_declaration()?;
            Ok(Declaration::Variable(var_decl))
        } else if self.check(TokenKind::Class) {
            self.parse_class_declaration(is_public, decorators)
        } else if self.check(TokenKind::Abstract) {
            self.parse_abstract_class_declaration(is_public)
        } else if self.check(TokenKind::Interface) {
            self.parse_interface_declaration(is_public)
        } else if self.check(TokenKind::Mixin) {
            self.parse_mixin_declaration(is_public)

        // Business logic declarations
        } else if self.check(TokenKind::Service) {
            self.parse_service_declaration(is_public, decorators)
        } else if self.check(TokenKind::Repository) {
            self.parse_repository_declaration(is_public)
        } else if self.check(TokenKind::Controller) {
            self.parse_controller_declaration(is_public, decorators)
        } else if self.check(TokenKind::UseCase) {
            self.parse_usecase_declaration(is_public)
        } else if self.check(TokenKind::Entity) {
            self.parse_entity_declaration(is_public)
        } else if self.check(TokenKind::ValueObject) {
            self.parse_valueobject_declaration(is_public)
        } else if self.check(TokenKind::Dto) {
            self.parse_dto_declaration(is_public)
        } else if self.check(TokenKind::Model) {
            self.parse_model_declaration(is_public)

        // UI declarations
        } else if self.check(TokenKind::Widget) {
            self.parse_widget_declaration(is_public)
        } else if self.check(TokenKind::Component) {
            self.parse_component_declaration(is_public)

        // Pattern declarations
        } else if self.check(TokenKind::Factory) {
            self.parse_factory_declaration(is_public)
        } else if self.check(TokenKind::Builder) {
            self.parse_builder_declaration(is_public)
        } else if self.check(TokenKind::Strategy) {
            self.parse_strategy_declaration(is_public)
        } else if self.check(TokenKind::Observer) {
            self.parse_observer_declaration(is_public)
        } else if self.check(TokenKind::Singleton) {
            self.parse_singleton_declaration(is_public)
        } else if self.check(TokenKind::Adapter) {
            self.parse_adapter_declaration(is_public)
        } else if self.check(TokenKind::Decorator) {
            self.parse_decorator_declaration(is_public)

        // Security declarations
        } else if self.check(TokenKind::Guard) {
            self.parse_guard_declaration(is_public)
        } else if self.check(TokenKind::Middleware) {
            self.parse_middleware_declaration(is_public)
        } else if self.check(TokenKind::Interceptor) {
            self.parse_interceptor_declaration(is_public)
        } else if self.check(TokenKind::Validator) {
            self.parse_validator_declaration(is_public)

        // Architecture declarations
        } else if self.check(TokenKind::Store) {
            self.parse_store_declaration(is_public)
        } else if self.check(TokenKind::Provider) {
            self.parse_provider_declaration(is_public)
        } else if self.check(TokenKind::Actor) {
            self.parse_actor_declaration(is_public)
        } else if self.check(TokenKind::Pipe) {
            self.parse_pipe_declaration(is_public)
        } else if self.check(TokenKind::Task) {
            self.parse_task_declaration(is_public)
        } else if self.check(TokenKind::Helper) {
            self.parse_helper_declaration(is_public)
        } else if self.check(TokenKind::Mapper) {
            self.parse_mapper_declaration(is_public)
        } else if self.check(TokenKind::Serializer) {
            self.parse_serializer_declaration(is_public)

        // Module declaration
        } else if self.check(TokenKind::Module) {
            self.parse_module_declaration(is_public, decorators)

        } else {
            println!("🔧 No valid declaration token found, current token: {:?}", self.current_token());
            Err(CompileError::Parse(self.error("Expected declaration")))
        }
    }

    /// Parsea una declaración de función.
    fn parse_function_declaration(&mut self, is_public: bool, decorators: Vec<Decorator>) -> CompileResult<Declaration> {
        println!("🔧 parse_function_declaration called");
        let start_pos = self.current_token().range.start.clone();

        self.consume(TokenKind::Fn)?;
        println!("🔧 consumed fn token");
        let name = self.consume_identifier()?;
        println!("🔧 consumed function name: {:?}", name);
        let generic_params = self.parse_generic_parameters()?;
        self.consume(TokenKind::LeftParen)?;
        let parameters = self.parse_parameters()?;
        self.consume(TokenKind::RightParen)?;
        let return_type = if self.check(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        let body = self.parse_block_statement()?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        let is_async = false; // TODO: implementar async

        let func = FunctionDeclaration::new(
            range,
            is_public,
            name,
            decorators,
            parameters,
            return_type,
            body,
            is_async,
            generic_params,
        );

        Ok(Declaration::Function(func))
    }

    /// Parsea una declaración de struct.
    fn parse_struct_declaration(&mut self, is_public: bool, decorators: Vec<Decorator>) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();

        self.consume(TokenKind::Struct)?;
        let name = self.consume_identifier()?;
        let generic_params = self.parse_generic_parameters()?;
        self.consume(TokenKind::LeftBrace)?;
        let fields = self.parse_struct_fields()?;
        self.consume(TokenKind::RightBrace)?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        let struct_decl = StructDeclaration::new(range, is_public, name, decorators, fields, generic_params);
        Ok(Declaration::Struct(struct_decl))
    }

    /// Parsea una declaración de enum.
    fn parse_enum_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();

        self.consume(TokenKind::Enum)?;
        let name = self.consume_identifier()?;
        let generic_params = self.parse_generic_parameters()?;
        self.consume(TokenKind::LeftBrace)?;
        let variants = self.parse_enum_variants()?;
        self.consume(TokenKind::RightBrace)?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        let enum_decl = EnumDeclaration::new(range, is_public, name, variants, generic_params);
        Ok(Declaration::Enum(enum_decl))
    }

    /// Parsea una declaración de type alias.
    fn parse_type_alias_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();

        self.consume(TokenKind::Type)?;
        let name = self.consume_identifier()?;
        self.consume(TokenKind::Equal)?;
        let type_annotation = self.parse_type_annotation()?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        let type_alias = TypeAliasDeclaration::new(range, is_public, name, type_annotation);
        Ok(Declaration::TypeAlias(type_alias))
    }

    /// Parsea parámetros de función.
    fn parse_parameters(&mut self) -> CompileResult<Vec<Parameter>> {
        let mut parameters = Vec::new();

        while !self.check(TokenKind::RightParen) {
            let pattern = self.parse_pattern()?;
            let type_annotation = if self.check(TokenKind::Colon) {
                self.advance();
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
            let default_value = if self.check(TokenKind::Equal) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            let range = Range::new(
                pattern.range().start.clone(),
                self.previous_token().range.end.clone(),
            );

            parameters.push(Parameter::new(pattern, type_annotation, default_value, range));

            if !self.check(TokenKind::RightParen) {
                self.consume(TokenKind::Comma)?;
            }
        }

        Ok(parameters)
    }

    /// Parsea campos de struct.
    fn parse_struct_fields(&mut self) -> CompileResult<Vec<StructField>> {
        let mut fields = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let is_public = self.check(TokenKind::Public);
            if is_public {
                self.advance();
            }

            let name = self.consume_identifier()?;
            self.consume(TokenKind::Colon)?;
            let type_annotation = self.parse_type_annotation()?;

            let range = Range::new(
                self.tokens[self.current - name.len() - 1].range.start.clone(),
                self.previous_token().range.end.clone(),
            );

            fields.push(StructField::new(name, type_annotation, is_public, range));

            if !self.check(TokenKind::RightBrace) {
                self.consume(TokenKind::Comma)?;
            }
        }

        Ok(fields)
    }

    /// Parsea variantes de enum.
    fn parse_enum_variants(&mut self) -> CompileResult<Vec<EnumVariant>> {
        let mut variants = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let name = self.consume_identifier()?;
            let fields = if self.check(TokenKind::LeftParen) {
                self.advance();
                let mut field_types = Vec::new();
                while !self.check(TokenKind::RightParen) {
                    field_types.push(self.parse_type_annotation()?);
                    if !self.check(TokenKind::RightParen) {
                        self.consume(TokenKind::Comma)?;
                    }
                }
                self.consume(TokenKind::RightParen)?;
                // Convert TypeAnnotation to StructField for enum variants
                Some(field_types.into_iter().enumerate().map(|(i, ty)| {
                    StructField::new(format!("field{}", i), ty, false, Range::new(Position::new(0, 0), Position::new(0, 0)))
                }).collect())
            } else {
                None
            };

            let range = Range::new(
                self.tokens[self.current - name.len() - 1].range.start.clone(),
                self.previous_token().range.end.clone(),
            );

            variants.push(EnumVariant::new(name, fields, range));

            if !self.check(TokenKind::RightBrace) {
                self.consume(TokenKind::Comma)?;
            }
        }

        Ok(variants)
    }

    /// Parsea parámetros genéricos.
    fn parse_generic_parameters(&mut self) -> CompileResult<Vec<GenericParameter>> {
        if !self.check(TokenKind::Less) {
            return Ok(Vec::new());
        }

        self.advance();
        let mut params = Vec::new();

        while !self.check(TokenKind::Greater) {
            let start_pos = self.current_token().range.start.clone();
            let name = self.consume_identifier()?;
            let end_pos = self.previous_token().range.end.clone();
            let range = Range::new(start_pos, end_pos);
            params.push(GenericParameter::new(name, Vec::new(), range)); // TODO: constraints
            if !self.check(TokenKind::Greater) {
                self.consume(TokenKind::Comma)?;
            }
        }

        self.consume(TokenKind::Greater)?;
        Ok(params)
    }

    /// Parsea una anotación de tipo.
    fn parse_type_annotation(&mut self) -> CompileResult<TypeAnnotation> {
        if self.check(TokenKind::LeftParen) {
            self.parse_tuple_type()
        } else if self.check(TokenKind::LeftBracket) {
            self.parse_array_type()
        } else if self.check(TokenKind::Fn) {
            self.parse_function_type()
        } else {
            self.parse_simple_type()
        }
    }

    /// Parsea un tipo simple (primitivo o nombrado).
    fn parse_simple_type(&mut self) -> CompileResult<TypeAnnotation> {
        let start_pos = self.current_token().range.start.clone();
        let name = self.consume_identifier()?;

        // Check for generic type
        if self.check(TokenKind::Less) {
            self.advance();
            let mut type_args = Vec::new();
            while !self.check(TokenKind::Greater) {
                type_args.push(self.parse_type_annotation()?);
                if !self.check(TokenKind::Greater) {
                    self.consume(TokenKind::Comma)?;
                }
            }
            self.consume(TokenKind::Greater)?;

            let end_pos = self.previous_token().range.end.clone();
            let range = Range::new(start_pos, end_pos);

            Ok(TypeAnnotation::Generic(GenericType::new(range, name, type_args)))
        } else {
            let end_pos = self.previous_token().range.end.clone();
            let range = Range::new(start_pos, end_pos);

            // Check if it's a primitive type
            match name.as_str() {
                "Number" | "Float" | "String" | "Bool" | "void" | "never" => {
                    Ok(TypeAnnotation::Primitive(PrimitiveType::new(range, name)))
                }
                _ => Ok(TypeAnnotation::Named(NamedType::new(range, name))),
            }
        }
    }

    /// Parsea un tipo array.
    fn parse_array_type(&mut self) -> CompileResult<TypeAnnotation> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::LeftBracket)?;
        let element_type = self.parse_type_annotation()?;
        self.consume(TokenKind::RightBracket)?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(TypeAnnotation::Array(ArrayType::new(range, element_type)))
    }

    /// Parsea un tipo tuple.
    fn parse_tuple_type(&mut self) -> CompileResult<TypeAnnotation> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::LeftParen)?;
        let mut element_types = Vec::new();

        while !self.check(TokenKind::RightParen) {
            element_types.push(self.parse_type_annotation()?);
            if !self.check(TokenKind::RightParen) {
                self.consume(TokenKind::Comma)?;
            }
        }

        self.consume(TokenKind::RightParen)?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(TypeAnnotation::Tuple(TupleType::new(range, element_types)))
    }

    /// Parsea un tipo función.
    fn parse_function_type(&mut self) -> CompileResult<TypeAnnotation> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Fn)?;
        self.consume(TokenKind::LeftParen)?;
        let mut param_types = Vec::new();

        while !self.check(TokenKind::RightParen) {
            param_types.push(self.parse_type_annotation()?);
            if !self.check(TokenKind::RightParen) {
                self.consume(TokenKind::Comma)?;
            }
        }

        self.consume(TokenKind::RightParen)?;
        self.consume(TokenKind::Arrow)?;
        let return_type = self.parse_type_annotation()?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(TypeAnnotation::Function(FunctionType::new(range, param_types, return_type)))
    }

    /// Parsea un statement.
    fn parse_statement(&mut self) -> CompileResult<Statement> {
        if self.check(TokenKind::LeftBrace) {
            Ok(Statement::Block(self.parse_block_statement()?))
        } else if self.check(TokenKind::If) {
            Ok(Statement::If(self.parse_if_statement()?))
        } else if self.check(TokenKind::Match) {
            Ok(Statement::Match(self.parse_match_statement()?))
        } else if self.check(TokenKind::Return) {
            Ok(Statement::Return(self.parse_return_statement()?))
        } else if self.check(TokenKind::State) {
            Ok(Statement::Variable(self.parse_variable_declaration()?))
        } else if matches!(self.current_token().kind, TokenKind::Identifier(_)) && self.current + 1 < self.tokens.len() && self.tokens[self.current + 1].kind == TokenKind::Colon {
            // Variable declaration without 'state' keyword
            Ok(Statement::Variable(self.parse_variable_declaration()?))
        } else {
            Ok(Statement::Expression(self.parse_expression_statement()?))
        }
    }

    /// Parsea un block statement.
    fn parse_block_statement(&mut self) -> CompileResult<BlockStatement> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::LeftBrace)?;
        let mut statements = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.consume(TokenKind::RightBrace)?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(BlockStatement::new(range, statements))
    }

    /// Parsea un if statement.
    fn parse_if_statement(&mut self) -> CompileResult<IfStatement> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::If)?;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_statement()?;
        let else_branch = if self.check(TokenKind::Else) {
            self.advance();
            Some(self.parse_statement()?)
        } else {
            None
        };

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(IfStatement::new(range, condition, then_branch, else_branch))
    }

    /// Parsea un statement match.
    fn parse_match_statement(&mut self) -> CompileResult<MatchStatement> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Match)?;

        let value = self.parse_expression()?;
        self.consume(TokenKind::LeftBrace)?;

        let mut arms = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            arms.push(self.parse_match_arm()?);

            // Consume comma if present (optional for last arm)
            if self.check(TokenKind::Comma) {
                self.advance();
            }
        }

        self.consume(TokenKind::RightBrace)?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(MatchStatement::new(range, value, arms))
    }

    /// Parsea un brazo de match (pattern [if guard] => body).
    fn parse_match_arm(&mut self) -> CompileResult<MatchArm> {
        let start_pos = self.current_token().range.start.clone();

        println!("🔧 parse_match_arm: starting with token {:?}", self.current_token());
        let pattern = self.parse_pattern()?;
        println!("🔧 parse_match_arm: after pattern, current token {:?}", self.current_token());

        // Parse optional guard (if condition)
        let guard = if self.check(TokenKind::If) {
            println!("🔧 parse_match_arm: found 'if', parsing guard");
            self.advance();
            let guard_expr = self.parse_expression()?;
            println!("🔧 parse_match_arm: after guard expression, current token {:?}", self.current_token());
            Some(guard_expr)
        } else {
            None
        };

        println!("🔧 After parsing guard, current token: {:?}", self.current_token());
        self.consume(TokenKind::DoubleArrow)?;

        // Parse body (statement)
        let body = if self.check(TokenKind::LeftBrace) {
            self.parse_statement()?
        } else {
            Statement::Expression(self.parse_expression_statement()?)
        };

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(MatchArm::new(pattern, guard, body, range))
    }

    /// Parsea un return statement.
    fn parse_return_statement(&mut self) -> CompileResult<ReturnStatement> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Return)?;
        let value = if !self.check(TokenKind::Semicolon) && !self.check(TokenKind::RightBrace) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Consume semicolon if present
        if self.check(TokenKind::Semicolon) {
            self.advance();
        }

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(ReturnStatement::new(range, value))
    }

    /// Parsea una declaración de variable.
    fn parse_variable_declaration(&mut self) -> CompileResult<VariableDeclaration> {
        let start_pos = self.current_token().range.start.clone();
        let is_state = if self.check(TokenKind::State) {
            self.advance(); // consume 'state'
            true
        } else {
            false
        };

        let name = self.consume_identifier()?;
        let type_annotation = if self.check(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        let initializer = if self.check(TokenKind::Assign) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(VariableDeclaration::new(range, name, type_annotation, initializer, is_state))
    }

    /// Parsea un expression statement.
    fn parse_expression_statement(&mut self) -> CompileResult<ExpressionStatement> {
        let start_pos = self.current_token().range.start.clone();
        let expression = self.parse_expression()?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(ExpressionStatement::new(range, expression))
    }

    /// Parsea una expresión con precedence climbing.
    fn parse_expression(&mut self) -> CompileResult<Expression> {
        self.parse_precedence(Precedence::Lowest)
    }

    /// Parsea expresión con precedence climbing.
    fn parse_precedence(&mut self, precedence: Precedence) -> CompileResult<Expression> {
        let mut left = self.parse_unary()?;

        loop {
            let current_kind = self.current_token_kind();

            // Handle member access (dot notation)
            if current_kind == TokenKind::Dot {
                self.advance(); // consume '.'
                let member_name = self.consume_identifier()?;
                let start_pos = match &left {
                    Expression::Literal(lit) => lit.node.range.start.clone(),
                    Expression::Identifier(ident) => ident.node.range.start.clone(),
                    Expression::Binary(bin) => bin.node.range.start.clone(),
                    Expression::Unary(un) => un.node.range.start.clone(),
                    Expression::Call(call) => call.node.range.start.clone(),
                    Expression::MemberAccess(mem) => mem.node.range.start.clone(),
                    _ => self.previous_token().range.start.clone(),
                };
                let end_pos = self.previous_token().range.end.clone();
                let range = Range::new(start_pos, end_pos);
                left = Expression::MemberAccess(MemberAccessExpression::new(range, left, member_name, false));
                continue;
            }

            // Handle binary operators
            if let Some(op_precedence) = self.get_precedence(&current_kind) {
                if op_precedence <= precedence {
                    break;
                }

                let operator = current_kind;
                self.advance();

                let right = self.parse_precedence(op_precedence)?;
                left = self.create_binary_expr(left, operator, right);
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parsea expresión unaria.
    fn parse_unary(&mut self) -> CompileResult<Expression> {
        if self.check(TokenKind::Not) || self.check(TokenKind::Minus) {
            let start_pos = self.current_token().range.start.clone();
            let operator = self.current_token_kind();
            self.advance();
            let operand = self.parse_unary()?;

            let end_pos = self.previous_token().range.end.clone();
            let range = Range::new(start_pos, end_pos);

            Ok(Expression::Unary(UnaryExpression::new(range, format!("{:?}", operator), operand)))
        } else {
            self.parse_primary()
        }
    }

    /// Parsea expresión primaria.
    fn parse_primary(&mut self) -> CompileResult<Expression> {
        let start_pos = self.current_token().range.start.clone();

        match &self.current_token().kind {
            TokenKind::NumberLiteral(value) => {
                let value = value.parse::<f64>().map_err(|_| CompileError::Parse(self.error("Invalid number")))?;
                self.advance();
                let end_pos = self.previous_token().range.end.clone();
                let range = Range::new(start_pos, end_pos);
                Ok(Expression::Literal(Literal::new(range, serde_json::json!(value), "number".to_string())))
            }
            TokenKind::StringLiteral(value) => {
                let value = value.clone();
                self.advance();
                let end_pos = self.previous_token().range.end.clone();
                let range = Range::new(start_pos, end_pos);
                Ok(Expression::Literal(Literal::new(range, serde_json::json!(value), "string".to_string())))
            }
            TokenKind::BoolLiteral(true) => {
                self.advance();
                let end_pos = self.previous_token().range.end.clone();
                let range = Range::new(start_pos, end_pos);
                Ok(Expression::Literal(Literal::new(range, serde_json::json!(true), "bool".to_string())))
            }
            TokenKind::BoolLiteral(false) => {
                self.advance();
                let end_pos = self.previous_token().range.end.clone();
                let range = Range::new(start_pos, end_pos);
                Ok(Expression::Literal(Literal::new(range, serde_json::json!(false), "bool".to_string())))
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                // Check if this is a function call
                if self.check(TokenKind::LeftParen) {
                    self.advance(); // consume '('
                    let mut arguments = Vec::new();
                    
                    // Parse arguments
                    if !self.check(TokenKind::RightParen) {
                        loop {
                            arguments.push(self.parse_expression()?);
                            if !self.check(TokenKind::Comma) {
                                break;
                            }
                            self.advance(); // consume ','
                        }
                    }
                    
                    self.consume(TokenKind::RightParen)?;
                    
                    let end_pos = self.previous_token().range.end.clone();
                    let range = Range::new(start_pos.clone(), end_pos);
                    let callee = Expression::Identifier(Identifier::new(Range::new(start_pos.clone(), start_pos), name));
                    
                    Ok(Expression::Call(CallExpression::new(range, callee, arguments)))
                } else {
                    let end_pos = self.previous_token().range.end.clone();
                    let range = Range::new(start_pos, end_pos);
                    Ok(Expression::Identifier(Identifier::new(range, name)))
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                // Check if this is a lambda: (params) => body
                if self.check(TokenKind::RightParen) {
                    // Empty parentheses, could be lambda with no params
                    self.advance(); // consume ')'
                    if self.check(TokenKind::DoubleArrow) {
                        // Lambda with no parameters: () => body
                        self.advance(); // consume '=>'
                        let body = self.parse_lambda_body()?;
                        let end_pos = self.previous_token().range.end.clone();
                        let range = Range::new(start_pos, end_pos);
                        Ok(Expression::Lambda(LambdaExpression::new(range, vec![], body)))
                    } else {
                        // Just empty parentheses
                        Ok(Expression::TupleLiteral(TupleLiteral::new(Range::new(start_pos, self.previous_token().range.end.clone()), vec![])))
                    }
                } else {
                    // Parse first expression/parameter
                    let first_expr = self.parse_expression()?;

                    if self.check(TokenKind::RightParen) {
                        // Single expression in parentheses
                        self.advance(); // consume ')'
                        if self.check(TokenKind::DoubleArrow) {
                            // Lambda with single parameter (expression as pattern)
                            self.advance(); // consume '=>'
                            let pattern = self.expression_to_pattern(first_expr.clone())?;
                            let body = self.parse_lambda_body()?;
                            let end_pos = self.previous_token().range.end.clone();
                            let range = Range::new(start_pos, end_pos);
                            let param_range = first_expr.range();
                            let param = Parameter::new(pattern, None, None, param_range.clone());
                            Ok(Expression::Lambda(LambdaExpression::new(range, vec![param], body)))
                        } else {
                            // Just parentheses around expression
                            Ok(first_expr)
                        }
                    } else if self.check(TokenKind::Comma) {
                        // Multiple parameters: (param1, param2) => body
                        self.advance(); // consume ','
                        let mut parameters = vec![self.expression_to_parameter(first_expr)?];

                        // Parse remaining parameters
                        while !self.check(TokenKind::RightParen) {
                            let param_expr = self.parse_expression()?;
                            parameters.push(self.expression_to_parameter(param_expr)?);

                            if self.check(TokenKind::Comma) {
                                self.advance(); // consume ','
                            } else {
                                break;
                            }
                        }

                        self.consume(TokenKind::RightParen)?;

                        if self.check(TokenKind::DoubleArrow) {
                            // This is a lambda
                            self.advance(); // consume '=>'
                            let body = self.parse_lambda_body()?;
                            let end_pos = self.previous_token().range.end.clone();
                            let range = Range::new(start_pos, end_pos);
                            Ok(Expression::Lambda(LambdaExpression::new(range, parameters, body)))
                        } else {
                            // This is a tuple
                            let elements = parameters.into_iter()
                                .map(|param| self.parameter_to_expression(param))
                                .collect::<CompileResult<Vec<Expression>>>()?;
                            let end_pos = self.previous_token().range.end.clone();
                            let range = Range::new(start_pos, end_pos);
                            Ok(Expression::TupleLiteral(TupleLiteral::new(range, elements)))
                        }
                    } else {
                        // Single expression in parentheses
                        let expr = first_expr;
                        self.consume(TokenKind::RightParen)?;
                        Ok(expr)
                    }
                }
            }
            TokenKind::Dispatch => {
                self.advance(); // consume 'dispatch'
                self.consume(TokenKind::LeftParen)?;
                let action = self.parse_expression()?;
                self.consume(TokenKind::RightParen)?;
                let end_pos = self.previous_token().range.end.clone();
                let range = Range::new(start_pos, end_pos);
                Ok(Expression::Dispatch(DispatchExpression::new(range, action)))
            }
            _ => Err(CompileError::Parse(self.error("Expected expression"))),
        }
    }

    /// Crea una expresión binaria.
    fn create_binary_expr(&mut self, left: Expression, operator: TokenKind, right: Expression) -> Expression {
        let start_pos = match &left {
            Expression::Literal(lit) => lit.node.range.start.clone(),
            Expression::Identifier(ident) => ident.node.range.start.clone(),
            Expression::Binary(bin) => bin.node.range.start.clone(),
            Expression::Unary(un) => un.node.range.start.clone(),
            Expression::Call(call) => call.node.range.start.clone(),
            _ => self.tokens[self.current - 1].range.start.clone(),
        };

        let end_pos = match &right {
            Expression::Literal(lit) => lit.node.range.end.clone(),
            Expression::Identifier(ident) => ident.node.range.end.clone(),
            Expression::Binary(bin) => bin.node.range.end.clone(),
            Expression::Unary(un) => un.node.range.end.clone(),
            Expression::Call(call) => call.node.range.end.clone(),
            _ => self.previous_token().range.end.clone(),
        };

        let range = Range::new(start_pos, end_pos);
        let operator_str = match operator {
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Equal => "==",
            TokenKind::NotEqual => "!=",
            TokenKind::Less => "<",
            TokenKind::LessEqual => "<=",
            TokenKind::Greater => ">",
            TokenKind::GreaterEqual => ">=",
            TokenKind::And => "&&",
            TokenKind::Or => "||",
            _ => "unknown",
        };

        Expression::Binary(BinaryExpression::new(range, left, operator_str.to_string(), right))
    }

    /// Obtiene la precedencia de un operador.
    fn get_precedence(&self, kind: &TokenKind) -> Option<Precedence> {
        match kind {
            TokenKind::Or => Some(Precedence::Or),
            TokenKind::And => Some(Precedence::And),
            TokenKind::Equal | TokenKind::NotEqual => Some(Precedence::Equality),
            TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => Some(Precedence::Comparison),
            TokenKind::Plus | TokenKind::Minus => Some(Precedence::Term),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(Precedence::Factor),
            TokenKind::Dot => Some(Precedence::Member),
            _ => None,
        }
    }

    /// Crea un rango desde el token actual hasta el anterior.
    fn create_range_from_start(&self) -> Range {
        if self.tokens.is_empty() {
            Range::new(Position::new(1, 1), Position::new(1, 1))
        } else if self.current == 0 {
            // No hay token anterior, usar el primer token
            Range::new(self.tokens[0].range.start.clone(), self.tokens[0].range.end.clone())
        } else {
            Range::new(self.tokens[0].range.start.clone(), self.previous_token().range.end.clone())
        }
    }

    /// Verifica si estamos al final de los tokens.
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.current_token().kind == TokenKind::EOF
    }

    /// Obtiene el token actual.
    fn current_token(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Obtiene el token anterior.
    fn previous_token(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    /// Obtiene el tipo del token actual.
    fn current_token_kind(&self) -> TokenKind {
        self.current_token().kind.clone()
    }

    /// Avanza al siguiente token.
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous_token()
    }

    /// Verifica si el token actual es del tipo esperado.
    fn check(&self, kind: TokenKind) -> bool {
        !self.is_at_end() && matches!(&self.current_token().kind, k if std::mem::discriminant(k) == std::mem::discriminant(&kind))
    }

    /// Consume un token del tipo esperado.
    fn consume(&mut self, kind: TokenKind) -> CompileResult<Token> {
        if self.check(kind.clone()) {
            Ok(self.advance().clone())
        } else {
            Err(CompileError::Parse(self.error(&format!("Expected {:?}", kind))))
        }
    }

    /// Consume un identificador.
    fn consume_identifier(&mut self) -> CompileResult<String> {
        if let TokenKind::Identifier(name) = &self.current_token().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(CompileError::Parse(self.error("Expected identifier")))
        }
    }

    /// Consume un string.
    fn consume_string(&mut self) -> CompileResult<String> {
        if let TokenKind::StringLiteral(value) = &self.current_token().kind {
            let value = value.clone();
            self.advance();
            Ok(value)
        } else {
            Err(CompileError::Parse(self.error("Expected string")))
        }
    }

    /// Consume un número.
    fn consume_number(&mut self) -> CompileResult<f64> {
        if let TokenKind::NumberLiteral(value) = &self.current_token().kind {
            let result = value.parse::<f64>().map_err(|_| CompileError::Parse(self.error("Invalid number")));
            self.advance();
            result
        } else {
            Err(CompileError::Parse(self.error("Expected number")))
        }
    }

    /// Crea un error de parsing.
    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            location: SourceLocation::new(
                self.current_token().range.start.line,
                self.current_token().range.start.column,
                0, // offset not tracked in parser
            ),
            expected: Vec::new(),
        }
    }

    // ===================================================================
    // PARSING FUNCTIONS FOR ALL KEYWORDS
    // ===================================================================

    /// Parsea una declaración de clase.
    fn parse_class_declaration(&mut self, is_public: bool, decorators: Vec<Decorator>) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Class)?;
        let name = self.consume_identifier()?;
        // TODO: Implementar parsing completo de clases
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let class_decl = ClassDeclaration::new(range, is_public, name, decorators, None, Vec::new(), Vec::new(), None, Vec::new(), Vec::new());
        Ok(Declaration::Class(class_decl))
    }

    /// Parsea una declaración de clase abstracta.
    fn parse_abstract_class_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Abstract)?;
        self.consume(TokenKind::Class)?;
        let name = self.consume_identifier()?;
        // TODO: Implementar parsing completo de clases abstractas
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let class_decl = ClassDeclaration::new(range, is_public, name, Vec::new(), None, Vec::new(), Vec::new(), None, Vec::new(), Vec::new());
        Ok(Declaration::Class(class_decl))
    }

    /// Parsea una declaración de interfaz.
    fn parse_interface_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Interface)?;
        let name = self.consume_identifier()?;
        // TODO: Implementar parsing completo de interfaces
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let interface_decl = InterfaceDeclaration::new(range, is_public, name, Vec::new(), Vec::new());
        Ok(Declaration::Interface(interface_decl))
    }

    /// Parsea una declaración de mixin.
    fn parse_mixin_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Mixin)?;
        let name = self.consume_identifier()?;
        // TODO: Implementar parsing completo de mixins
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        // Usar una declaración existente como placeholder
        let struct_decl = StructDeclaration::new(range, is_public, name, Vec::new(), Vec::new(), Vec::new());
        Ok(Declaration::Struct(struct_decl))
    }

    /// Parsea una declaración de servicio.
    fn parse_service_declaration(&mut self, is_public: bool, decorators: Vec<Decorator>) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Service)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let service_decl = ServiceDeclaration::new(range, is_public, name, decorators, Vec::new(), Vec::new());
        Ok(Declaration::Service(service_decl))
    }

    /// Parsea una declaración de repositorio.
    fn parse_repository_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Repository)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let repo_decl = RepositoryDeclaration::new(range, is_public, name, String::new(), Vec::new());
        Ok(Declaration::Repository(repo_decl))
    }

    /// Parsea una declaración de controlador.
    fn parse_controller_declaration(&mut self, is_public: bool, decorators: Vec<Decorator>) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Controller)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let controller_decl = ControllerDeclaration::new(range, is_public, name, decorators, Vec::new());
        Ok(Declaration::Controller(controller_decl))
    }

    /// Parsea una declaración de caso de uso.
    fn parse_usecase_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::UseCase)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        // Crear MethodDeclaration por defecto con BlockStatement vacío
        let empty_body = BlockStatement::new(range.clone(), Vec::new());
        let default_method = MethodDeclaration::new(
            String::from("execute"),
            Vec::new(),
            None,
            empty_body,
            range.clone(),
            false, // is_async
            false, // is_override
            true,  // is_public
            false, // is_protected
            false  // is_private
        );
        let usecase_decl = UseCaseDeclaration::new(range, is_public, name, default_method, Vec::new());
        Ok(Declaration::UseCase(usecase_decl))
    }

    /// Parsea una declaración de entidad.
    fn parse_entity_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Entity)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        // Crear StructField por defecto para id_field
        let id_type = TypeAnnotation::Named(NamedType::new(range.clone(), "Number".to_string()));
        let default_id_field = StructField::new(
            String::from("id"),
            id_type,
            true, // is_public
            range.clone()
        );
        let entity_decl = EntityDeclaration::new(range, is_public, name, default_id_field, Vec::new());
        Ok(Declaration::Entity(entity_decl))
    }

    /// Parsea una declaración de value object.
    fn parse_valueobject_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::ValueObject)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let vo_decl = ValueObjectDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::ValueObject(vo_decl))
    }

    /// Parsea una declaración de DTO.
    fn parse_dto_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Dto)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let dto_decl = DTODeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::DTO(dto_decl))
    }

    /// Parsea una declaración de modelo.
    fn parse_model_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Model)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let model_decl = ModelDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Model(model_decl))
    }

    /// Parsea una declaración de widget.
    fn parse_widget_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Widget)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let widget_decl = WidgetDeclaration::new(range, is_public, name, Vec::new(), Vec::new());
        Ok(Declaration::Widget(widget_decl))
    }

    /// Parsea una declaración de componente.
    fn parse_component_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Component)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let component_decl = ComponentDeclaration::new(range, is_public, name, Vec::new(), Vec::new());
        Ok(Declaration::Component(component_decl))
    }

    /// Parsea una declaración de factory.
    fn parse_factory_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Factory)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let factory_decl = FactoryDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Factory(factory_decl))
    }

    /// Parsea una declaración de builder.
    fn parse_builder_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Builder)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let builder_decl = BuilderDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Builder(builder_decl))
    }

    /// Parsea una declaración de strategy.
    fn parse_strategy_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Strategy)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let strategy_decl = StrategyDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Strategy(strategy_decl))
    }

    /// Parsea una declaración de observer.
    fn parse_observer_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Observer)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let observer_decl = ObserverDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Observer(observer_decl))
    }

    /// Parsea una declaración de singleton.
    fn parse_singleton_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Singleton)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let singleton_decl = SingletonDeclaration::new(range, is_public, name, Vec::new(), Vec::new());
        Ok(Declaration::Singleton(singleton_decl))
    }

    /// Parsea una declaración de adapter.
    fn parse_adapter_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Adapter)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let adapter_decl = AdapterDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Adapter(adapter_decl))
    }

    /// Parsea decoradores aplicados (con @) antes de una declaración.
    fn parse_decorators(&mut self) -> CompileResult<Vec<Decorator>> {
        let mut decorators = Vec::new();

        while self.check(TokenKind::At) {
            println!("🔧 parse_decorators: found @, parsing decorator");
            let start_pos = self.current_token().range.start.clone();
            self.consume(TokenKind::At)?;
            println!("🔧 parse_decorators: consumed @, current token: {:?}", self.current_token());

            let name = self.consume_identifier()?;
            println!("🔧 parse_decorators: consumed decorator name '{}', current token: {:?}", name, self.current_token());

            // Parse arguments if present
            let arguments = if self.check(TokenKind::LeftParen) {
                println!("🔧 parse_decorators: found (, parsing arguments");
                self.consume(TokenKind::LeftParen)?;
                println!("🔧 parse_decorators: consumed (, current token: {:?}", self.current_token());
                let mut args = Vec::new();
                if !self.check(TokenKind::RightParen) {
                    args.push(self.parse_expression()?);
                    println!("🔧 parse_decorators: parsed first argument, current token: {:?}", self.current_token());
                    while self.check(TokenKind::Comma) {
                        self.advance(); // consume ','
                        args.push(self.parse_expression()?);
                        println!("🔧 parse_decorators: parsed additional argument, current token: {:?}", self.current_token());
                    }
                }
                self.consume(TokenKind::RightParen)?;
                println!("🔧 parse_decorators: consumed ), current token: {:?}", self.current_token());
                args
            } else {
                Vec::new()
            };

            let end_pos = self.previous_token().range.end.clone();
            let range = Range::new(start_pos, end_pos);

            decorators.push(Decorator::new(name, arguments, range));
            println!("🔧 parse_decorators: added decorator, continuing loop");
        }

        println!("🔧 parse_decorators: finished parsing decorators, returning {} decorators", decorators.len());
        Ok(decorators)
    }

    /// Parsea una declaración de decorator.
    fn parse_decorator_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Decorator)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let decorator_decl = DecoratorDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Decorator(decorator_decl))
    }

    /// Parsea una declaración de guard.
    fn parse_guard_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Guard)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let guard_decl = GuardDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Guard(guard_decl))
    }

    /// Parsea una declaración de middleware.
    fn parse_middleware_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Middleware)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let middleware_decl = MiddlewareDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Middleware(middleware_decl))
    }

    /// Parsea una declaración de interceptor.
    fn parse_interceptor_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Interceptor)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let interceptor_decl = InterceptorDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Interceptor(interceptor_decl))
    }

    /// Parsea una declaración de validator.
    fn parse_validator_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Validator)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let validator_decl = ValidatorDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Validator(validator_decl))
    }

    /// Parsea una declaración de store.
    fn parse_store_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Store)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let store_decl = StoreDeclaration::new(range, is_public, name, Vec::new(), Vec::new());
        Ok(Declaration::Store(store_decl))
    }

    /// Parsea una declaración de provider.
    fn parse_provider_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Provider)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let provider_decl = ProviderDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Provider(provider_decl))
    }

    /// Parsea una declaración de actor.
    fn parse_actor_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Actor)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let actor_decl = ActorDeclaration::new(range, is_public, name, Vec::new(), Vec::new());
        Ok(Declaration::Actor(actor_decl))
    }

    /// Parsea una declaración de pipe.
    fn parse_pipe_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Pipe)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let pipe_decl = PipeDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Pipe(pipe_decl))
    }

    /// Parsea una declaración de task.
    fn parse_task_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Task)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let task_decl = TaskDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Task(task_decl))
    }

    /// Parsea una declaración de helper.
    fn parse_helper_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Helper)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let helper_decl = HelperDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Helper(helper_decl))
    }

    /// Parsea una declaración de mapper.
    fn parse_mapper_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Mapper)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let mapper_decl = MapperDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Mapper(mapper_decl))
    }

    /// Parsea una declaración de serializer.
    fn parse_serializer_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Serializer)?;
        let name = self.consume_identifier()?;
        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let serializer_decl = SerializerDeclaration::new(range, is_public, name, Vec::new());
        Ok(Declaration::Serializer(serializer_decl))
    }    /// Parsea una declaración de módulo.
    fn parse_module_declaration(&mut self, is_public: bool, decorators: Vec<Decorator>) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();
        self.consume(TokenKind::Module)?;
        let name = self.consume_identifier()?;

        // Parse module body
        self.consume(TokenKind::LeftBrace)?;
        let mut body = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            // Skip semicolons between declarations
            if self.check(TokenKind::Semicolon) {
                self.advance();
                continue;
            }

            // Parse declaration within module (can have decorators)
            let is_decl_public = self.check(TokenKind::Public);
            if is_decl_public {
                self.advance();
            }

            // Parse decorators for declarations within module
            let decl_decorators = self.parse_decorators()?;

            // Parse the actual declaration
            let declaration = self.parse_declaration_within_module(is_decl_public, decl_decorators)?;
            body.push(declaration);
        }

        self.consume(TokenKind::RightBrace)?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);
        let module_decl = ModuleDeclaration::new(range, is_public, name, decorators, body, Vec::new(), Vec::new(), Vec::new(), Vec::new());
        Ok(Declaration::Module(module_decl))
    }

    /// Parsea una lista de expresiones separadas por comas hasta encontrar el delimitador final.
    fn parse_expression_list(&mut self, end_token: TokenKind) -> CompileResult<Vec<Expression>> {
        let mut expressions = Vec::new();

        // Parse expressions
        if !self.check(end_token) {
            loop {
                expressions.push(self.parse_expression()?);
                if !self.check(TokenKind::Comma) {
                    break;
                }
                self.advance(); // consume ','
            }
        }

        Ok(expressions)
    }

    /// Parsea una declaración dentro de un módulo (puede tener decoradores).
    fn parse_declaration_within_module(&mut self, is_public: bool, decorators: Vec<Decorator>) -> CompileResult<Declaration> {
        println!("🔧 parse_declaration_within_module called, current token: {:?}", self.current_token());

        // Base language declarations within modules
        if self.check(TokenKind::Fn) {
            println!("🔧 Found fn token within module, parsing function declaration");
            self.parse_function_declaration(is_public, decorators)
        } else if self.check(TokenKind::Struct) {
            self.parse_struct_declaration(is_public, decorators)
        } else if self.check(TokenKind::Enum) {
            self.parse_enum_declaration(is_public)
        } else if self.check(TokenKind::Type) {
            self.parse_type_alias_declaration(is_public)
        } else {
            println!("🔧 No valid declaration token found within module, current token: {:?}", self.current_token());
            let location = SourceLocation::new(
                self.current_token().range.start.line,
                self.current_token().range.start.column,
                0 // offset not tracked in parser
            );
            return Err(CompileError::Parse(ParseError {
                message: "Expected declaration".to_string(),
                location,
                expected: vec!["fn".to_string(), "struct".to_string(), "enum".to_string(), "type".to_string()],
            }));
        }
    }

    // ===================================================================
    // PATTERN PARSING (TASK-117A: Destructuring avanzado)
    // ===================================================================

    /// Parsea un pattern.
    fn parse_pattern(&mut self) -> CompileResult<Pattern> {
        println!("🔧 parse_pattern called");

        // Parse first pattern
        let mut patterns = vec![self.parse_pattern_primary()?];

        // Check for or patterns: pattern1 | pattern2 | pattern3
        while self.check(TokenKind::Pipe) {
            self.advance(); // consume |
            patterns.push(self.parse_pattern_primary()?);
        }

        // If we have multiple patterns, create an OrPattern
        if patterns.len() > 1 {
            let start_pos = patterns[0].range().start.clone();
            let end_pos = patterns.last().unwrap().range().end.clone();
            Ok(Pattern::Or(OrPattern::new(
                Range::new(start_pos, end_pos),
                patterns,
            )))
        } else {
            // Single pattern, return as-is
            Ok(patterns.into_iter().next().unwrap())
        }
    }

    /// Parsea un patrón primario (sin operadores or)
    fn parse_pattern_primary(&mut self) -> CompileResult<Pattern> {
        println!("🔧 parse_pattern_primary called");
        let start_pos = self.current_token().range.start.clone();

        match &self.current_token().kind {
            TokenKind::Identifier(_) => {
                // Check if this is an enum pattern: Identifier(...
                if self.current + 1 < self.tokens.len() && self.tokens[self.current + 1].kind == TokenKind::LeftParen {
                    // Enum pattern: VariantName(inner_patterns...)
                    self.parse_enum_pattern()
                } else if self.current + 1 < self.tokens.len() && self.tokens[self.current + 1].kind == TokenKind::Colon {
                    // Struct pattern: {field: pattern, ...}
                    self.parse_struct_pattern()
                } else {
                    // Identifier pattern: variable
                    let name = self.consume_identifier()?;
                    let end_pos = self.previous_token().range.end.clone();
                    Ok(Pattern::Identifier(IdentifierPattern::new(
                        Range::new(start_pos, end_pos),
                        name,
                    )))
                }
            }
            TokenKind::LeftBracket => {
                // Array pattern: [elem1, elem2, ...rest]
                self.parse_array_pattern()
            }
            TokenKind::LeftParen => {
                // Tuple pattern: (elem1, elem2, elem3)
                self.parse_tuple_pattern()
            }
            TokenKind::LeftBrace => {
                // Struct pattern: {field1, field2, ...rest}
                self.parse_struct_pattern()
            }
            TokenKind::TripleDot => {
                // Rest pattern: ...variable (solo en contextos específicos)
                self.advance(); // consume ...
                let name = self.consume_identifier()?;
                let end_pos = self.previous_token().range.end.clone();
                Ok(Pattern::Identifier(IdentifierPattern::new(
                    Range::new(start_pos, end_pos),
                    format!("...{}", name),
                )))
            }
            TokenKind::Underscore => {
                // Wildcard pattern: _
                self.advance();
                let end_pos = self.previous_token().range.end.clone();
                Ok(Pattern::Wildcard(WildcardPattern::new(
                    Range::new(start_pos, end_pos),
                )))
            }
            _ => {
                // Intentar literal pattern, que puede ser seguido de range operator
                let literal_pattern = self.parse_literal_pattern()?;

                // Check if this is a range pattern: literal .. literal or literal ..= literal
                if self.check(TokenKind::DoubleDot) || self.check(TokenKind::DotDotEqual) {
                    let is_inclusive = self.check(TokenKind::DotDotEqual);
                    self.advance(); // consume .. or ..=

                    // Parse the end expression
                    let end_expr = self.parse_expression()?;

                    // Get ranges for the pattern
                    let start_range = literal_pattern.range().clone();
                    let end_range = match &end_expr {
                        Expression::Literal(lit) => lit.node.range.clone(),
                        Expression::Identifier(id) => id.node.range.clone(),
                        _ => return Err(CompileError::Parse(self.error("Range end must be a simple expression"))),
                    };

                    let pattern_range = Range::new(start_range.start.clone(), end_range.end.clone());

                    // Extract the start value from the literal pattern
                    let start_value = match &literal_pattern {
                        Pattern::Literal(lp) => lp.value.clone(),
                        _ => return Err(CompileError::Parse(self.error("Range start must be a literal"))),
                    };

                    // Create start expression from the literal value
                    let start_expr = Expression::Literal(Literal::new(
                        start_range.clone(),
                        start_value,
                        "number".to_string(), // Assume numbers for ranges
                    ));

                    Ok(Pattern::Range(RangePattern::new(
                        pattern_range,
                        start_expr,
                        end_expr,
                        is_inclusive,
                    )))
                } else {
                    Ok(literal_pattern)
                }
            }
        }
    }

    /// Parsea un array pattern: [elem1, elem2, ...rest]
    fn parse_array_pattern(&mut self) -> CompileResult<Pattern> {
        println!("🔧 parse_array_pattern called");
        let start_pos = self.current_token().range.start.clone();

        self.consume(TokenKind::LeftBracket)?;
        let mut elements = Vec::new();

        while !self.check(TokenKind::RightBracket) && !self.is_at_end() {
            if self.check(TokenKind::TripleDot) {
                // Spread operator: ...rest
                self.advance(); // consume ...
                let pattern = self.parse_pattern()?;
                elements.push(ArrayPatternElement::Rest(pattern));
            } else {
                // Normal element
                let pattern = self.parse_pattern()?;
                elements.push(ArrayPatternElement::Pattern(pattern));
            }

            if !self.check(TokenKind::RightBracket) {
                self.consume(TokenKind::Comma)?;
            }
        }

        self.consume(TokenKind::RightBracket)?;
        let end_pos = self.previous_token().range.end.clone();

        Ok(Pattern::Array(ArrayPattern::new(
            Range::new(start_pos, end_pos),
            elements,
        )))
    }

    /// Parsea un tuple pattern: (elem1, elem2, elem3)
    fn parse_tuple_pattern(&mut self) -> CompileResult<Pattern> {
        println!("🔧 parse_tuple_pattern called");
        let start_pos = self.current_token().range.start.clone();

        self.consume(TokenKind::LeftParen)?;
        let mut elements = Vec::new();

        while !self.check(TokenKind::RightParen) && !self.is_at_end() {
            elements.push(self.parse_pattern()?);

            if !self.check(TokenKind::RightParen) {
                self.consume(TokenKind::Comma)?;
            }
        }

        self.consume(TokenKind::RightParen)?;
        let end_pos = self.previous_token().range.end.clone();

        Ok(Pattern::Tuple(TuplePattern::new(
            Range::new(start_pos, end_pos),
            elements,
        )))
    }

    /// Parsea un struct pattern: StructName { field1, field2: pattern, ...rest }
    fn parse_struct_pattern(&mut self) -> CompileResult<Pattern> {
        println!("🔧 parse_struct_pattern called");
        let start_pos = self.current_token().range.start.clone();

        // Parse struct name
        let struct_name = if let TokenKind::Identifier(name) = &self.current_token().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(CompileError::Parse(self.error("Expected struct name")));
        };

        self.consume(TokenKind::LeftBrace)?;
        let mut fields = Vec::new();
        let mut has_rest = false;

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(TokenKind::TripleDot) {
                // Spread operator: ...rest
                self.advance(); // consume ...
                // Consume the identifier after ...
                let _rest_name = self.consume_identifier()?;
                has_rest = true;
                break; // ... debe ser el último elemento
            }

            // Parse field name
            let field_name = self.consume_identifier()?;

            let pattern = if self.check(TokenKind::Colon) {
                // field: pattern
                self.advance(); // consume :
                self.parse_pattern()?
            } else {
                // field (shorthand)
                Pattern::Identifier(IdentifierPattern::new(
                    self.previous_token().range.clone(),
                    field_name.clone(),
                ))
            };

            let field_range = Range::new(
                self.previous_token().range.start.clone(),
                pattern.range().end.clone(),
            );

            fields.push(StructPatternField::new(field_name, pattern, field_range));

            if !self.check(TokenKind::RightBrace) {
                self.consume(TokenKind::Comma)?;
            }
        }

        self.consume(TokenKind::RightBrace)?;
        let end_pos = self.previous_token().range.end.clone();

        Ok(Pattern::Struct(StructPattern::new(
            Range::new(start_pos, end_pos),
            struct_name,
            fields,
            has_rest,
        )))
    }

    /// Parsea un literal pattern (números, strings, booleanos)
    fn parse_literal_pattern(&mut self) -> CompileResult<Pattern> {
        let start_pos = self.current_token().range.start.clone();

        let value = match &self.current_token().kind {
            TokenKind::NumberLiteral(num) => {
                let parsed = num.parse::<f64>().map_err(|_| CompileError::Parse(self.error("Invalid number in pattern")))?;
                serde_json::json!(parsed)
            }
            TokenKind::StringLiteral(s) => {
                serde_json::json!(s)
            }
            TokenKind::True => {
                serde_json::json!(true)
            }
            TokenKind::False => {
                serde_json::json!(false)
            }
            _ => {
                return Err(CompileError::Parse(self.error("Expected literal")));
            }
        };

        self.advance();
        let end_pos = self.previous_token().range.end.clone();

        Ok(Pattern::Literal(LiteralPattern::new(
            Range::new(start_pos, end_pos),
            value,
        )))
    }

    /// Parsea un enum pattern: VariantName(inner_patterns...)
    fn parse_enum_pattern(&mut self) -> CompileResult<Pattern> {
        let start_pos = self.current_token().range.start.clone();

        // Consume variant name
        let variant_name = self.consume_identifier()?;

        // Consume left paren
        self.consume(TokenKind::LeftParen)?;

        // Parse inner patterns (optional)
        let inner_patterns = if self.check(TokenKind::RightParen) {
            None
        } else {
            let mut patterns = Vec::new();
            loop {
                patterns.push(self.parse_pattern()?);
                if !self.check(TokenKind::Comma) {
                    break;
                }
                self.advance(); // consume comma
            }
            Some(patterns)
        };

        // Consume right paren
        self.consume(TokenKind::RightParen)?;

        let end_pos = self.previous_token().range.end.clone();

        Ok(Pattern::Enum(EnumPattern::new(
            Range::new(start_pos, end_pos),
            variant_name,
            inner_patterns,
        )))
    }

    /// Parsea el cuerpo de una lambda expression
    fn parse_lambda_body(&mut self) -> CompileResult<LambdaBody> {
        if self.check(TokenKind::LeftBrace) {
            let block = self.parse_block_statement()?;
            Ok(LambdaBody::Block(block))
        } else {
            let expr = self.parse_expression()?;
            Ok(LambdaBody::Expression(Box::new(expr)))
        }
    }

    /// Convierte una expresión en un pattern (para lambdas con un solo parámetro)
    fn expression_to_pattern(&mut self, expr: Expression) -> CompileResult<Pattern> {
        match expr {
            Expression::Identifier(ident) => {
                Ok(Pattern::Identifier(IdentifierPattern::new(ident.node.range, ident.name)))
            }
            Expression::Literal(lit) => {
                Ok(Pattern::Literal(LiteralPattern::new(lit.node.range, lit.value)))
            }
            Expression::TupleLiteral(tuple_lit) => {
                // Convertir TupleLiteral a TuplePattern
                let elements = tuple_lit.elements.into_iter()
                    .map(|elem| self.expression_to_pattern(elem))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Pattern::Tuple(TuplePattern::new(
                    tuple_lit.node.range,
                    elements,
                )))
            }
            _ => Err(CompileError::Parse(self.error("Invalid pattern in lambda parameter"))),
        }
    }

    /// Convierte una expresión en un parámetro (para parsing de lambdas)
    fn expression_to_parameter(&mut self, expr: Expression) -> CompileResult<Parameter> {
        let pattern = self.expression_to_pattern(expr.clone())?;
        let range = expr.range().clone();
        Ok(Parameter::new(pattern, None, None, range.clone()))
    }

    /// Convierte un parámetro de vuelta a expresión (para tuples)
    fn parameter_to_expression(&mut self, param: Parameter) -> CompileResult<Expression> {
        match &param.pattern {
            Pattern::Identifier(ident) => {
                Ok(Expression::Identifier(Identifier::new(ident.node.range.clone(), ident.name.clone())))
            }
            Pattern::Literal(lit) => {
                Ok(Expression::Literal(Literal::new(lit.node.range.clone(), lit.value.clone(), "unknown".to_string())))
            }
            _ => Err(CompileError::Parse(self.error("Cannot convert complex pattern to expression"))),
        }
    }
}

/// Niveles de precedencia para operadores.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Or,          // ||
    And,         // &&
    Equality,    // == !=
    Comparison,  // < > <= >=
    Term,        // + -
    Factor,      // * / %
    Member,      // .
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_empty_program() {
        let tokens = vec![Token::new(TokenKind::EOF, "".to_string(), Range::new(Position::new(1, 1), Position::new(1, 1)))];
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.imports.len(), 0);
        assert_eq!(program.declarations.len(), 0);
    }

    #[test]
    fn test_parse_function_declaration() {
        let source = "fn add(a: Number, b: Number) -> Number { return a + b; }";
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = match lexer.tokenize() {
            Ok(result) => result.tokens,
            Err(e) => panic!("Lexer failed: {:?}", e),
        };
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        if result.is_err() {
            println!("Parse error: {:?}", result.as_ref().err());
        }
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Function(func) => {
                assert_eq!(func.name, "add");
                assert_eq!(func.parameters.len(), 2);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let source = "1 + 2 * 3";
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = match lexer.tokenize() {
            Ok(result) => result.tokens,
            Err(_) => panic!("Lexer failed"),
        };
        let mut parser = Parser::new(tokens);
        let result = parser.parse_expression();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_literal_expressions() {
        let test_cases = vec![
            ("42", "number"),
            ("\"hello\"", "string"),
            ("true", "bool"),
            ("false", "bool"),
        ];

        for (source, expected_kind) in test_cases {
            let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
            let tokens = match lexer.tokenize() {
                Ok(result) => result.tokens,
                Err(_) => panic!("Lexer failed for: {}", source),
            };
            let mut parser = Parser::new(tokens);
            let result = parser.parse_expression();
            assert!(result.is_ok(), "Failed to parse: {}", source);
            match result.unwrap() {
                Expression::Literal(lit) => assert_eq!(lit.kind, expected_kind),
                _ => panic!("Expected literal for: {}", source),
            }
        }
    }

    #[test]
    fn test_parse_identifier() {
        let source = "variable";
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = match lexer.tokenize() {
            Ok(result) => result.tokens,
            Err(_) => panic!("Lexer failed"),
        };
        let mut parser = Parser::new(tokens);
        let result = parser.parse_expression();
        assert!(result.is_ok());
        match result.unwrap() {
            Expression::Identifier(ident) => assert_eq!(ident.name, "variable"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_parse_parenthesized_expression() {
        let source = "(1 + 2)";
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = match lexer.tokenize() {
            Ok(result) => result.tokens,
            Err(_) => panic!("Lexer failed"),
        };
        let mut parser = Parser::new(tokens);
        let result = parser.parse_expression();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_unary_expression() {
        let sources = vec!["-42", "!true"];

        for source in sources {
            let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
            let tokens = match lexer.tokenize() {
                Ok(result) => result.tokens,
                Err(_) => panic!("Lexer failed for: {}", source),
            };
            let mut parser = Parser::new(tokens);
            let result = parser.parse_expression();
            assert!(result.is_ok(), "Failed to parse unary: {}", source);
            match result.unwrap() {
                Expression::Unary(_) => {} // Success
                _ => panic!("Expected unary expression for: {}", source),
            }
        }
    }

    #[test]
    fn test_parse_error_recovery() {
        let source = "fn invalid {";
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = match lexer.tokenize() {
            Ok(result) => result.tokens,
            Err(_) => panic!("Lexer failed"),
        };
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        // Should fail gracefully
        assert!(result.is_err());
    }

    // ===================================================================
    // TESTS PARA PATTERN DESTRUCTURING (TASK-117A)
    // ===================================================================

    #[test]
    fn test_parse_array_pattern_simple() {
        let source = "[x, y, z]";
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = lexer.tokenize().unwrap().tokens;
        let mut parser = Parser::new(tokens);

        // Parse manually since we don't have match expressions yet
        let pattern = parser.parse_array_pattern().unwrap();

        match pattern {
            Pattern::Array(array_pattern) => {
                assert_eq!(array_pattern.elements.len(), 3);
                // Check that elements are identifier patterns
                for element in &array_pattern.elements {
                    match element {
                        ArrayPatternElement::Pattern(Pattern::Identifier(_)) => {}
                        _ => panic!("Expected identifier pattern"),
                    }
                }
            }
            _ => panic!("Expected array pattern"),
        }
    }

    #[test]
    fn test_parse_array_pattern_with_spread() {
        let source = "[first, ...rest]";
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = lexer.tokenize().unwrap().tokens;
        let mut parser = Parser::new(tokens);

        let pattern = parser.parse_array_pattern().unwrap();

        match pattern {
            Pattern::Array(array_pattern) => {
                assert_eq!(array_pattern.elements.len(), 2);
                // First element should be normal pattern
                match &array_pattern.elements[0] {
                    ArrayPatternElement::Pattern(Pattern::Identifier(id)) => {
                        assert_eq!(id.name, "first");
                    }
                    _ => panic!("Expected identifier pattern for first element"),
                }
                // Second element should be rest pattern
                match &array_pattern.elements[1] {
                    ArrayPatternElement::Rest(Pattern::Identifier(id)) => {
                        assert_eq!(id.name, "rest");
                    }
                    _ => panic!("Expected rest pattern for second element"),
                }
            }
            _ => panic!("Expected array pattern"),
        }
    }

    #[test]
    fn test_parse_tuple_pattern() {
        let source = "(x, y, z)";
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = lexer.tokenize().unwrap().tokens;
        let mut parser = Parser::new(tokens);

        let pattern = parser.parse_tuple_pattern().unwrap();

        match pattern {
            Pattern::Tuple(tuple_pattern) => {
                assert_eq!(tuple_pattern.elements.len(), 3);
            }
            _ => panic!("Expected tuple pattern"),
        }
    }

    #[test]
    fn test_parse_struct_pattern_simple() {
        let source = "User { name, age }";
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = lexer.tokenize().unwrap().tokens;
        let mut parser = Parser::new(tokens);

        let pattern = parser.parse_struct_pattern().unwrap();

        match pattern {
            Pattern::Struct(struct_pattern) => {
                assert_eq!(struct_pattern.struct_name, "User");
                assert_eq!(struct_pattern.fields.len(), 2);
                assert_eq!(struct_pattern.has_rest, false);
                assert_eq!(struct_pattern.fields[0].name, "name");
                assert_eq!(struct_pattern.fields[1].name, "age");
            }
            _ => panic!("Expected struct pattern"),
        }
    }

    #[test]
    fn test_parse_struct_pattern_with_rest() {
        let source = "User { name, age, ...others }";
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = lexer.tokenize().unwrap().tokens;
        let mut parser = Parser::new(tokens);

        let pattern = parser.parse_struct_pattern().unwrap();

        match pattern {
            Pattern::Struct(struct_pattern) => {
                assert_eq!(struct_pattern.struct_name, "User");
                assert_eq!(struct_pattern.fields.len(), 2);
                assert_eq!(struct_pattern.has_rest, true);
            }
            _ => panic!("Expected struct pattern"),
        }
    }

    #[test]
    fn test_parse_match_statement_with_guards() {
        let source = r#"
match value {
    1 if value > 0 => "positive one"
    2 if value < 10 => "small two"
    x if x % 2 == 0 => "even"
    _ => "other"
}
"#.trim();
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = lexer.tokenize().unwrap().tokens;
        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().unwrap();

        match statement {
            Statement::Match(match_stmt) => {
                assert_eq!(match_stmt.arms.len(), 4);

                // First arm: 1 if value > 0 => "positive one"
                let arm1 = &match_stmt.arms[0];
                match &arm1.pattern {
                    Pattern::Literal(lit) => assert_eq!(lit.value, serde_json::json!(1.0)),
                    _ => panic!("Expected literal pattern"),
                }
                assert!(arm1.guard.is_some());
                match arm1.body {
                    Statement::Expression(ref expr_stmt) => {
                        match &expr_stmt.expression {
                            Expression::Literal(lit) => assert_eq!(lit.value, serde_json::json!("positive one")),
                            _ => panic!("Expected literal expression"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }

                // Second arm: 2 if value < 10 => "small two"
                let arm2 = &match_stmt.arms[1];
                match &arm2.pattern {
                    Pattern::Literal(lit) => assert_eq!(lit.value, serde_json::json!(2.0)),
                    _ => panic!("Expected literal pattern"),
                }
                assert!(arm2.guard.is_some());

                // Third arm: x if x % 2 == 0 => "even"
                let arm3 = &match_stmt.arms[2];
                match &arm3.pattern {
                    Pattern::Identifier(id) => assert_eq!(id.name, "x"),
                    _ => panic!("Expected identifier pattern"),
                }
                assert!(arm3.guard.is_some());

                // Fourth arm: _ => "other" (no guard)
                let arm4 = &match_stmt.arms[3];
                match &arm4.pattern {
                    Pattern::Wildcard(_) => {},
                    _ => panic!("Expected wildcard pattern"),
                }
                assert!(arm4.guard.is_none());
            }
            _ => panic!("Expected match statement"),
        }
    }

    #[test]
    fn test_parse_match_statement_complex_guards() {
        let source = r#"
match result {
    Ok(value) if value > 100 => "big success"
    Ok(value) if value > 0 => "small success"
    Err(error) if error.code == 404 => "not found"
    Err(_) => "error"
    _ => "unknown"
}
"#.trim();
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = lexer.tokenize().unwrap().tokens;
        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().unwrap();

        match statement {
            Statement::Match(match_stmt) => {
                assert_eq!(match_stmt.arms.len(), 5);

                // First arm: Ok(value) if value > 100 => "big success"
                let arm1 = &match_stmt.arms[0];
                match &arm1.pattern {
                    Pattern::Enum(enum_pattern) => {
                        assert_eq!(enum_pattern.variant_name, "Ok");
                        assert!(enum_pattern.inner_patterns.is_some());
                        assert_eq!(enum_pattern.inner_patterns.as_ref().unwrap().len(), 1);
                    }
                    _ => panic!("Expected enum pattern"),
                }
                assert!(arm1.guard.is_some());

                // Second arm: Ok(value) if value > 0 => "small success"
                let arm2 = &match_stmt.arms[1];
                match &arm2.pattern {
                    Pattern::Enum(enum_pattern) => {
                        assert_eq!(enum_pattern.variant_name, "Ok");
                        assert!(enum_pattern.inner_patterns.is_some());
                        assert_eq!(enum_pattern.inner_patterns.as_ref().unwrap().len(), 1);
                    }
                    _ => panic!("Expected enum pattern"),
                }
                assert!(arm2.guard.is_some());

                // Third arm: Err(error) if error.code == 404 => "not found"
                let arm3 = &match_stmt.arms[2];
                match &arm3.pattern {
                    Pattern::Enum(enum_pattern) => {
                        assert_eq!(enum_pattern.variant_name, "Err");
                        assert!(enum_pattern.inner_patterns.is_some());
                        assert_eq!(enum_pattern.inner_patterns.as_ref().unwrap().len(), 1);
                    }
                    _ => panic!("Expected enum pattern"),
                }
                assert!(arm3.guard.is_some());

                // Fourth arm: Err(_) => "error" (no guard)
                let arm4 = &match_stmt.arms[3];
                match &arm4.pattern {
                    Pattern::Enum(enum_pattern) => {
                        assert_eq!(enum_pattern.variant_name, "Err");
                        assert!(enum_pattern.inner_patterns.is_some());
                        assert_eq!(enum_pattern.inner_patterns.as_ref().unwrap().len(), 1);
                        match &enum_pattern.inner_patterns.as_ref().unwrap()[0] {
                            Pattern::Wildcard(_) => {},
                            _ => panic!("Expected wildcard in enum field"),
                        }
                    }
                    _ => panic!("Expected enum pattern"),
                }
                assert!(arm4.guard.is_none());

                // Fifth arm: _ => "unknown" (no guard)
                let arm5 = &match_stmt.arms[4];
                match &arm5.pattern {
                    Pattern::Wildcard(_) => {},
                    _ => panic!("Expected wildcard pattern"),
                }
                assert!(arm5.guard.is_none());
            }
            _ => panic!("Expected match statement"),
        }
    }

    #[test]
    fn test_parse_match_statement_with_block_body() {
        let source = r#"
match status {
    "active" if user.is_admin => {
        println("Admin user is active")
        return true
    }
    "inactive" => {
        println("User is inactive")
        return false
    }
    _ => {
        println("Unknown status")
        return false
    }
}
"#.trim();
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = lexer.tokenize().unwrap().tokens;
        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().unwrap();

        match statement {
            Statement::Match(match_stmt) => {
                assert_eq!(match_stmt.arms.len(), 3);

                // First arm has guard and block body
                let arm1 = &match_stmt.arms[0];
                assert!(arm1.guard.is_some());
                match arm1.body {
                    Statement::Block(_) => {}, // Block statement
                    _ => panic!("Expected block statement"),
                }

                // Second arm has no guard but block body
                let arm2 = &match_stmt.arms[1];
                assert!(arm2.guard.is_none());
                match arm2.body {
                    Statement::Block(_) => {}, // Block statement
                    _ => panic!("Expected block statement"),
                }

                // Third arm has no guard but block body
                let arm3 = &match_stmt.arms[2];
                assert!(arm3.guard.is_none());
                match arm3.body {
                    Statement::Block(_) => {}, // Block statement
                    _ => panic!("Expected block statement"),
                }
            }
            _ => panic!("Expected match statement"),
        }
    }

    #[test]
    fn test_lexer_double_arrow() {
        let source = "=>";
        let mut lexer = Lexer::new(source, &std::path::PathBuf::from("test.vela"));
        let tokens = lexer.tokenize().unwrap().tokens;

        assert_eq!(tokens.len(), 2); // DoubleArrow and EOF
        assert_eq!(tokens[0].kind, TokenKind::DoubleArrow);
    }
}