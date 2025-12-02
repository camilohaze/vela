# ADR-035I: DI Testing Strategy

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto

Necesitamos una estrategia robusta para testing de aplicaciones que usan Dependency Injection en Vela. Los desarrolladores requieren:

1. **Override de dependencias** en tests sin modificar código productivo
2. **Mocking automático** de services para tests unitarios aislados
3. **Test containers** con lifecycle management y auto-cleanup
4. **Integración con pytest** para fixtures reutilizables
5. **Spy/Mock capabilities** para verificar llamadas a dependencias

### Frameworks de Referencia Analizados

#### 1. **NestJS Testing Module**
```typescript
const module = await Test.createTestingModule({
  providers: [UserService, UserRepository]
})
  .overrideProvider(UserRepository)
  .useValue(mockRepository)
  .compile();

const userService = module.get<UserService>(UserService);
```

**Pros:**
- ✅ API limpia para override
- ✅ Test module aislado
- ✅ Mock providers integrados

**Cons:**
- ❌ Sintaxis verbosa (compile, get)
- ❌ Requiere importar todo el módulo

#### 2. **Spring Boot @MockBean**
```java
@SpringBootTest
class UserServiceTest {
  @MockBean
  private UserRepository repository;
  
  @Autowired
  private UserService service;
}
```

**Pros:**
- ✅ Declarativo con annotations
- ✅ Auto-wiring de dependencias
- ✅ Mocks automáticos

**Cons:**
- ❌ Coupling con test framework
- ❌ Overhead de inicializar contexto completo

#### 3. **pytest-mock + Dependency Injector (Python)**
```python
@pytest.fixture
def container():
    container = Container()
    container.register(UserService)
    return container

def test_user_service(container, mocker):
    mock_repo = mocker.Mock(spec=UserRepository)
    container.override_providers(UserRepository=mock_repo)
    service = container.resolve(UserService)
```

**Pros:**
- ✅ Integración nativa con pytest
- ✅ Fixtures reutilizables
- ✅ Override explícito

**Cons:**
- ❌ Mezcla pytest-mock con DI
- ❌ Verbose override syntax

## Decisión

### Arquitectura Elegida: **TestInjector + TestContainer + pytest fixtures**

Combinación de las mejores características de NestJS y pytest:

#### **1. TestInjector (Extiende Injector)**

```python
from vela.runtime.di.testing import TestInjector

@injectable
class UserService:
    def __init__(self, repo: UserRepository = inject(UserRepository)):
        self.repo = repo

# Test
def test_user_service():
    injector = TestInjector()
    
    # Override dependency con mock
    mock_repo = MockUserRepository()
    injector.override(UserRepository, mock_repo)
    
    # Resolve usa el mock
    service = injector.resolve(UserService)
    assert service.repo is mock_repo
```

**Características:**
- ✅ Hereda todas las capacidades de Injector
- ✅ Método `override(token, value)` para dependency replacement
- ✅ Método `spy(token)` para tracking de llamadas
- ✅ Auto-reset después de cada test

#### **2. @mock Decorator**

```python
from vela.runtime.di.testing import mock

# Crear mock provider automáticamente
@mock(UserRepository)
class MockUserRepository:
    def find_by_id(self, id: int):
        return User(id=id, name="Test User")

# En test
injector.register(MockUserRepository)
```

**Características:**
- ✅ Sintaxis declarativa
- ✅ Auto-registro como mock
- ✅ Type-safe (preserva interfaces)

#### **3. TestContainer**

```python
from vela.runtime.di.testing import create_test_container

def test_isolated_scenario():
    with create_test_container() as container:
        container.register(UserService)
        container.register(UserRepository)
        
        service = container.resolve(UserService)
        # Test...
    # Auto-cleanup al salir del context manager
```

**Características:**
- ✅ Context manager para auto-cleanup
- ✅ Aislamiento completo entre tests
- ✅ Lifecycle hooks (setup/teardown)

#### **4. pytest Fixtures**

```python
# conftest.py
import pytest
from vela.runtime.di.testing import TestInjector, create_test_container

@pytest.fixture
def injector():
    """Test injector que se resetea después de cada test."""
    test_injector = TestInjector()
    yield test_injector
    test_injector.reset()

@pytest.fixture
def test_container():
    """Test container con auto-cleanup."""
    with create_test_container() as container:
        yield container

# En test
def test_with_fixture(injector):
    injector.register(UserService)
    service = injector.resolve(UserService)
```

**Características:**
- ✅ Reutilizables en toda la suite de tests
- ✅ Scopes configurables (function, class, module)
- ✅ Composición de fixtures

## Comparación de Alternativas

| Opción | Override API | Mocking | Isolation | pytest Integration | Complejidad |
|--------|--------------|---------|-----------|-------------------|-------------|
| **TestInjector + pytest fixtures** | ✅ Excelente | ✅ Built-in | ✅ TestContainer | ✅ Native | 🟢 Baja |
| NestJS-style Module | ⚠️ Verboso | ✅ Sí | ✅ Sí | ❌ No | 🟡 Media |
| Spring @MockBean | ✅ Declarativo | ✅ Automático | ⚠️ Heavyweight | ❌ No | 🔴 Alta |
| pytest-mock solo | ❌ Manual | ✅ Sí | ⚠️ Manual | ✅ Sí | 🟡 Media |

