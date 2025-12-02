# ADR-035G2: Router HTTP para Sistema REST

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto

### Problema
Después de implementar el sistema DI con decoradores @controller, @get, @post, etc. (TASK-035D2, TASK-035D3), necesitamos un **Router HTTP funcional** que:

1. **Route Matching**: Coincida rutas HTTP con handlers
   - Path parameters: `/users/:id`, `/posts/:postId/comments/:commentId`
   - Query parameters: `/users?page=1&limit=10&tags=a&tags=b`
   - HTTP methods: GET, POST, PUT, DELETE, PATCH

2. **DI Integration**: Resuelva controller instances automáticamente
   - Inyección de dependencias en controllers
   - Scope management (Scoped per request)
   - Resolver handlers desde el Injector

3. **Middleware Pipeline**: Ejecute middleware antes/después de handlers
   - Pre-handler middleware (auth, logging)
   - Handler execution
   - Post-handler middleware (response transformation)
   - Error handling middleware

4. **Performance**: Routing eficiente para apps grandes
   - Algoritmo O(log n) o mejor para route matching
   - Caching de route matches
   - Lazy loading de handlers

### Lenguajes/Frameworks de Referencia

| Framework | Inspiración |
|-----------|-------------|
| **Express.js** | Routing simple: `app.get('/users/:id', handler)`, middleware pipeline |
| **NestJS** | Decoradores @Controller(), @Get(), DI integration |
| **Spring Boot** | @RequestMapping, @PathVariable, @RequestParam |
| **FastAPI** | Path operations, automatic param validation, dependency injection |
| **ASP.NET Core** | Routing middleware, endpoint routing, attribute routing |
| **Gin (Go)** | Radix tree router (high performance) |
| **Echo (Go)** | Router groups, middleware chains |

### Requisitos Clave

1. **Type Safety**: Path/query params deben ser type-safe
2. **Performance**: O(log n) route matching (trie/radix tree)
3. **Declarativo**: Decoradores para definir rutas (@get, @post)
4. **Flexible**: Soporte para middleware personalizados
5. **Debuggable**: Error messages claros para route conflicts

---

## Decisión

### 1. **Route Matching: Trie-Based Router**

**Decisión**: Usar **Radix Tree (prefix tree)** para route matching.

**Razones**:
- **O(log n) lookup** en lugar de O(n) linear search
- **Memory efficient**: Nodos comparten prefijos comunes
- **Path parameter extraction** natural: `/users/:id` → nodo parameterizado
- **Usado por**: Gin (Go), Echo (Go), Chi (Go) → probado en producción

**Alternativas Rechazadas**:
- ❌ **Linear search** (Express.js): O(n) lookup, lento con >100 rutas
- ❌ **Hash map routing**: No soporta path parameters sin preprocessing
- ❌ **Regex matching**: Lento, complejo, difícil de debuggear

**Estructura del Trie**:
```python
class RouteNode:
    segment: str                 # Segmento del path: "users" o ":id"
    is_param: bool               # True si es :id
    param_name: Optional[str]    # Nombre del parámetro: "id"
    handler: Optional[Callable]  # Handler si es nodo terminal
    children: Dict[str, RouteNode]  # Hijos del nodo
    middleware: List[Middleware]    # Middleware específico de esta ruta
```

**Ejemplo de Trie**:
```
/
├── users/
│   ├── :id/          → GET handler (param: "id")
│   │   └── posts/    → GET handler
│   └── /             → GET handler (list users)
└── posts/
    ├── :postId/      → GET handler (param: "postId")
    │   └── comments/
    │       └── :commentId/  → GET handler (params: "postId", "commentId")
    └── /             → POST handler (create post)
```

---

### 2. **Path Parameter Extraction**

**Decisión**: **Automatic extraction** durante route matching.

**Sintaxis**:
- `:paramName` → Named parameter: `/users/:id`
- `*wildcard` → Catch-all: `/files/*filepath` (captura todo lo que sigue)

**Type Coercion** (opcional):
```python
# En el handler:
def get_user(id: Number):  # Type hint → auto-coercion
    # id ya es Number, no String
```

**Validación**:
- Path params siempre String por defecto
- Type coercion basado en type hints (Number, Bool, etc.)
- Errores de coercion → HTTP 400 Bad Request

**Alternativas Rechazadas**:
- ❌ **Manual extraction**: `req.params['id']` (menos type-safe)
- ❌ **Regex groups**: `/users/(?P<id>\d+)` (complejo, hard-coded)

---

### 3. **Query Parameter Parsing**

**Decisión**: Parse query string con **soporte para arrays y type coercion**.

