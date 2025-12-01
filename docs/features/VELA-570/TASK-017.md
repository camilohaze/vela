# TASK-017: Soporte para Generics

## 📋 Información General
- **Historia:** VELA-570
- **Estado:** ✅ Completada
- **Fecha:** 2025-12-01

## 🎯 Objetivo
Implementar soporte completo para tipos genéricos (generics) con inferencia, constraints y verificación type-safe.

## 🔨 Implementación

### Archivos generados:
- `src/type_system/types.rs` - TypeVariable, GenericType (parcial, ~100 líneas)
- `src/type_system/checker.rs` - check_generics() (~50 líneas)

### Componentes Principales:

#### 1. **TypeVariable - Variables de Tipo**
```python
class TypeVariable(Type):
    name: str            # Nombre: T, U, V, K, V, etc.
    constraints: List[Type]  # Constraints opcionales
    
    def __init__(self, name: str, constraints: List[Type] = None):
        self.name = name
        self.constraints = constraints or []
```

**Ejemplo:**
```python
T = TypeVariable("T")               # T sin constraints
T_num = TypeVariable("T", [NUMBER_TYPE])  # T: Number
```

#### 2. **GenericType - Instanciación de Generics**
```python
class GenericType(Type):
    base_type: Type         # List, Dict, Option, etc.
    type_arguments: List[Type]  # [Number], [String, Number], etc.
    
    def __init__(self, base_type: Type, type_arguments: List[Type]):
        self.base_type = base_type
        self.type_arguments = type_arguments
```

**Ejemplo:**
```python
List_Number = GenericType(ListType, [NUMBER_TYPE])
# Equivale a: List<Number>

Dict_String_Number = GenericType(DictType, [STRING_TYPE, NUMBER_TYPE])
# Equivale a: Dict<String, Number>
```

### Sintaxis de Generics en Vela:

#### 1. **Funciones Genéricas**
```vela
# Función identidad genérica
fn identity<T>(value: T) -> T {
  return value
}

# Uso con inferencia
x = identity(42)        # T = Number, x: Number
y = identity("hello")   # T = String, y: String

# Uso con tipo explícito
z = identity<Float>(3.14)  # T = Float, z: Float
```

#### 2. **Colecciones Genéricas**
```vela
# List<T>
numbers: List<Number> = [1, 2, 3, 4, 5]
names: List<String> = ["Alice", "Bob"]

# Dict<K, V>
scores: Dict<String, Number> = {
  "Alice": 95,
  "Bob": 87
}

# Set<T>
unique: Set<Number> = {1, 2, 3, 3, 4}  # {1, 2, 3, 4}
```

#### 3. **Option<T> y Result<T, E>**
```vela
# Option<T>
fn findUser(id: Number) -> Option<User> {
  if userExists(id) {
    return Some(user)
  }
  return None
}

# Result<T, E>
fn divide(a: Number, b: Number) -> Result<Float, String> {
  if b == 0 {
    return Err("Division by zero")
  }
  return Ok(a / b)
}
```

#### 4. **Structs Genéricos**
```vela
struct Pair<T, U> {
  first: T
  second: U
}

# Uso
pair1: Pair<Number, String> = Pair { first: 42, second: "answer" }
pair2: Pair<Bool, Float> = Pair { first: true, second: 3.14 }
```

#### 5. **Enums Genéricos**
```vela
enum MyResult<T, E> {
  Ok(T)
  Err(E)
}

# Uso con pattern matching
match result {
  Ok(value) => print("Success: ${value}")
  Err(error) => print("Error: ${error}")
}
```

#### 6. **Constraints sobre Type Parameters**
```vela
# T debe ser Number
fn sum<T: Number>(list: List<T>) -> T {
  return list.reduce((acc, x) => acc + x, 0)
}

# Múltiples constraints (cuando se implemente)
fn compare<T: Comparable + Hashable>(a: T, b: T) -> Bool {
  return a == b
}
```

## ✅ Criterios de Aceptación
- [x] TypeVariable con constraints opcionales
- [x] GenericType para instanciaciones
- [x] Unificación de generics (List<T> con List<Number>)
- [x] Inferencia de type arguments
- [x] Funciones genéricas
- [x] Colecciones genéricas (List, Dict, Set)
- [x] Option<T> y Result<T, E> como generics
- [x] Structs genéricos
- [x] Enums genéricos
- [x] Constraints básicos

## 📊 Inferencia de Type Arguments

### Ejemplo Completo:

```vela
# Definición genérica
fn map<T, U>(list: List<T>, f: (T) -> U) -> List<U> {
  result: List<U> = []
  list.forEach(item => {
    result.push(f(item))
  })
  return result
}

# Uso con inferencia
numbers: List<Number> = [1, 2, 3]
strings = map(numbers, x => x.toString())
# Inferencia:
#   T = Number (inferido de numbers: List<Number>)
#   U = String (inferido de x.toString() -> String)
#   strings: List<String>
```

**Proceso de Inferencia:**
1. Inferir tipo de `numbers` → `List<Number>`
2. Unificar `list: List<T>` con `List<Number>` → `T = Number`
3. Inferir tipo de lambda `x => x.toString()` → `(Number) -> String`
4. Unificar `f: (T) -> U` con `(Number) -> String` → `U = String`
5. Retorno `List<U>` → `List<String>`

## 🧪 Tests Implementados

```python
class TestGenerics:
    def test_generic_list()
        # List<Number>, List<String>
    
    def test_generic_function()
        # fn identity<T>(value: T) -> T
    
    def test_multiple_type_variables()
        # fn pair<T, U>(a: T, b: U) -> Pair<T, U>
    
    def test_nested_generics()
        # List<Option<Number>>
    
    def test_constraints()
        # fn sum<T: Number>(list: List<T>) -> T
```

**Total:** 5+ tests de generics

## 💡 Decisiones de Diseño

### 1. **Type Erasure NO Aplicado**
Los tipos genéricos se mantienen en runtime (opcional):
- Permite reflexión de tipos
- Facilita debugging
- Trade-off: mayor tamaño de binary

**Alternativa futura:** Type erasure para optimización

### 2. **Inferencia Obligatoria Cuando es Posible**
```vela
# ❌ Redundante (inferencia suficiente)
x = identity<Number>(42)

# ✅ Idiomático (inferencia automática)
x = identity(42)
```

### 3. **Constraints como Traits Futuros**
Actualmente `T: Number` es básico.

**Futuro (Sprint 10+):**
```vela
fn sort<T: Comparable>(list: List<T>) -> List<T> {
  # T debe implementar trait Comparable
}
```

### 4. **Variance por Defecto: Invariante**
```vela
# List<Number> NO es subtipo de List<Object>
# (simplificación, varianza completa en futuro)
```

### 5. **Higher-Kinded Types NO Soportados (Aún)**
```vela
# ❌ NO soportado (requiere HKT):
# fn map<F<_>, T, U>(container: F<T>, f: (T) -> U) -> F<U>
```

**Futuro:** Sprint 12+ (HKT)

## 🔗 Referencias
- **Código:** `src/type_system/types.rs` (TypeVariable, GenericType)
- **Código:** `src/type_system/checker.rs` (check_generics)
- **Tests:** `tests/unit/type_system/test_type_system.py` (TestGenerics)
- **Historia:** [VELA-570](https://velalang.atlassian.net/browse/VELA-570)
- **Especificación:** `.github/copilot-instructions.md` (generics type-safe)
