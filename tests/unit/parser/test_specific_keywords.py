"""
Tests para Keywords Específicos del Parser de Vela

Tests de: VELA-569 (TASK-012B)
Historia: Sprint 7 - Keywords específicos por tipo
Fecha: 2025-12-01

Este módulo testea el parsing de los 30 keywords domain-specific:
- UI: widget, component
- Models: model
- Patterns: factory, builder, strategy, observer, singleton, adapter, decorator
- Web/API: guard, middleware, interceptor, validator
- State & DI: store, provider
- Concurrency: actor
- Utilities: pipe, task, helper, mapper, serializer
"""

import pytest
import sys
sys.path.append('../..')

from src.parser import parse_code
from src.parser.ast_nodes import (
    # UI Keywords
    WidgetDeclaration, ComponentDeclaration,
    # Model Keywords
    ModelDeclaration,
    # Design Pattern Keywords
    FactoryDeclaration, BuilderDeclaration, StrategyDeclaration,
    ObserverDeclaration, SingletonDeclaration, AdapterDeclaration,
    DecoratorDeclaration,
    # Web/API Keywords
    GuardDeclaration, MiddlewareDeclaration, InterceptorDeclaration,
    ValidatorDeclaration,
    # State & DI Keywords
    StoreDeclaration, ProviderDeclaration,
    # Concurrency Keywords
    ActorDeclaration,
    # Utility Keywords
    PipeDeclaration, TaskDeclaration, HelperDeclaration,
    MapperDeclaration, SerializerDeclaration
)


# ===================================================================
# UI KEYWORDS
# ===================================================================

class TestUIKeywords:
    """Tests para keywords de UI (widget, component)"""
    
    def test_widget_declaration(self):
        """Test widget keyword - Stateful widget"""
        code = """
        widget Counter {
            state count: Number = 0
            
            fn increment() -> void {
                this.count = this.count + 1
            }
            
            fn build() -> Widget {
                return Text("Count: ${this.count}")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], WidgetDeclaration)
        assert ast.declarations[0].name == "Counter"
        assert len(ast.declarations[0].fields) == 1  # state count
        assert len(ast.declarations[0].methods) == 2  # increment, build
    
    def test_component_declaration(self):
        """Test component keyword - UI component"""
        code = """
        component Button {
            text: String
            onClick: () -> void
            
            fn build() -> Widget {
                return Container(child: Text(this.text))
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], ComponentDeclaration)
        assert ast.declarations[0].name == "Button"
        assert len(ast.declarations[0].fields) == 2  # text, onClick
        assert len(ast.declarations[0].methods) == 1  # build


# ===================================================================
# MODEL KEYWORDS
# ===================================================================

