"""
Name Resolver - Resolución de Identificadores

Implementación de: TASK-022
Historia: VELA-572
Fecha: 2025-12-01

Descripción:
Resuelve identificadores (variables, funciones, clases, módulos) en todos
los scopes del programa. Integra Symbol Table para lookup de símbolos
y maneja shadowing, scopes anidados, y referencias forward.
"""

from enum import Enum, auto
from typing import Optional, List, Dict, Set
from dataclasses import dataclass

try:
    from src.semantic.symbol_table import SymbolTable, Symbol, SymbolKind, Scope, ScopeType
except ImportError:
    from symbol_table import SymbolTable, Symbol, SymbolKind, Scope, ScopeType


class ResolutionError(Exception):
    """Error durante resolución de nombres."""
    pass


class ReferenceKind(Enum):
    """Tipo de referencia a un identificador."""
    READ = auto()           # Lectura de variable
    WRITE = auto()          # Escritura a variable
    CALL = auto()           # Llamada a función
    INSTANTIATION = auto()  # Instanciación de clase
    IMPORT = auto()         # Import de módulo
    TYPE_ANNOTATION = auto()  # Uso en type annotation
    DECORATOR = auto()      # Uso en decorador


@dataclass
class Reference:
    """
    Referencia a un identificador en el código.
    
    Representa un uso de un nombre (variable, función, clase, etc.)
    en una ubicación específica del código.
    """
    name: str
    kind: ReferenceKind
    line: int
    column: int
    scope_level: int
    resolved_symbol: Optional[Symbol] = None
    
    def is_resolved(self) -> bool:
        """Verifica si la referencia está resuelta."""
        return self.resolved_symbol is not None


@dataclass
class UnresolvedReference:
    """Referencia no resuelta (error)."""
    reference: Reference
    message: str
    suggestion: Optional[str] = None


