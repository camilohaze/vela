# Arquitectura del Vela Language Server Protocol (LSP)

**Historia:** VELA-562 (US-00C)  
**Subtask:** TASK-000L  
**Fecha:** 2025-11-30  
**Estado:** ✅ Completado

---

## 📋 Resumen Ejecutivo

Este documento define la arquitectura del **Vela LSP** (Language Server Protocol), el servidor que proporciona features avanzados de editor (autocompletado, go-to-definition, diagnostics, etc.) para VS Code, Vim, Emacs y otros editores compatibles con LSP.

---

## 1. LSP Features Priority

### **P0: Must-Have (Vela 1.0)**

| Feature | Descripción | Complejidad | Sprint |
|---------|-------------|-------------|--------|
| **Syntax Highlighting** | Tokenización y colores | Baja | 5 |
| **Diagnostics** | Errores y warnings en tiempo real | Media | 5-6 |
| **Go to Definition** | Saltar a definición de símbolo | Media | 6 |
| **Completion** | Autocompletado de keywords, variables, funciones | Alta | 6-7 |

---

### **P1: Should-Have (Vela 1.1)**

| Feature | Descripción | Complejidad | Sprint |
|---------|-------------|-------------|--------|
| **Hover** | Mostrar tipo y documentación al pasar mouse | Media | 8 |
| **Rename** | Renombrar símbolo en todos los archivos | Alta | 8-9 |
| **Find References** | Encontrar todos los usos de un símbolo | Media | 9 |
| **Format Document** | Formatear código automáticamente | Baja | 9 |

---

### **P2: Nice-to-Have (Vela 1.2+)**

| Feature | Descripción | Complejidad | Sprint |
|---------|-------------|-------------|--------|
| **Code Actions** | Quick fixes y refactorings | Alta | 10+ |
| **Signature Help** | Ayuda con parámetros de función | Media | 10+ |
| **Semantic Tokens** | Highlighting avanzado (ej: tipo de variable) | Media | 10+ |
| **Inlay Hints** | Mostrar tipos inferidos inline | Media | 10+ |

---

## 2. Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                          VS Code                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │              Vela Extension (TypeScript)                   │  │
│  │  - Syntax highlighting (TextMate grammar)                  │  │
│  │  - LSP client (vscode-languageclient)                      │  │
│  │  - Commands (build, run, test)                             │  │
│  └────────────────────┬───────────────────────────────────────┘  │
│                       │ LSP Protocol (JSON-RPC over stdio)       │
└───────────────────────┼───────────────────────────────────────────┘
                        │
                        v
┌──────────────────────────────────────────────────────────────────┐
│                       Vela LSP Server (Rust)                      │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    Server Core                              │  │
│  │  - tower_lsp (LSP framework)                               │  │
│  │  - Document synchronization                                │  │
│  │  - Request handling (didOpen, didChange, etc.)             │  │
│  └────────────────────┬───────────────────────────────────────┘  │
│                       │                                           │
│  ┌────────────────────v───────────────────────────────────────┐  │
│  │                 Analysis Engine                            │  │
│  │  - Document cache (in-memory AST + symbol table)           │  │
│  │  - Incremental compilation                                 │  │
│  │  - Diagnostics aggregation                                 │  │
│  └────────────────────┬───────────────────────────────────────┘  │
│                       │                                           │
│  ┌────────────────────v───────────────────────────────────────┐  │
│  │            Compiler Integration Layer                      │  │
│  │  - Lexer (tokenization)                                    │  │
│  │  - Parser (AST generation)                                 │  │
│  │  - Semantic analyzer (type checking, name resolution)      │  │
│  │  - Error recovery (partial AST for invalid code)           │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 3. Integration with Vela Compiler

### **3.1. Shared Crates**

**Estrategia: Reutilizar crates del compilador**

```rust
// Workspace structure
vela/
├── compiler/
│   ├── vela_lexer/          # Tokenizer
│   ├── vela_parser/         # Parser
│   ├── vela_semantic/       # Type checker
│   └── vela_codegen/        # Code generation
│
├── lsp/
│   ├── vela_lsp/            # LSP server main
│   │   └── Cargo.toml       # depends on vela_lexer, vela_parser, vela_semantic
│   └── vscode-vela/         # VS Code extension
```

**Beneficios:**
- ✅ **Code reuse**: No duplicar lexer/parser
- ✅ **Consistency**: Same parsing logic
- ✅ **Maintenance**: Fix once, works everywhere

---

### **3.2. AST Sharing**

**Problema:** Compiler AST puede ser muy pesado para LSP (mucha memoria).

**Solución: Dual AST Strategy**
- **Full AST** (compiler): Para codegen, optimizaciones
- **Lightweight AST** (LSP): Solo metadatos necesarios (spans, types, symbols)

