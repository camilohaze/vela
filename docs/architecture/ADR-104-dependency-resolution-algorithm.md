# ADR-104: Algoritmo de Resolución de Dependencias

## Estado
✅ Aceptado

## Fecha
2025-01-30

## Contexto
El package manager de Vela necesita un algoritmo robusto para resolver dependencias que pueda manejar:

- Restricciones de versiones SemVer complejas (^, ~, rangos)
- Conflictos entre dependencias transitivas
- Optimización de selección de versiones
- Integración con bindings de lenguajes extranjeros

El algoritmo debe ser eficiente para casos comunes pero capaz de manejar casos complejos.

## Decisión
Implementar un **algoritmo híbrido de resolución** que combine:

1. **SAT Solver con CDCL** (primera opción) - Para problemas bien definidos
2. **Backtracking con poda** (fallback) - Para casos complejos
3. **Optimización heurística** - Para selección inteligente de versiones

### Arquitectura Elegida

```
DependencyResolver (Hybrid)
├── SATSolver (CDCL Algorithm)
│   ├── Unit Propagation
│   ├── Conflict Analysis
│   └── Clause Learning
├── BacktrackingResolver
│   ├── Depth-First Search
│   ├── Constraint Propagation
│   └── Conflict Detection
└── VersionConstraint System
    ├── SemVer Parsing
    ├── Satisfaction Checking
    └── Range Operations
```

## Consecuencias

### Positivas
- **Eficiencia**: SAT solver maneja eficientemente la mayoría de casos
- **Robustez**: Backtracking resuelve casos complejos que SAT no puede
- **Flexibilidad**: Soporte completo para restricciones SemVer
- **Mantenibilidad**: Arquitectura modular y bien testeada
- **Extensibilidad**: Fácil agregar nuevos tipos de restricciones

### Negativas
- **Complejidad**: Implementación más compleja que algoritmos simples
- **Performance**: Peor caso O(2^n) para SAT, O(d^n) para backtracking
- **Memoria**: Almacenamiento de grafo de dependencias y cláusulas SAT

### Trade-offs
- **SAT vs Backtracking**: SAT es más eficiente para problemas típicos pero backtracking es más simple de implementar
- **Optimización**: Selección de versiones óptimas vs resolución pura
- **Memoria vs Velocidad**: Mantener grafo completo vs resolución lazy

## Alternativas Consideradas

### 1. Solo Backtracking (Rechazada)
**Pros**: Simple de implementar, fácil de entender
**Cons**: Performance pobre para grafos grandes, no escala bien
**Razón**: No maneja eficientemente casos reales de dependencias

### 2. Solo SAT Solver (Rechazada)
**Pros**: Muy eficiente para problemas bien formados
**Cons**: Difícil manejar restricciones complejas, puede fallar en casos edge
**Razón**: No robusto para todos los escenarios de dependencias reales

### 3. Algoritmo de Dependencias de npm (Rechazada)
**Pros**: Probado en producción, maneja casos reales
**Cons**: Complejo de adaptar a Vela, dependiente de Node.js
**Razón**: No se alinea con la arquitectura funcional pura de Vela

### 4. Resolución Lazy (Rechazada)
**Pros**: Menor uso de memoria, resolución bajo demanda
**Cons**: Más complejo de implementar, peor para detección de conflictos
**Razón**: No proporciona la visibilidad completa necesaria para optimización

## Implementación

### Fases del Algoritmo

1. **Construcción del Grafo**
   ```rust
   fn build_dependency_graph(manifest: &Manifest) -> DependencyGraph
   ```

2. **Aplicación de Restricciones**
   ```rust
   fn apply_constraints(graph: DependencyGraph) -> DependencyGraph
   ```

3. **Resolución de Conflictos**
   ```rust
   fn resolve_conflicts(graph: DependencyGraph) -> Result<Resolution, Error>
   ```

4. **Optimización**
   ```rust
   fn optimize_selection(resolution: Resolution) -> Resolution
   ```

### Manejo de Conflictos

#### Detección
- Violaciones de restricciones de versión
- Dependencias circulares
- Versiones mutuamente excluyentes

#### Resolución
- Backtracking para explorar alternativas
- Relajación de restricciones cuando posible
- Selección de versiones compatibles

## Referencias

### Técnicos
- [CDCL Algorithm](https://en.wikipedia.org/wiki/Conflict-Driven_Clause_Learning)
- [SemVer Specification](https://semver.org/)
- [Dependency Resolution in Package Managers](https://research.swtch.com/version-sat)

### Vela
- TASK-103: Foreign Language Bindings
- VELA-104: Dependency Resolution Algorithm
- Package Manager Architecture

## Implementación
Ver código en: `src/package/resolver/`

Los archivos principales son:
- `constraints.rs` - Sistema de restricciones SemVer
- `graph.rs` - Estructuras del grafo de dependencias
- `solver.rs` - SAT solver con CDCL
- `backtracking.rs` - Resolver por backtracking
- `algorithm.rs` - Algoritmo principal de resolución