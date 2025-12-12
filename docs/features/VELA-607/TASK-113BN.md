# TASK-113BN: Implementar @ignore decorator

## 📋 Información General
- **Historia:** VELA-607
- **Estado:** Completada ✅
- **Fecha:** 2025-12-11

## 🎯 Objetivo
Implementar el decorator @ignore que permite excluir campos sensibles o no serializables de la serialización automática.

## 🔨 Implementación

### Decorator @ignore
```vela
@serializable
class User {
  id: Number
  name: String
  email: String

  @ignore
  password: String

  @ignore
  internalId: String
}
```

### Genera JSON Seguro
```json
{
  "id": 123,
  "name": "John Doe",
  "email": "john@example.com"
  // password e internalId no incluidos
}
```

### Casos de Uso
1. **Campos sensibles**: Passwords, tokens, keys
2. **Campos internos**: IDs internos, timestamps de sistema
3. **Campos calculados**: Valores derivados que no se persisten
4. **Campos temporales**: Datos de sesión o cache

### Implementación Técnica
- ✅ **FieldConfig::Ignore**: Configuración de campo ignorado
- ✅ **Code Generation**: Campos ignorados no se incluyen en JSON
- ✅ **Validation**: Verifica que @ignore no tenga argumentos
- ✅ **Type Safety**: Validación en compile-time

## ✅ Criterios de Aceptación
- [x] @ignore decorator procesado correctamente
- [x] Campos marcados como ignorados no se serializan
- [x] Validación de que no acepta argumentos
- [x] Tests de exclusión correcta
- [x] Documentación de casos de uso

## 🔗 Referencias
- **Jira:** [TASK-113BN](https://velalang.atlassian.net/browse/TASK-113BN)
- **Historia:** [VELA-607](https://velalang.atlassian.net/browse/VELA-607)
- **Código:** [serialization_decorators.rs](../../../compiler/src/serialization_decorators.rs)