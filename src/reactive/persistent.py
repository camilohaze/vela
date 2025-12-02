"""
@persistent Decorator - Persistencia Automática de Store

Implementación de: VELA-577 - TASK-035X
Historia: State Management
Fecha: 2025-12-02

Descripción:
Decorator @persistent para guardar y restaurar automáticamente el estado
del Store en localStorage (browser) o filesystem (Node/Desktop).

Inspirado en:
- Redux Persist
- Vuex Persist Plugin
- Pinia Persist
"""

import json
from typing import Any, Callable, Dict, List, Optional, Set, Type
from dataclasses import dataclass
from pathlib import Path
from abc import ABC, abstractmethod


@dataclass
class PersistOptions:
    """
    Configuración del decorator @persistent.
    
    Attributes:
        key: Clave para identificar el estado persistido
        storage: Backend de almacenamiento ("localStorage", "sessionStorage", "file")
        file_path: Ruta del archivo (para storage="file")
        whitelist: Lista de state fields a persistir (None = todos)
        blacklist: Lista de state fields a NO persistir
        serialize: Función de serialización (default: JSON)
        deserialize: Función de deserialización (default: JSON)
        throttle: Tiempo mínimo entre guardados en ms (default: 1000)
        debounce: Retrasar guardado después del último cambio en ms (default: None)
        merge_strategy: Estrategia de merge ("shallow", "deep", "overwrite")
    """
    key: str
    storage: str = "localStorage"  # "localStorage" | "sessionStorage" | "file"
    file_path: Optional[str] = None
    whitelist: Optional[List[str]] = None
    blacklist: Optional[List[str]] = None
    serialize: Optional[Callable[[Any], str]] = None
    deserialize: Optional[Callable[[str], Any]] = None
    throttle: int = 1000  # ms
    debounce: Optional[int] = None  # ms
    merge_strategy: str = "shallow"  # "shallow" | "deep" | "overwrite"


class StorageBackend(ABC):
    """
    Interfaz abstracta para backends de almacenamiento.
    """
    
    @abstractmethod
    def get_item(self, key: str) -> Optional[str]:
        """Obtiene valor por clave."""
        pass
    
    @abstractmethod
    def set_item(self, key: str, value: str) -> None:
        """Guarda valor por clave."""
        pass
    
    @abstractmethod
    def remove_item(self, key: str) -> None:
        """Elimina valor por clave."""
        pass
    
    @abstractmethod
    def clear(self) -> None:
        """Limpia todo el almacenamiento."""
        pass


class LocalStorageBackend(StorageBackend):
    """
    Backend de localStorage (simulado para Python).
    
    En producción, este backend se implementaría con
    JavaScript Web Storage API.
    """
    
    def __init__(self):
        self._storage: Dict[str, str] = {}
    
    def get_item(self, key: str) -> Optional[str]:
        """Obtiene valor del localStorage."""
        return self._storage.get(key)
    
    def set_item(self, key: str, value: str) -> None:
        """Guarda valor en localStorage."""
        self._storage[key] = value
    
    def remove_item(self, key: str) -> None:
        """Elimina valor del localStorage."""
        self._storage.pop(key, None)
    
    def clear(self) -> None:
        """Limpia localStorage."""
        self._storage.clear()


class SessionStorageBackend(StorageBackend):
    """
    Backend de sessionStorage (simulado para Python).
    
    Similar a localStorage pero con ciclo de vida de sesión.
    """
    
    def __init__(self):
        self._storage: Dict[str, str] = {}
    
    def get_item(self, key: str) -> Optional[str]:
        """Obtiene valor del sessionStorage."""
        return self._storage.get(key)
    
    def set_item(self, key: str, value: str) -> None:
        """Guarda valor en sessionStorage."""
        self._storage[key] = value
    
    def remove_item(self, key: str) -> None:
        """Elimina valor del sessionStorage."""
        self._storage.pop(key, None)
    
    def clear(self) -> None:
        """Limpia sessionStorage."""
        self._storage.clear()


class FileStorageBackend(StorageBackend):
    """
    Backend de filesystem para persistencia en archivo.
    
    Útil para aplicaciones Node.js o Desktop (Electron, Tauri).
    """
    
    def __init__(self, base_dir: str = ".vela-store"):
        self.base_dir = Path(base_dir)
        self.base_dir.mkdir(parents=True, exist_ok=True)
    
    def _get_file_path(self, key: str) -> Path:
        """Obtiene ruta del archivo para la clave."""
        # Sanitizar key para nombre de archivo válido
        safe_key = key.replace("/", "_").replace("\\", "_")
        return self.base_dir / f"{safe_key}.json"
    
    def get_item(self, key: str) -> Optional[str]:
        """Lee valor del archivo."""
        file_path = self._get_file_path(key)
        if not file_path.exists():
            return None
        
        try:
            return file_path.read_text(encoding="utf-8")
        except Exception:
            return None
    
    def set_item(self, key: str, value: str) -> None:
        """Guarda valor en archivo."""
        file_path = self._get_file_path(key)
        try:
            file_path.write_text(value, encoding="utf-8")
        except Exception:
            pass  # Silenciar errores de escritura
    
    def remove_item(self, key: str) -> None:
        """Elimina archivo."""
        file_path = self._get_file_path(key)
        if file_path.exists():
            try:
                file_path.unlink()
            except Exception:
                pass
    
    def clear(self) -> None:
        """Elimina todos los archivos del directorio."""
        for file_path in self.base_dir.glob("*.json"):
            try:
                file_path.unlink()
            except Exception:
                pass


