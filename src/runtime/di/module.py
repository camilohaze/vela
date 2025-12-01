"""
Implementación del decorador @module para Vela

Este módulo implementa el decorador @module que marca una clase como un módulo
de organización (NO instanciable) con metadata de dependencias.

Implementación de: TASK-035D
Historia: VELA-575
Fecha: 2025-12-01

Características:
- ModuleMetadata dataclass con declarations, controllers, providers, imports, exports
- Decorador @module(metadata) para marcar clases como módulos
- Validación: exports ⊆ (declarations ∪ providers)
- Registry global de módulos
- Helpers para introspección y búsqueda
"""

from dataclasses import dataclass, field
from typing import Any, Optional, Type, Dict, List


# ============================================================================
# METADATA
# ============================================================================

@dataclass
class ModuleMetadata:
    """
    Metadata de un módulo Vela.
    
    Un módulo es una unidad de organización que agrupa declaraciones,
    controllers, providers, imports y exports.
    
    Reglas de validación:
    - exports ⊆ (declarations ∪ providers)
    - imports debe contener solo clases con @module
    - declarations puede contener widgets, services, etc.
    - controllers solo para backend (HTTP endpoints)
    - providers son inyectables (services, repositories, etc.)
    """
    declarations: List[Type] = field(default_factory=list)
    controllers: List[Type] = field(default_factory=list)
    providers: List[Type] = field(default_factory=list)
    imports: List[Type] = field(default_factory=list)
    exports: List[Type] = field(default_factory=list)
    
    def __post_init__(self):
        """Validar metadata después de inicialización."""
        # Convertir a lists si no lo son
        self.declarations = list(self.declarations) if self.declarations else []
        self.controllers = list(self.controllers) if self.controllers else []
        self.providers = list(self.providers) if self.providers else []
        self.imports = list(self.imports) if self.imports else []
        self.exports = list(self.exports) if self.exports else []
        
        # Validar exports ⊆ (declarations ∪ providers)
        valid_exports = set(self.declarations) | set(self.providers)
        invalid_exports = set(self.exports) - valid_exports
        
        if invalid_exports:
            invalid_names = [cls.__name__ for cls in invalid_exports]
            raise ValueError(
                f"Invalid exports: {invalid_names}. "
                f"Exports must be a subset of declarations or providers."
            )
    
    def get_all_providers(self) -> List[Type]:
        """
        Obtiene todos los providers (declarations + providers).
        
        Returns:
            Lista de todas las clases que pueden ser inyectadas
        """
        # Combinar declarations y providers (sin duplicados)
        all_providers = list(set(self.declarations) | set(self.providers))
        return all_providers
    
    def get_exported_providers(self) -> List[Type]:
        """
        Obtiene solo los providers exportados.
        
        Returns:
            Lista de clases exportadas
        """
        return self.exports.copy()
    
    def has_controller(self, controller_cls: Type) -> bool:
        """
        Verifica si el módulo tiene un controller específico.
        
        Args:
            controller_cls: Clase del controller a verificar
            
        Returns:
            True si el controller está en la lista
        """
        return controller_cls in self.controllers
    
    def has_provider(self, provider_cls: Type) -> bool:
        """
        Verifica si el módulo tiene un provider específico.
        
        Args:
            provider_cls: Clase del provider a verificar
            
        Returns:
            True si el provider está en declarations o providers
        """
        return provider_cls in self.declarations or provider_cls in self.providers


# ============================================================================
# DECORADOR @module
# ============================================================================

# Atributo privado donde se guarda la metadata
_MODULE_METADATA_ATTR = "__module_metadata__"


