# TASK-093: Implementar JSON Encoder Avanzado

## 📋 Información General
- **Historia:** EPIC-07 (Standard Library)
- **Estado:** Completada ✅
- **Fecha:** 2024-12-30
- **Commit:** feat(EPIC-07): implementar TASK-093 JSON encoder avanzado

## 🎯 Objetivo
Implementar un encoder JSON avanzado con características de producción: pretty printing, streaming, configuración personalizable, ordenamiento de claves, límites de profundidad y manejo de casos especiales.

## 🔨 Implementación

### Arquitectura del Encoder
- **JsonEncoder**: Struct principal con configuración personalizable
- **JsonEncoderConfig**: Configuración con opciones de formato, escaping y límites
- **Métodos principales**:
  - `encode()`: Codifica a String
  - `encode_to_writer()`: Streaming a cualquier `std::io::Write`
  - `encode_pretty()`: Constructor para pretty printing
  - `encode_sorted()`: Constructor para ordenamiento de claves

### Características Implementadas

#### 1. Pretty Printing
```rust
let encoder = JsonEncoder::pretty();
let json = encoder.encode(&value);
// Produce JSON con indentación y saltos de línea
```

#### 2. Streaming Encoding
```rust
let mut buffer = Vec::new();
encoder.encode_to_writer(&value, &mut buffer)?;
// Escribe directamente a cualquier writer sin alocar string intermedia
```

#### 3. Configuración Personalizable
```rust
let config = JsonEncoderConfig {
    pretty: true,
    indent: "  ".to_string(),
    sort_keys: true,
    max_depth: 10,
    null_value: "null".to_string(),
    escape_slashes: false,
};
let encoder = JsonEncoder::with_config(config);
```

#### 4. Ordenamiento de Claves
```rust
let encoder = JsonEncoder::with_config(JsonEncoderConfig {
    sort_keys: true,
    ..Default::default()
});
// Ordena claves de objetos alfabéticamente
```

#### 5. Límites de Profundidad
```rust
let encoder = JsonEncoder::with_config(JsonEncoderConfig {
    max_depth: 3,
    ..Default::default()
});
// Trunca estructuras anidadas profundas a null
```

#### 6. Funciones de Conveniencia
```rust
// En stdlib/src/json/mod.rs
pub fn to_json_pretty(value: &JsonValue) -> String
pub fn to_json_sorted(value: &JsonValue) -> String
pub fn encode_to_writer<W: Write>(value: &JsonValue, writer: &mut W) -> std::io::Result<()>
```

### Manejo de Casos Especiales
- **NaN/Infinity**: Convertidos a "null" para cumplimiento JSON
- **Unicode**: Soporte completo con escaping apropiado
- **Strings**: Escaping de comillas, backslashes y caracteres de control
- **Números**: Manejo de enteros grandes y flotantes

## ✅ Criterios de Aceptación
- [x] Encoder básico funcional
- [x] Pretty printing con indentación
- [x] Streaming a writers
- [x] Configuración personalizable
- [x] Ordenamiento de claves
- [x] Límites de profundidad
- [x] Manejo de casos especiales (NaN, Infinity, Unicode)
- [x] Funciones de conveniencia
- [x] Tests unitarios completos (11 tests pasando)
- [x] Documentación completa

## 📊 Métricas de Calidad
- **Tests:** 11/11 pasando
- **Cobertura:** > 90%
- **Complejidad:** Encoder modular y extensible
- **Performance:** Streaming sin alocaciones intermedias

## 🔗 Referencias
- **Jira:** [EPIC-07 TASK-093](https://velalang.atlassian.net/browse/EPIC-07)
- **Código:** `stdlib/src/json/encoder.rs`
- **Tests:** `stdlib/src/json/encoder.rs` (tests integrados)
- **API:** `stdlib/src/json/mod.rs`

## 📁 Archivos Modificados
- `stdlib/src/json/encoder.rs` - Implementación completa del encoder
- `stdlib/src/json/mod.rs` - Exports y funciones de conveniencia
- `docs/features/EPIC-07/TASK-093.md` - Esta documentación