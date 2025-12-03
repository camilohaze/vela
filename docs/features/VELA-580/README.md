# VELA-580: Async/Await

## 📋 Información General
- **Epic:** EPIC-04 - Concurrency (Actors)
- **Sprint:** 18
- **Estado:** En Progreso 🔄
- **Fecha Inicio:** 2025-12-02
- **Estimación Total:** 240 horas

## 🎯 Descripción

Implementación completa de async/await en Vela para programación asíncrona moderna, incluyendo:

1. **Diseño de semántica** (TASK-045)
2. **Transformación CPS** en compilador (TASK-046)
3. **Runtime types** (`Future<T>`, `Promise<T>`) (TASK-047)
4. **Executor** (event loop) (TASK-048)
5. **Tests completos** (TASK-049)

**Motivación**:
- ✅ **I/O No Bloqueante**: Operaciones de red, filesystem, DB
- ✅ **Concurrencia Ligera**: Miles de tareas async sin overhead de threads
- ✅ **Integración Actors**: Async handlers en actores
- ✅ **APIs Modernas**: HTTP clients, WebSockets, timers

## 📦 Subtasks

### ✅ TASK-045: Diseñar async/await semantics (16h) - COMPLETADA
**Estado**: ✅ Finalizada  
**Fecha**: 2025-12-02

**Entregables**:
- ✅ ADR-012: Async/Await Semantics
- ✅ Especificación formal completa
- ✅ Gramática EBNF (sintaxis)
- ✅ Type rules (sistema de tipos)
- ✅ Semántica operacional (ejecución)
- ✅ Modelo de transformación CPS
- ✅ Diseño de Executor (event loop)
- ✅ APIs de Future<T> y Promise<T>
- ✅ Combinadores (all, race, timeout)
- ✅ Integración con actors

**Ver**: [TASK-045.md](TASK-045.md)

---

### ⏳ TASK-046: Implementar async transform (CPS) (80h) - PENDIENTE
**Estado**: ⏸️ No Iniciada  
**Dependencia**: TASK-045 ✅

**Objetivos**:
- Transformar `async fn` a state machines en compilador
- Análisis de control flow (if, match, loops)
- Generación de estados para cada `await`
- Preservación de tipos en transformación
- Optimizaciones (inline small futures, stack allocation)

**Entregables Esperados**:
- Código en `src/compiler/transforms/async_transform.rs`
- AST transformation para `async fn`
- State machine generation
- Continuation passing style (CPS)
- Tests de transformación

---

### ⏳ TASK-047: Implementar Future<T> y Promise<T> (48h) - PENDIENTE
**Estado**: ⏸️ No Iniciada  
**Dependencia**: TASK-046 ✅

**Objetivos**:
- Implementar `Future<T>` trait en runtime
- Implementar `Promise<T>` class
- Waker system (despertadores)
- Combinadores (map, flatMap, then, catch)
- Future.all(), Future.race(), Future.timeout()

**Entregables Esperados**:
- Código en `src/runtime/async/future.rs`
- Código en `src/runtime/async/promise.rs`
- Código en `src/runtime/async/waker.rs`
- Tests unitarios de Future/Promise

---

### ⏳ TASK-048: Implementar executor para futures (56h) - PENDIENTE
**Estado**: ⏸️ No Iniciada  
**Dependencia**: TASK-047 ✅

**Objetivos**:
- Event loop principal (executor)
- Task scheduling (ready queue, waiting queue)
- I/O polling (epoll/kqueue/IOCP según plataforma)
- Waker registry y wake mechanism
- Integration con timers

**Entregables Esperados**:
- Código en `src/runtime/async/executor.rs`
- Código en `src/runtime/async/task.rs`
- I/O poller por plataforma
- Tests de executor

---

### ⏳ TASK-049: Tests of async/await (40h) - PENDIENTE
**Estado**: ⏸️ No Iniciada  
**Dependencia**: TASK-048 ✅

**Objetivos**:
- Tests unitarios de transformación CPS
- Tests de Future/Promise APIs
- Tests de combinadores (all, race, timeout)
- Tests de integración con actors
- Performance benchmarks
- Edge cases (errors, nested awaits, loops)

**Entregables Esperados**:
- Tests en `tests/unit/async/`
- Tests en `tests/integration/async/`
- Benchmarks en `benches/async.rs`
- 100% cobertura de código

---

## 🔨 Arquitectura

### Modelo de Ejecución