class NameResolver:
    """
    Resolvedor de nombres con integración a Symbol Table.
    
    Funcionalidades:
    - Resolución de identificadores en todos los scopes
    - Manejo de shadowing
    - Detección de referencias forward (uso antes de definición)
    - Validación de mutabilidad (asignación a inmutables)
    - Tracking de referencias para análisis de uso
    """
    
    def __init__(self, symbol_table: SymbolTable):
        """
        Inicializar resolver con symbol table.
        
        Args:
            symbol_table: Tabla de símbolos a usar para lookups
        """
        self.symbol_table = symbol_table
        self.references: List[Reference] = []
        self.unresolved: List[UnresolvedReference] = []
        
        # Tracking de símbolos definidos por scope (para forward refs)
        self.defined_symbols: Dict[int, Set[str]] = {}
        
        # Tracking de usos de símbolos (para dead code detection)
        self.symbol_uses: Dict[str, List[Reference]] = {}
    
    def resolve(
        self,
        name: str,
        kind: ReferenceKind,
        line: int = 0,
        column: int = 0
    ) -> Optional[Symbol]:
        """
        Resuelve un identificador en el scope actual.
        
        Args:
            name: Nombre del identificador
            kind: Tipo de referencia (READ, WRITE, CALL, etc.)
            line: Línea del código
            column: Columna del código
        
        Returns:
            Symbol si se encontró, None si no existe
        
        Raises:
            ResolutionError: Si hay error en la resolución
        """
        # Crear referencia
        ref = Reference(
            name=name,
            kind=kind,
            line=line,
            column=column,
            scope_level=self.symbol_table.get_scope_level()
        )
        
        # Lookup en symbol table
        symbol = self.symbol_table.lookup(name)
        
        if symbol:
            # Resolución exitosa
            ref.resolved_symbol = symbol
            self.references.append(ref)
            
            # Tracking de usos
            if name not in self.symbol_uses:
                self.symbol_uses[name] = []
            self.symbol_uses[name].append(ref)
            
            # Validaciones adicionales según el tipo de referencia
            self._validate_reference(ref, symbol)
            
            return symbol
        else:
            # No se encontró el símbolo
            self.references.append(ref)
            
            # Generar error descriptivo
            self._report_unresolved(ref)
            
            return None
    
    def _validate_reference(self, ref: Reference, symbol: Symbol) -> None:
        """
        Valida que la referencia sea válida para el símbolo.
        
        Args:
            ref: Referencia a validar
            symbol: Símbolo resuelto
        
        Raises:
            ResolutionError: Si la referencia es inválida
        """
        # Validar escritura a inmutables
        if ref.kind == ReferenceKind.WRITE:
            if not symbol.is_mutable:
                raise ResolutionError(
                    f"Cannot assign to immutable variable '{ref.name}' at line {ref.line}. "
                    f"Variable declared at scope level {symbol.scope_level}. "
                    f"Suggestion: Use 'state {ref.name}' if you need mutability."
                )
        
        # Validar llamada a función
        if ref.kind == ReferenceKind.CALL:
            if symbol.kind != SymbolKind.FUNCTION:
                raise ResolutionError(
                    f"'{ref.name}' at line {ref.line} is not callable. "
                    f"It is a {symbol.kind.name}."
                )
        
        # Validar instanciación de clase
        if ref.kind == ReferenceKind.INSTANTIATION:
            if symbol.kind not in [SymbolKind.CLASS, SymbolKind.STRUCT]:
                raise ResolutionError(
                    f"'{ref.name}' at line {ref.line} is not instantiable. "
                    f"It is a {symbol.kind.name}."
                )
            
            # Validar que module NO sea instanciable
            if symbol.kind == SymbolKind.MODULE:
                raise ResolutionError(
                    f"Cannot instantiate module '{ref.name}' at line {ref.line}. "
                    f"Modules are not instantiable (use imports instead)."
                )
    
    def _report_unresolved(self, ref: Reference) -> None:
        """
        Reporta una referencia no resuelta.
        
        Args:
            ref: Referencia no resuelta
        """
        message = f"Name '{ref.name}' is not defined at line {ref.line}, column {ref.column}."
        
        # Sugerencias basadas en similitud (nombres parecidos en scope)
        suggestion = self._suggest_similar_names(ref.name)
        
        unresolved = UnresolvedReference(
            reference=ref,
            message=message,
            suggestion=suggestion
        )
        
        self.unresolved.append(unresolved)
    
    def _suggest_similar_names(self, name: str) -> Optional[str]:
        """
        Sugiere nombres similares disponibles en el scope.
        
        Args:
            name: Nombre no encontrado
        
        Returns:
            Sugerencia de nombre similar o None
        """
        # Obtener todos los símbolos en scope actual
        current_scope = self.symbol_table.current_scope
        if not current_scope:
            return None
        
        available_names = list(current_scope.symbols.keys())
        
        # Buscar nombres con distancia de Levenshtein pequeña
        similar = []
        for available in available_names:
            distance = self._levenshtein_distance(name, available)
            if distance <= 2:  # Máximo 2 caracteres de diferencia
                similar.append(available)
        
        if similar:
            return f"Did you mean: {', '.join(similar)}?"
        
        return None
    
    @staticmethod
    def _levenshtein_distance(s1: str, s2: str) -> int:
        """
        Calcula distancia de Levenshtein entre dos strings.
        
        Args:
            s1: Primera string
            s2: Segunda string
        
        Returns:
            Distancia (número de ediciones)
        """
        if len(s1) < len(s2):
            return NameResolver._levenshtein_distance(s2, s1)
        
        if len(s2) == 0:
            return len(s1)
        
        previous_row = range(len(s2) + 1)
        for i, c1 in enumerate(s1):
            current_row = [i + 1]
            for j, c2 in enumerate(s2):
                insertions = previous_row[j + 1] + 1
                deletions = current_row[j] + 1
                substitutions = previous_row[j] + (c1 != c2)
                current_row.append(min(insertions, deletions, substitutions))
            previous_row = current_row
        
        return previous_row[-1]
    
    def define(
        self,
        name: str,
        kind: SymbolKind,
        line: int = 0,
        column: int = 0,
        type_annotation: Optional[str] = None,
        is_public: bool = False,
        is_mutable: bool = False,
        metadata: Optional[Dict] = None
    ) -> Symbol:
        """
        Define un nuevo símbolo en el scope actual.
        
        Args:
            name: Nombre del símbolo
            kind: Tipo de símbolo
            line: Línea de definición
            column: Columna de definición
            type_annotation: Tipo anotado
            is_public: Si es público
            is_mutable: Si es mutable (state)
            metadata: Metadata adicional
        
        Returns:
            Symbol creado
        
        Raises:
            ResolutionError: Si ya existe en el scope actual
        """
        # Crear símbolo
        symbol = Symbol(
            name=name,
            kind=kind,
            scope_level=self.symbol_table.get_scope_level(),
            type_annotation=type_annotation,
            is_public=is_public,
            is_mutable=is_mutable,
            metadata=metadata or {}
        )
        
        # Definir en symbol table
        self.symbol_table.define(symbol)
        
        # Tracking de definición
        scope_level = self.symbol_table.get_scope_level()
        if scope_level not in self.defined_symbols:
            self.defined_symbols[scope_level] = set()
        self.defined_symbols[scope_level].add(name)
        
        return symbol
    
    def enter_scope(self, scope_type: ScopeType, name: Optional[str] = None) -> None:
        """
        Entra a un nuevo scope.
        
        Args:
            scope_type: Tipo de scope
            name: Nombre del scope (opcional, no usado actualmente)
        """
        self.symbol_table.push_scope(scope_type)
    
    def exit_scope(self) -> None:
        """Sale del scope actual."""
        # Limpiar tracking de símbolos definidos en este scope
        scope_level = self.symbol_table.get_scope_level()
        if scope_level in self.defined_symbols:
            del self.defined_symbols[scope_level]
        
        self.symbol_table.pop_scope()
    
    def get_references(self, name: str) -> List[Reference]:
        """
        Obtiene todas las referencias a un símbolo.
        
        Args:
            name: Nombre del símbolo
        
        Returns:
            Lista de referencias
        """
        return self.symbol_uses.get(name, [])
    
    def get_unresolved_references(self) -> List[UnresolvedReference]:
        """Obtiene todas las referencias no resueltas."""
        return self.unresolved.copy()
    
    def is_symbol_used(self, name: str) -> bool:
        """
        Verifica si un símbolo está siendo usado.
        
        Args:
            name: Nombre del símbolo
        
        Returns:
            True si tiene al menos una referencia
        """
        return name in self.symbol_uses and len(self.symbol_uses[name]) > 0
    
    def get_unused_symbols(self) -> List[Symbol]:
        """
        Obtiene símbolos definidos pero no usados (dead code).
        
        Returns:
            Lista de símbolos sin referencias
        """
        unused = []
        
        # Recorrer todos los símbolos definidos
        for scope_level, names in self.defined_symbols.items():
            for name in names:
                if not self.is_symbol_used(name):
                    symbol = self.symbol_table.lookup(name)
                    if symbol:
                        # Ignorar símbolos públicos (pueden ser usados externamente)
                        if not symbol.is_public:
                            unused.append(symbol)
        
        return unused
    
    def reset(self) -> None:
        """Resetea el estado del resolver."""
        self.references.clear()
        self.unresolved.clear()
        self.defined_symbols.clear()
        self.symbol_uses.clear()


