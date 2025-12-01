# TASK-012B: Parser 30 Keywords Específicos

## 📋 Información General
- **Historia:** VELA-569
- **Estado:** Completada ✅
- **Fecha:** 2025-12-01

## 🎯 Objetivo
Implementar parsers para 30 keywords domain-specific que permiten estructurar aplicaciones en Vela siguiendo patrones arquitectónicos estándar. Extender el parser para reconocer keywords especializados agrupados por categoría (UI, patrones de diseño, web/API, estado, concurrencia, utilidades).

## 🔨 Implementación

### Archivos modificados
- `src/parser/parser.py` (líneas agregadas: ~600)
  - Modificado `parse_declaration()` para routing de 23 keywords nuevos
  - Implementados 23 métodos `parse_*_declaration()`

- `src/parser/ast_nodes.py` (líneas agregadas: ~200)
  - Creadas 23 clases AST node correspondientes

### Keywords Implementados (23 nuevos)

#### 1. UI Keywords (2)
```vela
# widget - Stateful widget
widget Counter {
    state count: Number = 0
    
    fn increment() -> void {
        this.count = this.count + 1
    }
    
    fn build() -> Widget {
        return Text("Count: ${this.count}")
    }
}

# component - UI component
component Button {
    text: String
    onClick: () -> void
    
    fn build() -> Widget {
        return Container(child: Text(this.text))
    }
}
```

**Implementación:**
- Parser: `parse_widget_declaration()`, `parse_component_declaration()`
- AST nodes: `WidgetDeclaration`, `ComponentDeclaration`
- Características: Soportan `fields` (incluido `state`) y `methods`

#### 2. Model Keywords (1)
```vela
# model - Generic model
model Product {
    id: Number
    name: String
    price: Float
    inStock: Bool
}
```

**Implementación:**
- Parser: `parse_model_declaration()`
- AST node: `ModelDeclaration`
- Características: Solo soporta `fields` (como struct)

#### 3. Design Pattern Keywords (7)
```vela
# factory - Factory pattern
factory UserFactory {
    fn create(name: String, email: String) -> User {
        return User(name: name, email: email)
    }
}

# builder - Builder pattern
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

# strategy - Strategy pattern
strategy PaymentStrategy {
    fn pay(amount: Float) -> Result<void> {
        return Ok(void)
    }
}

# observer - Observer pattern
observer EventObserver {
    fn notify(event: Event) -> void {
        print("Event received")
    }
}

# singleton - Singleton pattern
singleton Database {
    connection: Connection
    
    fn getInstance() -> Database {
        return this
    }
}

# adapter - Adapter pattern
adapter LegacyAdapter {
    fn adapt(oldData: OldFormat) -> NewFormat {
        return NewFormat(data: oldData.value)
    }
}

# decorator - Decorator pattern
decorator LogDecorator {
    fn wrap(fn: Function) -> Function {
        return (args: Any[]) => {
            print("Calling...")
            result = fn(args)
            return result
        }
    }
}
```

**Implementación:**
- Parsers: `parse_factory_declaration()`, `parse_builder_declaration()`, `parse_strategy_declaration()`, `parse_observer_declaration()`, `parse_singleton_declaration()`, `parse_adapter_declaration()`, `parse_decorator_declaration()`
- AST nodes: `FactoryDeclaration`, `BuilderDeclaration`, `StrategyDeclaration`, `ObserverDeclaration`, `SingletonDeclaration`, `AdapterDeclaration`, `DecoratorDeclaration`
- Características:
  - factory, builder, strategy, observer, adapter, decorator: Solo `methods`
  - singleton: `fields` + `methods`

#### 4. Web/API Keywords (4)
```vela
# guard - Route guard
guard AuthGuard {
    fn canActivate(context: Context) -> Bool {
        return context.user.isAuthenticated()
    }
}

# middleware - HTTP middleware
middleware Logger {
    fn handle(request: Request, next: Function) -> Response {
        print("Request: ${request.path}")
        return next(request)
    }
}

# interceptor - Request/response interceptor
interceptor AuthInterceptor {
    fn intercept(request: Request) -> Request {
        request.headers["Authorization"] = "Bearer ${token}"
        return request
    }
}

# validator - Input validator
validator EmailValidator {
    fn validate(email: String) -> Bool {
        return email.contains("@")
    }
}
```

**Implementación:**
- Parsers: `parse_guard_declaration()`, `parse_middleware_declaration()`, `parse_interceptor_declaration()`, `parse_validator_declaration()`
- AST nodes: `GuardDeclaration`, `MiddlewareDeclaration`, `InterceptorDeclaration`, `ValidatorDeclaration`
- Características: Solo `methods`

#### 5. State & DI Keywords (2)
```vela
# store - State store
store AppStore {
    state count: Number = 0
    state user: Option<User> = None
    
    fn increment() -> void {
        this.count = this.count + 1
    }
}

# provider - Dependency provider
provider ServiceProvider {
    fn provide() -> Service {
        return Service()
    }
}
```

