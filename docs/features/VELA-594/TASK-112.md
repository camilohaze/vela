# TASK-112: Implementar textDocument/references

## 📋 Información General
- **Historia:** VELA-594
- **Estado:** Completada ✅
- **Fecha:** 2024-12-30

## 🎯 Objetivo
Implementar la funcionalidad textDocument/references del LSP para encontrar todas las referencias a un símbolo en el documento actual.

## 🔨 Implementación

### Métodos Agregados

#### `handle_references(request: Request) -> Result<Response>`
Maneja las solicitudes textDocument/references del LSP. Extrae los parámetros, calcula las referencias y retorna la respuesta.

#### `compute_references(params: &ReferenceParams) -> Result<Vec<Location>>`
Calcula todas las referencias al símbolo en la posición especificada. Obtiene el documento, analiza el símbolo y encuentra todas sus referencias.

#### `analyze_references_symbol(document: &str, position: Position) -> Result<String>`
Analiza el símbolo en la posición dada para referencias. Extrae la palabra en la posición del cursor.

#### `find_symbol_references(document: &str, symbol: &str, uri: Url) -> Vec<Location>`
Encuentra todas las referencias a un símbolo en el documento. Busca todas las ocurrencias del símbolo y verifica que sean palabras completas.

#### `is_word_boundary(line: &str, start: usize, end: usize) -> bool`
Verifica si un rango de caracteres representa una palabra completa (no parte de otra palabra). Comprueba que no haya caracteres alfanuméricos o guiones bajos adyacentes.

### Archivos Modificados
- `lsp/src/server.rs` - Implementación completa de references
- `lsp/src/lib.rs` - Arreglo de import en tests
- `tests/unit/test_lsp.rs` - Tests unitarios para references

### Protocolo LSP
- **Método:** `textDocument/references`
- **Parámetros:** `ReferenceParams` con posición y contexto
- **Respuesta:** `Vec<Location>` con todas las referencias encontradas

## ✅ Criterios de Aceptación
- [x] Implementado `handle_references` method
- [x] Implementado `compute_references` method
- [x] Implementado `analyze_references_symbol` method
- [x] Implementado `find_symbol_references` method
- [x] Implementado `is_word_boundary` helper method
- [x] Agregado capability `references_provider` en inicialización
- [x] Agregado case `"textDocument/references"` en `handle_request`
- [x] Código compila sin errores
- [x] Tests unitarios pasan
- [x] Documentación completa

## 🔗 Referencias
- **Jira:** [VELA-594](https://velalang.atlassian.net/browse/VELA-594)
- **Historia:** [VELA-594](https://velalang.atlassian.net/browse/VELA-594)
- **LSP Spec:** [textDocument/references](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_references)