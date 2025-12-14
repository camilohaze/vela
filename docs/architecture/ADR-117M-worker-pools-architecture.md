# ADR-117M: Arquitectura de Worker Pools para Procesamiento Paralelo

## Estado
✅ Aceptado

## Fecha
2025-12-14

## Contexto
Vela necesita capacidades de procesamiento paralelo para manejar tareas computacionalmente intensivas de manera eficiente. Los worker pools permiten distribuir trabajo entre múltiples hilos o procesos, aprovechando la concurrencia para mejorar el rendimiento. Esta funcionalidad es crítica para casos de uso como procesamiento de datos, cálculos científicos, y operaciones I/O intensivas.

## Decisión
Implementaremos una arquitectura de worker pools basada en los siguientes principios:

1. **WorkerPool Class**: Clase principal que gestiona un pool de workers
2. **Load Balancing**: Distribución automática de tareas usando round-robin
3. **Task Scheduling**: Sistema de colas con prioridades
4. **Parallel Operations**: Map/reduce operations sobre colecciones
5. **Resource Management**: Límites configurables de workers y memoria

### Componentes Arquitectónicos

#### WorkerPool
```rust
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    max_workers: usize,
    active_tasks: Arc<AtomicUsize>,
}
```

#### Worker
```rust
struct Worker {
    id: usize,
    handle: JoinHandle<()>,
    receiver: Receiver<Task>,
}
```

#### Task
```rust
enum Task {
    Map { data: Vec<T>, mapper: Box<dyn Fn(T) -> U> },
    Reduce { data: Vec<T>, reducer: Box<dyn Fn(T, T) -> T> },
    Custom { function: Box<dyn FnOnce() -> Result<(), Error>> },
}
```

## Consecuencias

### Positivas
- **Mejor Rendimiento**: Paralelización automática de operaciones costosas
- **Escalabilidad**: Fácil ajuste del número de workers según recursos
- **API Simple**: Integración natural con streams y colecciones
- **Resource Safety**: Gestión automática de lifecycle de workers

### Negativas
- **Overhead**: Costo adicional para tareas pequeñas
- **Complejidad**: Manejo de concurrencia y sincronización
- **Memory Usage**: Workers consumen memoria adicional

## Alternativas Consideradas
1. **Thread Pool Library Externa**: Usar tokio::spawn o rayon - Rechazada porque necesitamos integración nativa con Vela runtime y tipos
2. **Actor-Based**: Sistema basado en actores en lugar de threads - Rechazada porque menos eficiente para CPU-bound tasks
3. **Fork-Based**: Usar procesos en lugar de threads - Rechazada porque mayor overhead y complejidad de comunicación

## Referencias
- Jira: [VELA-1113]
- Documentación: docs/features/VELA-1113/

## Implementación
Ver código en: `runtime/src/worker_pool.rs`