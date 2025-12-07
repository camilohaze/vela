# TASK-094: Implementar Decorators JSON

## 📋 Información General
- **Historia:** VELA-592 (JSON serialization)
- **Estado:** Pendiente ⏳
- **Fecha:** 2024-12-30

## 🎯 Objetivo
Implementar decorators para automatizar la serialización JSON de tipos personalizados en Vela, permitiendo que structs y clases se conviertan automáticamente a/from JSON.

## 🔨 Implementación Planificada

### Decorators a Implementar

#### `@json_serializable`
Decorator para marcar clases/structs como serializables a JSON.

```vela
@json_serializable
class User {
    id: Number
    name: String
    email: String
    active: Bool
}

// Genera automáticamente:
// - fn to_json() -> String
// - static fn from_json(json: String) -> Result<Self>
```

#### `@json_field`
Decorator para customizar campos individuales.

```vela
@json_serializable
class Product {
    @json_field(name="product_id")
    id: Number
    
    @json_field(skip=true)
    internal_code: String
    
    @json_field(default="N/A")
    description: String
}
```

#### `@json_ignore`
Decorator para excluir campos de la serialización.

```vela
class Config {
    host: String
    port: Number
    
    @json_ignore
    password: String  // No se serializa
}
```

### Funcionalidades

#### Serialización Automática
- Campos públicos → propiedades JSON
- Campos privados → ignorados por defecto
- Tipos básicos → conversión automática
- Tipos complejos → recursión automática

#### Deserialización Automática
- JSON → instancia del tipo
- Validación de tipos
- Valores por defecto
- Manejo de campos opcionales

#### Customización
- Nombres de campos alternativos
- Campos opcionales vs requeridos
- Valores por defecto
- Transformaciones custom

## ✅ Criterios de Aceptación

### Funcionalidad
- [ ] `@json_serializable` funciona en structs y classes
- [ ] Serialización automática de campos públicos
- [ ] Deserialización con validación de tipos
- [ ] `@json_field` para customización de campos
- [ ] `@json_ignore` para excluir campos
- [ ] Manejo de tipos nested (structs dentro de structs)

### Calidad
- [ ] Tests unitarios completos
- [ ] Documentación completa
- [ ] Ejemplos de uso

### Performance
- [ ] Serialización eficiente
- [ ] Sin reflection runtime cost
- [ ] Code generation en compile-time

## 🔗 Referencias

- **Jira:** [TASK-094](https://velalang.atlassian.net/browse/TASK-094)
- **Historia:** [VELA-592](https://velalang.atlassian.net/browse/VELA-592)
- **Inspiración:** serde (Rust), Jackson (Java), System.Text.Json (C#)