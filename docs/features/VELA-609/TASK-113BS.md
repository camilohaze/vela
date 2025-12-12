# TASK-113BS: Implementar hot reload de config

## 📋 Información General
- **Historia:** VELA-609
- **Estado:** Completada ✅
- **Fecha:** 2025-12-11

## 🎯 Objetivo
Implementar sistema completo de hot reload que permita actualizar configuración sin reiniciar servicios, con notificaciones de cambios, debounce, y manejo robusto de errores.

## 🔨 Implementación

### Arquitectura de Hot Reload
1. **HotReloadManager**: Manager central que coordina múltiples loaders
2. **ConfigChangeEvent**: Eventos de cambio con estado y metadata
3. **Callbacks System**: Notificaciones síncronas de cambios
4. **Debounce System**: Evita reloads excesivos con ventana de tiempo configurable
5. **Broadcast Channel**: Canal async para suscripciones a cambios

### Características Implementadas
- **File Watching**: Monitoreo automático de archivos de configuración
- **Debounce**: Configurable (default 500ms) para evitar reloads excesivos
- **Error Handling**: Reloads fallidos no detienen el sistema
- **Multiple Loaders**: Soporte para múltiples instancias de ConfigLoader
- **Event Broadcasting**: Canal tokio::broadcast para notificaciones async
- **Callbacks**: Sistema síncrono de callbacks para lógica inmediata

### HotReloadManager API
```rust
let mut manager = HotReloadManager::new();

// Registrar loaders
manager.register_loader("app".to_string(), app_config)?;
manager.register_loader("db".to_string(), db_config)?;

// Agregar callbacks
manager.add_change_callback(|event| {
    match event.reload_state {
        ReloadState::Success => println!("Config reloaded successfully"),
        ReloadState::Failed(err) => eprintln!("Reload failed: {}", err),
        _ => {}
    }
});

// Iniciar hot reload
manager.start()?;

// Suscribirse a cambios
let mut rx = manager.change_channel();
while let Some(event) = rx.recv().await {
    // Handle event
}
```

### Builder Pattern
```rust
let manager = HotReloadBuilder::new()
    .with_loader("app".to_string(), app_config)?
    .with_callback(|event| { /* handle change */ })
    .with_debounce(Duration::from_millis(300))
    .build()?;
```

### Archivos generados
- `compiler/src/hot_reload.rs` - HotReloadManager y sistema completo
- `compiler/src/hot_reload_tests.rs` - Tests unitarios (15 tests)
- `compiler/src/lib.rs` - Módulos actualizados

## ✅ Criterios de Aceptación
- [x] HotReloadManager coordina múltiples ConfigLoaders
- [x] File watching automático con notify crate
- [x] Sistema de debounce configurable
- [x] Callbacks síncronos para notificaciones inmediatas
- [x] Broadcast channel para suscripciones async
- [x] Manejo robusto de errores durante reload
- [x] 15 tests unitarios pasando
- [x] Documentación completa generada

## 🔗 Referencias
- **Jira:** [TASK-113BS](https://velalang.atlassian.net/browse/TASK-113BS)
- **Historia:** [VELA-609](https://velalang.atlassian.net/browse/VELA-609)
- **Dependencia:** TASK-113BR (@config decorator)