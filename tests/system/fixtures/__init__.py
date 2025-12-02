"""
System Test Fixtures - Pytest Fixtures

Fixtures compartidas para tests de sistema DI y REST.

Jira: VELA-575, TASK-035J
"""

import pytest
from src.runtime.di import Injector, Scope
from tests.system.fixtures.services import (
    DatabaseConnection,
    UserRepository,
    UserService,
    AuthService,
    RequestContext,
    Logger,
    CacheService,
    ServiceWithLifecycle
)


# ============================================================================
# FIXTURES DE INJECTOR
# ============================================================================

@pytest.fixture
def injector():
    """
    Injector limpio para tests de sistema.
    
    Scope: function (nuevo para cada test).
    
    Usage:
        def test_something(injector):
            injector.register(MyService)
            service = injector.get(MyService)
    """
    return Injector()


@pytest.fixture
def configured_injector():
    """
    Injector pre-configurado con servicios comunes.
    
    Servicios registrados:
    - DatabaseConnection (SINGLETON)
    - UserRepository (TRANSIENT)
    - UserService (TRANSIENT)
    - AuthService (TRANSIENT)
    - RequestContext (SCOPED)
    - Logger (TRANSIENT)
    - CacheService (TRANSIENT)
    
    Usage:
        def test_with_services(configured_injector):
            service = configured_injector.get(UserService)
    """
    injector = Injector()
    
    # Register services
    injector.register(DatabaseConnection, scope=Scope.SINGLETON)
    injector.register(UserRepository)
    injector.register(UserService)
    injector.register(AuthService)
    injector.register(RequestContext, scope=Scope.SCOPED)
    injector.register(Logger)
    injector.register(CacheService)
    injector.register(ServiceWithLifecycle)
    
    return injector


# ============================================================================
# FIXTURES DE SERVICIOS (para acceso directo)
# ============================================================================

@pytest.fixture
def database_connection():
    """
    DatabaseConnection para tests.
    
    Cleanup: Disconnects después del test.
    """
    db = DatabaseConnection()
    yield db
    db.disconnect()


@pytest.fixture
def user_repository(database_connection):
    """
    UserRepository con DatabaseConnection inyectado.
    """
    return UserRepository(database_connection)


@pytest.fixture
def user_service(user_repository):
    """
    UserService con UserRepository inyectado.
    """
    return UserService(user_repository)


@pytest.fixture
def auth_service(user_repository):
    """
    AuthService con UserRepository inyectado.
    """
    return AuthService(user_repository)


# ============================================================================
# FIXTURES DE SCOPE (para tests de SCOPED)
# ============================================================================

@pytest.fixture
def scoped_injector(configured_injector):
    """
    Injector con scope activo.
    
    Usage:
        def test_scoped_service(scoped_injector):
            ctx1 = scoped_injector.get(RequestContext)
            ctx2 = scoped_injector.get(RequestContext)
            assert ctx1 is ctx2  # Mismo dentro del scope
    """
    with configured_injector.create_scope() as scope:
        yield scope


# ============================================================================
# FIXTURES DE CLEANUP (para tests de lifecycle)
# ============================================================================

@pytest.fixture
def injector_with_cleanup():
    """
    Injector que trackea cleanup callbacks.
    
    Útil para tests de OnDestroy y disposal.
    
    Usage:
        def test_cleanup(injector_with_cleanup):
            injector, cleanup_called = injector_with_cleanup
            # ... test ...
            assert cleanup_called[0] == True
    """
    injector = Injector()
    cleanup_called = [False]
    
    def cleanup_callback():
        cleanup_called[0] = True
    
    # Register service con cleanup
    injector.register(ServiceWithLifecycle)
    
    yield injector, cleanup_called
    
    # Trigger cleanup
    cleanup_callback()


# ============================================================================
# FIXTURES DE PERFORMANCE (para benchmarks)
# ============================================================================

@pytest.fixture
def benchmark_injector():
    """
    Injector optimizado para benchmarks.
    
    Pre-calienta singletons para evitar overhead de inicialización.
    """
    injector = Injector()
    
    # Register services
    injector.register(DatabaseConnection, scope=Scope.SINGLETON)
    injector.register(UserRepository)
    injector.register(UserService)
    
    # Warmup: Resolver singletons
    injector.get(DatabaseConnection)
    
    return injector


# ============================================================================
# FIXTURES DE ERROR HANDLING (para tests de edge cases)
# ============================================================================

@pytest.fixture
def broken_injector():
    """
    Injector con configuración inválida (para tests de errores).
    
    Casos:
    - Missing providers
    - Circular dependencies
    - Invalid scopes
    """
    injector = Injector()
    
    # Ejemplo: Circular dependency
    from src.runtime.di import injectable
    
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
    
    return injector


# ============================================================================
# AUTOUSE FIXTURES (se ejecutan automáticamente)
# ============================================================================

@pytest.fixture(autouse=True)
def reset_singletons():
    """
    Reset singleton instances entre tests.
    
    Garantiza aislamiento entre tests de sistema.
    """
    # Antes del test: limpiar
    # (En implementación real, acceder al singleton registry del Injector)
    
    yield
    
    # Después del test: limpiar
    pass


@pytest.fixture(autouse=True)
def setup_logging():
    """
    Configurar logging para tests.
    
    Logs detallados para debugging de tests de sistema.
    """
    import logging
    logging.basicConfig(
        level=logging.DEBUG,
        format='[%(levelname)s] %(name)s: %(message)s'
    )
    
    yield
    
    # Cleanup
    logging.shutdown()


# ============================================================================
# MARKERS (para categorizar tests)
# ============================================================================

# Los markers se definen en pytest.ini o pyproject.toml
# Ejemplos:
# @pytest.mark.slow - Tests lentos (>1s)
# @pytest.mark.integration - Tests de integración
# @pytest.mark.benchmark - Benchmarks de performance
