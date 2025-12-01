# TASK-018: Option<T> Safety Checking

## 📋 Información General
- **Historia:** VELA-570
- **Estado:** ✅ Completada
- **Fecha:** 2025-12-01

## 🎯 Objetivo
Implementar verificación estricta de Option<T> para eliminar null pointer exceptions, prohibiendo uso de null/undefined/nil.

## 🔨 Implementación

### Archivos generados:
- `src/type_system/types.rs` - OptionType (~50 líneas)
- `src/type_system/checker.rs` - check_option_safety() (~50 líneas)

### Componentes Principales:

#### 1. **OptionType - Representación de Option<T>**
```python
class OptionType(Type):
    inner_type: Type  # T en Option<T>
    
    def __init__(self, inner_type: Type):
        self.inner_type = inner_type
```

**Constructores:**
```python
# Some(value)
def Some(value: T) -> Option<T>

# None
def None() -> Option<T>
```

#### 2. **check_option_safety() - Verificador de Option<T>**
```python
def check_option_safety(self, expr) -> void:
    """
    Verifica uso seguro de Option<T>:
    
    1. ❌ PROHIBIDO: null, undefined, nil
    2. ✅ REQUERIDO: Some(value) o None
    3. ✅ REQUERIDO: unwrap, match o if-let antes de usar
    """
```

### Reglas de Option<T>:

#### 1. **❌ PROHIBIDO: null, undefined, nil**
```vela
# ❌ ERROR: null no existe en Vela
# user: User = null

# ❌ ERROR: undefined no existe en Vela
# value: String = undefined

# ❌ ERROR: nil no existe en Vela
# data: Number = nil

# ✅ CORRECTO: usar None
user: Option<User> = None
```

**Type Checker:**
- Rechaza AST nodes con `null`, `undefined`, `nil`
- Error: "null/undefined/nil not allowed, use Option<T> with None"

#### 2. **✅ REQUERIDO: Some(value) o None**
```vela
# Función que puede fallar
fn findUser(id: Number) -> Option<User> {
  if userExists(id) {
    return Some(getUser(id))  # ✅ Envolver en Some
  }
  return None  # ✅ Retornar None
}
```

#### 3. **✅ REQUERIDO: Manejo Explícito**

**Opción 1: Pattern Matching (Recomendado)**
```vela
user: Option<User> = findUser(123)

match user {
  Some(u) => {
    # Aquí u: User (tipo refinado)
    print("Found: ${u.name}")
  }
  None => {
    print("User not found")
  }
}
```

**Opción 2: if-let**
```vela
user: Option<User> = findUser(123)

if let Some(u) = user {
  # u: User (tipo refinado)
  print("Found: ${u.name}")
}
```

**Opción 3: unwrapOr() - Default Value**
```vela
user: User = findUser(123).unwrapOr(defaultUser)
# Si Some(u) → u
# Si None → defaultUser
```

**Opción 4: map/flatMap - Chaining**
```vela
userName: Option<String> = findUser(123)
  .map(u => u.name)
  .map(name => name.toUpperCase())

# Si Some(user) → Some(user.name.toUpperCase())
# Si None → None (propagación)
```

**Opción 5: unwrap() - Unsafe (Solo si estás seguro)**
```vela
user: User = findUser(123).unwrap()
# Si Some(u) → u
# Si None → panic! (crash)
```

⚠️ **ADVERTENCIA:** `unwrap()` puede hacer panic. Usar solo en tests o cuando estés 100% seguro.

#### 4. **❌ ERROR: Usar Option<T> sin unwrap**
```vela
user: Option<User> = findUser(123)

# ❌ ERROR: no puedes usar user directamente
# print(user.name)  // ERROR: Option<User> no tiene campo .name

# ✅ CORRECTO: unwrap primero
match user {
  Some(u) => print(u.name)  # OK
  None => print("N/A")
}
```

## ✅ Criterios de Aceptación
- [x] null/undefined/nil PROHIBIDOS por type checker
- [x] Option<T> como única forma de valores opcionales
- [x] Some(value) y None constructores
- [x] match exhaustivo requerido
- [x] if-let soportado
- [x] unwrapOr() con default value
- [x] map/flatMap para chaining
- [x] unwrap() con advertencia
- [x] Error si Option<T> usado sin unwrap

## 📊 API de Option<T>

### Métodos Principales:

