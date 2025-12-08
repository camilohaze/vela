# TASK-099: Implementar comando vela test

## 📋 Información General
- **Historia:** VELA-592
- **Estado:** Completada ✅
- **Fecha:** 2024-12-30

## 🎯 Objetivo
Implementar el comando `vela test` para ejecutar tests de Vela, incluyendo opciones para filtrado, verbose output y timing.

## 🔨 Implementación

### Comando CLI
```bash
vela test [OPTIONS] [FILES]...
```

### Opciones del Comando
- `--files <FILES>...`: Archivos específicos a testear (opcional)
- `--verbose` / `-v`: Output detallado durante la ejecución
- `--filter <FILTER>`: Filtrar tests por patrón en el nombre del archivo
- `--time` / `-t`: Mostrar tiempos de compilación y ejecución

### Funcionalidad Implementada

#### 1. Descubrimiento de Tests
- Busca automáticamente archivos `.vela` en el directorio `tests/`
- Si no existe `tests/`, muestra error claro
- Soporta archivos específicos como argumentos

#### 2. Compilación y Ejecución
- Compila cada archivo de test usando `vela_compiler`
- Deserializa el bytecode generado
- Ejecuta en la VM de Vela
- Actualmente considera exitoso si la ejecución no falla (placeholder para framework de assertions)

#### 3. Reporte de Resultados
- Muestra progreso en tiempo real
- Reporte final con estadísticas:
  - Tests ejecutados
  - Tests pasados
  - Tests fallidos
- Tiempos de compilación y ejecución (con `--time`)
- Salida detallada (con `--verbose`)

#### 4. Manejo de Errores
- Errores de compilación marcados como fallidos
- Errores de ejecución marcados como fallidos
- Exit code 1 si hay tests fallidos
- Mensajes de error descriptivos

### Archivos Generados
- `cli/src/main.rs`: Función `handle_test()` implementada
- `cli/Cargo.toml`: Dependencia `walkdir` agregada
- `tests/unit/test_cli_test.rs`: Tests unitarios completos

### Dependencias Agregadas
- `walkdir = "2.0"`: Para búsqueda recursiva de archivos de test

## ✅ Criterios de Aceptación
- [x] Comando `vela test` implementado
- [x] Descubrimiento automático de tests en `tests/`
- [x] Opciones `--verbose`, `--filter`, `--time` funcionando
- [x] Tests específicos pueden ser ejecutados
- [x] Reporte de resultados claro y estructurado
- [x] Manejo apropiado de errores de compilación/ejecución
- [x] Tests unitarios con cobertura completa
- [x] Código compila sin errores
- [x] Integración con CLI existente

## 🔗 Referencias
- **Jira:** [TASK-099](https://velalang.atlassian.net/browse/TASK-099)
- **Historia:** [VELA-592](https://velalang.atlassian.net/browse/VELA-592)
- **Dependencias:**
  - TASK-097: Comando `vela build`
  - TASK-098: Comando `vela run`

## 📊 Métricas de Implementación
- **Líneas de código:** ~80 líneas en `handle_test()`
- **Tests unitarios:** 9 tests cubriendo todos los casos
- **Tiempo de desarrollo:** ~2 horas
- **Complejidad:** Media (integración con compiler y VM)

## 🔮 Trabajo Futuro
- Framework de assertions nativo de Vela (`@test`, `assert()`, etc.)
- Tests paralelos para mejor performance
- Cobertura de código
- Integración con IDE (test runner en VS Code)
- Benchmarks y performance tests