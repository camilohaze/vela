# ADR-173: Function Inlining Optimization

## Estado
✅ Aceptado

## Fecha
2025-12-15

## Contexto
El compilador Vela genera bytecode que ejecuta llamadas a funciones, lo cual introduce overhead de pila y control flow. Para funciones pequeñas (menos de 5 instrucciones), el overhead de la llamada puede ser mayor que el costo de ejecutar la función inlineada.

## Decisión
Implementar function inlining conservador en el IROptimizer que:
- Identifica funciones candidatas pequeñas (≤5 instrucciones)
- Reemplaza llamadas a estas funciones con su cuerpo inlineado
- Maneja parámetros mediante sustitución de variables
- Preserva la corrección del programa evitando inlining recursivo

## Consecuencias

### Positivas
- Reducción significativa de overhead de llamadas para funciones pequeñas
- Mejor rendimiento en tiempo de ejecución
- Optimización automática sin intervención del programador

### Negativas
- Aumento del tamaño del bytecode generado
- Potencial duplicación de código
- Mayor complejidad en el análisis de dependencias

## Alternativas Consideradas
1. **Inlining agresivo**: Inlining de todas las funciones - Rechazado por aumento excesivo de tamaño de código
2. **No inlining**: Mantener todas las llamadas - Rechazado por rendimiento subóptimo
3. **Inlining basado en heurísticas complejas**: Análisis de frecuencia de llamadas - Rechazado por complejidad innecesaria en esta fase

## Referencias
- Jira: TASK-173
- Historia: VELA-1184

## Implementación
Ver código en: `compiler/src/codegen/ir_to_bytecode.rs::IROptimizer::function_inlining`</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-173-function-inlining.md