| Método | Firma | Descripción | Ejemplo |
|--------|-------|-------------|---------|
| `Some()` | `(T) -> Option<T>` | Constructor con valor | `Some(42)` |
| `None` | `Option<T>` | Constructor vacío | `None` |
| `isSome()` | `() -> Bool` | Verifica si tiene valor | `opt.isSome()` |
| `isNone()` | `() -> Bool` | Verifica si está vacío | `opt.isNone()` |
| `unwrap()` | `() -> T` | Extrae valor (panic si None) | `opt.unwrap()` |
| `unwrapOr()` | `(T) -> T` | Extrae o default | `opt.unwrapOr(0)` |
| `map()` | `((T) -> U) -> Option<U>` | Transforma valor | `opt.map(x => x * 2)` |
| `flatMap()` | `((T) -> Option<U>) -> Option<U>` | Chaining | `opt.flatMap(parse)` |
| `filter()` | `((T) -> Bool) -> Option<T>` | Filtra valor | `opt.filter(x => x > 0)` |
| `and()` | `(Option<U>) -> Option<U>` | AND lógico | `opt1.and(opt2)` |
| `or()` | `(Option<T>) -> Option<T>` | OR lógico | `opt1.or(opt2)` |

### Ejemplos Completos:

```vela
# Ejemplo 1: findUser con manejo seguro
fn getUserName(id: Number) -> String {
  user: Option<User> = findUser(id)
  return user
    .map(u => u.name)
    .unwrapOr("Unknown")
}

# Ejemplo 2: Chaining con flatMap
fn getUserEmail(id: Number) -> Option<String> {
  return findUser(id)
    .flatMap(u => u.email)  # u.email: Option<String>
    .filter(email => email.contains("@"))
}

# Ejemplo 3: Combinación con and/or
fn getPreferredName(id: Number) -> Option<String> {
  nickname: Option<String> = getNickname(id)
  fullName: Option<String> = getFullName(id)
  
  return nickname.or(fullName)  # nickname si existe, sino fullName
}
```

## 🧪 Tests Implementados

```python
class TestOptionSafety:
    def test_option_type_creation()
        # Some(value) y None
    
    def test_make_optional()
        # Convertir T a Option<T>
    
    def test_get_inner_type()
        # Extraer T de Option<T>
    
    def test_option_unification()
        # Option<Number> con Option<T>
    
    def test_null_prohibited()
        # Error si se usa null
    
    def test_unwrap_required()
        # Error si se usa Option<T> sin unwrap
```

**Total:** 6+ tests de Option<T> safety

## 💡 Decisiones de Diseño

### 1. **Option<T> vs null: Explícito > Implícito**
```vela
# ❌ Otros lenguajes (implícito, inseguro)
# user: User? = findUser(123)
# print(user.name)  // NPE si user == null

# ✅ Vela (explícito, seguro)
user: Option<User> = findUser(123)
match user {
  Some(u) => print(u.name)  # Safe
  None => print("N/A")
}
```

### 2. **Type Refinement en Pattern Matching**
Dentro de `Some(u)`, el tipo es refinado de `Option<User>` a `User`:
```vela
user: Option<User> = findUser(123)

match user {
  Some(u) => {
    # Aquí: u: User (NO Option<User>)
    u.name  # OK
  }
  None => { }
}
```

### 3. **Chaining Funcional**
Option<T> es un functor/monad:
```vela
result: Option<String> = findUser(123)
  .map(u => u.name)           # Option<User> -> Option<String>
  .filter(name => name.len() > 0)  # Filtro
  .map(name => name.toUpperCase())  # Transformación

# Si cualquier paso falla → None propagado
```

### 4. **unwrap() Desaconsejado**
Preferir `unwrapOr()` o pattern matching:
```vela
# ⚠️ Desaconsejado (puede hacer panic)
user: User = findUser(123).unwrap()

# ✅ Preferir unwrapOr
user: User = findUser(123).unwrapOr(defaultUser)

# ✅ O pattern matching
match findUser(123) {
  Some(u) => handleUser(u)
  None => handleNotFound()
}
```

### 5. **Composición con Result<T, E>**
```vela
# Option<T> para "valor opcional"
fn findUser(id: Number) -> Option<User>

# Result<T, E> para "operación que puede fallar con error"
fn loadUser(id: Number) -> Result<User, DatabaseError>

# Conversión: Result -> Option
loadUser(123).ok()  # Ok(user) -> Some(user), Err(_) -> None
```

## 🔗 Referencias
- **Código:** `src/type_system/types.rs` (OptionType)
- **Código:** `src/type_system/checker.rs` (check_option_safety)
- **Tests:** `tests/unit/type_system/test_type_system.py` (TestOptionSafety)
- **Historia:** [VELA-570](https://velalang.atlassian.net/browse/VELA-570)
- **Especificación:** `.github/copilot-instructions.md` (Option<T> en lugar de null)
- **Inspiración:** Rust's Option<T>, Haskell's Maybe
