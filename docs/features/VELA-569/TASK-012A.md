# TASK-012A: Imports con Prefijos

## 📋 Información General
- **Historia:** VELA-569
- **Estado:** Completada ✅ (en Sprint 6)
- **Fecha:** 2025-11-30 (Sprint 6)

## 🎯 Objetivo
Implementar sistema de imports con prefijos para distinguir diferentes tipos de módulos (system, package, module, library, extension, assets).

## 🔨 Implementación

### Estado
**Esta subtask YA FUE COMPLETADA en Sprint 6 (VELA-568).**

El sistema de imports con prefijos fue implementado como parte del parser base en Sprint 6. No se requirió trabajo adicional en Sprint 7.

### Archivos implementados (Sprint 6)
- `src/parser/parser.py` - Método `parse_import()` (líneas 242-318)
- `src/parser/ast_nodes.py` - `ImportKind` enum y `ImportDeclaration` (líneas 100-135)
- `src/lexer/token.py` - Tokens IMPORT, FROM, SHOW, HIDE, AS

## ✅ Características

### 6 Tipos de Prefijos Soportados

| Prefijo | Propósito | Ejemplo |
|---------|-----------|---------|
| `system:` | Módulos del sistema Vela | `import 'system:io'` |
| `package:` | Paquetes externos (npm-like) | `import 'package:http'` |
| `module:` | Módulos locales del proyecto | `import 'module:utils'` |
| `library:` | Bibliotecas compartidas | `import 'library:math'` |
| `extension:` | Extensiones del lenguaje | `import 'extension:async'` |
| `assets:` | Assets (imágenes, etc.) | `import 'assets:logo.png'` |

### Sintaxis de Imports

```vela
# Import completo
import 'system:io'

# Import con alias
import 'package:very-long-name' as vln

# Import selectivo (show)
import 'module:utils' show { sort, filter, map }

# Import con exclusión (hide)
import 'system:math' hide { deprecated_fn }

# Import sin prefijo (default: module)
import 'local-file'  # Se trata como module:local-file
```

### AST Nodes

```python
class ImportKind(Enum):
    """Tipos de imports"""
    SYSTEM = "system"       # system: - Módulos del sistema
    PACKAGE = "package"     # package: - Paquetes externos
    MODULE = "module"       # module: - Módulos locales
    LIBRARY = "library"     # library: - Bibliotecas
    EXTENSION = "extension" # extension: - Extensiones
    ASSETS = "assets"       # assets: - Assets

@dataclass
class ImportDeclaration(Declaration):
    """Import statement"""
    path: str
    alias: Optional[str]
    show: List[str]  # Símbolos específicos a importar
    hide: List[str]  # Símbolos a excluir
    kind: ImportKind  # Tipo de import
```

### Lógica de Detección de Prefijos

El parser detecta el prefijo desde el string del path:

```python
def parse_import(self, is_public: bool = False) -> ImportDeclaration:
    """Parsea import statement"""
    start = self.expect(TokenType.IMPORT)
    path = self.expect(TokenType.STRING).value
    
    # Detectar tipo de import desde prefijo
    kind = ImportKind.MODULE  # Default
    if path.startswith("system:"):
        kind = ImportKind.SYSTEM
    elif path.startswith("package:"):
        kind = ImportKind.PACKAGE
    elif path.startswith("module:"):
        kind = ImportKind.MODULE
    elif path.startswith("library:"):
        kind = ImportKind.LIBRARY
    elif path.startswith("extension:"):
        kind = ImportKind.EXTENSION
    elif path.startswith("assets:"):
        kind = ImportKind.ASSETS
    
    # ... parse alias, show, hide
    
    return ImportDeclaration(
        range=self.create_range_from_tokens(start, end),
        is_public=is_public,
        path=path,
        alias=alias,
        show=show,
        hide=hide,
        kind=kind
    )
```

## 📊 Métricas
- **Estimación:** 24 horas
- **Tiempo real:** 0 horas (Sprint 7) - Ya completado en Sprint 6
- **Prefijos implementados:** 6
- **Líneas de código:** ~76 (parser) + ~36 (AST nodes) = 112 líneas (Sprint 6)

## ✅ Criterios de Aceptación
- [x] Sistema de prefijos implementado
- [x] 6 tipos de imports soportados
- [x] Parse de alias (`as`)
- [x] Parse de `show` (import selectivo)
- [x] Parse de `hide` (import con exclusión)
- [x] AST nodes creados (ImportKind, ImportDeclaration)
- [x] Default kind = MODULE cuando no hay prefijo

## 🔗 Referencias
- **Jira:** [TASK-012A](https://velalang.atlassian.net/browse/VELA-569)
- **Historia:** [VELA-569](https://velalang.atlassian.net/browse/VELA-569)
- **Implementado en:** Sprint 6 (VELA-568)
- **Código:** `src/parser/parser.py` líneas 242-318

## 📝 Notas
- Esta funcionalidad fue implementada completamente en Sprint 6
- No se requirió trabajo adicional en Sprint 7
- Los imports sin prefijo se tratan como `module:` por defecto
- El sistema es extensible para agregar más tipos de prefijos en el futuro
