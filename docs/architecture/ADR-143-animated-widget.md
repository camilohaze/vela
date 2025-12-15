# ADR-143: Implementar Animated Widget

## Estado
✅ Aceptado

## Fecha
2025-12-14

## Contexto
Necesitamos implementar un sistema de animaciones fluidas para la UI de Vela. Las animaciones son esenciales para una buena experiencia de usuario en aplicaciones modernas, permitiendo transiciones suaves entre estados y mejorando la interactividad.

## Decisión
Implementaremos el `Animated` widget como un widget contenedor que anima propiedades de sus hijos usando interpolación. El sistema se basará en:

1. **AnimationController**: Controla el progreso de la animación (0.0 a 1.0)
2. **Curves**: Funciones de easing para diferentes tipos de animación
3. **Tween**: Define el rango de valores a interpolar
4. **Animated Widget**: Aplica la animación a propiedades específicas

## Consecuencias

### Positivas
- Animaciones fluidas y personalizables
- Mejor UX con transiciones suaves
- Sistema extensible para futuras animaciones
- Integración nativa con el sistema reactivo

### Negativas
- Complejidad adicional en el rendering
- Overhead de performance para animaciones complejas
- Curva de aprendizaje para desarrolladores

## Alternativas Consideradas
1. **CSS-like animations**: Rechazada porque Vela no usa CSS
2. **Imperative animations**: Rechazada porque va contra el paradigma declarativo
3. **Third-party library**: Rechazada porque queremos control total

## Referencias
- Jira: VELA-1149
- Flutter AnimatedWidget
- React Spring

## Implementación
Ver código en: `runtime/src/ui/animated.rs`