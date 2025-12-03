"""
Pytest fixtures for Workers + Channels tests.

Ensures WorkerPool is properly initialized and cleaned up between tests.
"""

import sys
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

import pytest
from src.runtime.workers.worker_pool import WorkerPool


@pytest.fixture
def reset_worker_pool():
    """Reset WorkerPool before and after test. Use explicitly when needed."""
    # Reset before test
    WorkerPool._global_pool = None
    
    yield
    
    # Cleanup after test - shutdown pool if exists
    if WorkerPool._global_pool is not None:
        pool = WorkerPool._global_pool
        if not pool._shutdown:
            pool.shutdown()
        WorkerPool._global_pool = None
