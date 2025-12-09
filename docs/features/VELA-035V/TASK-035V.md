# TASK-035V: Implementar @connect decorator

## 📋 Información General
- **Historia:** VELA-035 (EPIC-03D State Management)
- **Estado:** Completada ✅
- **Fecha:** 2025-12-09

## 🎯 Objetivo
Implementar el decorador `@connect` para conectar widgets al store global de state management.

## 🔨 Implementación

### 1. UI Framework (packages/ui)
- Agregado módulo `connect.rs` con trait `ConnectedWidget` y macro `connect!`
- Permite conectar cualquier widget al store global
- Inspirado en React-Redux `connect` y Flutter Provider

### 2. Re-exports
- Re-exportado macro y trait en `ui/src/lib.rs`

## ✅ Sintaxis Implementada

```vela
@connect(store)
component CounterWidget {
  // El widget recibe el estado global como prop
}
```

## 🔗 Referencias
- **Jira:** [VELA-035V](https://velalang.atlassian.net/browse/VELA-035V)
- **Historia:** [VELA-035](https://velalang.atlassian.net/browse/VELA-035)
- **Dependencias:** TASK-035U (dispatch keyword), TASK-035T (Action/Reducer types)