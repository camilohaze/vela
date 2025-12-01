"""
Parser Package para el lenguaje Vela

Este paquete implementa el parser completo de Vela que genera un AST (Abstract Syntax Tree).

Componentes principales:
- ast_nodes.py: Definición de nodos del AST ✅
- parser.py: Parser recursive descent principal ✅
- pratt_parser.py: Pratt parser para expresiones con precedencia (TODO)
- error_recovery.py: Estrategias de recuperación de errores (TODO)

Fecha: 2025-12-01
Sprint: 6 (VELA-568)
Versión: 0.2.0
"""

# Export Parser classes
from .parser import (
    Parser,
    ParserError,
    UnexpectedTokenError,
    UnexpectedEOFError,
    parse_code
)

# Export Pratt Parser classes  
from .pratt_parser import (
    PrattParser,
    PrattParserError,
    Precedence,
    parse_expression_from_tokens
)

# Export AST node types
from .ast_nodes import (
    # Base
    ASTNode, Position, Range, Program,
    
    # Imports
    ImportDeclaration, ImportKind,
    
    # Declarations
    Declaration, FunctionDeclaration, Parameter, GenericParameter,
    StructDeclaration, StructField, EnumDeclaration, EnumVariant,
    TypeAliasDeclaration, InterfaceDeclaration, FunctionSignature,
    ClassDeclaration, ClassField, ConstructorDeclaration, MethodDeclaration,
    
    # Domain-Specific Declarations
    ServiceDeclaration, RepositoryDeclaration, ControllerDeclaration,
    RouteDeclaration, UseCaseDeclaration, EntityDeclaration,
    ValueObjectDeclaration, DTODeclaration,
    
    # Statements
    Statement, BlockStatement, ExpressionStatement,
    VariableDeclaration, AssignmentStatement, ReturnStatement,
    IfStatement, MatchStatement, MatchArm,
    ThrowStatement, TryStatement, CatchClause,
    
    # Expressions
    Expression, Literal, Identifier,
    BinaryExpression, UnaryExpression, CallExpression,
    MemberAccessExpression, IndexAccessExpression,
    ArrayLiteral, TupleLiteral, StructLiteral, StructLiteralField,
    LambdaExpression, IfExpression, MatchExpression, MatchExpressionArm,
    StringInterpolation, AwaitExpression, ComputedExpression,
    
    # Patterns
    Pattern, LiteralPattern, IdentifierPattern, TuplePattern,
    StructPattern, StructPatternField, EnumPattern,
    OrPattern, RangePattern, WildcardPattern,
    
    # Type Annotations
    TypeAnnotation, PrimitiveType, ArrayType, TupleType,
    FunctionType, GenericType, UnionType, NamedType, OptionalType,
    
    # Visitor
    ASTVisitor,
    
    # Utilities
    create_position, create_range, is_expression_statement_valid
)

__all__ = [
    # Base
    'ASTNode', 'Position', 'Range', 'Program',
    
    # Imports
    'ImportDeclaration', 'ImportKind',
    
    # Declarations
    'Declaration', 'FunctionDeclaration', 'Parameter', 'GenericParameter',
    'StructDeclaration', 'StructField', 'EnumDeclaration', 'EnumVariant',
    'TypeAliasDeclaration', 'InterfaceDeclaration', 'FunctionSignature',
    'ClassDeclaration', 'ClassField', 'ConstructorDeclaration', 'MethodDeclaration',
    
    # Domain-Specific Declarations
    'ServiceDeclaration', 'RepositoryDeclaration', 'ControllerDeclaration',
    'RouteDeclaration', 'UseCaseDeclaration', 'EntityDeclaration',
    'ValueObjectDeclaration', 'DTODeclaration',
    
    # Statements
    'Statement', 'BlockStatement', 'ExpressionStatement',
    'VariableDeclaration', 'AssignmentStatement', 'ReturnStatement',
    'IfStatement', 'MatchStatement', 'MatchArm',
    'ThrowStatement', 'TryStatement', 'CatchClause',
    
    # Expressions
    'Expression', 'Literal', 'Identifier',
    'BinaryExpression', 'UnaryExpression', 'CallExpression',
    'MemberAccessExpression', 'IndexAccessExpression',
    'ArrayLiteral', 'TupleLiteral', 'StructLiteral', 'StructLiteralField',
    'LambdaExpression', 'IfExpression', 'MatchExpression', 'MatchExpressionArm',
    'StringInterpolation', 'AwaitExpression', 'ComputedExpression',
    
    # Patterns
    'Pattern', 'LiteralPattern', 'IdentifierPattern', 'TuplePattern',
    'StructPattern', 'StructPatternField', 'EnumPattern',
    'OrPattern', 'RangePattern', 'WildcardPattern',
    
    # Type Annotations
    'TypeAnnotation', 'PrimitiveType', 'ArrayType', 'TupleType',
    'FunctionType', 'GenericType', 'UnionType', 'NamedType', 'OptionalType',
    
    # Visitor
    'ASTVisitor',
    
    # Utilities
    'create_position', 'create_range', 'is_expression_statement_valid'
]

__version__ = '0.1.0'
__author__ = 'Vela Language Team'
