"""
Memoization System for Reactive Computed Values

Implementación de: VELA-574 - TASK-033
Historia: Sistema de Memoization
Fecha: 2025-12-01

Descripción:
Sistema de memoization para evitar recomputes redundantes de computed values.
Características:
- MemoCache con LRU eviction
- Invalidación inteligente basada en dependencies
- WeakKeyDictionary para auto-cleanup
- TTL opcional para cache entries
- Integration con Computed class
"""

from typing import Any, Callable, Dict, Optional, TypeVar, Tuple, Set
from collections import OrderedDict
from weakref import WeakKeyDictionary
import time
from functools import wraps

if False:  # TYPE_CHECKING
    from .computed import Computed


T = TypeVar('T')


class MemoCache:
    """
    Cache LRU (Least Recently Used) para computed values.
    
    Características:
    - Capacidad máxima configurable
    - Eviction automática de entradas menos usadas
    - TTL (Time To Live) opcional
    - Estadísticas de hit/miss
    
    Attributes:
        max_size: Capacidad máxima del cache (default: 1000)
        ttl: Time to live en segundos (None = sin expiración)
        _cache: OrderedDict para LRU
        _timestamps: Timestamps de cada entrada
        _hits: Contador de cache hits
        _misses: Contador de cache misses
    """
    
    def __init__(self, max_size: int = 1000, ttl: Optional[float] = None):
        """
        Inicializa el cache.
        
        Args:
            max_size: Capacidad máxima (default: 1000)
            ttl: Time to live en segundos (None = sin expiración)
        """
        self.max_size = max_size
        self.ttl = ttl
        self._cache: OrderedDict[Tuple[int, ...], Any] = OrderedDict()
        self._timestamps: Dict[Tuple[int, ...], float] = {}
        self._hits = 0
        self._misses = 0
    
    def get(self, key: Tuple[int, ...]) -> Optional[Any]:
        """
        Obtiene valor del cache.
        
        Args:
            key: Tupla de dependency hashes
            
        Returns:
            Optional[Any]: Valor cacheado o None si no existe/expiró
        """
        if key not in self._cache:
            self._misses += 1
            return None
        
        # Verificar TTL si está configurado
        if self.ttl is not None:
            timestamp = self._timestamps.get(key, 0)
            if time.time() - timestamp > self.ttl:
                # Expiró, eliminar entrada
                del self._cache[key]
                del self._timestamps[key]
                self._misses += 1
                return None
        
        # Cache hit: mover al final (LRU)
        self._cache.move_to_end(key)
        self._hits += 1
        return self._cache[key]
    
    def set(self, key: Tuple[int, ...], value: Any) -> None:
        """
        Almacena valor en el cache.
        
        Args:
            key: Tupla de dependency hashes
            value: Valor a cachear
        """
        # Si ya existe, actualizar y mover al final
        if key in self._cache:
            self._cache.move_to_end(key)
            self._cache[key] = value
            self._timestamps[key] = time.time()
            return
        
        # Si está lleno, eliminar entrada más antigua (LRU)
        if len(self._cache) >= self.max_size:
            oldest_key = next(iter(self._cache))
            del self._cache[oldest_key]
            if oldest_key in self._timestamps:
                del self._timestamps[oldest_key]
        
        # Agregar nueva entrada
        self._cache[key] = value
        self._timestamps[key] = time.time()
    
    def invalidate(self, key: Tuple[int, ...]) -> bool:
        """
        Invalida una entrada del cache.
        
        Args:
            key: Tupla de dependency hashes
            
        Returns:
            bool: True si se eliminó, False si no existía
        """
        if key in self._cache:
            del self._cache[key]
            if key in self._timestamps:
                del self._timestamps[key]
            return True
        return False
    
    def clear(self) -> None:
        """Limpia completamente el cache."""
        self._cache.clear()
        self._timestamps.clear()
        self._hits = 0
        self._misses = 0
    
    def size(self) -> int:
        """Retorna el tamaño actual del cache."""
        return len(self._cache)
    
    def stats(self) -> Dict[str, Any]:
        """
        Retorna estadísticas del cache.
        
        Returns:
            Dict con hits, misses, size, hit_rate (fracción 0.0-1.0)
        """
        total = self._hits + self._misses
        hit_rate = (self._hits / total) if total > 0 else 0.0
        
        return {
            'hits': self._hits,
            'misses': self._misses,
            'size': len(self._cache),
            'max_size': self.max_size,
            'hit_rate': hit_rate,
            'ttl': self.ttl,
        }
    
    def __repr__(self) -> str:
        """String representation."""
        stats = self.stats()
        hit_rate_pct = stats['hit_rate'] * 100
        return (f"MemoCache(size={stats['size']}, max_size={stats['max_size']}, "
                f"hits={stats['hits']}, misses={stats['misses']}, "
                f"hit_rate={hit_rate_pct:.1f}%)")


