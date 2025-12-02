"""
System Tests - DI Scope Behavior

Tests de comportamiento de scopes:
- SINGLETON: Misma instancia siempre
- TRANSIENT: Nueva instancia cada vez
- SCOPED: Misma instancia dentro del scope
- Anidación de scopes
- Mezcla de scopes
- Scope por defecto

Jira: VELA-575, TASK-035J
"""

import pytest
import random
from src.runtime.di import Injector, injectable, Scope
from tests.system.fixtures.services import (
    DatabaseConnection,
    UserRepository,
    UserService,
    RequestContext,
    Logger
)


# ============================================================================
# TESTS DE SINGLETON SCOPE
# ============================================================================

class TestSingletonScope:
    """Tests del scope SINGLETON."""
    
    def test_singleton_returns_same_instance(self, injector):
        """Test: SINGLETON retorna la misma instancia en múltiples get()."""
        
        @injectable(scope=Scope.SINGLETON)
        class SingletonService:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        injector.register(SingletonService)
        
        # Act: Resolver 5 veces
        instances = [injector.get(SingletonService) for _ in range(5)]
        
        # Assert: Todas son la misma instancia
        first_id = instances[0].id
        for instance in instances:
            assert instance is instances[0]
            assert instance.id == first_id
    
    def test_singleton_different_injectors_different_instances(self):
        """Test: SINGLETON en diferentes injectors → instancias diferentes."""
        
        @injectable(scope=Scope.SINGLETON)
        class SingletonService:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        injector1 = Injector()
        injector2 = Injector()
        
        injector1.register(SingletonService)
        injector2.register(SingletonService)
        
        # Act
        instance1 = injector1.get(SingletonService)
        instance2 = injector2.get(SingletonService)
        
        # Assert: Son instancias diferentes (cada injector tiene su singleton)
        assert instance1 is not instance2
        assert instance1.id != instance2.id
    
    def test_singleton_with_nested_dependencies(self, injector):
        """Test: SINGLETON con dependencias anidadas."""
        
        @injectable(scope=Scope.SINGLETON)
        class Config:
            def __init__(self):
                self.value = "config"
        
        @injectable(scope=Scope.SINGLETON)
        class DatabaseConnection:
            def __init__(self, config: Config):
                self.config = config
                self.id = random.randint(1, 1000000)
        
        @injectable
        class Repository:
            def __init__(self, db: DatabaseConnection):
                self.db = db
        
        injector.register(Config)
        injector.register(DatabaseConnection)
        injector.register(Repository)
        
        # Act
        repo1 = injector.get(Repository)
        repo2 = injector.get(Repository)
        
        # Assert: DatabaseConnection es singleton
        assert repo1.db is repo2.db
        assert repo1.db.config is repo2.db.config


# ============================================================================
# TESTS DE TRANSIENT SCOPE
# ============================================================================

class TestTransientScope:
    """Tests del scope TRANSIENT."""
    
    def test_transient_returns_new_instance(self, injector):
        """Test: TRANSIENT retorna nueva instancia en cada get()."""
        
        @injectable(scope=Scope.TRANSIENT)
        class TransientService:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        injector.register(TransientService)
        
        # Act: Resolver 5 veces
        instances = [injector.get(TransientService) for _ in range(5)]
        
        # Assert: Todas son instancias diferentes
        ids = [instance.id for instance in instances]
        assert len(set(ids)) == 5  # 5 IDs únicos
        
        for i in range(len(instances) - 1):
            assert instances[i] is not instances[i + 1]
    
    def test_transient_default_scope(self, injector):
        """Test: TRANSIENT es el scope por defecto (sin especificar)."""
        
        @injectable  # Sin especificar scope
        class DefaultScopeService:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        injector.register(DefaultScopeService)
        
        # Act
        instance1 = injector.get(DefaultScopeService)
        instance2 = injector.get(DefaultScopeService)
        
        # Assert: Son instancias diferentes (TRANSIENT por defecto)
        assert instance1 is not instance2
        assert instance1.id != instance2.id


# ============================================================================
# TESTS DE SCOPED SCOPE
# ============================================================================

