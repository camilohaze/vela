# ADR-065: Implementar Theme system con Context

## Estado
✅ Aceptado

## Fecha
2025-12-09

## Contexto
Ya existe un sistema de themes global en TASK-063, pero necesitamos un sistema de theming contextual que permita que diferentes partes de la UI tengan themes diferentes. Esto es esencial para aplicaciones complejas donde diferentes secciones pueden tener apariencia diferente (ej: un sidebar con theme oscuro mientras el contenido principal tiene theme claro).

## Decisión
Implementar un sistema de theming contextual basado en ThemeProvider/ThemeContext:

### ThemeProvider
- **Componente widget**: `ThemeProvider` que envuelve widgets hijos
- **Propósito**: Proporcionar theme a subtree de widgets
- **API**: `ThemeProvider(theme: Theme, child: Widget)`

### ThemeContext
- **Sistema de contexto**: Para acceder al theme actual desde cualquier widget
- **Método**: `useTheme()` hook para widgets funcionales
- **Fallback**: Si no hay ThemeProvider, usar theme global

### ThemeConsumer
- **Widget helper**: Para widgets que necesitan acceder al theme
- **Uso**: `ThemeConsumer(builder: (theme) => Widget)`

## Consecuencias

### Positivas
- ✅ Theming contextual por componente
- ✅ Composición de themes (local sobre global)
- ✅ Reutilización de themes existentes
- ✅ Compatibilidad con sistema global existente

### Negativas
- ⚠️ Complejidad adicional en árbol de widgets
- ⚠️ Performance overhead en resolución de themes
- ⚠️ Dependencia del sistema de context (si no existe)

## Alternativas Consideradas
1. **Solo global**: Rechazada - No permite theming por sección
2. **Props drilling**: Rechazada - Antipattern, difícil de mantener
3. **CSS variables**: Rechazada - No es idiomático para framework UI

## Referencias
- Jira: [VELA-065](https://velalang.atlassian.net/browse/VELA-065)
- Dependencia: TASK-064 (Color y EdgeInsets)
- Inspiración: React Context, Flutter Theme

## Implementación
Ver código en: `runtime/ui/src/style/theme_context.rs`