**Implementación:**
- Parsers: `parse_store_declaration()`, `parse_provider_declaration()`
- AST nodes: `StoreDeclaration`, `ProviderDeclaration`
- Características:
  - store: `fields` (incluido `state`) + `methods`
  - provider: Solo `methods`

#### 6. Concurrency Keywords (1)
```vela
# actor - Actor model
actor Counter {
    state count: Number = 0
    
    fn increment() -> void {
        this.count = this.count + 1
    }
    
    fn getCount() -> Number {
        return this.count
    }
}
```

**Implementación:**
- Parser: `parse_actor_declaration()`
- AST node: `ActorDeclaration`
- Características: `fields` (incluido `state`) + `methods`

#### 7. Utility Keywords (5)
```vela
# pipe - Transformation pipeline
pipe TransformPipe {
    fn transform(value: String) -> String {
        return value.toUpperCase()
    }
}

# task - Asynchronous task
task EmailTask {
    async fn run() -> Result<void> {
        await sendEmail()
        return Ok(void)
    }
}

# helper - Helper utilities
helper DateHelper {
    fn format(date: Date) -> String {
        return date.toString()
    }
}

# mapper - Object mapper
mapper UserMapper {
    fn toDTO(user: User) -> UserDTO {
        return UserDTO(name: user.name)
    }
    
    fn fromDTO(dto: UserDTO) -> User {
        return User(name: dto.name)
    }
}

# serializer - Data serializer
serializer JsonSerializer {
    fn serialize(data: Any) -> String {
        return JSON.stringify(data)
    }
    
    fn deserialize(json: String) -> Any {
        return JSON.parse(json)
    }
}
```

**Implementación:**
- Parsers: `parse_pipe_declaration()`, `parse_task_declaration()`, `parse_helper_declaration()`, `parse_mapper_declaration()`, `parse_serializer_declaration()`
- AST nodes: `PipeDeclaration`, `TaskDeclaration`, `HelperDeclaration`, `MapperDeclaration`, `SerializerDeclaration`
- Características: Solo `methods`

## 📊 Métricas
- **Estimación:** 64 horas
- **Keywords implementados:** 23 nuevos (+ 7 previos = 30 total)
- **Parsers creados:** 23 métodos `parse_*_declaration()`
- **AST nodes creados:** 23 clases `*Declaration`
- **Líneas de código:**
  - Parser: ~600 líneas
  - AST nodes: ~200 líneas
  - Total: ~800 líneas

## ✅ Criterios de Aceptación
- [x] 23 casos agregados en `parse_declaration()`
- [x] 23 métodos `parse_*_declaration()` implementados
- [x] 23 clases AST node creadas
- [x] Todos los keywords soportan modificador `public`
- [x] Implementación consistente con parsers existentes
- [x] Código committeado (c34d10e)

## 🔗 Referencias
- **Jira:** [TASK-012B](https://velalang.atlassian.net/browse/VELA-569)
- **Historia:** [VELA-569](https://velalang.atlassian.net/browse/VELA-569)
- **Commit:** c34d10e
- **Código:**
  - `src/parser/parser.py` líneas 1100-1545
  - `src/parser/ast_nodes.py` líneas 398-583

## 📝 Notas Técnicas

### Decisión: Fields vs Methods
- **Solo fields:** model
- **Solo methods:** factory, builder, strategy, observer, adapter, decorator, guard, middleware, interceptor, validator, provider, pipe, task, helper, mapper, serializer
- **Fields + methods:** widget, component, singleton, store, actor

### Plantilla de Parser
Todos los parsers siguen esta plantilla:

```python
def parse_X_declaration(self, is_public: bool = False) -> XDeclaration:
    """Parsea X declaration"""
    start = self.expect(TokenType.X)
    name = self.expect(TokenType.IDENTIFIER).value
    
    self.expect(TokenType.LBRACE)
    
    # Según tipo: fields, methods, o ambos
    fields = []
    methods = []
    while not self.check(TokenType.RBRACE) and not self.is_at_end():
        if self.check(TokenType.FN) or self.check(TokenType.ASYNC):
            methods.append(self.parse_method())
        else:
            fields.append(self.parse_class_field())
    
    self.expect(TokenType.RBRACE)
    end = self.peek(-1)
    
    return XDeclaration(
        range=self.create_range_from_tokens(start, end),
        is_public=is_public,
        name=name,
        fields=fields,  # O methods=methods según tipo
        methods=methods
    )
```

### Plantilla de AST Node
Todas las clases AST siguen esta plantilla:

```python
@dataclass
class XDeclaration(Declaration):
    """X keyword - Descripción"""
    name: str
    fields: List['ClassField']  # O methods: List['MethodDeclaration']
    methods: List['MethodDeclaration']
```

### Modificador `public`
Todos los keywords soportan `public`:

```vela
public widget MyWidget { }
public factory MyFactory { }
public store GlobalStore { }
```
