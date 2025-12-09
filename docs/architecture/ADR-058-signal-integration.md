# ADR-058: Integración de Signals con Widgets

## Estado
✅ Aceptado

## Fecha
2025-12-09

## Contexto
El sistema UI de Vela tiene widgets declarativos y un sistema reactivo completo con signals, pero actualmente no están integrados. Los widgets se construyen una vez y no responden automáticamente a cambios en el estado reactivo.

Necesitamos integrar signals con widgets para que:
- Los widgets puedan leer signals durante build()
- Los cambios en signals automaticen rebuilds de widgets
- Se mantenga el tracking automático de dependencias
- La UI sea reactiva a cambios de estado

## Decisión
Implementar integración completa de signals con widgets mediante:

### 1. BuildContext Reactivo
**Extender BuildContext** para incluir reactive graph y tracking automático:
```rust
pub struct BuildContext {
    pub reactive_graph: Arc<ReactiveGraph>,
    pub current_widget_path: Vec<String>,
    pub build_stack: Vec<WidgetId>,
}
```

### 2. Widget Reactive Trait
**Nuevo trait ReactiveWidget** que extiende Widget:
```rust
pub trait ReactiveWidget: Widget {
    fn reactive_build(&self, context: &mut ReactiveBuildContext) -> VDomNode;
}
```

### 3. Signal Reading en Build
**Método signal() en BuildContext** para leer signals con tracking:
```rust
impl BuildContext {
    pub fn signal<T: Clone + 'static>(&self, signal: &Signal<T>) -> T {
        // Registra dependencia automáticamente
        // Retorna valor actual
    }
}
```

### 4. Automatic Rebuilds
**Sistema de invalidación** que trigger rebuilds cuando signals cambian:
- Signals notifican al reactive graph
- Graph identifica widgets dependientes
- Widgets se marcan para rebuild
- Scheduler ejecuta rebuilds en próximo frame

## Consecuencias

### Positivas
- ✅ **Reactividad automática**: UI responde automáticamente a cambios de estado
- ✅ **Performance**: Solo rebuilds widgets afectados por cambios
- ✅ **Simplicidad**: Desarrolladores solo leen signals, sistema maneja tracking
- ✅ **Composición**: Funciona con todos los widgets existentes
- ✅ **Debugging**: Tracing de dependencias para debugging

### Negativas
- ⚠️ **Complejidad**: Sistema más complejo de implementar y mantener
- ⚠️ **Overhead**: Tracking automático tiene costo de performance
- ⚠️ **Memory**: Reactive graph consume memoria adicional
- ⚠️ **Learning curve**: Desarrolladores deben entender reactive patterns

## Alternativas Consideradas

### 1. Manual Subscription (Rechazada)
**Descripción**: Widgets se suscriben manualmente a signals
**Rechazada porque**: Error-prone, boilerplate excesivo, fácil olvidar subscriptions

### 2. Pull-based Updates (Rechazada)
**Descripción**: Widgets preguntan por cambios en cada render
**Rechazada porque**: Ineficiente, no escala con muchos widgets

### 3. Signal Props Only (Rechazada)
**Descripción**: Solo props de widgets pueden ser signals
**Rechazada porque**: Limitante, no permite lógica compleja en build()

## Implementación
Ver código en: `runtime/ui/src/reactive_widgets.rs`

## Referencias
- Jira: VELA-058
- Historia: US-13 (Sistema Reactivo Integrado)
- Arquitectura: Basada en ReactiveGraph existente
- Patrón: Similar a SolidJS reactivity model