**Sintaxis**:
```python
# URL: /users?page=1&limit=10&tags=python&tags=rust

# Auto-parsing:
page: Number = 1          # Single value → Number
limit: Number = 10
tags: List[String] = ["python", "rust"]  # Multiple values → List
```

**Parsing Rules**:
1. **Single value**: `?key=value` → `{key: value}`
2. **Multiple values**: `?key=a&key=b` → `{key: [a, b]}`
3. **Empty value**: `?key=` → `{key: ""}`
4. **No value**: `?key` → `{key: None}`
5. **Type coercion**: Basado en type hints

**Alternativas Rechazadas**:
- ❌ **Manual parsing**: `req.query['page']` (propenso a errores)
- ❌ **No arrays**: Solo single values (limitado)

---

### 4. **DI Integration**

**Decisión**: **Auto-resolve controller instances** desde el Injector.

**Flow**:
```python
# 1. Registro de controller en DI
@injectable(scope=Scope.SINGLETON)
@controller("/users")
class UserController:
    service: UserService = inject(UserService)
    
    @get("/:id")
    def get_user(self, id: Number) -> User:
        return self.service.find_by_id(id)

# 2. Router resuelve controller automáticamente
# router.resolve_handler("/users/123") →
#   - Injector.resolve(UserController) → instance
#   - Extract path params: {id: 123}
#   - Call: instance.get_user(id=123)
```

**Scope per Request** (Scoped):
- Cada request HTTP crea un **nuevo scope Scoped**
- Controllers/Services Scoped son únicos por request
- Singletons se reutilizan entre requests

**Alternativas Rechazadas**:
- ❌ **Manual instantiation**: `controller = UserController()` (pierde DI)
- ❌ **Global singletons only**: No permite request-scoped services

---

### 5. **Middleware Pipeline**

**Decisión**: **Chain of Responsibility** pattern con `next()` function.

**Execution Order**:
```
Request →
  [Auth Middleware] →
    [Logging Middleware] →
      [Handler] →
    [Response Transform Middleware] →
  [Error Handler Middleware] →
Response
```

**Middleware Interface**:
```python
class Middleware(Protocol):
    def handle(self, request: Request, next: Callable) -> Response:
        # Pre-handler logic
        try:
            response = next(request)  # Call next middleware or handler
        except Exception as e:
            # Error handling
            raise
        # Post-handler logic
        return response
```

**Registration**:
```python
@controller("/users")
@middleware(AuthMiddleware, LoggingMiddleware)
class UserController:
    @get("/:id")
    @middleware(CacheMiddleware)  # Route-specific middleware
    def get_user(self, id: Number) -> User:
        ...
```

**Alternativas Rechazadas**:
- ❌ **Before/After hooks**: Menos flexible que middleware chain
- ❌ **Decorators only**: No permite conditional execution (next() call)

---

### 6. **Route Groups y Prefixes**

**Decisión**: **Hierarchical routing** con prefijos compartidos.

**Sintaxis**:
```python
@controller("/api/v1")  # Prefix: /api/v1
@middleware(AuthMiddleware)  # Shared middleware
class ApiV1Controller:
    @controller("/users")  # Nested: /api/v1/users
    class UserController:
        @get("/:id")  # Full path: /api/v1/users/:id
        def get_user(self, id: Number) -> User:
            ...
```

**Benefits**:
- **DRY**: No repetir `/api/v1` en cada ruta
- **Shared middleware**: Auth/CORS para todo el grupo
- **Versioning**: `/api/v1`, `/api/v2` separados

**Alternativas Rechazadas**:
- ❌ **Flat routing**: Repetir prefijos en cada ruta
- ❌ **Manual grouping**: `router.group('/api/v1', ...)` (menos declarativo)

---

### 7. **Error Handling**

**Decisión**: **Exception to HTTP status mapping**.

**Mapping**:
```python
exception_to_status = {
    NotFoundException: 404,
    UnauthorizedException: 401,
    ForbiddenException: 403,
    ValidationException: 400,
    InternalServerException: 500,
}
```

**Error Middleware**:
```python
class ErrorHandlerMiddleware(Middleware):
    def handle(self, request: Request, next: Callable) -> Response:
        try:
            return next(request)
        except Exception as e:
            status = exception_to_status.get(type(e), 500)
            return Response(
                status=status,
                body={"error": str(e), "type": type(e).__name__}
            )
```

**Alternativas Rechazadas**:
- ❌ **Manual try-catch**: Repetitivo en cada handler
- ❌ **Global error handler only**: No permite custom error handling per route

---

## Consecuencias

### Positivas

1. **✅ Performance O(log n)**:
   - Radix tree routing es eficiente para 1000+ rutas
   - Caching de route matches reduce overhead

