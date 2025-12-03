# Arquitectura del Sistema de Tipos

## Visión General

El sistema de tipos de Vela es un sistema híbrido que combina tipado estático con inferencia automática de tipos, inspirado en el algoritmo Hindley-Milner usado en lenguajes funcionales como Haskell y OCaml.

## Componentes Principales

### 1. Definiciones de Tipos (`types.rs`)

El corazón del sistema es el enum `Type`, que representa todos los tipos posibles en Vela:

```rust
pub enum Type {
    Unit,           // ()
    Bool,           // true/false
    Int,            // i64
    Float,          // f64
    String,         // String
    Char,           // char
    Array(Box<Type>), // [T]
    Function { params: Vec<Type>, ret: Box<Type> }, // (T1, T2) -> R
    Tuple(Vec<Type>), // (T1, T2, T3)
    Record(HashMap<String, Type>), // {field: Type}
    Variant(HashMap<String, Type>), // enum variants
    Option(Box<Type>), // Option<T>
    Result { ok: Box<Type>, err: Box<Type> }, // Result<T, E>
    Var(TypeVar),   // Type variables for inference
    Generic { name: String, args: Vec<Type> }, // Generic<T>
    Never,          // !
}
```

### 2. Contexto de Tipos (`context.rs`)

Gestiona el entorno de tipos durante la verificación:

- **Variables de tipo**: `TypeVar` con IDs únicos
- **Esquemas de tipo**: `TypeScheme` para cuantificación existencial
- **Scopes**: Gestión de visibilidad de variables
- **Substitución**: Unificación de variables de tipo

### 3. Inferencia de Tipos (`inference.rs`)

Implementa el algoritmo Hindley-Milner:

- **Unificación**: Resolución de ecuaciones de tipos
- **Generalización**: Creación de esquemas polimórficos
- **Instanciación**: Creación de tipos concretos desde esquemas
- **Constraints**: Sistema de restricciones para tipos complejos

### 4. Verificación de Tipos (`checker.rs`)

Orquesta todo el proceso de verificación:

- **Type checking**: Verificación de expresiones
- **Pattern matching**: Verificación de patrones exhaustivos
- **Unificación**: Resolución de tipos
- **Error reporting**: Mensajes de error detallados

### 5. Sistema de Errores (`error.rs`)

Errores específicos del sistema de tipos:

```rust
pub enum TypeError {
    UnificationError { expected: Type, actual: Type },
    UndefinedVariable(String),
    TypeMismatch { expected: Type, actual: Type },
    InfiniteType(TypeVar),
    // ... más errores
}
```

## Flujo de Verificación

1. **Parsing**: AST generado por el compilador
2. **Inferencia**: Variables de tipo asignadas a expresiones
3. **Unificación**: Resolución de constraints
4. **Generalización**: Creación de tipos polimórficos
5. **Verificación**: Comprobación final de tipos

## Características Avanzadas

### Polimorfismo Paramétrico
```rust
// Función polimórfica
fn identity<T>(x: T) -> T { x }

// Esquema: ∀T. T → T
```

### Inferencia Bidireccional
- **De arriba hacia abajo**: Cuando hay anotaciones explícitas
- **De abajo hacia arriba**: Inferencia automática

### Type Classes (Futuro)
Sistema extensible para traits/polimorfismo ad-hoc.

## Integración con el Compilador

El crate `vela-types` se integra con:
- `vela-compiler`: Para análisis semántico
- `vela-vm`: Para verificación en runtime
- `vela-lsp`: Para IntelliSense y errores en IDE

## Performance

- **Complejidad**: O(n) para la mayoría de operaciones
- **Memoria**: Estructuras inmutables para sharing
- **Paralelización**: Verificación independiente por módulo