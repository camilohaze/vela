# ADR-163: Desktop Renderer Architecture with Skia

## Estado
✅ Aceptado

## Fecha
2025-01-30

## Contexto
Necesitamos implementar un renderer para aplicaciones desktop multiplataforma que proporcione:
- Hardware-accelerated 2D graphics
- Integración con VelaVDOM
- Alto performance comparable a aplicaciones nativas
- Soporte multiplataforma (Windows, macOS, Linux)

## Decisión
Implementar DesktopRenderer usando Skia graphics library con la siguiente arquitectura:

### 1. Skia Integration
- Usar `skia-safe` crate (Rust bindings para Skia)
- Surface raster N32 premultiplied para consistencia cross-platform
- FontMgr para gestión de fuentes del sistema

### 2. VDOM Architecture
- JSON serialization/deserialization para widget trees
- WidgetNode hierarchy: Container, Text, Button, Image, Custom
- Layout system con coordenadas absolutas
- Style system con colores RGBA y propiedades tipográficas

### 3. Rendering Pipeline
- Frame-based rendering con begin_frame/end_frame
- Recursive widget traversal con canvas operations
- Framebuffer access para display/output
- Mutable renderer state para surface management

## Consecuencias

### Positivas
- **Performance**: Hardware acceleration nativo
- **Calidad**: Anti-aliasing, subpixel rendering
- **Consistencia**: Mismo renderer en todas las plataformas
- **Mantenibilidad**: API limpia y bien documentada
- **Extensibilidad**: Fácil agregar nuevos tipos de widgets

### Negativas
- **Dependencia**: Locked a Skia ecosystem
- **Complejidad**: Gestión de surfaces y canvases mutable
- **Bundle size**: Skia agrega ~10-15MB al bundle
- **Build complexity**: Native compilation requerida

## Alternativas Consideradas

### 1. Web-based Renderer (Electron-style)
**Rechazada porque:**
- Bundle size mucho mayor (100MB+ vs 15MB)
- Performance inferior para gráficos 2D intensivos
- Dependencia de Chromium/WebKit

### 2. Qt/QML Renderer
**Rechazada porque:**
- Licencia GPL restrictiva para algunos casos
- Mayor complejidad de build
- Menos control sobre rendering pipeline

### 3. Custom Software Renderer
**Rechazada porque:**
- Desarrollo time mucho mayor
- Performance inferior en GPUs modernas
- Mantenimiento de algoritmos gráficos complejos

## Implementación
Ver código en: `runtime/desktop/src/renderer.rs`

### API Principal
```rust
// Creación y configuración
let mut renderer = DesktopRenderer::new(800, 600)?;

// Render loop
renderer.begin_frame()?;
renderer.render_widget(&widget_tree)?;
renderer.end_frame()?;

// Obtener framebuffer para display
if let Some(pixels) = renderer.get_framebuffer() {
    display.show_pixels(&pixels);
}
```

### VDOM Format
```json
{
  "type": "Container",
  "layout": {"x": 0, "y": 0, "width": 800, "height": 600},
  "style": {"background_color": {"r": 255, "g": 255, "b": 255, "a": 255}},
  "children": [...]
}
```

## Referencias
- Jira: [VELA-1173](https://velalang.atlassian.net/browse/VELA-1173)
- Documentación: `docs/features/VELA-1173/TASK-163.md`
- Código: `runtime/desktop/src/renderer.rs`