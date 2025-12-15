# ADR-138: Debug Info Generation Architecture

## Estado
✅ Aceptado

## Fecha
2025-12-14

## Contexto
Necesitamos implementar información de debug para permitir debugging efectivo en Vela. Los desarrolladores requieren poder inspeccionar la ejecución del código, establecer breakpoints, y examinar el estado de variables durante la ejecución.

## Decisión
Implementaremos un sistema de debug info generation que incluye:

1. **Debug Symbols Table**: Tabla de símbolos con mapeo entre código fuente y bytecode
2. **Source Maps**: Mapeo línea a línea entre código fuente y bytecode positions
3. **Variable Info**: Información sobre variables locales, parámetros y su ubicación en el stack
4. **Function Info**: Información sobre funciones con sus rangos de bytecode

La información de debug se almacenará como metadata en el bytecode program, separada del código ejecutable para mantener la eficiencia.

## Consecuencias

### Positivas
- Permite debugging completo con breakpoints y variable inspection
- Mantiene separación clara entre código ejecutable y metadata de debug
- Compatible con DAP (Debug Adapter Protocol) para integración con editores
- Soporta source maps para navegación precisa en el código fuente

### Negativas
- Aumenta el tamaño del bytecode program (metadata adicional)
- Requiere procesamiento adicional durante compilación
- Complejidad añadida en el generador de bytecode

## Alternativas Consideradas
1. **Debug Info Inline**: Mezclar debug info con bytecode - Rechazada porque aumenta complejidad de ejecución
2. **External Debug Files**: Archivos separados - Rechazada porque complica distribución y deployment
3. **No Debug Info**: Sin información de debug - Rechazada porque imposibilita debugging efectivo

## Referencias
- Jira: VELA-1143
- Documentación: Debug Adapter Protocol specification
- Código: compiler/src/bytecode.rs, compiler/src/codegen/ir_to_bytecode.rs

## Implementación
Ver código en: compiler/src/debug_info.rs, compiler/src/codegen/debug_info_generator.rs