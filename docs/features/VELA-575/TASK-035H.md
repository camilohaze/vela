# TASK-035H: Circular Dependency Detection

## 📋 Información General
- **Historia:** VELA-575 - Sistema de Dependency Injection
- **Epic:** VELA-574 - Runtime System
- **Estado:** Completada ✅
- **Sprint:** 13
- **Prioridad:** P0 (Bloqueante)
- **Estimado:** 32h
- **Fecha:** 2025-12-01

## 🎯 Objetivo

Implementar un sistema robusto de detección de dependencias circulares para el DI container de Vela, con análisis estático, detección en runtime, visualización de grafos, y advertencias preventivas.

## 📦 Componentes Implementados

### 1. CircularDependencyError (Enhanced)

**Ubicación:** `src/runtime/di/errors.py`

**Mejoras:**
- Muestra cadena completa del ciclo: `"A → B → C → A"`
- 4 sugerencias accionables para romper ciclos:
  1. **@lazy Injection**: Inyectar dependencia de forma perezosa
  2. **Intermediate Service**: Crear servicio intermedio para dividir responsabilidades
  3. **Event-Driven**: Usar eventos/observer pattern para desacoplar
  4. **Refactor**: Reestructurar código para eliminar dependencia mutua

**Ejemplo:**
```python
raise CircularDependencyError(
    cycle=[ServiceA, ServiceB, ServiceA],
    message="Circular dependency: ServiceA → ServiceB → ServiceA"
)
```

**Output:**
```
CircularDependencyError: Circular dependency detected: ServiceA → ServiceB → ServiceA

Suggestions to break the cycle:
  1. Use @lazy injection for one of the dependencies
  2. Create an intermediate service to split responsibilities
  3. Use event-driven communication instead of direct dependencies
  4. Consider refactoring to remove the mutual dependency
```

### 2. DependencyGraph (Static Analysis)

**Ubicación:** `src/runtime/di/graph_analyzer.py`

**API:**

#### `DependencyGraph.from_injector(injector: Injector) -> DependencyGraph`
Construye grafo desde un Injector registrado.

```python
from src.runtime.di import Injector, DependencyGraph

injector = Injector()
injector.register(ServiceA)
injector.register(ServiceB)

graph = DependencyGraph.from_injector(injector)
```

#### `find_cycles() -> List[List[Type]]`
Encuentra todos los ciclos en el grafo usando DFS con resolution stack.

**Algoritmo:** DFS con backtracking (O(V+E))
- Detecta ciclos durante el traversal
- Retorna path completo de cada ciclo
- Maneja múltiples ciclos independientes

```python
cycles = graph.find_cycles()
# [[ServiceA, ServiceB, ServiceA], [ServiceC, ServiceD, ServiceC]]
```

#### `visualize(highlight_cycles: bool = True) -> str`
Genera visualización ASCII del grafo de dependencias.

```python
print(graph.visualize())
```

**Output:**
```
Dependency Graph:
└─ ServiceA
   └─ ServiceB (CYCLE!)
      └─ ServiceA (CYCLE!)
```

#### `suggest_fixes() -> List[Dict]`
Genera sugerencias contextuales para romper ciclos.

```python
fixes = graph.suggest_fixes()
# [
#   {
#     "cycle": [ServiceA, ServiceB, ServiceA],
#     "suggestions": [
#       {"type": "lazy_injection", "target": ServiceB, ...},
#       {"type": "intermediate_service", ...}
#     ]
#   }
# ]
```

#### `get_statistics() -> Dict`
Retorna métricas del grafo.

```python
stats = graph.get_statistics()
# {
#   "total_nodes": 10,
#   "total_edges": 15,
#   "cyclic_nodes": 3,
#   "max_depth": 4
# }
```

### 3. Static Verification

**Ubicación:** `src/runtime/di/injector.py`

#### Opción 1: Manual Validation

```python
from src.runtime.di import Injector, verify_no_cycles

injector = Injector()
injector.register(ServiceA)
injector.register(ServiceB)

# Validar manualmente en cualquier momento
verify_no_cycles(injector)
# Throws CircularDependencyError if cycles found
```

**Método equivalente:**
```python
injector.validate_no_cycles()
```

#### Opción 2: Auto-Validation

```python
# Validar automáticamente en cada register()
injector = Injector(validate_cycles=True)

injector.register(ServiceA)  # OK
injector.register(ServiceB)  # Throws CircularDependencyError!
```

### 4. CycleWarningDetector (Preventive Warnings)

**Ubicación:** `src/runtime/di/cycle_warnings.py`

Detecta "near-cycles" ANTES de que se forme un ciclo completo.

**API:**

#### `check_near_cycles(injector, from_type, to_type)`
Verifica si agregar una dependencia `from_type → to_type` formaría un ciclo.

```python
from src.runtime.di import check_near_cycles

warnings = check_near_cycles(injector, ServiceA, ServiceB)

for warning in warnings:
    print(f"Warning: Adding {warning.from_type} → {warning.to_type}")
    print(f"  Would create cycle: {warning.potential_cycle}")
```

