# ADR-145: Sistema Completo de Curves y Easing

## Estado
✅ Aceptado

## Fecha
2025-12-14

## Contexto
El sistema de curvas actual tiene curvas básicas, pero necesita un conjunto completo de funciones de easing para animaciones profesionales, incluyendo curvas cúbicas, senoidales, exponenciales, y curvas personalizadas con Cubic Bezier.

## Decisión
Implementar un sistema completo de easing curves:

1. **Curvas Polinomiales**: Cubic, Quartic, Quintic
2. **Curvas Trigonométricas**: Sine, Circular
3. **Curvas Exponenciales**: Exponential
4. **Curvas Especiales**: Back (overshoot), Bounce avanzado
5. **Cubic Bezier**: Curvas personalizadas con control points
6. **Easing Functions**: EaseIn, EaseOut, EaseInOut para cada tipo
7. **Interpolation Helpers**: Funciones de interpolación para diferentes tipos de datos

## Consecuencias

### Positivas
- Conjunto completo de curvas de easing estándar
- Curvas personalizables con Cubic Bezier
- Mejor control artístico sobre animaciones
- Compatibilidad con estándares de animación web

### Negativas
- Mayor complejidad en el código
- Más cálculos matemáticos por frame
- Mayor tamaño del binario

## Alternativas Consideradas
1. **Biblioteca externa**: Usar una crate de Rust - Rechazada por dependencias
2. **Solo curvas básicas**: Mantener simple - Rechazada por limitaciones
3. **Runtime evaluation**: Evaluar expresiones - Rechazada por performance

## Referencias
- CSS Easing Functions
- Flutter Curves
- Robert Penner's Easing Equations

## Implementación
Ver código en: `runtime/src/ui/curves.rs`