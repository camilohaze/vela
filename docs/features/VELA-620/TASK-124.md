# TASK-124: Implementar linking pipeline

## 📋 Información General
- **Historia:** VELA-620
- **Estado:** En curso 🔄
- **Fecha:** 2025-12-14

## 🎯 Objetivo
Implementar el pipeline completo de linking que tome el LLVM IR generado y lo convierta en un ejecutable nativo linkeado con la runtime library en C.

## 🔨 Implementación

### Arquitectura del Linking Pipeline

El linking pipeline consistirá en los siguientes pasos:

1. **Generación de código objeto** desde LLVM IR
2. **Compilación de la runtime library** en C
3. **Linking del ejecutable final** con todas las dependencias
4. **Optimizaciones de linking** (strip, LTO, etc.)

### Componentes a Implementar

#### 1. LLVM Code Generation Enhancement
- Extender `ir_to_llvm.rs` para generar archivos objeto directamente
- Soporte para múltiples formatos de salida (objeto, ensamblador, LLVM IR)
- Configuración de target triple y optimizaciones

#### 2. Runtime Library Build Integration
- Integración del build system CMake en el pipeline de compilación
- Compilación automática de la runtime library
- Gestión de dependencias del sistema (pthreads, etc.)

#### 3. Linker Integration
- Uso del linker del sistema (lld, ld, link.exe)
- Configuración de flags de linking apropiados
- Resolución de símbolos entre código generado y runtime

#### 4. Cross-platform Support
- Detección automática del linker disponible
- Configuración específica por plataforma
- Manejo de bibliotecas estáticas vs dinámicas

### API del Linking Pipeline

```rust
pub struct LinkingPipeline {
    llvm_context: Context,
    target_machine: TargetMachine,
    runtime_build_dir: PathBuf,
}

impl LinkingPipeline {
    /// Crear nuevo pipeline de linking
    pub fn new() -> Result<Self, String> {
        // Configurar target machine
        // Preparar directorio de build runtime
    }

    /// Compilar módulo LLVM a código objeto
    pub fn compile_to_object(&self, module: &Module, output_path: &Path) -> Result<(), String> {
        // Generar código objeto desde LLVM IR
        // Configurar optimizaciones
    }

    /// Construir runtime library
    pub fn build_runtime(&self) -> Result<PathBuf, String> {
        // Ejecutar CMake para compilar runtime
        // Retornar path a librería compilada
    }

    /// Linkear ejecutable final
    pub fn link_executable(&self, object_files: &[PathBuf], output_path: &Path) -> Result<(), String> {
        // Invocar linker con todos los archivos objeto
        // Incluir runtime library
        // Configurar flags apropiados
    }

    /// Pipeline completo: IR -> Ejecutable
    pub fn build_executable(&self, ir_module: &IRModule, output_path: &Path) -> Result<(), String> {
        // 1. Generar LLVM IR
        // 2. Compilar a objeto
        // 3. Build runtime
        // 4. Linkear ejecutable
    }
}
```

### Configuración por Plataforma

#### Linux/macOS
```bash
# Compilación objeto
llc -filetype=obj -O3 input.ll -o output.o

# Linking
clang output.o -L/path/to/runtime -lvela_runtime -lpthread -o executable
```

#### Windows
```cmd
# Compilación objeto
llc -filetype=obj -O3 input.ll -o output.obj

# Linking
link output.obj /LIBPATH:path\to\runtime vela_runtime.lib /out:executable.exe
```

### Integración con Compiler Pipeline

El linking pipeline se integrará en el flujo general del compilador:

```rust
// En compiler/src/main.rs o similar
pub fn compile_to_native(input: &str, output: &Path) -> Result<(), String> {
    // 1. Parsear código fuente
    let ast = parser::parse(input)?;

    // 2. Generar IR
    let ir_module = semantic::analyze(ast)?;

    // 3. Generar LLVM IR
    let llvm_generator = LLVMGenerator::new()?;
    let llvm_module = llvm_generator.generate(&ir_module)?;

    // 4. Linking pipeline
    let linker = LinkingPipeline::new()?;
    linker.build_executable(&ir_module, output)?;

    Ok(())
}
```

## ✅ Criterios de Aceptación
- [ ] Código objeto generado correctamente desde LLVM IR
- [ ] Runtime library compila automáticamente
- [ ] Ejecutable final linkea sin errores
- [ ] Cross-platform support (Windows, Linux, macOS)
- [ ] Optimizaciones aplicadas correctamente
- [ ] Manejo apropiado de dependencias del sistema

## 🔗 Referencias
- **Historia:** [VELA-620](https://velalang.atlassian.net/browse/VELA-620)
- **Dependencia:** TASK-123 completada
- **Arquitectura:** Ver `docs/architecture/ADR-XXX-linking-pipeline.md`</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-620\TASK-124.md