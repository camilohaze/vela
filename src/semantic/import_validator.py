"""
Import Validator - Reglas por Keyword

Implementación de: TASK-021B
Historia: VELA-572
Fecha: 2025-12-01

Descripción:
Valida que cada keyword específico (service, repository, controller, widget, etc.)
SOLO importe lo permitido según sus reglas definidas en las especificaciones del lenguaje.

Ejemplo de regla:
- widget/component: PUEDE importar system:ui, module:, library: 
                    NO PUEDE importar package:http directamente
- service: PUEDE importar module:, library:, package:
           NO PUEDE importar system:ui (widgets)
"""

from enum import Enum, auto
from typing import List, Dict, Set, Optional
from dataclasses import dataclass


class VelaKeyword(Enum):
    """Keywords arquitectónicos de Vela con restricciones de imports."""
    # UI Components
    WIDGET = auto()
    STATEFUL_WIDGET = auto()
    STATELESS_WIDGET = auto()
    COMPONENT = auto()
    
    # DDD / Architecture
    SERVICE = auto()
    REPOSITORY = auto()
    CONTROLLER = auto()
    USECASE = auto()
    ENTITY = auto()
    DTO = auto()
    VALUE_OBJECT = auto()
    MODEL = auto()
    
    # Design Patterns
    FACTORY = auto()
    BUILDER = auto()
    STRATEGY = auto()
    OBSERVER = auto()
    SINGLETON = auto()
    ADAPTER = auto()
    DECORATOR = auto()
    
    # Web / API
    GUARD = auto()
    MIDDLEWARE = auto()
    INTERCEPTOR = auto()
    VALIDATOR = auto()
    PIPE = auto()
    
    # Utilities
    TASK = auto()
    HELPER = auto()
    MAPPER = auto()
    SERIALIZER = auto()
    PROVIDER = auto()
    STORE = auto()
    
    # Module System
    MODULE = auto()
    
    # Generic (sin restricciones)
    CLASS = auto()
    STRUCT = auto()
    ENUM = auto()
    INTERFACE = auto()


class ImportPrefix(Enum):
    """Prefijos de imports en Vela."""
    SYSTEM = "system"
    PACKAGE = "package"
    MODULE = "module"
    LIBRARY = "library"
    EXTENSION = "extension"
    ASSETS = "assets"


@dataclass
class ImportRule:
    """Regla de import para un keyword específico."""
    keyword: VelaKeyword
    allowed_prefixes: Set[ImportPrefix]
    forbidden_prefixes: Set[ImportPrefix]
    description: str
    
    def allows(self, prefix: ImportPrefix) -> bool:
        """Verifica si el prefijo está permitido."""
        # Si está explícitamente prohibido, rechazar
        if prefix in self.forbidden_prefixes:
            return False
        
        # Si allowed_prefixes está vacío, permitir todos excepto forbidden
        if not self.allowed_prefixes:
            return True
        
        # Verificar si está en la lista de permitidos
        return prefix in self.allowed_prefixes


@dataclass
class ImportViolation:
    """Violación de regla de import."""
    keyword: VelaKeyword
    import_statement: str
    prefix_used: ImportPrefix
    line: int
    column: int
    message: str
    suggestion: Optional[str] = None


