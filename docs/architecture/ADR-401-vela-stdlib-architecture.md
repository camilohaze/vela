# ADR-401: Arquitectura de Vela Standard Library

## Estado
✅ Aceptado

## Fecha
2025-12-03

## Contexto

El runtime de Vela (Sprint 4) está completo con async runtime, channels, DI, events y HTTP framework. Ahora necesitamos una **Standard Library** robusta que proporcione:

1. **Tipos primitivos** envueltos con métodos ricos (Number, String, Bool)
2. **Collections** inmutables/mutables (List, Map, Set)
3. **Option/Result** para manejo de valores opcionales y errores
4. **Iterator protocol** para programación funcional
5. **String utilities** avanzadas (interpolación, regex, format)

### Requisitos Clave

- ✅ **Type-safe**: Generics y trait bounds
- ✅ **Zero-cost abstractions**: Sin overhead de runtime
- ✅ **Inmutabilidad por defecto**: Tipos envolviendo Rust collections
- ✅ **Interop con runtime**: Integración con channels, DI, events
- ✅ **API inspirada en**: Rust, TypeScript, Python, Swift

---

## Decisión

### 1. Estructura de Módulos

```
vela-stdlib/
├── src/
│   ├── lib.rs                # Módulo raíz con exports públicos
│   ├── primitives/
│   │   ├── mod.rs            # Primitives module
│   │   ├── number.rs         # VelaNumber (i64, f64, conversions)
│   │   ├── string.rs         # VelaString (wrapper sobre String)
│   │   └── bool.rs           # VelaBool (wrapper sobre bool)
│   ├── collections/
│   │   ├── mod.rs            # Collections module
│   │   ├── list.rs           # VelaList<T> (Vec wrapper)
│   │   ├── map.rs            # VelaMap<K,V> (HashMap wrapper)
│   │   └── set.rs            # VelaSet<T> (HashSet wrapper)
│   ├── option_result/
│   │   ├── mod.rs            # Option/Result module
│   │   ├── option.rs         # VelaOption<T> (Some/None)
│   │   └── result.rs         # VelaResult<T,E> (Ok/Err)
│   ├── iterators/
│   │   ├── mod.rs            # Iterator protocol
│   │   ├── iterator.rs       # VelaIterator trait
│   │   └── adapters.rs       # map, filter, reduce, etc
│   └── strings/
│       ├── mod.rs            # String utilities
│       ├── interpolation.rs  # String interpolation
│       ├── format.rs         # format!() equivalent
│       └── regex.rs          # Regex support
├── tests/
│   ├── primitives.rs         # Tests de primitives
│   ├── collections.rs        # Tests de collections
│   ├── option_result.rs      # Tests de Option/Result
│   ├── iterators.rs          # Tests de iterators
│   └── strings.rs            # Tests de strings
├── benches/
│   └── stdlib_bench.rs       # Benchmarks
├── Cargo.toml
└── README.md
```

---

## 2. Diseño de Tipos Primitivos

### VelaNumber

Wrapper sobre tipos numéricos de Rust con API rica:

```rust
pub enum VelaNumber {
    Int(i64),
    Float(f64),
}

impl VelaNumber {
    // Constructores
    pub fn int(value: i64) -> Self;
    pub fn float(value: f64) -> Self;
    
    // Conversiones
    pub fn as_int(&self) -> Option<i64>;
    pub fn as_float(&self) -> f64;
    pub fn to_string(&self) -> String;
    
    // Operaciones aritméticas
    pub fn add(&self, other: &VelaNumber) -> VelaNumber;
    pub fn sub(&self, other: &VelaNumber) -> VelaNumber;
    pub fn mul(&self, other: &VelaNumber) -> VelaNumber;
    pub fn div(&self, other: &VelaNumber) -> VelaResult<VelaNumber>;
    
    // Comparaciones
    pub fn eq(&self, other: &VelaNumber) -> bool;
    pub fn gt(&self, other: &VelaNumber) -> bool;
    pub fn lt(&self, other: &VelaNumber) -> bool;
    
    // Utilidades
    pub fn abs(&self) -> VelaNumber;
    pub fn pow(&self, exp: &VelaNumber) -> VelaNumber;
    pub fn sqrt(&self) -> VelaResult<VelaNumber>;
}
```

