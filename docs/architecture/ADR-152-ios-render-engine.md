# ADR-152: Arquitectura del iOS Render Engine

## Estado
✅ Aceptado

## Fecha
2025-12-14

## Contexto
Necesitamos diseñar cómo Vela renderizará widgets en iOS de manera nativa y eficiente. La arquitectura debe:

- Traducir widgets Vela a componentes UIKit/SwiftUI
- Gestionar estado reactivo entre Vela y iOS
- Manejar eventos táctiles y gestos
- Implementar layout eficiente
- Gestionar memoria correctamente
- Integrarse con el run loop de iOS

## Decisión
Implementar una arquitectura de **Widget Bridge** con los siguientes componentes:

### 1. VelaWidgetRenderer (Factory Pattern)
- Traduce widgets Vela a UIView/UIViewController
- Registry de mapeos widget->UIView
- Widget pooling para performance

### 2. VelaStateManager (Observer Pattern)
- Sincronización bidireccional de señales
- KVO bridging para notificaciones
- Thread-safe state updates

### 3. VelaEventBridge (Adapter Pattern)
- Traducción de UITouch a VelaEvent
- Gesture recognizer integration
- Event bubbling simulation

### 4. VelaLayoutEngine (Yoga Integration)
- Flexbox-like layout system
- Constraint-based layout
- Auto-sizing calculations

## Consecuencias

### Positivas
- **Performance nativa**: Renderizado directo a UIKit/SwiftUI
- **Integración perfecta**: Acceso completo a APIs iOS
- **Reutilización de widgets**: Pooling reduce allocations
- **Estado reactivo**: Sincronización automática con señales Vela
- **Layout flexible**: Sistema de layout potente y familiar

### Negativas
- **Complejidad de bridging**: FFI layer entre Rust y Swift
- **Threading constraints**: UI debe estar en main thread
- **Memory management**: ARC bridging complejo
- **Testing difficulty**: Requiere iOS simulator/device

## Alternativas Consideradas

### 1. **WebView-based Rendering** (Rechazado)
- **Pros**: Fácil implementación inicial
- **Cons**: Performance pobre, no nativo, limitado acceso a APIs
- **Razón de rechazo**: No cumple con requerimiento de "apps nativas"

### 2. **React Native-style Bridge** (Rechazado)
- **Pros**: Patrón probado, comunidad grande
- **Cons**: Overhead de serialización JSON, latencia en updates
- **Razón de rechazo**: Vela necesita performance superior

### 3. **Direct SwiftUI Generation** (Rechazado)
- **Pros**: Sintaxis declarativa similar a Vela
- **Cons**: Limitado a iOS 13+, no UIKit compatibility
- **Razón de rechazo**: Necesitamos soporte amplio de versiones iOS

## Implementación
Ver código en: `runtime/src/mobile/ios/`

## Referencias
- Jira: [VELA-1161](https://velalang.atlassian.net/browse/VELA-1161)
- Documentación: docs/features/VELA-1161/TASK-152.md