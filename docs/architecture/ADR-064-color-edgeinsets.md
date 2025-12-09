# ADR-064: Implementar Color y EdgeInsets

## Estado
✅ Aceptado

## Fecha
2025-12-09

## Contexto
Como parte del sistema de estilos y theming de Vela UI, necesitamos asegurar que los tipos fundamentales `Color` y `EdgeInsets` estén completamente integrados en el sistema de estilos. Estos tipos son esenciales para el sistema de estilos implementado en TASK-063 y serán utilizados por widgets y componentes UI.

## Decisión
Integrar completamente `Color` y `EdgeInsets` en el sistema de estilos:

### Color
- **Ya implementado**: En `style/types.rs` como parte de TASK-063
- **Funcionalidad**: Soporte completo para colores (hex, rgb, rgba, hsl, hsla, named)
- **Uso**: Para colores de texto, fondo, bordes, etc.

### EdgeInsets
- **Ya implementado**: En `layout.rs` como parte de TASK-055 (widgets de layout)
- **Mejora agregada**: Método `to_css()` para conversión a CSS
- **Propósito**: Representar espaciado (padding/margin) con valores simétricos o asimétricos
- **API**: Constructores `all()`, `symmetric()`, `horizontal()`, `vertical()`, `new()`

## Consecuencias

### Positivas
- ✅ Tipos fundamentales ya disponibles en el sistema
- ✅ `EdgeInsets.to_css()` agregado para compatibilidad con estilos
- ✅ Reutilización en widgets de layout (Container, etc.)
- ✅ Consistencia con Flutter/React Native APIs
- ✅ Soporte completo para theming

### Negativas
- ⚠️ Dependencia del sistema de estilos (TASK-063)
- ⚠️ `EdgeInsets` estaba en `layout.rs`, no en `style/types.rs`

## Alternativas Consideradas
1. **Usar tuplas simples**: `(f32, f32, f32, f32)` - Rechazada porque menos expresiva
2. **Structs separados**: `Padding` y `Margin` - Rechazada porque funcionalidad idéntica
3. **Enums para tipos**: `EdgeInsets::All(f32)` - Rechazada porque menos eficiente

## Referencias
- Jira: [VELA-064](https://velalang.atlassian.net/browse/VELA-064)
- Dependencia: TASK-063 (TextStyle y styling APIs)
- Inspiración: Flutter EdgeInsets, React Native spacing

## Implementación
Ver código en: `runtime/ui/src/layout.rs` (EdgeInsets.to_css)