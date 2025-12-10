# TASK-085: Implementar Queue y Stack

## 📋 Información General
- **Historia:** EPIC-07: Standard Library
- **User Story:** US-19: Como desarrollador, quiero colecciones estándar (List, Set, Dict)
- **Estado:** Completada ✅
- **Fecha:** 2025-12-09
- **Sprint:** Sprint 26

## 🎯 Objetivo
Implementar las estructuras de datos Queue (FIFO) y Stack (LIFO) para la librería estándar de Vela, proporcionando operaciones eficientes para casos de uso comunes.

## 🔨 Implementación

### Queue<T> - FIFO Collection
```rust
use vela_stdlib::collections::Queue;

let mut queue = Queue::new();
queue.push(1);
queue.push(2);
queue.push(3);

assert_eq!(queue.pop(), Some(1)); // FIFO: first in, first out
assert_eq!(queue.peek(), Some(2)); // Look at next without removing
```

### Stack<T> - LIFO Collection
```rust
use vela_stdlib::collections::Stack;

let mut stack = Stack::new();
stack.push(1);
stack.push(2);
stack.push(3);

assert_eq!(stack.pop(), Some(3)); // LIFO: last in, first out
assert_eq!(stack.peek(), Some(2)); // Look at top without removing
```

## 📁 Archivos Generados
- `stdlib/src/collections/queue.rs` - Implementación completa de Queue<T>
- `stdlib/src/collections/stack.rs` - Implementación completa de Stack<T>
- `stdlib/src/collections/mod.rs` - Exports actualizados

## 🏗️ Arquitectura

### Queue<T>
- **Base:** `Vec<T>` para simplicidad y eficiencia
- **Semántica:** FIFO (First In, First Out)
- **Complejidad:** O(1) para push/pop/peek
- **API:** push, pop, peek, peek_mut, len, is_empty, clear, contains

### Stack<T>
- **Base:** `Vec<T>` para simplicidad y eficiencia
- **Semántica:** LIFO (Last In, First Out)
- **Complejidad:** O(1) para push/pop/peek
- **API:** push, pop, peek, peek_mut, len, is_empty, clear, contains

## ✅ API Methods

### Queue<T> Methods
| Method | Descripción | Complejidad |
|--------|-------------|-------------|
| `new()` | Crear queue vacío | O(1) |
| `with_capacity(n)` | Crear con capacidad inicial | O(1) |
| `push(item)` | Agregar al final | O(1) |
| `pop()` | Remover del frente | O(1) |
| `peek()` | Ver elemento del frente | O(1) |
| `peek_mut()` | Referencia mutable al frente | O(1) |
| `len()` | Número de elementos | O(1) |
| `is_empty()` | Verificar si está vacío | O(1) |
| `clear()` | Remover todos los elementos | O(n) |
| `contains(item)` | Verificar si contiene elemento | O(n) |
| `reserve(n)` | Reservar capacidad adicional | O(1) |
| `shrink_to_fit()` | Reducir capacidad al mínimo | O(n) |

### Stack<T> Methods
| Method | Descripción | Complejidad |
|--------|-------------|-------------|
| `new()` | Crear stack vacío | O(1) |
| `with_capacity(n)` | Crear con capacidad inicial | O(1) |
| `push(item)` | Agregar a la cima | O(1) |
| `pop()` | Remover de la cima | O(1) |
| `peek()` | Ver elemento de la cima | O(1) |
| `peek_mut()` | Referencia mutable a la cima | O(1) |
| `len()` | Número de elementos | O(1) |
| `is_empty()` | Verificar si está vacío | O(1) |
| `clear()` | Remover todos los elementos | O(n) |
| `contains(item)` | Verificar si contiene elemento | O(n) |
| `reserve(n)` | Reservar capacidad adicional | O(1) |
| `shrink_to_fit()` | Reducir capacidad al mínimo | O(n) |

## 🧪 Tests Implementados

### Queue Tests (12 tests)
- `test_queue_push_pop` - Operaciones básicas FIFO
- `test_queue_peek` - Peek sin remover
- `test_queue_peek_mut` - Peek mutable
- `test_queue_contains` - Verificación de contenido
- `test_queue_clear` - Limpieza completa
- `test_queue_into_vec` - Conversión a Vec
- `test_queue_from_vec` - Creación desde Vec
- `test_queue_from_slice` - Creación desde slice
- `test_queue_display` - Formato de display
- `test_queue_empty_display` - Display de queue vacío
- `test_queue_single_element_display` - Display con un elemento

### Stack Tests (12 tests)
- `test_stack_push_pop` - Operaciones básicas LIFO
- `test_stack_peek` - Peek sin remover
- `test_stack_peek_mut` - Peek mutable
- `test_stack_contains` - Verificación de contenido
- `test_stack_clear` - Limpieza completa
- `test_stack_into_vec` - Conversión a Vec
- `test_stack_from_vec` - Creación desde Vec
- `test_stack_from_slice` - Creación desde slice
- `test_stack_display` - Formato de display
- `test_stack_empty_display` - Display de stack vacío
- `test_stack_single_element_display` - Display con un elemento

## 📊 Métricas de Calidad

### Cobertura de Tests
- **Queue:** 12/12 tests pasando (100%)
- **Stack:** 12/12 tests pasando (100%)
- **Total:** 24 tests unitarios

### Complejidad
- **Tiempo:** Todas las operaciones principales O(1)
- **Espacio:** O(n) donde n es el número de elementos
- **Eficiencia:** Uso óptimo de Vec<T> interno

### Documentación
- **README:** Documentación completa en archivos fuente
- **Ejemplos:** Casos de uso básicos incluidos
- **API:** Todos los métodos documentados

## 🔗 Referencias
- **Jira:** [TASK-085](https://velalang.atlassian.net/browse/TASK-085)
- **Historia:** [EPIC-07](https://velalang.atlassian.net/browse/EPIC-07)
- **US:** [US-19](https://velalang.atlassian.net/browse/US-19)
- **Implementación:** `stdlib/src/collections/queue.rs`, `stdlib/src/collections/stack.rs`

## ✅ Criterios de Aceptación
- [x] Queue<T> implementado con semántica FIFO correcta
- [x] Stack<T> implementado con semántica LIFO correcta
- [x] API completa y consistente entre ambas estructuras
- [x] Tests exhaustivos (24 tests total)
- [x] Documentación completa con ejemplos
- [x] Exports actualizados en mod.rs
- [x] Integración correcta con el resto de la stdlib