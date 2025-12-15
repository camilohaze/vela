# VELA-1179: Sistema FFI para llamar código C desde Vela

## 📋 Información General
- **Epic:** EPIC-18: FFI & Interop
- **Sprint:** Sprint 62
- **Estado:** En desarrollo 🚧
- **Fecha:** 2025-12-15

## 🎯 Descripción
Como desarrollador, quiero poder llamar código C desde Vela para acceder a librerías nativas del sistema, mejorar performance en operaciones críticas, e integrar con el vasto ecosistema de código C existente.

## 📦 Subtasks Completadas
1. **TASK-167**: Diseñar FFI system - Sistema de FFI con C ✅
2. **TASK-168**: Implementar extern declarations - Sintaxis para funciones externas ✅

## 📦 Subtasks Pendientes
3. **TASK-169**: Implementar C FFI bridge - Bridge entre Vela y C
4. **TASK-170**: Tests de FFI - Tests de correctness del FFI

## 🔨 Implementación

### Arquitectura FFI

#### 1. **Sistema de Tipos FFI**
- Mapeo entre tipos Vela y tipos C
- Conversión automática de tipos primitivos
- Manejo de punteros y referencias
- Strings y arrays

#### 2. **Sintaxis Extern**
- Declaraciones `extern "C"` para funciones C
- Import de librerías dinámicas (.so/.dll/.dylib)
- Callbacks desde C hacia Vela

#### 3. **Bridge Runtime**
- Carga dinámica de librerías
- Resolución de símbolos
- Gestión de memoria compartida
- Error handling

#### 4. **Safety & Performance**
- Bounds checking
- Memory safety guarantees
- Zero-cost abstractions
- Performance comparable a C

### Ejemplos de Uso

#### Llamar funciones matemáticas de C
```vela
// Declarar función externa
extern "C" fn sin(angle: Float) -> Float;
extern "C" fn cos(angle: Float) -> Float;
extern "C" fn sqrt(value: Float) -> Float;

// Usar en Vela
fn calculateDistance(x1: Float, y1: Float, x2: Float, y2: Float) -> Float {
  let dx = x2 - x1;
  let dy = y2 - y1;
  return sqrt(dx * dx + dy * dy);
}
```

#### Integración con librerías del sistema
```vela
// SQLite binding
extern "C" {
  type sqlite3;
  fn sqlite3_open(filename: *const u8, db: *mut *mut sqlite3) -> i32;
  fn sqlite3_exec(db: *mut sqlite3, sql: *const u8, callback: extern fn, arg: *mut c_void, errmsg: *mut *mut u8) -> i32;
  fn sqlite3_close(db: *mut sqlite3) -> i32;
}

service DatabaseService {
  fn query(sql: String) -> Result<List<Row>> {
    // Implementación usando SQLite C API
    // ...
  }
}
```

#### High-performance computing
```vela
// BLAS/LAPACK para operaciones matriciales
extern "C" {
  fn cblas_dgemm(order: i32, transA: i32, transB: i32,
                 m: i32, n: i32, k: i32, alpha: f64,
                 A: *const f64, lda: i32, B: *const f64, ldb: i32,
                 beta: f64, C: *mut f64, ldc: i32);
}

service MatrixOps {
  fn multiply(a: Matrix, b: Matrix) -> Matrix {
    // Multiplicación de matrices usando BLAS
    cblas_dgemm(/* parámetros */);
    return result;
  }
}
```

## 📊 Métricas
- **Subtasks:** 2 completadas, 2 pendientes (50% completado)
- **Archivos creados:** 6 (ADR, documentación, lexer, AST, parser, tests)
- **Archivos a crear:** ~8 (FFI system, bridge, tests)
- **Líneas de código:** ~300 líneas implementadas
- **Complejidad:** Alta (safety crítica)

## ✅ Definición de Hecho
- [x] TASK-167: FFI system diseñado con arquitectura de 3 capas
- [x] TASK-168: Sintaxis extern implementada y funcional
- [ ] TASK-169: Bridge C implementado y probado
- [ ] TASK-170: Tests completos con cobertura >90%
- [ ] Todas las Subtasks completadas (4/4)
- [ ] Sistema FFI funcional con tipos seguros
- [ ] Documentación técnica completa
- [ ] Ejemplos de integración con librerías C populares

## 🔗 Referencias
- **Jira:** [VELA-1179](https://velalang.atlassian.net/browse/VELA-1179)
- **Epic:** [EPIC-18: FFI & Interop](https://velalang.atlassian.net/browse/EPIC-18)
- **RFC:** FFI Design Document
- **Standards:** C ABI specifications</content>
<parameter name="filePath">C:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-1179\README.md