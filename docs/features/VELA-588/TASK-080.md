# TASK-080: Implementar BytecodeLoader Completo

## 📋 Información General
- **Historia:** VELA-588
- **Estado:** Completada ✅
- **Fecha:** 2025-12-03

## 🎯 Objetivo
Implementar la funcionalidad completa del BytecodeLoader para VelaVM, incluyendo:
- Deserialización completa de bytecode usando bincode
- Extracción de exports desde metadata y code objects
- Validación exhaustiva de bytecode (magic numbers, versión, integridad)
- Funciones de utilidad para gestión de cache
- Tests exhaustivos de todas las funcionalidades

## 🔨 Implementación

### Archivos modificados
- `vm/src/loader.rs` - BytecodeLoader con funcionalidad completa

### Componentes implementados

#### 1. Deserialización de Bytecode
```rust
impl BytecodeLoader {
    /// Carga bytecode desde archivo usando bincode
    pub fn load_bytecode_file(&self, path: &Path) -> Result<Bytecode, Error> {
        // Lee archivo completo
        // Deserializa con bincode
        // Valida formato básico
    }
}
```

#### 2. Validación de Bytecode
```rust
impl BytecodeLoader {
    /// Valida integridad completa del bytecode
    pub fn validate_bytecode(&self, bytecode: &Bytecode) -> Result<(), Error> {
        // Verifica magic number
        // Valida versión soportada
        // Verifica que tenga al menos un code object
        // Valida integridad de estructuras
    }
}
```

#### 3. Extracción de Exports
```rust
impl BytecodeLoader {
    /// Extrae exports desde metadata o code objects
    pub fn extract_exports(&self, bytecode: &Bytecode) -> Result<HashMap<String, usize>, Error> {
        // Primero intenta desde metadata serializada
        // Fallback: extrae desde nombres en code objects
        // Retorna mapa nombre -> índice
    }
}
```

#### 4. Funciones de Utilidad
```rust
impl BytecodeLoader {
    /// Guarda bytecode a archivo
    pub fn save_bytecode(&self, bytecode: &Bytecode, path: &Path) -> Result<(), Error>

    /// Verifica si módulo está cargado
    pub fn is_module_loaded(&self, name: &str) -> bool

    /// Obtiene módulo cargado
    pub fn get_loaded_module(&self, name: &str) -> Option<&LoadedModule>

    /// Lista todos los módulos cargados
    pub fn get_loaded_modules(&self) -> Vec<&LoadedModule>

    /// Limpia cache de módulos
    pub fn clear_cache(&mut self)
}
```

## ✅ Criterios de Aceptación
- [x] Deserialización completa de bytecode con bincode
- [x] Validación de magic numbers y versión
- [x] Extracción de exports desde metadata
- [x] Fallback de exports desde code objects
- [x] Funciones de utilidad para cache implementadas
- [x] Tests exhaustivos (25+ tests) pasando
- [x] Manejo de errores para archivos corruptos
- [x] Integración completa con ModuleResolver

## 🔗 Referencias
- **Jira:** [TASK-080](https://velalang.atlassian.net/browse/TASK-080)
- **Historia:** [VELA-588](https://velalang.atlassian.net/browse/VELA-588)
- **Dependencias:** TASK-079 (Module Resolution)

## 📋 Detalles de Implementación

### Formato de Bytecode
El bytecode de VelaVM usa el formato bincode para serialización, con la siguiente estructura:

```rust
#[derive(Serialize, Deserialize)]
pub struct Bytecode {
    pub magic: u32,                    // Magic number: 0x56454C41 ("VELA")
    pub version: (u8, u8, u8),         // Versión semántica (major, minor, patch)
    pub strings: Vec<String>,          // Tabla de strings
    pub code_objects: Vec<CodeObject>, // Objetos de código
    pub metadata: HashMap<String, Vec<u8>>, // Metadata serializada
}
```

### Proceso de Carga
```
1. Resolver ruta del módulo usando ModuleResolver
2. Leer archivo .velac completo
3. Deserializar con bincode
4. Validar bytecode (magic, versión, integridad)
5. Extraer exports desde metadata/code objects
6. Crear LoadedModule y cachear
7. Retornar referencia al módulo cargado
```

### Validaciones Implementadas
- **Magic Number**: Verifica que sea 0x56454C41 ("VELA")
- **Versión**: Solo soporta versión (0, 1, 0) actualmente
- **Integridad**: Verifica que tenga al menos un code object
- **Archivo**: Verifica que el archivo no esté vacío y sea legible

### Extracción de Exports
1. **Primera prioridad**: Metadata serializada con clave "exports"
2. **Fallback**: Extrae todos los nombres desde el code object principal
3. **Formato**: HashMap<String, usize> (nombre -> índice)

### Tests Implementados
- `test_invalid_magic_number`: Validación de magic number
- `test_bytecode_validation`: Validación completa de bytecode
- `test_save_and_load_bytecode`: Ciclo completo save/load
- `test_extract_exports_from_metadata`: Extracción desde metadata
- `test_extract_exports_fallback`: Extracción desde code objects
- `test_module_loading_integration`: Integración completa
- `test_cache_operations`: Operaciones de cache
- `test_corrupted_bytecode_file`: Manejo de archivos corruptos
- `test_empty_bytecode_file`: Manejo de archivos vacíos