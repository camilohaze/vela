# TASK-175: Implementar JIT compilation

## 📋 Información General
- **Historia:** VELA-1184
- **Estado:** En curso 🚧
- **Fecha:** 2025-12-15

## 🎯 Objetivo
Implementar compilación JIT experimental para VelaVM que mejore el rendimiento en hotspots de ejecución mediante compilación dinámica a código nativo.

## 🔨 Implementación

### Arquitectura del JIT
```
vm/src/jit/
├── mod.rs              # Módulo principal JIT
├── profiler.rs         # Detección de hotspots
├── compiler.rs         # Compilador JIT
├── cache.rs           # Cache de código compilado
├── deoptimizer.rs     # Deoptimization
└── config.rs          # Configuración JIT
```

### Componentes Principales

#### 1. Hotspot Profiler (`profiler.rs`)
```rust
pub struct HotspotProfiler {
    call_counts: HashMap<FunctionId, u64>,
    threshold: u32,
}

impl HotspotProfiler {
    pub fn record_call(&mut self, function_id: FunctionId) {
        *self.call_counts.entry(function_id).or_insert(0) += 1;
    }

    pub fn is_hotspot(&self, function_id: FunctionId) -> bool {
        self.call_counts.get(&function_id).unwrap_or(&0) >= &self.threshold
    }
}
```

#### 2. JIT Compiler (`compiler.rs`)
```rust
pub struct JITCompiler {
    context: LLVMContext,
    builder: LLVMBuilder,
    module: LLVMModule,
}

impl JITCompiler {
    pub fn compile(&self, bytecode: &Bytecode, metadata: &FunctionMetadata) -> Result<CompiledCode> {
        // 1. Convertir bytecode a LLVM IR
        // 2. Aplicar optimizaciones
        // 3. Compilar a código máquina
        // 4. Retornar función compilada
    }
}
```

#### 3. Code Cache (`cache.rs`)
```rust
pub struct CodeCache {
    compiled_functions: HashMap<FunctionId, CompiledFunction>,
    max_size: usize,
}

impl CodeCache {
    pub fn store(&mut self, id: FunctionId, code: CompiledFunction) {
        // Implementar LRU eviction si es necesario
        self.compiled_functions.insert(id, code);
    }

    pub fn get(&self, id: &FunctionId) -> Option<&CompiledFunction> {
        self.compiled_functions.get(id)
    }
}
```

#### 4. Deoptimizer (`deoptimizer.rs`)
```rust
pub struct Deoptimizer {
    // Maneja rollback de optimizaciones fallidas
}

impl Deoptimizer {
    pub fn deoptimize(&self, function_id: FunctionId) -> Result<()> {
        // 1. Invalidar código compilado
        // 2. Forzar uso de bytecode interpretado
        // 3. Resetear contadores de profiling
    }
}
```

### Integración con VelaVM
```rust
pub struct VelaVM {
    interpreter: Interpreter,
    jit_compiler: Option<JITCompiler>,
    profiler: HotspotProfiler,
    code_cache: CodeCache,
    config: JITConfig,
}

impl VelaVM {
    pub fn execute_function(&mut self, function_id: FunctionId, args: Vec<Value>) -> Result<Value> {
        // 1. Verificar si hay código JIT disponible
        if let Some(compiled) = self.code_cache.get(&function_id) {
            return compiled.execute(args);
        }

        // 2. Record call para profiling
        self.profiler.record_call(function_id);

        // 3. Si es hotspot y JIT está habilitado, compilar
        if self.config.enabled && self.profiler.is_hotspot(function_id) {
            if let Ok(compiled) = self.jit_compiler.as_ref().unwrap().compile(bytecode, metadata) {
                self.code_cache.store(function_id, compiled);
                return compiled.execute(args);
            }
        }

        // 4. Fallback a interpretación
        self.interpreter.execute_function(function_id, args)
    }
}
```

## ✅ Criterios de Aceptación
- [ ] Hotspot profiler identifica funciones con alta frecuencia de ejecución
- [ ] JIT compiler convierte bytecode a código nativo usando LLVM
- [ ] Code cache almacena y reutiliza código compilado
- [ ] Deoptimization maneja rollback cuando optimizaciones fallan
- [ ] Configuración permite activar/desactivar JIT experimental
- [ ] Benchmarks muestran mejora de 50-200% en hotspots
- [ ] Tests unitarios cubren todos los componentes
- [ ] Integración con VelaVM funciona correctamente

## 🔗 Referencias
- **Jira:** [TASK-175](https://velalang.atlassian.net/browse/TASK-175)
- **Historia:** [VELA-1184](https://velalang.atlassian.net/browse/VELA-1184)
- **ADR:** `docs/architecture/ADR-175-jit-compilation.md`
- **Código:** `vm/src/jit/`

## 📊 Métricas Esperadas
- **Mejora de rendimiento**: 50-200% en hotspots
- **Overhead de profiling**: < 5% en performance normal
- **Tiempo de compilación**: < 100ms por función
- **Hit rate del cache**: > 90% para hotspots