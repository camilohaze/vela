# TASK-098: Implementar vela run

## 📋 Información General
- **Historia:** US-22 (Como desarrollador, quiero un CLI para gestionar proyectos)
- **Epic:** EPIC-08 Tooling (CLI)
- **Estado:** Completada ✅
- **Fecha:** 2025-01-30

## 🎯 Objetivo
Implementar el comando `vela run` para ejecutar bytecode compilado de Vela, completando el ciclo compile-and-run del toolchain.

## 🔨 Implementación

### Arquitectura del Comando Run

El comando `vela run` implementa la siguiente arquitectura:

1. **Búsqueda de Bytecode**: Busca archivos `.velac` en el directorio `target/src/`
2. **Carga de Bytecode**: Deserializa el bytecode usando bincode
3. **Conversión de Formato**: Convierte bytecode del compiler al formato VM
4. **Ejecución**: Ejecuta el bytecode en VelaVM
5. **Resultado**: Muestra el resultado de la ejecución

### Conversión de Bytecode

La conversión entre formatos del compiler y VM incluye:

- **Constantes**: Conversión de `Value` del compiler a `Constant` de la VM
- **Instrucciones**: Conversión de opcodes del compiler (0x10 LoadConst) a VM (0x00 LoadConst)
- **Code Objects**: Mapeo de funciones a objetos de código ejecutables

### Código Implementado

#### En `tooling/src/cli/commands.rs`:
```rust
pub fn execute_run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Búsqueda recursiva de archivos .velac
    let velac_files = find_velac_files(&target_dir)?;
    // Carga y ejecución del bytecode
}
```

#### En `compiler/src/lib.rs`:
```rust
fn convert_to_vm_bytecode(&self, program: BytecodeProgram) -> CompileResult<Bytecode> {
    // Conversión de constantes e instrucciones
    // Mapeo de opcodes del compiler a VM
}
```

### Casos de Uso Soportados

- ✅ Ejecución de módulos individuales: `vela run hello`
- ✅ Función main que retorna valores
- ✅ Constantes numéricas y de cadena
- ✅ Conversión automática de bytecode

### Limitaciones Actuales

- ⚠️ Sin soporte para argumentos de línea de comandos
- ⚠️ Solo ejecución de funciones main simples
- ⚠️ Sin manejo de errores runtime avanzado

## ✅ Criterios de Aceptación

- [x] Comando `vela run <module>` funciona
- [x] Búsqueda automática de archivos .velac
- [x] Conversión correcta de bytecode compiler → VM
- [x] Ejecución exitosa de código compilado
- [x] Manejo de errores de carga/ejecución
- [x] Integración completa con `vela build`

## 🔗 Referencias

- **Jira:** [TASK-098](https://velalang.atlassian.net/browse/TASK-098)
- **Historia:** [US-22](https://velalang.atlassian.net/browse/US-22)
- **Epic:** [EPIC-08](https://velalang.atlassian.net/browse/EPIC-08)

## 📊 Métricas de Implementación

- **Archivos modificados:** 3
- **Líneas de código:** ~150
- **Tiempo de ejecución:** < 1ms para bytecode simple
- **Compatibilidad:** Funciona con bytecode generado por `vela build`

## 🧪 Tests Realizados

```bash
# Compilación exitosa
vela build

# Ejecución exitosa
vela run hello
# Result: Value(4631107791820423168)  # 42.0 en IEEE 754
```

## 🔄 Integración con Build System

El comando `vela run` se integra perfectamente con `vela build`:

1. `vela build` genera `.velac` en `target/src/`
2. `vela run <module>` busca y ejecuta el bytecode correspondiente
3. Conversión automática entre formatos garantiza compatibilidad

Esta implementación completa el toolchain básico de Vela, permitiendo el ciclo completo de desarrollo: escribir código → compilar → ejecutar.