#### `CycleWarningDetector`
Detector reutilizable con logging integrado.

```python
from src.runtime.di.cycle_warnings import CycleWarningDetector

detector = CycleWarningDetector(injector)
warnings = detector.check_dependency(ServiceA, ServiceB, log_warnings=True)
```

## 🔬 Análisis de Algoritmo (ADR-035H)

**Decisión:** DFS con Resolution Stack

**Algoritmos considerados:**

| Algoritmo | Complejidad | Detecta Path | Ventajas | Desventajas |
|-----------|-------------|--------------|----------|-------------|
| **DFS con Stack** | O(V+E) | ✅ Sí | Simple, detecta path completo, memoria O(V) | - |
| Tarjan's SCC | O(V+E) | ❌ No | Encuentra SCC, óptimo para grafos grandes | Overkill para DI (grafos pequeños), NO retorna path |
| Kahn's Algorithm | O(V+E) | ❌ No | Detecta ciclos (topological sort fail) | NO retorna path del ciclo |

**Elegido:** DFS con Resolution Stack
- **Simple:** Implementación directa y fácil de mantener
- **Completo:** Retorna path exacto del ciclo para error messages
- **Eficiente:** O(1) detección por nodo (resolution stack)
- **Suficiente:** Grafos de DI suelen ser pequeños (<100 nodos)

**Referencia:** `docs/architecture/ADR-035H-circular-dependency-detection.md`

## 📝 Forward Reference Resolution

**Problema:** Forward references en type hints (`'ServiceB'` strings) no se resolvían correctamente.

**Solución Implementada:**

### 1. `inject()` Refactorizado

**Antes:** `inject()` retornaba función decorador sin metadata
```python
def inject(token=None):
    def parameter_decorator(param):
        # ...
    return parameter_decorator
```

**Después:** `inject()` retorna objeto sentinel con metadata
```python
class _InjectMarker:
    def __init__(self, token=None):
        self.__inject_metadata__ = InjectMetadata(
            param_name="__placeholder__",
            token=token
        )

def inject(token=None):
    return _InjectMarker(token)
```

### 2. Dependency Extraction con Forward Reference Handling

```python
def _extract_constructor_dependencies(self, cls: Type) -> List[Type]:
    """Extraer dependencias, resolviendo forward references."""
    constructor_meta = get_constructor_inject_metadata(cls)
    
    resolved_deps = []
    for meta in constructor_meta:
        token = meta.token
        
        # Si token es string (forward ref), buscar en registry
        if isinstance(token, str):
            for registered_token in self._registry._providers.keys():
                if registered_token.__name__ == token:
                    resolved_deps.append(registered_token)
                    break
        else:
            resolved_deps.append(token)
    
    return resolved_deps
```

### 3. Auto-Update de Forward References

Después de cada `register()`, re-procesar dependencias con forward references sin resolver:

```python
def _update_forward_references(self) -> None:
    """Re-procesar providers con forward references."""
    for token, entry in self._registry._providers.items():
        # Si hay strings en dependencies (forward refs sin resolver)
        has_unresolved = any(isinstance(dep, str) for dep in entry.dependencies)
        
        if has_unresolved:
            # Re-extraer dependencies (ahora con más tipos en registry)
            updated_deps = self._extract_constructor_dependencies(token)
            entry.dependencies = updated_deps
```

**Flujo:**
1. `ServiceA` registrado con dependencia `'ServiceB'` (string)
2. `ServiceB` registrado
3. `_update_forward_references()` llamado automáticamente
4. `ServiceA.dependencies` actualizado de `['ServiceB']` a `[<class 'ServiceB'>]`

## ✅ Tests

**Ubicación:** `tests/unit/di/test_circular_deps.py`

**Resultados:** 19/19 tests pasando (100% ✅)

**Categorías:**

### 1. Error Messages (1 test)
- ✅ `test_circular_dependency_error_message_has_suggestions`: Verifica mensaje con 4 sugerencias

### 2. Cycle Detection (8 tests)
- ✅ `test_simple_cycle_2_nodes`: A→B→A
- ✅ `test_simple_cycle_runtime_detection`: Runtime CircularDependencyError
- ✅ `test_long_cycle_4_nodes`: A→B→C→D→A
- ✅ `test_multiple_cycles`: Detecta múltiples ciclos independientes
- ✅ `test_self_dependency`: A→A
- ✅ `test_dependency_graph_find_cycles`: API DependencyGraph
- ✅ `test_dependency_graph_visualize`: ASCII visualization
- ✅ `test_dependency_graph_visualize_with_cycle`: CYCLE! markers

### 3. Static Verification (2 tests)
- ✅ `test_verify_no_cycles_success`: No lanza error si sin ciclos
- ✅ `test_verify_no_cycles_failure`: Lanza CircularDependencyError

