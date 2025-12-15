# ADR-144: AnimationController Avanzado

## Estado
✅ Aceptado

## Fecha
2025-12-14

## Contexto
El AnimationController básico implementado en TASK-143 necesita ser extendido con funcionalidades avanzadas para controlar animaciones complejas, incluyendo secuencias, repeticiones, callbacks y estados avanzados de animación.

## Decisión
Extender el AnimationController con:

1. **Animation Sequences**: Ejecutar animaciones en secuencia
2. **Parallel Animations**: Ejecutar múltiples animaciones simultáneamente
3. **Repeat & Reverse**: Funcionalidades de repetición y reversa
4. **Callbacks**: Eventos para start, complete, update
5. **Animation Status**: Estados detallados (idle, running, paused, completed, cancelled)
6. **Animation Builder**: API fluida para construir animaciones complejas

## Consecuencias

### Positivas
- Control completo sobre animaciones complejas
- API intuitiva para desarrolladores
- Soporte para animaciones anidadas y compuestas
- Mejor control de estado y eventos

### Negativas
- Complejidad adicional en el código
- Mayor uso de memoria para animaciones complejas
- Curva de aprendizaje más pronunciada

## Alternativas Consideradas
1. **Animation DSL**: Lenguaje específico para animaciones - Rechazado por complejidad
2. **State Machine**: Máquina de estados pura - Rechazado por verbosidad
3. **Callback-based**: Solo callbacks - Rechazado por falta de control

## Referencias
- Jira: VELA-1149
- Flutter AnimationController
- React Spring useSpring

## Implementación
Ver código en: `runtime/src/ui/animated.rs` (extensión)