"""
Semantic Analysis Module

Este módulo contiene las herramientas de análisis semántico para Vela:
- Symbol Table: Tabla de símbolos con scopes anidados (TASK-021)
- Import Resolver: Resolución de imports con prefijos (TASK-021A)
- Import Validator: Validación de reglas de imports por keyword (TASK-021B)
- Name Resolver: Resolución de identificadores (TASK-022)
- Visibility Validator: Enforcement de access control public/private (TASK-023)

Implementación de: VELA-572 (Sprint 10)
Historia: VELA-572
Fecha: 2025-12-01
"""

from .symbol_table import Symbol, SymbolTable, SymbolKind, Scope, ScopeType
from .import_resolver import ImportResolver, ImportPath, ImportPrefix as ResolverPrefix, ResolvedImport
from .import_validator import ImportValidator, VelaKeyword, ImportPrefix, ImportRule, ImportViolation
from .name_resolver import NameResolver, ReferenceKind, Reference, UnresolvedReference
from .visibility_validator import (
    VisibilityValidator,
    AccessLevel,
    ModuleType,
    ModuleContext,
    AccessViolation,
    VisibilityError
)

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
    
    # Name Resolver
    'NameResolver',
    'ReferenceKind',
    'Reference',
    'UnresolvedReference',
    
    # Visibility Validator
    'VisibilityValidator',
    'AccessLevel',
    'ModuleType',
    'ModuleContext',
    'AccessViolation',
    'VisibilityError',
]
