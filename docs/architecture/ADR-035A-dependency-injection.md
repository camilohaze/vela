# ADR-035A: Sistema de Dependency Injection (DI) en Vela

## Estado
✅ Aceptado

## Fecha
2025-12-01

## Contexto

Vela es un lenguaje diseñado para aplicaciones modernas que requieren **arquitectura limpia**, **desacoplamiento** y **testabilidad**. Para soportar estos principios, necesitamos un **sistema de Dependency Injection (DI)** integrado directamente en el lenguaje, similar a Spring (Java), Angular (TypeScript) o NestJS (Node.js).

### Problemas que Resolvemos

1. **Acoplamiento fuerte**: Sin DI, las clases crean sus propias dependencias (tight coupling)
2. **Testabilidad**: Difícil hacer unit testing sin poder mockear dependencias
3. **Reutilización**: Difícil reutilizar componentes si dependen de implementaciones concretas
4. **Lifecycle management**: Gestión manual del ciclo de vida de objetos (singletons, transient, etc.)
5. **Configuración**: Falta de infraestructura para configurar dependencias

### Requisitos

- **Decoradores first-class**: DI debe usarse via decoradores (`@injectable`, `@inject`, `@module`)
- **Type-safety**: Resolución de dependencias debe ser type-safe
- **Scopes estándar**: Singleton, Transient, Scoped (como Spring/Angular)
- **Circular dependency detection**: Detectar ciclos en tiempo de compilación o runtime
- **Integration con REST**: Soporte para controllers (`@controller`, `@get`, `@post`, etc.)
- **Testing support**: Facilitar mocking y test containers

---

## Decisión

Implementamos un **sistema de DI completo** con los siguientes componentes:

### 1. **Decoradores de DI**

#### `@injectable`
Marca un keyword arquitectónico como inyectable con soporte para scopes:

```vela
# service keyword con @injectable
@injectable(scope: Scope.Singleton)
service UserService {
  repository: UserRepository
  
  constructor(@inject repository: UserRepository) {
    this.repository = repository
  }
  
  fn getUser(id: Number) -> Result<User, Error> {
    return this.repository.findById(id)
  }
}

# repository keyword con @injectable
@injectable
repository UserRepository {
  db: DatabaseConnection
  
  constructor(@inject db: DatabaseConnection) {
    this.db = db
  }
  
  async fn findById(id: Number) -> Promise<Option<User>> {
    return this.db.query("SELECT * FROM users WHERE id = ?", [id])
  }
}
```

**Scopes soportados:**
- `Scope.Singleton`: Una única instancia compartida en toda la aplicación
- `Scope.Transient`: Nueva instancia cada vez que se inyecta
- `Scope.Scoped`: Una instancia por scope (útil para requests HTTP)

**⚠️ IMPORTANTE**: `@injectable` se usa SOLO con keywords arquitectónicos (`service`, `repository`, `guard`, `middleware`, `usecase`). **Controllers NO usan `@injectable`** (como NestJS). NO se usa con `class` genérica a menos que sea un caso especial.

#### `@inject`
Marca parámetros de constructor para inyección automática:

```vela
# ✅ CORRECTO: Controller NO usa @injectable
@controller("/api/users")
controller UserController {
  service: UserService
  logger: Logger
  
  constructor(
    @inject service: UserService,
    @inject logger: Logger
  ) {
    this.service = service
    this.logger = logger
  }
}
```

#### `@module`
Define un módulo funcional con configuración de providers y DI (patrón MULTIPLATAFORMA):

```vela
# Backend module
@module({
  controllers: [UserController],  # REST endpoints
  providers: [UserService, UserRepository, Logger],  # Business logic
  imports: [DatabaseModule, LoggerModule],
  exports: [UserService]
})
module UserBackendModule { }

# Frontend module
@module({
  declarations: [UserWidget, UserCard, UserList],  # UI components
  providers: [UserService],  # Shared services
  imports: [UiModule],
  exports: [UserService, UserWidget]
})
module UserFrontendModule { }

# Hybrid module (TÍPICO EN VELA)
@module({
  declarations: [UserWidget, UserCard],  # UI components
  controllers: [UserController],  # REST API
  providers: [UserService, UserRepository, Logger, DatabaseConnection],  # Business logic
  imports: [DatabaseModule, LoggerModule, UiModule],
  exports: [UserService, UserWidget]  # AMBOS: service + widget
})
module UserModule { }
```

