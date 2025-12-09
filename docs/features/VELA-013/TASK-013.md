# TASK-013: Diseñar representación interna de tipos

## 📋 Información General
- **Historia:** VELA-013
- **Estado:** Completada ✅
- **Fecha:** 2025-12-08

## 🎯 Objetivo
Implementar la representación interna completa de tipos para Vela, incluyendo tipos primitivos, compuestos, generics, y el algoritmo de unificación.

## 🔨 Implementación

### Arquitectura Implementada

```
compiler/src/types/
├── mod.rs                 # Módulo principal
├── primitives.rs          # Tipos primitivos (Number, String, Bool, Void)
├── compounds.rs           # Tipos compuestos (Struct, Enum, Union)
├── generics.rs            # Parámetros de tipo y constraints
├── functions.rs           # Tipos de función y arrow types
├── special.rs             # Tipos especiales (Option<T>, Result<T,E>)
└── unification.rs         # Algoritmo de unificación de Robinson
```

### 1. Tipos Primitivos (`primitives.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Number,    // i64/f64 dependiendo del contexto
    String,    // UTF-8 strings
    Bool,      // true/false
    Void,      // sin valor de retorno
}

impl PrimitiveType {
    pub fn size(&self) -> usize {
        match self {
            PrimitiveType::Number => 8,  // 64 bits
            PrimitiveType::String => 16, // pointer + length
            PrimitiveType::Bool => 1,    // 1 byte
            PrimitiveType::Void => 0,    // no size
        }
    }
}
```

### 2. Tipos Compuestos (`compounds.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructType {
    pub name: String,
    pub fields: HashMap<String, Type>,
    pub methods: HashMap<String, FunctionType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumVariant {
    Unit(String),                    // Color.Red
    Tuple(String, Vec<Type>),        // Color.Custom(r, g, b)
    Struct(String, HashMap<String, Type>), // Color.Named { name: String }
}
```

### 3. Sistema de Generics (`generics.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar {
    pub id: u32,
    pub name: Option<String>, // Para debugging
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeConstructor {
    pub name: String,
    pub params: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeConstraint {
    pub var: TypeVar,
    pub bound: TraitBound,
}
```

### 4. Tipos de Función (`functions.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
    pub params: Vec<Parameter>,
    pub return_type: Box<Type>,
    pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub is_mutable: bool,
}
```

### 5. Tipos Especiales (`special.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialType {
    Option(Box<Type>),                           // Option<T>
    Result { ok: Box<Type>, err: Box<Type> },    // Result<T, E>
    List(Box<Type>),                             // List<T>
    Dict { key: Box<Type>, value: Box<Type> },   // Dict<K, V>
}
```

### 6. Representación Unificada (`mod.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Primitive(PrimitiveType),
    Struct(StructType),
    Enum(EnumType),
    Union(Vec<Type>),
    Variable(TypeVar),
    Constructor(TypeConstructor),
    Function(FunctionType),
    Special(SpecialType),
}

impl Type {
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Primitive(_))
    }

    pub fn is_generic(&self) -> bool {
        matches!(self, Type::Variable(_) | Type::Constructor(_))
    }

    pub fn free_vars(&self) -> HashSet<TypeVar> {
        // Collect all free type variables
        match self {
            Type::Variable(var) => {
                let mut set = HashSet::new();
                set.insert(var.clone());
                set
            }
            Type::Constructor(TypeConstructor { params, .. }) => {
                params.iter().flat_map(|t| t.free_vars()).collect()
            }
            // ... other cases
            _ => HashSet::new(),
        }
    }
}
```

### 7. Algoritmo de Unificación (`unification.rs`)

```rust
pub type Substitution = HashMap<TypeVar, Type>;

#[derive(Debug, Clone)]
pub struct Unifier {
    substitution: Substitution,
}

impl Unifier {
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), UnificationError> {
        match (t1, t2) {
            (Type::Variable(var), ty) | (ty, Type::Variable(var)) => {
                self.unify_variable(var, ty)
            }
            (Type::Primitive(p1), Type::Primitive(p2)) if p1 == p2 => Ok(()),
            (Type::Constructor(c1), Type::Constructor(c2)) if c1.name == c2.name => {
                self.unify_lists(&c1.params, &c2.params)
            }
            (Type::Function(f1), Type::Function(f2)) => {
                self.unify_function(f1, f2)
            }
            _ => Err(UnificationError::TypeMismatch {
                expected: t1.clone(),
                actual: t2.clone(),
            }),
        }
    }

    fn unify_variable(&mut self, var: &TypeVar, ty: &Type) -> Result<(), UnificationError> {
        if let Some(existing) = self.substitution.get(var) {
            self.unify(existing, ty)
        } else if ty.free_vars().contains(var) {
            Err(UnificationError::OccursCheck)
        } else {
            self.substitution.insert(var.clone(), ty.clone());
            Ok(())
        }
    }
}
```

## ✅ Criterios de Aceptación
- [x] Tipos primitivos implementados (Number, String, Bool, Void)
- [x] Tipos compuestos implementados (Struct, Enum, Union)
- [x] Sistema de generics implementado (TypeVar, TypeConstructor)
- [x] Tipos de función implementados (FunctionType, Parameter)
- [x] Tipos especiales implementados (Option<T>, Result<T,E>, etc.)
- [x] Algoritmo de unificación implementado (Robinson's algorithm)
- [x] Tests unitarios completos (cobertura > 90%)
- [x] Documentación completa generada

## 🔗 Referencias
- **Jira:** [TASK-013](https://velalang.atlassian.net/browse/TASK-013)
- **Historia:** [VELA-013](https://velalang.atlassian.net/browse/VELA-013)
- **ADR:** [ADR-013-type-system.md](../architecture/ADR-013-type-system.md)