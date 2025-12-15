# TASK-153: Implementar bridging Swift/Vela

## 📋 Información General
- **Historia:** VELA-1161 (iOS deployment)
- **Estado:** En curso 🔄
- **Fecha:** 2025-01-30
- **Dependencias:** TASK-152 (iOS render engine design)

## 🎯 Objetivo
Implementar el sistema de bridging FFI (Foreign Function Interface) entre el runtime de Vela (Rust) y Swift/Objective-C para iOS, permitiendo la comunicación bidireccional entre ambos lenguajes.

## 🔨 Implementación

### Arquitectura del Bridging

#### 1. **VelaIOSBridge** (Rust → Swift)
- **Ubicación:** `runtime/ios/bridge/`
- **Propósito:** Punto de entrada FFI desde Swift hacia Vela
- **Funciones principales:**
  - `vela_ios_create_runtime()` - Inicializar runtime Vela
  - `vela_ios_render_widget()` - Renderizar widget Vela
  - `vela_ios_update_widget()` - Actualizar widget existente
  - `vela_ios_destroy_widget()` - Liberar recursos del widget
  - `vela_ios_handle_event()` - Procesar eventos desde iOS

#### 2. **Swift Bridging Layer**
- **Ubicación:** `runtime/ios/swift/` (nuevo)
- **Propósito:** Wrappers Swift para consumir FFI de Vela
- **Componentes:**
  - `VelaRuntime.swift` - Wrapper para runtime Vela
  - `VelaWidget.swift` - Protocolo para widgets Vela
  - `VelaEvent.swift` - Tipos de eventos Vela
  - `VelaBridge.swift` - API unificada para Swift

#### 3. **Memory Management**
- **ARC Integration:** Bridging entre Rust ownership y ARC de Swift
- **UIView Pool:** Reutilización de vistas UIKit para performance
- **Reference Counting:** Sincronización entre Rust Rc/Arc y Swift retain/release

### Funciones FFI Implementadas

```rust
// runtime/ios/bridge/ffi.rs
#[no_mangle]
pub extern "C" fn vela_ios_create_runtime(config: *const IOSRuntimeConfig) -> *mut VelaIOSRuntime {
    // Crear runtime con configuración
}

#[no_mangle]
pub extern "C" fn vela_ios_render_widget(
    runtime: *mut VelaIOSRuntime,
    widget_json: *const c_char,
    parent_view: *mut UIView
) -> *mut UIView {
    // Renderizar widget Vela como UIView
}

#[no_mangle]
pub extern "C" fn vela_ios_handle_touch_event(
    runtime: *mut VelaIOSRuntime,
    widget_id: u64,
    event: *const IOSTouchEvent
) -> bool {
    // Procesar evento táctil
}
```

### Swift API

```swift
// runtime/ios/swift/VelaBridge.swift
public class VelaBridge {
    private let runtime: OpaquePointer
    
    public init(config: VelaRuntimeConfig) {
        self.runtime = vela_ios_create_runtime(config.toFFI())
    }
    
    public func renderWidget(json: String, parent: UIView) -> UIView? {
        guard let widgetView = vela_ios_render_widget(runtime, json, parent) else {
            return nil
        }
        return Unmanaged<UIView>.fromOpaque(widgetView).takeRetainedValue()
    }
    
    public func handleEvent(widgetId: UInt64, event: VelaEvent) -> Bool {
        return vela_ios_handle_event(runtime, widgetId, event.toFFI())
    }
    
    deinit {
        vela_ios_destroy_runtime(runtime)
    }
}
```

## ✅ Criterios de Aceptación
- [ ] Bridging FFI implementado con funciones `vela_ios_*`
- [ ] Swift wrappers creados para consumir FFI
- [ ] Memory management correcto entre Rust y Swift
- [ ] UIView pool integrado para reutilización
- [ ] Eventos táctiles propagados correctamente
- [ ] Tests unitarios de bridging (80% cobertura)
- [ ] Documentación técnica completa
- [ ] Compilación exitosa verificada

## 🔗 Referencias
- **Jira:** [TASK-153](https://velalang.atlassian.net/browse/TASK-153)
- **Historia:** [VELA-1161](https://velalang.atlassian.net/browse/VELA-1161)
- **ADR:** [ADR-152](../architecture/ADR-152-ios-render-engine.md)