class ImportValidator:
    """
    Validador de reglas de imports por keyword.
    
    Verifica que cada tipo arquitectónico solo importe lo permitido.
    """
    
    def __init__(self):
        """Inicializar validador con reglas predefinidas."""
        self.rules: Dict[VelaKeyword, ImportRule] = self._build_rules()
    
    def _build_rules(self) -> Dict[VelaKeyword, ImportRule]:
        """
        Construir reglas de imports por keyword.
        
        Basado en las especificaciones del lenguaje Vela.
        """
        rules = {}
        
        # ===== UI COMPONENTS =====
        # Widgets SOLO pueden importar: system:ui, module:, library:, assets:
        # NO PUEDEN importar: package:http, package:express (acceso directo a HTTP)
        ui_keywords = [
            VelaKeyword.WIDGET,
            VelaKeyword.STATEFUL_WIDGET,
            VelaKeyword.STATELESS_WIDGET,
            VelaKeyword.COMPONENT
        ]
        for keyword in ui_keywords:
            rules[keyword] = ImportRule(
                keyword=keyword,
                allowed_prefixes={
                    ImportPrefix.SYSTEM,
                    ImportPrefix.MODULE,
                    ImportPrefix.LIBRARY,
                    ImportPrefix.EXTENSION,
                    ImportPrefix.ASSETS
                },
                forbidden_prefixes={
                    ImportPrefix.PACKAGE  # No acceso directo a packages externos
                },
                description=(
                    f"{keyword.name} puede importar system:ui, module:, library:, extension:, assets:. "
                    "NO puede importar package: directamente (usar service/repository como intermediario)."
                )
            )
        
        # ===== SERVICES / REPOSITORIES =====
        # Pueden importar casi todo EXCEPTO system:ui (no deben saber de UI)
        backend_keywords = [
            VelaKeyword.SERVICE,
            VelaKeyword.REPOSITORY,
            VelaKeyword.USECASE
        ]
        for keyword in backend_keywords:
            rules[keyword] = ImportRule(
                keyword=keyword,
                allowed_prefixes={
                    ImportPrefix.PACKAGE,
                    ImportPrefix.MODULE,
                    ImportPrefix.LIBRARY,
                    ImportPrefix.EXTENSION
                },
                forbidden_prefixes={
                    ImportPrefix.SYSTEM  # No acceso a system:ui (separación de concerns)
                },
                description=(
                    f"{keyword.name} puede importar package:, module:, library:, extension:. "
                    "NO puede importar system:ui (lógica de negocio no debe depender de UI)."
                )
            )
        
        # ===== CONTROLLERS =====
        # Pueden importar todo (son el puente entre UI y backend)
        rules[VelaKeyword.CONTROLLER] = ImportRule(
            keyword=VelaKeyword.CONTROLLER,
            allowed_prefixes=set(ImportPrefix),  # Todos los prefijos
            forbidden_prefixes=set(),
            description="controller puede importar cualquier prefijo (puente entre capas)."
        )
        
        # ===== ENTITIES / DTOs / VALUE OBJECTS =====
        # NO deben importar nada excepto tipos puros (module:, library:)
        domain_keywords = [
            VelaKeyword.ENTITY,
            VelaKeyword.DTO,
            VelaKeyword.VALUE_OBJECT,
            VelaKeyword.MODEL
        ]
        for keyword in domain_keywords:
            rules[keyword] = ImportRule(
                keyword=keyword,
                allowed_prefixes={
                    ImportPrefix.MODULE,
                    ImportPrefix.LIBRARY
                },
                forbidden_prefixes={
                    ImportPrefix.PACKAGE,
                    ImportPrefix.SYSTEM,
                    ImportPrefix.EXTENSION
                },
                description=(
                    f"{keyword.name} SOLO puede importar module: y library: (debe ser puro, sin dependencias externas)."
                )
            )
        
        # ===== MIDDLEWARE / GUARDS / INTERCEPTORS =====
        # Pueden importar todo excepto system:ui
        web_keywords = [
            VelaKeyword.GUARD,
            VelaKeyword.MIDDLEWARE,
            VelaKeyword.INTERCEPTOR,
            VelaKeyword.PIPE
        ]
        for keyword in web_keywords:
            rules[keyword] = ImportRule(
                keyword=keyword,
                allowed_prefixes={
                    ImportPrefix.PACKAGE,
                    ImportPrefix.MODULE,
                    ImportPrefix.LIBRARY,
                    ImportPrefix.EXTENSION
                },
                forbidden_prefixes={
                    ImportPrefix.SYSTEM
                },
                description=(
                    f"{keyword.name} puede importar package:, module:, library:, extension:. "
                    "NO puede importar system:ui."
                )
            )
        
        # ===== VALIDATORS =====
        # Solo imports puros (module:, library:)
        rules[VelaKeyword.VALIDATOR] = ImportRule(
            keyword=VelaKeyword.VALIDATOR,
            allowed_prefixes={
                ImportPrefix.MODULE,
                ImportPrefix.LIBRARY
            },
            forbidden_prefixes={
                ImportPrefix.PACKAGE,
                ImportPrefix.SYSTEM,
                ImportPrefix.EXTENSION
            },
            description="validator SOLO puede importar module: y library: (validación pura)."
        )
        
        # ===== HELPERS / MAPPERS / SERIALIZERS =====
        # Solo imports puros
        utility_keywords = [
            VelaKeyword.HELPER,
            VelaKeyword.MAPPER,
            VelaKeyword.SERIALIZER
        ]
        for keyword in utility_keywords:
            rules[keyword] = ImportRule(
                keyword=keyword,
                allowed_prefixes={
                    ImportPrefix.MODULE,
                    ImportPrefix.LIBRARY,
                    ImportPrefix.PACKAGE  # Mappers pueden usar librerías de serialización
                },
                forbidden_prefixes={
                    ImportPrefix.SYSTEM
                },
                description=(
                    f"{keyword.name} puede importar module:, library:, package:. "
                    "NO puede importar system:ui."
                )
            )
        
        # ===== DESIGN PATTERNS =====
        # Sin restricciones específicas (depende del contexto)
        pattern_keywords = [
            VelaKeyword.FACTORY,
            VelaKeyword.BUILDER,
            VelaKeyword.STRATEGY,
            VelaKeyword.OBSERVER,
            VelaKeyword.SINGLETON,
            VelaKeyword.ADAPTER,
            VelaKeyword.DECORATOR
        ]
        for keyword in pattern_keywords:
            rules[keyword] = ImportRule(
                keyword=keyword,
                allowed_prefixes=set(ImportPrefix),
                forbidden_prefixes=set(),
                description=f"{keyword.name} puede importar cualquier prefijo (patrón genérico)."
            )
        
        # ===== TASK / PROVIDER / STORE =====
        rules[VelaKeyword.TASK] = ImportRule(
            keyword=VelaKeyword.TASK,
            allowed_prefixes=set(ImportPrefix),
            forbidden_prefixes=set(),
            description="task puede importar cualquier prefijo (job asíncrono)."
        )
        
        rules[VelaKeyword.PROVIDER] = ImportRule(
            keyword=VelaKeyword.PROVIDER,
            allowed_prefixes=set(ImportPrefix),
            forbidden_prefixes=set(),
            description="provider puede importar cualquier prefijo (DI container)."
        )
        
        rules[VelaKeyword.STORE] = ImportRule(
            keyword=VelaKeyword.STORE,
            allowed_prefixes={
                ImportPrefix.MODULE,
                ImportPrefix.LIBRARY,
                ImportPrefix.SYSTEM
            },
            forbidden_prefixes={
                ImportPrefix.PACKAGE
            },
            description="store puede importar system:reactive, module:, library:. NO package: directo."
        )
        
        # ===== MODULE =====
        # Módulos pueden importar otros módulos
        rules[VelaKeyword.MODULE] = ImportRule(
            keyword=VelaKeyword.MODULE,
            allowed_prefixes={
                ImportPrefix.MODULE,
                ImportPrefix.PACKAGE
            },
            forbidden_prefixes=set(),
            description="module puede importar otros modules y packages."
        )
        
        # ===== GENERIC (class, struct, enum, interface) =====
        # Sin restricciones (pueden importar lo necesario)
        generic_keywords = [
            VelaKeyword.CLASS,
            VelaKeyword.STRUCT,
            VelaKeyword.ENUM,
            VelaKeyword.INTERFACE
        ]
        for keyword in generic_keywords:
            rules[keyword] = ImportRule(
                keyword=keyword,
                allowed_prefixes=set(ImportPrefix),
                forbidden_prefixes=set(),
                description=f"{keyword.name} puede importar cualquier prefijo (tipo genérico)."
            )
        
        return rules
    
    def validate_import(
        self,
        keyword: VelaKeyword,
        import_statement: str,
        prefix: ImportPrefix,
        line: int = 0,
        column: int = 0
    ) -> Optional[ImportViolation]:
        """
        Valida un import específico contra las reglas del keyword.
        
        Args:
            keyword: Keyword que hace el import
            import_statement: Statement completo (ej: "import 'system:ui'")
            prefix: Prefijo usado (system, package, etc.)
            line: Línea del código
            column: Columna del código
        
        Returns:
            ImportViolation si hay error, None si es válido
        """
        # Obtener regla para el keyword
        rule = self.rules.get(keyword)
        if not rule:
            # Keyword sin reglas específicas, permitir
            return None
        
        # Verificar si el prefijo está permitido
        if not rule.allows(prefix):
            # Generar mensaje de error
            allowed = ", ".join(p.value for p in rule.allowed_prefixes) if rule.allowed_prefixes else "ninguno específico"
            forbidden = ", ".join(p.value for p in rule.forbidden_prefixes) if rule.forbidden_prefixes else "ninguno"
            
            message = (
                f"Import inválido en {keyword.name}: '{import_statement}' usa prefijo '{prefix.value}' "
                f"que no está permitido. "
                f"Permitidos: [{allowed}]. Prohibidos: [{forbidden}]."
            )
            
            # Sugerencia
            suggestion = None
            if rule.allowed_prefixes:
                suggestion = (
                    f"Considera usar uno de estos prefijos permitidos: "
                    f"{', '.join(p.value + ':' for p in rule.allowed_prefixes)}"
                )
            
            return ImportViolation(
                keyword=keyword,
                import_statement=import_statement,
                prefix_used=prefix,
                line=line,
                column=column,
                message=message,
                suggestion=suggestion
            )
        
        return None
    
    def validate_imports(
        self,
        keyword: VelaKeyword,
        imports: List[tuple[str, ImportPrefix, int, int]]
    ) -> List[ImportViolation]:
        """
        Valida múltiples imports de un archivo.
        
        Args:
            keyword: Keyword del archivo (service, widget, etc.)
            imports: Lista de (import_statement, prefix, line, column)
        
        Returns:
            Lista de violaciones encontradas
        """
        violations = []
        
        for import_stmt, prefix, line, col in imports:
            violation = self.validate_import(keyword, import_stmt, prefix, line, col)
            if violation:
                violations.append(violation)
        
        return violations
    
    def get_rule(self, keyword: VelaKeyword) -> Optional[ImportRule]:
        """Obtiene la regla para un keyword específico."""
        return self.rules.get(keyword)
    
    def get_allowed_prefixes(self, keyword: VelaKeyword) -> Set[ImportPrefix]:
        """Obtiene prefijos permitidos para un keyword."""
        rule = self.rules.get(keyword)
        return rule.allowed_prefixes if rule else set(ImportPrefix)
    
    def get_forbidden_prefixes(self, keyword: VelaKeyword) -> Set[ImportPrefix]:
        """Obtiene prefijos prohibidos para un keyword."""
        rule = self.rules.get(keyword)
        return rule.forbidden_prefixes if rule else set()


