"""
Semantic Analysis Module

Este módulo contiene las herramientas de análisis semántico para Vela:
- Symbol Table: Tabla de símbolos con scopes anidados
- Import Resolver: Resolución de imports con prefijos
- Import Validator: Validación de reglas de imports por keyword
- Module Validator: Validación de reglas de @module
- Decorator Validator: Validación de metadata de decoradores

Implementación de: VELA-572 (Sprint 10)
Historia: VELA-572
Fecha: 2025-01-22
"""

from .symbol_table import Symbol, SymbolTable, SymbolKind, Scope

__all__ = [
    'Symbol',
    'SymbolTable',
    'SymbolKind',
    'Scope'
]