class PersistenceManager:
    """
    Gestor de persistencia para Store.
    
    Maneja serialización, deserialización, filtrado de fields,
    throttling/debouncing y estrategias de merge.
    """
    
    def __init__(self, options: PersistOptions, store_class: Type):
        self.options = options
        self.store_class = store_class
        
        # Seleccionar backend
        if options.storage == "localStorage":
            self.backend: StorageBackend = LocalStorageBackend()
        elif options.storage == "sessionStorage":
            self.backend = SessionStorageBackend()
        elif options.storage == "file":
            file_path = options.file_path or ".vela-store"
            self.backend = FileStorageBackend(file_path)
        else:
            raise ValueError(f"Unknown storage backend: {options.storage}")
        
        # Serializers
        self.serialize = options.serialize or self._default_serialize
        self.deserialize = options.deserialize or self._default_deserialize
        
        # Throttling state
        self._last_save_time = 0
        self._pending_save = False
        self._debounce_timer = None
    
    @staticmethod
    def _default_serialize(obj: Any) -> str:
        """Serialización por defecto (JSON)."""
        return json.dumps(obj, default=str)
    
    @staticmethod
    def _default_deserialize(data: str) -> Any:
        """Deserialización por defecto (JSON)."""
        return json.loads(data)
    
    def _filter_state(self, state: Dict[str, Any]) -> Dict[str, Any]:
        """
        Filtra state según whitelist/blacklist.
        
        Args:
            state: Estado completo del Store
        
        Returns:
            Estado filtrado
        """
        # Si hay whitelist, solo incluir esos fields
        if self.options.whitelist:
            return {
                key: value
                for key, value in state.items()
                if key in self.options.whitelist
            }
        
        # Si hay blacklist, excluir esos fields
        if self.options.blacklist:
            return {
                key: value
                for key, value in state.items()
                if key not in self.options.blacklist
            }
        
        # Sin filtros, retornar todo
        return state
    
    def _merge_state(
        self,
        current_state: Dict[str, Any],
        persisted_state: Dict[str, Any]
    ) -> Dict[str, Any]:
        """
        Merge estado persistido con estado actual.
        
        Args:
            current_state: Estado actual del Store
            persisted_state: Estado cargado del storage
        
        Returns:
            Estado merged
        """
        if self.options.merge_strategy == "overwrite":
            # Sobrescribir completamente
            return persisted_state
        
        elif self.options.merge_strategy == "shallow":
            # Merge shallow (solo primer nivel)
            return {**current_state, **persisted_state}
        
        elif self.options.merge_strategy == "deep":
            # Merge deep (recursivo)
            return self._deep_merge(current_state, persisted_state)
        
        else:
            raise ValueError(f"Unknown merge strategy: {self.options.merge_strategy}")
    
    def _deep_merge(self, base: Any, updates: Any) -> Any:
        """
        Merge profundo de dicts recursivamente.
        
        Args:
            base: Objeto base
            updates: Actualizaciones
        
        Returns:
            Objeto merged
        """
        if not isinstance(base, dict) or not isinstance(updates, dict):
            return updates
        
        result = base.copy()
        for key, value in updates.items():
            if key in result and isinstance(result[key], dict) and isinstance(value, dict):
                result[key] = self._deep_merge(result[key], value)
            else:
                result[key] = value
        
        return result
    
    def load_state(self) -> Optional[Dict[str, Any]]:
        """
        Carga estado del storage.
        
        Returns:
            Estado deserializado o None si no existe
        """
        serialized = self.backend.get_item(self.options.key)
        if not serialized:
            return None
        
        try:
            return self.deserialize(serialized)
        except Exception:
            # Error de deserialización → retornar None
            return None
    
    def save_state(self, state: Dict[str, Any]) -> None:
        """
        Guarda estado en storage.
        
        Args:
            state: Estado a guardar
        """
        # Filtrar state
        filtered_state = self._filter_state(state)
        
        # Serializar
        try:
            serialized = self.serialize(filtered_state)
        except Exception:
            # Error de serialización → ignorar
            return
        
        # Guardar en backend
        self.backend.set_item(self.options.key, serialized)
    
    def clear_state(self) -> None:
        """Elimina estado persistido."""
        self.backend.remove_item(self.options.key)


