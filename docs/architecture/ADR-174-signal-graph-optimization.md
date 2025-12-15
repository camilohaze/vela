# ADR-174: Signal Graph Optimization

## Estado
✅ Aceptado

## Fecha
2025-12-15

## Contexto
El sistema de señales reactivas de Vela necesita optimizaciones para manejar grafos complejos de dependencias de manera eficiente. Sin optimizaciones, la propagación de cambios puede ser ineficiente, causando actualizaciones innecesarias y degradación del rendimiento.

## Decisión
Implementar un sistema de optimización de grafo de señales que incluya:

1. **Análisis de dependencias estático**
2. **Memoización de valores computados**
3. **Lazy evaluation**
4. **Batching de actualizaciones**
5. **Gestión optimizada de memoria**

## Consecuencias

### Positivas
- **Mejora significativa del rendimiento** en aplicaciones reactivas complejas
- **Reducción de actualizaciones innecesarias** del DOM/UI
- **Mejor uso de memoria** con gestión optimizada
- **Escalabilidad** para aplicaciones con muchos componentes reactivos

### Negativas
- **Complejidad adicional** en el sistema de señales
- **Overhead de análisis** durante la inicialización
- **Posibles race conditions** si no se maneja correctamente el batching

## Alternativas Consideradas

### 1. Optimización Manual (Rechazada)
Los desarrolladores tendrían que optimizar manualmente sus señales.
- **Problema**: Error-prone, difícil de mantener, no escalable

### 2. Optimización Solo en Runtime (Rechazada)
Analizar dependencias solo durante ejecución.
- **Problema**: Overhead continuo, no aprovecha análisis estático

### 3. Sistema de Optimización Externo (Rechazada)
Usar un optimizador separado del runtime.
- **Problema**: Complejidad de integración, duplicación de lógica

## Implementación

### Fase 1: Análisis de Dependencias
```rust
pub struct SignalGraphAnalyzer {
    pub signals: HashMap<SignalId, SignalNode>,
    pub dependencies: HashMap<SignalId, Vec<SignalId>>,
}

impl SignalGraphAnalyzer {
    pub fn analyze_dependencies(&mut self) -> Result<(), SignalError> {
        // Análisis estático del grafo
    }

    pub fn detect_cycles(&self) -> Vec<Vec<SignalId>> {
        // Detección de dependencias circulares
    }
}
```

### Fase 2: Memoización
```rust
pub struct MemoizedSignal<T> {
    value: Option<T>,
    dependencies: Vec<SignalId>,
    last_update: u64,
}

impl<T> MemoizedSignal<T> {
    pub fn get(&mut self) -> &T {
        if self.needs_recomputation() {
            self.recompute();
        }
        self.value.as_ref().unwrap()
    }
}
```

### Fase 3: Lazy Evaluation
```rust
pub struct LazySignal<T> {
    computation: Box<dyn Fn() -> T>,
    cached_value: RefCell<Option<T>>,
    dirty: Cell<bool>,
}

impl<T> LazySignal<T> {
    pub fn get(&self) -> T {
        if self.dirty.get() {
            let value = (self.computation)();
            *self.cached_value.borrow_mut() = Some(value);
            self.dirty.set(false);
        }
        self.cached_value.borrow().clone().unwrap()
    }
}
```

## Referencias
- Jira: [TASK-174]
- Documentación: [Signals Reactive System](../../04-signals-reactive-system.md)
- Código: `runtime/src/reactive/signal_graph.rs`