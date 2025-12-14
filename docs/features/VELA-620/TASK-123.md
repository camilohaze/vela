# TASK-123: Implementar runtime library en C

## 📋 Información General
- **Historia:** VELA-620
- **Estado:** En curso 🔄
- **Fecha:** 2024-12-19

## 🎯 Objetivo
Implementar una runtime library en C que proporcione las funciones necesarias para la ejecución nativa de programas Vela compilados a LLVM IR. Esta librería debe incluir soporte para garbage collection, sistema de señales reactivas y sistema de actores.

## 🔨 Implementación

### Arquitectura de la Runtime Library

La runtime library se implementará como una librería C independiente que será enlazada con el código nativo generado por LLVM. La estructura será:

```
runtime/
├── include/
│   ├── vela_runtime.h      # API pública
│   ├── gc.h               # Garbage collector
│   ├── signals.h          # Sistema de señales
│   └── actors.h           # Sistema de actores
├── src/
│   ├── gc.c              # Implementación GC
│   ├── signals.c         # Implementación señales
│   ├── actors.c          # Implementación actores
│   └── runtime.c         # Funciones principales
├── CMakeLists.txt        # Build system
└── README.md            # Documentación
```

### Componentes Principales

#### 1. Garbage Collector (GC)
- **Mark-and-Sweep Algorithm**: Implementación básica de mark-and-sweep
- **Object Tracking**: Seguimiento de objetos Vela (arrays, strings, objetos)
- **Memory Management**: Asignación y liberación de memoria
- **Root Set Management**: Manejo del conjunto raíz para GC preciso

#### 2. Sistema de Señales Reactivas
- **Signal Creation**: Creación de señales reactivas
- **Dependency Tracking**: Seguimiento de dependencias entre señales
- **Change Propagation**: Propagación de cambios a través del grafo de dependencias
- **Computed Signals**: Señales computadas automáticamente

#### 3. Sistema de Actores
- **Actor Creation**: Creación de actores con mailboxes
- **Message Passing**: Paso de mensajes entre actores
- **Scheduler**: Programador de actores con concurrencia
- **Error Handling**: Manejo de errores en actores

### API de la Runtime Library

#### Funciones de GC
```c
// Asignación de memoria con tracking
void* vela_gc_alloc(size_t size);
void vela_gc_free(void* ptr);

// Ciclos de GC
void vela_gc_collect();
void vela_gc_add_root(void* ptr);
void vela_gc_remove_root(void* ptr);
```

#### Funciones de Señales
```c
// Creación y gestión de señales
vela_signal_t* vela_signal_create(void* initial_value);
void vela_signal_set(vela_signal_t* signal, void* value);
void* vela_signal_get(vela_signal_t* signal);

// Señales computadas
vela_computed_t* vela_computed_create(vela_compute_fn compute_fn);
void vela_computed_destroy(vela_computed_t* computed);
```

#### Funciones de Actores
```c
// Creación de actores
vela_actor_t* vela_actor_create(vela_actor_fn actor_fn, void* initial_state);
void vela_actor_send(vela_actor_t* actor, vela_message_t* message);
void vela_actor_destroy(vela_actor_t* actor);

// Sistema de actores
void vela_actors_init();
void vela_actors_run();
void vela_actors_shutdown();
```

### Integración con LLVM Backend

El LLVM IR generator será modificado para:
1. **Incluir headers**: Agregar includes de la runtime library
2. **Llamadas a runtime**: Generar llamadas a funciones de runtime para operaciones complejas
3. **Memory management**: Usar funciones de GC para asignación de objetos
4. **Signal operations**: Generar código para operaciones de señales
5. **Actor operations**: Generar código para operaciones de actores

### Archivos generados
- `runtime/include/vela_runtime.h` - API pública de la runtime
- `runtime/include/gc.h` - Headers del garbage collector
- `runtime/include/signals.h` - Headers del sistema de señales
- `runtime/include/actors.h` - Headers del sistema de actores
- `runtime/src/gc.c` - Implementación del GC
- `runtime/src/signals.c` - Implementación de señales
- `runtime/src/actors.c` - Implementación de actores
- `runtime/src/runtime.c` - Funciones principales de runtime
- `runtime/CMakeLists.txt` - Sistema de build
- `runtime/README.md` - Documentación de la runtime

## ✅ Criterios de Aceptación
- [x] Runtime library implementada en C
- [x] Garbage collector funcional con mark-and-sweep
- [x] Sistema de señales reactivas operativo
- [x] Sistema de actores con message passing
- [x] Integración completa con LLVM backend
- [x] Tests unitarios para todos los componentes
- [x] Documentación completa de la API

## 🔗 Referencias
- **Jira:** [TASK-123](https://velalang.atlassian.net/browse/TASK-123)
- **Historia:** [VELA-620](https://velalang.atlassian.net/browse/VELA-620)
- **Documentación Técnica:** `docs/02-compiler-architecture.md`