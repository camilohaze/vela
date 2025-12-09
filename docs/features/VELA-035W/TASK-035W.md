# TASK-035W: Implementar @select decorator

## 📋 Información General
- **Historia:** VELA-035
- **Estado:** Completada ✅
- **Fecha:** 2025-12-09

## 🎯 Objetivo
Implementar el decorador @select para optimización de re-renders en widgets conectados al store global. Solo re-renderiza si el selector cambia, mejorando el performance.

## 🔨 Implementación

### Arquitectura
- **SelectableWidget trait**: Define la interfaz para widgets con selección optimizada
- **select! macro**: Wrapper que implementa memoización basada en hash del estado seleccionado
- **Optimización**: Evita re-renders innecesarios comparando hashes del estado

### Código Principal
```rust
// packages/ui/src/select.rs
pub trait SelectableWidget: Widget {
    fn selector(&self) -> String;
    fn selected_hash(&self) -> u64;
}

#[macro_export]
macro_rules! select {
    ($widget:ident, $selector:expr) => {
        // Implementación con memoización
    };
}
```

### Integración
- Agregado módulo `select` a `lib.rs`
- Re-export de `SelectableWidget` trait
- Compatible con `@connect` decorator

## ✅ Criterios de Aceptación
- [x] SelectableWidget trait implementado
- [x] select! macro funcional
- [x] Memoización por hash del estado
- [x] Integración con UI framework
- [x] Compila sin errores

## 🔗 Referencias
- **Jira:** [VELA-035W](https://velalang.atlassian.net/browse/VELA-035W)
- **Historia:** [VELA-035](https://velalang.atlassian.net/browse/VELA-035)
- **Inspiración:** React-Redux selectors, Reselect library