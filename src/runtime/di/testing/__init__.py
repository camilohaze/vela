"""
Testing utilities for Dependency Injection system.

Provides:
- TestInjector: Extended Injector for testing with override capabilities
- @mock: Decorator for creating mock providers
- TestContainer: Isolated test environment with auto-cleanup
- pytest fixtures: Reusable testing fixtures

Jira: TASK-035I
Historia: VELA-575
"""

from .test_injector import TestInjector
from .mock import mock
from .container import TestContainer, create_test_container
from .fixtures import injector, test_container, module_injector

__all__ = [
    # Core testing classes
    'TestInjector',
    'TestContainer',
    
    # Decorators
    'mock',
    
    # Factory functions
    'create_test_container',
    
    # pytest fixtures
    'injector',
    'test_container',
    'module_injector',
]
