"""
Sistema de Dependency Injection para Vela

Implementación de: TASK-035B, TASK-035C, TASK-035D, TASK-035D2, TASK-035D3
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
from .controller import (
    controller,
    ControllerMetadata,
    is_controller,
    get_controller_metadata,
    get_controller_base_path,
    get_controller_full_path,
    get_controller_tags,
    register_controller,
    get_controller,
    get_all_controllers,
    clear_controller_registry,
    find_controller_by_path,
    get_controllers_by_tag
)
from .http_decorators import (
    HTTPMethod,
    ParameterType,
    RouteMetadata,
    ParameterMetadata,
    ParameterMarker,
    get, post, put, patch, delete, head, options,
    param, query, body, header, cookie, request, response,
    is_route_handler,
    get_route_metadata,
    get_all_routes,
    get_routes_by_method,
    get_route_by_path,
)
from .providers import (
    ProviderScope,
    ProviderMetadata,
    provides,
    is_provider,
    get_provider_metadata,
    get_all_providers,
    get_providers_by_scope,
    get_provider_by_token,
)
from .file_decorators import (
    FileMetadata,
    FormMetadata,
    FileMarker,
    FormMarker,
    file,
    files,
    upload,
    uploads,
    form,
    is_file_parameter,
    is_form_parameter,
    get_file_metadata,
    get_form_metadata,
)

__all__ = [
    # Scopes
    'Scope',
    'DEFAULT_SCOPE',
    
    # Decoradores
    'injectable',
    'inject',
    'module',
    'controller',
    
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
    
    # Metadata - @controller
    'ControllerMetadata',
    'is_controller',
    'get_controller_metadata',
    'get_controller_base_path',
    'get_controller_full_path',
    'get_controller_tags',
    
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
    
    # Registry - @controller
    'register_controller',
    'get_controller',
    'get_all_controllers',
    'clear_controller_registry',
    'find_controller_by_path',
    'get_controllers_by_tag',
    
    # HTTP Decorators (TASK-035D3)
    'HTTPMethod',
    'ParameterType',
    'RouteMetadata',
    'ParameterMetadata',
    'ParameterMarker',
    'get', 'post', 'put', 'patch', 'delete', 'head', 'options',
    'param', 'query', 'body', 'header', 'cookie', 'request', 'response',
    'is_route_handler',
    'get_route_metadata',
    'get_all_routes',
    'get_routes_by_method',
    'get_route_by_path',
    
    # Providers (TASK-035E)
    'ProviderScope',
    'ProviderMetadata',
    'provides',
    'is_provider',
    'get_provider_metadata',
    'get_all_providers',
    'get_providers_by_scope',
    'get_provider_by_token',
    
    # File Upload Decorators (TASK-035E)
    'FileMetadata',
    'FormMetadata',
    'FileMarker',
    'FormMarker',
    'file',
    'files',
    'upload',
    'uploads',
    'form',
    'is_file_parameter',
    'is_form_parameter',
    'get_file_metadata',
    'get_form_metadata',
]

__version__ = '0.7.0'  # TASK-035E: +@provides decorator + file upload decorators
__author__ = 'GitHub Copilot Agent'
