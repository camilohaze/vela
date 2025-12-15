# TASK-167: Diseñar FFI system

## 📋 Información General
- **Historia:** VELA-1179
- **Estado:** Completada ✅
- **Fecha:** 2025-12-15

## 🎯 Objetivo
Diseñar un sistema completo de Foreign Function Interface (FFI) que permita a Vela llamar código C de manera segura, ergonómica y performante.

## 🔨 Implementación

### Diseño del Sistema FFI

#### Arquitectura de Tres Capas

**1. Capa de Sintaxis (High-Level)**
- Declaraciones `extern "C"` con type checking
- Conversión automática de tipos Vela ↔ C
- Validación de safety en compile-time

**2. Capa de Bridge (Runtime)**
- Gestión automática de memoria y lifetimes
- Conversión de calling conventions
- Error handling y recovery

**3. Capa Core (Low-Level)**
- Interfaz con libffi o implementación custom
- Platform-specific optimizations
- Raw FFI calls

#### Mapeo de Tipos Seguro

**Primitivos con Validación:**
```rust
// En compiler/src/ffi/types.rs
pub enum FfiType {
    Bool,           // C: bool
    Number,         // C: int32_t (con range check)
    Float,          // C: double
    String,         // C: const char* (UTF-8 validated)
    Pointer(Box<FfiType>), // C: T*
    Struct(String), // C: struct name
}
```

**Conversión Automática:**
```rust
// Type conversion logic
impl FfiType {
    fn to_c_type(&self) -> CType {
        match self {
            FfiType::Bool => CType::Bool,
            FfiType::Number => CType::Int32,
            FfiType::Float => CType::Double,
            FfiType::String => CType::ConstCharPtr,
            // ... más mappings
        }
    }
}
```

### Sintaxis Extern Diseñada

#### Declaraciones Básicas
```vela
// Función externa simple
extern "C" fn abs(value: Number) -> Number;

// Con librería específica
extern "C" from "libm.so" fn sin(angle: Float) -> Float;

// Bloque de declaraciones
extern "C" {
  fn malloc(size: usize) -> *mut u8;
  fn free(ptr: *mut u8);
  fn strlen(s: *const u8) -> usize;
}
```

#### Structs Externos
```vela
extern "C" struct File {
  fd: Number,
  flags: Number,
}

extern "C" struct Stat {
  size: Number,
  mtime: Number,
  mode: Number,
}
```

### Bridge Runtime Architecture

#### Memory Management
```rust
// En runtime/src/ffi/bridge.rs
pub struct FfiBridge {
    lib_handles: HashMap<String, Library>,
    type_cache: TypeCache,
}

impl FfiBridge {
    pub fn call_function(
        &self,
        lib_name: &str,
        func_name: &str,
        args: &[FfiValue],
        return_type: &FfiType,
    ) -> Result<FfiValue, FfiError> {
        // 1. Load library if not loaded
        // 2. Resolve function symbol
        // 3. Convert arguments to C types
        // 4. Call function via libffi
        // 5. Convert result back to Vela types
        // 6. Handle errors and cleanup
    }
}
```

#### Error Handling
```rust
#[derive(Debug)]
pub enum FfiError {
    LibraryNotFound(String),
    SymbolNotFound(String),
    TypeConversionError(String),
    MemoryError(String),
    CallFailed(String),
}
```

### Safety Guarantees

#### Bounds Checking
- Arrays con length validation
- Pointers con null checks
- String UTF-8 validation

#### Memory Safety
- Ownership tracking
- Automatic cleanup
- No dangling pointers

#### Type Safety
- Compile-time type checking
- Runtime type validation
- Panic on invalid conversions

### Performance Optimizations

#### Zero-Cost Abstractions
- Inline type conversions cuando posible
- Direct FFI calls sin overhead
- Cached library handles

#### Platform-Specific Optimizations
```rust
#[cfg(target_os = "linux")]
mod platform {
    // Linux-specific FFI optimizations
}

#[cfg(target_os = "windows")]
mod platform {
    // Windows-specific FFI handling
}
```

## ✅ Criterios de Aceptación
- [x] **Arquitectura definida** - Tres capas con responsabilidades claras
- [x] **Type mapping completo** - Todos los tipos Vela mapeados a C
- [x] **Sintaxis diseñada** - Declaraciones extern ergonómicas
- [x] **Safety guarantees** - Bounds checking y memory safety
- [x] **Performance considerado** - Zero-cost abstractions
- [x] **Cross-platform** - Soporte Windows/macOS/Linux
- [x] **ADR creado** - docs/architecture/ADR-167-ffi-system-design.md

## 📊 Métricas
- **Complejidad:** Alta (safety crítica)
- **Archivos diseñados:** 8+ (types, bridge, platform-specific)
- **Líneas estimadas:** 600+ líneas
- **Dependencias:** libffi o implementación custom

## 🔗 Referencias
- **Jira:** [TASK-167](https://velalang.atlassian.net/browse/TASK-167)
- **Historia:** [VELA-1179](https://velalang.atlassian.net/browse/VELA-1179)
- **ADR:** docs/architecture/ADR-167-ffi-system-design.md
- **RFC:** FFI Design Document</content>
<parameter name="filePath">C:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-1179\TASK-167.md