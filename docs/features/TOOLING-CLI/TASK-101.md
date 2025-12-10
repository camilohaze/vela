# TASK-101: Implementar vela doctor

## 📋 Información General
- **Historia:** TOOLING-CLI
- **Estado:** Completada ✅
- **Fecha:** 2025-12-10

## 🎯 Objetivo
Implementar el comando `vela doctor` para diagnosticar la instalación y configuración de Vela.

## 🔨 Implementación

### Comando `vela doctor`
- **Ubicación:** `tooling/src/cli/commands.rs::execute_doctor()`
- **Funcionalidad:**
  - Verifica la versión de Vela instalada
  - Detecta si estamos en un proyecto Vela
  - Cuenta archivos fuente .vela
  - Verifica permisos de escritura
  - Comprueba directorios de build
  - Reporta problemas críticos y warnings

### Diagnósticos Realizados
- **Versión de Vela:** Muestra versión actual
- **Directorio actual:** Verifica accesibilidad
- **Detección de proyecto:** Busca archivos vela.toml, Cargo.toml, package.json
- **Archivos fuente:** Cuenta archivos .vela encontrados
- **Directorio de build:** Verifica target/
- **Permisos:** Prueba escritura en directorio actual
- **Variables de entorno:** Verifica HOME/USERPROFILE

### Función `find_vela_files()`
- **Ubicación:** `tooling/src/cli/commands.rs::find_vela_files()`
- **Funcionalidad:**
  - Búsqueda recursiva de archivos .vela
  - Excluye directorios comunes (target, node_modules, .git)
  - Reutilizada del comando fmt

## ✅ Criterios de Aceptación
- [x] Comando `vela doctor` implementado
- [x] Diagnóstico completo de instalación
- [x] Detección de proyectos Vela
- [x] Verificación de permisos y configuración
- [x] Reporte claro de problemas encontrados
- [x] Tests unitarios completos
- [x] Documentación completa

## 🔗 Referencias
- **Jira:** [TASK-101](https://velalang.atlassian.net/browse/TASK-101)
- **Código:** `tooling/src/cli/commands.rs`
- **Tests:** `tests/unit/test_cli_test.rs`
- **Documentación:** `docs/features/TOOLING-CLI/TASK-101.md`