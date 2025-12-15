# ADR-150: GridView Virtualizado

## Estado
🔄 Propuesto

## Fecha
2025-01-30

## Contexto
Después de implementar el ListView virtualizado (TASK-149), necesitamos extender el sistema de virtualización para manejar layouts de grid bidimensionales. Los grids requieren gestión de viewport en dos dimensiones (filas y columnas) y layout automático de elementos.

## Decisión
Implementar VirtualizedGridView como extensión del sistema de virtualización existente, reutilizando componentes como WidgetPool pero agregando GridViewportManager para manejo 2D.

## Consecuencias

### Positivas
- ✅ Reutilización de código existente (WidgetPool, base de virtualización)
- ✅ Arquitectura consistente con ListView
- ✅ Soporte completo para grids grandes (10,000+ elementos)
- ✅ Optimización automática de memoria y rendimiento

### Negativas
- ❌ Complejidad adicional en gestión de viewport 2D
- ❌ Layout calculations más complejos
- ❌ Mayor superficie de API (parámetros de columnas, etc.)

## Alternativas Consideradas
1. **Implementación independiente**: Crear sistema completamente separado
   - Rechazada porque: Duplicaría código y no aprovecharía la base existente
2. **Extensión del ListView**: Modificar ListView para soporte grid
   - Rechazada porque: Haría el código más complejo y menos mantenible
3. **Sistema de layout separado**: Grid como layout engine sobre ListView
   - Rechazada porque: Menos eficiente que virtualización nativa 2D

## Implementación
Ver código en: `runtime/src/ui/virtualization.rs`

## Referencias
- Jira: [VELA-1157]
- Documentación: docs/features/VELA-1157/TASK-150.md