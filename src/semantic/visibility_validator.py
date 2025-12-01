"""
TASK-023: Validar visibilidad (public/private)

Implementación de: VELA-572
Sprint: 10
Fecha: 2025-12-01

Descripción:
------------
Enforcement de access control (public/private) en el lenguaje Vela.

Este validador verifica que los símbolos privados solo sean accesibles
desde su módulo de origen, mientras que los símbolos públicos pueden
ser accedidos desde cualquier módulo.

REGLAS DE VISIBILIDAD:
----------------------
1. Símbolos SIN modificador `public` → PRIVADOS (solo accesibles en mismo módulo)
2. Símbolos CON modificador `public` → PÚBLICOS (accesibles desde cualquier módulo)
3. Validación se realiza DURANTE name resolution
4. Error messages claros con información de ubicación

CASOS ESPECIALES:
-----------------
- Módulos pueden declarar exports específicos
- Símbolos de tipo CLASS/INTERFACE pueden tener miembros privados
- Símbolos de stdlib (system:) son siempre públicos
"""

from enum import Enum
from dataclasses import dataclass
from typing import Optional, List, Set

try:
    from .symbol_table import Symbol, SymbolKind
except ImportError:
    from symbol_table import Symbol, SymbolKind


class AccessLevel(Enum):
    """Niveles de acceso para símbolos."""
    PUBLIC = "public"      # Accesible desde cualquier módulo
    PRIVATE = "private"    # Solo accesible desde módulo de origen
    PROTECTED = "protected"  # Accesible desde clase y subclases


class ModuleType(Enum):
    """Tipos de módulos en Vela."""
    USER_MODULE = "user"      # Módulo del usuario (archivo .vela)
    SYSTEM = "system"          # Módulo de stdlib (system:)
    PACKAGE = "package"        # Paquete externo (package:)
    LIBRARY = "library"        # Librería interna (library:)
    EXTENSION = "extension"    # Extensión (extension:)


@dataclass
class ModuleContext:
    """
    Contexto de un módulo.
    
    Representa información sobre un módulo para
    validación de visibilidad.
    """
    name: str                    # Nombre del módulo
    type: ModuleType             # Tipo de módulo
    exports: Set[str]            # Símbolos exportados explícitamente
    
    def __hash__(self):
        return hash(self.name)
    
    def is_stdlib(self) -> bool:
        """Verifica si es módulo de stdlib."""
        return self.type == ModuleType.SYSTEM
    
    def is_external(self) -> bool:
        """Verifica si es módulo externo."""
        return self.type in (ModuleType.PACKAGE, ModuleType.EXTENSION)


@dataclass
class AccessViolation:
    """
    Violación de visibilidad.
    
    Representa un error de acceso a símbolo privado
    desde un módulo no autorizado.
    """
    symbol: Symbol               # Símbolo accedido
    symbol_module: str           # Módulo donde se definió
    access_module: str           # Módulo que intenta acceder
    line: int                    # Línea del acceso
    column: int                  # Columna del acceso
    message: str                 # Mensaje de error
    
    def __str__(self) -> str:
        return (
            f"AccessViolation at line {self.line}, column {self.column}: "
            f"{self.message}"
        )


class VisibilityError(Exception):
    """Excepción lanzada en violación de visibilidad."""
    
    def __init__(self, violation: AccessViolation):
        self.violation = violation
        super().__init__(str(violation))


