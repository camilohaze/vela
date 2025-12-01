"""
Vela Lexer Module

Implementación de: VELA-567 (Sprint 5)
Fecha: 2025-11-30

Este módulo exporta los componentes principales del lexer de Vela.
"""

from .token import Token, TokenKind, Position, KEYWORDS
from .lexer import Lexer

__all__ = [
    'Lexer',
    'Token',
    'TokenKind',
    'Position',
    'KEYWORDS',
]