## Consecuencias

### Positivas

1. **API Intuitiva**: Override simple con `injector.override(token, value)`
2. **Aislamiento Automático**: TestContainer garantiza no contaminar tests
3. **Integración pytest**: Fixtures reutilizables con scopes
4. **Type Safety**: Mocks preservan interfaces
5. **Bajo Overhead**: No requiere inicializar contexto completo
6. **Composable**: Fixtures se pueden componer fácilmente

### Negativas

1. **Nueva API**: Desarrolladores deben aprender TestInjector
2. **No Auto-Mocking**: Requiere crear mocks explícitamente (trade-off: más control)
3. **No Compatible con unittest**: Diseñado para pytest (mayoría del ecosistema Python)

### Mitigaciones

1. **Documentación completa** con ejemplos de patrones comunes
2. **Helper functions** para crear mocks básicos rápidamente
3. **pytest plugin** (futuro) para auto-discovery de fixtures

## Implementación

### Componentes a Desarrollar

#### 1. **TestInjector Class**
- Hereda de `Injector`
- `override(token, value)` - Replace dependency
- `spy(token)` - Track calls
- `reset()` - Limpiar overrides
- `snapshot()` / `restore()` - Save/restore state

#### 2. **@mock Decorator**
- Marca clase como mock provider
- Metadata: `__mock_target__`
- Auto-registro con `injector.register(MockClass)`

#### 3. **TestContainer**
- `create_test_container()` - Factory function
- Context manager protocol (`__enter__`, `__exit__`)
- Auto-dispose al salir

#### 4. **pytest Fixtures (conftest.py)**
- `injector` fixture (function scope)
- `test_container` fixture (function scope)
- `module_injector` fixture (module scope)

#### 5. **Helper Functions**
- `create_mock(interface)` - Auto-generate mock from interface
- `create_spy(instance)` - Wrap instance with spy
- `assert_called(spy, method, args)` - Assertion helpers

### Estructura de Archivos

```
src/runtime/di/testing/
├── __init__.py              # Exports públicos
├── test_injector.py         # TestInjector class
├── mock.py                  # @mock decorator
├── container.py             # TestContainer
├── fixtures.py              # pytest fixtures
└── helpers.py               # Helper functions

tests/unit/di/testing/
├── test_test_injector.py    # Tests para TestInjector
├── test_mock.py             # Tests para @mock
├── test_container.py        # Tests para TestContainer
└── test_fixtures.py         # Tests para fixtures
```

## Ejemplos de Uso

### Ejemplo 1: Test Unitario con Override

```python
def test_user_service_creates_user(injector):
    # Setup mock
    mock_repo = MockUserRepository()
    injector.override(UserRepository, mock_repo)
    
    # Resolve service (usa mock)
    service = injector.resolve(UserService)
    
    # Test
    user = service.create_user("Alice")
    assert user.name == "Alice"
    assert mock_repo.save_called
```

### Ejemplo 2: Test de Integración con TestContainer

```python
def test_user_workflow():
    with create_test_container() as container:
        # Register real implementations
        container.register(UserService)
        container.register(UserRepository)
        container.register(EmailService)
        
        # Test workflow completo
        service = container.resolve(UserService)
        user = service.register_user("bob@example.com")
        assert user.email == "bob@example.com"
```

### Ejemplo 3: Spy para Verificar Llamadas

```python
def test_service_calls_repository(injector):
    # Spy en repository
    repo = RealUserRepository()
    spy_repo = injector.spy(repo)
    injector.override(UserRepository, spy_repo)
    
    # Test
    service = injector.resolve(UserService)
    service.find_user(123)
    
    # Verify
    assert_called(spy_repo, 'find_by_id', args=(123,))
```

### Ejemplo 4: Fixture Module-Scoped

```python
# conftest.py
@pytest.fixture(scope="module")
def database_container():
    """Container con DB real para tests de integración."""
    with create_test_container() as container:
        # Setup DB connection
        db = Database(connection_string="test_db")
        container.register_value(Database, db)
        yield container
        # Cleanup
        db.close()

# test_integration.py
def test_user_crud(database_container):
    service = database_container.resolve(UserService)
    user = service.create_user("test@example.com")
    assert user is not None
```

## Referencias

- **NestJS Testing**: https://docs.nestjs.com/fundamentals/testing
- **Spring Boot @MockBean**: https://docs.spring.io/spring-boot/docs/current/reference/html/features.html#features.testing
- **pytest-mock**: https://pytest-mock.readthedocs.io/
- **Python dependency-injector testing**: https://python-dependency-injector.ets-labs.org/examples/testing.html

## Próximos Pasos

1. ✅ Implementar TestInjector class
2. ✅ Implementar @mock decorator
3. ✅ Implementar TestContainer
4. ✅ Crear pytest fixtures
5. ✅ Tests comprehensivos (15+ tests)
6. ✅ Documentación con ejemplos
7. ⏸️ pytest plugin (futuro - TASK-036A)

---

**Decidido por:** GitHub Copilot Agent  
**Fecha:** 2025-12-02  
**Versión DI:** 0.12.0 → 0.13.0 (con testing utilities)