class VisibilityValidator:
    """
    Validador de visibilidad (public/private).
    
    Enforcement de access control durante semantic analysis.
    Verifica que símbolos privados solo sean accesibles desde
    su módulo de origen.
    
    Example:
        # module_a.vela
        private fn helper() { }      # Solo accesible en module_a
        public fn process() { }      # Accesible desde cualquier módulo
        
        # module_b.vela
        import 'module:module_a'
        
        process()  # ✅ OK: process es public
        helper()   # ❌ ERROR: helper es private
    """
    
    def __init__(self):
        """Inicializar el validador."""
        self.modules: dict[str, ModuleContext] = {}
        self.current_module: Optional[ModuleContext] = None
        self.violations: List[AccessViolation] = []
    
    def register_module(
        self,
        name: str,
        type: ModuleType = ModuleType.USER_MODULE,
        exports: Optional[Set[str]] = None
    ) -> ModuleContext:
        """
        Registra un módulo en el validador.
        
        Args:
            name: Nombre del módulo
            type: Tipo de módulo
            exports: Símbolos exportados explícitamente (opcional)
        
        Returns:
            ModuleContext registrado
        """
        if exports is None:
            exports = set()
        
        module = ModuleContext(name, type, exports)
        self.modules[name] = module
        return module
    
    def set_current_module(self, module_name: str) -> None:
        """
        Establece el módulo actual.
        
        Args:
            module_name: Nombre del módulo actual
        
        Raises:
            ValueError: Si el módulo no está registrado
        """
        if module_name not in self.modules:
            raise ValueError(f"Module '{module_name}' not registered")
        
        self.current_module = self.modules[module_name]
    
    def get_access_level(self, symbol: Symbol) -> AccessLevel:
        """
        Obtiene el nivel de acceso de un símbolo.
        
        Args:
            symbol: Símbolo a verificar
        
        Returns:
            AccessLevel del símbolo
        """
        # Verificar modificador public explícito
        if symbol.is_public:
            return AccessLevel.PUBLIC
        
        # Verificar metadata para protected
        if symbol.metadata and symbol.metadata.get("access_level") == "protected":
            return AccessLevel.PROTECTED
        
        # Por defecto es private
        return AccessLevel.PRIVATE
    
    def validate_access(
        self,
        symbol: Symbol,
        symbol_module: str,
        line: int,
        column: int
    ) -> bool:
        """
        Valida si el símbolo puede ser accedido desde el módulo actual.
        
        Args:
            symbol: Símbolo a acceder
            symbol_module: Módulo donde se definió el símbolo
            line: Línea del acceso
            column: Columna del acceso
        
        Returns:
            True si el acceso es válido
        
        Raises:
            VisibilityError: Si el acceso viola reglas de visibilidad
        """
        if self.current_module is None:
            raise ValueError("No current module set")
        
        # Obtener nivel de acceso
        access_level = self.get_access_level(symbol)
        
        # Caso 1: Símbolos públicos siempre accesibles
        if access_level == AccessLevel.PUBLIC:
            return True
        
        # Caso 2: Acceso desde mismo módulo siempre permitido
        if self.current_module.name == symbol_module:
            return True
        
        # Caso 3: Módulos de stdlib siempre públicos
        symbol_module_ctx = self.modules.get(symbol_module)
        if symbol_module_ctx and symbol_module_ctx.is_stdlib():
            return True
        
        # Caso 4: Protected solo para subclases (no implementado aún)
        if access_level == AccessLevel.PROTECTED:
            # TODO: Implementar validación de herencia
            pass
        
        # Caso 5: Verificar si está en exports explícitos
        if symbol_module_ctx and symbol.name in symbol_module_ctx.exports:
            return True
        
        # VIOLACIÓN: Símbolo privado accedido desde otro módulo
        violation = AccessViolation(
            symbol=symbol,
            symbol_module=symbol_module,
            access_module=self.current_module.name,
            line=line,
            column=column,
            message=(
                f"Cannot access private symbol '{symbol.name}' "
                f"(defined in module '{symbol_module}') "
                f"from module '{self.current_module.name}'. "
                f"Symbol must be marked as 'public' to be accessible."
            )
        )
        
        self.violations.append(violation)
        raise VisibilityError(violation)
    
    def validate_member_access(
        self,
        class_symbol: Symbol,
        member_symbol: Symbol,
        line: int,
        column: int
    ) -> bool:
        """
        Valida acceso a miembro de clase (field/method).
        
        Args:
            class_symbol: Símbolo de la clase
            member_symbol: Símbolo del miembro
            line: Línea del acceso
            column: Columna del acceso
        
        Returns:
            True si el acceso es válido
        
        Raises:
            VisibilityError: Si el acceso viola reglas de visibilidad
        """
        if self.current_module is None:
            raise ValueError("No current module set")
        
        # Obtener nivel de acceso del miembro
        access_level = self.get_access_level(member_symbol)
        
        # Miembros públicos siempre accesibles
        if access_level == AccessLevel.PUBLIC:
            return True
        
        # Verificar si accedemos desde la misma clase
        # (implementación simplificada)
        class_module = class_symbol.metadata.get("module") if class_symbol.metadata else None
        if class_module == self.current_module.name:
            return True
        
        # VIOLACIÓN: Miembro privado accedido desde fuera de la clase
        violation = AccessViolation(
            symbol=member_symbol,
            symbol_module=class_module or "unknown",
            access_module=self.current_module.name,
            line=line,
            column=column,
            message=(
                f"Cannot access private member '{member_symbol.name}' "
                f"of class '{class_symbol.name}'. "
                f"Member must be marked as 'public' to be accessible outside the class."
            )
        )
        
        self.violations.append(violation)
        raise VisibilityError(violation)
    
    def get_violations(self) -> List[AccessViolation]:
        """
        Obtiene todas las violaciones registradas.
        
        Returns:
            Lista de violaciones
        """
        return self.violations.copy()
    
    def clear_violations(self) -> None:
        """Limpia todas las violaciones registradas."""
        self.violations.clear()
    
    def reset(self) -> None:
        """Resetea el validador (útil para tests)."""
        self.modules.clear()
        self.current_module = None
        self.violations.clear()
    
    def get_module_info(self, module_name: str) -> Optional[ModuleContext]:
        """
        Obtiene información de un módulo.
        
        Args:
            module_name: Nombre del módulo
        
        Returns:
            ModuleContext o None si no existe
        """
        return self.modules.get(module_name)
    
    def list_public_symbols(self, module_name: str) -> List[str]:
        """
        Lista todos los símbolos públicos de un módulo.
        
        Args:
            module_name: Nombre del módulo
        
        Returns:
            Lista de nombres de símbolos públicos
        """
        module = self.modules.get(module_name)
        if not module:
            return []
        
        # Si hay exports explícitos, retornar esos
        if module.exports:
            return list(module.exports)
        
        # Si es stdlib, todo es público
        if module.is_stdlib():
            return []  # Retornar vacío porque todo es público
        
        return []


