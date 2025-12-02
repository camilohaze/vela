# ADR-035H: Circular Dependency Detection

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto

El sistema de Dependency Injection debe detectar dependencias circulares para evitar loops infinitos y stack overflows durante la resolución. Una dependencia circular ocurre cuando:

```
A → B → C → A  (ciclo directo)
A → B, B → A   (ciclo simple de 2 nodos)
A → A          (self-dependency)
```

### Problema

Sin detección de ciclos, el Injector entraría en recursión infinita:

```python
# A depende de B
class A:
    def __init__(self, b: B):
        self.b = b

# B depende de A → CICLO
class B:
    def __init__(self, a: A):
        self.a = a

injector.register(A)
injector.register(B)

# ❌ Stack overflow (sin detección)
injector.get(A)  # A → B → A → B → A → ...
```

### Requerimientos

1. **Detección en tiempo de resolución**: Detectar ciclos DURANTE `injector.get()`
2. **Detección en tiempo de registro** (opcional): Advertir ciclos ANTES de `injector.get()`
3. **Mensajes de error claros**: Mostrar cadena completa del ciclo
4. **Performance**: O(1) detección por nodo (usar stack)
5. **Soporte para todos los scopes**: Singleton, Transient, Scoped

## Decisión

### 1. Algoritmo de Detección: DFS con Resolution Stack

**Implementación actual en `ResolutionContext`:**

```python
@dataclass
class ResolutionContext:
    resolution_stack: List[Type] = field(default_factory=list)
    
    def push_resolution(self, token: Type) -> None:
        if token in self.resolution_stack:
            # CICLO DETECTADO
            cycle = self.resolution_stack + [token]
            raise CircularDependencyError(cycle)
        
        self.resolution_stack.append(token)
    
    def pop_resolution(self) -> None:
        self.resolution_stack.pop()
```

**Flujo de detección:**

```
injector.get(A)
  ├─ push(A) → stack: [A]
  ├─ resolve A → requiere B
  │   ├─ push(B) → stack: [A, B]
  │   ├─ resolve B → requiere A
  │   │   ├─ push(A) → ERROR: A ya está en stack
  │   │   └─ raise CircularDependencyError([A, B, A])
  │   └─ (no llega aquí)
  └─ (no llega aquí)
```

### 2. Mejoras Implementadas

#### A. Mensajes de Error Mejorados

```python
class CircularDependencyError(InjectionError):
    def __init__(self, dependency_chain: List[Type]):
        self.dependency_chain = dependency_chain
        chain_str = " -> ".join(cls.__name__ for cls in dependency_chain)
        
        # Mensaje claro
        super().__init__(
            f"Circular dependency detected: {chain_str}\n"
            f"\nSuggestions:\n"
            f"  1. Use @lazy() injection for one dependency\n"
            f"  2. Introduce intermediate service\n"
            f"  3. Use event-driven architecture"
        )
```

**Output:**
```
CircularDependencyError: Circular dependency detected: A -> B -> C -> A

Suggestions:
  1. Use @lazy() injection for one dependency
  2. Introduce intermediate service
  3. Use event-driven architecture
```

#### B. Dependency Graph Analyzer (Nuevo)

**Archivo**: `src/runtime/di/graph_analyzer.py`

```python
class DependencyGraph:
    """
    Analizador de grafo de dependencias.
    
    Permite:
    - Construir grafo desde Injector
    - Detectar ciclos (algoritmo DFS)
    - Visualizar dependencias
    - Sugerir refactorings
    """
    
    def __init__(self, injector: Injector):
        self.graph = self._build_graph(injector)
    
    def find_cycles(self) -> List[List[Type]]:
        """
        Encuentra TODOS los ciclos en el grafo (no solo el primero).
        
        Usa algoritmo DFS con backtracking.
        
        Returns:
            Lista de ciclos detectados.
            Ejemplo: [[A, B, A], [C, D, E, C]]
        """
        ...
    
    def visualize(self) -> str:
        """
        Genera visualización ASCII del grafo.
        
        Returns:
            String con representación visual.
            
        Example:
            A
            ├─ B
            │  └─ C
            │     └─ A (CYCLE)
            └─ D
        """
        ...
    
    def suggest_fixes(self, cycle: List[Type]) -> List[str]:
        """
        Sugiere refactorings para romper ciclo.
        
        Suggestions:
        - Lazy injection (@lazy)
        - Intermediate service
        - Interface segregation
        - Event-driven
        """
        ...
```

#### C. Verificación Estática (Opcional)

```python
def verify_no_cycles_at_registration(injector: Injector) -> None:
    """
    Verifica ANTES de runtime que no hay ciclos.
    
    Uso:
        injector.register(A)
        injector.register(B)
        injector.register(C)
        
        verify_no_cycles_at_registration(injector)  # Lanza si hay ciclo
    """
    graph = DependencyGraph(injector)
    cycles = graph.find_cycles()
    
    if cycles:
        for cycle in cycles:
            logger.warning(f"Potential cycle: {cycle}")
        raise CircularDependencyError(cycles[0])
```

## Algoritmos Considerados

### Opción 1: DFS con Stack (ELEGIDO ✅)

**Ventajas:**
- ✅ O(1) detección por nodo (check in stack)
- ✅ Memory eficiente (solo stack actual)
- ✅ Detecta ciclo en primera ocurrencia
- ✅ Mensaje de error con path completo
- ✅ Simple de implementar

**Desventajas:**
- ⚠️ Solo detecta UN ciclo (el primero encontrado)

