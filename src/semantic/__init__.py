"""
Semantic Analysis Module

Este módulo contiene las herramientas de análisis semántico para Vela:
- Symbol Table: Tabla de símbolos con scopes anidados (TASK-021)
- Import Resolver: Resolución de imports con prefijos (TASK-021A)
- Import Validator: Validación de reglas de imports por keyword (TASK-021B)
- Module Validator: Validación de reglas de @module (TASK-021C)
- Decorator Validator: Validación de metadata de decoradores (TASK-021D)

Implementación de: VELA-572 (Sprint 10)
Historia: VELA-572
Fecha: 2025-12-01
"""

from .symbol_table import Symbol, SymbolTable, SymbolKind, Scope, ScopeType
from .import_resolver import ImportResolver, ImportPath, ImportPrefix as ResolverPrefix, ResolvedImport
from .import_validator import ImportValidator, VelaKeyword, ImportPrefix, ImportRule, ImportViolation

__all__ = [
    # Symbol Table
    'Symbol',
    'SymbolTable',
    'SymbolKind',
    'Scope',
    'ScopeType',
    
    # Import Resolver
    'ImportResolver',
    'ImportPath',
    'ResolverPrefix',
    'ResolvedImport',
    
    # Import Validator
    'ImportValidator',
    'VelaKeyword',
    'ImportPrefix',
    'ImportRule',
    'ImportViolation',
]
