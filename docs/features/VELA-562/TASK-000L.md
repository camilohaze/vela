# TASK-000L: Diseñar Arquitectura del Language Server Protocol (LSP)

## 📋 Información General
- **Historia:** VELA-562 (Tooling Design - Phase 0)
- **Epic:** EPIC-00C: Tooling Design
- **Sprint:** 2
- **Estado:** Completado ✅
- **Prioridad:** P0 (Crítica)
- **Estimación:** 56 horas
- **Dependencias:** VELA-561 (Compiler Architecture)

---

## 🎯 Objetivo

Diseñar la arquitectura del **Vela Language Server**, incluyendo:

- **LSP features** priorizado (syntax, diagnostics, completion, etc.)
- **Architecture** (incremental compilation, caching)
- **Integration** con compiler frontend
- **Performance targets** (< 100ms latency)

---

## 🏗️ LSP Architecture

### 1. LSP Features Roadmap

#### 1.1 Priority Levels

**P0 (Phase 1 - MVP):**
- ✅ Syntax highlighting (TextMate grammar)
- ✅ Diagnostics (errors, warnings)
- ✅ Go to Definition (`Ctrl+Click`)
- ✅ Find All References (`Shift+F12`)
- ✅ Hover (show type info)
- ✅ Code Completion (autocomplete)
- ✅ Document Symbols (Outline view)
- ✅ Workspace Symbols (Search symbols)

**P1 (Phase 2 - Enhanced):**
- ⏳ Rename Symbol (`F2`)
- ⏳ Code Actions (Quick fixes)
- ⏳ Signature Help (parameter hints)
- ⏳ Folding Ranges (code folding)
- ⏳ Semantic Tokens (semantic highlighting)
- ⏳ Call Hierarchy (call graph)

**P2 (Phase 3 - Advanced):**
- ⏳ Inlay Hints (inline type hints)
- ⏳ Code Lens (inline actions)
- ⏳ Document Formatting (`Shift+Alt+F`)
- ⏳ Document Link (clickable links)
- ⏳ Color Provider (color picker)

---

#### 1.2 Feature Specifications

##### **Syntax Highlighting**

**Approach:** TextMate grammar (VS Code)

```json
{
  "name": "Vela",
  "scopeName": "source.vela",
  "fileTypes": ["vela"],
  "patterns": [
    {
      "comment": "Keywords",
      "name": "keyword.control.vela",
      "match": "\\b(if|else|match|fn|return|async|await|state|computed|effect)\\b"
    },
    {
      "comment": "Types",
      "name": "storage.type.vela",
      "match": "\\b(Number|String|Bool|Float|void|never|Option|Result)\\b"
    },
    {
      "comment": "String literals",
      "name": "string.quoted.double.vela",
      "begin": "\"",
      "end": "\"",
      "patterns": [
        {
          "name": "constant.character.escape.vela",
          "match": "\\\\(n|t|r|\\\"|\\\\|\\$\\{)"
        },
        {
          "name": "meta.embedded.line.vela",
          "begin": "\\$\\{",
          "end": "\\}",
          "patterns": [{"include": "$self"}]
        }
      ]
    },
    {
      "comment": "Comments",
      "name": "comment.line.number-sign.vela",
      "match": "#.*$"
    }
  ]
}
```

---

##### **Diagnostics**

**Propósito:** Mostrar errores y warnings en tiempo real

**Protocol:**
```typescript
interface Diagnostic {
  range: Range              // { start: Position, end: Position }
  severity: DiagnosticSeverity   // Error | Warning | Info | Hint
  message: string
  code?: string             // VELA-E001
  source: "vela"
  relatedInformation?: RelatedInformation[]
}
```

**Ejemplo:**
```vela
fn divide(a: Number, b: Number) -> Float {
  return a / b  # ERROR: Division by zero possible
}
```

**Diagnostic:**
```json
{
  "range": { "start": {"line": 1, "character": 9}, "end": {"line": 1, "character": 14} },
  "severity": 1,
  "message": "Division by zero possible. Consider checking if b == 0.",
  "code": "VELA-E042",
  "source": "vela"
}
```

---

##### **Go to Definition**

**Propósito:** Navegar a la definición de símbolo

**Protocol:**
```typescript
textDocument/definition(
  textDocument: TextDocumentIdentifier,
  position: Position
) -> Location | Location[] | null
```