**Decisiones:**
- ✅ Unión de int/float en un solo tipo (como JavaScript/Python)
- ✅ Conversiones automáticas en operaciones
- ✅ Operadores sobrecargados vía traits (Add, Sub, etc)

### VelaString

Wrapper sobre `String` con métodos ricos:

```rust
pub struct VelaString(String);

impl VelaString {
    // Constructores
    pub fn new(s: impl Into<String>) -> Self;
    pub fn from_chars(chars: Vec<char>) -> Self;
    
    // Inspección
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn chars(&self) -> Vec<char>;
    
    // Transformaciones
    pub fn to_uppercase(&self) -> VelaString;
    pub fn to_lowercase(&self) -> VelaString;
    pub fn trim(&self) -> VelaString;
    pub fn split(&self, delimiter: &str) -> VelaList<VelaString>;
    pub fn replace(&self, from: &str, to: &str) -> VelaString;
    
    // Búsqueda
    pub fn contains(&self, substring: &str) -> bool;
    pub fn starts_with(&self, prefix: &str) -> bool;
    pub fn ends_with(&self, suffix: &str) -> bool;
    pub fn index_of(&self, substring: &str) -> VelaOption<usize>;
    
    // Interpolación (soporte para ${})
    pub fn interpolate(&self, values: &VelaMap<VelaString, VelaString>) -> VelaString;
}
```

**Decisiones:**
- ✅ Inmutable por defecto (clone para modificar)
- ✅ API inspirada en JavaScript/TypeScript
- ✅ Soporte nativo para interpolación `${}`

### VelaBool

Wrapper simple sobre `bool`:

```rust
pub struct VelaBool(bool);

impl VelaBool {
    pub fn new(value: bool) -> Self;
    pub fn as_bool(&self) -> bool;
    pub fn and(&self, other: &VelaBool) -> VelaBool;
    pub fn or(&self, other: &VelaBool) -> VelaBool;
    pub fn not(&self) -> VelaBool;
}
```

---

## 3. Diseño de Collections

### VelaList<T>

Lista genérica con API funcional:

```rust
pub struct VelaList<T> {
    inner: Vec<T>,
}

impl<T> VelaList<T> {
    // Constructores
    pub fn new() -> Self;
    pub fn from_vec(vec: Vec<T>) -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
    
    // Acceso
    pub fn get(&self, index: usize) -> VelaOption<&T>;
    pub fn first(&self) -> VelaOption<&T>;
    pub fn last(&self) -> VelaOption<&T>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    
    // Modificación (retorna nueva lista)
    pub fn push(&self, value: T) -> VelaList<T> where T: Clone;
    pub fn pop(&self) -> VelaOption<(T, VelaList<T>)> where T: Clone;
    pub fn insert(&self, index: usize, value: T) -> VelaList<T> where T: Clone;
    pub fn remove(&self, index: usize) -> VelaOption<(T, VelaList<T>)> where T: Clone;
    
    // Iteradores funcionales
    pub fn map<U, F>(&self, f: F) -> VelaList<U> 
        where F: Fn(&T) -> U;
    pub fn filter<F>(&self, f: F) -> VelaList<T> 
        where F: Fn(&T) -> bool, T: Clone;
    pub fn reduce<U, F>(&self, init: U, f: F) -> U 
        where F: Fn(U, &T) -> U;
    pub fn for_each<F>(&self, f: F) 
        where F: Fn(&T);
    
    // Utilidades
    pub fn contains(&self, value: &T) -> bool where T: PartialEq;
    pub fn reverse(&self) -> VelaList<T> where T: Clone;
    pub fn sort(&self) -> VelaList<T> where T: Clone + Ord;
    pub fn concat(&self, other: &VelaList<T>) -> VelaList<T> where T: Clone;
}
```

**Decisiones:**
- ✅ Inmutable por defecto (operaciones retornan nueva lista)
- ✅ API funcional completa (.map, .filter, .reduce)
- ✅ Inspirada en JavaScript/TypeScript arrays

### VelaMap<K, V>

Mapa genérico con API rica:

