# TASK-019: Type Narrowing

## 📋 Información General
- **Historia:** VELA-570
- **Estado:** ✅ Completada (framework básico)
- **Fecha:** 2025-12-01

## 🎯 Objetivo
Implementar type narrowing para refinar tipos en control de flujo, permitiendo type safety con verificaciones en runtime.

## 🔨 Implementación

### Archivos generados:
- `src/type_system/checker.rs` - check_type_narrowing() (~50 líneas)
- `src/type_system/env.rs` - update() para type refinement (~20 líneas)

### Componentes Principales:

#### 1. **check_type_narrowing() - Refinamiento de Tipos**
```python
def check_type_narrowing(self, expr, narrowed_type: Type) -> void:
    """
    Refina el tipo de una expresión en un scope específico.
    
    Usado en:
    - if-let con Option<T>
    - match patterns
    - Guardas de tipo
    """
```

#### 2. **TypeEnvironment.update() - Actualización de Tipos**
```python
def update(self, name: str, new_type: Type) -> void:
    """
    Actualiza el tipo de una variable en el scope actual.
    Usado para type narrowing.
    """
```

### Casos de Type Narrowing:

#### 1. **if-let con Option<T>**
```vela
user: Option<User> = findUser(123)

# Antes del if-let: user: Option<User>
if let Some(u) = user {
  # Dentro del if: u: User (tipo refinado)
  print(u.name)  # OK: u es User, no Option<User>
  print(u.age)   # OK
}

# Después del if: user: Option<User> (tipo original)
```

**Proceso de Type Narrowing:**
1. Inferir tipo de `user` → `Option<User>`
2. Pattern match `Some(u)` → extraer tipo interno `User`
3. Crear nuevo Symbol `u: User` en scope del if
4. Dentro del if: `u` tiene tipo `User`
5. Fuera del if: `u` no existe

#### 2. **match con Pattern Matching**
```vela
result: Result<Number, String> = divide(10, 0)

match result {
  Ok(value) => {
    # Aquí: value: Number (refinado)
    print("Result: ${value}")
  }
  Err(error) => {
    # Aquí: error: String (refinado)
    print("Error: ${error}")
  }
}
```

**Type Narrowing en cada rama:**
- Rama `Ok(value)`: tipo refinado a `T` (Number)
- Rama `Err(error)`: tipo refinado a `E` (String)

#### 3. **Type Guards (Futuro - Sprint 10+)**
```vela
value: Number | String = getValue()

if value is String {
  # Aquí: value: String (refinado)
  print(value.toUpperCase())
}

if value is Number {
  # Aquí: value: Number (refinado)
  print(value + 10)
}
```

**Nota:** Union types (`Number | String`) en Sprint 10.

#### 4. **Null Checks (NO APLICA en Vela)**
```vela
# ❌ Otros lenguajes con null:
# if (user != null) {
#   print(user.name);  // user: User (refinado)
# }

# ✅ Vela usa Option<T>, no null:
if let Some(u) = user {
  print(u.name)  # u: User (refinado)
}
```

#### 5. **Truthiness Checks**
```vela
value: Option<String> = getOptionalValue()

# Narrowing con isSome()
if value.isSome() {
  # value sigue siendo Option<String> (NO refinado)
  # Debe hacer unwrap:
  print(value.unwrap())
}

# ✅ Mejor: if-let (refina automáticamente)
if let Some(v) = value {
  # v: String (refinado)
  print(v)
}
```

## ✅ Criterios de Aceptación
- [x] if-let refina Option<T> a T
- [x] match patterns refinan tipos en cada rama
- [x] Type narrowing dentro de scopes específicos
- [x] Tipo original restaurado fuera de scope
- [x] update() en TypeEnvironment para refinamiento
- [x] Framework básico para type guards futuros

## 📊 Tabla de Type Narrowing

| Construcción | Tipo Original | Tipo Refinado | Scope |
|--------------|---------------|---------------|-------|
| `if let Some(x) = opt` | `opt: Option<T>` | `x: T` | Dentro del if |
| `match Ok(v)` | `result: Result<T, E>` | `v: T` | Rama Ok |
| `match Err(e)` | `result: Result<T, E>` | `e: E` | Rama Err |
| `match Some(x)` | `opt: Option<T>` | `x: T` | Rama Some |
| `match None` | `opt: Option<T>` | No binding | Rama None |
| `if x is Type` | `x: Union<...>` | `x: Type` | Dentro del if (futuro) |

## 🧪 Tests Implementados

```python
class TestTypeNarrowing:
    def test_if_let_narrowing()
        # Option<T> -> T
    
    def test_match_narrowing_result()
        # Result<T, E> -> T y E
    
    def test_match_narrowing_option()
        # Option<T> -> T
    
    def test_scope_restoration()
        # Tipo original restaurado fuera de scope
    
    def test_nested_narrowing()
        # if-let dentro de match
```

**Total:** 5+ tests de type narrowing

## 💡 Decisiones de Diseño

### 1. **Type Narrowing Solo en Scopes Anidados**
El tipo refinado solo existe dentro del scope donde se hizo el narrowing:
```vela
user: Option<User> = findUser(123)

if let Some(u) = user {
  print(u.name)  # OK: u: User
}

# print(u.name)  # ERROR: u no existe aquí
```

### 2. **Inmutabilidad del Tipo Original**
El tipo original NO cambia:
```vela
opt: Option<Number> = Some(42)

if let Some(x) = opt {
  # x: Number (nueva variable)
  # opt: Option<Number> (sin cambios)
}
```

### 3. **Pattern Matching Exhaustivo**
match DEBE cubrir todos los casos para garantizar type safety:
```vela
# ✅ OK: exhaustivo
match result {
  Ok(v) => handleValue(v)
  Err(e) => handleError(e)
}

# ❌ ERROR: no exhaustivo
# match result {
#   Ok(v) => handleValue(v)
#   # Falta rama Err
# }
```

### 4. **Shadowing vs Narrowing**
Son conceptos diferentes:

**Shadowing (nueva variable):**
```vela
x: Number = 10
{
  x: String = "hello"  # Nueva variable x
  print(x)  # "hello"
}
print(x)  # 10
```

**Narrowing (mismo valor, tipo refinado):**
```vela
opt: Option<Number> = Some(42)
if let Some(x) = opt {
  # x NO es nueva variable, es opt "unwrapped"
  print(x)  # 42
}
```

### 5. **Type Guards Futuros (Sprint 10+)**
Framework preparado para type guards con union types:
```vela
# Futuro: union types
value: Number | String | Bool = getValue()

if value is String {
  # value: String (refinado)
}
```

**Requerimientos:**
- Union types (Sprint 10)
- `is` operator
- Runtime type checks

## 🔗 Referencias
- **Código:** `src/type_system/checker.rs` (check_type_narrowing)
- **Código:** `src/type_system/env.rs` (update)
- **Tests:** `tests/unit/type_system/test_type_system.py` (TestTypeNarrowing)
- **Historia:** [VELA-570](https://velalang.atlassian.net/browse/VELA-570)
- **Especificación:** `.github/copilot-instructions.md` (type narrowing)
- **Inspiración:** TypeScript's type narrowing, Flow's type refinement
