# TASK-138: Implementar debug info generation

## 📋 Información General
- **Historia:** VELA-1143
- **Estado:** En curso ✅
- **Fecha:** 2025-12-14

## 🎯 Objetivo
Implementar generación de información de debug en el compilador para permitir debugging efectivo del código Vela.

## 🔨 Implementación

### Arquitectura de Debug Info
La información de debug se estructura en varias capas:

1. **Source Maps**: Mapeo entre posiciones de código fuente y bytecode
2. **Symbol Table**: Tabla de símbolos con variables y funciones
3. **Line Info**: Información de línea para cada instrucción bytecode
4. **Variable Scopes**: Alcance de variables locales y parámetros

### Componentes Implementados

#### 1. DebugInfo Struct
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    pub source_maps: HashMap<String, SourceMap>,
    pub symbol_table: SymbolTable,
    pub line_info: Vec<LineInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    pub file_path: String,
    pub line_mappings: Vec<LineMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineMapping {
    pub source_line: usize,
    pub source_column: usize,
    pub bytecode_offset: usize,
}
```

#### 2. Symbol Table
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTable {
    pub functions: HashMap<String, FunctionSymbol>,
    pub variables: HashMap<String, VariableSymbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSymbol {
    pub name: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub parameters: Vec<ParameterInfo>,
    pub locals: Vec<LocalInfo>,
}
```

#### 3. Variable Information
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableSymbol {
    pub name: String,
    pub var_type: String,
    pub scope: VariableScope,
    pub location: VariableLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableLocation {
    Stack(usize),        // Offset en el stack
    Register(usize),     // Registro de VM
    Global(String),      // Variable global
}
```

### Modificaciones al Compilador

#### 1. BytecodeProgram Extended
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BytecodeProgram {
    pub functions: Vec<BytecodeFunction>,
    pub constants: Vec<Value>,
    pub debug_info: Option<DebugInfo>,  // Nueva metadata de debug
}
```

#### 2. IRToBytecodeGenerator Enhanced
```rust
pub struct IRToBytecodeGenerator {
    // ... campos existentes ...
    debug_info: DebugInfo,
    current_source_location: Option<SourceLocation>,
}
```

### Generación de Debug Info

#### Source Map Generation
```rust
impl IRToBytecodeGenerator {
    fn record_source_location(&mut self, line: usize, column: usize) {
        self.current_source_location = Some(SourceLocation { line, column });
    }

    fn emit_instruction(&mut self, opcode: Opcode) {
        let offset = self.code.len();
        self.code.push(opcode as u8);

        // Record line mapping
        if let Some(location) = self.current_source_location {
            self.debug_info.line_info.push(LineInfo {
                bytecode_offset: offset,
                source_line: location.line,
                source_column: location.column,
            });
        }
    }
}
```

#### Variable Tracking
```rust
fn declare_variable(&mut self, name: String, var_type: String, location: VariableLocation) {
    self.debug_info.symbol_table.variables.insert(name.clone(), VariableSymbol {
        name,
        var_type,
        scope: VariableScope::Local,
        location,
    });
}
```

## ✅ Criterios de Aceptación
- [x] Debug info se genera durante compilación
- [x] Source maps mapean correctamente líneas fuente a bytecode
- [x] Tabla de símbolos incluye variables y funciones
- [x] Información de debug es serializable con bytecode
- [x] Tests unitarios pasan para debug info generation

## 🔗 Referencias
- **Jira:** [TASK-138](https://velalang.atlassian.net/browse/VELA-1143)
- **Historia:** [VELA-1143](https://velalang.atlassian.net/browse/VELA-1143)
- **ADR:** docs/architecture/ADR-138-debug-info-generation.md