# ADR-059: Virtual DOM Architecture

## Estado
✅ Aceptado

## Fecha
2025-12-03

## Contexto
Necesitamos implementar un Virtual DOM (VDOM) para el framework de UI de Vela. El VDOM es una representación intermedia del árbol de widgets que permite:

- **Reconciliación eficiente**: Comparar cambios entre renders sin tocar el DOM real
- **Actualizaciones selectivas**: Solo actualizar los widgets que realmente cambiaron
- **Optimización de performance**: Minimizar operaciones costosas en el DOM real
- **Desarrollo declarativo**: Los widgets describen el estado deseado, no cómo lograrlo

## Decisión
Implementaremos un Virtual DOM inspirado en React, con las siguientes características:

### Arquitectura del VDOM
```rust
/// Nodo virtual del DOM
pub enum VDomNode {
    /// Elemento con hijos
    Element {
        tag: String,
        attributes: HashMap<String, String>,
        children: Vec<VDomNode>,
        key: Option<String>,
    },
    /// Texto plano
    Text(String),
    /// Fragmento (contenedor sin elemento)
    Fragment(Vec<VDomNode>),
}
```

### Sistema de Keys
- **Propósito**: Identificar widgets únicos para reconciliación eficiente
- **Implementación**: Sistema de keys explícito (similar a React)
- **Beneficio**: Evita reconstrucción innecesaria de widgets

### Reconciliación (Diffing)
- **Algoritmo**: Comparación recursiva de árboles VDOM
- **Estrategia**: 
  - Mismo tipo → actualizar atributos
  - Diferente tipo → reemplazar completamente
  - Keys → reordenamiento inteligente

## Consecuencias

### Positivas
- **Performance**: Actualizaciones O(n) en lugar de reconstruir todo
- **Desarrollo**: Programación declarativa más simple
- **Flexibilidad**: Fácil implementar optimizaciones futuras
- **Debugging**: Mejor tracing de cambios en UI

### Negativas
- **Complejidad**: Implementación más compleja que render directo
- **Memoria**: Doble representación (VDOM + DOM real)
- **Overhead**: Costo inicial de reconciliación

### Alternativas Consideradas

#### 1. Render Directo (Sin VDOM)
- **Pros**: Simple, sin overhead de reconciliación
- **Cons**: Performance pobre en apps complejas
- **Rechazada porque**: No escala para aplicaciones reales

#### 2. VDOM con Keys Automáticos
- **Pros**: Menos boilerplate para desarrolladores
- **Cons**: Algoritmo más complejo, potenciales bugs
- **Rechazada porque**: Keys explícitos dan más control y predictability

#### 3. Incremental DOM (Como Svelte)
- **Pros**: Menos memoria (no árbol completo)
- **Cons**: Más complejo de implementar y optimizar
- **Rechazada porque**: VDOM tradicional es más maduro y probado

## Implementación
Ver código en: `runtime/ui/src/vdom.rs`

### Componentes Principales
1. **VDomNode**: Representación virtual de elementos
2. **VDomTree**: Árbol completo de VDOM
3. **VDomDiff**: Algoritmo de comparación
4. **VDomPatch**: Aplicación de cambios al DOM real

### Integración con Widgets
- Widgets implementan `build()` → `VDomNode`
- Sistema reactivo invalida widgets → re-render selectivo
- VDOM diff genera patches → aplicación eficiente

## Referencias
- Jira: [VELA-059](https://velalang.atlassian.net/browse/VELA-059)
- Documentación: `docs/features/VELA-059/TASK-059.md`
- Inspiración: React Virtual DOM, Vue.js reactivity