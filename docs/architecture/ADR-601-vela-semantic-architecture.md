# ADR-601: Vela Semantic Analysis Architecture

## Status
✅ Aceptado

## Fecha
2025-01-15

## Contexto

El análisis semántico es la tercera fase del compilador de Vela, después del lexer y parser. Su responsabilidad es validar que el código fuente sea semánticamente correcto y construir estructuras de datos necesarias para la generación de código.

### Problemas a Resolver

1. **Symbol Resolution**: Resolver referencias a variables, funciones, tipos y otros símbolos
2. **Scope Management**: Gestionar ámbitos léxicos anidados con reglas de shadowing
3. **Name Binding**: Vincular nombres a sus definiciones
4. **Type Checking Integration**: Preparar información para el type checker
5. **Semantic Errors**: Detectar errores como variables no declaradas, redeclaraciones, etc.
6. **Closure Capture**: Detectar variables capturadas por closures

### Requisitos

- ✅ Thread-safe: Análisis concurrente en archivos múltiples
- ✅ Inmutable: Symbol tables inmutables después de construcción
- ✅ Eficiente: O(1) lookups, O(n) construction
- ✅ Error Recovery: Continuar análisis después de errores
- ✅ Spans: Ubicación precisa de símbolos para error reporting
- ✅ Nested Scopes: Soporte para scopes anidados ilimitados
- ✅ Module System: Soporte para imports/exports

## Decisión

Implementar un sistema de análisis semántico modular con las siguientes componentes:

### 1. Symbol Table

```rust
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
    pub scope_id: ScopeId,
    pub is_mutable: bool,
    pub is_captured: bool,
}

pub enum SymbolKind {
    Variable { type_hint: Option<String> },
    Function { params: Vec<String>, return_type: Option<String> },
    Class { base_class: Option<String> },
    Module,
    Import { source: String },
    Parameter,
}

pub struct SymbolTable {
    symbols: HashMap<SymbolId, Symbol>,
    names: HashMap<(ScopeId, String), SymbolId>,
    next_id: AtomicUsize,
}
```

**Características**:
- Hash-based lookups: O(1) average case
- Unique SymbolId per símbolo
- Bidirectional mapping: name → symbol, symbol → name
- Thread-safe con atomic counters

### 2. Scope Hierarchy

```rust
pub struct Scope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub symbols: HashSet<SymbolId>,
}

pub enum ScopeKind {
    Global,
    Module,
    Function,
    Block,
    Class,
    Loop,
}

pub struct ScopeManager {
    scopes: HashMap<ScopeId, Scope>,
    current: ScopeId,
    next_id: AtomicUsize,
}
```

**Características**:
- Tree structure: parent/children links
- Scope stack durante análisis
- Cada scope conoce sus símbolos
- Soporte para shadowing

### 3. Semantic Analyzer

```rust
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    scope_manager: ScopeManager,
    errors: Vec<SemanticError>,
    current_function: Option<SymbolId>,
}

impl SemanticAnalyzer {
    pub fn analyze(&mut self, ast: &Program) -> Result<AnalysisResult>;
    
    fn visit_declaration(&mut self, decl: &Declaration);
    fn visit_statement(&mut self, stmt: &Statement);
    fn visit_expression(&mut self, expr: &Expression);
    
    fn define_symbol(&mut self, name: String, kind: SymbolKind, span: Span);
    fn lookup_symbol(&self, name: &str, span: Span) -> Option<SymbolId>;
}
```

**Algoritmo de Análisis**:
```
1. Crear scope global
2. Primera pasada: Declaraciones de top-level (hoisting)
   - Definir funciones, clases, imports
3. Segunda pasada: Análisis de cuerpos
   - Resolver referencias
   - Validar scopes
   - Detectar closures
4. Validar símbolos usados pero no definidos
5. Generar errores semánticos
```

### 4. Semantic Errors

```rust
pub enum SemanticError {
    UndefinedVariable { name: String, span: Span },
    AlreadyDefined { name: String, original: Span, duplicate: Span },
    NotInScope { name: String, span: Span },
    CannotReassignImmutable { name: String, span: Span },
    InvalidShadowing { name: String, outer: Span, inner: Span },
    UseBeforeDefinition { name: String, use_span: Span, def_span: Span },
    CannotCaptureVariable { name: String, span: Span },
}

impl std::error::Error for SemanticError {}
impl std::fmt::Display for SemanticError {
    // Formateo amigable para errores
}
```

