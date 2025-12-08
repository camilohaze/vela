# TASK-110: Implementar textDocument/definition

## 📋 Información General
- **Historia:** VELA-594
- **Estado:** Completada ✅
- **Fecha:** 2025-12-07

## 🎯 Objetivo
Implementar el método `textDocument/definition` del LSP para permitir "go-to-definition" en símbolos definidos en el código Vela.

## 🔨 Implementación

### Cambios en `lsp/src/server.rs`

#### 1. Imports adicionales
```rust
use lsp_types::{
    // ... existing imports
    GotoDefinitionParams, GotoDefinitionResponse, Location, Range,
};
```

#### 2. Capabilities del servidor
Agregado soporte para definition provider:
```rust
definition_provider: Some(lsp_types::OneOf::Left(true)),
```

#### 3. Handler de requests
Agregado case para `textDocument/definition`:
```rust
"textDocument/definition" => self.handle_definition(request)?,
```

#### 4. Método `handle_definition`
```rust
fn handle_definition(&self, request: Request) -> Result<Response> {
    let params: GotoDefinitionParams = serde_json::from_value(request.params)
        .map_err(|e| anyhow::anyhow!("Invalid definition params: {}", e))?;

    info!("Definition requested at position: {:?}", params.text_document_position_params.position);

    let definition = self.compute_definition(&params)?;

    let response = Response::new_ok(request.id, definition);
    Ok(response)
}
```

#### 5. Método `compute_definition`
```rust
fn compute_definition(&self, params: &GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    // Get document content
    let store = self.document_store.lock().unwrap();
    let document = match store.get_document(uri) {
        Some(doc) => doc,
        None => return Ok(None), // No document found
    };

    // Analyze the symbol at position and find its definition
    let definition_location = self.analyze_definition_symbol(document, position, uri);

    Ok(definition_location.map(GotoDefinitionResponse::Scalar))
}
```

#### 6. Método `analyze_definition_symbol`
Analiza el símbolo bajo el cursor y encuentra su ubicación de definición:
```rust
fn analyze_definition_symbol(&self, document: &str, position: Position, uri: &lsp_types::Url) -> Option<Location> {
    // Extract word at position
    let word = self.extract_word_at_position(line, char_pos)?;
    // Find the definition location of the symbol
    self.find_symbol_definition(document, &word, uri)
}
```

#### 7. Método `find_symbol_definition`
Busca la definición de un símbolo en el documento actual:

- **Funciones**: Busca patrones `fn function_name(...)`
- **Variables**: Busca patrones `let variable_name:` y `state variable_name:`
- **Clases**: Busca patrones `class ClassName`
- **Interfaces**: Busca patrones `interface InterfaceName`

Para cada tipo de símbolo, devuelve un `Location` con:
- `uri`: URI del documento
- `range`: Rango exacto donde se define el símbolo

### Limitaciones Actuales
- **Intra-documento**: Solo busca definiciones dentro del mismo archivo
- **No cross-file**: No resuelve imports o referencias entre archivos
- **Pattern matching simple**: Usa búsqueda de texto básica, no análisis semántico completo

### Tests Unitarios

Agregados tests en `tests/unit/test_lsp.rs`:
- `test_find_symbol_definition_function`: Pruebas para encontrar definiciones de funciones
- `test_find_symbol_definition_variable`: Pruebas para variables `let` y `state`
- `test_find_symbol_definition_class`: Pruebas para definiciones de clases
- `test_find_symbol_definition_interface`: Pruebas para definiciones de interfaces
- `test_find_symbol_definition_not_found`: Pruebas para símbolos no encontrados
- `test_analyze_definition_symbol`: Pruebas de análisis completo en código de ejemplo

## ✅ Criterios de Aceptación
- [x] LSP server declara soporte para `textDocument/definition`
- [x] Handler implementado para requests de definition
- [x] Búsqueda de definiciones de funciones (`fn`)
- [x] Búsqueda de definiciones de variables (`let`, `state`)
- [x] Búsqueda de definiciones de clases (`class`)
- [x] Búsqueda de definiciones de interfaces (`interface`)
- [x] Retorno de `Location` con rango preciso
- [x] Manejo de símbolos no encontrados (retorna `null`)
- [x] Tests unitarios implementados
- [x] Código compila sin errores

## 🔗 Referencias
- **Jira:** [TASK-110](https://velalang.atlassian.net/browse/TASK-110)
- **Historia:** [VELA-594](https://velalang.atlassian.net/browse/VELA-594)
- **LSP Specification:** [textDocument/definition](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_definition)