# TASK-113BM: Implementar custom serializers

## 📋 Información General
- **Historia:** VELA-607
- **Estado:** Completada ✅
- **Fecha:** 2025-12-11

## 🎯 Objetivo
Implementar el sistema de serializers personalizados que permite definir lógica de serialización/deserialización customizada para tipos complejos.

## 🔨 Implementación

### Interface Serializer<T>
```rust
pub trait Serializer<T> {
    fn serialize(value: T) -> String;
    fn deserialize(json: &str) -> Result<T, VelaError>;
}
```

### Decorator @custom(serializer)
```vela
@serializable
class User {
  @custom(DateSerializer)
  birthDate: Date

  @custom(AddressSerializer)
  address: Address
}
```

### Serializer Registry
- ✅ **SerializerRegistry**: Registro global de serializers
- ✅ **register()**: Registra serializers por nombre
- ✅ **get()**: Obtiene serializer por nombre

### Ejemplo de Serializer Personalizado
```vela
class DateSerializer implements Serializer<Date> {
  fn serialize(date: Date) -> String {
    return "\"${date.year}-${date.month}-${date.day}\"";
  }

  fn deserialize(json: String) -> Result<Date, Error> {
    // Parse JSON string to Date
    return Date::parse(json);
  }
}
```

### Casos de Uso
1. **Tipos complejos**: Fechas, UUIDs, enums custom
2. **Formateo especial**: Números, monedas, coordenadas
3. **Validación**: Serializers con validación integrada
4. **Transformaciones**: Conversión de formatos

## ✅ Criterios de Aceptación
- [x] Interface Serializer<T> definida
- [x] @custom(serializer) decorator procesado
- [x] SerializerRegistry implementado
- [x] Code generation para custom serializers
- [x] Tests de serialización custom
- [x] Documentación de ejemplos

## 🔗 Referencias
- **Jira:** [TASK-113BM](https://velalang.atlassian.net/browse/TASK-113BM)
- **Historia:** [VELA-607](https://velalang.atlassian.net/browse/VELA-607)
- **Código:** [serialization_decorators.rs](../../../compiler/src/serialization_decorators.rs)