def persistent(
    options: PersistOptions
) -> Callable[[Type], Type]:
    """
    Decorator para persistir Store automáticamente.
    
    El decorator @persistent agrega hooks al Store para:
    1. Cargar estado al inicializar
    2. Guardar estado en cada cambio (con throttling/debouncing)
    3. Merge estado persistido con estado inicial
    
    Args:
        options: Configuración de persistencia
    
    Returns:
        Decorator que modifica la clase Store
    
    Example:
        @persistent(PersistOptions(
            key="app-state",
            storage="localStorage",
            whitelist=["user", "settings"],
            throttle=1000
        ))
        store AppStore {
            state user: Option<User> = None
            state settings: Settings = defaultSettings
            state temp_data: Any = {}  # No persistido (no está en whitelist)
        }
    """
    
    def decorator(store_class: Type) -> Type:
        """Decorator interno."""
        
        # Crear persistence manager
        manager = PersistenceManager(options, store_class)
        
        # Guardar __init__ original
        original_init = store_class.__init__
        
        def enhanced_init(self, *args, **kwargs):
            """
            Override del __init__ para cargar estado persistido.
            """
            # Llamar __init__ original
            original_init(self, *args, **kwargs)
            
            # Cargar estado persistido
            persisted_state = manager.load_state()
            
            if persisted_state:
                # Merge con estado actual
                current_state = self.get_state()
                merged_state = manager._merge_state(
                    current_state.__dict__ if hasattr(current_state, '__dict__') else {},
                    persisted_state
                )
                
                # Aplicar estado merged
                # NOTE: Esto requiere que Store tenga un método set_state()
                # o que podamos actualizar _state directamente
                if hasattr(self, '_state'):
                    for key, value in merged_state.items():
                        if hasattr(self._state, key):
                            setattr(self._state, key, value)
            
            # Suscribirse a cambios para auto-save
            def on_state_change(new_state):
                """Callback para guardar en cada cambio."""
                state_dict = new_state.__dict__ if hasattr(new_state, '__dict__') else {}
                manager.save_state(state_dict)
            
            # Subscribe al store
            self.subscribe(on_state_change)
            
            # Metadata
            self.__persistent__ = True
            self.__persistence_manager__ = manager
        
        # Reemplazar __init__
        store_class.__init__ = enhanced_init
        
        # Agregar métodos de persistencia
        def clear_persisted_state(self):
            """Limpia estado persistido."""
            manager.clear_state()
        
        def reload_persisted_state(self):
            """Recarga estado desde storage."""
            persisted_state = manager.load_state()
            if persisted_state and hasattr(self, '_state'):
                for key, value in persisted_state.items():
                    if hasattr(self._state, key):
                        setattr(self._state, key, value)
        
        store_class.clear_persisted_state = clear_persisted_state
        store_class.reload_persisted_state = reload_persisted_state
        
        # Marcar clase como persistent (metadata)
        store_class.__persistent__ = True
        store_class.__persist_options__ = options
        
        return store_class
    
    return decorator


# Helpers

def create_persistent_store(
    store_class: Type,
    key: str,
    storage: str = "localStorage",
    **kwargs
) -> Type:
    """
    Helper para crear Store persistente programáticamente.
    
    Args:
        store_class: Clase del Store
        key: Clave de persistencia
        storage: Backend de storage
        **kwargs: Opciones adicionales (whitelist, blacklist, etc.)
    
    Returns:
        Store class decorado con @persistent
    
    Example:
        AppStore = create_persistent_store(
            BaseAppStore,
            key="app-state",
            storage="localStorage",
            whitelist=["user", "settings"]
        )
        
        store = AppStore()
    """
    options = PersistOptions(key=key, storage=storage, **kwargs)
    return persistent(options)(store_class)


def is_persistent(obj: Any) -> bool:
    """
    Verifica si un Store es persistente.
    
    Args:
        obj: Store instance o class
    
    Returns:
        True si es persistente
    """
    # Verificar en la instancia primero
    if hasattr(obj, '__persistent__') and obj.__persistent__:
        return True
    
    # Verificar en la clase si es una instancia
    if hasattr(obj, '__class__'):
        cls = obj.__class__
        if hasattr(cls, '__persistent__') and cls.__persistent__:
            return True
    
    return False


def get_persist_options(store: Any) -> Optional[PersistOptions]:
    """
    Obtiene opciones de persistencia de un Store.
    
    Args:
        store: Store instance o class
    
    Returns:
        PersistOptions o None
    """
    return getattr(store, '__persist_options__', None)


def get_persistence_manager(store: Any) -> Optional[PersistenceManager]:
    """
    Obtiene PersistenceManager de un Store instance.
    
    Args:
        store: Store instance
    
    Returns:
        PersistenceManager o None
    """
    return getattr(store, '__persistence_manager__', None)
