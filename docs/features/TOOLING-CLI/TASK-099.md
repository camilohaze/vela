# TASK-099: Implementar vela test

## 📋 Información General
- **Historia:** TOOLING-CLI
- **Estado:** Completada ✅
- **Fecha:** 2025-12-10

## 🎯 Objetivo
Implementar el comando `vela test` para ejecutar tests en archivos `.spec.vela` siguiendo el patrón de Angular/NestJS.

## 🔨 Implementación

### Comando `vela test`
- **Ubicación:** `tooling/src/cli/commands.rs::execute_test()`
- **Funcionalidad:**
  - Busca archivos `.spec.vela` en directorios que contengan 'tests'
  - Compila cada archivo usando `vela_compiler`
  - Ejecuta el bytecode usando `vela_vm`
  - Reporta resultados con formato paso/falla

### Sintaxis de Test Files
Los archivos de test deben tener extensión `.spec.vela` y usar el framework de test definido en `stdlib/test.vela`:

```vela
import 'stdlib:test'

fn test_example() -> void {
    result: Number = 2 + 3
    assert(result == 5, "2 + 3 should equal 5")
}

run_test(test_example, "test_example")
report_results()
```

### Framework de Test
- **Ubicación:** `stdlib/test.vela`
- **Funciones:**
  - `assert(condition: Bool, message: String)`: Verifica condición
  - `run_test(test_fn: () -> void, name: String)`: Ejecuta test individual
  - `report_results()`: Muestra resumen final

## ✅ Criterios de Aceptación
- [x] Comando `vela test` implementado
- [x] Busca archivos `.spec.vela` correctamente
- [x] Compila y ejecuta tests usando compiler/VM
- [x] Reporta resultados con formato ✅/❌
- [x] Tests unitarios actualizados para usar `.spec.vela`
- [x] Documentación completa

## 🔗 Referencias
- **Jira:** [TASK-099](https://velalang.atlassian.net/browse/TASK-099)
- **Código:** `tooling/src/cli/commands.rs`
- **Tests:** `tests/unit/test_cli_test.rs`
- **Framework:** `stdlib/test.vela`