if __name__ == "__main__":
    # Demostración de uso
    validator = ImportValidator()
    
    print("=== VALIDACIÓN DE IMPORTS POR KEYWORD ===\n")
    
    # Test 1: widget intentando importar package:http (PROHIBIDO)
    print("Test 1: widget importando package:http")
    violation = validator.validate_import(
        keyword=VelaKeyword.WIDGET,
        import_statement="import 'package:http'",
        prefix=ImportPrefix.PACKAGE,
        line=5,
        column=0
    )
    if violation:
        print(f"❌ {violation.message}")
        if violation.suggestion:
            print(f"💡 {violation.suggestion}")
    else:
        print("✅ Import válido")
    
    print("\n" + "-" * 60 + "\n")
    
    # Test 2: widget importando system:ui (PERMITIDO)
    print("Test 2: widget importando system:ui")
    violation = validator.validate_import(
        keyword=VelaKeyword.WIDGET,
        import_statement="import 'system:ui'",
        prefix=ImportPrefix.SYSTEM,
        line=3,
        column=0
    )
    if violation:
        print(f"❌ {violation.message}")
    else:
        print("✅ Import válido")
    
    print("\n" + "-" * 60 + "\n")
    
    # Test 3: service intentando importar system:ui (PROHIBIDO)
    print("Test 3: service importando system:ui")
    violation = validator.validate_import(
        keyword=VelaKeyword.SERVICE,
        import_statement="import 'system:ui'",
        prefix=ImportPrefix.SYSTEM,
        line=8,
        column=0
    )
    if violation:
        print(f"❌ {violation.message}")
        if violation.suggestion:
            print(f"💡 {violation.suggestion}")
    else:
        print("✅ Import válido")
    
    print("\n" + "-" * 60 + "\n")
    
    # Test 4: entity importando package:lodash (PROHIBIDO)
    print("Test 4: entity importando package:lodash")
    violation = validator.validate_import(
        keyword=VelaKeyword.ENTITY,
        import_statement="import 'package:lodash'",
        prefix=ImportPrefix.PACKAGE,
        line=12,
        column=0
    )
    if violation:
        print(f"❌ {violation.message}")
        if violation.suggestion:
            print(f"💡 {violation.suggestion}")
    else:
        print("✅ Import válido")
    
    print("\n" + "-" * 60 + "\n")
    
    # Test 5: controller importando cualquier cosa (PERMITIDO)
    print("Test 5: controller importando package:express")
    violation = validator.validate_import(
        keyword=VelaKeyword.CONTROLLER,
        import_statement="import 'package:express'",
        prefix=ImportPrefix.PACKAGE,
        line=15,
        column=0
    )
    if violation:
        print(f"❌ {violation.message}")
    else:
        print("✅ Import válido")
    
    print("\n" + "=" * 60 + "\n")
    
    # Mostrar reglas de algunos keywords
    print("REGLAS DE IMPORTS POR KEYWORD:\n")
    
    for keyword in [VelaKeyword.WIDGET, VelaKeyword.SERVICE, VelaKeyword.ENTITY, VelaKeyword.CONTROLLER]:
        rule = validator.get_rule(keyword)
        if rule:
            allowed = ", ".join(p.value for p in rule.allowed_prefixes) if rule.allowed_prefixes else "todos"
            forbidden = ", ".join(p.value for p in rule.forbidden_prefixes) if rule.forbidden_prefixes else "ninguno"
            print(f"{keyword.name}:")
            print(f"  Permitidos: {allowed}")
            print(f"  Prohibidos: {forbidden}")
            print(f"  Descripción: {rule.description}")
            print()
