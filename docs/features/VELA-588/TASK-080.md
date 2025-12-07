# TASK-080: Bytecode Loader Implementation

## 📋 Información General
- **Historia:** VELA-588 (US-18: Module Loader)
- **Estado:** Pendiente ⏳
- **Fecha:** 2025-01-07

## 🎯 Objetivo
Implementar el cargador de bytecode que permita:
- Leer archivos .velac compilados
- Parsear el formato de bytecode de VelaVM
- Crear instancias de Module con el bytecode cargado
- Resolver símbolos entre módulos

## 🔨 Implementación

### Archivos generados
- `vm/bytecode_loader.vela` - Implementación del BytecodeLoader
- `vm/module.vela` - Definición de la estructura Module

### Componentes

#### 1. BytecodeLoader Class
```vela
class BytecodeLoader {
  fn loadFromFile(path: String) -> Result<Module>
  fn loadFromBytes(bytes: ByteArray) -> Result<Module>
  fn validateBytecode(bytecode: ByteArray) -> Result<void>
  fn parseHeader(bytecode: ByteArray) -> Result<ModuleHeader>
  fn parseSymbols(bytecode: ByteArray) -> Result<List<Symbol>>
  fn parseCode(bytecode: ByteArray) -> Result<ByteArray>
}
```

#### 2. Module Struct
```vela
struct Module {
  name: String
  header: ModuleHeader
  symbols: List<Symbol>
  code: ByteArray
  dependencies: List<String>
  exports: List<String>
}
```

#### 3. ModuleHeader Struct
```vela
struct ModuleHeader {
  version: Number
  timestamp: Number
  entryPoint: Option<String>
  flags: ModuleFlags
}
```

## ✅ Criterios de Aceptación
- [ ] Carga de archivos .velac funcionando
- [ ] Validación de formato de bytecode
- [ ] Parsing correcto de headers
- [ ] Extracción de símbolos exportados
- [ ] Manejo de errores para archivos corruptos
- [ ] Integración con ModuleResolver

## 🔗 Referencias
- **Jira:** [TASK-080](https://velalang.atlassian.net/browse/TASK-080)
- **Historia:** [VELA-588](https://velalang.atlassian.net/browse/VELA-588)
- **Dependencias:** TASK-079 (Module Resolution)

## 📋 Formato de Bytecode

### Estructura del Archivo .velac
```
[Header - 64 bytes]
  Magic: "VELA" (4 bytes)
  Version: u32 (4 bytes)
  Timestamp: u64 (8 bytes)
  Flags: u32 (4 bytes)
  Symbol Count: u32 (4 bytes)
  Code Size: u32 (4 bytes)
  Dependency Count: u32 (4 bytes)
  Export Count: u32 (4 bytes)
  Reserved: 24 bytes

[Symbols Table]
  For each symbol:
    Name Length: u16
    Name: [bytes]
    Type: u8 (0=function, 1=class, 2=variable)
    Offset: u32

[Dependencies]
  For each dependency:
    Name Length: u16
    Name: [bytes]

[Exports]
  For each export:
    Name Length: u16
    Name: [bytes]

[Code Section]
  Bytecode instructions...
```

### Parsing Process
```
1. Read header (first 64 bytes)
2. Validate magic number and version
3. Read symbols table
4. Read dependencies list
5. Read exports list
6. Read code section
7. Create Module instance
8. Return Result<Module>
```