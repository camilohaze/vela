# ADR-035J: Estrategia de Tests de Sistema para DI y REST

## Estado
✅ Aceptado

## Fecha
2025-01-31

## Contexto

La Historia VELA-575 (Sistema de Dependency Injection) está completa en su implementación:
- ✅ TASK-035A: Core DI (Injector, providers)
- ✅ TASK-035B: Scopes (SINGLETON, TRANSIENT, SCOPED)
- ✅ TASK-035C: @injectable decorator
- ✅ TASK-035D: inject() helper
- ✅ TASK-035E: Multi providers + Factory providers
- ✅ TASK-035E2: HTTP decorators (@get, @post, @put, @patch, @delete)
- ✅ TASK-035F: Lifecycle hooks (OnInit, OnDestroy)
- ✅ TASK-035G: Module system (@module)
- ✅ TASK-035H: Circular dependency detection
- ✅ TASK-035I: Testing utilities (TestInjector, @mock, TestContainer)

**TASK-035J es la última tarea del Sprint 13**: Validar que TODO el sistema DI funciona correctamente en escenarios reales.

### Problema
Necesitamos una **suite de tests de sistema** que valide:

1. **Correctness**: El DI resuelve dependencias correctamente
2. **Scopes**: SINGLETON/TRANSIENT/SCOPED se comportan como especificado
3. **Routing**: Decoradores HTTP (@get, @post, etc.) funcionan con DI
4. **Edge Cases**: Errores, circular deps, concurrencia, async, etc.
5. **Integration**: DI + REST + Middleware + Guards juntos

**Diferencia con tests unitarios (TASK-035I)**:
- **Tests unitarios (ya existen)**: Prueban componentes aislados (TestInjector, @mock, etc.)
- **Tests de sistema (TASK-035J)**: Prueban el sistema completo en escenarios reales

## Análisis de Alternativas

### Opción 1: Tests de Sistema con Instancias Reales del Injector (ELEGIDA)

**Descripción**: Usar el `Injector` real (no mocks) en escenarios completos.

**Ejemplo**:
```python
# Test de sistema: SINGLETON scope
def test_singleton_scope_returns_same_instance():
    # Setup: Injector real
    injector = Injector()
    
    @injectable(scope=Scope.SINGLETON)
    class DatabaseConnection:
        def __init__(self):
            self.id = random.randint(1, 1000000)
    
    injector.register(DatabaseConnection)
    
    # Act: Resolver múltiples veces
    instance1 = injector.get(DatabaseConnection)
    instance2 = injector.get(DatabaseConnection)
    instance3 = injector.get(DatabaseConnection)
    
    # Assert: Todas son la misma instancia
    assert instance1 is instance2
    assert instance2 is instance3
    assert instance1.id == instance2.id == instance3.id
```

**Ventajas**:
- ✅ Valida el comportamiento real del sistema
- ✅ Detecta bugs de integración entre componentes
- ✅ Alta confianza en correctness
- ✅ Tests reflejan casos de uso reales

**Desventajas**:
- ❌ Más lentos que tests unitarios
- ❌ Requieren más setup
- ❌ Difíciles de debuggear (muchas partes involucradas)

### Opción 2: Tests de Integración con Mocks

**Descripción**: Usar TestInjector con mocks para simular dependencias.

**Ejemplo**:
```python
def test_scope_with_mocks(test_injector):
    # Mock de servicio
    @mock(DatabaseConnection)
    class MockDatabase:
        pass
    
    test_injector.override(DatabaseConnection, MockDatabase)
    
    # Test con mock
    instance = test_injector.get(DatabaseConnection)
    assert isinstance(instance, MockDatabase)
```

**Ventajas**:
- ✅ Rápidos
- ✅ Aislamiento completo
- ✅ Control total del comportamiento

**Desventajas**:
- ❌ NO validan el sistema real
- ❌ NO detectan bugs de integración
- ❌ Falsa sensación de seguridad

### Opción 3: Tests End-to-End con Servidor HTTP Real

**Descripción**: Levantar servidor HTTP, hacer requests HTTP reales.

**Ejemplo**:
```python
@pytest.fixture
def app():
    # Levantar servidor real
    server = VelaHTTPServer()
    
    @server.controller("/users")
    class UserController:
        def __init__(self, service: UserService):
            self.service = service
        
        @get("/:id")
        async def get_user(self, id: int):
            return self.service.find_by_id(id)
    
    yield server
    server.shutdown()

def test_http_routing_with_di(app, http_client):
    # Request HTTP real
    response = http_client.get("/users/123")
    assert response.status_code == 200
```

**Ventajas**:
- ✅ Validación completa del stack
- ✅ Tests desde perspectiva del cliente
- ✅ Detecta problemas de serialización, middleware, etc.