class MemoizationManager:
    """
    Manager global de memoization para computed values.
    
    Mantiene un WeakKeyDictionary que mapea Computed -> MemoCache.
    Cuando un Computed es garbage collected, su cache se limpia automáticamente.
    
    Attributes:
        _caches: WeakKeyDictionary[Computed, MemoCache]
        _enabled: Flag global para habilitar/deshabilitar memoization
    """
    
    def __init__(self):
        """Inicializa el manager."""
        self._caches: WeakKeyDictionary = WeakKeyDictionary()
        self._enabled = True
    
    def get_cache(self, computed: 'Computed', create: bool = True) -> Optional[MemoCache]:
        """
        Obtiene el cache de un computed.
        
        Args:
            computed: Instancia de Computed
            create: Si True, crea cache si no existe
            
        Returns:
            Optional[MemoCache]: Cache del computed o None
        """
        if computed not in self._caches and create:
            self._caches[computed] = MemoCache()
        return self._caches.get(computed)
    
    def invalidate_computed(self, computed: 'Computed') -> bool:
        """
        Invalida completamente el cache de un computed.
        
        Args:
            computed: Instancia de Computed
            
        Returns:
            bool: True si se invalidó, False si no tenía cache
        """
        if computed in self._caches:
            self._caches[computed].clear()
            return True
        return False
    
    def enable(self) -> None:
        """Habilita memoization globalmente."""
        self._enabled = True
    
    def disable(self) -> None:
        """Deshabilita memoization globalmente."""
        self._enabled = False
    
    def is_enabled(self) -> bool:
        """Retorna si memoization está habilitado."""
        return self._enabled
    
    def clear_all(self) -> None:
        """Limpia todos los caches."""
        for cache in self._caches.values():
            cache.clear()
    
    def stats(self) -> Dict[str, Any]:
        """
        Retorna estadísticas globales de memoization.
        
        Returns:
            Dict con total_caches, total_hits, total_misses, etc.
        """
        total_hits = 0
        total_misses = 0
        total_size = 0
        
        for cache in self._caches.values():
            cache_stats = cache.stats()
            total_hits += cache_stats['hits']
            total_misses += cache_stats['misses']
            total_size += cache_stats['size']
        
        total = total_hits + total_misses
        hit_rate = (total_hits / total) if total > 0 else 0.0
        
        return {
            'total_caches': len(self._caches),
            'total_hits': total_hits,
            'total_misses': total_misses,
            'total_size': total_size,
            'hit_rate': hit_rate,
            'enabled': self._enabled,
        }
    
    def __repr__(self) -> str:
        """String representation."""
        stats = self.stats()
        hit_rate_pct = stats['hit_rate'] * 100
        return (f"MemoizationManager(caches={stats['total_caches']}, "
                f"size={stats['total_size']}, hit_rate={hit_rate_pct:.1f}%, "
                f"enabled={stats['enabled']})")


# Global memoization manager
_global_memo_manager = MemoizationManager()


def get_memo_manager() -> MemoizationManager:
    """
    Obtiene el manager global de memoization.
    
    Returns:
        MemoizationManager: Manager global
    """
    return _global_memo_manager


def compute_cache_key(computed: 'Computed') -> Tuple[int, ...]:
    """
    Computa la cache key basada en los valores de las dependencies.
    
    Args:
        computed: Instancia de Computed
        
    Returns:
        Tuple[int, ...]: Tupla de hashes de dependency values
    """
    key_parts = []
    
    # Iterar sobre dependencies del computed
    for dep in computed._node.dependencies:
        # Hash del valor actual de la dependencia
        try:
            value_hash = hash(dep.value)
        except TypeError:
            # Si el valor no es hashable, usar id del objeto
            value_hash = id(dep.value)
        
        key_parts.append(value_hash)
    
    return tuple(key_parts)


def memoize(max_size: int = 1000, ttl: Optional[float] = None):
    """
    Decorador para habilitar memoization en computed values.
    
    Args:
        max_size: Capacidad máxima del cache (default: 1000)
        ttl: Time to live en segundos (None = sin expiración)
        
    Returns:
        Callable: Decorador
        
    Example:
        >>> @memoize(max_size=500, ttl=60.0)
        ... def expensive_computed():
        ...     return expensive_calculation()
    """
    def decorator(fn: Callable[[], T]) -> Callable[[], T]:
        @wraps(fn)
        def wrapper(*args, **kwargs) -> T:
            return fn(*args, **kwargs)
        
        # Marcar función como memoizable
        wrapper._memoize_config = {  # type: ignore
            'max_size': max_size,
            'ttl': ttl,
        }
        
        return wrapper
    
    return decorator


# Exports
__all__ = [
    'MemoCache',
    'MemoizationManager',
    'get_memo_manager',
    'compute_cache_key',
    'memoize',
]


# Demo en __main__
if __name__ == "__main__":
    print("=== Memoization System Demo ===\n")
    
    # 1. MemoCache básico
    print("1. MemoCache básico:")
    cache = MemoCache(max_size=3)
    
    # Set values
    cache.set((1, 2, 3), "result1")
    cache.set((4, 5, 6), "result2")
    cache.set((7, 8, 9), "result3")
    
    print(f"Cache: {cache}")
    print(f"Get (1,2,3): {cache.get((1, 2, 3))}")  # Hit
    print(f"Get (99,): {cache.get((99,))}")  # Miss
    print(f"Stats: {cache.stats()}\n")
    
    # 2. LRU eviction
    print("2. LRU Eviction:")
    cache.set((10, 11, 12), "result4")  # Evict oldest (4,5,6)
    print(f"Get (4,5,6) after eviction: {cache.get((4, 5, 6))}")  # Miss
    print(f"Cache: {cache}\n")
    
    # 3. TTL expiration
    print("3. TTL Expiration:")
    cache_ttl = MemoCache(max_size=10, ttl=0.1)  # 100ms TTL
    cache_ttl.set((1,), "expires soon")
    print(f"Get immediately: {cache_ttl.get((1,))}")  # Hit
    time.sleep(0.15)  # Wait for expiration
    print(f"Get after TTL: {cache_ttl.get((1,))}")  # Miss (expired)
    print(f"Stats: {cache_ttl.stats()}\n")
    
    # 4. MemoizationManager
    print("4. MemoizationManager:")
    manager = get_memo_manager()
    print(f"Manager: {manager}")
    print(f"Enabled: {manager.is_enabled()}")
    print(f"Global stats: {manager.stats()}")
    
    print("\n✅ Memoization System Demo Complete!")
