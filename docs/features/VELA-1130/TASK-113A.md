# TASK-113A: Implementar API de Testing (describe/it/expect)

## 📋 Información General
- **Historia:** VELA-1130
- **Estado:** Completada ✅
- **Fecha:** 2025-12-14

## 🎯 Objetivo
Implementar la API de testing estilo Jest/Mocha con funciones `describe`, `it`, y `expect` para crear tests expresivos y legibles.

## 🔨 Implementación

### API Principal

#### `describe(name: String, fn: () -> void)`
Agrupa tests relacionados en un bloque descriptivo.

```vela
describe("Math operations", () => {
  // Tests relacionados con matemáticas van aquí
})
```

#### `it(description: String, fn: () -> void)`
Define un test individual.

```vela
it("should add two numbers correctly", () => {
  let result = add(2, 3)
  expect(result).toBe(5)
})
```

#### `expect(value: any)`
Crea una assertion que se puede encadenar con matchers.

```vela
expect(actualValue).toBe(expectedValue)
expect(array).toContain(item)
expect(result).toBeTruthy()
```

### Matchers Disponibles

#### Igualdad
- `.toBe(expected)` - Igualdad estricta
- `.toEqual(expected)` - Igualdad profunda
- `.toBeCloseTo(expected, precision?)` - Para números flotantes

#### Verdad/Falsedad
- `.toBeTruthy()` - Valor truthy
- `.toBeFalsy()` - Valor falsy
- `.toBeNull()` - Valor null
- `.toBeUndefined()` - Valor undefined

#### Números
- `.toBeGreaterThan(expected)`
- `.toBeGreaterThanOrEqual(expected)`
- `.toBeLessThan(expected)`
- `.toBeLessThanOrEqual(expected)`

#### Strings
- `.toMatch(pattern)` - Regex matching
- `.toContain(substring)` - Contiene substring

#### Arrays
- `.toContain(item)` - Contiene elemento
- `.toHaveLength(expected)` - Longitud específica
- `.toContainEqual(item)` - Contiene elemento con igualdad profunda

#### Objetos
- `.toHaveProperty(path, value?)` - Tiene propiedad
- `.toMatchObject(expected)` - Coincide parcialmente con objeto

#### Errores
- `.toThrow(error?)` - Lanza excepción
- `.toThrowError(error?)` - Lanza error específico

### Ejemplo Completo

```vela
import { describe, it, expect } from 'testing'

describe("Calculator", () => {
  describe("Addition", () => {
    it("should add positive numbers", () => {
      expect(add(2, 3)).toBe(5)
      expect(add(10, 15)).toBe(25)
    })

    it("should handle zero", () => {
      expect(add(0, 5)).toBe(5)
      expect(add(5, 0)).toBe(5)
    })
  })

  describe("Subtraction", () => {
    it("should subtract numbers", () => {
      expect(subtract(10, 3)).toBe(7)
      expect(subtract(5, 10)).toBe(-5)
    })
  })

  describe("Error handling", () => {
    it("should throw on division by zero", () => {
      expect(() => divide(10, 0)).toThrow("Division by zero")
    })
  })
})
```

## ✅ Criterios de Aceptación
- [x] `describe` agrupa tests correctamente
- [x] `it` define tests individuales
- [x] `expect` crea assertions
- [x] Todos los matchers funcionan
- [x] Mensajes de error descriptivos
- [x] Sintaxis compatible con Jest/Mocha

## 🔗 Referencias
- **Jira:** [TASK-113A](https://velalang.atlassian.net/browse/TASK-113A)
- **Historia:** [VELA-1130](https://velalang.atlassian.net/browse/VELA-1130)
- **Código:** `src/testing/api.vela`