**Propiedades de `@module`:**
- `declarations`: Widgets, components (frontend/general)
- `controllers`: Controllers REST (backend, NO son providers)
- `providers`: Clases inyectables (`@injectable`): services, repositories, guards, middleware, pipes
- `imports`: Otros módulos a importar
- `exports`: Declarations y/o providers disponibles para otros módulos (subconjunto de declarations ∪ providers)

**⚠️ IMPORTANTE**: 
- `@module` NO es instanciable. Es una unidad de organización y configuración, no una clase.
- **Vela es MULTIPLATAFORMA**: soporta `declarations` (frontend) + `controllers` (backend)
- **Controllers** van en `controllers: []`, NO en `providers: []` (como NestJS)
- **Declarations** para widgets/components (como Angular)
- **Providers** son servicios inyectables que necesitan `@injectable`

#### `@controller`
Define controlador REST con path prefix:

```vela
# ❌ INCORRECTO: Controller NO necesita @injectable
# @injectable
# @controller("/api/users")
# controller UserController { }

# ✅ CORRECTO: Controller usa SOLO @controller (como NestJS)
@controller("/api/users")
controller UserController {
  service: UserService
  
  constructor(@inject service: UserService) {
    this.service = service
  }
  
  @get("/:id")
  async fn getUser(@param id: Number) -> Response<User> {
    user = await this.service.getUser(id)
    return match user {
      Ok(u) => Response.ok(u)
      Err(e) => Response.notFound(e.message)
    }
  }
  
  @post("/")
  async fn createUser(@body dto: CreateUserDTO) -> Response<User> {
    user = await this.service.createUser(dto)
    return Response.created(user)
  }
}
```

**Decoradores HTTP disponibles:**
- `@get(path)`: GET endpoint
- `@post(path)`: POST endpoint
- `@put(path)`: PUT endpoint
- `@patch(path)`: PATCH endpoint
- `@delete(path)`: DELETE endpoint

**Decoradores de parámetros:**
- `@param`: Path parameters (`:id`)
- `@body`: Request body
- `@query`: Query parameters

**⚠️ IMPORTANTE**: 
- `controller` **NO necesita** `@injectable` (igual que NestJS)
- Se registra en `controllers: []` del `@module`, NO en `providers: []`
- El decorador `@controller(path)` es suficiente para routing y DI

#### `@provides`
Marca factory methods que proveen instancias en módulos:

```vela
@module({
  declarations: [DatabaseConnection],
  providers: [DatabaseConnection]
})
module DatabaseModule {
  @provides(scope: Scope.Singleton)
  fn provideDatabase() -> DatabaseConnection {
    return DatabaseConnection(
      host: Config.get("DB_HOST"),
      port: Config.get("DB_PORT")
    )
  }
}
```

**Características**:
- Se usa DENTRO de módulos para crear instancias custom
- Permite configuración avanzada de dependencias
- Soporta scopes (Singleton, Transient, Scoped)

### 2. **Injector (Contenedor DI Core)**

El **Injector** es el corazón del sistema DI. Responsabilidades:

- **Registrar providers**: Mantener registro de todas las clases inyectables
- **Resolver dependencias**: Crear instancias resolviendo dependencias recursivamente
- **Gestionar scopes**: Controlar lifecycle de instancias según scope
- **Detectar ciclos**: Identificar dependencias circulares
- **Cache de singletons**: Mantener instancias singleton en memoria

**Algoritmo de resolución:**