### 5. Name Resolution Algorithm

**Lexical Scoping** (de dentro hacia afuera):

```rust
fn lookup_symbol(&self, name: &str, start_scope: ScopeId) -> Option<SymbolId> {
    let mut current = start_scope;
    
    loop {
        // Buscar en scope actual
        if let Some(symbol_id) = self.names.get(&(current, name.to_string())) {
            return Some(*symbol_id);
        }
        
        // Subir a scope padre
        match self.scopes.get(&current).and_then(|s| s.parent) {
            Some(parent) => current = parent,
            None => return None, // Llegamos a global, no encontrado
        }
    }
}
```

**Complejidad**:
- Lookup: O(d) donde d = profundidad de scopes (típicamente < 10)
- Insert: O(1)
- Memory: O(s + n) donde s = símbolos, n = scopes

### 6. Closure Capture Detection

```rust
struct ClosureAnalyzer {
    function_scopes: HashMap<SymbolId, ScopeId>,
    captured_vars: HashMap<SymbolId, HashSet<SymbolId>>,
}

impl ClosureAnalyzer {
    fn detect_captures(&mut self, func_id: SymbolId, body: &Block) {
        // Variables usadas dentro de la función
        let used = self.collect_used_vars(body);
        
        // Variables definidas en la función
        let defined = self.get_local_symbols(func_id);
        
        // Captures = used - defined
        let captures: HashSet<_> = used.difference(&defined).cloned().collect();
        
        // Marcar como capturadas
        for var_id in captures {
            self.mark_as_captured(var_id);
        }
    }
}
```

### 7. Module System Integration

```rust
pub struct ModuleResolver {
    modules: HashMap<String, ModuleInfo>,
    imports: HashMap<ScopeId, Vec<Import>>,
}

pub struct ModuleInfo {
    pub path: String,
    pub exports: HashSet<SymbolId>,
    pub symbol_table: SymbolTable,
}

impl ModuleResolver {
    pub fn resolve_import(&mut self, import: &Import, current_scope: ScopeId) {
        // Cargar módulo si no está cacheado
        let module = self.load_module(&import.path)?;
        
        // Validar que símbolos existen
        for name in &import.symbols {
            if !module.exports.contains(name) {
                self.errors.push(SemanticError::ImportNotFound { ... });
            }
        }
        
        // Agregar símbolos al scope actual
        for name in &import.symbols {
            let symbol_id = module.symbol_table.lookup(name)?;
            self.define_imported_symbol(current_scope, name, symbol_id);
        }
    }
}
```

## Consecuencias

### Positivas

1. **Separación de Concerns**: Symbol table, scopes y analyzer separados
2. **Reusabilidad**: Symbol table puede ser usado por type checker y codegen
3. **Performance**: O(1) lookups, O(d) name resolution (d pequeño)
4. **Thread Safety**: Estructuras inmutables después de construcción
5. **Error Recovery**: Análisis continúa después de errores
6. **Extensibilidad**: Fácil agregar nuevos tipos de símbolos/scopes

### Negativas

1. **Memory Overhead**: ~200 bytes por símbolo (aceptable)
2. **Two-Pass**: Requiere dos pasadas para hoisting (inevitable)
3. **Closure Detection**: Requiere análisis adicional (necesario)

### Trade-offs

| Aspecto | Decisión | Alternativa | Razón |
|---------|----------|-------------|-------|
| Lookup | HashMap | BTreeMap | O(1) vs O(log n) |
| Scope Storage | HashMap | Arena | Flexibilidad vs Memory locality |
| Error Handling | Vec<Error> | Result early exit | Recolectar todos los errores |
| Mutability | Arc<RwLock> para concurrent | RefCell local | Thread-safety necesaria |

## Alternativas Consideradas

### 1. Single-Pass Analysis (Rechazada)

**Descripción**: Hacer análisis en una sola pasada sin hoisting.

**Pros**:
- Más simple
- Más rápido

**Cons**:
- ❌ No soporta hoisting de funciones (requerido por Vela)
- ❌ No puede resolver forward references
- ❌ Incompatible con spec del lenguaje

### 2. Global Symbol Table sin Scopes (Rechazada)

**Descripción**: Una sola tabla global con fully qualified names.

**Pros**:
- Más simple
- Lookup más rápido

**Cons**:
- ❌ No soporta shadowing
- ❌ Dificulta closures
- ❌ No escala con módulos

