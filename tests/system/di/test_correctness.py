"""
System Tests - DI Correctness

Tests de correctness del sistema DI:
- Resolución correcta de dependencias
- Constructor injection
- Dependencias anidadas
- Factory providers
- Multi providers
- Value providers
- Async providers
- Lifecycle hooks
- Disposal de recursos

Jira: VELA-575, TASK-035J
"""

import pytest
import asyncio
from src.runtime.di import Injector, injectable, Scope, Provider
from tests.system.fixtures.services import (
    DatabaseConnection,
    UserRepository,
    UserService,
    AuthService,
    Logger,
    ServiceWithLifecycle
)


# ============================================================================
# TESTS DE RESOLUCIÓN BÁSICA
# ============================================================================

class TestBasicResolution:
    """Tests de resolución básica de dependencias."""
    
    def test_resolve_simple_class_no_deps(self, injector):
        """Test: Resolver clase sin dependencias."""
        
        @injectable
        class SimpleService:
            def __init__(self):
                self.value = 42
        
        injector.register(SimpleService)
        
        # Act
        service = injector.get(SimpleService)
        
        # Assert
        assert service is not None
        assert isinstance(service, SimpleService)
        assert service.value == 42
    
    def test_resolve_class_with_single_dependency(self, injector):
        """Test: Resolver clase con una dependencia."""
        
        @injectable
        class Logger:
            def __init__(self):
                self.logs = []
        
        @injectable
        class Service:
            def __init__(self, logger: Logger):
                self.logger = logger
        
        injector.register(Logger)
        injector.register(Service)
        
        # Act
        service = injector.get(Service)
        
        # Assert
        assert service is not None
        assert isinstance(service.logger, Logger)
    
    def test_resolve_class_with_multiple_dependencies(self, injector):
        """Test: Resolver clase con múltiples dependencias."""
        
        @injectable
        class DatabaseConnection:
            pass
        
        @injectable
        class CacheService:
            pass
        
        @injectable
        class Logger:
            pass
        
        @injectable
        class ComplexService:
            def __init__(self, db: DatabaseConnection, cache: CacheService, logger: Logger):
                self.db = db
                self.cache = cache
                self.logger = logger
        
        injector.register(DatabaseConnection)
        injector.register(CacheService)
        injector.register(Logger)
        injector.register(ComplexService)
        
        # Act
        service = injector.get(ComplexService)
        
        # Assert
        assert isinstance(service.db, DatabaseConnection)
        assert isinstance(service.cache, CacheService)
        assert isinstance(service.logger, Logger)


# ============================================================================
# TESTS DE DEPENDENCIAS ANIDADAS
# ============================================================================

class TestNestedDependencies:
    """Tests de dependencias anidadas (A → B → C)."""
    
    def test_resolve_nested_dependencies_3_levels(self, injector):
        """Test: Resolver 3 niveles de dependencias anidadas."""
        
        @injectable
        class Logger:
            def __init__(self):
                self.name = "Logger"
        
        @injectable
        class Repository:
            def __init__(self, logger: Logger):
                self.logger = logger
                self.name = "Repository"
        
        @injectable
        class Service:
            def __init__(self, repo: Repository):
                self.repo = repo
                self.name = "Service"
        
        injector.register(Logger)
        injector.register(Repository)
        injector.register(Service)
        
        # Act
        service = injector.get(Service)
        
        # Assert
        assert service.name == "Service"
        assert service.repo.name == "Repository"
        assert service.repo.logger.name == "Logger"
    
    def test_resolve_nested_dependencies_5_levels(self, injector):
        """Test: Resolver 5 niveles de dependencias anidadas."""
        
        @injectable
        class Level5:
            def __init__(self):
                self.level = 5
        
        @injectable
        class Level4:
            def __init__(self, dep: Level5):
                self.dep = dep
                self.level = 4
        
        @injectable
        class Level3:
            def __init__(self, dep: Level4):
                self.dep = dep
                self.level = 3
        
        @injectable
        class Level2:
            def __init__(self, dep: Level3):
                self.dep = dep
                self.level = 2
        
        @injectable
        class Level1:
            def __init__(self, dep: Level2):
                self.dep = dep
                self.level = 1
        
        for cls in [Level5, Level4, Level3, Level2, Level1]:
            injector.register(cls)
        
        # Act
        level1 = injector.get(Level1)
        
        # Assert
        assert level1.level == 1
        assert level1.dep.level == 2
        assert level1.dep.dep.level == 3
        assert level1.dep.dep.dep.level == 4
        assert level1.dep.dep.dep.dep.level == 5