```python
def resolve(token: Type[T]) -> T:
    # 1. Verificar si ya existe instancia (singleton/scoped)
    if token in cache[current_scope]:
        return cache[current_scope][token]
    
    # 2. Obtener metadata del provider
    provider = registry[token]
    
    # 3. Verificar circular dependencies
    if token in resolution_stack:
        raise CircularDependencyError(resolution_stack + [token])
    
    resolution_stack.push(token)
    
    # 4. Resolver dependencias recursivamente
    dependencies = []
    for param in provider.constructor_params:
        if param.has_inject_decorator:
            dep = resolve(param.type)
            dependencies.append(dep)
    
    # 5. Crear instancia
    instance = provider.create(*dependencies)
    
    # 6. Cachear si es singleton/scoped
    if provider.scope in [Scope.Singleton, Scope.Scoped]:
        cache[current_scope][token] = instance
    
    resolution_stack.pop()
    
    return instance
```

### 3. **Scopes y Lifecycle Management**

#### **Singleton**
- **Lifecycle**: Una única instancia por aplicación
- **Uso**: Servicios stateless, conexiones DB, loggers
- **Cache**: Global

```vela
@injectable(scope: Scope.Singleton)
class DatabaseConnection {
  # Solo se crea una vez
}
```

#### **Transient**
- **Lifecycle**: Nueva instancia cada inyección
- **Uso**: Objetos con estado temporal
- **Cache**: No cachea

```vela
@injectable(scope: Scope.Transient)
class EmailMessage {
  # Nueva instancia por cada uso
}
```

#### **Scoped**
- **Lifecycle**: Una instancia por scope (request HTTP)
- **Uso**: Objetos con estado por request (user session, transaction)
- **Cache**: Por scope

```vela
@injectable(scope: Scope.Scoped)
class UserSession {
  # Una instancia por request HTTP
}
```

### 4. **Integración con REST (Controllers)**

Para microservicios y APIs REST, integramos DI con routing:

#### `@controller`
Define controlador REST con path prefix:

```vela
@injectable
@controller(path: "/users")
class UserController {
  service: UserService
  
  constructor(@inject service: UserService) {
    this.service = service
  }
  
  @get("/:id")
  async fn getUser(id: Number) -> Result<Response<User>, Error> {
    user = await this.service.getUser(id)
    return match user {
      Ok(u) => Ok(Response.ok(u))
      Err(e) => Err(e)
    }
  }
  
  @post("/")
  async fn createUser(dto: CreateUserDTO) -> Result<Response<User>, Error> {
    user = await this.service.createUser(dto)
    return Ok(Response.created(user))
  }
}
```

#### Decoradores HTTP
- `@get(path)`: GET endpoint
- `@post(path)`: POST endpoint
- `@put(path)`: PUT endpoint
- `@patch(path)`: PATCH endpoint
- `@delete(path)`: DELETE endpoint

**Path parameters**: `/users/:id` → `id: Number` en parámetros
**Query parameters**: `/users?role=admin` → automáticamente parseado

### 5. **Middleware y Guards**

#### `middleware` keyword
Define middleware HTTP para pre/post procesamiento:

```vela
@injectable
middleware LoggerMiddleware {
  logger: Logger
  
  constructor(@inject logger: Logger) {
    this.logger = logger
  }
  
  async fn apply(req: Request, res: Response, next: NextFunction) -> Promise<void> {
    this.logger.info("Request: ${req.method} ${req.path}")
    await next()
    this.logger.info("Response: ${res.statusCode}")
  }
}
```

**Características obligatorias**:
- DEBE ser `@injectable`
- DEBE implementar `apply(req, res, next): Promise<void>`
- DEBE llamar `next()` para continuar pipeline

#### `guard` keyword
Define guards de autorización:

```vela
@injectable
guard AuthGuard {
  authService: AuthService
  
  constructor(@inject authService: AuthService) {
    this.authService = authService
  }
  
  async fn canActivate(context: ExecutionContext) -> Promise<Result<Bool, Error>> {
    token = context.getRequest().headers["Authorization"]
    return this.authService.validateToken(token)
      .map(user => true)
      .mapErr(() => false)
  }
}
```

**Características obligatorias**:
- DEBE ser `@injectable`
- DEBE implementar `canActivate(context): Promise<Result<Bool, Error>>`
- NO puede tener lógica de negocio

