"""
Tests unitarios para @persistent decorator

Jira: VELA-577 - TASK-035X
Historia: State Management
"""

import pytest
import json
from dataclasses import dataclass
from typing import Optional, List
from src.reactive.persistent import (
    PersistOptions,
    LocalStorageBackend,
    SessionStorageBackend,
    FileStorageBackend,
    PersistenceManager,
    persistent,
    create_persistent_store,
    is_persistent,
    get_persist_options,
    get_persistence_manager
)


# Mock Classes

@dataclass
class User:
    """Usuario mock."""
    id: int
    name: str
    email: str


@dataclass
class Settings:
    """Settings mock."""
    theme: str
    language: str
    notifications: bool


@dataclass
class AppState:
    """Estado mock de la aplicación."""
    user: Optional[User] = None
    settings: Settings = None
    temp_data: dict = None
    counter: int = 0
    
    def __post_init__(self):
        if self.settings is None:
            self.settings = Settings(
                theme="light",
                language="en",
                notifications=True
            )
        if self.temp_data is None:
            self.temp_data = {}


class MockStore:
    """
    Mock de Store<T> para testing.
    
    Implementa la interfaz mínima requerida por @persistent.
    """
    
    def __init__(self):
        self._state = AppState()
        self._subscribers = []
    
    def get_state(self) -> AppState:
        """Obtiene estado actual."""
        return self._state
    
    def subscribe(self, callback):
        """Suscribe a cambios de estado."""
        self._subscribers.append(callback)
        return lambda: self._subscribers.remove(callback)
    
    def _notify(self):
        """Notifica subscribers."""
        for callback in self._subscribers:
            callback(self._state)
    
    def update_state(self, **kwargs):
        """Actualiza estado."""
        for key, value in kwargs.items():
            if hasattr(self._state, key):
                setattr(self._state, key, value)
        self._notify()


# Tests de Storage Backends

class TestLocalStorageBackend:
    """Tests para LocalStorageBackend."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.backend = LocalStorageBackend()
    
    def test_set_and_get_item(self):
        """Test de set_item y get_item."""
        self.backend.set_item("key1", "value1")
        assert self.backend.get_item("key1") == "value1"
    
    def test_get_nonexistent_item(self):
        """Test de get_item con clave inexistente."""
        assert self.backend.get_item("nonexistent") is None
    
    def test_remove_item(self):
        """Test de remove_item."""
        self.backend.set_item("key1", "value1")
        self.backend.remove_item("key1")
        assert self.backend.get_item("key1") is None
    
    def test_clear(self):
        """Test de clear."""
        self.backend.set_item("key1", "value1")
        self.backend.set_item("key2", "value2")
        self.backend.clear()
        assert self.backend.get_item("key1") is None
        assert self.backend.get_item("key2") is None


class TestSessionStorageBackend:
    """Tests para SessionStorageBackend."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.backend = SessionStorageBackend()
    
    def test_set_and_get_item(self):
        """Test de set_item y get_item."""
        self.backend.set_item("session-key", "session-value")
        assert self.backend.get_item("session-key") == "session-value"
    
    def test_clear(self):
        """Test de clear."""
        self.backend.set_item("key1", "value1")
        self.backend.clear()
        assert self.backend.get_item("key1") is None


class TestFileStorageBackend:
    """Tests para FileStorageBackend."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.backend = FileStorageBackend(base_dir=".test-storage")
    
    def teardown_method(self):
        """Cleanup después de cada test."""
        self.backend.clear()
        import shutil
        try:
            shutil.rmtree(".test-storage")
        except:
            pass
    
    def test_set_and_get_item(self):
        """Test de set_item y get_item."""
        self.backend.set_item("file-key", "file-value")
        assert self.backend.get_item("file-key") == "file-value"
    
    def test_get_nonexistent_item(self):
        """Test de get_item con archivo inexistente."""
        assert self.backend.get_item("nonexistent") is None
    
    def test_remove_item(self):
        """Test de remove_item."""
        self.backend.set_item("file-key", "file-value")
        self.backend.remove_item("file-key")
        assert self.backend.get_item("file-key") is None
    
    def test_clear(self):
        """Test de clear."""
        self.backend.set_item("key1", "value1")
        self.backend.set_item("key2", "value2")
        self.backend.clear()
        assert self.backend.get_item("key1") is None
        assert self.backend.get_item("key2") is None


# Tests de PersistenceManager

class TestPersistenceManager:
    """Tests para PersistenceManager."""
    
    def setup_method(self):
        """Setup para cada test."""
        self.options = PersistOptions(
            key="test-app",
            storage="localStorage"
        )
        self.manager = PersistenceManager(self.options, MockStore)
    
    def test_initialization(self):
        """Test de inicialización del manager."""
        assert self.manager.options.key == "test-app"
        assert isinstance(self.manager.backend, LocalStorageBackend)
    
    def test_filter_state_with_whitelist(self):
        """Test de filtrado con whitelist."""
        options = PersistOptions(
            key="test",
            whitelist=["user", "settings"]
        )
        manager = PersistenceManager(options, MockStore)
        
        state = {
            "user": {"id": 1},
            "settings": {"theme": "dark"},
            "temp_data": {"cached": True},
            "counter": 42
        }
        
        filtered = manager._filter_state(state)
        assert "user" in filtered
        assert "settings" in filtered
        assert "temp_data" not in filtered
        assert "counter" not in filtered
    
    def test_filter_state_with_blacklist(self):
        """Test de filtrado con blacklist."""
        options = PersistOptions(
            key="test",
            blacklist=["temp_data"]
        )
        manager = PersistenceManager(options, MockStore)
        
        state = {
            "user": {"id": 1},
            "settings": {"theme": "dark"},
            "temp_data": {"cached": True},
            "counter": 42
        }
        
        filtered = manager._filter_state(state)
        assert "user" in filtered
        assert "settings" in filtered
        assert "temp_data" not in filtered
        assert "counter" in filtered
    
    def test_merge_shallow(self):
        """Test de merge shallow."""
        options = PersistOptions(
            key="test",
            merge_strategy="shallow"
        )
        manager = PersistenceManager(options, MockStore)
        
        current = {"a": 1, "b": {"x": 1}}
        persisted = {"b": {"y": 2}, "c": 3}
        
        merged = manager._merge_state(current, persisted)
        
        assert merged["a"] == 1
        assert merged["b"] == {"y": 2}  # Shallow merge
        assert merged["c"] == 3
    
    def test_merge_deep(self):
        """Test de merge deep."""
        options = PersistOptions(
            key="test",
            merge_strategy="deep"
        )
        manager = PersistenceManager(options, MockStore)
        
        current = {"a": 1, "b": {"x": 1, "z": 3}}
        persisted = {"b": {"y": 2}, "c": 3}
        
        merged = manager._merge_state(current, persisted)
        
        assert merged["a"] == 1
        assert merged["b"]["x"] == 1  # Deep merge
        assert merged["b"]["y"] == 2
        assert merged["b"]["z"] == 3
        assert merged["c"] == 3
    
    def test_merge_overwrite(self):
        """Test de merge overwrite."""
        options = PersistOptions(
            key="test",
            merge_strategy="overwrite"
        )
        manager = PersistenceManager(options, MockStore)
        
        current = {"a": 1, "b": 2}
        persisted = {"c": 3}
        
        merged = manager._merge_state(current, persisted)
        
        assert "a" not in merged
        assert "b" not in merged
        assert merged["c"] == 3
    
    def test_save_and_load_state(self):
        """Test de save_state y load_state."""
        state = {
            "user": {"id": 1, "name": "Alice"},
            "counter": 42
        }
        
        self.manager.save_state(state)
        loaded = self.manager.load_state()
        
        assert loaded is not None
        assert loaded["user"]["id"] == 1
        assert loaded["user"]["name"] == "Alice"
        assert loaded["counter"] == 42
    
    def test_load_nonexistent_state(self):
        """Test de load_state cuando no existe estado."""
        loaded = self.manager.load_state()
        assert loaded is None
    
    def test_clear_state(self):
        """Test de clear_state."""
        state = {"counter": 42}
        self.manager.save_state(state)
        
        self.manager.clear_state()
        loaded = self.manager.load_state()
        
        assert loaded is None


# Tests de @persistent decorator

class TestPersistentDecorator:
    """Tests para decorator @persistent."""
    
    def test_decorator_marks_class(self):
        """Test que @persistent marca la clase."""
        options = PersistOptions(key="test-store")
        
        @persistent(options)
        class TestStore(MockStore):
            pass
        
        assert hasattr(TestStore, '__persistent__')
        assert TestStore.__persistent__ is True
        assert hasattr(TestStore, '__persist_options__')
        assert TestStore.__persist_options__.key == "test-store"
    
    def test_decorator_adds_methods(self):
        """Test que @persistent agrega métodos."""
        options = PersistOptions(key="test-store")
        
        @persistent(options)
        class TestStore(MockStore):
            pass
        
        store = TestStore()
        
        assert hasattr(store, 'clear_persisted_state')
        assert hasattr(store, 'reload_persisted_state')
        assert callable(store.clear_persisted_state)
        assert callable(store.reload_persisted_state)
    
    def test_auto_save_on_state_change(self):
        """Test de auto-save en cambio de estado."""
        options = PersistOptions(key="test-auto-save")
        
        @persistent(options)
        class TestStore(MockStore):
            pass
        
        store = TestStore()
        manager = get_persistence_manager(store)
        
        # Cambiar estado
        store.update_state(counter=99)
        
        # Verificar que se guardó
        loaded = manager.load_state()
        assert loaded is not None
        assert loaded.get("counter") == 99
    
    def test_restore_on_init(self):
        """Test de restauración al inicializar."""
        options = PersistOptions(key="test-restore")
        
        @persistent(options)
        class TestStore(MockStore):
            pass
        
        # Primera instancia: guardar estado
        store1 = TestStore()
        store1.update_state(counter=123)
        
        # Segunda instancia: debe restaurar
        store2 = TestStore()
        assert store2.get_state().counter == 123
    
    def test_whitelist_filter(self):
        """Test de filtrado con whitelist."""
        options = PersistOptions(
            key="test-whitelist",
            whitelist=["counter"]
        )
        
        @persistent(options)
        class TestStore(MockStore):
            pass
        
        store = TestStore()
        store.update_state(
            counter=42,
            temp_data={"should_not_persist": True}
        )
        
        manager = get_persistence_manager(store)
        loaded = manager.load_state()
        
        assert "counter" in loaded
        assert "temp_data" not in loaded
    
    def test_blacklist_filter(self):
        """Test de filtrado con blacklist."""
        options = PersistOptions(
            key="test-blacklist",
            blacklist=["temp_data"]
        )
        
        @persistent(options)
        class TestStore(MockStore):
            pass
        
        store = TestStore()
        store.update_state(
            counter=42,
            temp_data={"should_not_persist": True}
        )
        
        manager = get_persistence_manager(store)
        loaded = manager.load_state()
        
        assert "counter" in loaded
        assert "temp_data" not in loaded
    
    def test_clear_persisted_state(self):
        """Test del método clear_persisted_state."""
        options = PersistOptions(key="test-clear")
        
        @persistent(options)
        class TestStore(MockStore):
            pass
        
        store = TestStore()
        store.update_state(counter=42)
        
        # Limpiar estado persistido
        store.clear_persisted_state()
        
        # Verificar que se eliminó
        manager = get_persistence_manager(store)
        loaded = manager.load_state()
        assert loaded is None


# Tests de Helper Functions

class TestHelperFunctions:
    """Tests para funciones helper."""
    
    def test_create_persistent_store(self):
        """Test de create_persistent_store."""
        StoreClass = create_persistent_store(
            MockStore,
            key="helper-test",
            storage="localStorage"
        )
        
        assert hasattr(StoreClass, '__persistent__')
        assert StoreClass.__persistent__ is True
        
        store = StoreClass()
        assert is_persistent(store)
    
    def test_is_persistent(self):
        """Test de is_persistent."""
        options = PersistOptions(key="test")
        
        @persistent(options)
        class PersistentStore(MockStore):
            pass
        
        # Crear clase completamente nueva sin herencia de persistent
        class NonPersistentStore:
            def __init__(self):
                self._state = AppState()
        
        persistent_store = PersistentStore()
        non_persistent_store = NonPersistentStore()
        
        assert is_persistent(persistent_store) is True
        assert is_persistent(non_persistent_store) is False
    
    def test_get_persist_options(self):
        """Test de get_persist_options."""
        options = PersistOptions(
            key="test-options",
            storage="localStorage",
            whitelist=["user"]
        )
        
        @persistent(options)
        class TestStore(MockStore):
            pass
        
        store = TestStore()
        retrieved_options = get_persist_options(store)
        
        assert retrieved_options is not None
        assert retrieved_options.key == "test-options"
        assert retrieved_options.storage == "localStorage"
        assert retrieved_options.whitelist == ["user"]
    
    def test_get_persistence_manager(self):
        """Test de get_persistence_manager."""
        options = PersistOptions(key="test-manager")
        
        @persistent(options)
        class TestStore(MockStore):
            pass
        
        store = TestStore()
        manager = get_persistence_manager(store)
        
        assert manager is not None
        assert isinstance(manager, PersistenceManager)
        assert manager.options.key == "test-manager"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
