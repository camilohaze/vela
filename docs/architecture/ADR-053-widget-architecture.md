# ADR-053: Arquitectura de Widgets para UI Framework

## Estado
✅ Aceptado

## Fecha
2025-12-09

## Contexto
Vela necesita un sistema de UI declarativo y reactivo para crear interfaces de usuario. El framework debe ser funcional puro, con composición de widgets, Virtual DOM, y integración completa con el sistema reactivo existente (signals, computed, effect).

El problema principal es diseñar una arquitectura que:
- Permita composición declarativa de UI
- Sea eficiente en rendimiento (Virtual DOM + diffing)
- Se integre perfectamente con el sistema reactivo
- Mantenga la pureza funcional de Vela
- Sea extensible y modular

## Decisión
Implementaremos una arquitectura de widgets inspirada en Flutter/React, pero adaptada al paradigma funcional puro de Vela:

### 1. Widget Base Class
```rust
pub trait Widget {
    fn build(&self, context: &BuildContext) -> VDomNode;
    fn key(&self) -> Option<Key>;
}
```

### 2. Composición Jerárquica
- Widgets se componen de otros widgets
- Props inmutables pasan de padre a hijo
- Estado local manejado con signals reactivos

### 3. Virtual DOM (VDOM)
- Representación intermedia del árbol de widgets
- Estructura: `VDomNode { widget: Box<dyn Widget>, children: Vec<VDomNode> }`

### 4. Diffing Algorithm
- Algoritmo de reconciliación basado en keys
- Comparación eficiente de árboles VDOM
- Generación de patches mínimos

### 5. Lifecycle Hooks
- `mount()`: Al insertar en DOM
- `update()`: Al actualizar propiedades
- `destroy()`: Al remover del DOM

### 6. Integración Reactiva
- Widgets se reconstruyen automáticamente cuando signals cambian
- `effect()` hooks para side effects
- Lazy evaluation de sub-árboles

## Consecuencias

### Positivas
- **Rendimiento**: Virtual DOM + diffing optimiza re-renders
- **Reactividad**: Integración perfecta con signals existentes
- **Composición**: Fácil composición declarativa de UI compleja
- **Pureza Funcional**: Widgets son funciones puras (build methods)
- **Extensibilidad**: Sistema de widgets fácilmente extensible
- **Debugging**: Virtual DOM facilita debugging y hot reload

### Negativas
- **Complejidad Inicial**: Implementación del diffing algorithm es compleja
- **Memory Overhead**: Virtual DOM requiere memoria adicional
- **Learning Curve**: Paradigma declarativo diferente a imperativo
- **Runtime Cost**: Reconstrucción de widgets en cada render

## Alternativas Consideradas
1. **DOM Directo (como Vue.js)**: Manipulación directa del DOM real
   - Rechazada porque: No permite optimizaciones de diffing, más difícil debugging, no tan funcional puro

2. **Sin Virtual DOM (como Svelte)**: Compilación a operaciones DOM directas
   - Rechazada porque: Menos optimizable, más complejo el compilador, pierde beneficios de VDOM para debugging

3. **Actor-based UI (como Elm)**: Cada widget como actor independiente
   - Rechazada porque: Overhead de actores para UI simple, complejidad innecesaria, peor rendimiento

4. **Template-based (como Angular)**: Templates separados de lógica
   - Rechazada porque: No funcional puro, mezcla de paradigmas, más boilerplate

## Referencias
- Jira: [VELA-053](https://velalang.atlassian.net/browse/VELA-053)
- Documentación: docs/features/VELA-053/
- Inspiración: Flutter Widget System, React Fiber, Vue Composition API

## Implementación
Ver código en: `runtime/ui/`
- `runtime/ui/src/widget.rs` - Widget trait y base classes
- `runtime/ui/src/vdom.rs` - Virtual DOM implementation
- `runtime/ui/src/diff.rs` - Diffing algorithm
- `runtime/ui/src/patch.rs` - DOM patching system