def module(
    declarations: Optional[List[Type]] = None,
    controllers: Optional[List[Type]] = None,
    providers: Optional[List[Type]] = None,
    imports: Optional[List[Type]] = None,
    exports: Optional[List[Type]] = None
):
    """
    Decorador @module para marcar una clase como módulo de organización.
    
    Un módulo NO es instanciable. Es una unidad de organización que agrupa
    declarations, controllers, providers, imports y exports.
    
    Uso:
        @module(
            declarations=[MyWidget, MyService],
            controllers=[UserController],
            providers=[UserService],
            imports=[CommonModule],
            exports=[MyService]
        )
        class MyModule:
            pass
    
    Args:
        declarations: Lista de widgets, services, etc. (frontend)
        controllers: Lista de controllers HTTP (backend)
        providers: Lista de services, repositories (inyectables)
        imports: Lista de otros módulos a importar
        exports: Lista de providers a exportar (debe ser subconjunto de declarations ∪ providers)
        
    Returns:
        Decorador que agrega metadata al módulo
        
    Raises:
        ValueError: Si exports no es subconjunto de (declarations ∪ providers)
    """
    def decorator(cls: Type) -> Type:
        # Crear metadata
        metadata = ModuleMetadata(
            declarations=declarations or [],
            controllers=controllers or [],
            providers=providers or [],
            imports=imports or [],
            exports=exports or []
        )
        
        # Agregar metadata a la clase
        setattr(cls, _MODULE_METADATA_ATTR, metadata)
        
        # Auto-registrar el módulo en el registry global
        register_module(cls, metadata)
        
        return cls
    
    return decorator


# ============================================================================
# HELPERS - INTROSPECCIÓN
# ============================================================================

def is_module(cls: Type) -> bool:
    """
    Verifica si una clase es un módulo (tiene decorador @module).
    
    Args:
        cls: Clase a verificar
        
    Returns:
        True si la clase tiene @module, False en caso contrario
        
    Example:
        @module(declarations=[MyService])
        class MyModule:
            pass
        
        is_module(MyModule)  # True
        is_module(str)       # False
    """
    return hasattr(cls, _MODULE_METADATA_ATTR)


def get_module_metadata(cls: Type) -> Optional[ModuleMetadata]:
    """
    Obtiene la metadata de un módulo.
    
    Args:
        cls: Clase del módulo
        
    Returns:
        ModuleMetadata si la clase es un módulo, None en caso contrario
        
    Example:
        @module(declarations=[MyService])
        class MyModule:
            pass
        
        metadata = get_module_metadata(MyModule)
        print(metadata.declarations)  # [MyService]
    """
    if not is_module(cls):
        return None
    
    return getattr(cls, _MODULE_METADATA_ATTR)


def get_module_declarations(cls: Type) -> List[Type]:
    """
    Obtiene las declarations de un módulo.
    
    Args:
        cls: Clase del módulo
        
    Returns:
        Lista de declarations o lista vacía si no es módulo
    """
    metadata = get_module_metadata(cls)
    return metadata.declarations if metadata else []


def get_module_controllers(cls: Type) -> List[Type]:
    """
    Obtiene los controllers de un módulo.
    
    Args:
        cls: Clase del módulo
        
    Returns:
        Lista de controllers o lista vacía si no es módulo
    """
    metadata = get_module_metadata(cls)
    return metadata.controllers if metadata else []


def get_module_providers(cls: Type) -> List[Type]:
    """
    Obtiene los providers de un módulo.
    
    Args:
        cls: Clase del módulo
        
    Returns:
        Lista de providers o lista vacía si no es módulo
    """
    metadata = get_module_metadata(cls)
    return metadata.providers if metadata else []


def get_module_imports(cls: Type) -> List[Type]:
    """
    Obtiene los imports de un módulo.
    
    Args:
        cls: Clase del módulo
        
    Returns:
        Lista de imports o lista vacía si no es módulo
    """
    metadata = get_module_metadata(cls)
    return metadata.imports if metadata else []


def get_module_exports(cls: Type) -> List[Type]:
    """
    Obtiene los exports de un módulo.
    
    Args:
        cls: Clase del módulo
        
    Returns:
        Lista de exports o lista vacía si no es módulo
    """
    metadata = get_module_metadata(cls)
    return metadata.exports if metadata else []


# ============================================================================
# MODULE REGISTRY
# ============================================================================

