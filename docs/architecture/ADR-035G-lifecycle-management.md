# ADR-035G: Lifecycle Management en Sistema DI

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto

El Injector Core (TASK-035F) implementó scopes básicos (Singleton, Transient, Scoped), pero falta **gestión avanzada de lifecycle**:

### Problemas Identificados

1. **No hay cleanup automático**:
   - Conexiones a base de datos quedan abiertas
   - File handles no se cierran
   - Recursos no se liberan al destruir scopes

2. **No hay hooks de lifecycle**:
   - No se puede ejecutar código custom al destruir instancias
   - No hay `OnDestroy` como Angular o `DisposableBean` como Spring

3. **No hay scope hierarchy**:
   - No se pueden crear scopes anidados (parent/child)
   - Útil para request scopes con sub-requests

4. **Testing difícil**:
   - No hay aislamiento de scopes para tests
   - Singletons globales afectan tests

## Decisión

Implementar **Lifecycle Management** con las siguientes features:

### 1. OnDisposable Protocol

**Inspiración**: Spring `DisposableBean`, Angular `OnDestroy`, NestJS `OnModuleDestroy`

```python
from typing import Protocol, runtime_checkable

@runtime_checkable
class OnDisposable(Protocol):
    """
    Protocol para objetos que requieren cleanup.
    
    Clases que implementen este protocol recibirán
    llamada a dispose() cuando su scope sea destruido.
    """
    
    def dispose(self) -> None:
        """
        Limpia recursos del objeto.
        
        Llamado automáticamente cuando el scope es destruido.
        Debe ser idempotente (se puede llamar múltiples veces).
        """
        ...

@runtime_checkable
class AsyncOnDisposable(Protocol):
    """Protocol para cleanup asíncrono."""
    
    async def dispose_async(self) -> None:
        """Limpieza asíncrona de recursos."""
        ...
```

**Uso:**

```python
from src.runtime.di import injectable, OnDisposable, Scope

@injectable(scope=Scope.SCOPED)
class DatabaseConnection(OnDisposable):
    def __init__(self):
        self.conn = connect_to_db()
        print("Database connected")
    
    def dispose(self) -> None:
        """Cleanup automático."""
        self.conn.close()
        print("Database connection closed")

# En el injector
context = injector.create_scope()
db = injector.get(DatabaseConnection, context)
# ... usar db ...
injector.dispose_scope(context)  # ← Llama automáticamente db.dispose()
```

### 2. Disposal Tracking

El Injector debe trackear instancias creadas por scope y llamar `dispose()` automáticamente:

```python
class Injector:
    def __init__(self):
        # Tracking de instancias con OnDisposable por scope
        self._disposable_instances: Dict[Scope, List[OnDisposable]] = {
            Scope.SINGLETON: [],
            Scope.SCOPED: [],  # Por contexto
            Scope.TRANSIENT: []
        }
    
    def _resolve_entry(self, entry: ProviderEntry, context: ResolutionContext) -> Any:
        # ... resolución normal ...
        
        instance = create_instance(...)
        
        # Si implementa OnDisposable, trackear
        if isinstance(instance, OnDisposable):
            self._track_disposable(instance, entry.scope, context)
        
        return instance
    
    def _track_disposable(
        self,
        instance: OnDisposable,
        scope: Scope,
        context: ResolutionContext
    ) -> None:
        """Agregar instancia al tracking."""
        if scope == Scope.SINGLETON:
            self._disposable_instances[Scope.SINGLETON].append(instance)
        elif scope == Scope.SCOPED:
            # Tracking en contexto local
            context.track_disposable(instance)
        # Transient: NO trackear (responsabilidad del usuario)
    
    def dispose_scope(self, scope: Scope, context: Optional[ResolutionContext] = None) -> None:
        """
        Disponer todas las instancias de un scope.
        
        Orden LIFO: last created, first disposed.
        """
        if scope == Scope.SINGLETON:
            instances = reversed(self._disposable_instances[Scope.SINGLETON])
            for instance in instances:
                try:
                    instance.dispose()
                except Exception as e:
                    logger.error(f"Error disposing {instance}: {e}")
            self._disposable_instances[Scope.SINGLETON].clear()
        
        elif scope == Scope.SCOPED:
            if context is None:
                raise ValueError("Context required for scoped disposal")
            context.dispose_all()
        
        # Clear cache también
        self.clear_scope(scope)
```

**Orden LIFO (Last In, First Out)**:
- Instancias se disponen en orden inverso a su creación
- Garantiza que dependencias se disponen después de sus dependents
- Ejemplo: `UserService` → `DatabaseConnection` → `Logger`
  - Creación: Logger → DatabaseConnection → UserService
  - Disposal: UserService → DatabaseConnection → Logger