if __name__ == "__main__":
    # Demostración de uso
    print("=== NAME RESOLUTION DEMO ===\n")
    
    # Crear symbol table y resolver
    symbol_table = SymbolTable()
    resolver = NameResolver(symbol_table)
    
    print("1. Definiendo símbolos en scope global:")
    print("-" * 40)
    
    # Definir variable global
    pi_symbol = resolver.define("PI", SymbolKind.VARIABLE, line=1, type_annotation="Float", is_public=True)
    print(f"✅ Definido: {pi_symbol.name} ({pi_symbol.kind.name})")
    
    # Definir función
    add_symbol = resolver.define("add", SymbolKind.FUNCTION, line=3, is_public=True)
    print(f"✅ Definido: {add_symbol.name} ({add_symbol.kind.name})")
    
    print("\n2. Resolviendo referencias:")
    print("-" * 40)
    
    # Leer PI (válido)
    resolved = resolver.resolve("PI", ReferenceKind.READ, line=10)
    if resolved:
        print(f"✅ Resuelto 'PI': {resolved.kind.name} en scope level {resolved.scope_level}")
    
    # Llamar add (válido)
    resolved = resolver.resolve("add", ReferenceKind.CALL, line=11)
    if resolved:
        print(f"✅ Resuelto 'add': {resolved.kind.name}")
    
    # Referencia no existente
    print("\n3. Intentando resolver nombre inexistente:")
    print("-" * 40)
    resolved = resolver.resolve("undefined_var", ReferenceKind.READ, line=12)
    if not resolved:
        print("❌ 'undefined_var' no está definido")
        for unresolved in resolver.get_unresolved_references():
            print(f"   Error: {unresolved.message}")
            if unresolved.suggestion:
                print(f"   {unresolved.suggestion}")
    
    print("\n4. Definir variable mutable y mutar:")
    print("-" * 40)
    
    # Definir state variable
    count_symbol = resolver.define("count", SymbolKind.STATE, line=15, is_mutable=True)
    print(f"✅ Definido: state {count_symbol.name}")
    
    # Escribir a state (válido)
    try:
        resolved = resolver.resolve("count", ReferenceKind.WRITE, line=16)
        print(f"✅ Escritura a 'count' permitida (es mutable)")
    except ResolutionError as e:
        print(f"❌ {e}")
    
    print("\n5. Intentar mutar variable inmutable:")
    print("-" * 40)
    
    # Intentar escribir a PI (inválido)
    try:
        resolved = resolver.resolve("PI", ReferenceKind.WRITE, line=20)
        print("❌ ERROR: Debería haber lanzado excepción")
    except ResolutionError as e:
        print(f"✅ Error capturado: {str(e)[:80]}...")
    
    print("\n6. Scopes anidados:")
    print("-" * 40)
    
    # Entrar a scope de función
    resolver.enter_scope(ScopeType.FUNCTION, "myFunction")
    print("Entrando a scope: myFunction")
    
    # Definir parámetro
    x_symbol = resolver.define("x", SymbolKind.PARAMETER, line=25)
    print(f"✅ Definido parámetro: {x_symbol.name}")
    
    # Resolver x dentro de función (válido)
    resolved = resolver.resolve("x", ReferenceKind.READ, line=26)
    if resolved:
        print(f"✅ Resuelto 'x' en scope level {resolved.scope_level}")
    
    # Resolver PI desde función (válido - lookup en parent scope)
    resolved = resolver.resolve("PI", ReferenceKind.READ, line=27)
    if resolved:
        print(f"✅ Resuelto 'PI' desde scope padre (level {resolved.scope_level})")
    
    # Salir del scope
    resolver.exit_scope()
    print("Saliendo de scope: myFunction")
    
    # Intentar resolver x fuera de función (inválido)
    resolved = resolver.resolve("x", ReferenceKind.READ, line=30)
    if not resolved:
        print("❌ 'x' no accesible fuera de su scope")
    
    print("\n7. Tracking de usos:")
    print("-" * 40)
    
    # Ver referencias a PI
    pi_refs = resolver.get_references("PI")
    print(f"'PI' tiene {len(pi_refs)} referencias:")
    for ref in pi_refs:
        print(f"  - Línea {ref.line}: {ref.kind.name}")
    
    print("\n8. Dead code detection:")
    print("-" * 40)
    
    # Definir símbolo sin usar
    unused_symbol = resolver.define("unusedVar", SymbolKind.VARIABLE, line=35, is_public=False)
    print(f"Definido: {unused_symbol.name} (sin usar)")
    
    unused = resolver.get_unused_symbols()
    print(f"Símbolos no usados: {len(unused)}")
    for sym in unused:
        print(f"  - {sym.name} (definido en scope level {sym.scope_level})")
    
    print("\n9. Resumen:")
    print("=" * 40)
    print(f"Total referencias: {len(resolver.references)}")
    print(f"Referencias resueltas: {sum(1 for r in resolver.references if r.is_resolved())}")
    print(f"Referencias no resueltas: {len(resolver.unresolved)}")
    print(f"Símbolos sin usar: {len(unused)}")
    
    print("\n✅ Demo completada!")
