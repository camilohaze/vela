# TASK-035X: Implementar @persistent decorator

## 📋 Información General
- **Historia:** VELA-035
- **Estado:** Completada ✅
- **Fecha:** 2025-12-09

## 🎯 Objetivo
Implementar el decorador @persistent para persistencia automática del store. Permite guardar y cargar el estado del store automáticamente entre sesiones.

## 🔨 Implementación

### Arquitectura
- **PersistentStore trait**: Define la interfaz para stores con persistencia
- **PersistentStoreImpl**: Wrapper que implementa persistencia automática
- **persistent! macro**: Decorador para aplicar persistencia a un store
- **Persistencia**: localStorage en WASM, archivos en desktop

### Código Principal
```rust
// packages/state-management/src/persistent.rs
pub trait PersistentStore<T>: Store<T> {
    fn persistence_key(&self) -> &str;
    fn save_state(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn load_state(&self) -> Result<(), Box<dyn std::error::Error>>;
}

#[macro_export]
macro_rules! persistent {
    ($store:expr, $key:expr) => {
        Arc::new(PersistentStoreImpl::new($store, $key.to_string()))
    };
}
```

### Integración
- Agregado módulo `persistent` a `lib.rs`
- Re-export de `PersistentStore` y `PersistentStoreImpl`
- Compatible con cualquier `Store<T>` que implemente Serialize/Deserialize

## ✅ Criterios de Aceptación
- [x] PersistentStore trait implementado
- [x] PersistentStoreImpl con guardado/carga automática
- [x] persistent! macro funcional
- [x] Persistencia en localStorage (WASM) y archivos (desktop)
- [x] Compila sin errores

## 🔗 Referencias
- **Jira:** [VELA-035X](https://velalang.atlassian.net/browse/VELA-035X)
- **Historia:** [VELA-035](https://velalang.atlassian.net/browse/VELA-035)
- **Inspiración:** Redux-persist, NgRx Store Devtools