**Ejemplo:**
```vela
fn main() -> void {
  result = calculate(5, 3)
           ^^^^^^^^^ Ctrl+Click → jump to definition
}

fn calculate(a: Number, b: Number) -> Number {  # <- Jump here
  return a + b
}
```

**Response:**
```json
{
  "uri": "file:///path/to/file.vela",
  "range": {
    "start": {"line": 5, "character": 3},
    "end": {"line": 5, "character": 12}
  }
}
```

---

##### **Find All References**

**Propósito:** Encontrar todos los usos de un símbolo

**Protocol:**
```typescript
textDocument/references(
  textDocument: TextDocumentIdentifier,
  position: Position,
  context: { includeDeclaration: boolean }
) -> Location[]
```

**Ejemplo:**
```vela
fn calculate(a: Number, b: Number) -> Number { ... }
   ^^^^^^^^^ Find references (Shift+F12)

fn main() -> void {
  result1 = calculate(5, 3)      # <- Reference 1
  result2 = calculate(10, 20)    # <- Reference 2
}
```

---

##### **Hover**

**Propósito:** Mostrar tipo e información al hover

**Protocol:**
```typescript
textDocument/hover(
  textDocument: TextDocumentIdentifier,
  position: Position
) -> Hover | null
```

**Ejemplo:**
```vela
fn main() -> void {
  numbers = [1, 2, 3, 4, 5]
  ^^^^^^^ Hover → Show type: List<Number>
}
```

**Response:**
```json
{
  "contents": {
    "kind": "markdown",
    "value": "```vela\nnumbers: List<Number>\n```\n\nList of 5 elements"
  },
  "range": {
    "start": {"line": 1, "character": 2},
    "end": {"line": 1, "character": 9}
  }
}
```

---

##### **Code Completion**

**Propósito:** Autocompletar código

**Protocol:**
```typescript
textDocument/completion(
  textDocument: TextDocumentIdentifier,
  position: Position
) -> CompletionItem[]
```

**Ejemplo:**
```vela
fn main() -> void {
  list = [1, 2, 3]
  list.m
       ^ Trigger completion (Ctrl+Space)
}
```

**Response:**
```json
[
  {
    "label": "map",
    "kind": 2,  // Method
    "detail": "(fn: (T) -> U) -> List<U>",
    "documentation": "Transform each element with function",
    "insertText": "map(${1:fn})",
    "insertTextFormat": 2  // Snippet
  },
  {
    "label": "filter",
    "kind": 2,
    "detail": "(predicate: (T) -> Bool) -> List<T>",
    "documentation": "Filter elements by predicate",
    "insertText": "filter(${1:predicate})",
    "insertTextFormat": 2
  }
]
```

---

##### **Rename Symbol**

**Propósito:** Renombrar símbolo en todo el workspace

**Protocol:**
```typescript
textDocument/rename(
  textDocument: TextDocumentIdentifier,
  position: Position,
  newName: string
) -> WorkspaceEdit
```

**Ejemplo:**
```vela
fn calculate(a: Number, b: Number) -> Number {
   ^^^^^^^^^ Press F2, type "compute"
  return a + b
}

fn main() -> void {
  result = calculate(5, 3)  # <- Will rename to compute()
}
```

**Response:**
```json
{
  "changes": {
    "file:///path/to/file.vela": [
      {
        "range": {"start": {"line": 0, "character": 3}, "end": {"line": 0, "character": 12}},
        "newText": "compute"
      },
      {
        "range": {"start": {"line": 5, "character": 11}, "end": {"line": 5, "character": 20}},
        "newText": "compute"
      }
    ]
  }
}
```

---

##### **Code Actions (Quick Fixes)**

**Propósito:** Acciones rápidas (import, fix, refactor)

**Protocol:**
```typescript
textDocument/codeAction(
  textDocument: TextDocumentIdentifier,
  range: Range,
  context: CodeActionContext
) -> CodeAction[]
```

**Ejemplo:**
```vela
fn main() -> void {
  user = User("Alice")  # ERROR: User not imported
         ^^^^
}
```

**Code Action:**
```json
{
  "title": "Import 'User' from 'module:auth'",
  "kind": "quickfix",
  "edit": {
    "changes": {
      "file:///path/to/file.vela": [
        {
          "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}},
          "newText": "import 'module:auth' show { User }\n\n"
        }
      ]
    }
  }
}
```

---

