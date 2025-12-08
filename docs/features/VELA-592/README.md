# US-22: CLI tooling para gestión de proyectos Vela

## 📋 Información General
- **Epic:** Sprint 29
- **Estado:** En progreso ⏳
- **Fecha:** 2025-01-07

## 🎯 Descripción
Como desarrollador, quiero un CLI completo para gestionar proyectos Vela que incluya comandos para crear, construir, ejecutar, probar y mantener proyectos.

## 📦 Subtasks Completadas
1. **TASK-096**: Implementar vela create ✅
   - Comando `vela create <name> --template <type>`
   - 5 templates: web, cli, lib, api, module
   - Estructura completa de proyecto
   - Tests unitarios incluidos

2. **TASK-097**: Implementar vela build ✅
   - Comando `vela build <input> --output <file>`
   - Compilación completa con optimizaciones
   - Manejo de errores y warnings
   - Tests unitarios incluidos

3. **TASK-098**: Implementar vela run ✅
   - Comando `vela run <file.velac> --trace --gc-stats`
   - Ejecución de bytecode en VM
   - Opciones de debugging
   - Tests unitarios incluidos

4. **TASK-099**: Implementar vela test ✅
   - Comando `vela test [--verbose] [--filter <pattern>] [--time] [files...]`
   - Descubrimiento automático de tests
   - Reporte detallado de resultados
   - Tests unitarios incluidos

## 📋 Subtasks Pendientes
5. **TASK-100**: Implementar vela fmt (P1)
6. **TASK-101**: Implementar vela doctor (P2)

## 🔨 Comandos Implementados

### ✅ vela create
```bash
vela create my-project --template web
```

**Templates disponibles:**
- `web` - Aplicación web reactiva
- `cli` - Herramienta de línea de comandos
- `lib` - Librería reutilizable
- `api` - API REST con endpoints
- `module` - Módulo funcional

### 🔄 vela run (parcialmente implementado)
```bash
vela run my-app.velac
```

Ejecuta bytecode .velac con VM integrada.

### ✅ vela test
```bash
vela test [--verbose] [--filter <pattern>] [--time] [files...]
```

Ejecuta tests de Vela con opciones avanzadas:
- `--verbose`: Output detallado
- `--filter <pattern>`: Filtrar por nombre de archivo
- `--time`: Mostrar tiempos de ejecución
- `files...`: Archivos específicos (opcional, busca en `tests/` por defecto)

## 📊 Métricas
- **Subtasks completadas:** 4/6 (66.7%)
- **Archivos creados:** ~25
- **Líneas de código:** ~2500
- **Templates:** 5
- **Comandos CLI:** 4/6 implementados
- **Tests:** Básicos incluidos

## ✅ Definición de Hecho
- [x] TASK-096 completada con templates funcionales
- [x] TASK-097: build command implementado
- [x] TASK-098: run command mejorado
- [x] TASK-099: test runner implementado
- [ ] TASK-100: code formatter implementado
- [ ] TASK-101: diagnostic tool implementado
- [ ] Todos los comandos probados e integrados
- [ ] Documentación completa generada

## 🔗 Referencias
- **Jira:** [US-22](https://velalang.atlassian.net/browse/US-22)
- **Arquitectura:** CLI basado en Clap con comandos jerárquicos