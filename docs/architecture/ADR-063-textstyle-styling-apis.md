# ADR-063: Sistema de Estilos y TextStyle APIs

## Estado
✅ Aceptado

## Fecha
2025-12-09

## Contexto
Necesitamos implementar un sistema de estilos completo para Vela que permita:
- Estilizar texto con propiedades tipográficas avanzadas
- Crear estilos reutilizables para componentes
- Soporte para theming y responsive design
- Integración fluida con el sistema de widgets existente

El sistema debe ser:
- **Declarativo**: Estilos definidos junto con componentes
- **Componible**: Estilos que se pueden combinar y heredar
- **Performante**: Optimizado para reconciliación eficiente
- **Type-safe**: Validación estática de propiedades de estilo

## Decisión
Implementar un sistema de estilos en capas con las siguientes componentes:

### 1. TextStyle Core
```rust
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub font_family: Option<String>,
    pub font_size: Option<FontSize>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub letter_spacing: Option<LetterSpacing>,
    pub word_spacing: Option<WordSpacing>,
    pub line_height: Option<LineHeight>,
    pub text_decoration: Option<TextDecoration>,
    pub text_align: Option<TextAlign>,
    pub text_transform: Option<TextTransform>,
}
```

### 2. Style System Architecture
- **Inline Styles**: Aplicados directamente en widgets
- **Class-based Styles**: Estilos nombrados reutilizables
- **Theme-based Styles**: Estilos contextuales por tema
- **Responsive Styles**: Estilos condicionales por breakpoint

### 3. Style Composition Strategy
- **Merge Strategy**: Propiedades posteriores sobrescriben anteriores
- **Inheritance**: Estilos padre se heredan a hijos
- **Cascade Priority**: inline > class > theme > default

## Consecuencias

### Positivas
- ✅ **Type Safety**: Validación estática de todas las propiedades de estilo
- ✅ **Performance**: Estilos inmutables permiten memoización eficiente
- ✅ **Composability**: Sistema de composición flexible y extensible
- ✅ **Developer Experience**: API intuitiva similar a CSS-in-JS frameworks
- ✅ **Theming Support**: Fácil implementación de temas claros/oscuros
- ✅ **Responsive Design**: Soporte nativo para estilos condicionales

### Negativas
- ❌ **Bundle Size**: Mayor tamaño del runtime por tipos de estilo completos
- ❌ **Learning Curve**: Nuevos conceptos de composición de estilos
- ❌ **Complexity**: Sistema más complejo que estilos inline simples

## Alternativas Consideradas

### 1. CSS-in-JS Approach (Rechazado)
**Pros**: Familiar para desarrolladores web
**Cons**: No type-safe, runtime overhead alto, no optimizable en compile-time
**Razón de rechazo**: No aprovecha las ventajas de compilación de Vela

### 2. Flutter-style Inline Only (Rechazado)
**Pros**: Simple, directo
**Cons**: No reutilizable, difícil mantenimiento, no theming
**Razón de rechazo**: No escala para aplicaciones complejas

### 3. Tailwind-style Utility Classes (Rechazado)
**Pros**: Reutilizable, pequeño bundle
**Cons**: No expresivo, difícil customización, limitado
**Razón de rechazo**: Menos flexible que un sistema de composición completo

## Implementación

### API Design
```rust
// Inline styles
Text("Hello", style: TextStyle {
    font_size: Some(FontSize::Px(16.0)),
    color: Some(Color::hex("#333")),
    font_weight: Some(FontWeight::Bold),
    ..
})

// Named styles
let title_style = TextStyle::new()
    .font_size(FontSize::Px(24.0))
    .font_weight(FontWeight::Bold)
    .color(Color::hex("#1a1a1a"));

Text("Title", style: title_style)

// Style composition
let base_style = TextStyle::new().font_family("Inter");
let bold_style = base_style.clone().font_weight(FontWeight::Bold);
let title_style = bold_style.color(Color::hex("#0066cc"));
```

### Style Resolution Algorithm
1. **Collect Styles**: Reunir todos los estilos aplicables (theme, class, inline)
2. **Merge Properties**: Aplicar estrategia de merge por prioridad
3. **Resolve Inheritance**: Propagar propiedades heredables a hijos
4. **Optimize**: Memoizar estilos resueltos para evitar recálculos

### Performance Optimizations
- **Style Memoization**: Cache de estilos resueltos por hash
- **Lazy Resolution**: Resolver estilos solo cuando cambian
- **Minimal Diffing**: Comparar solo propiedades que cambiaron
- **Tree Shaking**: Eliminar estilos no utilizados en compile-time

## Referencias
- Jira: [VELA-063](https://velalang.atlassian.net/browse/VELA-063)
- Historia: [VELA-063](https://velalang.atlassian.net/browse/VELA-063)
- Inspiración: Flutter TextStyle, SwiftUI modifiers, CSS-in-JS patterns
- Dependencias: TASK-057 (widgets de display)

## Implementación
Ver código en: `runtime/ui/src/style/` y `runtime/ui/src/text_style.rs`