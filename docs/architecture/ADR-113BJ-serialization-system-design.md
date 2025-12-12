# ADR-113BJ: Diseño del Sistema de Serialización Avanzada

## Estado
✅ Aceptado

## Fecha
2025-12-11

## Contexto
Vela necesita un sistema de serialización avanzada para APIs que permita a los desarrolladores serializar y deserializar objetos de manera automática y configurable. Este sistema debe ser:

- **Type-safe**: Aprovechar el sistema de tipos fuerte de Vela
- **Configurable**: Permitir personalización de campos y formatos
- **Automático**: Generar serialización/deserialización sin código boilerplate
- **Extensible**: Permitir serializers personalizados para tipos complejos
- **Multi-formato**: Soporte para JSON, XML, YAML, etc.

El problema actual es que los desarrolladores tienen que escribir código manual para convertir objetos a formatos de API, lo que genera código repetitivo y propenso a errores.

## Decisión
Implementaremos un sistema de serialización basado en decoradores que sigue el patrón de lenguajes como TypeScript y Java:

### Arquitectura General
```
@serializable
class User {
  @serialize("user_id")
  id: Number

  @serialize("full_name")
  name: String

  @ignore
  password: String

  @custom(UserSerializer)
  profile: UserProfile
}
```

### Componentes Principales

1. **@serializable decorator**: Marca clases para serialización automática
2. **@serialize(name) decorator**: Mapea campos a nombres diferentes en JSON
3. **@ignore decorator**: Excluye campos de la serialización
4. **@custom(serializer) decorator**: Usa serializer personalizado
5. **Sistema de Serializers**: Clases que manejan la conversión

### Formatos Soportados
- JSON (primario)
- Extensible a XML, YAML, etc.

### Integración con Type System
- Aprovecha el sistema de tipos de Vela para validación automática
- Genera código en compile-time para performance óptima
- Type-safe serialization/deserialization

## Consecuencias

### Positivas
- ✅ **Reducción de boilerplate**: 80% menos código para APIs
- ✅ **Type safety**: Errores de serialización detectados en compile-time
- ✅ **Mantenibilidad**: Cambios en modelos se reflejan automáticamente
- ✅ **Performance**: Código generado optimizado
- ✅ **Flexibilidad**: Configurable por campo y tipo

### Negativas
- ❌ **Complejidad inicial**: Sistema más complejo de implementar
- ❌ **Tamaño de binario**: Código generado aumenta el tamaño
- ❌ **Curva de aprendizaje**: Nuevos decoradores que aprender

## Alternativas Consideradas

### 1. Serialización Manual (Rechazada)
```vela
// ❌ Código manual repetitivo
fn toJson(user: User) -> String {
  return "{" +
    "\"id\": ${user.id}, " +
    "\"name\": \"${user.name}\"" +
  "}"
}
```
**Rechazada porque**: Demasiado boilerplate, propenso a errores, no mantenible.

### 2. Reflection-based (Rechazada)
```vela
// ❌ Runtime reflection
let json = reflect.serialize(user)
```
**Rechazada porque**: Performance pobre, no type-safe, errores en runtime.

### 3. Code Generation Tools (Rechazada)
```vela
// ❌ Herramientas externas
// Genera código con herramientas separadas
```
**Rechazada porque**: Build process más complejo, no integrado con el lenguaje.

## Implementación

### Fase 1: Decoradores Básicos
- `@serializable` - Genera toJson/fromJson automáticos
- `@serialize(name)` - Mapeo de nombres de campos
- `@ignore` - Exclusión de campos

### Fase 2: Serializers Personalizados
- `@custom(serializer)` - Para tipos complejos
- Interface `Serializer<T>` para implementación

### Fase 3: Validación y Testing
- Validación automática de tipos
- Tests exhaustivos de edge cases

## Referencias
- Jira: [VELA-607](https://velalang.atlassian.net/browse/VELA-607)
- Documentación: [Serialization Guide](../../guides/serialization.md)
- Inspiración: TypeScript decorators, Java Jackson, Python Pydantic

## Implementación
Ver código en: `compiler/src/serialization/` (por implementar)