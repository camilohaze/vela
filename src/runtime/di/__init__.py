"""
Sistema de Dependency Injection para Vela

Implementación de: TASK-035B, TASK-035C
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

__all__ = [
    # Scopes
    'Scope',
    'DEFAULT_SCOPE',
    
    # Decoradores
    'injectable',
    'inject',
    
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
    
    # Registry
    'register_provider',
    'get_provider',
    'clear_registry',
]

__version__ = '0.2.0'
__author__ = 'GitHub Copilot Agent'