### 4. Validate Methods (2 tests)
- ✅ `test_injector_validate_no_cycles_method`: `injector.validate_no_cycles()`
- ✅ `test_injector_validate_cycles_option`: `Injector(validate_cycles=True)`

### 5. Analyze Injector (1 test)
- ✅ `test_analyze_injector`: DependencyGraph.from_injector()

### 6. Near-Cycle Warnings (2 tests)
- ✅ `test_check_near_cycles_no_warnings`: Sin warnings cuando no hay near-cycles
- ✅ `test_check_near_cycles_with_warning`: Detecta warning antes de ciclo

### 7. Suggest Fixes (1 test)
- ✅ `test_dependency_graph_suggest_fixes`: Sugerencias contextuales

### 8. Statistics (1 test)
- ✅ `test_dependency_graph_get_statistics`: Métricas del grafo

### 9. Cycle Warning Detector (1 test)
- ✅ `test_cycle_warning_detector`: API CycleWarningDetector

## 📚 Ejemplos de Uso

### Ejemplo 1: Detección Básica

```python
from src.runtime.di import Injector, injectable, inject

@injectable
class ServiceA:
    def __init__(self, b: 'ServiceB' = inject('ServiceB')):
        self.b = b

@injectable
class ServiceB:
    def __init__(self, a: ServiceA = inject(ServiceA)):
        self.a = a

injector = Injector()
injector.register(ServiceA)
injector.register(ServiceB)

try:
    injector.resolve(ServiceA)
except CircularDependencyError as e:
    print(e)
    # Circular dependency detected: ServiceA → ServiceB → ServiceA
```

### Ejemplo 2: Análisis Estático

```python
from src.runtime.di import Injector, DependencyGraph

injector = Injector()
# ... register services ...

# Analizar grafo
graph = DependencyGraph.from_injector(injector)
cycles = graph.find_cycles()

if cycles:
    print("Cycles detected:")
    for cycle in cycles:
        cycle_names = [cls.__name__ for cls in cycle]
        print(f"  {' → '.join(cycle_names)}")
    
    # Mostrar visualización
    print("\nDependency Graph:")
    print(graph.visualize())
    
    # Obtener sugerencias
    fixes = graph.suggest_fixes()
    for fix in fixes:
        print(f"\nSuggestions for cycle:")
        for suggestion in fix["suggestions"]:
            print(f"  - {suggestion['type']}: {suggestion['description']}")
```

### Ejemplo 3: Auto-Validación

```python
# Validar automáticamente en cada register()
injector = Injector(validate_cycles=True)

try:
    injector.register(ServiceA)
    injector.register(ServiceB)  # Lanza CircularDependencyError aquí!
except CircularDependencyError as e:
    print("Cycle detected after registering ServiceB")
    print(e)
```

### Ejemplo 4: Advertencias Preventivas

```python
from src.runtime.di import check_near_cycles

# Antes de registrar ServiceB
warnings = check_near_cycles(injector, ServiceB, ServiceA)

if warnings:
    print("WARNING: Adding this dependency would create a cycle!")
    for warning in warnings:
        print(f"  Potential cycle: {warning.potential_cycle}")
    # Decide: abortar o usar @lazy injection
```

## 📊 Métricas

**Archivos modificados/creados:** 5
- `src/runtime/di/injector.py` (modificado)
- `src/runtime/di/inject.py` (modificado)
- `src/runtime/di/graph_analyzer.py` (creado, 400 líneas)
- `src/runtime/di/cycle_warnings.py` (creado, 280 líneas)
- `src/runtime/di/errors.py` (modificado)

**Archivos de documentación:** 2
- `docs/architecture/ADR-035H-circular-dependency-detection.md` (450 líneas)
- `docs/features/VELA-575/TASK-035H.md` (este archivo)

**Tests:** 19 tests (100% passing ✅)

**Líneas de código totales:** ~1,400 líneas

**Cobertura de tests:** >95% (estimado)

**Versión:** 0.12.0

## 🔗 Referencias

- **ADR:** `docs/architecture/ADR-035H-circular-dependency-detection.md`
- **Jira:** [TASK-035H](https://velalang.atlassian.net/browse/VELA-575)
- **Historia:** [VELA-575](https://velalang.atlassian.net/browse/VELA-575)
- **Epic:** [VELA-574](https://velalang.atlassian.net/browse/VELA-574)

**Frameworks de referencia:**
- **Spring Framework:** Detección de ciclos en tiempo de inicialización
- **Angular DI:** Error messages con sugerencias
- **NestJS DI:** forwardRef() para ciclos intencionales (no implementado en Vela aún)

**Próximos pasos (TASK-035I):**
- Integrar DI con testing framework
- Mocking y test containers
- @lazy injection para ciclos intencionales

---

**COMPLETADO:** 2025-12-01  
**AUTOR:** GitHub Copilot Agent  
**ESTADO:** ✅ Todos los criterios de aceptación cumplidos
