# Changelog

Todos los cambios notables del proyecto Vela serán documentados en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/),
y este proyecto adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [Unreleased]

### En Desarrollo
- Sprint 6 (Dependency Injection) pendiente

---

## [0.4.0] - Sprint 5 - 2025-01-15

### 🎯 Resumen del Sprint
- **Epic completada:** EPIC-RUST-05 (StdLib Migration)
- **Módulos implementados:** 5 (primitives, collections, option_result, iterators, strings)
- **Tests agregados:** 168 tests unitarios (100% passing)
- **Documentación:** ADR-401 + README.md completo

### ✨ Added - Standard Library Implementation

#### [EPIC-RUST-05] StdLib Migration
Como desarrollador del runtime, necesito una standard library completa con tipos seguros y APIs funcionales.

**Módulos implementados:**

- **[Primitives Module]** (54 tests) ✅
  - **VelaNumber**: Union type Int/Float con 40+ métodos
    - Operaciones: add, sub, mul, div, pow, sqrt, abs, round, floor, ceil
    - Comparaciones: min, max, clamp
    - Operator overloading: Add, Sub, Mul, Div, Neg
  - **VelaString**: String inmutable con 20+ métodos
    - Transformaciones: to_uppercase, to_lowercase, trim, replace
    - Búsqueda: contains, starts_with, ends_with, index_of
    - Manipulación: substring, concat, repeat, join
  - **VelaBool**: Booleano con lógica completa
    - Operaciones: and, or, not, xor, nand, nor, xnor, implies

- **[Collections Module]** (26 tests) ✅
  - **VelaList<T>**: Array dinámico con API funcional
    - Construcción: new, from, with_capacity
    - Mutación: push, pop, insert, remove
    - Transformación: map, filter, reduce, reverse, sort
    - Búsqueda: find, any, all, contains
    - Slicing: take, skip, concat
    - Inmutabilidad: operaciones retornan nuevas listas
  - **VelaMap<K,V>**: Hash map con operaciones inmutables
    - CRUD: insert, get, remove, contains_key
    - Transformación: map, filter, for_each
    - Iteradores: keys, values, entries
    - Trait bounds: K: Eq + Hash
  - **VelaSet<T>**: Set con operaciones matemáticas
    - CRUD: insert, remove, contains
    - Operaciones de sets: union, intersection, difference, symmetric_difference
    - Relaciones: is_subset, is_superset, is_disjoint
    - Funcionales: map, filter, any, all

- **[Option/Result Module]** (25 tests) ✅
  - **VelaOption<T>**: Manejo de valores opcionales (NO null)
    - Constructores: Some(T), None
    - Unwrap: unwrap, unwrap_or, unwrap_or_else
    - Transformación: map, and_then, or_else, filter, zip
    - Conversión: from/to Rust Option
  - **VelaResult<T,E>**: Manejo de errores (NO exceptions)
    - Constructores: Ok(T), Err(E)
    - Unwrap: unwrap, unwrap_or, unwrap_err
    - Transformación: map, map_err, and_then, or_else
    - Combinadores: and, or
    - Conversión: from/to Rust Result

- **[Iterators Module]** (22 tests) ✅
  - **VelaIterator**: Iteración lazy con evaluación diferida
    - Constructores: from_vec, from_iter
    - Transformación: map, filter, flat_map
    - Slicing: take, skip, take_while, skip_while
    - Agregación: reduce, fold, sum, product, count
    - Búsqueda: find, position, any, all, min, max
    - Combinación: chain, zip, enumerate
    - Terminales: collect, partition, for_each, inspect

- **[Strings Module]** (41 tests) ✅
  - **Interpolation**: Template strings con sintaxis ${variable}
    - Funciones: interpolate(), interpolate_with_fallback()
    - Extracción: extract_variables()
    - Error handling: VariableNotFound, InvalidSyntax
  - **Formatting**: Format strings con {} y {name}
    - Posicionales: format_string("{} v{}", args)
    - Named: format_named("{name} v{version}", map)
    - Escapado: {{ }} para braces literales
    - Error handling: MismatchedArguments, InvalidSpecifier
  - **Regex**: Pattern matching simplificado
    - Patrones: \\d+ (digits), \\w+ (words), literals
    - Operaciones: is_match, find, find_all, replace, replace_all, split
    - Error handling: InvalidPattern, CompileError
  - **Splitting**: Utilidades avanzadas de división
    - split_advanced, split_whitespace, split_by_any
    - split_n, rsplit, split_inclusive, split_lines, chunk

**Documentación:** 
- `docs/architecture/ADR-401-vela-stdlib-architecture.md` (520+ líneas)
- `stdlib/README.md` (326 líneas con ejemplos completos)

