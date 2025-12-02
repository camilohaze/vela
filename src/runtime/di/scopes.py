"""
Scopes de Lifecycle para Dependency Injection

Implementación de: TASK-035B
Historia: VELA-575 - Sistema de Dependency Injection
Sprint: 13
Fecha: 2025-12-01

Descripción:
Define los scopes de lifecycle soportados por el sistema DI:
- Singleton: Una instancia por aplicación
- Transient: Nueva instancia en cada inyección
- Scoped: Una instancia por scope (request HTTP)
"""

from enum import Enum, auto


class Scope(Enum):
    """
    Enum que define los scopes de lifecycle para providers inyectables.
    
    Scopes:
        SINGLETON: Una única instancia compartida en toda la aplicación.
                   Se crea una vez y se reutiliza siempre.
                   Cache: Global
                   Uso: Servicios stateless, DB connections, loggers
        
        TRANSIENT: Nueva instancia cada vez que se inyecta.
                   NO se cachea.
                   Uso: Objetos con estado temporal
        
        SCOPED: Una instancia por scope (generalmente request HTTP).
                Cache: Por scope
                Uso: User sessions, transactions, objetos con estado por request
    
    Ejemplos:
        >>> @injectable(scope=Scope.SINGLETON)
        >>> service DatabaseConnection:
        >>>     # Solo se crea una vez en toda la app
        >>>     pass
        
        >>> @injectable(scope=Scope.TRANSIENT)
        >>> service EmailMessage:
        >>>     # Nueva instancia cada vez
        >>>     pass
        
        >>> @injectable(scope=Scope.SCOPED)
        >>> service UserSession:
        >>>     # Una instancia por request HTTP
        >>>     pass
    """
    
    SINGLETON = auto()
    TRANSIENT = auto()
    SCOPED = auto()
    
    def __str__(self) -> str:
        """String representation para debugging."""
        return self.name
    
    def __repr__(self) -> str:
        """Representation para debugging."""
        return f"Scope.{self.name}"
    
    @classmethod
    def from_string(cls, value: str) -> 'Scope':
        """
        Convierte string a Scope enum.
        
        Args:
            value: String con el nombre del scope ("SINGLETON", "TRANSIENT", "SCOPED")
                  Case-insensitive. Whitespace es trimmed automáticamente.
        
        Returns:
            Scope enum correspondiente
        
        Raises:
            ValueError: Si el valor no es un scope válido
        
        Examples:
            >>> Scope.from_string("SINGLETON")
            Scope.SINGLETON
            >>> Scope.from_string("  singleton  ")
            Scope.SINGLETON
            >>> Scope.from_string("invalid")
            ValueError: Invalid scope: 'invalid'. Valid values: SINGLETON, TRANSIENT, SCOPED
        """
        # Trim whitespace y convertir a uppercase
        normalized = value.strip().upper()
        
        # Validar que no esté vacío después de trim
        if not normalized:
            raise ValueError(
                f"Invalid scope: '{value}'. Valid values: SINGLETON, TRANSIENT, SCOPED"
            )
        
        try:
            return cls[normalized]
        except KeyError:
            raise ValueError(
                f"Invalid scope: '{value}'. Valid values: SINGLETON, TRANSIENT, SCOPED"
            )
    
    def is_cacheable(self) -> bool:
        """
        Indica si este scope requiere caching de instancias.
        
        Returns:
            True si el scope cachea instancias (Singleton, Scoped)
            False si NO cachea (Transient)
        
        Examples:
            >>> Scope.SINGLETON.is_cacheable()
            True
            >>> Scope.TRANSIENT.is_cacheable()
            False
            >>> Scope.SCOPED.is_cacheable()
            True
        """
        return self in [Scope.SINGLETON, Scope.SCOPED]
    
    def cache_key_prefix(self) -> str:
        """
        Retorna el prefijo para keys de cache según el scope.
        
        Returns:
            String con prefijo para cache keys
        
        Examples:
            >>> Scope.SINGLETON.cache_key_prefix()
            "global"
            >>> Scope.SCOPED.cache_key_prefix()
            "scoped"
        """
        if self == Scope.SINGLETON:
            return "global"
        elif self == Scope.SCOPED:
            return "scoped"
        else:
            return "transient"  # No cachea, pero por consistencia


# Default scope si no se especifica
DEFAULT_SCOPE = Scope.SINGLETON


if __name__ == "__main__":
    # Tests básicos
    print("=== Tests de Scope ===")
    
    # Test 1: Crear scopes
    singleton = Scope.SINGLETON
    transient = Scope.TRANSIENT
    scoped = Scope.SCOPED
    
    print(f"Singleton: {singleton}")
    print(f"Transient: {transient}")
    print(f"Scoped: {scoped}")
    
    # Test 2: is_cacheable
    print(f"\nSingleton cacheable: {singleton.is_cacheable()}")  # True
    print(f"Transient cacheable: {transient.is_cacheable()}")    # False
    print(f"Scoped cacheable: {scoped.is_cacheable()}")          # True
    
    # Test 3: from_string
    parsed = Scope.from_string("SINGLETON")
    print(f"\nParsed 'SINGLETON': {parsed}")
    print(f"Parsed == SINGLETON: {parsed == Scope.SINGLETON}")
    
    # Test 4: cache_key_prefix
    print(f"\nSingleton prefix: {singleton.cache_key_prefix()}")
    print(f"Scoped prefix: {scoped.cache_key_prefix()}")
    
    print("\n✅ Todos los tests de Scope pasaron")