class TestScopedScope:
    """Tests del scope SCOPED."""
    
    def test_scoped_same_instance_within_scope(self, injector):
        """Test: SCOPED retorna la misma instancia dentro del scope."""
        
        @injectable(scope=Scope.SCOPED)
        class ScopedService:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        injector.register(ScopedService)
        
        # Act: Crear scope y resolver múltiples veces
        with injector.create_scope() as scope:
            instance1 = scope.get(ScopedService)
            instance2 = scope.get(ScopedService)
            instance3 = scope.get(ScopedService)
            
            # Assert: Todas son la misma instancia dentro del scope
            assert instance1 is instance2
            assert instance2 is instance3
            assert instance1.id == instance2.id == instance3.id
    
    def test_scoped_different_instance_outside_scope(self, injector):
        """Test: SCOPED retorna instancias diferentes en scopes diferentes."""
        
        @injectable(scope=Scope.SCOPED)
        class ScopedService:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        injector.register(ScopedService)
        
        # Act: Crear 2 scopes
        with injector.create_scope() as scope1:
            instance1 = scope1.get(ScopedService)
            id1 = instance1.id
        
        with injector.create_scope() as scope2:
            instance2 = scope2.get(ScopedService)
            id2 = instance2.id
        
        # Assert: Son instancias diferentes (diferentes scopes)
        assert id1 != id2
    
    def test_scoped_nested_scopes(self, injector):
        """Test: SCOPED con anidación de scopes."""
        
        @injectable(scope=Scope.SCOPED)
        class ScopedService:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        injector.register(ScopedService)
        
        # Act: Scope externo
        with injector.create_scope() as outer_scope:
            outer_instance = outer_scope.get(ScopedService)
            outer_id = outer_instance.id
            
            # Scope interno (anidado)
            with injector.create_scope() as inner_scope:
                inner_instance = inner_scope.get(ScopedService)
                inner_id = inner_instance.id
                
                # Assert: Instancias diferentes (scopes diferentes)
                assert outer_id != inner_id
            
            # Resolver nuevamente en outer scope
            outer_instance2 = outer_scope.get(ScopedService)
            
            # Assert: Misma instancia que antes (mismo scope)
            assert outer_instance is outer_instance2
    
    def test_scoped_with_request_context(self, configured_injector):
        """Test: SCOPED con RequestContext (caso real)."""
        
        # RequestContext ya está registrado como SCOPED
        
        # Act: Crear 3 scopes (simular 3 requests HTTP)
        contexts = []
        
        for _ in range(3):
            with configured_injector.create_scope() as scope:
                ctx1 = scope.get(RequestContext)
                ctx2 = scope.get(RequestContext)
                
                # Assert: Mismo contexto dentro del scope
                assert ctx1 is ctx2
                assert ctx1.request_id == ctx2.request_id
                
                contexts.append(ctx1.request_id)
        
        # Assert: Contextos diferentes en cada scope (request)
        assert len(set(contexts)) == 3


# ============================================================================
# TESTS DE MEZCLA DE SCOPES
# ============================================================================

