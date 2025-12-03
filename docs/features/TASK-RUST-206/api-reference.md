# API Reference - Sistema de Tipos

## Tipos Principales

### `Type`

Enum que representa todos los tipos del lenguaje.

```rust
pub enum Type {
    Unit,
    Bool,
    Int,
    Float,
    String,
    Char,
    Array(Box<Type>),
    Function { params: Vec<Type>, ret: Box<Type> },
    Tuple(Vec<Type>),
    Record(HashMap<String, Type>),
    Variant(HashMap<String, Type>),
    Option(Box<Type>),
    Result { ok: Box<Type>, err: Box<Type> },
    Var(TypeVar),
    Generic { name: String, args: Vec<Type> },
    Never,
}
```

#### Métodos

```rust
impl Type {
    pub fn is_mono(&self) -> bool
    pub fn free_vars(&self) -> HashSet<TypeVar>
    pub fn apply_subst(&self, subst: &Substitution) -> Type
    pub fn generalize(&self, context: &TypeContext) -> TypeScheme
}
```

### `TypeScheme`

Representa tipos polimórficos cuantificados.

```rust
pub struct TypeScheme {
    pub vars: Vec<TypeVar>,
    pub ty: Type,
}
```

#### Métodos

```rust
impl TypeScheme {
    pub fn mono(ty: Type) -> Self
    pub fn poly(vars: Vec<TypeVar>, ty: Type) -> Self
    pub fn instantiate(&self) -> Type
}
```

### `TypeVar`

Variables de tipo para inferencia.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(pub usize);

impl TypeVar {
    pub fn fresh() -> Self
}
```

## Contexto de Tipos

### `TypeContext`

Gestiona el entorno de tipos.

```rust
pub struct TypeContext {
    // Implementación interna
}
```

#### Métodos

```rust
impl TypeContext {
    pub fn new() -> Self
    pub fn extend(&mut self, name: String, scheme: TypeScheme)
    pub fn lookup(&self, name: &str) -> Option<&TypeScheme>
    pub fn fresh_var(&mut self) -> TypeVar
}
```

## Verificador de Tipos

### `TypeChecker`

Clase principal para verificación de tipos.

```rust
pub struct TypeChecker {
    context: TypeContext,
}
```

#### Métodos

```rust
impl TypeChecker {
    pub fn new() -> Self
    pub fn check_expr(&mut self, expr: &Expr) -> TypeResult<Type>
    pub fn check_stmt(&mut self, stmt: &Stmt) -> TypeResult<()>
    pub fn infer_expr(&mut self, expr: &Expr) -> TypeResult<Type>
}
```

## Sistema de Errores

### `TypeError`

Errores específicos del sistema de tipos.

```rust
pub enum TypeError {
    UnificationError { expected: Type, actual: Type },
    UndefinedVariable(String),
    TypeMismatch { expected: Type, actual: Type },
    InfiniteType(TypeVar),
    ArityMismatch { expected: usize, actual: usize },
    FieldNotFound { record: Type, field: String },
    VariantNotFound { variant: Type, case: String },
    RecursiveType(Type),
    ConstraintViolation(String),
}
```

## Inferencia de Tipos

### Funciones de Inferencia

```rust
pub fn unify(left: &Type, right: &Type) -> Result<Substitution, TypeError>

pub fn generalize(ty: Type, context: &TypeContext) -> TypeScheme

pub fn instantiate(scheme: &TypeScheme) -> Type

pub fn occurs_check(var: TypeVar, ty: &Type) -> bool
```

## Ejemplos de Uso

### Verificación Básica

```rust
use vela_types::{TypeChecker, TypeResult};

let mut checker = TypeChecker::new();

// Verificar expresión
let expr = /* AST de expresión */;
match checker.check_expr(&expr) {
    Ok(ty) => println!("Tipo inferido: {:?}", ty),
    Err(err) => println!("Error de tipo: {:?}", err),
}
```

### Extensión del Contexto

```rust
let mut context = TypeContext::new();

// Agregar variable al contexto
context.extend("x".to_string(), TypeScheme::mono(Type::Int));

// Buscar variable
if let Some(scheme) = context.lookup("x") {
    println!("Esquema encontrado: {:?}", scheme);
}
```

### Unificación Manual

```rust
use vela_types::inference::unify;

let ty1 = Type::Function {
    params: vec![Type::Int],
    ret: Box::new(Type::Bool),
};

let ty2 = Type::Function {
    params: vec![Type::Var(TypeVar::fresh())],
    ret: Box::new(Type::Bool),
};

match unify(&ty1, &ty2) {
    Ok(subst) => println!("Unificación exitosa: {:?}", subst),
    Err(err) => println!("Error de unificación: {:?}", err),
}
```

## Constraints y Extensiones

### Sistema de Constraints (Futuro)

```rust
pub trait Constraint {
    fn check(&self, context: &TypeContext) -> bool;
}

pub struct EqualityConstraint {
    pub left: Type,
    pub right: Type,
}
```

### Type Classes (Futuro)

```rust
pub trait TypeClass {
    fn name(&self) -> &str;
    fn instances(&self) -> Vec<Type>;
    fn methods(&self) -> HashMap<String, TypeScheme>;
}
```