# TASK-111: Implementar textDocument/signatureHelp

## 📋 Información General
- **Historia:** VELA-594
- **Estado:** Completada ✅
- **Fecha:** 2025-12-07

## 🎯 Objetivo
Implementar el método `textDocument/signatureHelp` del LSP para mostrar información de ayuda de firmas de funciones mientras se escriben llamadas a funciones.

## 🔨 Implementación

### Cambios en `lsp/src/server.rs`

#### 1. Imports adicionales
```rust
use lsp_types::{
    // ... existing imports
    SignatureHelpParams, SignatureHelp, SignatureInformation, ParameterInformation,
    SignatureHelpOptions,
};
```

#### 2. Nueva estructura de datos
```rust
/// Context information for a function call
#[derive(Debug)]
struct FunctionCallContext {
    function_name: String,
    active_parameter: usize,
}
```

#### 3. Capabilities del servidor
Agregado soporte para signature help provider:
```rust
signature_help_provider: Some(SignatureHelpOptions {
    trigger_characters: Some(vec!["(".to_string()]),
    retrigger_characters: Some(vec![",".to_string()]),
    ..Default::default()
}),
```

#### 4. Handler de requests
Agregado case para `textDocument/signatureHelp`:
```rust
"textDocument/signatureHelp" => self.handle_signature_help(request)?,
```

#### 5. Método `handle_signature_help`
```rust
fn handle_signature_help(&self, request: Request) -> Result<Response> {
    let params: SignatureHelpParams = serde_json::from_value(request.params)
        .map_err(|e| anyhow::anyhow!("Invalid signatureHelp params: {}", e))?;

    info!("Signature help requested at position: {:?}", params.text_document_position_params.position);

    let signature_help = self.compute_signature_help(&params)?;

    let response = Response::new_ok(request.id, signature_help);
    Ok(response)
}
```

#### 6. Método `compute_signature_help`
```rust
fn compute_signature_help(&self, params: &SignatureHelpParams) -> Result<Option<SignatureHelp>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    // Get document content
    let store = self.document_store.lock().unwrap();
    let document = match store.get_document(uri) {
        Some(doc) => doc,
        None => return Ok(None), // No document found
    };

    // Analyze the function call at position
    let signature_help = self.analyze_signature_help(document, position);

    Ok(signature_help)
}
```

#### 7. Método `analyze_signature_help`
Analiza la llamada a función en la posición del cursor:
```rust
fn analyze_signature_help(&self, document: &str, position: Position) -> Option<SignatureHelp> {
    // Extract function call context
    let function_call = self.extract_function_call_context(line, char_pos)?;

    // Get signature information for the function
    let signatures = self.get_function_signatures(&function_call.function_name)?;

    // Determine active parameter based on position in call
    let active_parameter = self.calculate_active_parameter(&function_call, char_pos);

    Some(SignatureHelp {
        signatures,
        active_signature: Some(0), // We only provide one signature for now
        active_parameter: Some(active_parameter),
    })
}
```

#### 8. Método `extract_function_call_context`
Extrae el contexto de llamada a función desde la línea en la posición:
```rust
fn extract_function_call_context(&self, line: &str, char_pos: usize) -> Option<FunctionCallContext> {
    // Find the opening parenthesis before the cursor
    let before_cursor = &line[..char_pos];
    let open_paren_pos = before_cursor.rfind('(')?;

    // Find the function name before the opening parenthesis
    let before_paren = &before_cursor[..open_paren_pos];
    let function_name = self.extract_word_at_position(before_paren, before_paren.len())?;

    // Count commas to determine active parameter
    let after_open = &line[open_paren_pos..char_pos];
    let comma_count = after_open.chars().filter(|&c| c == ',').count();

    Some(FunctionCallContext {
        function_name,
        active_parameter: comma_count,
    })
}
```

#### 9. Método `get_function_signatures`
Proporciona información de firmas para funciones conocidas:

- **print**: `print(value: any) -> void`
- **len**: `len(collection) -> Number`
- **add**: `add(a: Number, b: Number) -> Number`

Cada firma incluye:
- `label`: Firma completa de la función
- `documentation`: Descripción de la función
- `parameters`: Lista de parámetros con documentación individual

#### 10. Método `calculate_active_parameter`
Calcula qué parámetro está activo basado en la posición del cursor:
```rust
fn calculate_active_parameter(&self, function_call: &FunctionCallContext, char_pos: usize) -> u32 {
    function_call.active_parameter as u32
}
```

### Características del Signature Help

- **Trigger Characters**: `(` (abre paréntesis) y `,` (coma)
- **Active Parameter**: Resaltado del parámetro actual basado en comas
- **Multiple Signatures**: Soporte para múltiples sobrecargas (futuro)
- **Parameter Documentation**: Documentación individual por parámetro

### Limitaciones Actuales
- **Funciones built-in only**: Solo reconoce funciones predefinidas (`print`, `len`, `add`)
- **No análisis semántico**: No analiza definiciones de funciones del usuario
- **Single signature**: Solo muestra una firma por función (no sobrecargas)

### Tests Unitarios

Agregados tests en `tests/unit/test_lsp.rs`:
- `test_extract_function_call_context`: Pruebas para extracción de contexto de llamadas
- `test_get_function_signatures`: Pruebas para obtención de firmas de funciones conocidas
- `test_calculate_active_parameter`: Pruebas para cálculo de parámetro activo
- `test_analyze_signature_help`: Pruebas de análisis completo en código de ejemplo
- `test_signature_help_with_multiple_parameters`: Pruebas con múltiples parámetros

## ✅ Criterios de Aceptación
- [x] LSP server declara soporte para `textDocument/signatureHelp`
- [x] Handler implementado para requests de signature help
- [x] Trigger characters configurados (`(`, `,`)
- [x] Análisis de llamadas a funciones en posición del cursor
- [x] Información de firmas para funciones built-in (`print`, `len`, `add`)
- [x] Cálculo correcto del parámetro activo
- [x] Formato `SignatureHelp` con `signatures`, `active_signature`, `active_parameter`
- [x] Documentación de parámetros individuales
- [x] Tests unitarios implementados
- [x] Código compila sin errores

## 🔗 Referencias
- **Jira:** [TASK-111](https://velalang.atlassian.net/browse/TASK-111)
- **Historia:** [VELA-594](https://velalang.atlassian.net/browse/VELA-594)
- **LSP Specification:** [textDocument/signatureHelp](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_signatureHelp)