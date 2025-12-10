# ADR-104: Dependency Resolution Algorithm

## Estado
🔄 Propuesto

## Fecha
2025-12-10

## Contexto
Vela necesita un sistema robusto de resolución de dependencias que pueda manejar versiones, conflictos y dependencias transitivas. El comando `vela install` (TASK-103) instala paquetes individuales, pero necesitamos un algoritmo que:

1. **Resuelva versiones compatibles** entre dependencias directas e indirectas
2. **Detecte y resuelva conflictos** de versiones
3. **Optimice la selección** de versiones para minimizar el grafo de dependencias
4. **Sea determinístico** y reproducible

## Decisión
Implementaremos un algoritmo de resolución de dependencias basado en **Satisfiability Solving** con backtracking, inspirado en algoritmos modernos como el de Cargo (Rust) y npm.

### Algoritmo Principal: Conflict-Driven Resolution

```
1. Construir grafo de dependencias inicial
2. Resolver restricciones de versiones
3. Detectar conflictos usando SAT solver
4. Backtracking para encontrar solución válida
5. Seleccionar versiones óptimas
```

### Estrategias de Resolución

#### 1. **Version Selection Strategy**
- **SemVer-aware**: Respeta rangos semánticos (^1.2.3, ~1.2.3, >=1.0.0)
- **Latest compatible**: Prefiere versiones más recientes dentro de rangos
- **Minimal graph**: Minimiza el número total de dependencias

#### 2. **Conflict Resolution**
- **Backtracking**: Retrocede en decisiones cuando encuentra conflictos
- **Version relaxation**: Amplía rangos cuando es posible
- **Alternative selection**: Prueba diferentes combinaciones de versiones

#### 3. **Performance Optimizations**
- **Caching**: Cache de resoluciones previas
- **Parallel resolution**: Resolución concurrente de subgrafos
- **Incremental updates**: Actualización mínima en cambios pequeños

## Consecuencias

### Positivas
- **Resolución robusta**: Maneja casos complejos de dependencias
- **Determinística**: Resultados predecibles y reproducibles
- **Optimizada**: Selección eficiente de versiones
- **Escalable**: Maneja grandes grafos de dependencias

### Negativas
- **Complejidad**: Algoritmo sofisticado aumenta complejidad
- **Performance**: Resolución puede ser costosa para grafos grandes
- **Debugging**: Conflictos pueden ser difíciles de diagnosticar

## Alternativas Consideradas

### 1. **Simple Topological Sort**
**Descripción**: Resolución básica sin manejo de conflictos.
**Rechazada porque**: No maneja versiones ni conflictos.

### 2. **npm-style Resolution**
**Descripción**: Algoritmo de npm con nested dependencies.
**Rechazada porque**: Crea dependency hell y no es determinístico.

### 3. **Manual Resolution Only**
**Descripción**: Usuario resuelve conflictos manualmente.
**Rechazada porque**: Mala experiencia de desarrollador.

## Implementación

### Arquitectura del Sistema

```
src/package/
├── resolver/
│   ├── mod.rs              # Módulo principal
│   ├── algorithm.rs        # Algoritmo core de resolución
│   ├── constraints.rs      # Manejo de restricciones de versión
│   ├── graph.rs            # Grafo de dependencias
│   └── solver.rs           # SAT solver para conflictos
├── registry/
│   ├── client.rs           # Cliente del registry
│   └── cache.rs            # Cache de metadatos
└── lockfile.rs             # Manejo de vela.lock
```

### Algoritmo de Resolución (Pseudocódigo)

```rust
pub fn resolve_dependencies(manifest: &Manifest) -> Result<Resolution, Error> {
    let mut resolver = DependencyResolver::new();

    // 1. Construir grafo inicial
    let mut graph = build_dependency_graph(manifest)?;

    // 2. Resolver restricciones
    apply_version_constraints(&mut graph)?;

    // 3. Resolver conflictos con backtracking
    let solution = resolve_conflicts_with_backtracking(&graph)?;

    // 4. Optimizar selección
    let optimized = optimize_version_selection(solution)?;

    Ok(optimized)
}

fn resolve_conflicts_with_backtracking(graph: &DependencyGraph)
    -> Result<VersionSolution, Error>
{
    let mut solver = SATSolver::new();

    // Convertir restricciones a cláusulas SAT
    for constraint in graph.constraints() {
        solver.add_clause(constraint.to_sat_clause());
    }

    // Resolver con backtracking
    match solver.solve() {
        Some(solution) => Ok(solution),
        None => Err(Error::UnsatisfiableConstraints)
    }
}
```

### Manejo de Versiones

#### SemVer Constraints
```rust
pub enum VersionConstraint {
    Exact(Version),           // 1.2.3
    Caret(Version),           // ^1.2.3 (compatible con 1.x.x)
    Tilde(Version),           // ~1.2.3 (compatible con 1.2.x)
    GreaterThan(Version),     // >1.2.3
    GreaterEqual(Version),    // >=1.2.3
    LessThan(Version),        // <1.2.3
    LessEqual(Version),       // <=1.2.3
    Range(Version, Version),  // 1.0.0 - 2.0.0
}
```

#### Conflict Resolution Strategies
```rust
pub enum ConflictStrategy {
    Backtrack,           // Retroceder y probar alternativas
    RelaxConstraints,    // Ampliar rangos de versión
    UpgradeAll,          // Actualizar todas las dependencias
    DowngradeConflicting // Bajar versiones conflictivas
}
```

## Referencias
- Jira: [VELA-104](https://velalang.atlassian.net/browse/VELA-104)
- Documentación: `docs/architecture/ADR-104-dependency-resolution.md`
- Código: `src/package/resolver/`
- Tests: `tests/unit/test_dependency_resolution.rs`

## Implementación
Ver código en: `src/package/resolver/`
Tests en: `tests/unit/test_dependency_resolution.rs`