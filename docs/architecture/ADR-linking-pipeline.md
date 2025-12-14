# ADR-XXX: Arquitectura del Linking Pipeline para Backend Nativo

## Estado
✅ Aceptado

## Fecha
2025-12-14

## Contexto
Para completar el backend nativo LLVM de Vela, necesitamos un sistema que tome el LLVM IR generado y lo convierta en ejecutables nativos funcionales. Este proceso requiere:

1. **Generación de código objeto** desde LLVM IR
2. **Compilación de la runtime library** en C
3. **Linking del ejecutable final** con todas las dependencias
4. **Soporte cross-platform** (Windows, Linux, macOS)

El linking pipeline debe integrarse perfectamente con el flujo existente del compilador y manejar todas las complejidades del linking nativo.

## Decisión
Implementar un `LinkingPipeline` como módulo separado en `compiler/src/codegen/linking.rs` que:

- **Usa LLVM TargetMachine** para generar código objeto directamente desde LLVM IR
- **Integra CMake build** de la runtime library desde código Rust
- **Detecta automáticamente** el linker disponible por plataforma
- **Configura flags apropiados** para cada plataforma y linker
- **Proporciona API unificada** para el pipeline completo IR → Ejecutable

## Consecuencias

### Positivas
- **Integración perfecta**: Pipeline completamente integrado con LLVM backend existente
- **Cross-platform**: Soporte nativo para Windows, Linux y macOS sin configuración adicional
- **Build automation**: Runtime library se compila automáticamente cuando se necesita
- **Error handling**: Mensajes claros de error para debugging de linking issues
- **Performance**: Optimizaciones LLVM aplicadas durante codegen
- **Mantenibilidad**: Código modular y bien documentado

### Negativas
- **Dependencia de CMake**: Requiere CMake instalado para build de runtime
- **Complejidad de linking**: Manejo de diferentes linkers y sus flags específicos
- **Target detection**: Lógica adicional para detectar y configurar targets apropiados

## Alternativas Consideradas

### 1. Usar LLVM LLD directamente
**Descripción**: Invocar LLD (LLVM linker) directamente desde Rust
**Ventajas**: Mejor integración con LLVM toolchain, más control sobre linking
**Desventajas**: No soporta linking de C runtime libraries fácilmente, complejidad adicional
**Rechazada porque**: Necesitamos linkear contra librerías C estándar y pthreads, LLD es más limitado

### 2. Generar Makefiles en lugar de usar CMake
**Descripción**: Generar Makefiles directamente desde Rust para build de runtime
**Ventajas**: Menos dependencias externas, más control sobre build process
**Desventajas**: Cross-platform más complejo, mantenimiento de Makefiles manual
**Rechazada porque**: CMake proporciona mejor soporte cross-platform y es estándar en proyectos C/C++

### 3. Linking solo en tiempo de ejecución
**Descripción**: Generar código objeto y linkear dinámicamente en runtime
**Ventajas**: Build más rápido, linking lazy
**Desventajas**: Complejidad de runtime linking, dependencias dinámicas, peor performance
**Rechazada porque**: Queremos ejecutables standalone, no linking dinámico complejo

## Implementación

### API Principal

```rust
pub struct LinkingPipeline {
    target_machine: TargetMachine,
    runtime_build_dir: PathBuf,
}

impl LinkingPipeline {
    pub fn new() -> Result<Self, String>
    pub fn compile_to_object(&self, module: &Module, output_path: &Path) -> Result<(), String>
    pub fn build_runtime(&self) -> Result<PathBuf, String>
    pub fn link_executable(&self, object_files: &[PathBuf], output_path: &Path) -> Result<(), String>
    pub fn build_executable(&self, ir_module: &IRModule, output_path: &Path) -> Result<(), String>
}
```

### Configuración por Plataforma

**Linux/macOS:**
```bash
# Compilación objeto
llc -filetype=obj -O3 input.ll -o output.o

# Linking
clang output.o -L/path/to/runtime -lvela_runtime -lpthread -o executable
```

**Windows:**
```cmd
# Compilación objeto
llc -filetype=obj -O3 input.ll -o output.obj

# Linking
link output.obj /LIBPATH:path\to\runtime vela_runtime.lib /out:executable.exe
```

### Integración con Compiler

```rust
// En compiler/src/main.rs o similar
pub fn compile_to_native(input: &str, output: &Path) -> Result<(), String> {
    // 1. Parsear y generar IR
    let ir_module = compile_to_ir(input)?;

    // 2. Linking pipeline
    let linker = LinkingPipeline::new()?;
    linker.build_executable(&ir_module, output)?;

    Ok(())
}
```

## Referencias
- **Jira:** [VELA-620](https://velalang.atlassian.net/browse/VELA-620)
- **TASK:** [TASK-124](https://velalang.atlassian.net/browse/TASK-124)
- **Dependencias:** TASK-121, TASK-122, TASK-123 completadas
- **Documentación:** Ver `docs/features/VELA-620/TASK-124.md`

## Implementación
Ver código en: `compiler/src/codegen/linking.rs`</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-linking-pipeline.md