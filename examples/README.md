# Vela Testing Examples

Este directorio contiene ejemplos de **specs** escritas en el lenguaje Vela, utilizando el framework de testing BDD que hemos implementado.

## 📋 Estado Actual del Runtime

⚠️ **IMPORTANTE**: Actualmente, el runtime completo de Vela aún no está implementado. Estos archivos de ejemplo muestran cómo se verán y funcionarán los tests cuando el lenguaje esté completamente operativo.

### 🚧 Qué falta para ejecutar estos tests:

1. **Compilador funcional**: El compilador tiene algunos errores de compilación que necesitan ser corregidos
2. **VM operativa**: La máquina virtual necesita completar su implementación
3. **Runtime system**: El sistema de runtime con GC, concurrencia, etc.
4. **Biblioteca estándar**: Implementación completa de tipos y funciones built-in

### ✅ Qué ya está implementado:

- ✅ **Framework de testing BDD** (decoradores `@describe`, `@it`, `@test`, etc.)
- ✅ **Sintaxis del lenguaje** definida y especificada
- ✅ **Parser y AST** parcialmente implementados
- ✅ **Sistema de tipos** en desarrollo
- ✅ **IR y bytecode generation** en progreso

## 🧪 Ejemplos de Specs

### 1. `basic-spec.vela`
Tests fundamentales que cubren:
- Aserciones básicas (`assert_eq`, `assert`)
- Operaciones matemáticas
- Manipulación de strings
- Operaciones con colecciones (List, Set, Map)
- Control de flujo (if, match, loops)
- Manejo de errores (Result, Option, try-catch)
- Operaciones asíncronas

### 2. `calculator-spec.vela`
Tests más avanzados que demuestran:
- Tests organizados jerárquicamente con `@describe`
- Funciones helper y utilidades de test
- Testing de lógica de negocio compleja
- Manejo de colecciones funcionales (map, filter, reduce)
- Testing de operaciones asíncronas

### 3. `reactive-ui-spec.vela`
Tests de UI y programación reactiva:
- Signals y valores computados
- Componentes UI reactivos
- Manejo de estado complejo
- Validación de formularios
- Testing de flujos de trabajo integrados

## 🚀 Cómo ejecutar cuando esté listo

Cuando el runtime de Vela esté completamente implementado, podrás ejecutar estos tests con:

```bash
# Ejecutar todos los tests
vela test

# Ejecutar tests específicos
vela test basic-spec.vela
vela test calculator-spec.vela

# Ejecutar con filtro
vela test --filter "Calculator"

# Ejecutar con modo verbose
vela test --verbose
```

## 📖 Sintaxis de Testing en Vela

### Estructura Básica

```vela
@describe("Suite de tests")
module MySpec {

    @describe("Sub-suite")
    module SubSpec {

        @it("debería hacer algo específico")
        fn test_something() -> void {
            // Arrange
            let expected = 42

            // Act
            let actual = some_function()

            // Assert
            assert_eq(actual, expected, "La función debería retornar 42")
        }
    }
}
```

### Decoradores Disponibles

| Decorador | Propósito | Ejemplo |
|-----------|-----------|---------|
| `@describe` | Agrupa tests relacionados | `@describe("Math Operations")` |
| `@it` | Define un caso de test | `@it("should add numbers")` |
| `@test` | Test unitario simple | `@test fn my_test() { ... }` |
| `@beforeAll` | Setup global | `@beforeAll fn setup() { ... }` |
| `@afterAll` | Cleanup global | `@afterAll fn teardown() { ... }` |
| `@beforeEach` | Setup por test | `@beforeEach fn setup() { ... }` |
| `@afterEach` | Cleanup por test | `@afterEach fn teardown() { ... }` |

### Funciones de Aserción

```vela
// Aserción de igualdad
assert_eq(actual, expected, "Mensaje descriptivo")

// Aserción de condición
assert(condition, "Mensaje si falla")

// Aserciones para Option
assert_eq(some_value, Some(expected), "Debería tener un valor")
assert_eq(none_value, None, "Debería ser None")

// Aserciones para Result
match result {
    Ok(value) => assert_eq(value, expected, "Valor correcto")
    Err(error) => assert(false, "No debería fallar")
}
```

### Testing Asíncrono

```vela
@it("debería manejar operaciones async")
async fn test_async_operation() -> void {
    let result = await some_async_function()
    assert(result.is_ok(), "La operación async debería tener éxito")
}

@it("debería manejar múltiples promises")
async fn test_multiple_promises() -> void {
    let promise1 = async { return "hello" }
    let promise2 = async { return "world" }

    let results = await Promise.all([promise1, promise2])
    assert_eq(results, ["hello", "world"], "Todas las promises deberían resolverse")
}
```

## 🎯 Beneficios del Framework BDD en Vela

### 1. **Sintaxis Declarativa**
- Tests que se leen como especificaciones
- `@describe` y `@it` crean documentación viva
- Nombres descriptivos en lugar de `test_function_name`

### 2. **Organización Jerárquica**
- Suites anidadas con `@describe`
- Tests agrupados por funcionalidad
- Fácil navegación y mantenimiento

### 3. **Reactividad Integrada**
- Tests pueden usar signals y computed values
- Verificación de actualizaciones reactivas
- Testing de UI components reactivos

### 4. **Type Safety**
- Aserciones type-safe
- Verificación de tipos en compile-time
- Option/Result handling idiomático

### 5. **Async/Await Nativo**
- Testing de operaciones asíncronas
- Manejo de Promises integrado
- Sin necesidad de callbacks complejos

## 🔄 Próximos Pasos

Para poder ejecutar estos tests, necesitamos:

1. **Corregir errores del compilador** en `compiler/src/`
2. **Completar la implementación de la VM** en `vm/src/`
3. **Implementar el runtime system** en `runtime/`
4. **Crear el ejecutable `vela`** que compile y ejecute código Vela
5. **Integrar el framework de testing** con el runtime

Una vez completado esto, Vela tendrá un sistema de testing moderno y poderoso que combina lo mejor de lenguajes como Jest, RSpec, y frameworks de testing funcionales.