```rust
pub struct VelaMap<K, V> 
where K: Eq + Hash {
    inner: HashMap<K, V>,
}

impl<K, V> VelaMap<K, V> 
where K: Eq + Hash + Clone, V: Clone {
    // Constructores
    pub fn new() -> Self;
    pub fn from_hashmap(map: HashMap<K, V>) -> Self;
    
    // Acceso
    pub fn get(&self, key: &K) -> VelaOption<&V>;
    pub fn contains_key(&self, key: &K) -> bool;
    pub fn keys(&self) -> VelaList<K>;
    pub fn values(&self) -> VelaList<V>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    
    // Modificación (retorna nuevo mapa)
    pub fn insert(&self, key: K, value: V) -> VelaMap<K, V>;
    pub fn remove(&self, key: &K) -> VelaMap<K, V>;
    
    // Iteradores
    pub fn map<U, F>(&self, f: F) -> VelaMap<K, U> 
        where F: Fn(&K, &V) -> U, U: Clone;
    pub fn filter<F>(&self, f: F) -> VelaMap<K, V> 
        where F: Fn(&K, &V) -> bool;
    pub fn for_each<F>(&self, f: F) 
        where F: Fn(&K, &V);
}
```

### VelaSet<T>

Set genérico:

```rust
pub struct VelaSet<T> 
where T: Eq + Hash {
    inner: HashSet<T>,
}

impl<T> VelaSet<T> 
where T: Eq + Hash + Clone {
    // Constructores
    pub fn new() -> Self;
    pub fn from_hashset(set: HashSet<T>) -> Self;
    
    // Acceso
    pub fn contains(&self, value: &T) -> bool;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    
    // Modificación (retorna nuevo set)
    pub fn insert(&self, value: T) -> VelaSet<T>;
    pub fn remove(&self, value: &T) -> VelaSet<T>;
    
    // Operaciones de conjunto
    pub fn union(&self, other: &VelaSet<T>) -> VelaSet<T>;
    pub fn intersection(&self, other: &VelaSet<T>) -> VelaSet<T>;
    pub fn difference(&self, other: &VelaSet<T>) -> VelaSet<T>;
}
```

---

## 4. Option/Result Types

### VelaOption<T>

Type-safe para valores opcionales:

```rust
pub enum VelaOption<T> {
    Some(T),
    None,
}

impl<T> VelaOption<T> {
    // Constructores
    pub fn some(value: T) -> Self;
    pub fn none() -> Self;
    
    // Inspección
    pub fn is_some(&self) -> bool;
    pub fn is_none(&self) -> bool;
    
    // Extracción
    pub fn unwrap(self) -> T;
    pub fn unwrap_or(self, default: T) -> T;
    pub fn unwrap_or_else<F>(self, f: F) -> T where F: FnOnce() -> T;
    
    // Transformación
    pub fn map<U, F>(self, f: F) -> VelaOption<U> where F: FnOnce(T) -> U;
    pub fn and_then<U, F>(self, f: F) -> VelaOption<U> where F: FnOnce(T) -> VelaOption<U>;
    pub fn or_else<F>(self, f: F) -> VelaOption<T> where F: FnOnce() -> VelaOption<T>;
    pub fn filter<F>(self, f: F) -> VelaOption<T> where F: FnOnce(&T) -> bool;
}
```

### VelaResult<T, E>

Type-safe para manejo de errores:

```rust
pub enum VelaResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> VelaResult<T, E> {
    // Constructores
    pub fn ok(value: T) -> Self;
    pub fn err(error: E) -> Self;
    
    // Inspección
    pub fn is_ok(&self) -> bool;
    pub fn is_err(&self) -> bool;
    
    // Extracción
    pub fn unwrap(self) -> T;
    pub fn unwrap_or(self, default: T) -> T;
    pub fn unwrap_err(self) -> E;
    
    // Transformación
    pub fn map<U, F>(self, f: F) -> VelaResult<U, E> where F: FnOnce(T) -> U;
    pub fn map_err<U, F>(self, f: F) -> VelaResult<T, U> where F: FnOnce(E) -> U;
    pub fn and_then<U, F>(self, f: F) -> VelaResult<U, E> where F: FnOnce(T) -> VelaResult<U, E>;
}
```

**Decisiones:**
- ✅ API idéntica a Rust std::option/result
- ✅ Soporte para pattern matching futuro
- ✅ Métodos funcionales (map, and_then, or_else)

---

## 5. Iterator Protocol

### VelaIterator Trait

Trait para iteración:

