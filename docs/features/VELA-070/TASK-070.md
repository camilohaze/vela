# TASK-070: Implementar bytecode generator desde IR

## 📋 Información General
- **Historia:** VELA-070
- **Epic:** EPIC-06 Compiler Backend (VelaVM)
- **Estado:** En curso ✅
- **Fecha:** 2025-01-30

## 🎯 Objetivo
Implementar un generador de bytecode que traduzca la Representación Intermedia (IR) de Vela al bytecode ejecutable por VelaVM.

## 🔨 Implementación

### Arquitectura del Sistema
```
AST → IR → Bytecode → VelaVM Execution

Donde:
- AST: Árbol de Sintaxis Abstracta (del parser)
- IR: Intermediate Representation (nueva fase)
- Bytecode: Instrucciones para VelaVM
```

### Fases de Implementación

#### 1. Definir Estructura IR
Crear tipos de datos para representar el código en forma intermedia:

```rust
// IR Types
pub enum IRInstruction {
    // Variables y constantes
    DeclareVar { name: String, ty: Type },
    AssignVar { name: String, value: IRExpr },
    LoadConst { value: Value },
    
    // Control flow
    Jump { target: Label },
    JumpIf { condition: IRExpr, target: Label },
    Label { name: String },
    
    // Funciones
    Call { function: String, args: Vec<IRExpr> },
    Return { value: Option<IRExpr> },
    
    // Operaciones
    BinaryOp { op: BinaryOp, left: IRExpr, right: IRExpr },
    UnaryOp { op: UnaryOp, operand: IRExpr },
}

pub struct IRFunction {
    name: String,
    params: Vec<IRParam>,
    body: Vec<IRInstruction>,
    return_type: Type,
}

pub struct IRModule {
    functions: Vec<IRFunction>,
    globals: Vec<IRGlobal>,
}
```

#### 2. Convertidor AST → IR
Implementar transformación del AST a IR:

```rust
pub struct ASTToIRConverter {
    current_function: Option<String>,
    label_counter: usize,
}

impl ASTToIRConverter {
    pub fn convert_program(&mut self, program: &Program) -> IRModule {
        // Convertir cada declaración del programa
    }
    
    pub fn convert_function(&mut self, func: &FunctionDecl) -> IRFunction {
        // Convertir declaración de función
    }
    
    pub fn convert_statement(&mut self, stmt: &Statement) -> Vec<IRInstruction> {
        // Convertir statement individual
    }
}
```

#### 3. Generador IR → Bytecode
Implementar el generador final:

```rust
pub struct IRToBytecodeGenerator {
    bytecode: Bytecode,
    symbol_table: HashMap<String, u16>,
    label_positions: HashMap<String, usize>,
}

impl IRToBytecodeGenerator {
    pub fn generate(&mut self, ir_module: &IRModule) -> Result<Bytecode, CodegenError> {
        // Primera pasada: recolectar labels
        self.collect_labels(ir_module)?;
        
        // Segunda pasada: generar bytecode
        self.generate_bytecode(ir_module)?;
        
        Ok(self.bytecode.clone())
    }
    
    fn collect_labels(&mut self, ir_module: &IRModule) -> Result<(), CodegenError> {
        // Recolectar posiciones de labels
    }
    
    fn generate_bytecode(&mut self, ir_module: &IRModule) -> Result<(), CodegenError> {
        // Generar instrucciones bytecode
    }
}
```

### Optimizaciones IR
El sistema IR permitirá futuras optimizaciones:

- **Constant Folding**: Evaluar expresiones constantes en compile-time
- **Dead Code Elimination**: Remover código unreachable
- **Common Subexpression Elimination**: Reutilizar cálculos comunes
- **Register Allocation**: Asignación óptima de registros

## ✅ Criterios de Aceptación
- [ ] IR types definidos y documentados
- [ ] Convertidor AST→IR implementado
- [ ] Generador IR→Bytecode funcional
- [ ] Tests unitarios para cada componente
- [ ] Tests de integración end-to-end
- [ ] Documentación técnica completa
- [ ] Performance benchmarks

## 🔗 Dependencias
- **TASK-010**: Definir estructura completa de AST ✅
- **TASK-069**: Diseñar bytecode instruction set ✅

## 📊 Métricas Esperadas
- **Complejidad**: IR debe ser más simple que AST para optimizaciones
- **Performance**: Generación < 50ms para programas típicos
- **Coverage**: 90%+ de construcciones del lenguaje soportadas
- **Correctness**: 100% de tests pasando

## 🚀 Beneficios
1. **Optimizaciones**: Base para futuras optimizaciones del compilador
2. **Mantenibilidad**: Código más modular y testeable
3. **Extensibilidad**: Fácil agregar nuevos backends (JS, WASM, LLVM)
4. **Debugging**: Mejor tracing y error reporting