# ADR-061: Sistema de Patching para DOM

## Estado
✅ Aceptado

## Fecha
2025-12-09

## Contexto
Necesitamos un sistema eficiente para aplicar los cambios calculados por el algoritmo de diffing al DOM real. Este sistema debe:

- Aplicar patches en el orden correcto para mantener consistencia
- Manejar todos los tipos de operaciones (insert, remove, replace, update)
- Ser eficiente en términos de operaciones DOM
- Proporcionar rollback en caso de errores

## Decisión
Implementar sistema de patching con:

1. **Aplicación secuencial** de patches en orden topológico
2. **Validación de precondiciones** antes de cada patch
3. **Rollback automático** en caso de fallos
4. **Optimización batch** para operaciones relacionadas

### Arquitectura del Sistema
```rust
pub fn apply_patches(patches: Vec<Patch>) -> Result<(), PatchError> {
    // 1. Validar precondiciones
    validate_patches(&patches)?;

    // 2. Ordenar patches topológicamente
    let ordered_patches = topological_sort(patches);

    // 3. Aplicar con rollback
    let mut applied_patches = Vec::new();

    for patch in ordered_patches {
        match apply_single_patch(patch) {
            Ok(_) => applied_patches.push(patch),
            Err(e) => {
                rollback_patches(applied_patches)?;
                return Err(e);
            }
        }
    }

    Ok(())
}
```

### Tipos de Patches Soportados
```rust
pub enum Patch {
    Insert { path: VDomPath, node: VDomNode },
    Remove { path: VDomPath },
    Replace { path: VDomPath, new_node: VDomNode },
    UpdateText { path: VDomPath, text: String },
    UpdateAttributes { path: VDomPath, attrs: HashMap<String, String> },
    UpdateProperties { path: VDomPath, props: HashMap<String, serde_json::Value> },
    UpdateEvents { path: VDomPath, events: HashMap<String, String> },
}
```

## Consecuencias

### Positivas
- ✅ **Consistencia**: Aplicación ordenada mantiene DOM consistente
- ✅ **Robustez**: Rollback automático previene estados corruptos
- ✅ **Performance**: Optimizaciones batch reducen operaciones DOM
- ✅ **Type Safety**: Patches tipados evitan errores runtime
- ✅ **Extensibilidad**: Fácil agregar nuevos tipos de patches

### Negativas
- ❌ **Complejidad**: Lógica de ordenamiento y rollback compleja
- ❌ **Overhead**: Validación agrega overhead en runtime
- ❌ **Memory**: Almacenamiento de patches aplicados para rollback
- ❌ **Coordinación**: Requiere coordinación con diffing algorithm

## Alternativas Consideradas

### 1. Aplicación Directa Sin Ordenamiento
**Descripción**: Aplicar patches en orden de generación
**Rechazada porque**: Puede causar inconsistencias si patches dependen entre sí

### 2. Patching Sin Rollback
**Descripción**: Aplicar patches sin capacidad de deshacer
**Rechazada porque**: Estados corruptos del DOM difíciles de recuperar

### 3. Patching Lazy
**Descripción**: Aplicar patches solo cuando se acceden
**Rechazada porque**: Complica el modelo mental, performance impredecible

## Referencias
- Jira: [VELA-061](https://velalang.atlassian.net/browse/VELA-061)
- Documentación: React Reconciliation, Vue Patching
- Código: `runtime/ui/src/patch.rs`

## Implementación
Ver código en: `runtime/ui/src/patch.rs`

El sistema incluye:
- 7 tipos de patches soportados
- Aplicación secuencial con validación
- Sistema de rollback automático
- Tests exhaustivos de correctness