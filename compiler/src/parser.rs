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
        let alias = if self.check(TokenKind::At) {
            self.advance();
            Some(self.consume_identifier()?)
        } else {
            None
        };

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        Ok(ImportDeclaration::new(range, kind, path, alias, None, None))
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

        if self.check(TokenKind::Fn) {
            println!("🔧 Found fn token, parsing function declaration");
            self.parse_function_declaration(is_public)
        } else if self.check(TokenKind::Struct) {
            self.parse_struct_declaration(is_public)
        } else if self.check(TokenKind::Enum) {
            self.parse_enum_declaration(is_public)
        } else if self.check(TokenKind::Type) {
            self.parse_type_alias_declaration(is_public)
        } else if self.check(TokenKind::State) {
            // Variable declarations at top level
            let var_decl = self.parse_variable_declaration()?;
            Ok(Declaration::Variable(var_decl))
        } else {
            println!("🔧 No valid declaration token found, current token: {:?}", self.current_token());
            Err(CompileError::Parse(self.error("Expected declaration")))
        }
    }

    /// Parsea una declaración de función.
    fn parse_function_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
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
            parameters,
            return_type,
            body,
            is_async,
            generic_params,
        );

        Ok(Declaration::Function(func))
    }

    /// Parsea una declaración de struct.
    fn parse_struct_declaration(&mut self, is_public: bool) -> CompileResult<Declaration> {
        let start_pos = self.current_token().range.start.clone();

        self.consume(TokenKind::Struct)?;
        let name = self.consume_identifier()?;
        let generic_params = self.parse_generic_parameters()?;
        self.consume(TokenKind::LeftBrace)?;
        let fields = self.parse_struct_fields()?;
        self.consume(TokenKind::RightBrace)?;

        let end_pos = self.previous_token().range.end.clone();
        let range = Range::new(start_pos, end_pos);

        let struct_decl = StructDeclaration::new(range, is_public, name, fields, generic_params);
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
            let name = self.consume_identifier()?;
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
                self.tokens[self.current - name.len() - 1].range.start.clone(),
                self.previous_token().range.end.clone(),
            );

            parameters.push(Parameter::new(name, type_annotation, default_value, range));

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
        println!("🔧 parse_precedence called with precedence: {:?}", precedence);
        let mut left = self.parse_unary()?;
        println!("🔧 parse_precedence: parsed left expression: {:?}", left);

        while let Some(op_precedence) = self.get_precedence(self.current_token_kind()) {
            println!("🔧 parse_precedence: found operator {:?} with precedence {:?}", self.current_token_kind(), op_precedence);
            if op_precedence <= precedence {
                break;
            }

            let operator = self.current_token_kind();
            self.advance();

            let right = self.parse_precedence(op_precedence)?;
            left = self.create_binary_expr(left, operator, right);
        }

        Ok(left)
    }

    /// Parsea expresión unaria.
    fn parse_unary(&mut self) -> CompileResult<Expression> {
        println!("🔧 parse_unary called, current token: {:?}", self.current_token_kind());
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
                    println!("🔧 parse_primary: found function call for '{}'", name);
                    self.advance(); // consume '('
                    let mut arguments = Vec::new();
                    
                    // Parse arguments
                    if !self.check(TokenKind::RightParen) {
                        loop {
                            println!("🔧 parse_primary: parsing argument");
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
                let expr = self.parse_expression()?;
                self.consume(TokenKind::RightParen)?;
                Ok(expr)
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
    fn get_precedence(&self, kind: TokenKind) -> Option<Precedence> {
        match kind {
            TokenKind::Or => Some(Precedence::Or),
            TokenKind::And => Some(Precedence::And),
            TokenKind::Equal | TokenKind::NotEqual => Some(Precedence::Equality),
            TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => Some(Precedence::Comparison),
            TokenKind::Plus | TokenKind::Minus => Some(Precedence::Term),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(Precedence::Factor),
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
}