# ============================================================================
# DEMO
# ============================================================================

if __name__ == "__main__":
    print("=== VISIBILITY VALIDATOR DEMO ===\n")
    
    # Crear validador
    validator = VisibilityValidator()
    
    # Crear símbolos de prueba
    public_symbol = Symbol(
        name="process",
        kind=SymbolKind.FUNCTION,
        scope_level=0,
        is_public=True
    )
    
    private_symbol = Symbol(
        name="helper",
        kind=SymbolKind.FUNCTION,
        scope_level=0,
        is_public=False
    )
    
    class_symbol = Symbol(
        name="User",
        kind=SymbolKind.CLASS,
        scope_level=0,
        is_public=True,
        metadata={"module": "module_a"}
    )
    
    private_member = Symbol(
        name="password",
        kind=SymbolKind.VARIABLE,
        scope_level=1,
        is_public=False,
        metadata={"module": "module_a"}
    )
    
    public_member = Symbol(
        name="name",
        kind=SymbolKind.VARIABLE,
        scope_level=1,
        is_public=True
    )
    
    print("1. Registrando módulos:")
    module_a = validator.register_module("module_a", ModuleType.USER_MODULE)
    module_b = validator.register_module("module_b", ModuleType.USER_MODULE)
    stdlib = validator.register_module("system:core", ModuleType.SYSTEM)
    print(f"   ✅ Registrado: {module_a.name} ({module_a.type.value})")
    print(f"   ✅ Registrado: {module_b.name} ({module_b.type.value})")
    print(f"   ✅ Registrado: {stdlib.name} ({stdlib.type.value})")
    
    print("\n2. Accediendo a símbolo PÚBLICO desde mismo módulo:")
    validator.set_current_module("module_a")
    try:
        validator.validate_access(public_symbol, "module_a", 10, 5)
        print("   ✅ Acceso permitido (public symbol, same module)")
    except VisibilityError as e:
        print(f"   ❌ Error inesperado: {e}")
    
    print("\n3. Accediendo a símbolo PÚBLICO desde otro módulo:")
    validator.set_current_module("module_b")
    try:
        validator.validate_access(public_symbol, "module_a", 15, 10)
        print("   ✅ Acceso permitido (public symbol, cross-module)")
    except VisibilityError as e:
        print(f"   ❌ Error inesperado: {e}")
    
    print("\n4. Accediendo a símbolo PRIVADO desde mismo módulo:")
    validator.set_current_module("module_a")
    try:
        validator.validate_access(private_symbol, "module_a", 20, 5)
        print("   ✅ Acceso permitido (private symbol, same module)")
    except VisibilityError as e:
        print(f"   ❌ Error inesperado: {e}")
    
    print("\n5. Intentando acceder a símbolo PRIVADO desde otro módulo:")
    validator.set_current_module("module_b")
    try:
        validator.validate_access(private_symbol, "module_a", 25, 10)
        print("   ❌ ERROR: Acceso debió fallar!")
    except VisibilityError as e:
        print(f"   ✅ Error capturado correctamente:")
        print(f"      {e.violation.message}")
    
    print("\n6. Accediendo a símbolo de STDLIB (siempre público):")
    stdlib_symbol = Symbol(
        name="print",
        kind=SymbolKind.FUNCTION,
        scope_level=0,
        is_public=False  # Aunque esté marcado privado, stdlib es público
    )
    try:
        validator.validate_access(stdlib_symbol, "system:core", 30, 0)
        print("   ✅ Acceso permitido (stdlib is always public)")
    except VisibilityError as e:
        print(f"   ❌ Error inesperado: {e}")
    
    print("\n7. Accediendo a miembro PÚBLICO de clase:")
    validator.set_current_module("module_b")
    try:
        validator.validate_member_access(class_symbol, public_member, 35, 10)
        print("   ✅ Acceso permitido (public member)")
    except VisibilityError as e:
        print(f"   ❌ Error inesperado: {e}")
    
    print("\n8. Intentando acceder a miembro PRIVADO de clase:")
    try:
        validator.validate_member_access(class_symbol, private_member, 40, 10)
        print("   ❌ ERROR: Acceso debió fallar!")
    except VisibilityError as e:
        print(f"   ✅ Error capturado correctamente:")
        print(f"      {e.violation.message}")
    
    print("\n9. Verificando niveles de acceso:")
    print(f"   public_symbol: {validator.get_access_level(public_symbol).value}")
    print(f"   private_symbol: {validator.get_access_level(private_symbol).value}")
    print(f"   public_member: {validator.get_access_level(public_member).value}")
    print(f"   private_member: {validator.get_access_level(private_member).value}")
    
    print("\n10. Resumen de violaciones:")
    violations = validator.get_violations()
    print(f"    Total violaciones capturadas: {len(violations)}")
    for i, v in enumerate(violations, 1):
        print(f"    {i}. '{v.symbol.name}' en línea {v.line}")
        print(f"       De: {v.symbol_module} → A: {v.access_module}")
    
    print("\n✅ Demo completada!")