**Uso en controllers:**

```vela
# ✅ Controller NO usa @injectable
@controller("/api/admin")
@guard(AuthGuard)
controller AdminController {
  # Todos los endpoints requieren AuthGuard
}
```

### 6. **Request y Response Types**

#### `Request`
```vela
struct Request {
  method: String        # "GET", "POST", etc.
  path: String          # "/users/123"
  headers: Dict<String, String>
  query: Dict<String, String>
  params: Dict<String, String>   # Path params
  body: Option<String>
  
  fn json<T>() -> Result<T, Error> {
    # Parse body as JSON
  }
}
```

#### `Response`
```vela
struct Response<T> {
  status: Number
  headers: Dict<String, String>
  body: T
  
  # Factory methods
  static fn ok<T>(data: T) -> Response<T> {
    return Response { status: 200, headers: {}, body: data }
  }
  
  static fn created<T>(data: T) -> Response<T> {
    return Response { status: 201, headers: {}, body: data }
  }
  
  static fn badRequest(message: String) -> Response<String> {
    return Response { status: 400, headers: {}, body: message }
  }
}
```

### 7. **Router HTTP**

El router gestiona routing de requests a controllers:

```python
class Router:
    def __init__(self):
        self.routes = []
        self.middleware_stack = []
    
    def register_controller(self, controller: Type):
        # Extraer metadata de @controller
        base_path = controller.metadata.path
        middleware = controller.metadata.middleware
        
        # Registrar cada método con decorador HTTP
        for method in controller.methods:
            if method.has_http_decorator:
                route = Route(
                    method=method.http_method,  # "GET", "POST", etc.
                    path=base_path + method.http_path,
                    handler=method.function,
                    middleware=middleware + method.middleware
                )
                self.routes.append(route)
    
    def match(self, request: Request) -> Option<Route>:
        # Encontrar ruta que matchea path y method
        for route in self.routes:
            if route.matches(request):
                return Some(route)
        return None
    
    def handle(self, request: Request) -> Response:
        # 1. Encontrar ruta
        route = self.match(request)
        if route.is_none():
            return Response.notFound()
        
        # 2. Ejecutar middleware chain
        response = self.execute_middleware_chain(
            route.middleware,
            request,
            lambda: route.handler(request)
        )
        
        return response
```

**Path matching con parámetros:**
- `/users/:id` → `{ id: "123" }`
- `/posts/:postId/comments/:commentId` → `{ postId: "1", commentId: "5" }`

### 8. **Circular Dependency Detection**

Detectamos ciclos durante resolución:

```vela
# ❌ ERROR: Dependencia circular detectada
@injectable
class ServiceA {
  constructor(@inject b: ServiceB) { }
}

@injectable
class ServiceB {
  constructor(@inject a: ServiceA) { }
}

# Error en runtime:
# CircularDependencyError: ServiceA -> ServiceB -> ServiceA
```

**Solución recomendada**: Usar `@provides` con lazy initialization:

```vela
@injectable
class ServiceA {
  b_factory: () -> ServiceB
  
  constructor(@inject b_factory: () -> ServiceB) {
    this.b_factory = b_factory
  }
  
  fn doSomething() -> void {
    b = this.b_factory()  # Lazy resolve
    b.method()
  }
}
```

### 9. **Testing Support**

#### **Test Container**
Contenedor DI especial para tests con mocking fácil:

```vela
@test
fn testUserService() -> void {
  # Crear test container
  container = TestContainer()
  
  # Mock de repositorio
  mockRepository = MockUserRepository()
  mockRepository.stub("findById", (id) => Ok(User { id: id, name: "Test" }))
  
  # Registrar mock
  container.register(UserRepository, mockRepository)
  container.register(UserService)
  
  # Obtener service con mock inyectado
  service = container.get(UserService)
  
  # Test
  result = service.getUser(123)
  assert(result.is_ok())
  assert(result.unwrap().name == "Test")
}
```

---

## Consecuencias

### Positivas

