# TASK-011: Error Recovery

## 📋 Información General
- **Historia:** VELA-568 (Parser que genere AST válido)
- **Estimación:** 40 horas
- **Estado:** ✅ Completada
- **Fecha:** 2025-11-30
- **Commit:** f05543f
- **Archivo:** `src/parser/error_recovery.py` (650+ líneas)

## 🎯 Objetivo

Implementar estrategias avanzadas de **Error Recovery** para que el parser pueda:
1. **Continuar parsing** después de encontrar errores
2. **Reportar múltiples errores** en una sola pasada
3. **Sugerir correcciones** específicas para errores comunes de Vela
4. **Mejorar UX** del compilador con mensajes descriptivos

## 🔨 Implementación

### Arquitectura de Error Recovery

```
ErrorRecoveryParser (extiende Parser)
│
├── Panic Mode Recovery
│   ├── synchronize()              # Sincronizar en puntos seguros
│   ├── synchronize_to_declaration() # Sync a siguiente declaration
│   └── synchronize_to_statement() # Sync a siguiente statement
│
├── Phrase-Level Recovery
│   ├── try_insert_token()        # Insertar token faltante
│   └── try_delete_token()        # Eliminar token inesperado
│
├── Error Productions
│   ├── check_common_mistakes()   # Detectar errores comunes Vela
│   │   ├── let/const/var         # Keywords prohibidas
│   │   ├── null/undefined/nil    # Usar None/Option<T>
│   │   ├── for/while/loop        # Usar métodos funcionales
│   │   ├── switch/case           # Usar match
│   │   └── export                # Usar public modifier
│   │
│   └── suggest_correction()      # Generar sugerencias
│
└── Error Accumulation
    ├── errors: List[ParseError]  # Acumular errores
    ├── parse_with_recovery()     # Parsear y acumular
    └── collect_error_statistics() # Estadísticas
```

### Tipos de Errores

```python
class ErrorSeverity(Enum):
    """Severidad del error"""
    ERROR = "error"       # Error crítico
    WARNING = "warning"   # Advertencia
    INFO = "info"        # Información

@dataclass
class ParseError:
    """Error de parsing con contexto completo"""
    severity: ErrorSeverity
    message: str
    position: Position
    token: Optional[Token] = None
    expected: Optional[List[str]] = None
    suggestion: Optional[str] = None
    
    def __str__(self) -> str:
        return f"[{self.severity.value}] Line {self.position.line}: {self.message}"
```

### Estrategia 1: Panic Mode

El parser sincroniza en puntos seguros después de un error:

```python
# Sync tokens: puntos donde el parser puede recuperarse
SYNC_TOKENS = {
    'SEMICOLON',    # ;
    'RBRACE',       # }
    'FN',           # fn
    'STRUCT',       # struct
    'CLASS',        # class
    'ENUM',         # enum
    'INTERFACE',    # interface
    'SERVICE',      # service
    'REPOSITORY',   # repository
    # ... otros keywords de declaration
}

def synchronize(self):
    """
    Panic mode: avanzar hasta encontrar sync point
    """
    self.advance()
    
    while not self.is_at_end():
        # Sync en fin de statement
        if self.previous().type == 'SEMICOLON':
            return
        
        # Sync en inicio de declaration
        if self.current_token.type in SYNC_TOKENS:
            return
        
        self.advance()

def synchronize_to_declaration(self):
    """Sincronizar específicamente a declaration"""
    while not self.is_at_end():
        if self.current_token.type in ['FN', 'STRUCT', 'CLASS', 'ENUM']:
            return
        self.advance()

def synchronize_to_statement(self):
    """Sincronizar específicamente a statement"""
    while not self.is_at_end():
        if self.current_token.type in ['IF', 'MATCH', 'RETURN', 'LBRACE']:
            return
        if self.previous().type == 'SEMICOLON':
            return
        self.advance()
```

### Estrategia 2: Phrase-Level Recovery

Intenta reparar el error localmente antes de sincronizar:

```python
def try_insert_token(self, expected_type: str) -> bool:
    """
    Intenta insertar un token faltante
    
    Ejemplo: Missing } en función
    fn test() -> void {
        return 42
    # ← insertar } aquí
    """
    # Registrar error con sugerencia
    self.errors.append(ParseError(
        severity=ErrorSeverity.ERROR,
        message=f"Expected '{expected_type}', inserting it automatically",
        position=self.current_token.position,
        suggestion=f"Add '{expected_type}' before this token"
    ))
    
    # Simular que el token está ahí
    # (no avanzar current_token)
    return True

def try_delete_token(self) -> bool:
    """
    Intenta eliminar un token inesperado
    
    Ejemplo: Extra token
    fn test() -> void UNEXPECTED {
        return 42
    }
    """
    # Registrar error
    self.errors.append(ParseError(
        severity=ErrorSeverity.ERROR,
        message=f"Unexpected token '{self.current_token.value}', ignoring it",
        position=self.current_token.position,
        token=self.current_token,
        suggestion="Remove this token"
    ))
    
    # Saltar token
    self.advance()
    return True
```

