# ADR-172: Implementación de Dead Code Elimination

## Estado
✅ Aceptado

## Fecha
2025-12-15

## Contexto
Como parte de las optimizaciones de rendimiento del compilador Vela (VELA-1184), necesitamos implementar dead code elimination (DCE) para reducir el tamaño del bytecode generado y mejorar el rendimiento al eliminar instrucciones que nunca se ejecutan.

## Decisión
Implementar dead code elimination en el módulo `IROptimizer` con enfoque conservador, eliminando únicamente código provadamente inalcanzable o no utilizado, preservando la corrección del programa.

## Consecuencias

### Positivas
- **Reducción de bytecode**: 10-15% menos instrucciones generadas
- **Mejora de rendimiento**: Menos instrucciones ejecutadas en runtime
- **Mejor experiencia de desarrollo**: Código más eficiente
- **Fundamento para optimizaciones**: Base para análisis de flujo más avanzados

### Negativas
- **Complejidad añadida**: Análisis de flujo de control más complejo
- **Tiempo de compilación**: Mayor tiempo de compilación debido al análisis
- **Riesgo de errores**: Posible eliminación incorrecta de código necesario

## Alternativas Consideradas

### 1. DCE Agresivo (Rechazada)
**Descripción**: Eliminar todo código potencialmente dead, incluyendo funciones no llamadas.
**Razones de rechazo**:
- Riesgo de romper programas: eliminación de entry points o funciones llamadas dinámicamente
- Dificultad de análisis: determinar con certeza absoluta si código es dead
- Problemas de interoperabilidad: eliminación de funciones exportadas

### 2. DCE en AST (Rechazada)
**Descripción**: Implementar DCE directamente en el AST antes de conversión a IR.
**Razones de rechazo**:
- Menor precisión: AST no tiene información completa de flujo de control
- Dificultad de mantenimiento: cambios en AST afectan múltiples fases
- Cobertura limitada: no puede eliminar código optimizado en IR

### 3. DCE Separado (Rechazada)
**Descripción**: Crear módulo `DeadCodeEliminator` independiente.
**Razones de rechazo**:
- Mayor complejidad: múltiples fases de optimización
- Duplicación de código: análisis similar en diferentes módulos
- Mayor overhead: procesamiento adicional innecesario

## Implementación
La implementación se realizará extendiendo `IROptimizer` con métodos:

- `analyze_control_flow()`: Construcción del grafo de flujo de control
- `mark_live_code()`: Marcado de instrucciones alcanzables
- `eliminate_dead_code()`: Eliminación de código no utilizado
- `preserve_side_effects()`: Preservación de código con efectos secundarios

## Referencias
- **Jira**: [VELA-1184](https://velalang.atlassian.net/browse/VELA-1184)
- **Task**: [TASK-172](https://velalang.atlassian.net/browse/TASK-172)
- **Dependencia**: TASK-171 (Constant Folding)
- **Código**: `compiler/src/codegen/ir_to_bytecode.rs`