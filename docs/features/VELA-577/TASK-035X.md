# TASK-035X: Implementar @persistent Decorator

## 📋 Información General
- **Historia:** VELA-577 - State Management
- **Estado:** Completada ✅
- **Fecha:** 2025-12-02
- **Prioridad:** P1 (opcional)

## 🎯 Objetivo

Implementar el decorator `@persistent` para **persistencia automática** del estado del Store en `localStorage` (browser), `sessionStorage` (browser), o `filesystem` (Node/Desktop).

**Inspirado en:**
- Redux Persist
- Vuex Persist Plugin
- Pinia Persist
- Zustand Persist Middleware

## 🏗️ Arquitectura

### Flujo de Persistencia

```
┌──────────────────────────────────────────────────────┐
│                   @persistent                        │
│                                                      │
│  ┌────────────┐     ┌──────────────┐               │
│  │   Store    │────▶│ Persistence  │               │
│  │  Instance  │     │   Manager    │               │
│  └────────────┘     └──────────────┘               │
│         │                    │                       │
│         │                    ▼                       │
│         │           ┌────────────────┐              │
│         │           │ Storage Backend│              │
│         │           │ - localStorage │              │
│         │           │ - sessionStorage│             │
│         │           │ - file system  │              │
│         │           └────────────────┘              │
│         │                                            │
│  ┌─────▼──────┐                                     │
│  │ Auto-save  │ (on state change)                   │
│  │ Auto-load  │ (on init)                           │
│  └────────────┘                                     │
└──────────────────────────────────────────────────────┘
```

### Componentes Principales

#### 1. **PersistOptions** - Configuración

```python
@dataclass
class PersistOptions:
    key: str                             # Clave única para identificar estado
    storage: str = "localStorage"        # Backend: "localStorage" | "sessionStorage" | "file"
    file_path: Optional[str] = None      # Ruta (para storage="file")
    whitelist: Optional[List[str]] = None  # Fields a persistir (None = todos)
    blacklist: Optional[List[str]] = None  # Fields a NO persistir
    serialize: Optional[Callable] = None   # Función serialización (default: JSON)
    deserialize: Optional[Callable] = None # Función deserialización (default: JSON)
    throttle: int = 1000                  # Tiempo mínimo entre guardados (ms)
    debounce: Optional[int] = None        # Retrasar guardado (ms)
    merge_strategy: str = "shallow"       # "shallow" | "deep" | "overwrite"
```

#### 2. **StorageBackend** - Abstracción de almacenamiento

**Interfaz:**
```python
class StorageBackend(ABC):
    @abstractmethod
    def get_item(self, key: str) -> Optional[str]: ...
    
    @abstractmethod
    def set_item(self, key: str, value: str) -> None: ...
    
    @abstractmethod
    def remove_item(self, key: str) -> None: ...
    
    @abstractmethod
    def clear(self) -> None: ...
```

**Implementaciones:**
- `LocalStorageBackend` - Web localStorage
- `SessionStorageBackend` - Web sessionStorage
- `FileStorageBackend` - Node/Desktop filesystem

#### 3. **PersistenceManager** - Gestor de persistencia

```python
class PersistenceManager:
    def __init__(self, options: PersistOptions, store_class: Type):
        # Seleccionar backend según options.storage
        # Configurar serialize/deserialize
    
    def load_state(self) -> Optional[Dict[str, Any]]:
        """Carga estado del storage."""
    
    def save_state(self, state: Dict[str, Any]) -> None:
        """Guarda estado en storage (con filtros)."""
    
    def _filter_state(self, state: Dict) -> Dict:
        """Aplica whitelist/blacklist."""
    
    def _merge_state(self, current: Dict, persisted: Dict) -> Dict:
        """Merge con estrategia configurada."""
    
    def clear_state(self) -> None:
        """Elimina estado persistido."""
```

#### 4. **@persistent** - Decorator principal

```python
def persistent(options: PersistOptions) -> Callable[[Type], Type]:
    """
    Decorator que:
    1. Carga estado al inicializar Store
    2. Subscribe a cambios para auto-save
    3. Agrega métodos clear_persisted_state() y reload_persisted_state()
    """
```

## 🔧 Implementación

### Archivos Creados

1. **src/reactive/persistent.py** (~520 LOC)
   - `PersistOptions` dataclass
   - `StorageBackend` abstract class
   - `LocalStorageBackend`, `SessionStorageBackend`, `FileStorageBackend`
   - `PersistenceManager` class
   - `@persistent` decorator
   - Helper functions: `create_persistent_store()`, `is_persistent()`, `get_persist_options()`, `get_persistence_manager()`