### Estrategia 3: Error Productions

Detecta errores comunes de Vela y sugiere correcciones:

```python
def check_common_mistakes(self) -> Optional[ParseError]:
    """
    Detecta errores comunes del lenguaje Vela
    
    Prohibiciones:
    - let, const, var → usar state o nada
    - null, undefined, nil → usar None o Option<T>
    - for, while, loop → usar métodos funcionales
    - switch, case → usar match
    - export → usar public modifier
    """
    token = self.current_token
    
    # 1. Keywords de mutabilidad prohibidas
    if token.type == 'IDENTIFIER' and token.value in ['let', 'const', 'var']:
        return ParseError(
            severity=ErrorSeverity.ERROR,
            message=f"'{token.value}' is not a keyword in Vela",
            position=token.position,
            token=token,
            suggestion=(
                "Use 'state' for mutable variables or nothing for immutable variables.\n"
                "Examples:\n"
                "  state count: Number = 0  # mutable\n"
                "  name: String = 'Vela'    # immutable"
            )
        )
    
    # 2. Null values prohibidos
    if token.type == 'IDENTIFIER' and token.value in ['null', 'undefined', 'nil']:
        return ParseError(
            severity=ErrorSeverity.ERROR,
            message=f"'{token.value}' is not allowed in Vela",
            position=token.position,
            token=token,
            suggestion=(
                "Use 'None' or 'Option<T>' for optional values.\n"
                "Examples:\n"
                "  value: Option<String> = None\n"
                "  result: Option<Number> = Some(42)"
            )
        )
    
    # 3. Loops imperativos prohibidos
    if token.type == 'FOR' or (token.type == 'IDENTIFIER' and token.value in ['while', 'loop']):
        return ParseError(
            severity=ErrorSeverity.ERROR,
            message=f"'{token.value}' loops are not allowed in Vela (functional programming)",
            position=token.position,
            token=token,
            suggestion=(
                "Use functional methods instead:\n"
                "  .forEach(fn)  # Execute function for each element\n"
                "  .map(fn)      # Transform elements\n"
                "  .filter(fn)   # Filter elements\n"
                "  .reduce(fn)   # Reduce to single value\n"
                "Example:\n"
                "  (0..10).forEach(i => print(i))"
            )
        )
    
    # 4. Switch prohibido
    if token.type == 'IDENTIFIER' and token.value == 'switch':
        return ParseError(
            severity=ErrorSeverity.ERROR,
            message="'switch' is not allowed in Vela",
            position=token.position,
            token=token,
            suggestion=(
                "Use 'match' with pattern matching instead.\n"
                "Example:\n"
                "  match value {\n"
                "    1 => print('one')\n"
                "    2 => print('two')\n"
                "    _ => print('other')\n"
                "  }"
            )
        )
    
    # 5. Export prohibido
    if token.type == 'IDENTIFIER' and token.value == 'export':
        return ParseError(
            severity=ErrorSeverity.ERROR,
            message="'export' keyword is not used in Vela",
            position=token.position,
            token=token,
            suggestion=(
                "Use 'public' modifier instead.\n"
                "Example:\n"
                "  public fn myFunction() -> void { }"
            )
        )
    
    return None
```

### parse_with_recovery()

Método principal que parsea y acumula errores:

```python
def parse_with_recovery(self) -> Tuple[Program, List[ParseError]]:
    """
    Parsea el programa completo acumulando errores
    
    Returns:
        Tuple[Program, List[ParseError]]: AST (posiblemente incompleto) y lista de errores
    """
    self.errors = []
    
    imports = []
    declarations = []
    
    # Parse imports
    while self.current_token.type == 'IMPORT':
        try:
            imports.append(self.parse_import())
        except ParserError as e:
            # Registrar error
            self.errors.append(ParseError(
                severity=ErrorSeverity.ERROR,
                message=e.message,
                position=self.current_token.position,
                token=self.current_token
            ))
            # Sincronizar
            self.synchronize_to_declaration()
    
    # Parse declarations
    while not self.is_at_end():
        # Check common mistakes antes de parsear
        mistake = self.check_common_mistakes()
        if mistake:
            self.errors.append(mistake)
            self.advance()
            continue
        
        try:
            declarations.append(self.parse_declaration())
        except ParserError as e:
            # Registrar error
            self.errors.append(ParseError(
                severity=ErrorSeverity.ERROR,
                message=e.message,
                position=self.current_token.position,
                token=self.current_token
            ))
            # Sincronizar
            self.synchronize_to_declaration()
    
    program = Program(imports=imports, declarations=declarations)
    return program, self.errors
```

### Error Statistics

```python
@dataclass
class ErrorStatistics:
    """Estadísticas de errores"""
    total_errors: int = 0
    errors_by_severity: Dict[ErrorSeverity, int] = field(default_factory=dict)
    recovery_attempts: int = 0
    successful_recoveries: int = 0

def collect_error_statistics(self) -> ErrorStatistics:
    """Recopila estadísticas de errores"""
    stats = ErrorStatistics(
        total_errors=len(self.errors)
    )
    
    # Contar por severidad
    for error in self.errors:
        if error.severity not in stats.errors_by_severity:
            stats.errors_by_severity[error.severity] = 0
        stats.errors_by_severity[error.severity] += 1
    
    return stats
```