### 2. Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Editor (VS Code)                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   Editor    │  │  Diagnostics│  │  Completion │            │
│  │   Buffer    │  │    Panel    │  │    Menu     │            │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘            │
│         │                 │                 │                   │
│         └─────────────────┴─────────────────┘                   │
│                           │                                     │
│                   JSON-RPC over stdio                          │
│                           │                                     │
└───────────────────────────┼─────────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────────┐
│                    Vela Language Server                         │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                  Request Handler                          │ │
│  │  (textDocument/*, workspace/*)                            │ │
│  └────────────────────┬──────────────────────────────────────┘ │
│                       │                                         │
│  ┌────────────────────▼──────────────────────────────────────┐ │
│  │                  Document Manager                         │ │
│  │  - In-memory document state                               │ │
│  │  - Incremental updates (TextDocumentSyncKind.Incremental) │ │
│  │  - Version tracking                                        │ │
│  └────────────────────┬──────────────────────────────────────┘ │
│                       │                                         │
│  ┌────────────────────▼──────────────────────────────────────┐ │
│  │              Incremental Parser                           │ │
│  │  - Tree-sitter (incremental parsing)                      │ │
│  │  - CST → AST                                               │ │
│  │  - Error recovery                                          │ │
│  └────────────────────┬──────────────────────────────────────┘ │
│                       │                                         │
│  ┌────────────────────▼──────────────────────────────────────┐ │
│  │                Type Checker                               │ │
│  │  - Hindley-Milner type inference                          │ │
│  │  - Constraint solving                                      │ │
│  │  - Error reporting                                         │ │
│  └────────────────────┬──────────────────────────────────────┘ │
│                       │                                         │
│  ┌────────────────────▼──────────────────────────────────────┐ │
│  │                 Symbol Index                              │ │
│  │  - HashMap<SymbolId, SymbolInfo>                          │ │
│  │  - Definition locations                                    │ │
│  │  - Reference locations                                     │ │
│  │  - Type information                                        │ │
│  └────────────────────┬──────────────────────────────────────┘ │
│                       │                                         │
│  ┌────────────────────▼──────────────────────────────────────┐ │
│  │                 Salsa Cache                               │ │
│  │  - Query-based caching                                     │ │
│  │  - Incremental recomputation                               │ │
│  │  - Dependency tracking                                     │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

### 3. Incremental Compilation (Salsa)

#### 3.1 Salsa Framework

**Propósito:** Incrementalidad eficiente (solo recomputar lo necesario)

**Concepto:**
- **Query-based**: Todo computation es una query
- **Memoization**: Resultados se cachean
- **Dependency tracking**: Queries rastrean dependencias
- **Invalidation**: Cuando input cambia, invalidar queries dependientes

**Ejemplo:**
```rust
// Salsa database
#[salsa::query_group(CompilerDatabase)]
trait CompilerQueries {
    // Input query (manual)
    #[salsa::input]
    fn source_text(&self, file: FileId) -> Arc<String>;
    
    // Derived query (computed)
    fn parse(&self, file: FileId) -> Arc<Ast>;
    fn typecheck(&self, file: FileId) -> Arc<TypedAst>;
    fn diagnostics(&self, file: FileId) -> Arc<Vec<Diagnostic>>;
}

// Parse query implementation
fn parse(db: &dyn CompilerQueries, file: FileId) -> Arc<Ast> {
    let source = db.source_text(file);  // Depends on source_text
    let ast = vela_parser::parse(&source);
    Arc::new(ast)
}
```

**Flujo:**
```
User edits file → source_text(file) invalidated
                → parse(file) invalidated (depends on source_text)
                → typecheck(file) invalidated (depends on parse)
                → diagnostics(file) invalidated (depends on typecheck)
                
Next request for diagnostics:
  → diagnostics(file) recomputed
    → typecheck(file) recomputed
      → parse(file) recomputed
        → source_text(file) retrieved (new value)
```

---

#### 3.2 Query Graph

**Example scenario:**

```vela
# file: main.vela (User types "x = 5")
x: Number = 5

fn main() -> void {
  print(x)
}
```

**Query graph:**
```
source_text(main.vela)
    ↓
parse(main.vela)
    ↓
typecheck(main.vela)
    ↓
diagnostics(main.vela)  →  [No errors]
```

**User types "x = "hello"" (type error):**
```
source_text(main.vela) [CHANGED]
    ↓
parse(main.vela) [RECOMPUTE]
    ↓
typecheck(main.vela) [RECOMPUTE]
    ↓
diagnostics(main.vela) [RECOMPUTE]  →  [Error: Type mismatch]
```

---

### 4. Integration with Compiler

#### 4.1 Shared Codebase

**Estructura:**
```
vela/
├── compiler/
│   ├── lexer/          # Shared by compiler & LSP
│   ├── parser/         # Shared by compiler & LSP
│   ├── typechecker/    # Shared by compiler & LSP
│   └── codegen/        # Compiler only (NOT in LSP)
│
├── lsp/
│   ├── server.rs       # LSP server main loop
│   ├── handlers.rs     # Request handlers
│   ├── document.rs     # Document manager
│   └── symbols.rs      # Symbol index
│
└── cli/
    └── main.rs         # CLI entry point
```

**Ventajas:**
- ✅ No duplicación de código
- ✅ Bugs arreglados en un lugar
- ✅ Misma semántica en compiler y LSP

---

#### 4.2 Error Recovery

**Problema:** Parser debe ser tolerante a errores (documento incompleto)

**Approach:** Error recovery en parser

**Ejemplo:**
```vela
fn main() -> void {
  x = 5
  y =       # User typing... incomplete statement
  z = 10
}
```

**Parser behavior:**
```
Parse tree:
  FnDecl(
    name: "main",
    body: [
      Assignment(x, 5),     # OK
      Assignment(y, ERROR), # ERROR node (missing RHS)
      Assignment(z, 10)     # OK (recovered)
    ]
  )
```

**Diagnostics:**
```
Line 3: error: Expected expression after '='
  y =
     ^
```

**LSP continues:** Completion, hover, etc. still work for `x` and `z`

---

### 5. Performance Targets

#### 5.1 Latency Targets

| Operation | Target Latency | Notes |
|-----------|----------------|-------|
| **Diagnostics** | < 100ms | After keystroke |
| **Completion** | < 50ms | Trigger: `.` or `Ctrl+Space` |
| **Go to Definition** | < 50ms | Instant navigation |
| **Hover** | < 20ms | Show immediately |
| **Find References** | < 200ms | Can be slower (full workspace scan) |
| **Rename** | < 500ms | Complex operation |

---

#### 5.2 Optimization Strategies

##### **Incremental Parsing (Tree-sitter)**

**Concept:** Re-parse only changed regions

```rust
// Initial parse
let mut parser = Parser::new();
parser.set_language(tree_sitter_vela::language()).unwrap();
let tree = parser.parse(&source, None).unwrap();

// User edits: insert "x = 5\n" at line 10
let new_source = source.insert_line(10, "x = 5\n");
let edit = InputEdit {
    start_byte: 245,
    old_end_byte: 245,
    new_end_byte: 251,
    start_position: Point { row: 10, column: 0 },
    old_end_position: Point { row: 10, column: 0 },
    new_end_position: Point { row: 10, column: 6 },
};
tree.edit(&edit);

// Re-parse (only changed region)
let new_tree = parser.parse(&new_source, Some(&tree)).unwrap();
```

**Performance:**
- Without incremental: O(n) parse time (n = file size)
- With incremental: O(k) parse time (k = edit size)
- Typical speedup: 10-100x

---

##### **On-Demand Computation**

**Concept:** Solo computar lo que se necesita

```rust
// BAD: Compute everything upfront
fn on_file_change(file: FileId) {
    let ast = parse(file);          // Always compute
    let typed_ast = typecheck(file); // Always compute
    let diagnostics = check_errors(file); // Always compute
}

// GOOD: Compute on-demand
fn on_file_change(file: FileId) {
    // Just invalidate queries (O(1))
    db.set_source_text(file, new_text);
}

fn on_diagnostics_request(file: FileId) {
    // Compute only if needed (lazy)
    db.diagnostics(file)  // Triggers parse → typecheck → diagnostics
}
```

---

##### **Parallel Processing**

**Concept:** Type-check múltiples archivos en paralelo

```rust
use rayon::prelude::*;

fn typecheck_workspace(files: &[FileId]) -> Vec<Diagnostic> {
    files.par_iter()  // Parallel iterator
        .flat_map(|file| db.diagnostics(*file))
        .collect()
}
```

**Performance:**
- 4-core machine: ~3.5x speedup
- 8-core machine: ~6x speedup

---

### 6. Implementation Stack

#### 6.1 Technology Choices

| Component | Technology | Razón |
|-----------|------------|-------|
| **Language** | Rust | Performance + memory safety |
| **LSP Library** | tower-lsp | Async, modern, usado por rust-analyzer |
| **Parser** | Tree-sitter | Incremental, error recovery |
| **Incremental** | Salsa | Query-based caching (usado por rust-analyzer) |
| **Type Checker** | Custom (Hindley-Milner) | Reuso de compiler |

---

#### 6.2 Code Example

```rust
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct VelaLspServer {
    client: Client,
    db: salsa::Database,
}

#[tower_lsp::async_trait]
impl LanguageServer for VelaLspServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        // Get file from URI
        let file_id = self.db.file_id_from_uri(&uri);
        
        // Get symbol at position
        let symbol = self.db.symbol_at_position(file_id, position)?;
        
        // Get type info
        let type_info = self.db.type_of_symbol(symbol)?;
        
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```vela\n{}: {}\n```", symbol.name, type_info),
            }),
            range: Some(symbol.range),
        }))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        let file_id = self.db.file_id_from_uri(&uri);
        let completions = self.db.completions_at_position(file_id, position)?;
        
        Ok(Some(CompletionResponse::Array(completions)))
    }
    
    // ... more handlers
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| VelaLspServer {
        client,
        db: salsa::Database::default(),
    });
    
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

---

### 7. Testing Strategy

#### 7.1 Unit Tests

```rust
#[test]
fn test_goto_definition() {
    let code = r#"
        fn calculate(a: Number) -> Number { return a * 2 }
        fn main() {
            result = calculate(5)
                     ^^^^^^^^^
        }
    "#;
    
    let db = create_test_db(code);
    let position = Position { line: 3, character: 17 };
    let definition = db.definition_at_position(file_id, position).unwrap();
    
    assert_eq!(definition.line, 1);
    assert_eq!(definition.character, 12);
}
```

---

#### 7.2 Integration Tests

```rust
#[tokio::test]
async fn test_lsp_hover() {
    let (client, server) = create_lsp_test_client().await;
    
    // Open document
    client.did_open(TextDocumentItem {
        uri: "file:///test.vela".parse().unwrap(),
        language_id: "vela".into(),
        version: 1,
        text: "x: Number = 5".into(),
    }).await;
    
    // Request hover
    let hover = client.hover(HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: "file:///test.vela".parse().unwrap() },
            position: Position { line: 0, character: 0 },
        },
        work_done_progress_params: Default::default(),
    }).await.unwrap();
    
    assert!(hover.unwrap().contents.value().contains("Number"));
}
```

---

### 8. Comparación con Otros LSPs

| Feature | Vela LSP | rust-analyzer | TypeScript LSP | Dart LSP |
|---------|----------|---------------|----------------|----------|
| **Incremental** | ✅ Salsa | ✅ Salsa | ✅ Custom | ✅ Incremental |
| **Error Recovery** | ✅ Tree-sitter | ✅ Custom | ✅ Custom | ✅ Custom |
| **Latency (Completion)** | < 50ms | ~20ms | ~30ms | ~40ms |
| **Language** | Rust | Rust | TypeScript | Dart |
| **Lines of Code** | ~15K (est.) | ~200K | ~150K | ~100K |

---

## ✅ Criterios de Aceptación

- [x] LSP features priorizado (P0, P1, P2)
- [x] Architecture diagram completo
- [x] Incremental compilation especificado (Salsa)
- [x] Integration con compiler definida (shared codebase)
- [x] Performance targets establecidos (< 100ms diagnostics)
- [x] Technology stack seleccionado (Rust + tower-lsp + Tree-sitter + Salsa)
- [x] Code examples provistos
- [x] Testing strategy definida
- [x] Comparación con rust-analyzer, TypeScript LSP

---

## 🔗 Referencias

### LSP Specification
- [LSP Specification v3.17](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)

### Implementations
- [rust-analyzer](https://github.com/rust-lang/rust-analyzer)
- [TypeScript Language Server](https://github.com/microsoft/TypeScript/tree/main/src/server)
- [Dart Analysis Server](https://github.com/dart-lang/sdk/tree/main/pkg/analysis_server)

### Technologies
- [tower-lsp](https://github.com/ebkalderon/tower-lsp)
- [Salsa](https://github.com/salsa-rs/salsa)
- [Tree-sitter](https://tree-sitter.github.io/tree-sitter/)

---

**Estado:** ✅ Diseño completo  
**Prioridad:** P0 - Crítico para developer experience  
**Siguiente paso:** TASK-000M (DevTools Architecture)