2. **tests/unit/state/test_persistent.py** (~520 LOC, 30 tests pasando)
   - `TestLocalStorageBackend`: 4 tests
   - `TestSessionStorageBackend`: 2 tests
   - `TestFileStorageBackend`: 4 tests
   - `TestPersistenceManager`: 9 tests
   - `TestPersistentDecorator`: 8 tests
   - `TestHelperFunctions`: 3 tests

## 📝 APIs Principales

### 1. Decorator @persistent

```vela
@persistent(PersistOptions(
  key: "app-state",
  storage: "localStorage",
  whitelist: ["user", "settings"],
  throttle: 1000
))
store AppStore {
  state user: Option<User> = None
  state settings: Settings = defaultSettings
  state temp_data: Any = {}  # No persistido (no está en whitelist)
}
```

### 2. Helper: create_persistent_store()

```vela
# Crear Store persistente programáticamente
AppStore = create_persistent_store(
  BaseAppStore,
  key: "app-state",
  storage: "localStorage",
  whitelist: ["user", "settings"]
)

store = AppStore()
```

### 3. Métodos Agregados al Store

```vela
store AppStore { ... }

store = AppStore()

# Limpiar estado persistido
store.clear_persisted_state()

# Recargar desde storage
store.reload_persisted_state()
```

### 4. Metadata Helpers

```vela
# Verificar si Store es persistente
is_persistent(store)  # → true

# Obtener opciones de persistencia
options = get_persist_options(store)
print(options.key)  # "app-state"

# Obtener manager (para operaciones avanzadas)
manager = get_persistence_manager(store)
manager.clear_state()
```

## 📚 Ejemplos de Uso

### Ejemplo 1: localStorage con whitelist

```vela
@dataclass
struct User {
  id: Number
  name: String
  email: String
}

@dataclass
struct Settings {
  theme: String
  language: String
  notifications: Bool
}

@persistent(PersistOptions(
  key: "todo-app",
  storage: "localStorage",
  whitelist: ["user", "settings"],  # Solo persistir estos fields
  throttle: 1000  # Guardar cada 1 segundo como máximo
))
store TodoStore {
  state user: Option<User> = None
  state settings: Settings = Settings(theme: "light", language: "en", notifications: true)
  state todos: List<Todo> = []  # NO persistido (no está en whitelist)
  state temp_cache: Map<String, Any> = {}  # NO persistido
  
  # ... reducers
}

# Uso
store = TodoStore()

# Estado se carga automáticamente al inicializar
# Si hay estado persistido → se restaura

# Cambiar estado
store.dispatch(LoginAction(user: User(id: 1, name: "Alice", email: "alice@example.com")))
store.dispatch(ChangeThemeAction(theme: "dark"))

# Estado se guarda automáticamente en localStorage
# (solo user y settings, no todos ni temp_cache)

# Al recargar la página, el estado se restaura
store2 = TodoStore()
print(store2.get_state().user.name)  # "Alice"
print(store2.get_state().settings.theme)  # "dark"
print(store2.get_state().todos.length)  # 0 (no persistido)
```

### Ejemplo 2: File storage con blacklist

```vela
@persistent(PersistOptions(
  key: "desktop-app",
  storage: "file",
  file_path: "./storage",  # Guardar en ./storage/desktop-app.json
  blacklist: ["temp_data", "cache"],  # NO persistir estos fields
  merge_strategy: "deep"  # Merge profundo en restauración
))
store DesktopStore {
  state user: Option<User> = None
  state documents: List<Document> = []
  state settings: Settings = defaultSettings
  state temp_data: Any = {}  # NO persistido (blacklist)
  state cache: Map<String, Any> = {}  # NO persistido (blacklist)
}
```

### Ejemplo 3: Merge strategies

```vela
# Shallow merge (default) - Solo primer nivel
@persistent(PersistOptions(
  key: "app-shallow",
  merge_strategy: "shallow"
))
store ShallowStore {
  state config: Map<String, Any> = { "ui": { "theme": "light" } }
}

# Deep merge - Recursivo en todos los niveles
@persistent(PersistOptions(
  key: "app-deep",
  merge_strategy: "deep"
))
store DeepStore {
  state config: Map<String, Any> = { "ui": { "theme": "light", "fontSize": 14 } }
}

# Overwrite - Sobrescribir completamente con estado persistido
@persistent(PersistOptions(
  key: "app-overwrite",
  merge_strategy: "overwrite"
))
store OverwriteStore {
  state config: Map<String, Any> = { "ui": { "theme": "light" } }
}
```

### Ejemplo 4: Custom serializers