### Formateo de Errores

```python
def format_errors(self, errors: List[ParseError]) -> str:
    """
    Formatea errores para display user-friendly
    
    Output:
    ========================================
    Found 3 errors:
    ========================================
    
    [error] Line 3: 'let' is not a keyword in Vela
      → Use 'state' for mutable variables
    
    [error] Line 5: 'null' is not allowed in Vela
      → Use 'None' or 'Option<T>'
    
    [error] Line 8: 'for' loops not allowed
      → Use .forEach() or other functional methods
    ========================================
    """
    if not errors:
        return "No errors found."
    
    output = []
    output.append("=" * 60)
    output.append(f"Found {len(errors)} error(s):")
    output.append("=" * 60)
    output.append("")
    
    for error in errors:
        # Error header
        output.append(f"[{error.severity.value}] Line {error.position.line}: {error.message}")
        
        # Suggestion
        if error.suggestion:
            for line in error.suggestion.split('\n'):
                output.append(f"  → {line}")
        
        output.append("")
    
    output.append("=" * 60)
    return '\n'.join(output)
```

## 📊 Errores Comunes Detectados

### 1. **Keywords Prohibidas de Mutabilidad**
```python
# ❌ ERROR
let x = 5
const PI = 3.14
var counter = 0

# ✅ SUGERENCIA
state x: Number = 5         # mutable
PI: Float = 3.14            # immutable (por defecto)
state counter: Number = 0   # mutable
```

### 2. **Null Values Prohibidos**
```python
# ❌ ERROR
value = null
data = undefined
result = nil

# ✅ SUGERENCIA
value: Option<String> = None
data: Option<Number> = None
result: Option<Bool> = Some(true)
```

### 3. **Loops Imperativos Prohibidos**
```python
# ❌ ERROR
for i in 0..10 {
    print(i)
}

while condition {
    doSomething()
}

# ✅ SUGERENCIA
(0..10).forEach(i => print(i))

# Usar recursión o métodos funcionales
items.map(x => x * 2).filter(x => x > 10)
```

### 4. **Switch Prohibido**
```python
# ❌ ERROR
switch value {
    case 1: print("one")
    case 2: print("two")
    default: print("other")
}

# ✅ SUGERENCIA
match value {
    1 => print("one")
    2 => print("two")
    _ => print("other")
}
```

### 5. **Export Prohibido**
```python
# ❌ ERROR
export fn myFunction() { }

# ✅ SUGERENCIA
public fn myFunction() -> void { }
```

## 📁 Ubicación de Archivos

```
src/parser/error_recovery.py     # Implementación (650+ líneas)
src/parser/__init__.py           # Exports (incluye ErrorRecoveryParser)
```

## ✅ Criterios de Aceptación

- [x] ErrorRecoveryParser implementado
- [x] Panic mode con sincronización en puntos seguros
- [x] Phrase-level recovery (insert/delete tokens)
- [x] Error productions para errores comunes Vela
- [x] Detección de: let/const/var, null/undefined, for/while, switch, export
- [x] Sugerencias específicas por cada error
- [x] Acumulación de múltiples errores
- [x] parse_with_recovery() retorna AST + errores
- [x] Error statistics
- [x] Formateo user-friendly de errores
- [x] Código committeado y versionado

## 🎓 Decisiones de Diseño

### 1. **Tres Niveles de Recovery**

**Nivel 1: Local (Phrase-Level)**
- Intenta reparar inmediatamente
- Insert/delete tokens

**Nivel 2: Statement (Synchronize)**
- Si nivel 1 falla, sincroniza a statement
- Continúa parsing

**Nivel 3: Declaration (Synchronize)**
- Si nivel 2 falla, sincroniza a declaration
- Última línea de defensa

### 2. **Error Severity**
```python
ERROR   # Bloquea compilación
WARNING # Advertencia, no bloquea
INFO    # Informativo
```

### 3. **Sugerencias Específicas de Vela**
No solo reportar error, sino enseñar el lenguaje:
```
Error: 'let' is not a keyword
Suggestion: Use 'state' for mutable or nothing for immutable
Examples: ...
```

### 4. **Continuar Siempre**
Parser nunca aborta, siempre intenta completar el AST.

## 📊 Métricas

- **Total líneas:** 650+
- **Estrategias:** 3 (panic, phrase-level, error productions)
- **Errores comunes detectados:** 5 tipos
- **Commit:** f05543f

## 🔗 Referencias

- **Jira:** [VELA-568](https://velalang.atlassian.net/browse/VELA-568)
- **Historia:** [Sprint 6](../README.md)
- **Archivo:** `src/parser/error_recovery.py`
- **Anterior:** [TASK-009: Pratt Parser](./TASK-009.md)
- **Siguiente:** [TASK-012: Test Suite](./TASK-012.md)

---

**Autor:** GitHub Copilot Agent  
**Fecha:** 2025-11-30  
**Commit:** f05543f
