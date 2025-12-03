# ADR-201: Arquitectura del Sistema de Tipos Vela en Rust

## Estado
✅ Aceptado

## Fecha
2025-12-03

## Contexto
EPIC-RUST-03 requiere migrar el sistema de tipos de Python a Rust. El sistema de tipos actual en Python es dinámico con algunas anotaciones opcionales. Necesitamos diseñar un sistema de tipos estático robusto para Rust que mantenga la flexibilidad de Vela mientras aproveche las fortalezas de tipos de Rust.

## Decisión
Implementaremos un sistema de tipos híbrido que combine:

1. **Sistema de tipos estático** con inferencia (Hindley-Milner)
2. **Types runtime** para metaprogramming
3. **Generic types** con constraints
4. **Union types** e **intersection types**
5. **Type aliases** y **opaque types**

## Consecuencias

### Positivas
- **Type Safety**: Prevención de errores en tiempo de compilación
- **Performance**: Optimizaciones basadas en tipos conocidos
- **Developer Experience**: Mejor IDE support y autocompletion
- **Interoperabilidad**: Fácil integración con crates existentes

### Negativas
- **Complejidad**: Sistema de tipos más complejo que el dinámico de Python
- **Learning Curve**: Desarrolladores necesitan entender tipos estáticos
- **Migration Effort**: Requiere cambios significativos en el código existente

## Arquitectura del Crate `vela-types`

```
vela-types/
├── src/
│   ├── lib.rs                 # Exports principales
│   ├── types.rs               # Definiciones de tipos base
│   ├── checker.rs             # Type checker principal
│   ├── inference.rs           # Algoritmo de inferencia
│   ├── constraints.rs         # Sistema de constraints
│   ├── unification.rs         # Unificación de tipos
│   ├── substitution.rs        # Sustitución de variables
│   ├── error.rs               # Errores de tipos
│   └── prelude.rs             # Re-exports comunes
├── benches/
│   └── type_checking.rs       # Benchmarks de performance
└── tests/
    ├── unit/
    │   ├── test_types.rs
    │   ├── test_checker.rs
    │   └── test_inference.rs
    └── integration/
        └── test_type_system.rs
```

### Componentes Principales

#### 1. Type System Core
```rust
pub enum Type {
    // Tipos primitivos
    Unit,           // ()
    Bool,           // bool
    Int,            // i64
    Float,          // f64
    String,         // String
    Char,           // char

    // Tipos compuestos
    Array(Box<Type>),           // [T]
    Tuple(Vec<Type>),           // (T1, T2, ...)
    Function(Vec<Type>, Box<Type>), // (T1, T2) -> T3

    // Tipos avanzados
    Generic(String, Vec<Type>), // Generic<T>
    Union(Vec<Type>),           // T1 | T2
    Intersection(Vec<Type>),    // T1 & T2

    // Tipos especiales
    Variable(String),           // Variables de tipo 'T
    Unknown,                    // Tipo no inferido aún
}
```

#### 2. Type Checker
```rust
pub struct TypeChecker {
    context: TypeContext,
    constraints: Vec<Constraint>,
}

impl TypeChecker {
    pub fn check(&mut self, expr: &Expression) -> Result<Type, TypeError> {
        // Implementación del algoritmo de type checking
    }

    pub fn infer(&mut self, expr: &Expression) -> Result<Type, TypeError> {
        // Inferencia de tipos automática
    }
}
```

#### 3. Type Inference Engine
```rust
pub struct TypeInference {
    variables: HashMap<String, Type>,
    constraints: Vec<Constraint>,
}

impl TypeInference {
    pub fn infer(&mut self, expr: &Expression) -> Result<Type, InferenceError> {
        // Algoritmo Hindley-Milner
    }

    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<Substitution, UnificationError> {
        // Unificación de tipos
    }
}
```

## Alternativas Consideradas

### 1. Sistema de Tipos Dinámico (como Python)
**Rechazada porque:**
- Pierde las ventajas de type safety que ofrece Rust
- Dificulta la optimización del compilador
- No aprovecha las fortalezas del sistema de tipos de Rust

### 2. Type System Minimal (solo tipos básicos)
**Rechazada porque:**
- No soporta las características avanzadas que necesita Vela
- Limitaría la expresividad del lenguaje
- No justifica la complejidad de migración a Rust

### 3. Reutilizar un Type System existente (como en TypeScript compiler)
**Rechazada porque:**
- No se adapta a las necesidades específicas de Vela
- Aumentaría dependencias externas
- Dificultaría el mantenimiento y evolución

## Implementación

### Fase 1: Core Types (TASK-RUST-201)
- [x] Definir enum Type con variantes básicas
- [x] Implementar TypeContext para scope management
- [x] Crear estructura básica del crate
- [x] Tests unitarios básicos

### Fase 2: Type Checker (TASK-RUST-202)
- [ ] Implementar algoritmo de type checking
- [ ] Soporte para expresiones básicas
- [ ] Error reporting detallado
- [ ] Integration con AST del compiler

### Fase 3: Type Inference (TASK-RUST-203)
- [ ] Algoritmo Hindley-Milner
- [ ] Constraint solving
- [ ] Unificación de tipos
- [ ] Generic type instantiation

### Fase 4: Testing & Optimization (TASK-RUST-204, 205)
- [ ] Suite completa de tests
- [ ] Benchmarks de performance
- [ ] Optimizaciones del algoritmo
- [ ] Memory profiling

## Referencias
- Jira: [EPIC-RUST-03](https://velalang.atlassian.net/browse/EPIC-RUST-03)
- Documentación: [Type Systems: Design and Implementation](https://www.cs.cornell.edu/~kozen/papers/types.pdf)
- Código: `src/types.rs` (implementación inicial)

## Implementación
Ver código en: `crates/vela-types/src/lib.rs`