### 3. AST Annotations (Rechazada)

**Descripción**: Anotar el AST directamente con información semántica.

**Pros**:
- Información junto al nodo
- No requiere estructuras separadas

**Cons**:
- ❌ Mutabilidad del AST
- ❌ Dificulta análisis concurrente
- ❌ Mezcla concerns

## Referencias

### Papers & Books
- "Engineering a Compiler" (Cooper & Torczon) - Capítulo 5: Semantic Analysis
- "Modern Compiler Implementation in ML" (Appel) - Capítulo 5: Symbol Tables
- "Compilers: Principles, Techniques, and Tools" (Dragon Book) - Capítulo 2.7

### Implementaciones de Referencia
- **Rust Compiler**: `rustc_resolve` crate
  - https://github.com/rust-lang/rust/tree/master/compiler/rustc_resolve
  - Two-pass resolution con namespace system
  
- **TypeScript Compiler**: Symbol tables y binder
  - https://github.com/microsoft/TypeScript/blob/main/src/compiler/binder.ts
  - Multi-pass con hoisting

- **Swift Compiler**: Name binding
  - https://github.com/apple/swift/tree/main/lib/Sema
  - Scope-based resolution

- **Zig Compiler**: Semantic analysis
  - https://github.com/ziglang/zig/blob/master/src/Sema.zig
  - Single-pass con deferred resolution

### Lenguajes Similares
- **Python**: LEGB rule (Local, Enclosing, Global, Built-in)
- **JavaScript**: Lexical scoping con hoisting
- **Ruby**: Method/variable scoping rules

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Symbol definition | O(1) | HashMap insert |
| Symbol lookup | O(d) | d = scope depth (típicamente < 10) |
| Scope creation | O(1) | Atomic counter |
| Full analysis | O(n) | n = AST nodes |
| Closure detection | O(n × m) | n = functions, m = avg symbols |

### Space Complexity

| Structure | Size per Item | Total |
|-----------|--------------|-------|
| Symbol | ~200 bytes | O(s) donde s = símbolos |
| Scope | ~100 bytes | O(k) donde k = scopes |
| Name mapping | ~50 bytes | O(s) |
| **Total** | - | **O(s + k)** ≈ O(s) pues k ≪ s |

### Benchmarks Esperados

| Métrica | Objetivo | Medición |
|---------|----------|----------|
| Symbols/sec | > 100,000 | Symbol table insert |
| Lookups/sec | > 10,000,000 | Name resolution |
| Analysis time | < 1ms/KLOC | Full semantic pass |
| Memory | < 200 bytes/symbol | Overhead por símbolo |

## Implementation Plan

### Phase 1: Core Structures (TASK-RUST-601)
- ✅ Symbol, SymbolKind enums
- ✅ SymbolTable with HashMap
- ✅ Scope, ScopeKind, ScopeManager
- ✅ Basic tests

### Phase 2: Semantic Analyzer (TASK-RUST-602)
- ✅ SemanticAnalyzer struct
- ✅ Two-pass algorithm
- ✅ Declaration visiting
- ✅ Symbol definition/lookup
- ✅ Tests con AST simple

### Phase 3: Scope Resolution (TASK-RUST-603)
- ✅ Nested scope traversal
- ✅ Shadowing rules
- ✅ Lexical scoping
- ✅ Tests de scopes complejos

### Phase 4: Error Handling (TASK-RUST-604)
- ✅ SemanticError enum
- ✅ Error collection
- ✅ Span tracking
- ✅ Error messages
- ✅ Tests de errores

### Phase 5: Advanced Features (Future)
- ⏳ Closure capture detection
- ⏳ Module resolver
- ⏳ Type integration
- ⏳ Incremental analysis

## Testing Strategy

### Unit Tests (Objetivo: 40+ tests)

1. **Symbol Table Tests** (10 tests):
   - Insert/lookup básico
   - Múltiples scopes
   - Shadowing
   - Symbol kinds
   - Thread safety

2. **Scope Tests** (8 tests):
   - Crear scopes
   - Parent/child links
   - Scope kinds
   - Nested traversal

3. **Analyzer Tests** (12 tests):
   - Análisis de funciones
   - Variables locales/globales
   - Parámetros de función
   - Classes y métodos
   - Imports básicos

4. **Error Tests** (10 tests):
   - Undefined variable
   - Redeclaración
   - Shadowing inválido
   - Uso antes de definición
   - Inmutabilidad

### Integration Tests