```
┌─────────────────────────────────────────────────┐
│            Vela Async/Await Stack               │
├─────────────────────────────────────────────────┤
│  User Code (async fn, await)                    │
├─────────────────────────────────────────────────┤
│  Compiler Transform (CPS)                       │
│  ├─ async fn → State Machine                    │
│  ├─ await → Poll + Suspend                      │
│  └─ Optimization (inline, stack alloc)          │
├─────────────────────────────────────────────────┤
│  Runtime (Future<T>, Promise<T>)                │
│  ├─ Future trait (poll, map, flatMap)           │
│  ├─ Promise class (resolve, reject)             │
│  ├─ Waker system                                │
│  └─ Combinators (all, race, timeout)            │
├─────────────────────────────────────────────────┤
│  Executor (Event Loop)                          │
│  ├─ Task scheduling (ready/waiting queues)      │
│  ├─ I/O polling (epoll/kqueue/IOCP)             │
│  ├─ Waker registry                              │
│  └─ Timer wheel                                 │
├─────────────────────────────────────────────────┤
│  OS (epoll/kqueue/IOCP)                         │
└─────────────────────────────────────────────────┘
```

### Transformación CPS (Ejemplo)

**Código Original**:
```vela
async fn example() -> Number {
  x = await fetch1()
  y = await fetch2()
  return x + y
}
```

**Después de Transformación**:
```rust
fn example() -> Future<Number> {
  enum State {
    Start,
    Await1(Future<Number>),
    Await2(Number, Future<Number>),
  }
  
  StateMachine::new(State::Start, |state| match state {
    State::Start => {
      let future = fetch1();
      (State::Await1(future), Poll::Pending)
    }
    State::Await1(x) => {
      let future = fetch2();
      (State::Await2(x, future), Poll::Pending)
    }
    State::Await2(x, y) => {
      (State::Done(x + y), Poll::Ready(x + y))
    }
  })
}
```

### Integración con Actors

```vela
actor AsyncWorker {
  async fn handleMessage(self, msg: Message) -> void {
    match msg {
      FetchData(url) => {
        data = await httpClient.get(url)  # No bloquea mailbox
        this.send(self, DataReceived(data))
      }
    }
  }
}
```

**Garantías**:
- ✅ Mailbox NO se bloquea durante `await`
- ✅ Actor procesa siguiente mensaje si handler suspende
- ✅ Orden de mensajes preservado

## 📊 Métricas Objetivo

### Performance

| Operación | Target |
|-----------|--------|
| `await` ready future | < 5ns |
| Crear Future | < 20ns |
| Poll future | < 10ns |
| Wake task | < 15ns |

### Cobertura de Tests

- **Unit Tests**: >= 95%
- **Integration Tests**: >= 90%
- **Benchmarks**: >= 10 casos

## ✅ Definición de Hecho (Sprint 18)

- [ ] ✅ TASK-045: Diseño completo (ADR + Spec) ✅
- [ ] ⏳ TASK-046: Transformación CPS implementada
- [ ] ⏳ TASK-047: Future<T> y Promise<T> implementados
- [ ] ⏳ TASK-048: Executor funcionando
- [ ] ⏳ TASK-049: Tests completos (>= 95% cobertura)
- [ ] ⏳ Documentación actualizada
- [ ] ⏳ Benchmarks ejecutados
- [ ] ⏳ Pull Request merged

## 🔗 Referencias

- **Jira Historia**: [VELA-580](https://velalang.atlassian.net/browse/VELA-580)
- **Epic**: [EPIC-04 - Concurrency](https://velalang.atlassian.net/browse/VELA-04)
- **ADR**: [ADR-012 - Async/Await Semantics](../../architecture/ADR-012-async-await-semantics.md)
- **Especificación**: [Async/Await Spec](../../specifications/async-await-spec.md)

### Referencias Externas

- **Rust Async Book**: https://rust-lang.github.io/async-book/
- **JavaScript Promises**: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
- **Swift Concurrency**: https://docs.swift.org/swift-book/LanguageGuide/Concurrency.html
- **Kotlin Coroutines**: https://kotlinlang.org/docs/coroutines-overview.html

## 📈 Progreso

**Total**: 16 / 240 horas completadas (6.67%)

```
[████░░░░░░░░░░░░░░░░] 6.67%
```

**Por Tarea**:
- ✅ TASK-045: 16/16h (100%) ✅
- ⏳ TASK-046: 0/80h (0%)
- ⏳ TASK-047: 0/48h (0%)
- ⏳ TASK-048: 0/56h (0%)
- ⏳ TASK-049: 0/40h (0%)

## 🎯 Próximo Paso

**TASK-046**: Implementar async transform (CPS) en compilador (80h)

**Inicio Estimado**: Después de aprobación de diseño (TASK-045)

---

**Última Actualización**: 2025-12-02  
**Estado**: TASK-045 completada ✅
