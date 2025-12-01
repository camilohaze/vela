"""
Tests unitarios para parsing de module y decoradores.

Jira: VELA-571
Sprint: Sprint 9
Task: TASK-016H.3
"""

import pytest
from src.lexer.lexer import Lexer
from src.parser.parser import Parser
from src.parser.ast_nodes import (
    ModuleDeclaration,
    Decorator,
    StructLiteral,
    ArrayLiteral,
    Identifier,
    Literal
)


class TestModuleParsing:
    """Tests para parsing de module declarations con decoradores."""
    
    def parse_code(self, code: str):
        """Helper para parsear código Vela."""
        lexer = Lexer(code, "test.vela")
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        return parser.parse()
    
    def test_module_simple_empty(self):
        """Test de module vacío con @module decorator."""
        code = """
        @module({
          declarations: [],
          exports: []
        })
        module EmptyModule {
        }
        """
        
        ast = self.parse_code(code)
        assert len(ast.declarations) == 1
        
        module = ast.declarations[0]
        assert isinstance(module, ModuleDeclaration)
        assert module.name == "EmptyModule"
        assert len(module.decorators) == 1
        assert module.decorators[0].name == "module"
    
    def test_module_with_declarations(self):
        """Test de module con declarations."""
        code = """
        @module({
          declarations: [UserService, ProductService],
          exports: [UserService]
        })
        module AppModule {
        }
        """
        
        ast = self.parse_code(code)
        module = ast.declarations[0]
        
        assert isinstance(module, ModuleDeclaration)
        assert module.name == "AppModule"
        
        # Verificar metadata extraída
        assert len(module.declarations) == 2
        assert isinstance(module.declarations[0], Identifier)
        assert module.declarations[0].name == "UserService"
        assert module.declarations[1].name == "ProductService"
        
        assert len(module.exports) == 1
        assert module.exports[0].name == "UserService"
    
    def test_module_with_providers(self):
        """Test de module con providers."""
        code = """
        @module({
          declarations: [AuthService, TokenService],
          exports: [AuthService],
          providers: [AuthService, DatabaseConnection]
        })
        module AuthModule {
        }
        """
        
        ast = self.parse_code(code)
        module = ast.declarations[0]
        
        assert len(module.providers) == 2
        assert module.providers[0].name == "AuthService"
        assert module.providers[1].name == "DatabaseConnection"
    
    def test_module_with_imports(self):
        """Test de module con imports (string literals)."""
        code = """
        @module({
          declarations: [UserComponent],
          exports: [UserComponent],
          imports: ['system:http', 'module:shared']
        })
        module UserModule {
        }
        """
        
        ast = self.parse_code(code)
        module = ast.declarations[0]
        
        assert len(module.imports) == 2
        assert isinstance(module.imports[0], Literal)
        assert module.imports[0].value == "system:http"
        assert module.imports[1].value == "module:shared"
    
    def test_module_with_body(self):
        """Test de module con body (declaraciones internas)."""
        code = """
        @module({
          declarations: [MyService],
          exports: [MyService]
        })
        module MyModule {
          fn helperFunction() -> void {
            # Internal helper
          }
        }
        """
        
        ast = self.parse_code(code)
        module = ast.declarations[0]
        
        assert len(module.body) == 1  # helperFunction
    
    def test_module_complete_metadata(self):
        """Test de module con todos los campos de metadata."""
        code = """
        @module({
          declarations: [Service1, Service2, Widget1],
          exports: [Service1, Widget1],
          providers: [Service1, DatabaseConnection],
          imports: ['system:ui', 'package:axios', 'module:core']
        })
        module CompleteModule {
        }
        """
        
        ast = self.parse_code(code)
        module = ast.declarations[0]
        
        # Verificar todas las listas
        assert len(module.declarations) == 3
        assert len(module.exports) == 2
        assert len(module.providers) == 2
        assert len(module.imports) == 3
        
        # Verificar tipos
        assert all(isinstance(d, Identifier) for d in module.declarations)
        assert all(isinstance(e, Identifier) for e in module.exports)
        assert all(isinstance(p, Identifier) for p in module.providers)
        assert all(isinstance(i, Literal) for i in module.imports)
    
    def test_module_public_modifier(self):
        """Test de module con modificador public."""
        code = """
        @module({
          declarations: [],
          exports: []
        })
        public module PublicModule {
        }
        """
        
        ast = self.parse_code(code)
        module = ast.declarations[0]
        
        assert module.is_public == True
    
    def test_decorator_parsing(self):
        """Test de parsing de decorador @module con object literal."""
        code = """
        @module({
          declarations: [Service1],
          exports: [Service1]
        })
        module TestModule {
        }
        """
        
        ast = self.parse_code(code)
        module = ast.declarations[0]
        
        # Verificar decorador
        assert len(module.decorators) == 1
        decorator = module.decorators[0]
        assert decorator.name == "module"
        
        # Verificar arguments (debe tener el object literal)
        assert len(decorator.arguments) == 1
        assert isinstance(decorator.arguments[0], StructLiteral)
        
        # Verificar fields del object literal
        obj_literal = decorator.arguments[0]
        assert len(obj_literal.fields) == 2  # declarations y exports
        assert obj_literal.fields[0].name == "declarations"
        assert obj_literal.fields[1].name == "exports"


