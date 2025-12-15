# TASK-168: Implementar sintaxis de extern declarations

## 📋 Información General
- **Historia:** VELA-1179
- **Estado:** Completada ✅
- **Fecha:** 2025-01-30

## 🎯 Objetivo
Implementar la sintaxis completa para declarar funciones y structs externas (FFI) en Vela, permitiendo llamar código C/C++ desde Vela de forma segura y performante.

## 🔨 Implementación

### ✅ Cambios en Lexer (lexer.rs)
- ✅ Agregado `Extern` a `TokenKind` enum
- ✅ Reconocimiento del keyword `extern` en `identifier()` match

### ✅ Cambios en AST (ast.rs)
- ✅ Agregado `ExternDeclaration` struct con campos:
  - `abi: String` - ABI objetivo ("C", "C++", etc.)
  - `library: Option<String>` - Librería opcional
  - `function_name: String` - Nombre de la función
  - `parameters: Vec<Parameter>` - Parámetros
  - `return_type: Option<TypeAnnotation>` - Tipo de retorno opcional
- ✅ Agregado `ExternStructDeclaration` struct para structs C
- ✅ Agregados variants `Extern` y `ExternStruct` a `Declaration` enum

### ✅ Cambios en Parser (parser.rs)
- ✅ Agregado case `TokenKind::Extern` en `parse_declaration()`
- ✅ Implementado `parse_extern_declaration()` que maneja:
  - Parsing de ABI string (e.g., `"C"`)
  - Parsing opcional de librería (`from "library.so"`)
  - Distinción entre funciones (`fn`) y structs (`struct`)
- ✅ Implementado `parse_extern_function_declaration()`
- ✅ Implementado `parse_extern_struct_declaration()`

### 📝 Sintaxis Implementada

#### Funciones Externas
```vela
// Función básica
extern "C" fn strlen(s: *const u8) -> usize;

// Con librería específica
extern "C" from "libc.so" fn printf(format: *const u8, ...) -> i32;

// Sin retorno
extern "C" fn free(ptr: *mut u8);
```

#### Structs Externas
```vela
extern "C" struct tm {
    tm_sec: i32,
    tm_min: i32,
    tm_hour: i32,
    tm_mday: i32,
    tm_mon: i32,
    tm_year: i32,
};

extern "C" from "libcustom.so" struct MyStruct {
    field1: i32,
    field2: *mut u8,
};
```

## ✅ Criterios de Aceptación
- [x] Lexer reconoce `extern` keyword
- [x] AST soporta `ExternDeclaration` y `ExternStructDeclaration`
- [x] Parser maneja sintaxis completa de extern declarations
- [x] Compilación exitosa sin errores
- [x] Tests básicos implementados
- [x] Documentación completa

## 🧪 Tests Implementados
- ✅ `test_extern_function_declaration_basic()` - Función externa básica
- ✅ `test_extern_function_with_library()` - Función con librería específica
- ✅ `test_extern_struct_declaration()` - Struct externa
- ✅ `test_extern_multiple_declarations()` - Múltiples declaraciones
- ✅ `test_extern_different_abis()` - Diferentes ABIs ("C", "C++", "Rust")

## 🔗 Referencias
- **Jira:** [VELA-1179](https://velalang.atlassian.net/browse/VELA-1179)
- **Historia:** [VELA-1179](https://velalang.atlassian.net/browse/VELA-1179)
- **ADR:** [docs/architecture/ADR-167-ffi-system-design.md](../architecture/ADR-167-ffi-system-design.md)