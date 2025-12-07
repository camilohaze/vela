# TASK-084: Implementar Dict<K,V>

## 📋 Información General
- **Historia:** VELA-589
- **Estado:** Pendiente
- **Fecha:** 2025-12-07

## 🎯 Objetivo
Implementar Dict<K,V>, la colección mutable de hash map para key-value pairs en Vela.

## 🔨 Implementación

### API Completa de Dict<K,V>

#### Constructores
```rust
// Crear diccionario vacío
let mut dict = Dict::new();

// Crear con capacidad inicial
let mut dict = Dict::with_capacity(10);

// Crear desde iterador de pares (K, V)
let dict = Dict::from_iter(vec![("a", 1), ("b", 2)]);
```

#### Operaciones Básicas
```rust
// Insertar/actualizar valor
dict.insert("key", "value");  // Retorna Option<V> (valor anterior si existía)

// Obtener valor
let value = dict.get("key");  // Retorna Option<&V>

// Remover entrada
let removed = dict.remove("key");  // Retorna Option<V>

// Verificar existencia
let exists = dict.contains_key("key");

// Longitud
let len = dict.len();
let is_empty = dict.is_empty();

// Limpiar
dict.clear();
```

#### Iteración y Acceso
```rust
// Obtener todas las keys
let keys: Vec<&K> = dict.keys();

// Obtener todos los values
let values: Vec<&V> = dict.values();

// Obtener todos los entries como (key, value)
let entries: Vec<(&K, &V)> = dict.entries();
```

#### Operaciones Avanzadas
```rust
// Obtener con valor por defecto
let value = dict.get_or_default("key", "default_value");

// Insertar solo si no existe
let inserted = dict.insert_if_absent("key", "value");  // Retorna bool

// Actualizar valor existente
dict.update("key", |old_value| old_value + 1);

// Fusionar con otro diccionario
dict.merge(other_dict);  // Sobrescribe valores existentes
dict.merge_new(other_dict);  // Solo inserta si key no existe
```

#### API Funcional
```rust
// Mapear valores
let doubled = dict.map_values(|v| v * 2);

// Filtrar entries
let filtered = dict.filter(|k, v| k.starts_with("prefix"));

// Ejecutar función por cada entry
dict.for_each(|k, v| println!("{}: {}", k, v));

// Encontrar primera entry que cumple condición
let found = dict.find(|k, v| v > 10);  // Retorna Option<(&K, &V)>

// Verificar si alguna entry cumple condición
let some_match = dict.some(|k, v| v % 2 == 0);

// Verificar si todas las entries cumplen condición
let all_match = dict.every(|k, v| v > 0);
```

### Restricciones Genéricas
```rust
// Dict<K, V> requiere que K: Eq + Hash
pub struct Dict<K, V>
where
    K: Eq + Hash,
{
    items: HashMap<K, V>,
}
```

### Conversiones
```rust
// Convertir a HashMap
let hashmap: HashMap<String, i32> = dict.into();

// Crear desde HashMap
let dict = Dict::from(hashmap);

// Convertir a vector de pares
let pairs: Vec<(K, V)> = dict.into_pairs();

// Crear desde vector de pares
let dict = Dict::from_pairs(vec![("a", 1), ("b", 2)]);
```

## ✅ Criterios de Aceptación

### Funcionalidad Básica
- [ ] `Dict::new()` crea diccionario vacío
- [ ] `insert(key, value)` funciona correctamente
- [ ] `get(key)` retorna `Some(&value)` o `None`
- [ ] `remove(key)` retorna valor removido o `None`
- [ ] `contains_key(key)` funciona correctamente
- [ ] `len()` e `is_empty()` reportan correctamente
- [ ] `clear()` remueve todos los elementos

### Iteración
- [ ] `keys()` retorna vector con todas las keys
- [ ] `values()` retorna vector con todos los values
- [ ] `entries()` retorna vector de tuplas (key, value)

### Operaciones Avanzadas
- [ ] `get_or_default(key, default)` funciona
- [ ] `insert_if_absent(key, value)` solo inserta si no existe
- [ ] `update(key, f)` actualiza valor existente
- [ ] `merge(other)` combina diccionarios
- [ ] `merge_new(other)` combina sin sobrescribir

### API Funcional
- [ ] `map_values(f)` transforma valores
- [ ] `filter(f)` filtra entries
- [ ] `for_each(f)` ejecuta función por entry
- [ ] `find(f)` encuentra primera entry que cumple
- [ ] `some(f)` verifica si alguna entry cumple
- [ ] `every(f)` verifica si todas las entries cumplen

### Conversiones
- [ ] `From<HashMap<K, V>>` funciona
- [ ] `Into<HashMap<K, V>>` funciona
- [ ] `from_pairs(pairs)` funciona
- [ ] `into_pairs()` funciona

### Tests
- [ ] 20+ tests unitarios
- [ ] Tests de edge cases (keys duplicadas, valores None)
- [ ] Tests de operaciones funcionales
- [ ] Cobertura >80%

## 🔗 Referencias

### Inspiración
- **Rust**: `HashMap<K, V>` - API base y restricciones
- **Swift**: `Dictionary<Key, Value>` - ergonomía y métodos
- **Kotlin**: `MutableMap<K, V>` - API funcional completa
- **Python**: `dict` - simplicidad y flexibilidad

### Documentación Técnica
- [Rust HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
- [Swift Dictionary](https://developer.apple.com/documentation/swift/dictionary)
- [Kotlin Map](https://kotlinlang.org/api/latest/jvm/stdlib/kotlin.collections/-map/)

## 📊 Métricas Esperadas
- **Líneas de código**: ~500 líneas en dict.rs
- **Tests**: 20+ tests unitarios
- **Complejidad**: Similar a Set<T> pero con key-value pairs
- **Performance**: O(1) promedio para operaciones básicas</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-589\TASK-084.md