class TestModelKeywords:
    """Tests para keywords de modelos (model)"""
    
    def test_model_declaration(self):
        """Test model keyword - Generic model"""
        code = """
        model Product {
            id: Number
            name: String
            price: Float
            inStock: Bool
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], ModelDeclaration)
        assert ast.declarations[0].name == "Product"
        assert len(ast.declarations[0].fields) == 4


# ===================================================================
# DESIGN PATTERN KEYWORDS
# ===================================================================

class TestDesignPatternKeywords:
    """Tests para keywords de design patterns"""
    
    def test_factory_declaration(self):
        """Test factory keyword - Factory pattern"""
        code = """
        factory UserFactory {
            fn create(name: String, email: String) -> User {
                return User(name: name, email: email)
            }
            
            fn createDefault() -> User {
                return User(name: "Guest", email: "guest@example.com")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], FactoryDeclaration)
        assert ast.declarations[0].name == "UserFactory"
        assert len(ast.declarations[0].methods) == 2
    
    def test_builder_declaration(self):
        """Test builder keyword - Builder pattern"""
        code = """
        builder QueryBuilder {
            fn select(fields: String[]) -> Self {
                return this
            }
            
            fn where(condition: String) -> Self {
                return this
            }
            
            fn build() -> Query {
                return Query()
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], BuilderDeclaration)
        assert ast.declarations[0].name == "QueryBuilder"
        assert len(ast.declarations[0].methods) == 3
    
    def test_strategy_declaration(self):
        """Test strategy keyword - Strategy pattern"""
        code = """
        strategy PaymentStrategy {
            fn pay(amount: Float) -> Result<void> {
                return Ok(void)
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], StrategyDeclaration)
        assert ast.declarations[0].name == "PaymentStrategy"
    
    def test_observer_declaration(self):
        """Test observer keyword - Observer pattern"""
        code = """
        observer EventObserver {
            fn notify(event: Event) -> void {
                print("Event received: ${event}")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], ObserverDeclaration)
        assert ast.declarations[0].name == "EventObserver"
    
    def test_singleton_declaration(self):
        """Test singleton keyword - Singleton pattern"""
        code = """
        singleton Database {
            connection: Connection
            
            fn getInstance() -> Database {
                return this
            }
            
            fn query(sql: String) -> Result<Data> {
                return Ok(Data())
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], SingletonDeclaration)
        assert ast.declarations[0].name == "Database"
        assert len(ast.declarations[0].fields) == 1
        assert len(ast.declarations[0].methods) == 2
    
    def test_adapter_declaration(self):
        """Test adapter keyword - Adapter pattern"""
        code = """
        adapter LegacyAdapter {
            fn adapt(oldData: OldFormat) -> NewFormat {
                return NewFormat(data: oldData.value)
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], AdapterDeclaration)
        assert ast.declarations[0].name == "LegacyAdapter"
    
    def test_decorator_declaration(self):
        """Test decorator keyword - Decorator pattern"""
        code = """
        decorator LogDecorator {
            fn wrap(fn: Function) -> Function {
                return (args: Any[]) => {
                    print("Calling function...")
                    result = fn(args)
                    print("Function completed")
                    return result
                }
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], DecoratorDeclaration)
        assert ast.declarations[0].name == "LogDecorator"


# ===================================================================
# WEB/API KEYWORDS
# ===================================================================

class TestWebAPIKeywords:
    """Tests para keywords de Web/API"""
    
    def test_guard_declaration(self):
        """Test guard keyword - Route guard"""
        code = """
        guard AuthGuard {
            fn canActivate(context: Context) -> Bool {
                return context.user.isAuthenticated()
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], GuardDeclaration)
        assert ast.declarations[0].name == "AuthGuard"
    
    def test_middleware_declaration(self):
        """Test middleware keyword - HTTP middleware"""
        code = """
        middleware Logger {
            fn handle(request: Request, next: Function) -> Response {
                print("Request: ${request.path}")
                response = next(request)
                print("Response: ${response.status}")
                return response
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], MiddlewareDeclaration)
        assert ast.declarations[0].name == "Logger"
    
    def test_interceptor_declaration(self):
        """Test interceptor keyword - Request/response interceptor"""
        code = """
        interceptor AuthInterceptor {
            fn intercept(request: Request) -> Request {
                request.headers["Authorization"] = "Bearer ${token}"
                return request
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], InterceptorDeclaration)
        assert ast.declarations[0].name == "AuthInterceptor"
    
    def test_validator_declaration(self):
        """Test validator keyword - Input validator"""
        code = """
        validator EmailValidator {
            fn validate(email: String) -> Bool {
                return email.contains("@") && email.contains(".")
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], ValidatorDeclaration)
        assert ast.declarations[0].name == "EmailValidator"


# ===================================================================
# STATE & DI KEYWORDS
# ===================================================================

class TestStateDIKeywords:
    """Tests para keywords de estado y DI"""
    
    def test_store_declaration(self):
        """Test store keyword - State store"""
        code = """
        store AppStore {
            state count: Number = 0
            state user: Option<User> = None
            
            fn increment() -> void {
                this.count = this.count + 1
            }
            
            fn setUser(user: User) -> void {
                this.user = Some(user)
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], StoreDeclaration)
        assert ast.declarations[0].name == "AppStore"
        assert len(ast.declarations[0].fields) == 2
        assert len(ast.declarations[0].methods) == 2
    
    def test_provider_declaration(self):
        """Test provider keyword - Dependency provider"""
        code = """
        provider ServiceProvider {
            fn provide() -> Service {
                return Service()
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], ProviderDeclaration)
        assert ast.declarations[0].name == "ServiceProvider"


