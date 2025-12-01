"""
Sistema de Dependency Injection para Vela

Implementación de: TASK-035B, TASK-035C, TASK-035D
Historia: VELA-575
Sprint: 13

Exports públicos del módulo DI.
"""

from .scopes import Scope, DEFAULT_SCOPE
from .injectable import (
    injectable,
    InjectableMetadata,
    is_injectable,
    get_injectable_metadata,
    get_scope,
    get_token,
    register_provider,
    get_provider,
    clear_registry
)
from .inject import (
    inject,
    InjectMetadata,
    get_inject_metadata,
    set_inject_metadata,
    get_constructor_inject_metadata,
    has_inject_params,
    get_inject_token
)
from .module import (
    module,
    ModuleMetadata,
    is_module,
    get_module_metadata,
    get_module_declarations,
    get_module_controllers,
    get_module_providers,
    get_module_imports,
    get_module_exports,
    register_module,
    get_module,
    get_all_modules,
    clear_module_registry,
    find_module_by_provider,
    find_module_by_controller
)

__all__ = [
    # Scopes
    'Scope',
    'DEFAULT_SCOPE',
    
    # Decoradores
    'injectable',
    'inject',
    'module',
    
    # Metadata - @injectable
    'InjectableMetadata',
    'is_injectable',
    'get_injectable_metadata',
    'get_scope',
    'get_token',
    
    # Metadata - @inject
    'InjectMetadata',
    'get_inject_metadata',
    'set_inject_metadata',
    'get_constructor_inject_metadata',
    'has_inject_params',
    'get_inject_token',
    
    # Metadata - @module
    'ModuleMetadata',
    'is_module',
    'get_module_metadata',
    'get_module_declarations',
    'get_module_controllers',
    'get_module_providers',
    'get_module_imports',
    'get_module_exports',
    
    # Registry - @injectable
    'register_provider',
    'get_provider',
    'clear_registry',
    
    # Registry - @module
    'register_module',
    'get_module',
    'get_all_modules',
    'clear_module_registry',
    'find_module_by_provider',
    'find_module_by_controller',
]

__version__ = '0.3.0'
__author__ = 'GitHub Copilot Agent'
