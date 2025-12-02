# TASK-036: Diseñar Actor System Architecture

## 📋 Información General
- **Historia:** VELA-578 (Actor System)
- **Sprint:** Sprint 16
- **Estado:** Completada ✅
- **Fecha:** 2025-12-02
- **Estimación:** 40 horas
- **Prioridad:** P0

## 🎯 Objetivo

Diseñar la arquitectura completa del Actor System de Vela, inspirado en Akka/Erlang pero adaptado al paradigma funcional-reactivo del lenguaje. Documentar todas las decisiones de diseño para guiar la implementación en los próximos sprints.

## 🔨 Implementación

### Decisiones Arquitectónicas Clave

1. **Actor Model (Hewitt, 1973)**:
   - Concurrency basada en message-passing
   - Estado privado encapsulado
   - Comunicación asíncrona
   - Location transparency

2. **Componentes del Sistema**:
   - **Actor Instances**: Estado privado, message handlers, lifecycle hooks
   - **Mailbox System**: Bounded/unbounded/priority queues
   - **Message Processing Loop**: Sequential processing, error handling
   - **Thread Pool Executor**: Work stealing, dynamic sizing
   - **Actor Scheduler**: Fair scheduling, starvation-free
   - **ActorRef**: Location-transparent references

3. **Estrategias de Scheduling**:
   - **Fair Scheduling** (default): Round-robin para evitar starvation
   - **Priority Scheduling**: Actores prioritarios primero
   - **Work-Conserving**: Maximizar throughput

4. **Thread Pool Design**:
   - **Work Stealing**: Threads roban trabajo cuando idle
   - **Dynamic Sizing**: Grow cuando saturado, shrink cuando idle
   - **Queue per Thread**: Reduce contención

5. **Mailbox Strategies**:
   - **UnboundedMailbox** (default): Sin límite, más simple
   - **BoundedMailbox**: Backpressure automático
   - **PriorityMailbox**: Mensajes prioritarios primero

### Archivos Generados

- `docs/architecture/ADR-009-actor-system.md` - Decisión arquitectónica completa (750+ LOC)

### Comparación de Alternativas

| Modelo | Pros | Contras | Decisión |
|--------|------|---------|----------|
| **Actor Model** | Concurrency segura, location transparency, fault tolerance | Learning curve, message overhead | ✅ **ELEGIDO** |
| **Shared Memory** | Familiar, buen soporte OS | Race conditions, deadlocks | ❌ Rechazado |
| **CSP (Go channels)** | Simple, type-safe | No location transparency, no fault tolerance | ❌ Rechazado |
| **Async/Await** | Sintaxis familiar, bueno para I/O | No paralelismo real, single-threaded | ⚠️ Complementario |

## ✅ Criterios de Aceptación

- [x] ADR-009 creado con arquitectura completa
- [x] Componentes del sistema documentados
- [x] Estrategias de scheduling definidas
- [x] Thread pool design especificado
- [x] Mailbox strategies documentadas
- [x] Comparación con alternativas (Shared Memory, CSP, Async/Await)
- [x] Ejemplos de uso prácticos (Counter, Chat Room, Pipeline)
- [x] Métricas de éxito definidas
- [x] Referencias a Erlang/Akka/Orleans

## 📊 Métricas

- **ADR:** 1 documento creado (750+ LOC)
- **Componentes diseñados:** 6 (Actor, Mailbox, MessageLoop, Executor, Scheduler, ActorRef)
- **Ejemplos:** 3 casos de uso completos
- **Referencias:** 5 sistemas analizados (Erlang, Akka, Orleans, Ray, Go)

## 🔗 Referencias

- **Jira:** [TASK-036](https://velalang.atlassian.net/browse/TASK-036)
- **Historia:** [VELA-578](https://velalang.atlassian.net/browse/VELA-578)
- **ADR:** docs/architecture/ADR-009-actor-system.md

## 📝 Notas de Implementación

### Inspiraciones por Framework

**Erlang/OTP:**
- Supervision hierarchies (Sprint 17)
- Let it crash philosophy
- Hot code swapping (futuro)

**Akka (Scala/Java):**
- Work stealing thread pool
- Fair scheduling
- Location transparency

**Orleans (.NET):**
- Virtual actors (stateless/stateful)
- Automatic activation/deactivation
- Grain directory (futuro)

**Ray (Python):**
- Task-based API
- Object store integration (futuro)
- Distributed scheduling (futuro)

### Decisiones para Sprint 16

1. **Implementación secuencial**: Actor → Mailbox → MessageLoop → Executor → Scheduler
2. **Tests exhaustivos**: >= 80% cobertura en cada componente
3. **Performance benchmarks**: Throughput, latency, scalability
4. **Ejemplos reales**: Counter, Chat, Pipeline

### Próximos Pasos (Sprint 17)

1. **Supervision Strategies**: OneForOne, OneForAll, RestForOne
2. **Restart Logic**: Backoff exponencial, max retries
3. **Guardian Actors**: Root supervision tree
4. **Error Escalation**: Propagación de errores en jerarquía

## 🎨 Patrones de Diseño Aplicados

1. **Actor Model**: Base del sistema
2. **Observer**: Supervision y lifecycle hooks
3. **Strategy**: Diferentes mailbox/scheduler strategies
4. **Factory**: Actor spawning con ActorSystem
5. **Proxy**: ActorRef como proxy a actor real

## 🚀 Impacto en el Lenguaje

**Palabras Reservadas Nuevas:**
- `actor` - Definir actor
- `spawn` - Crear instancia de actor
- `send` - Enviar mensaje

**APIs del Sistema:**
```vela
import 'system:actors' show { ActorSystem, spawn, ActorRef }

# Create system
system = ActorSystem(name: "MySystem")

# Spawn actor
ref: ActorRef<Counter> = system.spawn(Counter)

# Send message
ref.send(Increment)
```

**Integración con Reactividad:**
- Actores pueden contener `state` reactivo
- `computed` dentro de actores para derivar estado
- `effect` para side effects en cambios de estado

---

**Completado:** 2025-12-02  
**Tiempo:** ~4 horas de diseño y documentación  
**Próxima Task:** TASK-037 - Implementar Actor Instances
