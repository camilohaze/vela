# TASK-098: Implementar vela run

## 📋 Información General
- **Historia:** VELA-592 (US-22: CLI para gestionar proyectos)
- **Estado:** Completada ✅ (Funcionalidad extendida implementada)
- **Fecha:** 2025-12-07
- **Nota:** Incluye funcionalidad extendida para archivos fuente .vela

## 🎯 Objetivo
Implementar el comando `vela run` para ejecutar proyectos Vela, soportando tanto archivos fuente (.vela) con compilación automática como archivos bytecode (.velac) precompilados.

## 🔨 Implementación

### Comando `vela run`
```bash
vela run <archivo.vela|.velac> [args...] [opciones]
```

### Opciones implementadas:
- `--trace`: Muestra trace de ejecución de VM (debug)
- `--gc-stats`: Muestra estadísticas de garbage collection

### Funcionalidades:
1. **Detección automática de tipo de archivo**:
   - `.vela`: Compilación on-the-fly + ejecución
   - `.velac`: Carga directa de bytecode + ejecución

2. **Compilación automática**: Para archivos .vela
   - Integración completa con vela-compiler
   - Análisis léxico, sintáctico y generación de bytecode
   - Mensajes de error detallados

3. **Ejecución de bytecode**: Para archivos .velac
   - Deserialización eficiente de bytecode
   - Ejecución directa en VirtualMachine

4. **Manejo de argumentos**: Pasa argumentos de CLI al programa
5. **Debugging**: Opcional disassembly y trace de ejecución
6. **Estadísticas**: Tiempo de ejecución y estadísticas de GC

### Archivos generados
- Código implementado en `cli/src/main.rs` (función `handle_run` extendida)
- Tests unitarios en `cli/src/test_cli_run.rs`
- Documentación actualizada en `docs/features/VELA-592/TASK-098.md`

## ✅ Criterios de Aceptación
- [x] Comando `vela run archivo.vela` compila y ejecuta automáticamente
- [x] Comando `vela run archivo.velac` ejecuta bytecode directamente
- [x] Detección automática de tipo de archivo por extensión
- [x] Manejo de errores para archivos inexistentes o inválidos
- [x] Soporte para argumentos de línea de comandos
- [x] Opción `--trace` muestra disassembly y trace de VM
- [x] Opción `--gc-stats` muestra estadísticas de GC
- [x] Tests unitarios completos (3 tests)
- [x] Documentación completa del comando

## 🧪 Tests Implementados
1. `test_run_vela_source_file` - Verifica compilación y ejecución de .vela
2. `test_run_file_not_found` - Manejo de errores de archivo inexistente
3. `test_run_unsupported_file_type` - Rechazo de tipos de archivo no soportados

## 🔄 Estado de Implementación de Proyectos

**Funcionalidad básica implementada**: El comando puede ejecutar archivos individuales .vela y .velac.

**Funcionalidad futura pendiente** (para "ejecución de proyectos completa"):
- Detección automática de entry point en proyectos
- Soporte para archivos de configuración (vela.yaml)
- Gestión de dependencias de proyecto
- Ejecución de suites de test completas
- Modo watch para recarga automática

## 🔗 Referencias
- **Jira:** [TASK-098](https://velalang.atlassian.net/browse/TASK-098)
- **Historia:** [VELA-592](https://velalang.atlassian.net/browse/VELA-592)
- **Dependencias:** TASK-097 (vela build), TASK-074 (tests VelaVM)
- **Código:** `cli/src/main.rs` (función `handle_run`)
- **Tests:** `cli/src/test_cli_run.rs`