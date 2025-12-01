# TASK-016: Type Checking de Statements

## 📋 Información General
- **Historia:** VELA-570
- **Estado:** ✅ Completada
- **Fecha:** 2025-12-01

## 🎯 Objetivo
Implementar verificación de tipos para todos los statements de Vela, incluyendo declaraciones, control de flujo y prohibición de loops imperativos.

## 🔨 Implementación

### Archivos generados:
- `src/type_system/checker.rs` - TypeChecker (parcial, ~100 líneas para statements)

### Componentes Principales:

#### 1. **check_statement() - Verificador de Statements**
```python
def check_statement(self, stmt) -> void:
    """
    Verifica el tipo de un statement.
    
    Tipos de statements manejados:
    - Variable declaration (inmutable y state)
    - If statement
    - Expression statement
    - Return statement
    - Function declaration
    - Class declaration
    
    ❌ PROHIBIDO: for, while, loop (paradigma funcional)
    """
```

### Tipos de Statements:

#### 1. **Variable Declaration (Inmutable)**
```python
name: String = "Vela"
age: Number = 37
PI: Float = 3.14159
```

**Verificaciones:**
1. Si hay anotación de tipo, verificar que coincida con el valor
2. Si NO hay anotación, inferir tipo del valor
3. Agregar al TypeEnvironment como inmutable
4. Error si variable ya existe en scope actual

**Proceso:**
```python
# Con anotación explícita
name: String = "Vela"
# 1. Inferir tipo de "Vela" → String
# 2. Unificar String con String → OK
# 3. Define Symbol(name="name", type=String, mutable=False)

# Sin anotación (inferencia)
x = 42
# 1. Inferir tipo de 42 → Number
# 2. Define Symbol(name="x", type=Number, mutable=False)
```

#### 2. **State Variable Declaration (Mutable)**
```python
state count: Number = 0
state isActive: Bool = true
```

**Verificaciones:**
1. SOLO state puede ser mutable
2. Verificar tipo como variable inmutable
3. Agregar al TypeEnvironment con `mutable=True`
4. Permite reasignación posterior

**Proceso:**
```python
state count: Number = 0
# 1. Inferir tipo de 0 → Number
# 2. Unificar Number con Number → OK
# 3. Define Symbol(name="count", type=Number, mutable=True)

# Reasignación permitida
count = count + 1  # OK porque count es mutable
```

#### 3. **If Statement**
```python
if condition {
  # then branch
} else {
  # else branch
}
```

**Verificaciones:**
1. Condición DEBE ser Bool
2. Verificar statements de then branch
3. Verificar statements de else branch (si existe)
4. Type narrowing aplicado en ramas (ver TASK-019)

**Ejemplo:**
```python
x: Number = 10
if x > 5 {
  print("big")  # OK
}
```

#### 4. **Expression Statement**
```python
print("hello")
calculate(10, 20)
```

**Verificaciones:**
1. Verificar tipo de la expresión
2. Ignorar tipo de retorno (no se usa)

#### 5. **Return Statement**
```python
fn add(a: Number, b: Number) -> Number {
  return a + b  # Type checked: Number
}
```

**Verificaciones:**
1. Inferir tipo de expresión retornada
2. Unificar con tipo de retorno declarado de función
3. Error si no coinciden

#### 6. **Function Declaration**
```python
fn greet(name: String) -> void {
  print("Hello, ${name}")
}
```

**Verificaciones:**
1. Crear nuevo scope para parámetros
2. Agregar parámetros al environment
3. Verificar body de función
4. Verificar que todos los paths retornen el tipo correcto
5. Agregar función al environment

#### 7. **❌ PROHIBIDO: Loops Imperativos**
```python
# ❌ ERROR: for no existe en Vela
# for i in 0..10 { print(i) }

# ✅ CORRECTO: métodos funcionales
(0..10).forEach(i => print(i))

# ❌ ERROR: while no existe en Vela
# while condition { doSomething() }

# ✅ CORRECTO: recursión
fn repeatUntil(condition: () -> Bool) {
  if !condition() {
    doSomething()
    repeatUntil(condition)
  }
}
```

**Verificación:**
- TypeChecker rechaza AST nodes de tipo ForLoop, WhileLoop
- Error: "for/while loops not allowed in functional Vela"

## ✅ Criterios de Aceptación
- [x] Variable declaration con type checking
- [x] State variables identificadas como mutables
- [x] If statements con condición Bool
- [x] Expression statements verificados
- [x] Return statements con unificación
- [x] Function declarations con scope correcto
- [x] Loops imperativos PROHIBIDOS
- [x] Error reporting claro

## 📊 Tabla de Statements

| Statement | Sintaxis | Type Checking | Ejemplo |
|-----------|----------|---------------|---------|
| **Variable inmutable** | `name: Type = value` | Unificar type con valor | `age: Number = 37` |
| **Variable inferida** | `name = value` | Inferir tipo de valor | `x = 42` |
| **State mutable** | `state name: Type = value` | Marcar mutable | `state count: Number = 0` |
| **If statement** | `if cond { ... }` | Condición Bool | `if x > 5 { ... }` |
| **Return** | `return expr` | Unificar con retorno fn | `return x + y` |
| **Function** | `fn name(...) -> T { }` | Verificar body | `fn add(a, b) -> Number` |
| **Expression** | `expr` | Verificar expr | `print("hello")` |

## 🧪 Tests Implementados

```python
class TestTypeChecker:
    def test_variable_declaration()
    def test_state_variable_mutability()
    def test_if_statement_bool_condition()
    def test_return_type_checking()
    def test_function_declaration()
    def test_loops_prohibited()
```

**Total:** 6+ tests de statements

## 💡 Decisiones de Diseño

### 1. **Inmutabilidad por Defecto**
Sin keyword → inmutable:
```python
x: Number = 10
x = 20  # ERROR: x es inmutable
```

Con `state` → mutable:
```python
state x: Number = 10
x = 20  # OK: x es mutable
```

### 2. **Shadowing Permitido**
```python
x: Number = 10
{
  x: String = "hello"  # OK: nueva variable (shadowing)
  print(x)  # "hello"
}
print(x)  # 10
```

### 3. **Function Scope Aislado**
Parámetros y variables locales de función están en scope separado:
```python
fn test(param: Number) -> Number {
  local: Number = param * 2
  return local
}
# param y local no visibles aquí
```

### 4. **Return Type Checking Estricto**
Todos los paths deben retornar el tipo declarado:
```python
fn divide(a: Number, b: Number) -> Option<Float> {
  if b == 0 {
    return None  # OK: Option<Float>
  }
  return Some(a / b)  # OK: Option<Float>
}
```

### 5. **No Loops = Funcional Puro**
Forzar paradigma funcional:
- ❌ `for`, `while`, `loop` → ERROR en parser/checker
- ✅ `.map()`, `.filter()`, `.forEach()` → OK
- ✅ Recursión → OK

## 🔗 Referencias
- **Código:** `src/type_system/checker.rs` (check_statement, check_var_declaration, check_if_statement)
- **Tests:** `tests/unit/type_system/test_type_system.py` (TestTypeChecker)
- **Historia:** [VELA-570](https://velalang.atlassian.net/browse/VELA-570)
- **Especificación:** `.github/copilot-instructions.md` (paradigma funcional)