class TestDecoratorParsing:
    """Tests para parsing genérico de decoradores."""
    
    def parse_code(self, code: str):
        """Helper para parsear código Vela."""
        lexer = Lexer(code, "test.vela")
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        return parser.parse()
    
    def test_single_decorator_no_args(self):
        """Test de decorador simple sin argumentos."""
        code = """
        @injectable
        class UserService {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert len(cls.decorators) == 1
        assert cls.decorators[0].name == "injectable"
        assert len(cls.decorators[0].arguments) == 0
    
    def test_single_decorator_with_string_arg(self):
        """Test de decorador con argumento string."""
        code = """
        @controller("/api/users")
        class UserController {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert len(cls.decorators) == 1
        assert cls.decorators[0].name == "controller"
        assert len(cls.decorators[0].arguments) == 1
        assert isinstance(cls.decorators[0].arguments[0], Literal)
        assert cls.decorators[0].arguments[0].value == "/api/users"
    
    def test_multiple_decorators(self):
        """Test de múltiples decoradores en una declaración."""
        code = """
        @injectable
        @controller("/api")
        @middleware(AuthMiddleware)
        class ApiController {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert len(cls.decorators) == 3
        assert cls.decorators[0].name == "injectable"
        assert cls.decorators[1].name == "controller"
        assert cls.decorators[2].name == "middleware"
    
    def test_decorator_with_object_literal(self):
        """Test de decorador con object literal como argumento."""
        code = """
        @validate({ min: 5, max: 100 })
        fn processNumber(num: Number) -> void {
        }
        """
        
        ast = self.parse_code(code)
        func = ast.declarations[0]
        
        assert len(func.decorators) == 1
        decorator = func.decorators[0]
        assert decorator.name == "validate"
        
        # Verificar object literal
        assert len(decorator.arguments) == 1
        assert isinstance(decorator.arguments[0], StructLiteral)
        
        obj = decorator.arguments[0]
        assert len(obj.fields) == 2
        assert obj.fields[0].name == "min"
        assert obj.fields[1].name == "max"


class TestObjectLiteralParsing:
    """Tests para parsing de object literals."""
    
    def parse_expression(self, code: str):
        """Helper para parsear una expresión."""
        lexer = Lexer(code, "test.vela")
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        return parser.parse_expression()
    
    def test_empty_object(self):
        """Test de object literal vacío."""
        expr = self.parse_expression("{}")
        
        assert isinstance(expr, StructLiteral)
        assert len(expr.fields) == 0
    
    def test_object_with_string_values(self):
        """Test de object con valores string."""
        expr = self.parse_expression('{ name: "Alice", email: "alice@example.com" }')
        
        assert isinstance(expr, StructLiteral)
        assert len(expr.fields) == 2
        assert expr.fields[0].name == "name"
        assert expr.fields[1].name == "email"
    
    def test_object_with_number_values(self):
        """Test de object con valores numéricos."""
        expr = self.parse_expression("{ age: 25, score: 95.5 }")
        
        assert isinstance(expr, StructLiteral)
        assert len(expr.fields) == 2
        assert isinstance(expr.fields[0].value, Literal)
        assert expr.fields[0].value.value == 25
        assert expr.fields[1].value.value == 95.5
    
    def test_object_with_array_values(self):
        """Test de object con arrays como valores."""
        expr = self.parse_expression("{ items: [1, 2, 3], names: ['Alice', 'Bob'] }")
        
        assert isinstance(expr, StructLiteral)
        assert len(expr.fields) == 2
        
        # Verificar arrays
        assert isinstance(expr.fields[0].value, ArrayLiteral)
        assert len(expr.fields[0].value.elements) == 3
        
        assert isinstance(expr.fields[1].value, ArrayLiteral)
        assert len(expr.fields[1].value.elements) == 2
    
    def test_object_with_identifier_values(self):
        """Test de object con identificadores como valores."""
        expr = self.parse_expression("{ service: UserService, repository: UserRepo }")
        
        assert isinstance(expr, StructLiteral)
        assert len(expr.fields) == 2
        
        assert isinstance(expr.fields[0].value, Identifier)
        assert expr.fields[0].value.name == "UserService"
        
        assert isinstance(expr.fields[1].value, Identifier)
        assert expr.fields[1].value.name == "UserRepo"
    
    def test_object_trailing_comma(self):
        """Test de object con trailing comma."""
        expr = self.parse_expression("{ a: 1, b: 2, }")
        
        assert isinstance(expr, StructLiteral)
        assert len(expr.fields) == 2


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
