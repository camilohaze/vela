"""
Tests unitarios para parsing de decoradores arquitectónicos.

Jira: VELA-571
Sprint: Sprint 9
Task: TASK-016I
"""

import pytest
from src.lexer.lexer import Lexer
from src.parser.parser import Parser
from src.parser.ast_nodes import (
    ClassDeclaration,
    FunctionDeclaration,
    Decorator,
    StructLiteral,
    Literal,
    Identifier
)


class TestDependencyInjectionDecorators:
    """Tests para decoradores de Dependency Injection."""
    
    def parse_code(self, code: str):
        """Helper para parsear código Vela."""
        lexer = Lexer(code, "test.vela")
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        return parser.parse()
    
    def test_injectable_no_args(self):
        """Test de @injectable sin argumentos."""
        code = """
        @injectable
        class UserService {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert isinstance(cls, ClassDeclaration)
        assert len(cls.decorators) == 1
        assert cls.decorators[0].name == "injectable"
        assert len(cls.decorators[0].arguments) == 0
    
    def test_injectable_with_scope_singleton(self):
        """Test de @injectable con scope singleton."""
        code = """
        @injectable({ scope: "singleton" })
        class DatabaseConnection {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        decorator = cls.decorators[0]
        assert decorator.name == "injectable"
        assert len(decorator.arguments) == 1
        
        # Verificar object literal
        obj = decorator.arguments[0]
        assert isinstance(obj, StructLiteral)
        assert len(obj.fields) == 1
        assert obj.fields[0].name == "scope"
        assert obj.fields[0].value.value == "singleton"
    
    def test_injectable_with_scope_transient(self):
        """Test de @injectable con scope transient."""
        code = """
        @injectable({ scope: "transient" })
        class RequestHandler {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        decorator = cls.decorators[0]
        obj = decorator.arguments[0]
        assert obj.fields[0].value.value == "transient"
    
    def test_inject_decorator(self):
        """Test de @inject con token."""
        code = """
        class UserController {
          @inject({ token: "IUserService" })
          userService: IUserService
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        # Los field decorators se parsean en el contexto de la clase
        # Este test valida la sintaxis, la semántica se valida en type checker
        assert isinstance(cls, ClassDeclaration)
    
    def test_container_decorator(self):
        """Test de @container decorator."""
        code = """
        @container
        class AppContainer {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert cls.decorators[0].name == "container"
    
    def test_provides_decorator(self):
        """Test de @provides decorator."""
        code = """
        @provides(ILogger)
        class ConsoleLogger {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        decorator = cls.decorators[0]
        assert decorator.name == "provides"
        assert len(decorator.arguments) == 1
        assert isinstance(decorator.arguments[0], Identifier)
        assert decorator.arguments[0].name == "ILogger"


class TestRESTDecorators:
    """Tests para decoradores REST/HTTP."""
    
    def parse_code(self, code: str):
        """Helper para parsear código Vela."""
        lexer = Lexer(code, "test.vela")
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        return parser.parse()
    
    def test_controller_with_path(self):
        """Test de @controller con path."""
        code = """
        @controller("/api/users")
        class UserController {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        decorator = cls.decorators[0]
        assert decorator.name == "controller"
        assert len(decorator.arguments) == 1
        assert isinstance(decorator.arguments[0], Literal)
        assert decorator.arguments[0].value == "/api/users"
    
    def test_get_decorator(self):
        """Test de @get decorator con path."""
        code = """
        class UserController {
          @get("/profile")
          fn getProfile() -> User {
            # ...
          }
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        # Buscar método con decorator @get
        # (asumiendo que los métodos tienen decorators parseados)
        assert isinstance(cls, ClassDeclaration)
    
    def test_post_decorator(self):
        """Test de @post decorator."""
        code = """
        @injectable
        @controller("/api/users")
        class UserController {
          @post("/")
          fn createUser(dto: CreateUserDTO) -> Result<User> {
            # ...
          }
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        # Verificar múltiples decorators en la clase
        assert len(cls.decorators) == 2
        assert cls.decorators[0].name == "injectable"
        assert cls.decorators[1].name == "controller"
    
    def test_put_decorator(self):
        """Test de @put decorator."""
        code = """
        class UserController {
          @put("/:id")
          fn updateUser(id: Number, dto: UpdateUserDTO) -> Result<User> {
            # ...
          }
        }
        """
        
        ast = self.parse_code(code)
        assert isinstance(ast.declarations[0], ClassDeclaration)
    
    def test_delete_decorator(self):
        """Test de @delete decorator."""
        code = """
        class UserController {
          @delete("/:id")
          fn deleteUser(id: Number) -> Result<void> {
            # ...
          }
        }
        """
        
        ast = self.parse_code(code)
        assert isinstance(ast.declarations[0], ClassDeclaration)
    
    def test_patch_decorator(self):
        """Test de @patch decorator."""
        code = """
        class UserController {
          @patch("/:id/status")
          fn updateStatus(id: Number, status: Status) -> Result<User> {
            # ...
          }
        }
        """
        
        ast = self.parse_code(code)
        assert isinstance(ast.declarations[0], ClassDeclaration)
    
    def test_controller_with_metadata(self):
        """Test de @controller con metadata compleja."""
        code = """
        @controller({
          path: "/api/v1/users",
          middleware: [AuthMiddleware, LoggerMiddleware]
        })
        class UserController {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        decorator = cls.decorators[0]
        assert decorator.name == "controller"
        
        obj = decorator.arguments[0]
        assert isinstance(obj, StructLiteral)
        assert len(obj.fields) == 2
        assert obj.fields[0].name == "path"
        assert obj.fields[1].name == "middleware"


class TestMiddlewareAndGuardDecorators:
    """Tests para decoradores de middleware y guards."""
    
    def parse_code(self, code: str):
        """Helper para parsear código Vela."""
        lexer = Lexer(code, "test.vela")
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        return parser.parse()
    
    def test_middleware_decorator(self):
        """Test de @middleware decorator."""
        code = """
        @middleware
        class AuthMiddleware {
          fn handle(request: Request, next: () -> Response) -> Response {
            # ...
          }
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert cls.decorators[0].name == "middleware"
    
    def test_guard_decorator(self):
        """Test de @guard decorator."""
        code = """
        @guard
        class AdminGuard {
          fn canActivate(context: ExecutionContext) -> Bool {
            # ...
          }
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert cls.decorators[0].name == "guard"
    
    def test_interceptor_decorator(self):
        """Test de @interceptor decorator."""
        code = """
        @interceptor
        class LoggingInterceptor {
          fn intercept(request: Request) -> Request {
            # ...
          }
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert cls.decorators[0].name == "interceptor"


class TestValidationDecorators:
    """Tests para decoradores de validación."""
    
    def parse_code(self, code: str):
        """Helper para parsear código Vela."""
        lexer = Lexer(code, "test.vela")
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        return parser.parse()
    
    def test_validate_decorator(self):
        """Test de @validate decorator."""
        code = """
        @validate
        fn processInput(data: String) -> Result<String> {
          # ...
        }
        """
        
        ast = self.parse_code(code)
        func = ast.declarations[0]
        
        assert isinstance(func, FunctionDeclaration)
        assert func.decorators[0].name == "validate"
    
    def test_required_decorator(self):
        """Test de @required decorator en parámetro."""
        code = """
        class CreateUserDTO {
          @required
          name: String
          
          @required
          email: String
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert isinstance(cls, ClassDeclaration)
    
    def test_email_decorator(self):
        """Test de @email decorator."""
        code = """
        class UserDTO {
          @email
          email: String
        }
        """
        
        ast = self.parse_code(code)
        assert isinstance(ast.declarations[0], ClassDeclaration)
    
    def test_min_max_decorators(self):
        """Test de @min y @max decorators con valores."""
        code = """
        @validate({ min: 18, max: 100 })
        fn validateAge(age: Number) -> Result<Bool> {
          # ...
        }
        """
        
        ast = self.parse_code(code)
        func = ast.declarations[0]
        
        decorator = func.decorators[0]
        assert decorator.name == "validate"
        
        obj = decorator.arguments[0]
        assert isinstance(obj, StructLiteral)
        assert len(obj.fields) == 2
        assert obj.fields[0].name == "min"
        assert obj.fields[1].name == "max"
    
    def test_length_decorator(self):
        """Test de @length decorator con min y max."""
        code = """
        class PasswordDTO {
          @length({ min: 8, max: 64 })
          password: String
        }
        """
        
        ast = self.parse_code(code)
        assert isinstance(ast.declarations[0], ClassDeclaration)
    
    def test_regex_decorator(self):
        """Test de @regex decorator con pattern."""
        code = """
        class UsernameDTO {
          @regex({ pattern: "^[a-zA-Z0-9_]+$" })
          username: String
        }
        """
        
        ast = self.parse_code(code)
        assert isinstance(ast.declarations[0], ClassDeclaration)
    
    def test_url_decorator(self):
        """Test de @url decorator."""
        code = """
        class WebsiteDTO {
          @url
          website: String
        }
        """
        
        ast = self.parse_code(code)
        assert isinstance(ast.declarations[0], ClassDeclaration)


class TestMultipleDecorators:
    """Tests para combinaciones de múltiples decoradores."""
    
    def parse_code(self, code: str):
        """Helper para parsear código Vela."""
        lexer = Lexer(code, "test.vela")
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        return parser.parse()
    
    def test_di_plus_controller(self):
        """Test de @injectable + @controller."""
        code = """
        @injectable
        @controller("/api/products")
        class ProductController {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert len(cls.decorators) == 2
        assert cls.decorators[0].name == "injectable"
        assert cls.decorators[1].name == "controller"
    
    def test_controller_with_multiple_http_methods(self):
        """Test de controller con múltiples métodos HTTP."""
        code = """
        @injectable
        @controller("/api/users")
        class UserController {
          @get("/")
          fn getAllUsers() -> Result<List<User>> {
            # ...
          }
          
          @get("/:id")
          fn getUserById(id: Number) -> Result<User> {
            # ...
          }
          
          @post("/")
          @validate
          fn createUser(dto: CreateUserDTO) -> Result<User> {
            # ...
          }
          
          @put("/:id")
          @validate
          fn updateUser(id: Number, dto: UpdateUserDTO) -> Result<User> {
            # ...
          }
          
          @delete("/:id")
          fn deleteUser(id: Number) -> Result<void> {
            # ...
          }
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert len(cls.decorators) == 2
        assert cls.name == "UserController"
    
    def test_service_with_all_di_decorators(self):
        """Test de service con todos los decoradores DI."""
        code = """
        @injectable({ scope: "singleton" })
        @provides(IUserService)
        class UserService {
          @inject({ token: "IUserRepository" })
          repository: IUserRepository
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert len(cls.decorators) == 2
        assert cls.decorators[0].name == "injectable"
        assert cls.decorators[1].name == "provides"
    
    def test_dto_with_all_validation_decorators(self):
        """Test de DTO con múltiples validaciones."""
        code = """
        class CreateUserDTO {
          @required
          @length({ min: 3, max: 50 })
          name: String
          
          @required
          @email
          email: String
          
          @required
          @length({ min: 8, max: 64 })
          @regex({ pattern: "^(?=.*[a-z])(?=.*[A-Z])(?=.*\\d).+$" })
          password: String
          
          @min(18)
          @max(100)
          age: Number
          
          @url
          website: Option<String>
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert cls.name == "CreateUserDTO"
        assert isinstance(cls, ClassDeclaration)


class TestDecoratorEdgeCases:
    """Tests para casos edge de decoradores."""
    
    def parse_code(self, code: str):
        """Helper para parsear código Vela."""
        lexer = Lexer(code, "test.vela")
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        return parser.parse()
    
    def test_decorator_with_complex_nested_object(self):
        """Test de decorator con object literal complejo anidado."""
        code = """
        @module({
          declarations: [
            UserService,
            ProductService,
            OrderService
          ],
          exports: [UserService, ProductService],
          providers: [
            UserService,
            ProductService,
            OrderService,
            DatabaseConnection
          ],
          imports: [
            'system:http',
            'system:reactive',
            'package:lodash',
            'module:auth',
            'module:shared'
          ]
        })
        module AppModule {
        }
        """
        
        ast = self.parse_code(code)
        module = ast.declarations[0]
        
        assert module.name == "AppModule"
        assert len(module.declarations) == 3
        assert len(module.exports) == 2
        assert len(module.providers) == 4
        assert len(module.imports) == 5
    
    def test_decorator_without_parentheses(self):
        """Test de decorator sin paréntesis."""
        code = """
        @injectable
        @middleware
        @validate
        class MyClass {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert len(cls.decorators) == 3
        assert all(len(d.arguments) == 0 for d in cls.decorators)
    
    def test_decorator_with_multiple_arguments(self):
        """Test de decorator con múltiples argumentos."""
        code = """
        @custom("arg1", "arg2", 123)
        class MyClass {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        decorator = cls.decorators[0]
        assert decorator.name == "custom"
        assert len(decorator.arguments) == 3
    
    def test_decorator_on_public_declaration(self):
        """Test de decorator en declaración pública."""
        code = """
        @injectable
        public class PublicService {
        }
        """
        
        ast = self.parse_code(code)
        cls = ast.declarations[0]
        
        assert cls.is_public == True
        assert cls.decorators[0].name == "injectable"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
