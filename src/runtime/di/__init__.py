"""
Sistema de Dependency Injection para Vela

Implementación de: TASK-035B (en progreso)
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

__all__ = [
    # Scopes
    'Scope',
    'DEFAULT_SCOPE',
    
    # Decoradores
    'injectable',
    
    # Metadata
    'InjectableMetadata',
    'is_injectable',
    'get_injectable_metadata',
    'get_scope',
    'get_token',
    
    # Registry
    'register_provider',
    'get_provider',
    'clear_registry',
]

__version__ = '0.1.0'
__author__ = 'GitHub Copilot Agent'
