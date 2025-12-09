# ADR-056: Input Widgets Architecture

## Estado
✅ Aceptado

## Fecha
2025-01-30

## Contexto
Necesitamos implementar widgets de input básicos para Vela UI que permitan la interacción del usuario. Los widgets requeridos son Button, TextField y Checkbox, que son fundamentales para cualquier interfaz de usuario.

Esta implementación debe ser consistente con la arquitectura de widgets ya establecida (BaseWidget, lifecycle hooks, constraint-based layout) y proporcionar una API intuitiva para desarrolladores.

## Decisión
Implementaremos tres widgets de input principales:

### Button Widget
- **Propósito**: Widget clickable que ejecuta acciones
- **Estados**: Normal, Hover, Pressed, Disabled
- **Eventos**: onClick, onPress, onRelease
- **Styling**: Background color, text color, border, padding
- **Layout**: Sizing automático basado en contenido + padding

### TextField Widget
- **Propósito**: Campo de entrada de texto single-line
- **Estados**: Normal, Focused, Disabled, Error
- **Funcionalidad**: Placeholder, validation, max length
- **Eventos**: onChange, onFocus, onBlur, onSubmit
- **Styling**: Border, background, text styling
- **Layout**: Width flexible, height fijo

### Checkbox Widget
- **Propósito**: Toggle boolean con estado visual
- **Estados**: Checked, Unchecked, Indeterminate, Disabled
- **Eventos**: onChange
- **Styling**: Checkmark icon, colors, sizing
- **Layout**: Size fijo con label opcional

## Consecuencias

### Positivas
- **API consistente**: Todos los widgets siguen el mismo patrón de BaseWidget
- **Event handling integrado**: Sistema de eventos consistente con el framework
- **Styling flexible**: CSS generation automática con propiedades customizables
- **Accessibility**: Soporte para ARIA attributes y keyboard navigation
- **Type safety**: Strongly typed properties y eventos

### Negativas
- **Complejidad inicial**: Tres widgets requieren implementación cuidadosa
- **Event system overhead**: Sistema de eventos agrega complejidad
- **Browser compatibility**: Diferentes browsers manejan inputs diferente

## Alternativas Consideradas

### 1. Web Components Nativos
**Descripción**: Usar `<input>`, `<button>` nativos del browser
**Rechazada porque**: No permite control total del styling y behavior, inconsistente con arquitectura Vela

### 2. Single Input Widget
**Descripción**: Un widget genérico que se configura para diferentes tipos
**Rechazada porque**: Menos intuitivo para developers, más complejo de mantener

### 3. Minimal API
**Descripción**: Solo propiedades básicas sin advanced features
**Rechazada porque**: Limita la utilidad, requiere reimplementación posterior

## Implementación

### Button Implementation
```rust
pub struct Button {
    base: BaseWidget,
    pub text: String,
    pub on_click: Option<Box<dyn Fn()>>,
    pub disabled: bool,
    pub variant: ButtonVariant,
}

pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
}
```

### TextField Implementation
```rust
pub struct TextField {
    base: BaseWidget,
    pub value: String,
    pub placeholder: String,
    pub disabled: bool,
    pub max_length: Option<usize>,
    pub on_change: Option<Box<dyn Fn(String)>>,
}
```

### Checkbox Implementation
```rust
pub struct Checkbox {
    base: BaseWidget,
    pub checked: bool,
    pub label: Option<String>,
    pub disabled: bool,
    pub on_change: Option<Box<dyn Fn(bool)>>,
}
```

## Referencias
- Jira: [VELA-056](https://velalang.atlassian.net/browse/VELA-056)
- Documentación: Flutter Button, TextField, Checkbox APIs
- Arquitectura: ADR-053 (Widget Architecture), ADR-054 (Base Widget), ADR-055 (Layout Widgets)</content>
<parameter name="filePath">C:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-056-input-widgets.md