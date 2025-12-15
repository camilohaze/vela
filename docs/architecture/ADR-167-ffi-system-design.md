# ADR-167: Diseño del Sistema FFI para Vela

## Estado
✅ Aceptado

## Fecha
2025-12-15

## Contexto
Vela necesita interoperabilidad con código C existente para:
- Acceder a librerías del sistema operativo
- Integrar con el vasto ecosistema de código C/C++
- Mejorar performance en operaciones críticas
- Reutilizar algoritmos optimizados en C

El desafío es proporcionar una interfaz segura y ergonómica mientras se mantiene la filosofía de Vela de seguridad y simplicidad.

## Decisión
Implementar un sistema FFI de tres capas:

1. **Sintaxis de Alto Nivel**: Declaraciones `extern` con type safety
2. **Bridge Runtime**: Gestión automática de conversión de tipos y memoria
3. **FFI Core**: Interfaz de bajo nivel con libffi o equivalente

### Arquitectura Elegida

```
┌─────────────────┐
│   Vela Code     │  ← Sintaxis extern
└─────────────────┘
         │
    Type Mapping   ← Conversión automática
         │
┌─────────────────┐
│  FFI Bridge     │  ← Gestión de memoria y llamadas
└─────────────────┘
         │
    libffi/calling
         │
┌─────────────────┐
│   C Libraries   │  ← Código C nativo
└─────────────────┘
```

## Consecuencias

### Positivas
- **Type Safety**: Conversión automática y validación de tipos
- **Memory Safety**: Gestión automática de lifetimes y ownership
- **Ergonomics**: Sintaxis simple similar a Rust/Kotlin
- **Performance**: Zero-cost abstractions cuando es posible
- **Portability**: Funciona en todas las plataformas soportadas

### Negativas
- **Complejidad**: Sistema complejo de type mapping
- **Runtime Overhead**: Validaciones en runtime para safety
- **Limited Generics**: No todos los tipos genéricos se pueden mapear
- **C ABI Only**: Solo interoperabilidad con C ABI (no C++ directamente)

## Alternativas Consideradas

### 1. FFI Manual (como LuaJIT FFI)
```vela
// Alternativa: FFI manual
let lib = ffi.load("mylib.so")
let func = lib.get("myfunction", "pointer")
func.call(args...)
```
**Rechazada porque**: Demasiado error-prone, no type-safe, compleja para usuarios.

### 2. Bindings Generados (como bindgen)
```vela
// Alternativa: Bindings generados automáticamente
// Generar bindings desde headers C automáticamente
```
**Rechazada porque**: Complejidad de tooling, headers parsing, mantenimiento.

### 3. Solo C ABI Directo (como Go)
```vela
// Alternativa: Solo C ABI
import "C"
// Usar cgo-style imports
```
**Rechazada porque**: No es idiomático en Vela, requiere preprocesamiento.

## Mapeo de Tipos

### Primitivos
| Vela Type | C Type | Conversión |
|-----------|--------|------------|
| `Bool` | `bool` | Directa |
| `Number` | `int32_t` | Range check |
| `Float` | `double` | Directa |
| `String` | `const char*` | UTF-8 validation |

### Compuestos
| Vela Type | C Type | Estrategia |
|-----------|--------|------------|
| `struct` | `struct` | Field mapping |
| `enum` | `enum` | Value mapping |
| `Option<T>` | `T*` | Null checking |
| `Result<T,E>` | Custom | Error handling |

### Arrays y Punteros
- `List<T>` → `T*` con length parameter
- `*T` → Raw pointers (unsafe)
- `&T` → Const pointers

## Sintaxis Extern

### Declaraciones Básicas
```vela
// Función externa
extern "C" fn strlen(s: *const u8) -> usize;

// Con librería específica
extern "C" from "mylib.so" fn my_func(x: i32) -> i32;

// Múltiples funciones
extern "C" {
  fn func1(a: i32) -> i32;
  fn func2(b: f64) -> f64;
}
```

### Structs Externos
```vela
extern "C" struct File {
  fd: i32,
  flags: u32,
}

extern "C" fn open(path: *const u8, flags: i32) -> File;
```

## Implementación

### Fase 1: Core FFI (TASK-167)
- [x] Diseño de type mapping
- [x] Arquitectura de bridge
- [x] Selección de libffi vs custom

### Fase 2: Sintaxis Extern (TASK-168)
- [ ] Parser para `extern` declarations
- [ ] AST nodes para extern functions/structs
- [ ] Type checking de extern declarations

### Fase 3: Bridge Runtime (TASK-169)
- [ ] Implementación del bridge en Rust
- [ ] Gestión de memoria y lifetimes
- [ ] Error handling y panics

### Fase 4: Tests (TASK-170)
- [ ] Tests de type conversion
- [ ] Tests de memory safety
- [ ] Integration tests con librerías C reales

## Referencias
- Jira: [TASK-167](https://velalang.atlassian.net/browse/TASK-167)
- RFC: FFI Design Document
- Standards: System V ABI, C99 Standard
- Inspiración: Rust FFI, Kotlin Native, Swift C Interop</content>
<parameter name="filePath">C:\Users\cristian.naranjo\Downloads\Vela\docs\architecture/ADR-167-ffi-system-design.md