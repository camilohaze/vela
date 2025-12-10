# TASK-111: Implementar textDocument/publishDiagnostics

## 📋 Información General
- **Historia:** VELA-561
- **Estado:** Completada ✅
- **Fecha:** 2025-01-30

## 🎯 Objetivo
Implementar el método `textDocument/publishDiagnostics` del LSP para proporcionar diagnósticos en tiempo real (errores, warnings, info) en archivos Vela.

## 🔨 Implementación

### Funciones Agregadas

#### `analyze_diagnostics(content: &str, uri: &Url) -> Vec<Diagnostic>`
Analiza el contenido de un documento y detecta:
- **Errores**: Desbalance de llaves/brackets/paréntesis
- **Warnings**: Comentarios TODO, líneas muy largas (>120 caracteres)
- **Info**: Información adicional sobre el código

#### `send_diagnostics(uri: Url, diagnostics: Vec<Diagnostic>) -> Result<()>`
Envía notificaciones `textDocument/publishDiagnostics` al cliente LSP con los diagnósticos encontrados.

### Integración con Handlers
- **`handle_did_open`**: Analiza diagnósticos cuando se abre un documento
- **`handle_did_change`**: Re-analiza diagnósticos cuando cambia el contenido

### Archivos Modificados
- `packages/lsp/src/server.rs`: Agregadas funciones de análisis y envío de diagnósticos
- `packages/lsp/src/lib.rs`: Agregados tests unitarios para diagnósticos

## ✅ Criterios de Aceptación
- [x] Diagnósticos se envían automáticamente al abrir archivos
- [x] Diagnósticos se actualizan en tiempo real al editar
- [x] Detección de errores de sintaxis básicos (llaves desbalanceadas)
- [x] Detección de warnings (TODO, líneas largas)
- [x] Tests unitarios pasan
- [x] Integración completa con LSP

## 🔗 Referencias
- **Jira:** [TASK-111](https://velalang.atlassian.net/browse/TASK-111)
- **Historia:** [VELA-561](https://velalang.atlassian.net/browse/VELA-561)
- **LSP Spec:** [textDocument/publishDiagnostics](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics)