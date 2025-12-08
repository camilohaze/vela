# TASK-098: Implementar vela run

## 📋 Información General
- **Historia:** VELA-592 (US-22: CLI para gestionar proyectos)
- **Estado:** Completada ✅
- **Fecha:** 2025-12-07

## 🎯 Objetivo
Implementar el comando `vela run` para ejecutar bytecode compilado (.velac) de proyectos Vela, permitiendo la ejecución de programas con argumentos de línea de comandos, debugging y estadísticas de rendimiento.

## 🔨 Implementación

### Comando `vela run`
```bash
vela run <archivo.velac> [args...] [opciones]
```

### Opciones implementadas:
- `--trace`: Muestra trace de ejecución de VM (debug)
- `--gc-stats`: Muestra estadísticas de garbage collection

### Funcionalidades:
1. **Validación de archivo**: Verifica existencia y extensión .velac
2. **Carga de bytecode**: Deserializa bytecode desde archivo
3. **Ejecución en VM**: Ejecuta bytecode en VirtualMachine
4. **Manejo de argumentos**: Pasa argumentos de CLI al programa
5. **Debugging**: Opcional disassembly y trace de ejecución
6. **Estadísticas**: Tiempo de ejecución y estadísticas de GC

### Archivos generados
- Código implementado en `cli/src/main.rs` (función `handle_run`)
- Tests unitarios en `tests/unit/test_cli_run.rs`
- Documentación en `docs/features/VELA-592/TASK-098/`

## ✅ Criterios de Aceptación
- [x] Comando `vela run archivo.velac` ejecuta bytecode correctamente
- [x] Manejo de errores para archivos inexistentes o inválidos
- [x] Soporte para argumentos de línea de comandos
- [x] Opción `--trace` muestra disassembly y trace de VM
- [x] Opción `--gc-stats` muestra estadísticas de GC
- [x] Tests unitarios con cobertura >= 80%
- [x] Documentación completa del comando

## 🔗 Referencias
- **Jira:** [TASK-098](https://velalang.atlassian.net/browse/TASK-098)
- **Historia:** [VELA-592](https://velalang.atlassian.net/browse/VELA-592)
- **Dependencias:** TASK-097 (vela build), TASK-074 (tests VelaVM)</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-592\TASK-098\README.md