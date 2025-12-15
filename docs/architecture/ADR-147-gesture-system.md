# ADR-147: Arquitectura del Sistema de Gestures

## Estado
✅ Aceptado

## Fecha
2025-01-30

## Contexto
Necesitamos implementar un sistema completo de reconocimiento de gestos para UI interactiva avanzada en Vela. Los gestos incluyen drag, pinch, rotate, swipe, tap, long press, y otros gestos complejos. El sistema debe ser:

- **Preciso**: Reconocimiento confiable de patrones de toque
- **Componible**: Capacidad de combinar múltiples gestos
- **Configurable**: Umbrales y sensibilidades ajustables
- **Performante**: Procesamiento eficiente en tiempo real
- **Thread-safe**: Compatible con el sistema reactivo de Vela

## Decisión
Implementaremos un sistema de gestures inspirado en Flutter y React Native, pero adaptado al paradigma funcional y reactivo de Vela.

### Arquitectura Elegida
```
GestureDetector (Widget)
├── GestureRecognizer (Core Logic)
│   ├── Pointer Events → Gesture Events
│   ├── State Machine per Gesture Type
│   └── Arena System (Gesture Competition)
├── Gesture Types
│   ├── TapGestureRecognizer
│   ├── DragGestureRecognizer
│   ├── PinchGestureRecognizer
│   ├── RotateGestureRecognizer
│   ├── SwipeGestureRecognizer
│   └── LongPressGestureRecognizer
└── Gesture Composition
    ├── Simultaneous Gestures
    ├── Competing Gestures
    └── Gesture Priority
```

### Componentes Principales

#### 1. GestureDetector Widget
Widget declarativo que envuelve otros widgets y detecta gestos:

```vela
GestureDetector {
  onTap: () => handleTap()
  onDrag: (details) => handleDrag(details)
  onPinch: (scale, velocity) => handlePinch(scale, velocity)

  child: MyWidget()
}
```

#### 2. Gesture Recognizers
Clases especializadas para cada tipo de gesto con state machines.

#### 3. Pointer Event System
Sistema de eventos de bajo nivel que alimenta los recognizers.

#### 4. Arena System
Sistema de competición entre gestos para resolver conflictos.

## Consecuencias

### Positivas
- ✅ **API Declarativa**: Sintaxis clara y composable
- ✅ **Precisión**: Reconocimiento confiable de gestos complejos
- ✅ **Performance**: Procesamiento eficiente con state machines
- ✅ **Extensibilidad**: Fácil agregar nuevos tipos de gestos
- ✅ **Integración Reactiva**: Compatible con señales y efectos

### Negativas
- ❌ **Complejidad**: Sistema complejo con múltiples state machines
- ❌ **Configuración**: Muchos parámetros de configuración por gesto
- ❌ **Testing**: Difícil testear interacciones complejas

## Alternativas Consideradas

### 1. Sistema Simple (Rechazado)
**Descripción**: Solo gestos básicos sin competición ni composición.
**Razones de Rechazo**: No soporta casos de uso avanzados como pinch-to-zoom con drag simultáneo.

### 2. Sistema Basado en Eventos (Rechazado)
**Descripción**: Eventos directos sin state machines.
**Razones de Rechazo**: Difícil manejar gestos complejos y composición.

### 3. Librería Externa (Rechazado)
**Descripción**: Integrar librería de gestures existente.
**Razones de Rechazo**: Necesidad de integración nativa con el sistema reactivo de Vela.

## Implementación
Ver código en: `runtime/src/ui/gestures.rs`

## Referencias
- Jira: [VELA-1154](https://velalang.atlassian.net/browse/VELA-1154)
- Flutter Gestures: https://flutter.dev/docs/development/ui/advanced/gestures
- React Native Gestures: https://reactnative.dev/docs/gesture-responder-system