**Ejemplo:**
```rust
// Full AST (compiler)
pub struct FunctionDecl {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Block,  // Full body AST
    pub attributes: Vec<Attribute>,
    // ... más metadata
}

// Lightweight AST (LSP)
pub struct FunctionDeclLsp {
    pub span: Span,  // Position in file
    pub name: String,
    pub params: Vec<ParamLsp>,
    pub return_type: TypeLsp,
    // No body AST (no necesario para completion/hover)
}
```

---

### **3.3. Type Queries API**

**LSP necesita consultar tipos sin recompilar todo:**

```rust
pub trait TypeQueryEngine {
    /// Get type of expression at position
    fn type_at_position(&self, file: &Path, line: u32, col: u32) -> Option<Type>;
    
    /// Get all symbols in scope at position
    fn symbols_in_scope(&self, file: &Path, line: u32, col: u32) -> Vec<Symbol>;
    
    /// Find definition of symbol at position
    fn find_definition(&self, file: &Path, line: u32, col: u32) -> Option<Location>;
    
    /// Find all references of symbol
    fn find_references(&self, file: &Path, line: u32, col: u32) -> Vec<Location>;
}
```

**Implementación:**
```rust
impl TypeQueryEngine for SemanticAnalyzer {
    fn type_at_position(&self, file: &Path, line: u32, col: u32) -> Option<Type> {
        let ast = self.get_cached_ast(file)?;
        let expr = ast.find_expr_at_position(line, col)?;
        self.type_check_expr(expr)
    }
}
```

---

### **3.4. Incremental Compilation**

**Problema:** Recompilar todo el proyecto en cada keystroke es lento.

**Solución: Incremental + Parallel**
1. **Solo recompilar archivos modificados**
2. **Cachear AST + symbol tables**
3. **Invalidar dependencias transitivas**

**Estrategia:**
```rust
pub struct IncrementalCompiler {
    /// File path → (AST, symbol table, last modified time)
    cache: HashMap<PathBuf, CachedFile>,
}

impl IncrementalCompiler {
    pub fn recompile(&mut self, changed_file: &Path) {
        // 1. Invalidar archivo modificado
        self.cache.remove(changed_file);
        
        // 2. Invalidar dependencias
        for dependent in self.find_dependents(changed_file) {
            self.cache.remove(dependent);
        }
        
        // 3. Recompilar solo archivos invalidados
        self.compile_file(changed_file);
    }
}
```

---

## 4. LSP Server Implementation

### **4.1. Framework: `tower-lsp`**

**Razones:**
- ✅ **Async**: Tokio-based, non-blocking
- ✅ **Type-safe**: Strong types for LSP protocol
- ✅ **Mature**: Used by rust-analyzer

**Ejemplo básico:**
```rust
use tower_lsp::{LspService, Server, LanguageServer};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

struct VelaLanguageServer {
    client: Client,
    document_cache: Arc<RwLock<HashMap<Url, Document>>>,
    analyzer: Arc<RwLock<SemanticAnalyzer>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for VelaLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "vela-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                // ... más capabilities
                ..Default::default()
            },
        })
    }
    
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        
        // 1. Parse document
        let ast = parse_document(&text);
        
        // 2. Cache AST
        self.document_cache.write().await.insert(uri.clone(), Document {
            text,
            ast: ast.clone(),
        });
        
        // 3. Run diagnostics
        let diagnostics = self.run_diagnostics(&ast);
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }
    
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        // Get symbols in scope
        let symbols = self.analyzer.read().await
            .symbols_in_scope(&uri, position.line, position.character);
        
        // Convert to LSP CompletionItems
        let items: Vec<CompletionItem> = symbols.iter().map(|sym| {
            CompletionItem {
                label: sym.name.clone(),
                kind: Some(match sym.kind {
                    SymbolKind::Function => CompletionItemKind::FUNCTION,
                    SymbolKind::Variable => CompletionItemKind::VARIABLE,
                    // ...
                }),
                detail: Some(sym.type_str.clone()),
                ..Default::default()
            }
        }).collect();
        
        Ok(Some(CompletionResponse::Array(items)))
    }
    
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let location = self.analyzer.read().await
            .find_definition(&uri, position.line, position.character);
        
        Ok(location.map(|loc| GotoDefinitionResponse::Scalar(loc)))
    }
}
```

---

### **4.2. Document Synchronization**

**LSP Protocol:**
- `textDocument/didOpen`: Editor abre archivo
- `textDocument/didChange`: Usuario edita (incremental updates)
- `textDocument/didSave`: Usuario guarda
- `textDocument/didClose`: Editor cierra archivo