1. ✅ **Arquitectura limpia**: Desacoplamiento completo entre componentes
2. ✅ **Testabilidad máxima**: Fácil mockear dependencias en tests
3. ✅ **Type-safety**: Resolución de dependencias con tipos estáticos
4. ✅ **Estándar de industria**: Patterns conocidos (Spring, Angular, NestJS)
5. ✅ **Microservicios ready**: REST, middleware, guards integrados
6. ✅ **Lifecycle management**: Scopes automáticos (singleton, transient, scoped)
7. ✅ **DX excelente**: Decoradores intuitivos y documentación clara

### Negativas

1. ⚠️ **Complejidad adicional**: Sistema DI añade complejidad al runtime
2. ⚠️ **Performance overhead**: Resolución de dependencias tiene costo
3. ⚠️ **Curva de aprendizaje**: Developers deben entender DI patterns
4. ⚠️ **Runtime errors**: Algunos errores solo aparecen en runtime (circular deps)
5. ⚠️ **Debugging**: Stack traces más complejos con DI

### Mitigaciones

- **Optimización**: Cachear resolución de dependencias cuando sea posible
- **Error messages**: Mensajes de error claros para circular dependencies
- **Documentation**: Guías extensas y ejemplos de uso
- **Tooling**: LSP support para autocompletar y detectar errores

---

## Alternativas Consideradas

### 1. **Manual Dependency Injection (sin framework)**

**Pros:**
- Sin overhead de runtime
- Más explícito

**Cons:**
- ❌ Boilerplate masivo en apps grandes
- ❌ Difícil gestionar lifecycle
- ❌ Sin soporte para testing

**Decisión**: **Rechazada** - No es escalable para aplicaciones enterprise

### 2. **Service Locator Pattern**

```vela
# Service locator global
service = ServiceLocator.get(UserService)
```

**Pros:**
- Más simple que DI

**Cons:**
- ❌ Acoplamiento a ServiceLocator (anti-pattern)
- ❌ No type-safe
- ❌ Difícil testear
- ❌ Oculta dependencias

**Decisión**: **Rechazada** - Anti-pattern conocido

### 3. **Constructor Injection sin decoradores**

```vela
class UserService {
  constructor(repository: UserRepository) { }
}

# Creación manual
repository = UserRepository()
service = UserService(repository)
```

**Pros:**
- Más explícito
- Sin magic

**Cons:**
- ❌ Boilerplate en apps grandes
- ❌ Difícil gestionar scopes
- ❌ No soporta middleware/guards

**Decisión**: **Rechazada** - No escalable

---

## Referencias