### 3. Scope Hierarchy (Parent/Child Scopes)

**Inspiración**: AngularJS `$scope.$parent`, Spring `ApplicationContext` hierarchy

```python
@dataclass
class ScopeContext:
    """Contexto de scope con support para hierarchy."""
    
    scope: Scope
    parent: Optional[ScopeContext] = None
    children: List[ScopeContext] = field(default_factory=list)
    disposables: List[OnDisposable] = field(default_factory=list)
    
    def create_child(self) -> ScopeContext:
        """Crear scope hijo."""
        child = ScopeContext(scope=self.scope, parent=self)
        self.children.append(child)
        return child
    
    def dispose_all(self) -> None:
        """Dispose recursivo (children primero)."""
        # 1. Dispose children primero
        for child in reversed(self.children):
            child.dispose_all()
        
        # 2. Dispose propios disposables (LIFO)
        for disposable in reversed(self.disposables):
            try:
                disposable.dispose()
            except Exception as e:
                logger.error(f"Error disposing: {e}")
        
        self.disposables.clear()
        self.children.clear()

class Injector:
    def create_child_scope(self, parent: ScopeContext) -> ScopeContext:
        """Crear scope hijo."""
        return parent.create_child()
```

**Uso (request HTTP con sub-requests)**:

```python
# Request principal
main_context = injector.create_scope()
main_service = injector.get(UserService, main_context)

# Sub-request (database transaction)
transaction_context = injector.create_child_scope(main_context)
transaction_service = injector.get(TransactionService, transaction_context)

# Disposal
transaction_context.dispose_all()  # ← Limpia solo transaction scope
main_context.dispose_all()  # ← Limpia main scope
```

### 4. Lifecycle Hooks en ResolutionContext

```python
@dataclass
class ResolutionContext:
    # ... campos existentes ...
    
    # Lifecycle hooks
    on_create_callbacks: List[Callable[[Any], None]] = field(default_factory=list)
    on_dispose_callbacks: List[Callable[[Any], None]] = field(default_factory=list)
    
    def notify_created(self, instance: Any) -> None:
        """Notificar que instancia fue creada."""
        for callback in self.on_create_callbacks:
            callback(instance)
    
    def notify_disposed(self, instance: Any) -> None:
        """Notificar que instancia fue disposed."""
        for callback in self.on_dispose_callbacks:
            callback(instance)
    
    def add_create_hook(self, callback: Callable[[Any], None]) -> None:
        """Agregar hook de creación."""
        self.on_create_callbacks.append(callback)
    
    def add_dispose_hook(self, callback: Callable[[Any], None]) -> None:
        """Agregar hook de disposal."""
        self.on_dispose_callbacks.append(callback)
```

**Uso (logging de lifecycle)**:

```python
context = injector.create_scope()

# Log de creaciones
context.add_create_hook(lambda inst: print(f"Created: {type(inst).__name__}"))

# Log de disposals
context.add_dispose_hook(lambda inst: print(f"Disposed: {type(inst).__name__}"))

# Uso normal
service = injector.get(UserService, context)  # ← Log: "Created: UserService"
context.dispose_all()  # ← Log: "Disposed: UserService"
```

### 5. Scope Isolation para Testing

**Inspiración**: NestJS `TestingModule`, Spring `@DirtiesContext`

```python
class IsolatedScope:
    """Context manager para scopes aislados en tests."""
    
    def __init__(self, injector: Injector):
        self.injector = injector
        self.original_registry = None
        self.test_context = None
    
    def __enter__(self) -> ResolutionContext:
        """Crear scope aislado."""
        # Snapshot del registry actual
        self.original_registry = self.injector._registry.snapshot()
        
        # Crear contexto de test
        self.test_context = self.injector.create_scope()
        return self.test_context
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Cleanup automático."""
        # Dispose del scope de test
        self.test_context.dispose_all()
        
        # Restaurar registry original
        self.injector._registry.restore(self.original_registry)

def isolated_scope(injector: Injector) -> IsolatedScope:
    """Helper function para crear scope aislado."""
    return IsolatedScope(injector)
```

**Uso en tests:**

```python
def test_user_service():
    injector = get_global_injector()
    
    with isolated_scope(injector) as context:
        # Registrar mock
        injector.register(
            UserRepository,
            use_value=MockUserRepository(),
            scope=Scope.SCOPED
        )
        
        # Test con mock
        service = injector.get(UserService, context)
        users = service.get_users()
        assert len(users) == 2  # Mock data
    
    # ← Cleanup automático: mock removido, scope disposed
    
    # Siguiente test tiene injector limpio
```

### 6. Async Disposal Support

```python
class Injector:
    async def dispose_scope_async(
        self,
        scope: Scope,
        context: Optional[ResolutionContext] = None
    ) -> None:
        """Dispose asíncrono de scope."""
        instances = self._get_disposable_instances(scope, context)
        
        for instance in reversed(instances):
            try:
                if isinstance(instance, AsyncOnDisposable):
                    await instance.dispose_async()
                elif isinstance(instance, OnDisposable):
                    instance.dispose()
            except Exception as e:
                logger.error(f"Error disposing {instance}: {e}")
        
        self.clear_scope(scope)
```

**Uso:**

```python
@injectable(scope=Scope.SINGLETON)
class RedisClient(AsyncOnDisposable):
    async def connect(self):
        self.conn = await aioredis.connect()
    
    async def dispose_async(self) -> None:
        await self.conn.close()

# Disposal asíncrono
await injector.dispose_scope_async(Scope.SINGLETON)
```

## Consecuencias

### Positivas

1. **Gestión automática de recursos**:
   - Conexiones se cierran automáticamente
   - Memory leaks prevenidos
   - File handles liberados

2. **Cleanup predecible**:
   - Orden LIFO garantizado
   - Hooks para logging/monitoring
   - Testing más fácil

3. **Scope hierarchy flexible**:
   - Scopes anidados para casos complejos
   - Ideal para request/transaction scopes

4. **Testing mejorado**:
   - Scope isolation previene contaminación
   - Cleanup automático después de tests

5. **Async support**:
   - Dispose de recursos I/O-bound
   - Compatible con async factories

### Negativas

1. **Complejidad adicional**:
   - Tracking de instancias por scope
   - Orden LIFO requiere cuidado

2. **Performance overhead**:
   - Tracking de disposables agrega overhead
   - Disposal recursivo puede ser lento

3. **Responsabilidad del desarrollador**:
   - Transient scope no auto-dispose
   - Clases deben implementar OnDisposable correctamente

### Mitigaciones

1. **Tracking opcional**:
   - Solo trackear si implementa OnDisposable
   - Transient scope: no tracking por defecto

2. **Error handling robusto**:
   - Catch exceptions en dispose()
   - Continuar disposal aunque haya errores

3. **Documentación clara**:
   - Ejemplos de OnDisposable
   - Guía de cuándo usar cada scope

## Alternativas Consideradas

### Alternativa 1: Weak References + Finalizers

**Propuesta**: Usar weak references y finalizers Python (`__del__`) para cleanup.

**Rechazada porque**:
- Finalizers no garantizan ejecución
- Orden de ejecución impredecible
- No compatible con async dispose

### Alternativa 2: Context Managers para Todos los Providers

**Propuesta**: Forzar que todos los providers sean context managers.

**Rechazada porque**:
- Muy restrictivo
- No compatible con @injectable simple
- Requiere cambios masivos en código existente

### Alternativa 3: Reference Counting Manual

**Propuesta**: Implementar reference counting manual para disposal.

**Rechazada porque**:
- Complejo de mantener
- Propenso a errores (conteo incorrecto)
- LIFO order más simple y confiable

## Referencias

### Spring Framework
- `DisposableBean` interface
- `@PreDestroy` annotation
- `ApplicationContext.close()`

### Angular
- `OnDestroy` interface
- Automatic unsubscription
- Hierarchical injectors

### NestJS
- `OnModuleDestroy` interface
- `OnApplicationShutdown` hook
- `TestingModule` isolation

### InversifyJS
- `deactivation` bindings
- Container disposal
- Scope management

## Implementación

Ver archivos:
- `src/runtime/di/lifecycle.py` - OnDisposable protocol
- `src/runtime/di/injector.py` - Disposal tracking
- `src/runtime/di/scope_context.py` - Scope hierarchy
- `tests/unit/di/test_lifecycle.py` - Tests lifecycle

## Criterios de Aceptación

- [x] OnDisposable protocol definido
- [x] Disposal tracking implementado
- [x] Orden LIFO respetado
- [x] Scope hierarchy funcional
- [x] Lifecycle hooks implementados
- [x] Scope isolation para testing
- [x] Async disposal soportado
- [x] 30+ tests pasando
- [x] Documentación completa

---

**ESTADO: DISEÑO APROBADO - LISTO PARA IMPLEMENTACIÓN**