**Desventajas**:
- ❌ MUY lentos
- ❌ Requieren infraestructura (servidor, red, etc.)
- ❌ Frágiles (dependencias externas)

## Decisión

**Estrategia híbrida**:

1. **Tests de sistema con Injector real (Opción 1)** - 70% de los tests
   - Correctness: Resolución, lifecycle, providers
   - Scopes: SINGLETON, TRANSIENT, SCOPED
   - Edge cases: Errores, circular deps, async

2. **Tests end-to-end con servidor real (Opción 3)** - 30% de los tests
   - Routing: @get, @post, @put, @patch, @delete
   - Integration: DI + REST + Middleware
   - Escenarios reales: User CRUD, Auth flow

**NO usar mocks (Opción 2)** en tests de sistema:
- Los mocks son para tests unitarios (TASK-035I)
- Los tests de sistema DEBEN usar componentes reales

## Categorías de Tests de Sistema

### 1. Tests de Correctness DI (20 tests)

**Objetivo**: Verificar que el DI resuelve correctamente.

**Escenarios**:
- ✅ Resolución de clase simple (sin deps)
- ✅ Resolución de clase con deps (constructor injection)
- ✅ Resolución de clase con deps anidadas (A → B → C)
- ✅ Resolución con factory provider
- ✅ Resolución con multi provider
- ✅ Resolución con value provider
- ✅ Resolución con async provider
- ✅ Lifecycle hooks (OnInit, OnDestroy) se ejecutan
- ✅ Disposal de recursos
- ✅ Metadata (@injectable) se respeta

**Ejemplo**:
```python
# Test: Resolución con deps anidadas
def test_nested_dependencies():
    injector = Injector()
    
    @injectable
    class Logger:
        def log(self, msg): print(msg)
    
    @injectable
    class Repository:
        def __init__(self, logger: Logger):
            self.logger = logger
    
    @injectable
    class Service:
        def __init__(self, repo: Repository):
            self.repo = repo
    
    injector.register(Logger)
    injector.register(Repository)
    injector.register(Service)
    
    # Act: Resolver Service (debe resolver Logger → Repository → Service)
    service = injector.get(Service)
    
    # Assert
    assert isinstance(service, Service)
    assert isinstance(service.repo, Repository)
    assert isinstance(service.repo.logger, Logger)
```

### 2. Tests de Scope Behavior (15 tests)

**Objetivo**: Verificar que los scopes funcionan correctamente.

**Escenarios**:
- ✅ SINGLETON: Misma instancia en múltiples get()
- ✅ SINGLETON: Misma instancia en múltiples injectors (NO)
- ✅ TRANSIENT: Nueva instancia en cada get()
- ✅ SCOPED: Misma instancia dentro del scope
- ✅ SCOPED: Diferente instancia fuera del scope
- ✅ SCOPED: Anidación de scopes
- ✅ Mezcla de scopes (SINGLETON → TRANSIENT → SCOPED)
- ✅ Scope por defecto (SINGLETON)
- ✅ Scope inválido lanza error

**Ejemplo**:
```python
# Test: SCOPED scope con anidación
def test_scoped_nested_scopes():
    injector = Injector()
    
    @injectable(scope=Scope.SCOPED)
    class RequestContext:
        def __init__(self):
            self.id = random.randint(1, 1000000)
    
    injector.register(RequestContext)
    
    # Scope 1
    with injector.create_scope() as scope1:
        ctx1a = scope1.get(RequestContext)
        ctx1b = scope1.get(RequestContext)
        assert ctx1a is ctx1b  # Mismo dentro del scope
        
        # Scope 2 (anidado)
        with injector.create_scope() as scope2:
            ctx2 = scope2.get(RequestContext)
            assert ctx2 is not ctx1a  # Diferente en scope hijo
```

### 3. Tests de REST Routing (20 tests)

**Objetivo**: Verificar que los decoradores HTTP funcionan con DI.

**Escenarios**:
- ✅ @get resuelve deps del controller
- ✅ @post resuelve deps del controller
- ✅ @put, @patch, @delete funcionan
- ✅ Path params se extraen correctamente
- ✅ Query params se extraen
- ✅ Request body se parsea
- ✅ Headers se extraen
- ✅ Response serialization
- ✅ Error handling (404, 500, etc.)
- ✅ Middleware se ejecuta antes del handler
- ✅ Guards bloquean requests
- ✅ Pipes transforman datos

