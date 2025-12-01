# TASK-015: Type Checking de Expresiones

## 📋 Información General
- **Historia:** VELA-570
- **Estado:** ✅ Completada
- **Fecha:** 2025-12-01

## 🎯 Objetivo
Implementar verificación de tipos para todas las expresiones de Vela, garantizando type safety en operaciones.

## 🔨 Implementación

### Archivos generados:
- `src/type_system/checker.rs` - TypeChecker (parcial, ~150 líneas para expresiones)

### Componentes Principales:

#### 1. **check_expression() - Verificador de Expresiones**
```python
def check_expression(self, expr, expected_type=None) -> Type:
    """
    Verifica el tipo de una expresión.
    
    Tipos de expresiones manejadas:
    - Literales (números, strings, booleanos)
    - Variables (con lookup en environment)
    - Operaciones binarias (+, -, *, /, ==, <, etc.)
    - Llamadas a función
    - Acceso a campos (obj.field)
    - Expresiones if (ternario)
    - Match expressions
    """
```

### Tipos de Expresiones:

#### 1. **Literales**
```python
# Números enteros
42 → Number
-17 → Number

# Flotantes
3.14 → Float
-0.5 → Float

# Strings
"hello" → String

# Booleanos
true → Bool
false → Bool
```

#### 2. **Variables**
```python
name: String = "Vela"
x = name  # Type checked: x: String
```

**Proceso:**
1. Lookup en TypeEnvironment
2. Retornar tipo del Symbol
3. Error si variable no definida

#### 3. **Operaciones Binarias**
```python
# Aritméticas: +, -, *, /, %
x: Number = 10
y: Number = 5
result = x + y  # Type checked: Number

# Comparación: ==, !=, <, >, <=, >=
x: Number = 10
y: Number = 5
result = x < y  # Type checked: Bool

# Lógicas: and, or
a: Bool = true
b: Bool = false
result = a and b  # Type checked: Bool
```

**Implementación:**
- `check_binary_op(expr)` delega a operador específico
- Verifica tipos de operandos
- Retorna tipo de resultado

#### 4. **Llamadas a Función**
```python
fn add(a: Number, b: Number) -> Number {
  return a + b
}

result = add(10, 20)  # Type checked: Number
```

**Verificaciones:**
- Función debe tener tipo FunctionType
- Número de argumentos == número de parámetros
- Cada argumento unifica con parámetro correspondiente
- Resultado tiene tipo de retorno de función

#### 5. **Expresiones if (Ternario)**
```python
x: Number = 10
result = if x > 5 { "big" } else { "small" }
# Type checked: String
```

**Verificaciones:**
- Condición debe ser Bool
- Ambas ramas deben tener el mismo tipo (o unificar)
- Resultado tiene tipo unificado

#### 6. **Match Expressions**
```python
match result {
  Ok(value) => value
  Err(error) => 0
}
```

**Verificaciones:**
- Valor matched debe ser enum/Result/Option
- Cada patrón debe ser exhaustivo
- Todos los brazos deben retornar mismo tipo

## ✅ Criterios de Aceptación
- [x] Literales inferidos correctamente (Number, Float, String, Bool)
- [x] Variables con lookup en environment
- [x] Operaciones aritméticas type-checked (+, -, *, /, %)
- [x] Operaciones de comparación type-checked (==, <, >, etc.)
- [x] Operaciones lógicas type-checked (and, or)
- [x] Llamadas a función verificadas (aridad + tipos)
- [x] If expressions con type checking de ramas
- [x] Match expressions con exhaustividad
- [x] Error reporting claro

## 📊 Tabla de Operadores

| Operador | Tipos de Operandos | Tipo de Resultado | Ejemplo |
|----------|-------------------|-------------------|---------|
| `+` | Number, Number | Number | `10 + 5` → `15` |
| `+` | String, String | String | `"a" + "b"` → `"ab"` |
| `-` | Number, Number | Number | `10 - 5` → `5` |
| `*` | Number, Number | Number | `10 * 5` → `50` |
| `/` | Number, Number | Float | `10 / 3` → `3.333...` |
| `%` | Number, Number | Number | `10 % 3` → `1` |
| `==` | T, T | Bool | `10 == 10` → `true` |
| `!=` | T, T | Bool | `10 != 5` → `true` |
| `<` | Number, Number | Bool | `5 < 10` → `true` |
| `>` | Number, Number | Bool | `10 > 5` → `true` |
| `<=` | Number, Number | Bool | `5 <= 5` → `true` |
| `>=` | Number, Number | Bool | `10 >= 5` → `true` |
| `and` | Bool, Bool | Bool | `true and false` → `false` |
| `or` | Bool, Bool | Bool | `true or false` → `true` |

## 🧪 Tests Implementados

```python
class TestTypeChecker:
    def test_literal_inference()
    def test_arithmetic_operations()
    def test_comparison_operations()
    def test_logical_operations()
    def test_type_error_detection()
```

**Total:** 8+ tests de type checking de expresiones

## 💡 Decisiones de Diseño

### 1. **Type Coercion Mínimo**
Solo se permite coerción implícita segura:
- Number NO se convierte automáticamente a Float
- Requiere conversión explícita: `x.toFloat()`

### 2. **String Concatenation con +**
```python
"hello" + " world"  # OK: String
"hello" + 123       # ERROR: no coerción automática
```

### 3. **Comparación Solo de Tipos Compatibles**
```python
10 == 10    # OK
10 == "10"  # ERROR: Number != String
```

### 4. **Division Retorna Float**
```python
10 / 3  # → Float (3.333...)
```

Para división entera, usar `//`:
```python
10 // 3  # → Number (3)
```

### 5. **Error Accumulation**
El TypeChecker acumula errores en lugar de fallar al primero:
- Permite reportar múltiples errores de tipo
- Mejor experiencia de desarrollo

## 🔗 Referencias
- **Código:** `src/type_system/checker.rs` (check_expression, check_binary_op, check_call)
- **Tests:** `tests/unit/type_system/test_type_system.py` (TestTypeChecker)
- **Historia:** [VELA-570](https://velalang.atlassian.net/browse/VELA-570)
