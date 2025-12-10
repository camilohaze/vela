# TASK-112: Implementar textDocument/rename

## 📋 Información General
- **Historia:** VELA-561
- **Estado:** En curso 🚧
- **Fecha:** 2025-01-30

## 🎯 Objetivo
Implementar el método `textDocument/rename` del LSP para proporcionar funcionalidad de renombrado refactor automático de símbolos en archivos Vela.

## 🔨 Implementación

### Funciones a Implementar

#### `analyze_rename_symbol(position: Position, new_name: String) -> Result<WorkspaceEdit>`
Analiza el símbolo en la posición dada y genera un WorkspaceEdit con todos los cambios necesarios para renombrar el símbolo.

#### `find_all_symbol_references(symbol: String, document: &str) -> Vec<Range>`
Encuentra todas las referencias al símbolo en el documento actual.

### Integración con LSP
- **Handler**: `handle_rename` para procesar requests `textDocument/rename`
- **Respuesta**: `WorkspaceEdit` con cambios en múltiples archivos si es necesario
- **Soporte**: Renombrado de variables, funciones, clases, etc.

## ✅ Criterios de Aceptación
- [ ] Renombrado básico de variables locales
- [ ] Renombrado de funciones y métodos
- [ ] Renombrado de clases y tipos
- [ ] Múltiples referencias en el mismo archivo
- [ ] WorkspaceEdit correcto generado
- [ ] Tests unitarios con cobertura completa

## 🔗 Referencias
- **Jira:** [TASK-112](https://velalang.atlassian.net/browse/TASK-112)
- **Historia:** [VELA-561](https://velalang.atlassian.net/browse/VELA-561)
- **LSP Spec:** [textDocument/rename](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_rename)