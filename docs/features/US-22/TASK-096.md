# TASK-096: Implementar vela create

## 📋 Información General
- **Historia:** US-22
- **Estado:** En curso ⏳
- **Fecha:** 2025-01-07

## 🎯 Objetivo
Implementar el comando `vela create` para crear nuevos proyectos Vela con templates predefinidos.

## 🔨 Implementación

### Comando vela create
```bash
vela create <project-name> [template]
```

**Templates disponibles:**
- `web` - Aplicación web con UI reactiva
- `cli` - Aplicación de línea de comandos
- `lib` - Librería reutilizable
- `api` - API REST con endpoints HTTP
- `module` - Módulo funcional

### Archivos generados
- `src/main.vela` - Punto de entrada principal
- `vela.toml` - Configuración del proyecto
- `README.md` - Documentación del proyecto
- `tests/` - Estructura de tests
- `docs/` - Documentación adicional

## ✅ Criterios de Aceptación
- [x] Comando `vela create` implementado
- [x] Templates básicos funcionando (web, cli, lib)
- [x] Estructura de proyecto correcta
- [x] Tests unitarios para el comando
- [x] Documentación generada

## 🔗 Referencias
- **Jira:** [TASK-096](https://velalang.atlassian.net/browse/TASK-096)
- **Historia:** [US-22](https://velalang.atlassian.net/browse/US-22)