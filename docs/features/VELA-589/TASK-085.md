# TASK-085: Implementar Queue y Stack

## 📋 Información General
- **Historia:** VELA-589
- **Estado:** Pendiente
- **Fecha:** 2025-12-07

## 🎯 Objetivo
Implementar Queue (FIFO) y Stack (LIFO) como estructuras de datos adicionales para casos de uso específicos en Vela.

## 🔨 Implementación

### Queue<T> - FIFO (First In, First Out)

#### API Básica
```rust
// Crear queue vacío
let mut queue = Queue::new();

// Agregar elementos (enqueue)
queue.push(1);
queue.push(2);
queue.push(3);

// Remover elementos (dequeue)
let first = queue.pop();  // Some(1)
let second = queue.pop(); // Some(2)

// Inspeccionar sin remover
let front = queue.peek(); // Some(3)

// Estado
let len = queue.len();     // 1
let is_empty = queue.is_empty(); // false
```

#### Operaciones Avanzadas
```rust
// Crear con capacidad
let mut queue = Queue::with_capacity(10);

// Limpiar
queue.clear();

// Verificar si contiene elemento
let contains = queue.contains(&value);

// Convertir a vector
let vec = queue.into_vec();
```

### Stack<T> - LIFO (Last In, First Out)

#### API Básica
```rust
// Crear stack vacío
let mut stack = Stack::new();

// Agregar elementos (push)
stack.push(1);
stack.push(2);
stack.push(3);

// Remover elementos (pop)
let last = stack.pop();   // Some(3)
let second = stack.pop(); // Some(2)

// Inspeccionar sin remover
let top = stack.peek();   // Some(1)

// Estado
let len = stack.len();     // 1
let is_empty = stack.is_empty(); // false
```

#### Operaciones Avanzadas
```rust
// Crear con capacidad
let mut stack = Stack::with_capacity(10);

// Limpiar
stack.clear();

// Verificar si contiene elemento
let contains = stack.contains(&value);

// Convertir a vector
let vec = stack.into_vec();
```

## ✅ Criterios de Aceptación

### Queue<T>
- [ ] `Queue::new()` crea queue vacío
- [ ] `push(item)` agrega elemento al final
- [ ] `pop()` remueve y retorna primer elemento (FIFO)
- [ ] `peek()` retorna primer elemento sin removerlo
- [ ] `len()` e `is_empty()` reportan correctamente
- [ ] `clear()` remueve todos los elementos
- [ ] `contains(item)` verifica existencia
- [ ] `into_vec()` convierte a vector
- [ ] 8-10 tests unitarios

### Stack<T>
- [ ] `Stack::new()` crea stack vacío
- [ ] `push(item)` agrega elemento a la cima
- [ ] `pop()` remueve y retorna elemento de la cima (LIFO)
- [ ] `peek()` retorna elemento de la cima sin removerlo
- [ ] `len()` e `is_empty()` reportan correctamente
- [ ] `clear()` remueve todos los elementos
- [ ] `contains(item)` verifica existencia
- [ ] `into_vec()` convierte a vector
- [ ] 8-10 tests unitarios

### Diseño Común
- [ ] Ambas estructuras usan `Vec<T>` internamente
- [ ] API consistente y simple
- [ ] Eficiencia O(1) para operaciones principales
- [ ] Bounds checking apropiado
- [ ] Display trait implementado

## 🔗 Referencias

### Inspiración
- **Rust**: `VecDeque<T>` para Queue, pero API más simple
- **Swift**: `Array` con métodos push/pop para Stack
- **Java**: `Queue` interface, `Stack` class
- **Python**: `collections.deque` para Queue, list para Stack

### Casos de Uso
- **Queue**: Breadth-first search, task scheduling, print queues
- **Stack**: Depth-first search, expression evaluation, undo/redo
- **Vela**: Algoritmos, parsing, ejecución de código

## 📊 Métricas Esperadas
- **Líneas de código**: ~200 líneas en queue.rs + ~200 líneas en stack.rs
- **Tests**: 16-20 tests unitarios totales
- **Complejidad**: Simple, enfocadas en casos de uso específicos
- **Performance**: O(1) para push/pop/peek</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-589\TASK-085.md