### 📚 Documentation
- **ADR-401**: Arquitectura completa con especificaciones de 5 módulos
- **stdlib/README.md**: Guía completa con:
  - Overview de módulos
  - Ejemplos de uso para cada tipo
  - Design principles (immutability, type safety, functional composition)
  - Test coverage metrics
  - Integration guide
  - API reference
- **Cargo doc**: Documentación HTML generada automáticamente

### 🔧 Technical Implementation
- **Language:** Rust 1.75.0, edition 2021
- **Dependencies:** std (HashMap, HashSet, Vec), anyhow, thiserror
- **Architecture:** Thin wrappers over Rust stdlib (zero-cost abstractions)
- **Type Safety:** Option<T> y Result<T,E> en lugar de null/exceptions
- **Immutability:** Estructuras inmutables por defecto
- **Functional:** APIs estilo Rust/TypeScript/Python con map/filter/reduce

### ✅ Quality Metrics
- **Tests unitarios:** 168/168 pasando ✅ (100% success rate)
- **Cobertura por módulo:**
  - Primitives: 54 tests
  - Collections: 26 tests
  - Option/Result: 25 tests
  - Iterators: 22 tests
  - Strings: 41 tests
- **Warnings:** 3 unused_mut (no críticos)
- **Build time:** ~16s (release)
- **Test time:** ~2.9s

### 🎯 Performance Characteristics
- **Memory:** Zero-cost abstractions (thin wrappers)
- **Allocation:** Lazy evaluation en iteradores (no allocations intermedias)
- **Collections:** O(1) insert/get para Map/Set, O(n) para List operations
- **Strings:** O(n) para interpolation/format

### 📊 Architecture Highlights
- **Modularity:** 5 módulos independientes y reutilizables
- **Type Safety:** Sistema de tipos completo sin null/undefined
- **Functional Programming:** Composición de operaciones con iteradores lazy
- **Integration Ready:** Diseñado para VM, Compiler y Runtime de Vela

### 🔗 Integration Points
- **Vela VM**: Runtime execution con tipos nativos
- **Vela Compiler**: Type checking y code generation
- **Vela Runtime**: Async/actors/channels (próximo Sprint)

---

## [0.3.0] - Sprint 2 - 2025-12-01

### 🎯 Resumen del Sprint
- **Historia completada:** VELA-562 (Tooling Design - Phase 0)
- **Subtasks completadas:** 4
- **Documentación generada:** 2,250+ líneas
- **Diseños completados:** 4 arquitecturas de tooling

### ✨ Added - Diseños de Herramientas de Desarrollo

#### [VELA-562] Tooling Design - Phase 0
Como equipo de desarrollo, necesitamos los diseños de las herramientas de desarrollo para el ecosistema Vela.

**Subtasks completadas:**

- **[TASK-000J]** Diseñar arquitectura del CLI (600+ líneas)
  - 14 comandos documentados: new, init, build, run, watch, test, bench, fmt, lint, doc, add, remove, update, publish
  - Sistema de configuración jerárquico (vela.yaml: global → workspace → project)
  - Sistema de plugins extensible (~/.vela/plugins/)
  - Performance targets: < 50ms startup (cold), < 10ms (warm)
  - Framework seleccionado: **Clap (Rust)** (vs Commander.js, Click, Cobra)
  - Parallel compilation: 100% core utilization
  - UX: mensajes de error específicos, progress bars, interactive mode

- **[TASK-000K]** Diseñar arquitectura del Package Manager (650+ líneas)
  - Registry architecture: **vela-registry.io** (REST API v1)
  - Dependency resolution: **PubGrub algorithm** (Dart-style)
  - Security: SHA-256 checksums + Ed25519 digital signatures
  - Hosting: AWS S3 + CloudFront CDN + PostgreSQL + Redis
  - Lockfile format: vela.lock (reproducibilidad garantizada)
  - Scalability: 50K packages, 100M downloads/mes
  - SemVer compliance: ^1.2.3 ranges
  - Audit: vela audit para vulnerability scanning

- **[TASK-000L]** Diseñar arquitectura del LSP (550+ líneas)
  - LSP features priorizado: P0 (syntax, diagnostics, completion, go-to-def, references, hover)
  - Incremental compilation: **Salsa framework** (query-based caching)
  - Parser: **Tree-sitter** (incremental, error recovery)
  - Integration: shared codebase con compiler frontend
  - Performance target: < 100ms latency para completions
  - Tech stack: **Rust + tower-lsp + Tree-sitter + Salsa**
  - Error recovery: parser tolerante a errores

