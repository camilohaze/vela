# ADR-013: Type System Representation

## Estado
✅ Aceptado

## Fecha
2025-12-08

## Contexto
Vela necesita un sistema de tipos robusto y completo para validar código en tiempo de compilación. Sin una representación interna adecuada de tipos, no podemos implementar type checking, inference, o generics. El type system debe ser:

- **Type-safe**: Prevenir errores en runtime
- **Extensible**: Soporte para tipos primitivos, compuestos, generics
- **Eficiente**: Representación optimizada para unificación y checking
- **Completo**: Cubrir todos los tipos de Vela (primitivos, structs, enums, functions, etc.)

## Decisión
Implementar un sistema de representación de tipos basado en:

1. **Type Expressions**: Representación algebraica de tipos
2. **Type Variables**: Para type inference (Hindley-Milner)
3. **Type Constructors**: Para tipos compuestos (List<T>, Option<T>)
4. **Unification Algorithm**: Para resolver constraints de tipos
5. **Type Environment**: Para tracking de variables y sus tipos

### Arquitectura Elegida

```
Type System Architecture
├── types/
│   ├── primitives.rs      # Number, String, Bool, Void
│   ├── compounds.rs       # Struct, Enum, Union types
│   ├── generics.rs        # Type parameters, constraints
│   ├── functions.rs       # Function signatures, arrow types
│   └── special.rs         # Option<T>, Result<T,E>, etc.
├── inference/
│   ├── hindley_milner.rs  # HM type inference algorithm
│   ├── unification.rs     # Robinson's unification
│   └── constraints.rs     # Type constraints system
├── checker/
│   ├── expressions.rs     # Expression type checking
│   ├── statements.rs      # Statement type checking
│   └── patterns.rs        # Pattern matching type checking
└── environment.rs         # Type environment management
```

## Consecuencias

### Positivas
- ✅ **Type Safety**: Prevención de errores en runtime
- ✅ **Better IDE Support**: Autocompletado y error reporting precisos
- ✅ **Performance**: Code generation optimizada con tipos conocidos
- ✅ **Developer Experience**: Mejor feedback en desarrollo
- ✅ **Extensibility**: Fácil agregar nuevos tipos y features

### Negativas
- ❌ **Complejidad**: Sistema complejo de implementar y mantener
- ❌ **Performance Overhead**: Type checking agrega tiempo de compilación
- ❌ **Learning Curve**: Desarrolladores deben entender tipos

## Alternativas Consideradas

### 1. Dynamic Typing (Rechazada)
- **Pros**: Simple, flexible
- **Cons**: No type safety, errores en runtime
- **Razón**: No cumple con objetivos de Vela (type safety)

### 2. Structural Typing (Rechazada)
- **Pros**: Flexible, duck typing
- **Cons**: Difícil de razonar, errores sutiles
- **Razón**: Vela necesita explicit typing para claridad

### 3. Nominal Typing Only (Rechazada)
- **Pros**: Simple, claro
- **Cons**: No generics, limitado expressiveness
- **Razón**: Vela necesita generics y type inference

## Implementación

### Fase 1: Type Representation (TASK-013)
```rust
// compiler/src/types/mod.rs
pub mod primitives;
pub mod compounds;
pub mod generics;
pub mod functions;
pub mod special;

// Core type representation
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitives
    Number,
    String,
    Bool,
    Void,

    // Compounds
    Struct(StructType),
    Enum(EnumType),
    Union(Vec<Type>),

    // Generics
    Variable(TypeVar),
    Constructor(TypeConstructor),

    // Functions
    Function(FunctionType),

    // Special
    Option(Box<Type>),
    Result { ok: Box<Type>, err: Box<Type> },
}
```

### Fase 2: Type Inference (TASK-014)
```rust
// Hindley-Milner Algorithm
pub struct TypeInferer {
    supply: TypeVarSupply,
    constraints: Vec<Constraint>,
}

impl TypeInferer {
    pub fn infer(&mut self, expr: &Expression) -> Result<Type, TypeError> {
        match expr {
            Expression::Literal(lit) => self.infer_literal(lit),
            Expression::Variable(var) => self.infer_variable(var),
            Expression::Binary(op, left, right) => self.infer_binary(op, left, right),
            // ... more cases
        }
    }
}
```

### Fase 3: Type Checking (TASK-015, TASK-016)
```rust
pub struct TypeChecker {
    inferer: TypeInferer,
    environment: TypeEnvironment,
}

impl TypeChecker {
    pub fn check(&mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        for stmt in &program.statements {
            self.check_statement(stmt)?;
        }
        Ok(())
    }
}
```

## Referencias
- Jira: [VELA-013](https://velalang.atlassian.net/browse/VELA-013)
- Documentación: [Hindley-Milner Type Inference](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system)
- Código: `compiler/src/types/`

## Implementación
Ver código en: `compiler/src/types/`
Tests en: `tests/unit/compiler/types/`