**Ejemplo**:
```python
# Test: @get con DI
@pytest.fixture
def app():
    injector = Injector()
    
    @injectable
    class UserService:
        def find_by_id(self, id: int):
            return {"id": id, "name": "Alice"}
    
    injector.register(UserService)
    
    @controller("/users")
    class UserController:
        def __init__(self, service: UserService):
            self.service = service
        
        @get("/:id")
        async def get_user(self, id: int):
            return self.service.find_by_id(id)
    
    return create_test_app(injector, [UserController])

async def test_get_endpoint_resolves_deps(app, http_client):
    response = await http_client.get("/users/123")
    assert response.status_code == 200
    assert response.json() == {"id": 123, "name": "Alice"}
```

### 4. Tests de Edge Cases (20 tests)

**Objetivo**: Verificar manejo de errores y casos extremos.

**Escenarios**:
- ✅ Missing provider lanza DependencyNotFoundError
- ✅ Circular dependency lanza CircularDependencyError
- ✅ Invalid scope lanza ValueError
- ✅ Async provider se espera correctamente
- ✅ Error en factory provider se propaga
- ✅ Dispose() con error no bloquea otros dispose()
- ✅ Concurrent access a SINGLETON (thread-safe)
- ✅ Concurrent scope creation
- ✅ Memory leak prevention (weak refs)
- ✅ Provider registration después de resolve()
- ✅ Override() con tipo incompatible
- ✅ Inject() con default value
- ✅ @injectable sin register() falla
- ✅ Multi provider vacío retorna []

**Ejemplo**:
```python
# Test: Circular dependency detection
def test_circular_dependency_detected():
    injector = Injector()
    
    @injectable
    class ServiceA:
        def __init__(self, b: 'ServiceB'):
            self.b = b
    
    @injectable
    class ServiceB:
        def __init__(self, a: ServiceA):
            self.a = a
    
    injector.register(ServiceA)
    injector.register(ServiceB)
    
    # Act & Assert: Debe lanzar CircularDependencyError
    with pytest.raises(CircularDependencyError) as exc_info:
        injector.get(ServiceA)
    
    assert "ServiceA -> ServiceB -> ServiceA" in str(exc_info.value)
```

### 5. Tests de Integración End-to-End (10 tests)

**Objetivo**: Validar escenarios reales completos.

**Escenarios**:
- ✅ User CRUD: POST /users, GET /users/:id, PUT /users/:id, DELETE /users/:id
- ✅ Auth flow: POST /auth/login, middleware valida token, guard bloquea
- ✅ Middleware chain: Logger → Auth → CORS → Handler
- ✅ Error handling: 404, 500, validation errors
- ✅ Performance: 1000 requests concurrentes

**Ejemplo**:
```python
# Test: User CRUD end-to-end
async def test_user_crud_flow(app, http_client):
    # 1. POST /users (crear)
    response = await http_client.post("/users", json={"name": "Alice"})
    assert response.status_code == 201
    user_id = response.json()["id"]
    
    # 2. GET /users/:id (leer)
    response = await http_client.get(f"/users/{user_id}")
    assert response.status_code == 200
    assert response.json()["name"] == "Alice"
    
    # 3. PUT /users/:id (actualizar)
    response = await http_client.put(f"/users/{user_id}", json={"name": "Bob"})
    assert response.status_code == 200
    
    # 4. DELETE /users/:id (eliminar)
    response = await http_client.delete(f"/users/{user_id}")
    assert response.status_code == 204
    
    # 5. GET /users/:id (verificar eliminación)
    response = await http_client.get(f"/users/{user_id}")
    assert response.status_code == 404
```

## Estructura de Archivos

```
tests/system/
├── di/
│   ├── test_correctness.py           # 20 tests de correctness
│   ├── test_scopes.py                # 15 tests de scopes
│   └── test_edge_cases.py            # 20 tests de edge cases
│
├── rest/
│   ├── test_routing.py               # 20 tests de routing
│   └── test_integration.py           # 10 tests end-to-end
│
└── fixtures/
    ├── app.py                        # Fixtures de aplicación test
    ├── http_client.py                # Cliente HTTP test
    └── services.py                   # Servicios mock para tests
```

**Total**: ~85 tests de sistema

## Herramientas y Fixtures

### Fixture: `test_app` - Aplicación HTTP para tests

```python
# tests/system/fixtures/app.py
import pytest
from src.runtime.di import Injector
from src.runtime.http import VelaHTTPServer

@pytest.fixture
def test_app():
    """
    Aplicación HTTP de test con DI configurado.
    
    Usage:
        def test_endpoint(test_app, http_client):
            response = await http_client.get("/users/123")
    """
    injector = Injector()
    server = VelaHTTPServer(injector)
    
    # Registrar servicios comunes
    from tests.system.fixtures.services import (
        UserService, 
        AuthService, 
        DatabaseConnection
    )
    
    injector.register(DatabaseConnection, scope=Scope.SINGLETON)
    injector.register(UserService)
    injector.register(AuthService)
    
    yield server
    
    # Cleanup
    server.shutdown()
```

