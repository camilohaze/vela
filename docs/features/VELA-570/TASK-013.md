# TASK-013: Diseñar Representación Interna de Tipos

## 📋 Información General
- **Historia:** VELA-570
- **Estado:** ✅ Completada
- **Fecha:** 2025-12-01

## 🎯 Objetivo
Diseñar y implementar la representación interna completa de todos los tipos de Vela, desde primitivos hasta tipos complejos con generics.

## 🔨 Implementación

### Archivos generados:
- `src/type_system/types.rs` - Representación completa (700+ líneas)

### Tipos Implementados:

#### 1. **Tipos Primitivos**
```python
NUMBER_TYPE   # Number (entero 64-bit)
FLOAT_TYPE    # Float (punto flotante 64-bit)
STRING_TYPE   # String (cadena de texto)
BOOL_TYPE     # Bool (true/false)
VOID_TYPE     # void (sin retorno)
NEVER_TYPE    # never (nunca retorna)
```

#### 2. **Tipos Especiales**
```python
OptionType(inner_type)           # Option<T> - valor opcional
ResultType(ok_type, err_type)    # Result<T, E> - manejo de errores
```

#### 3. **Colecciones**
```python
ListType(element_type)                    # List<T>
SetType(element_type)                     # Set<T>
DictType(key_type, value_type)           # Dict<K, V>
TupleType([type1, type2, ...])           # (T1, T2, ...)
```

#### 4. **Funciones**
```python
FunctionType(param_types, return_type, is_async)
# Ejemplo: (Number, Number) -> Number
# Ejemplo: async (String) -> void
```

#### 5. **Estructurales**
```python
StructType(name, fields, type_params)      # struct User { ... }
EnumType(name, variants, type_params)      # enum Color { Red, Green }
ClassType(name, fields, methods, ...)      # class Person { ... }
InterfaceType(name, methods, type_params)  # interface Drawable { ... }
```

#### 6. **Keywords Específicos (Sprint 7)**
```python
KeywordSpecificType(keyword_kind, name, fields, methods)
# widget, component, service, repository, controller, usecase,
# entity, dto, valueObject, model, factory, builder, strategy,
# observer, singleton, adapter, decorator, guard, middleware,
# interceptor, validator, store, provider, actor, pipe, task,
# helper, mapper, serializer
```

#### 7. **Generics**
```python
TypeVariable(name, constraints)     # T, U, V
GenericType(base, type_args)        # List<Number>, Dict<String, T>
```

#### 8. **Tipos de Inferencia**
```python
UnknownType(id)  # Tipo desconocido durante inferencia
```

## ✅ Criterios de Aceptación
- [x] Todos los tipos primitivos definidos
- [x] Soporte completo para Option<T> y Result<T, E>
- [x] Colecciones genéricas (List, Set, Dict)
- [x] Funciones con async support
- [x] Structs, enums, classes, interfaces
- [x] Keywords específicos del Sprint 7 integrados
- [x] Generics con type variables
- [x] UnknownType para inferencia
- [x] Utilidades: is_primitive(), is_collection(), etc.

## 📊 Estructura de Clases

```
Type (base class)
├── PrimitiveType
│   ├── NUMBER_TYPE
│   ├── FLOAT_TYPE
│   ├── STRING_TYPE
│   ├── BOOL_TYPE
│   ├── VOID_TYPE
│   └── NEVER_TYPE
├── OptionType
├── ResultType
├── TupleType
├── ListType
├── SetType
├── DictType
├── FunctionType
├── StructType
├── EnumType
├── ClassType
├── InterfaceType
├── TypeVariable
├── GenericType
├── UnknownType
└── KeywordSpecificType
```

## 🧪 Tests
Todos los tests implementados en `test_type_system.py`:
- ✅ Test de tipos primitivos
- ✅ Test de Option<T>
- ✅ Test de Result<T, E>
- ✅ Test de List<T>, Set<T>, Dict<K, V>
- ✅ Test de funciones (sync y async)
- ✅ Test de tuplas
- ✅ Test de structs
- ✅ Test de enums
- ✅ Test de type variables
- ✅ Test de UnknownType

## 💡 Decisiones de Diseño

### 1. **Inmutabilidad por Defecto**
Todos los tipos son inmutables. La mutabilidad se indica con el flag `mutable` en el Symbol, no en el Type.

### 2. **Option<T> en lugar de null**
No existe `null`, `undefined` ni `nil`. Se usa `Option<T>` con `Some(value)` o `None`.

### 3. **Keywords Específicos como Tipos**
Los 30 keywords del Sprint 7 (widget, service, etc.) son tipos de primera clase con metadatos específicos.

### 4. **Generics Estructurados**
- `TypeVariable`: Variable de tipo genérica (T, U, V)
- `GenericType`: Instanciación de tipo genérico (List<Number>)
- Constraints opcionales sobre type variables

### 5. **UnknownType para Inferencia**
Cada tipo desconocido tiene un ID único para tracking durante inferencia Hindley-Milner.

## 🔗 Referencias
- **Código:** `src/type_system/types.rs`
- **Tests:** `tests/unit/type_system/test_type_system.py`
- **Historia:** [VELA-570](https://velalang.atlassian.net/browse/VELA-570)