```rust
#[test]
fn test_full_program_analysis() {
    let source = r#"
        fn outer() {
            x = 10
            
            fn inner() {
                y = x + 5  // x es capturada
                return y
            }
            
            return inner()
        }
    "#;
    
    let ast = parse(source)?;
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(&ast)?;
    
    assert_eq!(analyzer.errors.len(), 0);
    assert!(result.symbol_table.lookup("outer").is_some());
    assert!(result.captured_vars.contains(&"x"));
}
```

## Documentation Requirements

### README.md
- ✅ Overview del semantic analysis
- ✅ Uso básico con ejemplos
- ✅ Symbol table API
- ✅ Scope management
- ✅ Error handling
- ✅ Performance tips

### API Documentation
- ✅ Rustdoc completo para todas las structs públicas
- ✅ Ejemplos en cada función principal
- ✅ Links entre tipos relacionados

### Architecture Docs
- ✅ Este ADR (arquitectura completa)
- ✅ Diagramas de flujo
- ✅ Ejemplos de uso

## Example Usage

```rust
use vela_semantic::{SemanticAnalyzer, SymbolTable, ScopeManager};

fn main() {
    // Parse código Vela
    let source = r#"
        x: Number = 10
        
        fn greet(name: String) -> String {
            message: String = "Hello, ${name}"
            return message
        }
        
        result = greet("Vela")
    "#;
    
    let ast = vela_parser::parse(source).unwrap();
    
    // Análisis semántico
    let mut analyzer = SemanticAnalyzer::new();
    
    match analyzer.analyze(&ast) {
        Ok(result) => {
            println!("Símbolos definidos: {}", result.symbol_table.len());
            println!("Scopes creados: {}", result.scope_manager.len());
            
            // Lookup de símbolo
            if let Some(symbol) = result.symbol_table.lookup("greet") {
                println!("Función 'greet' encontrada en {}", symbol.span);
            }
        }
        Err(errors) => {
            for error in errors {
                eprintln!("Error semántico: {}", error);
            }
        }
    }
}
```

## Migration from Python

### Python Original
```python
class SymbolTable:
    def __init__(self):
        self.symbols = {}
        self.scopes = []
    
    def define(self, name, kind):
        self.symbols[name] = Symbol(name, kind)
    
    def lookup(self, name):
        return self.symbols.get(name)
```

### Rust Migration
```rust
pub struct SymbolTable {
    symbols: HashMap<SymbolId, Symbol>,
    names: HashMap<(ScopeId, String), SymbolId>,
    next_id: AtomicUsize,
}

impl SymbolTable {
    pub fn define(&mut self, name: String, kind: SymbolKind, scope: ScopeId) -> SymbolId {
        let id = SymbolId(self.next_id.fetch_add(1, Ordering::SeqCst));
        let symbol = Symbol { name: name.clone(), kind, scope, ... };
        self.symbols.insert(id, symbol);
        self.names.insert((scope, name), id);
        id
    }
    
    pub fn lookup(&self, name: &str, scope: ScopeId) -> Option<&Symbol> {
        let id = self.names.get(&(scope, name.to_string()))?;
        self.symbols.get(id)
    }
}
```

**Mejoras en Rust**:
- ✅ Type safety: SymbolId en lugar de strings
- ✅ Thread safety: AtomicUsize para IDs
- ✅ Ownership: Eliminación automática de símbolos
- ✅ Performance: HashMap con hash óptimo

## Security Considerations

1. **No Unsafe Code**: Todo el análisis es safe Rust
2. **Bounds Checking**: Todos los accesos validados
3. **No Panics**: Uso de Result/Option para errores
4. **Memory Safety**: No memory leaks gracias a ownership

## Future Enhancements

1. **Incremental Analysis**: Solo re-analizar archivos modificados
2. **Parallel Analysis**: Analizar módulos en paralelo
3. **Symbol Caching**: Cachear symbol tables de módulos
4. **IDE Integration**: LSP para autocompletado
5. **Visualization**: Graficar scope trees y symbol tables

## Conclusion

Esta arquitectura provee un sistema de análisis semántico robusto, eficiente y extensible para Vela. El diseño modular permite evolución futura sin romper el API público. La separación entre symbol table, scopes y analyzer facilita testing y mantenibilidad.

---

**Refs**: EPIC-RUST-07, TASK-RUST-601  
**Version**: 1.0.0  
**Author**: Vela Team  
**Date**: 2025-01-15