- **Jira**: [VELA-575](https://velalang.atlassian.net/browse/VELA-575)
- **Epic**: EPIC-03B: Dependency Injection
- **Story**: US-07B: Como desarrollador, quiero inyección de dependencias automática
- **Sprint**: 13

### Inspiración Externa

- [Spring Framework DI](https://spring.io/guides/gs/di/)
- [Angular Dependency Injection](https://angular.io/guide/dependency-injection)
- [NestJS Dependency Injection](https://docs.nestjs.com/fundamentals/custom-providers)
- [InversifyJS](https://inversify.io/) (TypeScript DI)

---

## Implementación

La implementación se divide en 15 subtasks:

1. **TASK-035A**: ✅ Diseño de arquitectura (este ADR)
2. **TASK-035B**: Implementar `@injectable` decorator
3. **TASK-035C**: Implementar `@inject` decorator
4. **TASK-035D**: Implementar `@module` decorator con DI support
5. **TASK-035D2**: Implementar `@controller` decorator
6. **TASK-035D3**: Implementar decoradores HTTP
7. **TASK-035E**: Implementar `@provides` decorator
8. **TASK-035E2**: Implementar `@middleware` y `@guard` decorators
9. **TASK-035F**: Implementar Injector core
10. **TASK-035G**: Implementar Scopes (Singleton, Transient, Scoped)
11. **TASK-035G2**: Implementar Router HTTP
12. **TASK-035G3**: Implementar Request/Response types
13. **TASK-035H**: Implementar circular dependency detection
14. **TASK-035I**: Integrar DI con testing framework
15. **TASK-035J**: Tests de sistema DI y REST

### Archivos Principales

```
src/runtime/di/
├── injector.py           # Contenedor DI core
├── injectable.py         # @injectable decorator
├── inject.py             # @inject decorator
├── module.py             # @module decorator
├── provides.py           # @provides decorator
├── scopes.py             # Scope management
└── circular_detection.py # Detección de ciclos

src/runtime/web/
├── controller.py         # @controller decorator
├── http_decorators.py    # @get, @post, @put, @delete, @patch
├── middleware.py         # @middleware decorator
├── guard.py              # @guard decorator
├── router.py             # Sistema de routing
├── request.py            # Request type
└── response.py           # Response type

tests/unit/di/
├── test_injector.py
├── test_injectable.py
├── test_inject.py
├── test_module.py
├── test_provides.py
├── test_scopes.py
└── test_circular_detection.py

tests/unit/web/
├── test_controller.py
├── test_http_decorators.py
├── test_middleware_guard.py
├── test_router.py
└── test_request_response.py

tests/integration/
├── di/
│   └── test_di_integration.py
└── web/
    └── test_rest_integration.py
```

---

## Ejemplo Completo de Aplicación

```vela
# ========================================
# Domain Layer (NO inyectables)
# ========================================

entity User {
  id: Number
  name: String
  email: String
}

dto CreateUserDTO {
  public readonly name: String
  public readonly email: String
  
  fn validate() -> Result<void, ValidationError> {
    if !this.email.contains("@") {
      return Err(ValidationError("Invalid email"))
    }
    return Ok(())
  }
}

# ========================================
# Infrastructure Layer
# ========================================

@injectable(scope: Scope.Singleton)
service DatabaseConnection {
  async fn query(sql: String) -> Promise<Result<List<Dict>, Error>> {
    # Implementación
  }
}

# ========================================
# Repository Layer (keyword: repository)
# ========================================

@injectable
repository UserRepository {
  db: DatabaseConnection
  
  constructor(@inject db: DatabaseConnection) {
    this.db = db
  }
  
  async fn findById(id: Number) -> Promise<Result<User, Error>> {
    result = await this.db.query("SELECT * FROM users WHERE id = ${id}")
    return match result {
      Ok(rows) if rows.length > 0 => Ok(User.from(rows[0]))
      Ok(_) => Err(Error("User not found"))
      Err(e) => Err(e)
    }
  }
  
  async fn create(user: User) -> Promise<Result<User, Error>> {
    # Implementación
  }
}

# ========================================
# Service Layer (keyword: service)
# ========================================

@injectable(scope: Scope.Singleton)
service UserService {
  repository: UserRepository
  logger: Logger
  
  constructor(
    @inject repository: UserRepository,
    @inject logger: Logger
  ) {
    this.repository = repository
    this.logger = logger
  }
  
  async fn getUser(id: Number) -> Promise<Result<User, Error>> {
    this.logger.info("Getting user ${id}")
    result = await this.repository.findById(id)
    return match result {
      Ok(user) => {
        this.logger.info("User found: ${user.name}")
        Ok(user)
      }
      Err(e) => {
        this.logger.error("User not found: ${e}")
        Err(e)
      }
    }
  }
  
  async fn createUser(dto: CreateUserDTO) -> Promise<Result<User, Error>> {
    # Validar DTO
    validation = dto.validate()
    if validation.is_err() {
      return Err(validation.unwrap_err())
    }
    
    user = User {
      id: 0,
      name: dto.name,
      email: dto.email
    }
    
    return await this.repository.create(user)
  }
}

# ========================================
# Controller Layer (keyword: controller)
# ========================================

# ✅ CORRECTO: Controller NO usa @injectable (como NestJS)
@controller("/api/users")
controller UserController {
  service: UserService
  
  constructor(@inject service: UserService) {
    this.service = service
  }
  
  @get("/:id")
  async fn getUser(@param id: Number) -> Response<User> {
    user = await this.service.getUser(id)
    return match user {
      Ok(u) => Response.ok(u)
      Err(e) => Response.notFound(e.message)
    }
  }
  
  @get("/")
  async fn listUsers(@query page: Number = 1) -> Response<List<User>> {
    # Implementación
  }
  
  @post("/")
  async fn createUser(@body dto: CreateUserDTO) -> Response<User> {
    user = await this.service.createUser(dto)
    return match user {
      Ok(u) => Response.created(u)
      Err(e) => Response.badRequest(e.message)
    }
  }
  
  @put("/:id")
  async fn updateUser(@param id: Number, @body dto: CreateUserDTO) -> Response<User> {
    # Implementación
  }
  
  @delete("/:id")
  async fn deleteUser(@param id: Number) -> Response<void> {
    # Implementación
  }
}

# ========================================
# Middleware (keyword: middleware)
# ========================================

# ✅ CORRECTO: Middleware SÍ usa @injectable (es provider, no controller)
@injectable
middleware LoggerMiddleware {
  logger: Logger
  
  constructor(@inject logger: Logger) {
    this.logger = logger
  }
  
  async fn apply(req: Request, res: Response, next: NextFunction) -> Promise<void> {
    start = Time.now()
    this.logger.info("${req.method} ${req.path}")
    
    await next()
    
    duration = Time.now() - start
    this.logger.info("${res.statusCode} (${duration}ms)")
  }
}

# ========================================
# Guard (keyword: guard)
# ========================================

# ✅ CORRECTO: Guard SÍ usa @injectable (es provider, no controller)
@injectable
guard AuthGuard {
  authService: AuthService
  
  constructor(@inject authService: AuthService) {
    this.authService = authService
  }
  
  async fn canActivate(context: ExecutionContext) -> Promise<Result<Bool, Error>> {
    token = context.getRequest().headers["Authorization"]
    return match token {
      Some(t) => await this.authService.validateToken(t)
      None => Err(Error("No token provided"))
    }
  }
}

# ========================================
# Application Module (keyword: module)
# ========================================

@module({
  # Controllers se registran separadamente (NO en providers)
  controllers: [UserController],
  
  # Providers: services, repositories, guards, middleware, etc.
  providers: [
    DatabaseConnection,
    UserRepository,
    UserService,
    Logger,
    AuthService,
    LoggerMiddleware,
    AuthGuard
  ],
  
  imports: [],
  exports: [UserService]
})
module AppModule { }

# ⚠️ NOTA: 
# - controllers: [] → Solo controllers (NO son providers)
# - providers: [] → Services, repositories, guards, middleware, pipes
# - Como NestJS: @Module({ controllers, providers, imports, exports })

# ========================================
# Main Entry Point
# ========================================

fn main() -> void {
  # Crear injector desde módulo
  injector = Injector.fromModule(AppModule)
  
  # Crear router HTTP
  router = Router()
  
  # Registrar controllers
  userController = injector.get(UserController)
  router.registerController(userController)
  
  # Iniciar servidor
  server = HttpServer(router, port: 8080)
  server.start()
  
  print("Server running on http://localhost:8080")
}
```

---

## Métricas de Éxito

- ✅ **Cobertura de tests**: >= 90% en código DI
- ✅ **Performance**: Resolución de dependencias < 1ms
- ✅ **Documentación**: Ejemplos completos en docs
- ✅ **Adoption**: Usado en 100% de ejemplos de microservicios
- ✅ **Error reporting**: Mensajes claros para circular dependencies

---

## Conclusión

El sistema de **Dependency Injection** de Vela proporciona:

1. **Arquitectura enterprise-grade** para microservicios
2. **Decoradores intuitivos** para DI y REST
3. **Type-safety completo** con resolución estática
4. **Testing first-class** con mocking integrado
5. **Performance optimizada** con caching de singletons

Este diseño posiciona a Vela como un lenguaje moderno para backend development, compitiendo con Spring, NestJS y .NET Core en funcionalidades de DI.

---

**Autor**: GitHub Copilot Agent  
**Revisores**: Cristian Naranjo  
**Última actualización**: 2025-12-01
