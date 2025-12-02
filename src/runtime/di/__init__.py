"""
Sistema de Dependency Injection para Vela

Implementación de: TASK-035B, TASK-035C, TASK-035D, TASK-035D2, TASK-035D3, TASK-035E, TASK-035E2
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
from .pipes import (
    PipeContext,
    HTTPPipeMetadata,
    UIPipeMetadata,
    ParameterPipeMetadata,
    pipe,
    is_ui_pipe,
    is_http_pipe,
    is_parameter_pipe,
    get_ui_pipe_metadata,
    get_http_pipe_metadata,
    get_pipe_name,
    get_pipe_classes,
    validate_pipe_class,
)
from .middleware import (
    MiddlewareMetadata,
    middleware,
    is_middleware,
    get_middleware_metadata,
    get_middleware_classes,
    get_middleware_order,
    combine_middleware,
    validate_middleware_class,
)
from .guards import (
    ExecutionContext,
    GuardMetadata,
    guard,
    is_guard,
    get_guard_metadata,
    get_guard_classes,
    combine_guards,
    validate_guard_class,
)
from .injector import (
    Injector,
    Container,
    ResolutionContext,
    ProviderEntry,
    ProviderRegistry,
    InjectionError,
    CircularDependencyError,
    ProviderNotFoundError,
    InvalidScopeError,
    create_injector,
    get_global_injector,
    create_container,
)
from .lifecycle import (
    OnDisposable,
    AsyncOnDisposable,
    LifecycleHooks,
    ScopeContext,
    IsolatedScope,
    isolated_scope,
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
    
    # Pipes (TASK-035E2 - Context-Aware)
    'PipeContext',
    'HTTPPipeMetadata',
    'UIPipeMetadata',
    'ParameterPipeMetadata',
    'pipe',
    'is_ui_pipe',
    'is_http_pipe',
    'is_parameter_pipe',
    'get_ui_pipe_metadata',
    'get_http_pipe_metadata',
    'get_pipe_name',
    'get_pipe_classes',
    'validate_pipe_class',
    
    # Middleware (TASK-035E2 - Backend Only)
    'MiddlewareMetadata',
    'middleware',
    'is_middleware',
    'get_middleware_metadata',
    'get_middleware_classes',
    'get_middleware_order',
    'combine_middleware',
    'validate_middleware_class',
    
    # Guards (TASK-035E2 - Backend Only)
    'ExecutionContext',
    'GuardMetadata',
    'guard',
    'is_guard',
    'get_guard_metadata',
    'get_guard_classes',
    'combine_guards',
    'validate_guard_class',
    
    # Injector Core (TASK-035F)
    'Injector',
    'Container',
    'ResolutionContext',
    'ProviderEntry',
    'ProviderRegistry',
    'InjectionError',
    'CircularDependencyError',
    'ProviderNotFoundError',
    'InvalidScopeError',
    'create_injector',
    'get_global_injector',
    'create_container',
    
    # Lifecycle Management (TASK-035G)
    'OnDisposable',
    'AsyncOnDisposable',
    'LifecycleHooks',
    'ScopeContext',
    'IsolatedScope',
    'isolated_scope',
]

__version__ = '0.10.0'  # TASK-035G: +Lifecycle Management (disposal automático, scope hierarchy, hooks)
__author__ = 'GitHub Copilot Agent'