class TestMixedScopes:
    """Tests de mezcla de scopes (SINGLETON + TRANSIENT + SCOPED)."""
    
    def test_singleton_injected_into_transient(self, injector):
        """Test: SINGLETON inyectado en TRANSIENT."""
        
        @injectable(scope=Scope.SINGLETON)
        class Config:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        @injectable(scope=Scope.TRANSIENT)
        class Service:
            def __init__(self, config: Config):
                self.config = config
                self.id = random.randint(1, 1000000)
        
        injector.register(Config)
        injector.register(Service)
        
        # Act
        service1 = injector.get(Service)
        service2 = injector.get(Service)
        
        # Assert: Services son diferentes (TRANSIENT)
        assert service1 is not service2
        assert service1.id != service2.id
        
        # Assert: Config es el mismo (SINGLETON)
        assert service1.config is service2.config
        assert service1.config.id == service2.config.id
    
    def test_transient_injected_into_singleton(self, injector):
        """Test: TRANSIENT inyectado en SINGLETON (anti-pattern)."""
        
        @injectable(scope=Scope.TRANSIENT)
        class Logger:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        @injectable(scope=Scope.SINGLETON)
        class Service:
            def __init__(self, logger: Logger):
                self.logger = logger
        
        injector.register(Logger)
        injector.register(Service)
        
        # Act
        service1 = injector.get(Service)
        service2 = injector.get(Service)
        
        # Assert: Service es singleton
        assert service1 is service2
        
        # Assert: Logger es el mismo (capturado en el singleton)
        # NOTA: Este es un anti-pattern (transient en singleton)
        assert service1.logger is service2.logger
    
    def test_scoped_injected_into_transient(self, injector):
        """Test: SCOPED inyectado en TRANSIENT."""
        
        @injectable(scope=Scope.SCOPED)
        class RequestContext:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        @injectable(scope=Scope.TRANSIENT)
        class Service:
            def __init__(self, ctx: RequestContext):
                self.ctx = ctx
        
        injector.register(RequestContext)
        injector.register(Service)
        
        # Act: Dentro de un scope
        with injector.create_scope() as scope:
            service1 = scope.get(Service)
            service2 = scope.get(Service)
            
            # Assert: Services son diferentes (TRANSIENT)
            assert service1 is not service2
            
            # Assert: Contexto es el mismo (SCOPED)
            assert service1.ctx is service2.ctx
            assert service1.ctx.id == service2.ctx.id
    
    def test_complex_scope_hierarchy(self, injector):
        """Test: Jerarquía compleja de scopes."""
        
        @injectable(scope=Scope.SINGLETON)
        class DatabaseConnection:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        @injectable(scope=Scope.SCOPED)
        class RequestContext:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        @injectable(scope=Scope.TRANSIENT)
        class Repository:
            def __init__(self, db: DatabaseConnection, ctx: RequestContext):
                self.db = db
                self.ctx = ctx
                self.id = random.randint(1, 1000000)
        
        @injectable(scope=Scope.TRANSIENT)
        class Service:
            def __init__(self, repo: Repository):
                self.repo = repo
        
        injector.register(DatabaseConnection)
        injector.register(RequestContext)
        injector.register(Repository)
        injector.register(Service)
        
        # Act: Scope 1
        with injector.create_scope() as scope1:
            service1a = scope1.get(Service)
            service1b = scope1.get(Service)
            
            # Assert: Services diferentes (TRANSIENT)
            assert service1a is not service1b
            
            # Assert: Repositories diferentes (TRANSIENT)
            assert service1a.repo is not service1b.repo
            
            # Assert: RequestContext mismo (SCOPED)
            assert service1a.repo.ctx is service1b.repo.ctx
            
            # Assert: DatabaseConnection mismo (SINGLETON)
            assert service1a.repo.db is service1b.repo.db
            
            scope1_db_id = service1a.repo.db.id
            scope1_ctx_id = service1a.repo.ctx.id
        
        # Act: Scope 2
        with injector.create_scope() as scope2:
            service2 = scope2.get(Service)
            
            # Assert: DatabaseConnection mismo (SINGLETON global)
            assert service2.repo.db.id == scope1_db_id
            
            # Assert: RequestContext diferente (SCOPED diferente)
            assert service2.repo.ctx.id != scope1_ctx_id


# ============================================================================
# TESTS DE SCOPE INVÁLIDO
# ============================================================================

class TestInvalidScope:
    """Tests de scope inválido."""
    
    def test_invalid_scope_raises_error(self, injector):
        """Test: Scope inválido lanza ValueError."""
        
        with pytest.raises(ValueError) as exc_info:
            @injectable(scope="INVALID_SCOPE")
            class BadService:
                pass
        
        assert "Invalid scope" in str(exc_info.value)


# ============================================================================
# TESTS DE SCOPE POR DEFECTO
# ============================================================================

class TestDefaultScope:
    """Tests del scope por defecto."""
    
    def test_default_scope_is_transient(self, injector):
        """Test: Sin especificar scope → TRANSIENT por defecto."""
        
        @injectable  # Sin scope
        class DefaultService:
            def __init__(self):
                self.id = random.randint(1, 1000000)
        
        injector.register(DefaultService)
        
        # Act
        instance1 = injector.get(DefaultService)
        instance2 = injector.get(DefaultService)
        
        # Assert: TRANSIENT (instancias diferentes)
        assert instance1 is not instance2


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