# ============================================================================
# TESTS DE FACTORY PROVIDERS
# ============================================================================

class TestFactoryProviders:
    """Tests de factory providers."""
    
    def test_factory_provider_creates_instance(self, injector):
        """Test: Factory provider crea instancia correctamente."""
        
        @injectable
        class Config:
            def __init__(self):
                self.value = None
        
        def config_factory() -> Config:
            config = Config()
            config.value = "from_factory"
            return config
        
        injector.register_factory(Config, config_factory)
        
        # Act
        config = injector.get(Config)
        
        # Assert
        assert isinstance(config, Config)
        assert config.value == "from_factory"
    
    def test_factory_provider_with_dependencies(self, injector):
        """Test: Factory provider con dependencias inyectadas."""
        
        @injectable
        class DatabaseConnection:
            def __init__(self):
                self.connected = True
        
        @injectable
        class Repository:
            def __init__(self, db: DatabaseConnection):
                self.db = db
                self.initialized = False
        
        def repository_factory(db: DatabaseConnection) -> Repository:
            repo = Repository(db)
            repo.initialized = True
            return repo
        
        injector.register(DatabaseConnection)
        injector.register_factory(Repository, repository_factory)
        
        # Act
        repo = injector.get(Repository)
        
        # Assert
        assert isinstance(repo, Repository)
        assert repo.initialized is True
        assert isinstance(repo.db, DatabaseConnection)


# ============================================================================
# TESTS DE MULTI PROVIDERS
# ============================================================================

class TestMultiProviders:
    """Tests de multi providers."""
    
    def test_multi_provider_returns_list(self, injector):
        """Test: Multi provider retorna lista de instancias."""
        
        @injectable
        class Plugin:
            def __init__(self, name: str):
                self.name = name
        
        # Register multiple providers
        injector.register_multi(Plugin, lambda: Plugin("plugin1"))
        injector.register_multi(Plugin, lambda: Plugin("plugin2"))
        injector.register_multi(Plugin, lambda: Plugin("plugin3"))
        
        # Act
        plugins = injector.get_all(Plugin)
        
        # Assert
        assert isinstance(plugins, list)
        assert len(plugins) == 3
        assert plugins[0].name == "plugin1"
        assert plugins[1].name == "plugin2"
        assert plugins[2].name == "plugin3"
    
    def test_multi_provider_empty_returns_empty_list(self, injector):
        """Test: Multi provider sin providers retorna lista vacía."""
        
        @injectable
        class Plugin:
            pass
        
        # Act (sin registrar providers)
        plugins = injector.get_all(Plugin)
        
        # Assert
        assert isinstance(plugins, list)
        assert len(plugins) == 0


# ============================================================================
# TESTS DE VALUE PROVIDERS
# ============================================================================

class TestValueProviders:
    """Tests de value providers."""
    
    def test_value_provider_returns_exact_value(self, injector):
        """Test: Value provider retorna el valor exacto."""
        
        config = {"host": "localhost", "port": 8080}
        
        injector.register_value("Config", config)
        
        # Act
        resolved_config = injector.get("Config")
        
        # Assert
        assert resolved_config is config
        assert resolved_config["host"] == "localhost"
    
    def test_value_provider_with_string_token(self, injector):
        """Test: Value provider con string token."""
        
        injector.register_value("DATABASE_URL", "postgresql://localhost/testdb")
        
        # Act
        url = injector.get("DATABASE_URL")
        
        # Assert
        assert url == "postgresql://localhost/testdb"


