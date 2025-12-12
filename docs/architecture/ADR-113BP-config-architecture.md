# ADR-113BP: Arquitectura de Config Management para Microservicios

## Estado
✅ Aceptado

## Fecha
2025-12-11

## Contexto
Como desarrollador de microservicios en Vela, necesito un sistema robusto de gestión de configuración que:

- Cargue configuración desde múltiples fuentes con orden de prioridad
- Proporcione type safety en tiempo de compilación
- Soporte validación automática de valores
- Permita hot reload sin reiniciar servicios
- Sea compatible con sistemas de configuración distribuida (Consul, Vault)

El problema actual es que las configuraciones se manejan manualmente con riesgo de errores de tipos y falta de validación.

## Decisión
Implementar arquitectura de Config Management con los siguientes componentes:

### 1. **ConfigLoader** (Backend)
- Clase central que orquesta la carga desde múltiples fuentes
- Jerarquía de fuentes con prioridad: `file < env vars < Consul < Vault`
- Soporte para perfiles (dev, staging, prod)
- Validación automática con decoradores `@required`, `@min`, `@max`

### 2. **@config Decorator** (Compiler Frontend)
- Decorador compile-time que genera clases type-safe
- Auto-genera validadores y parsers
- Soporte para nested configs y arrays

### 3. **Hot Reload System** (Backend)
- File watchers para cambios en archivos de config
- Integration con Consul/Vault watch APIs
- Event-driven updates sin downtime

### 4. **Validation Framework** (Stdlib)
- Decoradores `@required`, `@email`, `@url`, `@range`
- Custom validators con funciones lambda
- Errores detallados en tiempo de carga

## Consecuencias

### Positivas
- **Type Safety**: Configuración strongly-typed previene errores runtime
- **Developer Experience**: Decoradores declarativos reducen boilerplate
- **Reliability**: Validación automática y hot reload mejoran estabilidad
- **Scalability**: Soporte nativo para microservicios distribuidos
- **Performance**: Compile-time generation evita reflexión runtime

### Negativas
- **Complejidad**: Arquitectura más compleja que configs manuales
- **Dependencias**: Requiere Consul/Vault para features avanzadas
- **Learning Curve**: Nuevos decoradores y conceptos para developers

## Alternativas Consideradas
1. **Config manual con structs** - Rechazada porque falta validación y hot reload
2. **Solo env vars** - Rechazada porque no soporta configs complejas anidadas
3. **Spring Boot style** - Rechazada porque Vela es funcional puro, no OOP
4. **JSON only** - Rechazada porque falta type safety y validación

## Referencias
- Jira: VELA-609
- Documentación: docs/features/VELA-609/
- Inspiración: Spring Boot Config, NestJS Config, Viper (Go)

## Implementación
Ver código en: `src/config/`
Ver tests en: `tests/unit/test_config.py`