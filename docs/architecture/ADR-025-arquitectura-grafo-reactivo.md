# ADR-025: Arquitectura del Grafo de Dependencias Reactivo

## Estado
✅ Aceptado

## Fecha
2025-12-01

## Contexto

El sistema reactivo de Vela necesita un mecanismo eficiente para:

1. **Tracking automático de dependencias** entre signals, computed values y effects
2. **Propagación eficiente** de cambios solo a los nodos afectados
3. **Detección de ciclos** para evitar loops infinitos
4. **Batching de actualizaciones** para optimizar el rendimiento
5. **Limpieza automática** de suscripciones cuando los nodos son destruidos

### Requisitos Técnicos

- **Performance**: O(1) para agregar/remover dependencias, O(n) para propagación
- **Memory**: Limpieza automática de nodos no referenciados (weak references)
- **Concurrency**: Thread-safe para uso en entornos multi-hilo
- **Debugging**: Trazabilidad completa del grafo para debugging

## Decisión

Implementaremos un **Dependency Graph basado en Adjacency List** con las siguientes características:

### 1. Estructura del Grafo

```python
class ReactiveNode:
    """Nodo base del grafo reactivo."""
    id: str                          # ID único
    dependencies: Set[ReactiveNode]  # Nodos de los que depende
    dependents: Set[ReactiveNode]    # Nodos que dependen de este
    dirty: bool                      # Si necesita recalcularse
    value: Any                       # Valor actual (cached)
    
class ReactiveGraph:
    """Grafo de dependencias reactivo."""
    nodes: Dict[str, ReactiveNode]   # Todos los nodos por ID
    active_computations: Stack       # Stack de computaciones activas (para tracking)
    batch_queue: Set[ReactiveNode]   # Cola de nodos a actualizar (batching)
    is_batching: bool                # Si estamos en modo batch
```

### 2. Algoritmo de Tracking Automático

**Auto-tracking durante evaluación:**

```python
def track_dependencies(computation_fn):
    """Decorator para auto-tracking de dependencias."""
    # 1. Push current node al stack
    reactive_graph.active_computations.push(current_node)
    
    # 2. Ejecutar computation (lee signals)
    result = computation_fn()
    
    # 3. Pop del stack
    reactive_graph.active_computations.pop()
    
    return result
```

**Registro de dependencia cuando se lee un signal:**

```python
def signal_get(signal):
    """Getter de signal con tracking automático."""
    # Si hay computación activa, registrar dependencia
    if reactive_graph.active_computations.not_empty():
        dependent = reactive_graph.active_computations.peek()
        signal.add_dependent(dependent)
        dependent.add_dependency(signal)
    
    return signal.value
```

### 3. Algoritmo de Propagación (Push-based)

**Propagación eficiente con topological sort:**

```python
def propagate_changes(changed_node):
    """Propaga cambios desde un nodo modificado."""
    # 1. Marcar nodo como dirty
    changed_node.dirty = True
    
    # 2. BFS desde el nodo cambiado
    queue = [changed_node]
    visited = set()
    
    while queue:
        node = queue.pop(0)
        
        if node in visited:
            continue
        visited.add(node)
        
        # 3. Marcar dependientes como dirty
        for dependent in node.dependents:
            dependent.dirty = True
            queue.append(dependent)
    
    # 4. Recalcular en orden topológico
    sorted_nodes = topological_sort(visited)
    
    for node in sorted_nodes:
        if node.dirty and node.should_recompute():
            node.recompute()
            node.dirty = False
```

### 4. Detección de Ciclos

```python
def detect_cycle(node, path=None):
    """Detecta ciclos en el grafo."""
    if path is None:
        path = []
    
    if node in path:
        cycle = path[path.index(node):] + [node]
        raise CyclicDependencyError(f"Ciclo detectado: {' -> '.join(n.id for n in cycle)}")
    
    path.append(node)
    
    for dependency in node.dependencies:
        detect_cycle(dependency, path.copy())
```

### 5. Batching de Actualizaciones

```python
class ReactiveGraph:
    def batch(self, fn):
        """Ejecuta función en modo batch."""
        self.is_batching = True
        self.batch_queue.clear()
        
        try:
            fn()  # Ejecutar operaciones (acumulan en batch_queue)
        finally:
            self.is_batching = False
            self.flush_batch()  # Propagar todos los cambios acumulados
    
    def flush_batch(self):
        """Propaga todos los cambios acumulados."""
        # Ordenar por nivel topológico
        sorted_nodes = topological_sort(self.batch_queue)
        
        for node in sorted_nodes:
            if node.dirty:
                node.recompute()
                node.dirty = False
```

### 6. Garbage Collection