**Complejidad:**
- Tiempo: O(V + E) donde V = nodos, E = edges
- Espacio: O(V) (stack depth)

### Opción 2: Tarjan's Algorithm (RECHAZADO ❌)

**Ventajas:**
- ✅ Detecta TODOS los Strongly Connected Components (SCCs)
- ✅ O(V + E) complejidad

**Desventajas:**
- ❌ Más complejo de implementar
- ❌ Overkill para este caso (solo necesitamos primer ciclo)
- ❌ No da path directo del ciclo

### Opción 3: Kahn's Algorithm (Topological Sort) (RECHAZADO ❌)

**Ventajas:**
- ✅ Detecta existencia de ciclos (sort falla)

**Desventajas:**
- ❌ No da path del ciclo
- ❌ Requiere construir grafo completo
- ❌ O(V + E) preprocessing

## Consecuencias

### Positivas

1. **Prevención de stack overflow**: Detecta ciclos ANTES de recursión infinita
2. **Mensajes claros**: Developer sabe exactamente qué ciclo existe
3. **Performance mínima**: O(1) check por nodo
4. **Sugerencias de fix**: Ayuda a resolver el problema

### Negativas

1. **Solo detecta primer ciclo**: Si hay múltiples ciclos, solo muestra uno
2. **Runtime detection**: No detecta en tiempo de compilación (Python limitación)
3. **Forward references**: Require `typing.get_type_hints()` para type annotations

### Mitigaciones

1. **DependencyGraph analyzer**: Para encontrar TODOS los ciclos (análisis estático)
2. **IDE integration** (futuro): Lint rules para detectar ciclos
3. **Unit tests exhaustivos**: Cubrir todos los casos de ciclos

## Ejemplos de Uso

### Ejemplo 1: Ciclo Simple (A → B → A)

```python
# A depende de B
class A:
    def __init__(self, b: B):
        self.b = b

# B depende de A → CICLO
class B:
    def __init__(self, a: A):
        self.a = a

injector = Injector()
injector.register(A)
injector.register(B)

try:
    injector.get(A)
except CircularDependencyError as e:
    print(e)
    # Output: Circular dependency detected: A -> B -> A
    #
    # Suggestions:
    #   1. Use @lazy() injection for one dependency
    #   2. Introduce intermediate service
    #   3. Use event-driven architecture
```

### Ejemplo 2: Ciclo Largo (A → B → C → D → A)

```python
class A:
    def __init__(self, b: B): ...

class B:
    def __init__(self, c: C): ...

class C:
    def __init__(self, d: D): ...

class D:
    def __init__(self, a: A): ...  # CICLO

try:
    injector.get(A)
except CircularDependencyError as e:
    print(e.dependency_chain)  # [A, B, C, D, A]
```

### Ejemplo 3: Self-Dependency (A → A)

```python
class A:
    def __init__(self, a: A):  # Self-dependency
        self.a = a

try:
    injector.get(A)
except CircularDependencyError as e:
    print(e)  # Circular dependency detected: A -> A
```

### Ejemplo 4: Análisis Estático con DependencyGraph

```python
from src.runtime.di.graph_analyzer import DependencyGraph

# Construir grafo
graph = DependencyGraph(injector)

# Encontrar TODOS los ciclos
cycles = graph.find_cycles()
for cycle in cycles:
    print(f"Cycle found: {cycle}")

# Visualizar grafo
print(graph.visualize())
# Output:
#   A
#   ├─ B
#   │  └─ C
#   │     └─ A (CYCLE!)
#   └─ D

# Sugerencias de fix
suggestions = graph.suggest_fixes(cycles[0])
for suggestion in suggestions:
    print(f"  - {suggestion}")
```

## Referencias

### Algoritmos
- **DFS (Depth-First Search)**: Graph traversal para detección de ciclos
- **Tarjan's Algorithm**: Strongly Connected Components
- **Kahn's Algorithm**: Topological sort
- **Backtracking**: Encontrar todos los ciclos

### Frameworks con Detección de Ciclos
- **Spring Framework (Java)**: BeanCurrentlyInCreationException
- **Angular (TypeScript)**: Cyclic dependency error
- **NestJS (TypeScript)**: Circular dependency detected
- **InversifyJS (TypeScript)**: Circular dependency
- **Autofac (.NET)**: DependencyResolutionException

### Python Type Checking
- **typing.get_type_hints()**: Resolver forward references
- **__annotations__**: Type hints en runtime
- **inspect.signature()**: Inspeccionar constructor params

## Implementación

### Archivos Modificados/Creados

1. **src/runtime/di/injector.py** (existente)
   - ResolutionContext.push_resolution() (ya implementado ✅)
   - CircularDependencyError (mejorar mensaje)

2. **src/runtime/di/graph_analyzer.py** (nuevo)
   - DependencyGraph class
   - find_cycles()
   - visualize()
   - suggest_fixes()

3. **tests/unit/di/test_circular_deps.py** (nuevo)
   - test_simple_cycle()
   - test_long_cycle()
   - test_self_dependency()
   - test_multiple_cycles()
   - test_graph_analyzer()

4. **docs/features/VELA-575/TASK-035H.md**
   - Documentación completa
   - Ejemplos de ciclos
   - Cómo romper ciclos

## Métricas Esperadas

- **Detection time**: O(1) per node
- **Memory overhead**: O(depth) stack
- **False positives**: 0 (DFS es exacto)
- **Test coverage**: >= 95%

---

**Estado**: ✅ Aceptado  
**Implementado en**: TASK-035H  
**Fecha**: 2025-12-02  
**Autor**: Vela Development Team
