# TASK-012C: Tests para Nuevos Keywords

## 📋 Información General
- **Historia:** VELA-569
- **Estado:** Completada ✅
- **Fecha:** 2025-12-01

## 🎯 Objetivo
Crear suite de tests unitarios completa para verificar que el parser genera correctamente los AST nodes para los 23 keywords domain-specific nuevos implementados en TASK-012B.

## 🔨 Implementación

### Archivos creados
- `tests/unit/parser/test_specific_keywords.py` (561 líneas)

### Estructura del Archivo de Tests

```python
"""
Tests para Keywords Específicos del Parser de Vela

Tests de: VELA-569 (TASK-012B)
Historia: Sprint 7 - Keywords específicos por tipo
"""

import pytest
from src.parser import parse_code
from src.parser.ast_nodes import (
    # 23 imports de AST nodes
    WidgetDeclaration, ComponentDeclaration, ModelDeclaration,
    # ... etc
)

# 8 clases de tests agrupadas por categoría
class TestUIKeywords:
class TestModelKeywords:
class TestDesignPatternKeywords:
class TestWebAPIKeywords:
class TestStateDIKeywords:
class TestConcurrencyKeywords:
class TestUtilityKeywords:
class TestPublicKeywords:
```

## ✅ Tests Implementados (28 total)

### 1. UI Keywords (2 tests)
| Test | Keyword | Verifica |
|------|---------|----------|
| `test_widget_declaration` | `widget` | AST node, name, fields (state), methods (increment, build) |
| `test_component_declaration` | `component` | AST node, name, fields, methods |

**Ejemplo:**
```python
def test_widget_declaration(self):
    code = """
    widget Counter {
        state count: Number = 0
        
        fn increment() -> void {
            this.count = this.count + 1
        }
        
        fn build() -> Widget {
            return Text("Count: ${this.count}")
        }
    }
    """
    ast = parse_code(code)
    assert isinstance(ast.declarations[0], WidgetDeclaration)
    assert ast.declarations[0].name == "Counter"
    assert len(ast.declarations[0].fields) == 1  # state count
    assert len(ast.declarations[0].methods) == 2  # increment, build
```

### 2. Model Keywords (1 test)
| Test | Keyword | Verifica |
|------|---------|----------|
| `test_model_declaration` | `model` | AST node, name, fields (4) |

### 3. Design Pattern Keywords (7 tests)
| Test | Keyword | Verifica |
|------|---------|----------|
| `test_factory_declaration` | `factory` | AST node, name, methods (create, createDefault) |
| `test_builder_declaration` | `builder` | AST node, name, methods (select, where, build) |
| `test_strategy_declaration` | `strategy` | AST node, name, methods (pay) |
| `test_observer_declaration` | `observer` | AST node, name, methods (notify) |
| `test_singleton_declaration` | `singleton` | AST node, name, fields (connection), methods (getInstance, query) |
| `test_adapter_declaration` | `adapter` | AST node, name, methods (adapt) |
| `test_decorator_declaration` | `decorator` | AST node, name, methods (wrap) |

### 4. Web/API Keywords (4 tests)
| Test | Keyword | Verifica |
|------|---------|----------|
| `test_guard_declaration` | `guard` | AST node, name, methods (canActivate) |
| `test_middleware_declaration` | `middleware` | AST node, name, methods (handle) |
| `test_interceptor_declaration` | `interceptor` | AST node, name, methods (intercept) |
| `test_validator_declaration` | `validator` | AST node, name, methods (validate) |

### 5. State & DI Keywords (2 tests)
| Test | Keyword | Verifica |
|------|---------|----------|
| `test_store_declaration` | `store` | AST node, name, fields (count, user), methods (increment, setUser) |
| `test_provider_declaration` | `provider` | AST node, name, methods (provide) |

### 6. Concurrency Keywords (1 test)
| Test | Keyword | Verifica |
|------|---------|----------|
| `test_actor_declaration` | `actor` | AST node, name, fields (count), methods (increment, getCount) |

