# TASK-057: Implementar widgets de display (Text, Image, Icon)

## 📋 Información General
- **Historia:** US-12 (Widgets Declarativos)
- **Estado:** En desarrollo ✅
- **Fecha:** 2025-12-09
- **Estimación:** 48 horas
- **Dependencias:** TASK-056 (Input Widgets)

## 🎯 Objetivo
Implementar widgets básicos de presentación (Text, Image, Icon) que completen la capa de display del UI Framework de Vela, siguiendo el patrón establecido por los input widgets.

## 🔨 Implementación Técnica

### Arquitectura General
- **Patrón**: Igual que input widgets (struct + builder methods + Widget trait)
- **VDOM**: Renderizado directo a HTML nativo
- **CSS**: Generación dinámica de estilos
- **Testing**: Cobertura completa con unit tests

### 1. Text Widget
**Propósito**: Mostrar texto con opciones básicas de formato

**API:**
```rust
pub struct Text {
    pub content: String,
    pub font_size: Option<f32>,    // px
    pub color: Option<String>,     // hex color
    pub font_weight: Option<String>, // normal, bold, etc.
    pub text_align: Option<String>,  // left, center, right
    pub display: TextDisplay,      // Inline, Block
}

pub enum TextDisplay {
    Inline,  // <span>
    Block,   // <p>
}
```

**Métodos Builder:**
- `Text::new(content: impl Into<String>) -> Self`
- `font_size(mut self, size: f32) -> Self`
- `color(mut self, color: impl Into<String>) -> Self`
- `bold(mut self) -> Self`
- `align_center(mut self) -> Self`
- `block(mut self) -> Self`

**Renderizado:**
- `Inline`: `<span class="text" style="...">content</span>`
- `Block`: `<p class="text" style="...">content</p>`

### 2. Image Widget
**Propósito**: Mostrar imágenes con opciones básicas

**API:**
```rust
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<f32>,        // px
    pub height: Option<f32>,       // px
    pub fit: ImageFit,            // Contain, Cover, Fill, None, ScaleDown
}

pub enum ImageFit {
    Contain,   // object-fit: contain
    Cover,     // object-fit: cover
    Fill,      // object-fit: fill
    None,      // object-fit: none
    ScaleDown, // object-fit: scale-down
}
```

**Métodos Builder:**
- `Image::new(src: impl Into<String>) -> Self`
- `alt(mut self, alt: impl Into<String>) -> Self`
- `size(mut self, width: f32, height: f32) -> Self`
- `width(mut self, width: f32) -> Self`
- `height(mut self, height: f32) -> Self`
- `fit(mut self, fit: ImageFit) -> Self`

**Renderizado:**
```html
<img src="..." alt="..." class="image" style="width: Xpx; height: Ypx; object-fit: ...">
```

### 3. Icon Widget
**Propósito**: Mostrar iconos Unicode con opciones de estilo

**API:**
```rust
pub struct Icon {
    pub code: char,                // Unicode character
    pub size: Option<f32>,         // px
    pub color: Option<String>,     // hex color
    pub weight: Option<String>,    // font-weight
}
```

**Métodos Builder:**
- `Icon::new(code: char) -> Self`
- `size(mut self, size: f32) -> Self`
- `color(mut self, color: impl Into<String>) -> Self`
- `bold(mut self) -> Self`

**Renderizado:**
```html
<span class="icon" style="font-size: Xpx; color: #...; font-weight: ...">🔥</span>
```

## ✅ Criterios de Aceptación
- [x] Text widget con opciones básicas de formato
- [x] Image widget con soporte para dimensiones y fit
- [x] Icon widget con caracteres Unicode
- [x] Tests unitarios para todos los widgets (mínimo 80% cobertura)
- [x] Documentación completa de API
- [x] Ejemplos de uso funcionales
- [x] Integración con sistema VDOM existente
- [x] Generación correcta de CSS

## 🧪 Plan de Testing

### Unit Tests (Mínimo 15 tests)
1. **Text Widget Tests**:
   - Creación básica
   - Builder methods
   - Renderizado inline
   - Renderizado block
   - CSS generation

2. **Image Widget Tests**:
   - Creación con src
   - Propiedades opcionales
   - Diferentes fit modes
   - VDOM generation

3. **Icon Widget Tests**:
   - Creación con Unicode
   - Styling options
   - Renderizado correcto

### Integration Tests
- Renderizado completo en VDOM tree
- CSS generation válido
- Compatibilidad con layout system

## 📁 Archivos a Crear
- `runtime/ui/src/display_widgets.rs` - Implementación principal
- `runtime/ui/src/widget.rs` - Re-exports
- `runtime/ui/src/lib.rs` - Exports públicos
- `examples/ui/display_widgets_example.rs` - Ejemplos de uso
- `docs/features/VELA-057/TASK-057.md` - Esta documentación
- `docs/features/VELA-057/README.md` - Resumen completo

## 🔗 Referencias
- **Jira:** [VELA-057](https://velalang.atlassian.net/browse/VELA-057)
- **Historia:** [US-12](https://velalang.atlassian.net/browse/US-12)
- **Dependencias:** [TASK-056](https://velalang.atlassian.net/browse/TASK-056)
- **Arquitectura:** [ADR-057](docs/architecture/ADR-057-display-widgets.md)
- **Patrón:** `runtime/ui/src/input_widgets.rs`