```python
class ReactiveNode:
    def dispose(self):
        """Limpia el nodo del grafo."""
        # Remover de dependencias
        for dependency in self.dependencies:
            dependency.dependents.discard(self)
        
        # Remover de dependientes
        for dependent in self.dependents:
            dependent.dependencies.discard(self)
        
        # Remover del grafo
        reactive_graph.nodes.pop(self.id, None)
```

## Consecuencias

### Positivas

✅ **Performance Óptima**
- O(1) para agregar/remover dependencias
- O(n) para propagación (solo nodos afectados)
- Batching reduce actualizaciones redundantes

✅ **Auto-tracking Transparente**
- No requiere declaración manual de dependencias
- Funciona automáticamente al leer signals dentro de computed/effect

✅ **Detección de Ciclos**
- Previene loops infinitos en tiempo de ejecución
- Mensajes de error descriptivos con el path del ciclo

✅ **Memory Safety**
- Garbage collection automático de nodos no referenciados
- No memory leaks

✅ **Debugging Friendly**
- Grafo trazable en tiempo real
- Visualización de dependencias
- Historial de propagación

### Negativas

⚠️ **Overhead de Tracking**
- Stack adicional para tracking de computaciones activas
- Pequeño overhead en cada lectura de signal

⚠️ **Complejidad de Implementación**
- Algoritmos no triviales (topological sort, cycle detection)
- Requiere manejo cuidadoso de edge cases

⚠️ **Memory Overhead**
- Cada nodo mantiene sets de dependencies/dependents
- Grafo completo en memoria

## Alternativas Consideradas

### 1. **Pull-based Reactivity (Lazy Evaluation)**

```python
# Computed values se recalculan solo cuando se leen
computed_value.get()  # Trigger recalculation si dirty
```

**Rechazada porque:**
- ❌ No permite effects (side effects automáticos)
- ❌ Performance impredecible (recalcula en get)
- ❌ No permite batching eficiente

### 2. **Observables (RxJS-style)**

```python
# Streams de eventos con operadores
signal.pipe(
    map(x => x * 2),
    filter(x => x > 10),
    subscribe(x => print(x))
)
```

**Rechazada porque:**
- ❌ API más compleja para casos simples
- ❌ No auto-tracking (requiere pipes manuales)
- ❌ Overhead de streams para casos simples

### 3. **Dirty Checking (Angular 1.x style)**

```python
# Revisar todos los bindings periódicamente
def digest_cycle():
    for binding in all_bindings:
        old_value = binding.last_value
        new_value = binding.evaluate()
        if old_value != new_value:
            binding.update()
```

**Rechazada porque:**
- ❌ Performance O(n) constante (revisa todo)
- ❌ No escala con muchos bindings
- ❌ Requiere digest cycles manuales

### 4. **Immutable Data + Structural Sharing (Redux/Immer)**

```python
# Crear nuevo estado completo en cada cambio
new_state = {**old_state, count: old_state.count + 1}
```

**Rechazada porque:**
- ❌ No granularidad fina (actualiza todo el árbol)
- ❌ Requiere comparaciones profundas
- ❌ No propagación automática

## Implementación

### Archivos Generados

```
src/reactive/
├── __init__.py          # Exports públicos
├── graph.py             # ReactiveGraph, ReactiveNode
├── tracking.py          # Auto-tracking context
└── types.py             # Type definitions

tests/unit/reactive/
├── test_graph.py        # Tests del grafo
├── test_tracking.py     # Tests de auto-tracking
└── test_cycles.py       # Tests de detección de ciclos

docs/features/US-06/
└── TASK-025.md          # Documentación completa
```

### Métricas de Testing

- **Coverage**: >= 90%
- **Tests**: 30+ test cases
- **Edge cases**: Ciclos, memory leaks, threading, batching

## Referencias

- **Jira**: [US-06](https://velalang.atlassian.net/browse/US-06)
- **Inspired by**:
  - Vue 3 Reactivity System (Composition API)
  - SolidJS Signals
  - Svelte 5 Runes
  - MobX Observables

## Comparación con Otros Sistemas

| Sistema | Tracking | Propagación | Batching | Ciclos | Score |
|---------|----------|-------------|----------|--------|-------|
| **Vela** | Auto | Push | ✅ | ✅ | 10/10 |
| Vue 3 | Auto | Push | ✅ | ✅ | 10/10 |
| SolidJS | Auto | Push | ✅ | ⚠️ | 9/10 |
| React | Manual | Pull | ❌ | ❌ | 6/10 |
| Angular | Zone.js | Push | ⚠️ | ❌ | 7/10 |
| Svelte 5 | Auto | Push | ✅ | ✅ | 10/10 |

**Vela adopta las mejores prácticas de Vue 3 y SolidJS, con mejoras en debugging y cycle detection.**

---

**Última actualización:** 2025-12-01  
**Estado:** Implementación en progreso  
**Sprint:** 11 (Sistema Reactivo)
