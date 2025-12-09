# TASK-058: Integrar signals con widgets

## 📋 Información General
- **Historia:** US-13 (Sistema Reactivo Integrado)
- **Estado:** En desarrollo ✅
- **Fecha:** 2025-12-09
- **Estimación:** 48 horas
- **Dependencias:** TASK-029 (Effect), TASK-054 (Widget Base)

## 🎯 Objetivo
Integrar el sistema de signals reactivos con el framework de widgets para que la UI responda automáticamente a cambios de estado, implementando tracking automático de dependencias durante el build de widgets.

## 🔨 Implementación Técnica

### Arquitectura General
- **Extensión**: BuildContext con reactive graph integrado
- **Tracking**: Automático durante signal.get() en build methods
- **Invalidación**: Sistema de invalidación para rebuilds selectivos
- **Performance**: Lazy evaluation y batching de updates

### 1. ReactiveBuildContext
**Extender BuildContext** con capacidades reactivas:

```rust
pub struct ReactiveBuildContext {
    pub base_context: BuildContext,
    pub reactive_graph: Arc<ReactiveGraph>,
    pub current_widget_id: WidgetId,
    pub dependency_stack: Vec<SignalId>,
}

impl ReactiveBuildContext {
    /// Leer signal con tracking automático
    pub fn signal<T: Clone + 'static>(&mut self, signal: &Signal<T>) -> T {
        // Registra dependencia widget -> signal
        // Retorna valor actual
    }

    /// Leer computed con tracking automático
    pub fn computed<T: Clone + 'static>(&mut self, computed: &Computed<T>) -> T {
        // Registra dependencia widget -> computed
        // Retorna valor actual
    }
}
```

### 2. ReactiveWidget Trait
**Nuevo trait** para widgets que soportan reactividad:

```rust
pub trait ReactiveWidget: Widget {
    /// Build con contexto reactivo
    fn reactive_build(&self, context: &mut ReactiveBuildContext) -> VDomNode;

    /// ID único del widget para tracking
    fn widget_id(&self) -> WidgetId {
        // Default: hash de type_name + key
    }
}
```

### 3. Signal Integration
**Métodos de extensión** para usar signals en widgets:

```rust
pub trait SignalWidgetExt: Widget {
    /// Convertir widget regular a reactive
    fn reactive(self) -> ReactiveWidgetWrapper<Self>
    where
        Self: Sized;
}

impl<T: Widget> SignalWidgetExt for T {
    fn reactive(self) -> ReactiveWidgetWrapper<Self> {
        ReactiveWidgetWrapper { widget: self }
    }
}
```

### 4. Automatic Rebuilds
**Sistema de invalidación** que maneja rebuilds:

```rust
pub struct WidgetInvalidator {
    pub invalid_widgets: HashSet<WidgetId>,
    pub rebuild_queue: VecDeque<WidgetId>,
}

impl WidgetInvalidator {
    /// Marcar widget para rebuild
    pub fn invalidate(&mut self, widget_id: WidgetId) {
        self.invalid_widgets.insert(widget_id);
        self.rebuild_queue.push_back(widget_id);
    }

    /// Procesar cola de rebuilds
    pub fn process_rebuilds(&mut self, widget_tree: &mut WidgetTree) {
        while let Some(widget_id) = self.rebuild_queue.pop_front() {
            if let Some(widget) = widget_tree.get_widget(widget_id) {
                // Trigger rebuild
                widget.rebuild();
            }
        }
    }
}
```

## ✅ Criterios de Aceptación
- [x] BuildContext puede leer signals con tracking automático
- [x] Cambios en signals trigger rebuilds de widgets dependientes
- [x] Sistema funciona con widgets existentes (Text, Button, etc.)
- [x] Performance: rebuilds selectivos, no rebuilds innecesarios
- [x] Tests unitarios para integración signal-widget (≥80% cobertura)
- [x] Documentación completa de API reactiva
- [x] Ejemplos de widgets reactivos funcionales
- [x] Sistema de debugging para dependencias reactivas

## 🧪 Plan de Testing

### Unit Tests (Mínimo 20 tests)
1. **Signal Reading Tests**:
   - BuildContext.signal() registra dependencias
   - Múltiples signals en un widget
   - Signals anidados (computed depende de signal)

2. **Invalidation Tests**:
   - Cambio en signal invalida widgets dependientes
   - No invalida widgets no dependientes
   - Rebuild queue processing

3. **Reactive Widget Tests**:
   - ReactiveWidgetWrapper funciona con widgets existentes
   - reactive_build() vs build() behavior
   - Widget IDs únicos

### Integration Tests
- Widget tree con signals interconectados
- Performance: medir overhead de tracking
- Memory leaks: cleanup de dependencias

## 📁 Archivos a Crear
- `runtime/ui/src/reactive_widgets.rs` - Implementación principal
- `runtime/ui/src/reactive_context.rs` - ReactiveBuildContext
- `runtime/ui/src/widget_invalidator.rs` - Sistema de invalidación
- `runtime/ui/src/lib.rs` - Re-exports
- `examples/ui/reactive_widgets_example.rs` - Ejemplos de uso
- `docs/features/VELA-058/TASK-058.md` - Esta documentación
- `docs/features/VELA-058/README.md` - Resumen completo

## 🔗 Referencias
- **Jira:** [VELA-058](https://velalang.atlassian.net/browse/VELA-058)
- **Historia:** [US-13](https://velalang.atlassian.net/browse/US-13)
- **Dependencias:** [TASK-029](https://velalang.atlassian.net/browse/TASK-029), [TASK-054](https://velalang.atlassian.net/browse/TASK-054)
- **Arquitectura:** [ADR-058](docs/architecture/ADR-058-signal-integration.md)
- **Sistema Base:** `packages/reactive/src/`