```rust
pub trait VelaIterator {
    type Item;
    
    fn next(&mut self) -> VelaOption<Self::Item>;
    
    // Métodos funcionales
    fn map<U, F>(self, f: F) -> Map<Self, F>
        where Self: Sized, F: FnMut(Self::Item) -> U;
    
    fn filter<F>(self, f: F) -> Filter<Self, F>
        where Self: Sized, F: FnMut(&Self::Item) -> bool;
    
    fn reduce<B, F>(self, init: B, f: F) -> B
        where Self: Sized, F: FnMut(B, Self::Item) -> B;
    
    fn for_each<F>(self, f: F)
        where Self: Sized, F: FnMut(Self::Item);
    
    fn collect<B: FromVelaIterator<Self::Item>>(self) -> B
        where Self: Sized;
    
    // Utilidades
    fn count(self) -> usize where Self: Sized;
    fn take(self, n: usize) -> Take<Self> where Self: Sized;
    fn skip(self, n: usize) -> Skip<Self> where Self: Sized;
    fn zip<U>(self, other: U) -> Zip<Self, U::IntoIter>
        where Self: Sized, U: IntoVelaIterator;
}
```

**Decisiones:**
- ✅ Trait similar a Iterator de Rust
- ✅ Lazy evaluation (iteradores no consumen hasta collect)
- ✅ Composición funcional completa

---

## 6. String Utilities

### Interpolación

Soporte nativo para `${}`:

```rust
pub fn interpolate(template: &str, values: &VelaMap<String, String>) -> String;

// Ejemplo:
// interpolate("Hello, ${name}!", map![("name", "Vela")])
// → "Hello, Vela!"
```

### Format

Función de formato estilo Rust:

```rust
pub fn format(template: &str, args: &[VelaString]) -> VelaString;

// Ejemplo:
// format("User {} has {} points", &["Alice", "100"])
// → "User Alice has 100 points"
```

### Regex Support

Integración con `regex` crate:

```rust
pub struct VelaRegex {
    inner: regex::Regex,
}

impl VelaRegex {
    pub fn new(pattern: &str) -> VelaResult<Self>;
    pub fn is_match(&self, text: &str) -> bool;
    pub fn find(&self, text: &str) -> VelaOption<VelaString>;
    pub fn find_all(&self, text: &str) -> VelaList<VelaString>;
    pub fn replace(&self, text: &str, replacement: &str) -> VelaString;
}
```

---

## Consecuencias

### Positivas

1. **Type-safety completo**: Generics y traits garantizan seguridad
2. **Zero-cost abstractions**: Wrappers no agregan overhead
3. **API familiar**: Inspirada en lenguajes populares (Rust, TS, Python)
4. **Inmutabilidad por defecto**: Menos bugs, código más predecible
5. **Composición funcional**: Iteradores lazy y métodos funcionales
6. **Interop con runtime**: Collections/Option/Result funcionan con channels, DI, etc

### Negativas

1. **Complejidad inicial**: Muchos módulos para implementar
2. **Testing exhaustivo**: Cada método necesita tests completos
3. **Performance tuning**: Benchmarks para optimizar clones
4. **Documentación extensa**: API docs completos para cada tipo

---

## Alternativas Consideradas

### 1. Usar tipos de Rust directamente
**Rechazada**: No permite personalización de API ni mensajes de error específicos de Vela

### 2. Collections mutables por defecto
**Rechazada**: Va contra el paradigma funcional puro de Vela

### 3. Option/Result como traits
**Rechazada**: Enums son más idiomáticos y eficientes

---

## Referencias

- **Rust std**: https://doc.rust-lang.org/std/
- **TypeScript**: https://www.typescriptlang.org/docs/
- **Python collections**: https://docs.python.org/3/library/collections.html
- **Swift stdlib**: https://developer.apple.com/documentation/swift/swift-standard-library

---

## Implementación

Ver código en:
- `stdlib/src/primitives/` - Tipos primitivos
- `stdlib/src/collections/` - Collections
- `stdlib/src/option_result/` - Option/Result
- `stdlib/src/iterators/` - Iterator protocol
- `stdlib/src/strings/` - String utilities

Tests en: `stdlib/tests/`

---

**Fecha de Implementación**: Sprint 5 (Diciembre 2025)  
**Jira**: EPIC-RUST-05, TASK-RUST-401