**Manejo de cambios incrementales:**
```rust
async fn did_change(&self, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri;
    
    for change in params.content_changes {
        match change.range {
            Some(range) => {
                // Incremental update
                self.apply_change(&uri, range, &change.text).await;
            }
            None => {
                // Full document sync
                self.replace_document(&uri, &change.text).await;
            }
        }
    }
    
    // Re-run diagnostics
    self.publish_diagnostics(&uri).await;
}
```

---

## 5. Feature Implementations

### **5.1. Syntax Highlighting**

**Implementación:** TextMate grammar (JSON).

**Archivo:** `vscode-vela/syntaxes/vela.tmLanguage.json`

**Ejemplo:**
```json
{
  "scopeName": "source.vela",
  "patterns": [
    {
      "name": "keyword.control.vela",
      "match": "\\b(if|else|while|for|return|break|continue)\\b"
    },
    {
      "name": "keyword.declaration.vela",
      "match": "\\b(let|fn|class|widget|signal)\\b"
    },
    {
      "name": "string.quoted.double.vela",
      "begin": "\"",
      "end": "\"",
      "patterns": [
        {
          "name": "constant.character.escape.vela",
          "match": "\\\\."
        }
      ]
    }
  ]
}
```

**VS Code themes automáticamente colorean según scopes.**

---

### **5.2. Diagnostics**

**Implementación:**
```rust
fn run_diagnostics(&self, ast: &Ast) -> Vec<Diagnostic> {
    let mut diagnostics = vec![];
    
    // 1. Syntax errors
    for error in ast.syntax_errors() {
        diagnostics.push(Diagnostic {
            range: error.span.to_lsp_range(),
            severity: Some(DiagnosticSeverity::ERROR),
            message: error.message.clone(),
            source: Some("vela-lsp".to_string()),
            ..Default::default()
        });
    }
    
    // 2. Type errors
    for error in self.type_check(ast) {
        diagnostics.push(Diagnostic {
            range: error.span.to_lsp_range(),
            severity: Some(DiagnosticSeverity::ERROR),
            message: format!("Type error: {}", error.message),
            ..Default::default()
        });
    }
    
    // 3. Linter warnings
    for warning in self.lint(ast) {
        diagnostics.push(Diagnostic {
            range: warning.span.to_lsp_range(),
            severity: Some(DiagnosticSeverity::WARNING),
            message: warning.message.clone(),
            ..Default::default()
        });
    }
    
    diagnostics
}
```

**Output en VS Code:**
```
× Error: Type mismatch
  --> main.vela:10:5
   |
10 |     return "hello";
   |            ^^^^^^^ expected `Int`, found `String`
```

---

### **5.3. Completion**

**Tipos de completion:**
1. **Keywords**: `let`, `fn`, `if`, `while`, etc.
2. **Symbols in scope**: Variables, funciones locales
3. **Imported symbols**: De `import` statements
4. **Struct fields**: Al escribir `obj.`
5. **Type names**: Para type annotations

**Implementación:**
```rust
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    
    let mut items = vec![];
    
    // 1. Keywords
    items.extend(KEYWORDS.iter().map(|kw| CompletionItem {
        label: kw.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        ..Default::default()
    }));
    
    // 2. Symbols in scope
    let scope_symbols = self.analyzer.read().await
        .symbols_in_scope(&uri, position.line, position.character);
    items.extend(scope_symbols.iter().map(|sym| CompletionItem {
        label: sym.name.clone(),
        kind: Some(match sym.kind {
            SymbolKind::Function => CompletionItemKind::FUNCTION,
            SymbolKind::Variable => CompletionItemKind::VARIABLE,
            SymbolKind::Class => CompletionItemKind::CLASS,
            _ => CompletionItemKind::TEXT,
        }),
        detail: Some(sym.type_str.clone()),
        documentation: sym.doc_comment.as_ref().map(|doc| {
            Documentation::String(doc.clone())
        }),
        ..Default::default()
    }));
    
    Ok(Some(CompletionResponse::Array(items)))
}
```

**UX en VS Code:**
```
let x = 10;
let y = x.  // Trigger completion
         └── [to_string(), abs(), ...] (methods de Int)
```

---

### **5.4. Hover**

**Implementación:**
```rust
async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;
    
    // Find symbol at position
    let symbol = self.analyzer.read().await
        .symbol_at_position(&uri, position.line, position.character)?;
    
    // Format hover content
    let content = format!(
        "```vela\n{}: {}\n```\n\n{}",
        symbol.name,
        symbol.type_str,
        symbol.doc_comment.unwrap_or_default()
    );
    
    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: Some(symbol.span.to_lsp_range()),
    }))
}
```

**UX en VS Code:**
```
let x = 10;
    ^ Hover aquí

┌─────────────────────────┐
│ ```vela                 │
│ x: Int                  │
│ ```                     │
│                         │
│ Local variable          │
└─────────────────────────┘
```