```vela
# Serializar con compresión o encriptación
fn compress_and_encrypt(obj: Any) -> String {
  json_str = JSON.stringify(obj)
  compressed = compress(json_str)
  encrypted = encrypt(compressed, secret_key)
  return encrypted
}

fn decrypt_and_decompress(data: String) -> Any {
  decrypted = decrypt(data, secret_key)
  decompressed = decompress(decrypted)
  return JSON.parse(decompressed)
}

@persistent(PersistOptions(
  key: "secure-app",
  serialize: compress_and_encrypt,
  deserialize: decrypt_and_decompress
))
store SecureStore {
  state sensitive_data: SecretData = defaultSecret
}
```

### Ejemplo 5: Clear y reload manual

```vela
@persistent(PersistOptions(key: "app"))
store AppStore {
  state count: Number = 0
}

store = AppStore()

# Cambiar estado
store.dispatch(IncrementAction)
print(store.get_state().count)  # 1

# Limpiar estado persistido
store.clear_persisted_state()

# Recargar desde storage (ahora está vacío)
store.reload_persisted_state()
print(store.get_state().count)  # 0 (restaurado a default)
```

## 🔄 Integración con Store<T>

El decorator `@persistent` **se integra perfectamente** con el Store<T> existente:

```vela
@persistent(PersistOptions(key: "todo-app"))
store TodoStore {
  state todos: List<Todo> = []
  
  reducer(state: TodoState, action: TodoAction) -> TodoState {
    match action {
      AddTodoAction(text) => {
        newTodo = Todo(id: state.todos.length, text: text, completed: false)
        return { ...state, todos: state.todos.push(newTodo) }
      }
      _ => state
    }
  }
}

# Al inicializar → carga estado persistido
store = TodoStore()

# Al dispatch → auto-save
store.dispatch(AddTodoAction(text: "Learn Vela"))  # Se guarda automáticamente

# Al recargar app → estado restaurado
```

## 📊 Comparación con Redux Persist

| Feature | Redux Persist | Vela @persistent ✅ |
|---------|---------------|---------------------|
| **Configuración** | Compleja (enhancer + config) | Simple (decorator) |
| **Storage backends** | localStorage, sessionStorage, AsyncStorage | localStorage, sessionStorage, file |
| **Whitelist/Blacklist** | ✅ | ✅ |
| **Merge strategies** | ❌ | ✅ (shallow, deep, overwrite) |
| **Throttle/Debounce** | Manual | ✅ Built-in |
| **Custom serializers** | ✅ | ✅ |
| **Type safety** | TS only | ✅ Nativo |

## ✅ Tests

### Cobertura

- **30 tests pasando** (100%)
- **6 test classes**:
  1. `TestLocalStorageBackend`: 4 tests
  2. `TestSessionStorageBackend`: 2 tests
  3. `TestFileStorageBackend`: 4 tests
  4. `TestPersistenceManager`: 9 tests
  5. `TestPersistentDecorator`: 8 tests
  6. `TestHelperFunctions`: 3 tests

### Tests Destacados

#### Storage Backends
- ✅ Set/get/remove items
- ✅ Clear storage
- ✅ File system operations

#### PersistenceManager
- ✅ Filtrado con whitelist/blacklist
- ✅ Merge strategies (shallow, deep, overwrite)
- ✅ Save/load state con serialización JSON
- ✅ Clear state

#### @persistent Decorator
- ✅ Marca clase con metadata
- ✅ Agrega métodos (clear_persisted_state, reload_persisted_state)
- ✅ Auto-save en cambios de estado
- ✅ Restauración al inicializar
- ✅ Whitelist/blacklist filtering en save

#### Helpers
- ✅ `create_persistent_store()`
- ✅ `is_persistent()`
- ✅ `get_persist_options()`
- ✅ `get_persistence_manager()`

## 🔗 Referencias

- **Jira:** [VELA-577](https://velalang.atlassian.net/browse/VELA-577)
- **Sprint:** Sprint 15
- **ADR:** [ADR-008](../../architecture/ADR-008-state-management-architecture.md)

## 📦 Entregables

- ✅ `src/reactive/persistent.py` (~520 LOC)
- ✅ `tests/unit/state/test_persistent.py` (~520 LOC)
- ✅ 30 tests pasando (100%)
- ✅ Documentación completa

## 🎯 Criterios de Aceptación

- [x] `PersistOptions` dataclass con configuración completa
- [x] 3 backends de storage (localStorage, sessionStorage, file)
- [x] `PersistenceManager` con filtrado y merge
- [x] Decorator `@persistent` funcional
- [x] Auto-save en cambios de estado
- [x] Auto-load al inicializar
- [x] Whitelist/blacklist filtering
- [x] Merge strategies (shallow, deep, overwrite)
- [x] Helper functions (create_persistent_store, is_persistent, etc.)
- [x] 30 tests pasando (100%)
- [x] Documentación completa

---

**Última actualización:** 2025-12-02  
**Versión:** 1.0.0  
**Estado:** ✅ Completada
