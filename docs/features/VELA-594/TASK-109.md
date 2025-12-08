# TASK-109: Implementar textDocument/hover

## 📋 Información General
- **Historia:** VELA-594
- **Estado:** Completada ✅
- **Fecha:** 2025-12-07

## 🎯 Objetivo
Implementar el método `textDocument/hover` del LSP para proporcionar información de tooltip sobre símbolos en el código Vela.

## 🔨 Implementación

### Cambios en `lsp/src/server.rs`

#### 1. Imports adicionales
```rust
use lsp_types::{
    // ... existing imports
    HoverProviderCapability, HoverParams, Hover,
    MarkupContent, MarkupKind,
};
```

#### 2. Capabilities del servidor
Agregado soporte para hover provider:
```rust
hover_provider: Some(HoverProviderCapability::Simple(true)),
```

#### 3. Handler de requests
Agregado case para `textDocument/hover`:
```rust
"textDocument/hover" => self.handle_hover(request)?,
```

#### 4. Método `handle_hover`
```rust
fn handle_hover(&self, request: Request) -> Result<Response> {
    let params: HoverParams = serde_json::from_value(request.params)
        .map_err(|e| anyhow::anyhow!("Invalid hover params: {}", e))?;

    info!("Hover requested at position: {:?}", params.text_document_position_params.position);

    let hover = self.compute_hover(&params)?;

    let response = Response::new_ok(request.id, hover);
    Ok(response)
}
```

#### 5. Método `compute_hover`
```rust
fn compute_hover(&self, params: &HoverParams) -> Result<Option<Hover>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    // Get document content
    let store = self.document_store.lock().unwrap();
    let document = match store.get_document(uri) {
        Some(doc) => doc,
        None => return Ok(None), // No document found
    };

    // Analyze the symbol at position
    let hover_info = self.analyze_hover_symbol(document, position);

    Ok(hover_info)
}
```

#### 6. Método `analyze_hover_symbol`
Analiza el símbolo bajo el cursor y genera información de hover:
```rust
fn analyze_hover_symbol(&self, document: &str, position: Position) -> Option<Hover> {
    // Extract word at position
    let word = self.extract_word_at_position(line, char_pos)?;
    // Generate hover information based on the word
    self.generate_hover_for_word(&word)
}
```

#### 7. Método `extract_word_at_position`
Extrae la palabra en la posición del cursor:
```rust
fn extract_word_at_position(&self, line: &str, char_pos: usize) -> Option<String> {
    // Find word boundaries (alphanumeric and underscore)
    // Return the word at the position
}
```

#### 8. Método `generate_hover_for_word`
Genera contenido de hover para palabras conocidas:

- **Keywords**: `fn`, `let`, `state`, `if`, `match`, `class`, `interface`, `public`, `return`
- **Types**: `String`, `Number`, `Float`, `Bool`, `void`
- **Functions**: `print`, `len`

Cada hover incluye:
- Nombre del símbolo en negrita
- Descripción breve
- Ejemplos de código en sintaxis Vela
- Formato Markdown

### Tests Unitarios

Creado `tests/unit/test_lsp.rs` con tests para:
- Extracción de palabras en posiciones específicas
- Generación de hover para keywords, types y functions
- Análisis de símbolos en código de ejemplo
- Manejo de palabras desconocidas

## ✅ Criterios de Aceptación
- [x] LSP server declara soporte para `textDocument/hover`
- [x] Handler implementado para requests de hover
- [x] Análisis de símbolos bajo el cursor
- [x] Información de hover para keywords principales
- [x] Información de hover para tipos primitivos
- [x] Información de hover para funciones built-in
- [x] Formato Markdown en contenido de hover
- [x] Tests unitarios implementados
- [x] Código compila sin errores

## 🔗 Referencias
- **Jira:** [TASK-109](https://velalang.atlassian.net/browse/TASK-109)
- **Historia:** [VELA-594](https://velalang.atlassian.net/browse/VELA-594)
- **LSP Specification:** [textDocument/hover](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover)