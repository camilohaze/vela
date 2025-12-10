# TASK-094: Implementar JSON decorators para types

## 📋 Información General
- **Historia:** US-21: Como desarrollador, quiero serialización JSON
- **Epic:** EPIC-07: Standard Library
- **Estado:** En curso ✅
- **Fecha:** 2025-12-09
- **Sprint:** Sprint 28
- **Milestone:** Vela 1.0

## 🎯 Objetivo
Implementar sistema de decoradores JSON para serialización automática de tipos custom en Vela, permitiendo que los desarrolladores marquen clases y structs para serialización/deserialización JSON automática.

## 🔨 Implementación Técnica

### Arquitectura de Decoradores JSON

```
@json({
  include: ["field1", "field2"],     // Campos a incluir
  exclude: ["field3"],               // Campos a excluir
  rename: { "oldName": "newName" },  // Renombrar campos
  defaultValues: { "field": "value" } // Valores por defecto
})
class User {
  id: Number
  name: String
  email: String
  @json(skip)                        // Omitir campo
  password: String
  @json(rename="created_at")         // Renombrar campo
  createdAt: DateTime
}
```

### Componentes a Implementar

#### 1. JSON Decorator Parser
- Parser para `@json` decorator en el AST
- Validación de parámetros del decorator
- Integración con el sistema de tipos

#### 2. Serialization Engine
- Generación automática de código de serialización
- Soporte para tipos primitivos (Number, String, Bool)
- Soporte para colecciones (List<T>, Set<T>, Dict<K,V>)
- Soporte para tipos custom anidados
- Manejo de campos opcionales (Option<T>)

#### 3. Field-Level Decorators
- `@json(skip)` - Omitir campo en serialización
- `@json(rename="newName")` - Renombrar campo
- `@json(default="value")` - Valor por defecto
- `@json(flatten)` - Aplanar objeto anidado

#### 4. Runtime Support
- Macros de compilación para generar código
- Cache de serializadores por tipo
- Error handling para tipos no soportados

### Casos de Uso

```vela
// Caso básico
@json
class Person {
  name: String
  age: Number
}

// Con configuración
@json({
  exclude: ["internalId"],
  rename: { "createdAt": "created_at" }
})
class Product {
  id: String
  name: String
  price: Float
  @json(skip)
  internalId: String
  @json(rename="created_at")
  createdAt: DateTime
}

// Serialización anidada
@json
class Order {
  id: String
  customer: Person
  items: List<Product>
  total: Float
}
```

### API de Serialización

```vela
// Serialización
let user = User { id: 1, name: "John", email: "john@example.com" }
let json = user.toJson()  // {"id":1,"name":"John","email":"john@example.com"}

// Deserialización
let jsonStr = '{"id":2,"name":"Jane","email":"jane@example.com"}'
let user = User.fromJson(jsonStr)  // User instance

// Con opciones
let json = user.toJson({
  pretty: true,
  includeNulls: false
})
```

## ✅ Criterios de Aceptación

### Funcionalidad Core
- [ ] `@json` decorator básico funciona
- [ ] Serialización automática de campos públicos
- [ ] Deserialización automática de JSON
- [ ] Soporte para tipos primitivos
- [ ] Soporte para colecciones estándar

### Configuración Avanzada
- [ ] `@json(skip)` omite campos
- [ ] `@json(rename="...")` renombra campos
- [ ] Configuración global por clase
- [ ] Campos opcionales (Option<T>)
- [ ] Tipos custom anidados

### Calidad y Testing
- [ ] Tests unitarios completos (>=80% cobertura)
- [ ] Tests de integración con JSON parser/encoder
- [ ] Tests de error handling
- [ ] Documentación completa
- [ ] Performance benchmarks

## 🔗 Referencias
- **Jira:** [TASK-094](https://velalang.atlassian.net/browse/TASK-094)
- **Epic:** [EPIC-07](https://velalang.atlassian.net/browse/EPIC-07)
- **Dependencias:** TASK-093 (JSON encoder)
- **Documentación:** [JSON Serialization Guide](../../stdlib/json/README.md)

## 📊 Métricas de Implementación
- **Complejidad:** Alta (sistema de macros, AST manipulation)
- **Archivos nuevos:** ~5 (decorators, macros, runtime)
- **Líneas de código:** ~800
- **Tests:** ~50 casos de prueba
- **Tiempo estimado:** 48 horas