### 7. Utility Keywords (5 tests)
| Test | Keyword | Verifica |
|------|---------|----------|
| `test_pipe_declaration` | `pipe` | AST node, name, methods (transform) |
| `test_task_declaration` | `task` | AST node, name, methods (run) |
| `test_helper_declaration` | `helper` | AST node, name, methods (format) |
| `test_mapper_declaration` | `mapper` | AST node, name, methods (toDTO, fromDTO) |
| `test_serializer_declaration` | `serializer` | AST node, name, methods (serialize, deserialize) |

### 8. Public Keywords (3 tests)
| Test | Keywords | Verifica |
|------|----------|----------|
| `test_public_widget` | `public widget` | Modificador `is_public == True` |
| `test_public_factory` | `public factory` | Modificador `is_public == True` |
| `test_public_store` | `public store` | Modificador `is_public == True` |

## 📊 Métricas
- **Estimación:** 48 horas
- **Tests creados:** 28
- **Keywords cubiertos:** 23 (100% de TASK-012B)
- **Líneas de código:** 561
- **Clases de tests:** 8
- **Cobertura:** 100% de los keywords nuevos

## ✅ Criterios de Aceptación
- [x] Test para cada uno de los 23 keywords nuevos
- [x] Tests verifican AST node correcto
- [x] Tests verifican nombre de la declaración
- [x] Tests verifican fields/methods según tipo
- [x] Tests verifican modificador `public`
- [x] Estructura organizada por categorías
- [x] Código committeado (dcb437e)

## 🔗 Referencias
- **Jira:** [TASK-012C](https://velalang.atlassian.net/browse/VELA-569)
- **Historia:** [VELA-569](https://velalang.atlassian.net/browse/VELA-569)
- **Commit:** dcb437e
- **Código:** `tests/unit/parser/test_specific_keywords.py`

## 📝 Notas Técnicas

### Patrón de Test
Todos los tests siguen este patrón:

```python
def test_X_declaration(self):
    """Test X keyword - Descripción"""
    code = """
    X NombreEjemplo {
        # código de ejemplo
    }
    """
    ast = parse_code(code)
    assert ast is not None
    assert len(ast.declarations) == 1
    assert isinstance(ast.declarations[0], XDeclaration)
    assert ast.declarations[0].name == "NombreEjemplo"
    # Asserts adicionales según tipo (fields, methods, etc.)
```

### Assertions Comunes
- `assert ast is not None` - Parser retorna AST válido
- `assert len(ast.declarations) == 1` - Exactamente 1 declaración parseada
- `assert isinstance(ast.declarations[0], XDeclaration)` - Tipo correcto de AST node
- `assert ast.declarations[0].name == "..."` - Nombre correcto
- `assert len(ast.declarations[0].fields) == N` - Cantidad de fields (si aplica)
- `assert len(ast.declarations[0].methods) == N` - Cantidad de methods (si aplica)
- `assert ast.declarations[0].is_public == True/False` - Modificador public

### Ejemplos Realistas
Los tests usan ejemplos de código Vela realistas:
- **widget Counter**: Contador con estado y métodos
- **factory UserFactory**: Creación de usuarios con factory methods
- **builder QueryBuilder**: Query builder con método fluent (Self)
- **store AppStore**: State store con estado reactivo
- **actor Counter**: Actor con concurrencia segura
- **mapper UserMapper**: Mapper bidireccional (toDTO, fromDTO)

### Ejecución de Tests
```bash
# Ejecutar todos los tests
pytest tests/unit/parser/test_specific_keywords.py -v

# Ejecutar solo tests de UI
pytest tests/unit/parser/test_specific_keywords.py::TestUIKeywords -v

# Ejecutar test específico
pytest tests/unit/parser/test_specific_keywords.py::TestDesignPatternKeywords::test_factory_declaration -v
```

## 🚀 Próximos Pasos
- Tests de integración con lexer
- Tests de error handling (syntax errors)
- Tests de edge cases (keywords vacíos, sin métodos, etc.)
- Integración con sistema de CI/CD