# ============================================================================
# TESTS DE ASYNC PROVIDERS
# ============================================================================

class TestAsyncProviders:
    """Tests de async providers."""
    
    @pytest.mark.asyncio
    async def test_async_factory_provider(self, injector):
        """Test: Factory provider async."""
        
        @injectable
        class AsyncService:
            def __init__(self):
                self.value = None
        
        async def async_factory() -> AsyncService:
            await asyncio.sleep(0.01)  # Simular I/O
            service = AsyncService()
            service.value = "from_async_factory"
            return service
        
        injector.register_async_factory(AsyncService, async_factory)
        
        # Act
        service = await injector.get_async(AsyncService)
        
        # Assert
        assert isinstance(service, AsyncService)
        assert service.value == "from_async_factory"


# ============================================================================
# TESTS DE LIFECYCLE HOOKS
# ============================================================================

class TestLifecycleHooks:
    """Tests de lifecycle hooks (OnInit, OnDestroy)."""
    
    def test_on_init_hook_called(self, injector):
        """Test: OnInit hook se ejecuta después de creación."""
        
        @injectable
        class ServiceWithInit:
            def __init__(self):
                self.initialized = False
            
            def on_init(self):
                self.initialized = True
        
        injector.register(ServiceWithInit)
        
        # Act
        service = injector.get(ServiceWithInit)
        
        # Assert
        assert service.initialized is True
    
    def test_on_destroy_hook_called(self, configured_injector):
        """Test: OnDestroy hook se ejecuta en cleanup."""
        
        # ServiceWithLifecycle ya está registrado en configured_injector
        service = configured_injector.get(ServiceWithLifecycle)
        
        assert service.initialized is True
        assert service.destroyed is False
        
        # Act: Trigger cleanup
        configured_injector.dispose()
        
        # Assert
        assert service.destroyed is True


# ============================================================================
# TESTS DE METADATA Y @injectable
# ============================================================================

class TestInjectableDecorator:
    """Tests del decorator @injectable."""
    
    def test_injectable_decorator_sets_metadata(self):
        """Test: @injectable agrega metadata a la clase."""
        
        @injectable
        class MyService:
            pass
        
        # Assert
        assert hasattr(MyService, "__injectable__")
        assert MyService.__injectable__ is True
    
    def test_injectable_decorator_with_scope(self):
        """Test: @injectable con scope personalizado."""
        
        @injectable(scope=Scope.SINGLETON)
        class MySingleton:
            pass
        
        # Assert
        assert hasattr(MySingleton, "__injectable__")
        assert hasattr(MySingleton, "__scope__")
        assert MySingleton.__scope__ == Scope.SINGLETON


# ============================================================================
# TESTS DE CORRECTNESS CON SERVICIOS REALES (fixtures)
# ============================================================================

class TestRealServices:
    """Tests con servicios reales (UserService, AuthService, etc.)."""
    
    def test_user_service_resolution(self, configured_injector):
        """Test: UserService se resuelve con todas sus dependencias."""
        
        # Act
        service = configured_injector.get(UserService)
        
        # Assert
        assert isinstance(service, UserService)
        assert isinstance(service.repository, UserRepository)
        assert isinstance(service.repository.db, DatabaseConnection)
    
    def test_auth_service_resolution(self, configured_injector):
        """Test: AuthService se resuelve correctamente."""
        
        # Act
        service = configured_injector.get(AuthService)
        
        # Assert
        assert isinstance(service, AuthService)
        assert isinstance(service.user_repo, UserRepository)
    
    def test_database_connection_is_singleton(self, configured_injector):
        """Test: DatabaseConnection es singleton (misma instancia)."""
        
        # Act: Resolver múltiples veces
        db1 = configured_injector.get(DatabaseConnection)
        db2 = configured_injector.get(DatabaseConnection)
        
        # Assert
        assert db1 is db2
        assert db1.connection_id == db2.connection_id


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