2. **✅ Type Safety**:
   - Path/query params con type coercion automático
   - Type hints en handlers → validación en tiempo de ejecución

3. **✅ DI Integration**:
   - Controllers auto-resolved desde Injector
   - Request-scoped services (DB connections, logging contexts)

4. **✅ Flexible Middleware**:
   - Chain of Responsibility permite composición
   - next() call permite conditional execution

5. **✅ Declarativo**:
   - Decoradores @controller, @get, @post
   - Route definitions co-located con handlers

6. **✅ Versioning**:
   - Route groups con prefijos: `/api/v1`, `/api/v2`

### Negativas

1. **⚠️ Memory Overhead**:
   - Radix tree consume más memoria que linear search
   - Mitigación: Lazy loading de handlers

2. **⚠️ Route Conflicts**:
   - `/users/:id` vs `/users/new` → ambigüedad
   - Mitigación: Priority rules (static > param > wildcard)

3. **⚠️ Debugging Complexity**:
   - Middleware chain puede ser difícil de debuggear
   - Mitigación: Logging detallado de middleware execution

---

## Alternativas Consideradas

### Alternativa 1: Linear Search Router (Express.js)
**Descripción**: Iterar rutas en orden de registro.

**Pros**:
- Simple de implementar
- No overhead de trie construction

**Cons**:
- ❌ O(n) lookup (lento con >100 rutas)
- ❌ Route order matters (frágil)

**Razón de rechazo**: Performance inaceptable para apps grandes.

---

### Alternativa 2: Hash Map Router
**Descripción**: Map de ruta exacta → handler.

**Pros**:
- O(1) lookup para rutas exactas

**Cons**:
- ❌ No soporta path parameters sin preprocessing
- ❌ Requiere generar todas las combinaciones de rutas

**Razón de rechazo**: No soporta path parameters dinámicos.

---

### Alternativa 3: Regex-Based Router
**Descripción**: Compilar cada ruta a regex.

**Pros**:
- Flexible (soporta patterns complejos)

**Cons**:
- ❌ Lento (regex matching es costoso)
- ❌ Difícil de debuggear
- ❌ Hard to extract path parameters

**Razón de rechazo**: Performance y complejidad.

---

## Referencias

### Frameworks
- **Express.js**: https://expressjs.com/en/guide/routing.html
- **NestJS**: https://docs.nestjs.com/controllers
- **Spring Boot**: https://spring.io/guides/gs/rest-service/
- **FastAPI**: https://fastapi.tiangolo.com/tutorial/path-params/
- **Gin (Go)**: https://github.com/gin-gonic/gin (Radix tree router)
- **Echo (Go)**: https://echo.labstack.com/guide/routing/

### Algorithms
- **Radix Tree**: https://en.wikipedia.org/wiki/Radix_tree
- **Chain of Responsibility**: https://refactoring.guru/design-patterns/chain-of-responsibility

### Jira
- **Tarea**: [TASK-035G2](https://velalang.atlassian.net/browse/TASK-035G2)
- **Historia**: [VELA-575](https://velalang.atlassian.net/browse/VELA-575)
- **Dependencia**: TASK-035G (Lifecycle Management) ✅

---

## Implementación

### Archivos a Crear
1. `src/runtime/http/route.py` - Clase Route
2. `src/runtime/http/router.py` - Router class con radix tree
3. `src/runtime/http/middleware.py` - Middleware protocol y base classes
4. `src/runtime/http/request.py` - Request type
5. `src/runtime/http/response.py` - Response type
6. `src/runtime/http/__init__.py` - Exports

### Tests
- `tests/unit/http/test_route.py` - Route matching
- `tests/unit/http/test_router.py` - Router core
- `tests/unit/http/test_path_params.py` - Path parameter extraction
- `tests/unit/http/test_query_params.py` - Query parameter parsing
- `tests/unit/http/test_middleware.py` - Middleware pipeline
- `tests/unit/http/test_di_integration.py` - DI integration

### Métricas Esperadas
- **Cobertura**: >= 90%
- **Performance**: <1ms route matching (1000 rutas)
- **Tests**: >= 50 test cases

---

## Criterios de Aceptación

1. ✅ Route matching con path parameters
2. ✅ Query parameter parsing con arrays
3. ✅ DI integration con Injector
4. ✅ Middleware pipeline funcional
5. ✅ Route grouping con prefijos
6. ✅ Error handling con exception mapping
7. ✅ Tests pasando (>= 90% cobertura)
8. ✅ Documentación completa

---

**Última Actualización**: 2025-12-02  
**Estado**: Aceptado ✅  
**Implementador**: GitHub Copilot Agent