### Fixture: `http_client` - Cliente HTTP para tests

```python
# tests/system/fixtures/http_client.py
import pytest
import httpx

@pytest.fixture
async def http_client(test_app):
    """
    Cliente HTTP asíncrono para hacer requests a test_app.
    
    Usage:
        response = await http_client.get("/users/123")
        response = await http_client.post("/users", json={"name": "Alice"})
    """
    base_url = f"http://localhost:{test_app.port}"
    
    async with httpx.AsyncClient(base_url=base_url) as client:
        yield client
```

### Servicios Mock para Tests

```python
# tests/system/fixtures/services.py
from src.runtime.di import injectable, Scope

@injectable(scope=Scope.SINGLETON)
class DatabaseConnection:
    def __init__(self):
        self.connected = True
        self.queries_executed = []
    
    def execute(self, query: str):
        self.queries_executed.append(query)
        return {"success": True}

@injectable
class UserService:
    def __init__(self, db: DatabaseConnection):
        self.db = db
    
    def find_by_id(self, id: int):
        self.db.execute(f"SELECT * FROM users WHERE id = {id}")
        return {"id": id, "name": f"User {id}"}
    
    def create(self, data: dict):
        self.db.execute(f"INSERT INTO users {data}")
        return {"id": 1, **data}

@injectable
class AuthService:
    def login(self, username: str, password: str):
        if username == "admin" and password == "secret":
            return {"token": "jwt-token-12345"}
        return None
```

## Métricas y Cobertura

### Objetivo de Cobertura
- **DI Core**: >= 95% (crítico)
- **Scopes**: >= 95% (crítico)
- **HTTP Routing**: >= 90%
- **Edge Cases**: >= 80%

### Métricas a Reportar
- Número de tests pasando
- Coverage % por módulo
- Tiempo de ejecución (objetivo: < 30s)
- Memory usage (objetivo: < 100MB)
- Performance (requests/sec)

### Comandos

```bash
# Ejecutar solo tests de sistema
pytest tests/system/ -v

# Con coverage
pytest tests/system/ --cov=src/runtime/di --cov=src/runtime/http --cov-report=html

# Con benchmark
pytest tests/system/ --benchmark-only
```

## Consecuencias

### Positivas
- ✅ Alta confianza en correctness del sistema DI
- ✅ Validación de casos reales (no solo unitarios)
- ✅ Detección temprana de bugs de integración
- ✅ Documentación viva de cómo usar el sistema
- ✅ Facilita refactoring (tests validan comportamiento)
- ✅ Benchmarks de performance

### Negativas
- ❌ Tests más lentos que unitarios (~30s vs ~5s)
- ❌ Requieren más setup (servidor HTTP, DB mock, etc.)
- ❌ Más difíciles de debuggear
- ❌ Dependencias entre tests (estado compartido)

### Mitigaciones
- Usar fixtures pytest para reducir boilerplate
- Paralelizar tests con `pytest-xdist`
- Cleanup automático en teardown
- Logs detallados para debugging

## Alternativas Consideradas

### Alternativa 1: Solo Tests Unitarios
**Rechazada**: No validan integración real, pueden dar falsos positivos.

### Alternativa 2: Solo Tests End-to-End
**Rechazada**: Muy lentos, frágiles, difíciles de debuggear.

### Alternativa 3: Property-Based Testing (hypothesis)
**Rechazada para esta iteración**: Útil pero requiere más tiempo de diseño. Considerar para futuros sprints.

## Referencias

### Jira
- **Historia**: [VELA-575](https://velalang.atlassian.net/browse/VELA-575)
- **Task**: [TASK-035J](https://velalang.atlassian.net/browse/VELA-575) - Tests de sistema DI y REST

### Frameworks de Referencia
- **NestJS Testing**: https://docs.nestjs.com/fundamentals/testing
- **Spring Boot Test**: https://docs.spring.io/spring-boot/docs/current/reference/html/features.html#features.testing
- **pytest fixtures**: https://docs.pytest.org/en/stable/fixture.html
- **httpx (async client)**: https://www.python-httpx.org/

### Código Relacionado
- `src/runtime/di/` - Core DI
- `src/runtime/di/testing/` - Testing utilities (TASK-035I)
- `src/runtime/http/` - HTTP server (TASK-035E2)

## Implementación

Ver código en:
- `tests/system/di/` - Tests de DI
- `tests/system/rest/` - Tests de REST
- `tests/system/fixtures/` - Fixtures compartidas

## Autor
GitHub Copilot Agent

## Fecha de Última Actualización
2025-01-31
