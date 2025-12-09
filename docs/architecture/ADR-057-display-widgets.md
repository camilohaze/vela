# ADR-057: Arquitectura de Display Widgets

## Estado
✅ Aceptado

## Fecha
2025-12-09

## Contexto
Necesitamos implementar widgets de display básicos (Text, Image, Icon) para completar la capa de presentación del UI Framework de Vela. Estos widgets deben ser simples, eficientes y consistentes con la arquitectura existente basada en Virtual DOM.

## Decisión
Implementar tres widgets de display siguiendo el patrón establecido:

### Text Widget
- **Tag HTML**: `<span>` para inline text, `<p>` para block text
- **Propiedades**: text content, font size, color, weight
- **Estilos**: CSS inline o clases generadas dinámicamente
- **Justificación**: Simplicidad y performance sobre complejidad

### Image Widget
- **Tag HTML**: `<img>` nativo
- **Propiedades**: src URL, alt text, width, height
- **Loading**: Lazy loading opcional
- **Fallback**: Placeholder cuando falla la carga

### Icon Widget
- **Implementación**: Unicode characters inicialmente
- **Extensibilidad**: Soporte futuro para SVG y font icons
- **Propiedades**: icon code, size, color
- **Performance**: Mínima sobrecarga

## Consecuencias

### Positivas
- ✅ **Simplicidad**: Widgets básicos y predecibles
- ✅ **Performance**: Renderizado directo a HTML nativo
- ✅ **Consistencia**: Patrón idéntico a input widgets
- ✅ **Extensibilidad**: Fácil agregar más propiedades después
- ✅ **Accesibilidad**: Soporte básico para alt text y semantic HTML

### Negativas
- ⚠️ **Limitaciones iniciales**: Solo Unicode para iconos (no SVG)
- ⚠️ **Sin sistema de temas**: Estilos hardcoded inicialmente
- ⚠️ **Sin responsive images**: Imágenes fijas inicialmente

## Alternativas Consideradas

### 1. Sistema de Iconos Complejo
**Descripción**: Implementar sistema completo con SVG sprites, font icons, etc.
**Rechazada porque**: Aumentaría complejidad innecesariamente. Podemos agregar después.

### 2. Text Widget con Rich Text
**Descripción**: Soporte completo para markdown, HTML, etc.
**Rechazada porque**: Widgets de display deben ser simples. Rich text puede ser widget separado.

### 3. Image Widget con Transformaciones
**Descripción**: Soporte para crop, resize, filters desde el widget.
**Rechazada porque**: Debe ser manejado por CSS o librerías especializadas.

## Implementación
Ver código en: `runtime/ui/src/display_widgets.rs`

## Referencias
- Jira: VELA-057
- Historia: US-12 (Widgets Declarativos)
- Arquitectura: Basada en BaseWidget y Virtual DOM
- Patrón: Seguido de input_widgets.rs