- **[TASK-000M]** Diseñar arquitectura de DevTools (450+ líneas)
  - **UI Inspector**: component tree, properties editor (editable en vivo), layout overlay
  - **Signal Graph Visualizer**: dependency graph (D3.js), recomputation timeline, dirty signals highlighting
  - **Performance Profiler**: CPU profiling (flame graphs), memory snapshots, event timeline
  - Protocol: **JSON-RPC over WebSocket** (ws://localhost:9229)
  - UI: Web-based (React + D3.js + Speedscope)
  - Deployment: Browser Extension (Chrome/Firefox) o Electron app
  - Performance overhead: < 5%

**Documentación:** `docs/features/VELA-562/`

### 📚 Documentation
- 4 especificaciones técnicas completas (TASK-000J, K, L, M)
- Total: 2,250+ líneas de diseños arquitectónicos
- Comparaciones con npm/Cargo (Package Manager), rust-analyzer/TypeScript LSP, React/Vue/Flutter DevTools
- Technology stack seleccionado con justificación

### 🔧 Technical Decisions
- **CLI Framework:** Clap (Rust) - performance, robustez, completions
- **Dependency Resolution:** PubGrub algorithm - solución óptima, error messages claros
- **LSP Incremental:** Salsa framework - query-based caching (usado por rust-analyzer)
- **DevTools Protocol:** JSON-RPC - simple, estándar, bidireccional

### 🎯 Performance Targets Establecidos
- CLI startup: < 50ms (cold), < 10ms (warm)
- LSP completion: < 50ms latency
- LSP diagnostics: < 100ms after keystroke
- DevTools overhead: < 5%
- Package download: 10-50 MB/s (según región)

### 📊 Architecture Highlights
- Registry: AWS S3 + CloudFront (CDN global, 450+ edge locations)
- LSP: Shared codebase con compiler (no duplicación)
- DevTools: Chrome DevTools Protocol-style (familiar para developers)

---

## [0.2.0] - Sprint 1 - 2025-12-01

### 🎯 Resumen del Sprint
- **Historia completada:** VELA-561 (Core Language Specification)
- **Subtasks completadas:** 4
- **Documentación generada:** 2,550+ líneas
- **Especificaciones formales:** 4

### ✨ Added - Especificaciones Formales del Lenguaje

#### [VELA-561] Core Language Specification
Como equipo de desarrollo, necesitamos especificaciones formales completas antes de implementar el compiler.

**Subtasks completadas:**

- **[TASK-000F]** Especificación formal completa del lenguaje (700+ líneas)
  - Lexical structure: EBNF grammar completa (60+ keywords)
  - Type system formal: 7 reglas (inmutabilidad, Hindley-Milner inference, Option<T>, Result<T,E>)
  - Operational semantics: evaluation rules (⟨e, σ⟩ ⇓ v)
  - Expression evaluation: left-to-right order guaranteed
  - Statement execution: secuencial, determinista
  - Function call semantics: closures, async/await
  - Comparación: Rust, TypeScript, Dart

- **[TASK-000G]** Modelo de memoria formal (650+ líneas)
  - Object lifetime rules: 4 reglas formales
  - **ARC algorithm**: retain/release (código Rust completo)
  - **Cycle detection**: weak references + tracing GC (mark & sweep)
  - Thread safety: Send/Sync traits automáticos
  - Memory visibility: Acquire-Release semantics
  - Memory layout: 16 bytes overhead por objeto (header + refcount)
  - Performance: O(1) retain/release, O(n) cycle collection

- **[TASK-000H]** Modelo de concurrencia formal (650+ líneas)
  - **Actor model**: message passing, FIFO mailbox, at-most-once delivery
  - **Signal propagation**: topological sort, no glitches
  - Memory visibility guarantees: happened-before relationship
  - Race condition prevention: no shared mutable state
  - Deadlock prevention: no locks, async-only
  - Formal verification properties
  - Comparación: Erlang actors, Solid.js signals

- **[TASK-000I]** Contratos formales de stdlib (550+ líneas)
  - Collections: List<T>, Map<K,V> con preconditions/postconditions
  - Option<T> y Result<T,E>: operations completas
  - String operations: complejidades Big-O
  - Future<T>: async/await semantics
  - Tabla de complejidades: 50+ APIs con Big-O notation
  - Thread-safety: garantías por API
  - Platform-specific behavior documentado

**Documentación:** `docs/features/VELA-561/`

### 📚 Documentation
- 4 especificaciones formales completas (TASK-000F, G, H, I)
- Total: 2,550+ líneas de especificaciones
- 30+ reglas formales definidas
- 10+ algoritmos especificados (ARC, GC, Actor scheduling)
- 50+ APIs con contratos formales
- Referencias académicas incluidas

### 🔧 Technical Specifications
- **Type System:** Hindley-Milner con extensions (Option, Result, ADTs)
- **Memory Model:** ARC + Cycle Detection (weak refs + tracing GC)
- **Concurrency:** Actor model + Fine-grained reactivity (signals)
- **Stdlib:** 50+ APIs con complejidades garantizadas

### 📊 Formal Rules Defined
- Type system: 7 reglas formales
- Memory management: 4 lifetime rules
- Operational semantics: evaluation rules completas
- Thread safety: Send/Sync trait rules

---

## [0.1.0] - Sprint 0 - 2025-11-30

### 🎯 Resumen del Sprint
- **Historias completadas:** 1
- **Subtasks completadas:** 5
- **Tests agregados:** 25 tests unitarios
- **Documentación:** 6 documentos generados

### ✨ Added - Nuevas Features

#### [US-00A] Decisiones Arquitectónicas Críticas
Como líder técnico, necesito tomar decisiones arquitectónicas críticas antes de escribir código.

**Subtasks completadas:**
- **[TASK-000A]** Decidir lenguaje de implementación
  - ADR creado: `docs/architecture/ADR-1195-decidir-lenguaje.md`
  - Código: `src/decidir-lenguaje-de-implementacion.py`
  - Tests: `tests/unit/test_decidir-lenguaje-de-implementacion.py`

- **[TASK-000B]** Definir arquitectura del build system
  - ADR creado: `docs/architecture/ADR-1196-definir-arquitectura-build-system.md`
  - Código: `src/definir-arquitectura-del-build-system.py`
  - Tests: `tests/unit/test_definir-arquitectura-del-build-system.py`

- **[TASK-000C]** Elegir licencia open source
  - ADR creado: `docs/architecture/ADR-1197-elegir-licencia.md`
  - Código: `src/elegir-licencia-open-source.py`
  - Tests: `tests/unit/test_elegir-licencia-open-source.py`

- **[TASK-000D]** Seleccionar plataforma CI/CD
  - ADR creado: `docs/architecture/ADR-1198-seleccionar-plataforma-cicd.md`
  - Código: `src/seleccionar-plataforma-cicd.py`
  - Tests: `tests/unit/test_seleccionar-plataforma-cicd.py`

- **[TASK-000E]** Elegir plataforma de documentación
  - ADR creado: `docs/architecture/ADR-1199-elegir-plataforma-docs.md`
  - Código: `src/elegir-plataforma-de-documentacion.py`
  - Tests: `tests/unit/test_elegir-plataforma-de-documentacion.py`

**Documentación:** `docs/features/VELA-560/README.md`

### 📚 Documentation
- Creada guía de contribución: `.github/CONTRIBUTING.md`
- Creado template de Pull Request: `.github/PULL_REQUEST_TEMPLATE.md`
- Creados 5 ADRs para decisiones arquitectónicas
- Documentación de Historia: `docs/features/VELA-560/`

### 🔧 Technical Changes
- Inicializado repositorio Git
- Estructura de directorios establecida
- Sistema de automatización de desarrollo implementado
- Integración con Jira configurada

### ✅ Quality Metrics
- **Tests unitarios:** 25/25 pasando ✅
- **Cobertura de código:** ~95%
- **ADRs creados:** 5
- **Documentos generados:** 11

### 🎉 Milestone
- ✅ Sprint 0 completado y cerrado
- ✅ Primera Historia desarrollada con éxito
- ✅ Proceso de desarrollo automatizado establecido

---

## Template para Futuras Entradas

```markdown
## [X.Y.Z] - Sprint N - YYYY-MM-DD

### 🎯 Resumen del Sprint
- **Historias completadas:** X
- **Subtasks completadas:** XX
- **Tests agregados:** XX tests
- **Documentación:** XX documentos

### ✨ Added
- [US-XXX] Título de la Historia
  - [TASK-XXX] Descripción del cambio

### 🔧 Changed
- [TASK-XXX] Descripción del cambio

### 🐛 Fixed
- [TASK-XXX] Descripción del fix

### 📚 Documentation
- Documentación agregada/actualizada

### ⚠️ Breaking Changes
- Descripción de breaking changes (si los hay)
```

---

**Nota:** Este archivo se actualiza automáticamente al completar cada Sprint.

[Unreleased]: https://github.com/camilohaze/vela/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/camilohaze/vela/releases/tag/v0.3.0
[0.2.0]: https://github.com/camilohaze/vela/releases/tag/v0.2.0
[0.1.0]: https://github.com/camilohaze/vela/releases/tag/v0.1.0