# Registry global: module_class -> ModuleMetadata
_module_registry: Dict[Type, ModuleMetadata] = {}


def register_module(module_cls: Type, metadata: ModuleMetadata) -> None:
    """
    Registra un módulo en el registry global.
    
    Args:
        module_cls: Clase del módulo
        metadata: Metadata del módulo
        
    Example:
        metadata = ModuleMetadata(declarations=[MyService])
        register_module(MyModule, metadata)
    """
    _module_registry[module_cls] = metadata


def get_module(module_cls: Type) -> Optional[ModuleMetadata]:
    """
    Obtiene la metadata de un módulo desde el registry.
    
    Args:
        module_cls: Clase del módulo
        
    Returns:
        ModuleMetadata si el módulo está registrado, None en caso contrario
        
    Example:
        metadata = get_module(MyModule)
        if metadata:
            print(metadata.declarations)
    """
    return _module_registry.get(module_cls)


def get_all_modules() -> Dict[Type, ModuleMetadata]:
    """
    Obtiene todos los módulos registrados.
    
    Returns:
        Diccionario de module_class -> ModuleMetadata
        
    Example:
        all_modules = get_all_modules()
        for module_cls, metadata in all_modules.items():
            print(f"Module: {module_cls.__name__}")
    """
    return _module_registry.copy()


def clear_module_registry() -> None:
    """
    Limpia el registry de módulos (útil para tests).
    
    Example:
        clear_module_registry()
        assert len(get_all_modules()) == 0
    """
    _module_registry.clear()


def find_module_by_provider(provider_cls: Type) -> Optional[Type]:
    """
    Encuentra el módulo que contiene un provider específico.
    
    Args:
        provider_cls: Clase del provider a buscar
        
    Returns:
        Clase del módulo que contiene el provider, None si no se encuentra
        
    Example:
        module = find_module_by_provider(MyService)
        if module:
            print(f"Found in module: {module.__name__}")
    """
    for module_cls, metadata in _module_registry.items():
        if metadata.has_provider(provider_cls):
            return module_cls
    return None


def find_module_by_controller(controller_cls: Type) -> Optional[Type]:
    """
    Encuentra el módulo que contiene un controller específico.
    
    Args:
        controller_cls: Clase del controller a buscar
        
    Returns:
        Clase del módulo que contiene el controller, None si no se encuentra
        
    Example:
        module = find_module_by_controller(UserController)
        if module:
            print(f"Found in module: {module.__name__}")
    """
    for module_cls, metadata in _module_registry.items():
        if metadata.has_controller(controller_cls):
            return module_cls
    return None


# ============================================================================
# TESTS BÁSICOS (para verificación)
# ============================================================================

if __name__ == "__main__":
    # Test 1: Crear módulo básico
    @module(
        declarations=[str],
        providers=[int],
        exports=[str, int]
    )
    class TestModule:
        pass
    
    assert is_module(TestModule), "TestModule debe ser módulo"
    metadata = get_module_metadata(TestModule)
    assert metadata is not None, "Metadata no debe ser None"
    assert str in metadata.declarations, "str debe estar en declarations"
    assert int in metadata.providers, "int debe estar en providers"
    assert str in metadata.exports, "str debe estar en exports"
    
    # Test 2: Validación exports ⊆ (declarations ∪ providers)
    try:
        @module(
            declarations=[str],
            exports=[int]  # int NO está en declarations
        )
        class InvalidModule:
            pass
        assert False, "Debería lanzar ValueError"
    except ValueError as e:
        assert "Invalid exports" in str(e), "Debe mencionar invalid exports"
    
    # Test 3: Registry
    assert TestModule in get_all_modules(), "TestModule debe estar en registry"
    assert get_module(TestModule) == metadata, "Metadata debe coincidir"
    
    # Test 4: find_module_by_provider
    found = find_module_by_provider(str)
    assert found == TestModule, "Debe encontrar TestModule por str"
    
    print("✅ Todos los tests básicos pasaron")