# ===================================================================
# CONCURRENCY KEYWORDS
# ===================================================================

class TestConcurrencyKeywords:
    """Tests para keywords de concurrencia"""
    
    def test_actor_declaration(self):
        """Test actor keyword - Actor model"""
        code = """
        actor Counter {
            state count: Number = 0
            
            fn increment() -> void {
                this.count = this.count + 1
            }
            
            fn getCount() -> Number {
                return this.count
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], ActorDeclaration)
        assert ast.declarations[0].name == "Counter"
        assert len(ast.declarations[0].fields) == 1
        assert len(ast.declarations[0].methods) == 2


# ===================================================================
# UTILITY KEYWORDS
# ===================================================================

class TestUtilityKeywords:
    """Tests para keywords de utilidades"""
    
    def test_pipe_declaration(self):
        """Test pipe keyword - Transformation pipeline"""
        code = """
        pipe TransformPipe {
            fn transform(value: String) -> String {
                return value.toUpperCase()
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], PipeDeclaration)
        assert ast.declarations[0].name == "TransformPipe"
    
    def test_task_declaration(self):
        """Test task keyword - Asynchronous task"""
        code = """
        task EmailTask {
            async fn run() -> Result<void> {
                await sendEmail()
                return Ok(void)
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], TaskDeclaration)
        assert ast.declarations[0].name == "EmailTask"
    
    def test_helper_declaration(self):
        """Test helper keyword - Helper utilities"""
        code = """
        helper DateHelper {
            fn format(date: Date) -> String {
                return date.toString()
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], HelperDeclaration)
        assert ast.declarations[0].name == "DateHelper"
    
    def test_mapper_declaration(self):
        """Test mapper keyword - Object mapper"""
        code = """
        mapper UserMapper {
            fn toDTO(user: User) -> UserDTO {
                return UserDTO(name: user.name, email: user.email)
            }
            
            fn fromDTO(dto: UserDTO) -> User {
                return User(name: dto.name, email: dto.email)
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], MapperDeclaration)
        assert ast.declarations[0].name == "UserMapper"
        assert len(ast.declarations[0].methods) == 2
    
    def test_serializer_declaration(self):
        """Test serializer keyword - Data serializer"""
        code = """
        serializer JsonSerializer {
            fn serialize(data: Any) -> String {
                return JSON.stringify(data)
            }
            
            fn deserialize(json: String) -> Any {
                return JSON.parse(json)
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], SerializerDeclaration)
        assert ast.declarations[0].name == "JsonSerializer"
        assert len(ast.declarations[0].methods) == 2


# ===================================================================
# PUBLIC KEYWORDS
# ===================================================================

class TestPublicKeywords:
    """Tests para keywords con modificador public"""
    
    def test_public_widget(self):
        """Test public widget"""
        code = """
        public widget MyWidget {
            fn build() -> Widget {
                return Container()
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], WidgetDeclaration)
        assert ast.declarations[0].is_public == True
    
    def test_public_factory(self):
        """Test public factory"""
        code = """
        public factory MyFactory {
            fn create() -> Object {
                return Object()
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], FactoryDeclaration)
        assert ast.declarations[0].is_public == True
    
    def test_public_store(self):
        """Test public store"""
        code = """
        public store GlobalStore {
            state data: String = ""
            
            fn setData(value: String) -> void {
                this.data = value
            }
        }
        """
        ast = parse_code(code)
        assert ast is not None
        assert len(ast.declarations) == 1
        assert isinstance(ast.declarations[0], StoreDeclaration)
        assert ast.declarations[0].is_public == True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
