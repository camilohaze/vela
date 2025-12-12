# TASK-113BL: Implementar field mapping (@serialize name)

## 📋 Información General
- **Historia:** VELA-607
- **Estado:** Completada ✅
- **Fecha:** 2025-12-11

## 🎯 Objetivo
Implementar el decorator @serialize(name) que permite mapear nombres de campos de clases a nombres diferentes en la serialización JSON.

## 🔨 Implementación

### Decorator @serialize(name)
```vela
@serializable
class User {
  @serialize("user_id")
  id: Number

  @serialize("full_name")
  name: String

  email: String  // Sin decorator, usa nombre original
}
```

### Genera JSON
```json
{
  "user_id": 123,
  "full_name": "John Doe",
  "email": "john@example.com"
}
```

### Implementación Técnica
- ✅ **Parsing**: Decorator acepta un argumento string literal
- ✅ **Validación**: Verifica que el argumento sea string válido
- ✅ **Code Generation**: Mapea nombres en toJson/fromJson
- ✅ **Type Safety**: Validación en compile-time

### Casos de Uso
1. **APIs externas**: Adaptar nombres de campos a estándares externos
2. **Legacy systems**: Mantener compatibilidad con sistemas existentes
3. **Naming conventions**: Convertir camelCase a snake_case o viceversa

## ✅ Criterios de Aceptación
- [x] @serialize("name") acepta string literals
- [x] Validación de argumentos en compile-time
- [x] Code generation usa nombres mapeados
- [x] Integración con @serializable
- [x] Tests de mapeo correcto
- [x] Documentación de uso

## 🔗 Referencias
- **Jira:** [TASK-113BL](https://velalang.atlassian.net/browse/TASK-113BL)
- **Historia:** [VELA-607](https://velalang.atlassian.net/browse/VELA-607)
- **Código:** [serialization_decorators.rs](../../../compiler/src/serialization_decorators.rs)