---

### **5.5. Go to Definition**

**Implementación:**
```rust
async fn goto_definition(
    &self,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;
    
    // Find definition location
    let location = self.analyzer.read().await
        .find_definition(&uri, position.line, position.character)?;
    
    Ok(Some(GotoDefinitionResponse::Scalar(Location {
        uri: location.file.into(),
        range: location.span.to_lsp_range(),
    })))
}
```

**UX:**
```vela
// main.vela
fn foo() { ... }

fn main() {
    foo();  // Ctrl+Click → jumps to line 1
}
```

---

## 6. VS Code Extension

### **6.1. Extension Structure**

```
vscode-vela/
├── package.json          # Extension manifest
├── src/
│   ├── extension.ts      # Extension entry point
│   └── client.ts         # LSP client
├── syntaxes/
│   └── vela.tmLanguage.json  # Syntax highlighting
└── snippets/
    └── vela.json         # Code snippets
```

---

### **6.2. `package.json`**

```json
{
  "name": "vela",
  "displayName": "Vela Language Support",
  "version": "0.1.0",
  "publisher": "velalang",
  "engines": {
    "vscode": "^1.75.0"
  },
  "categories": ["Programming Languages"],
  "activationEvents": ["onLanguage:vela"],
  "main": "./out/extension.js",
  "contributes": {
    "languages": [{
      "id": "vela",
      "aliases": ["Vela", "vela"],
      "extensions": [".vela"],
      "configuration": "./language-configuration.json"
    }],
    "grammars": [{
      "language": "vela",
      "scopeName": "source.vela",
      "path": "./syntaxes/vela.tmLanguage.json"
    }],
    "commands": [
      {
        "command": "vela.build",
        "title": "Vela: Build Project"
      },
      {
        "command": "vela.run",
        "title": "Vela: Run Project"
      },
      {
        "command": "vela.test",
        "title": "Vela: Run Tests"
      }
    ],
    "configuration": {
      "title": "Vela",
      "properties": {
        "vela.lsp.serverPath": {
          "type": "string",
          "default": "vela-lsp",
          "description": "Path to vela-lsp binary"
        },
        "vela.lsp.trace.server": {
          "type": "string",
          "enum": ["off", "messages", "verbose"],
          "default": "off",
          "description": "LSP tracing level"
        }
      }
    }
  }
}
```

---

### **6.3. LSP Client (`extension.ts`)**

```typescript
import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  // LSP server executable
  const serverPath = workspace.getConfiguration('vela').get<string>('lsp.serverPath') || 'vela-lsp';
  
  const serverOptions: ServerOptions = {
    command: serverPath,
    args: [],
  };
  
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'vela' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.vela'),
    },
  };
  
  client = new LanguageClient(
    'vela-lsp',
    'Vela Language Server',
    serverOptions,
    clientOptions
  );
  
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
```

---

## 7. Performance Considerations

### **7.1. Benchmarks Target**

| Operation | Target Latency |
|-----------|----------------|
| **didChange** → diagnostics | < 100ms (p95) |
| **Completion request** | < 50ms (p95) |
| **Go to Definition** | < 30ms (p95) |
| **Hover** | < 30ms (p95) |

---

### **7.2. Optimization Strategies**

1. **Lazy AST parsing**: Solo parsear archivos abiertos
2. **Background diagnostics**: Compute async, no bloquear UI
3. **Debounce didChange**: No recompilar en cada keystroke
4. **Symbol index**: Pre-indexar todos los símbolos del proyecto

---

## 8. Testing

### **8.1. Unit Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_completion_keywords() {
        let server = VelaLanguageServer::new();
        let uri = Url::parse("file:///test.vela").unwrap();
        server.did_open(...).await;
        
        let result = server.completion(...).await.unwrap();
        assert!(result.unwrap().len() > 0);
    }
}
```

---

### **8.2. Integration Tests**

**Escenario:** Abrir archivo → Editar → Verificar diagnostics.

```rust
#[tokio::test]
async fn test_diagnostics_on_type_error() {
    let server = setup_test_server().await;
    
    // Open file with type error
    server.did_open(params_for("let x: Int = \"hello\";")).await;
    
    // Assert diagnostic published
    let diagnostics = server.get_diagnostics().await;
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Type mismatch"));
}
```

---

## 9. Referencias

- **LSP Specification**: https://microsoft.github.io/language-server-protocol/
- **tower-lsp**: https://docs.rs/tower-lsp/
- **rust-analyzer**: https://github.com/rust-lang/rust-analyzer
- **VS Code LSP Guide**: https://code.visualstudio.com/api/language-extensions/language-server-extension-guide

---

**Autor:** Vela Core Team  
**Revisión:** 2025-11-30  
**Versión:** 1.0
