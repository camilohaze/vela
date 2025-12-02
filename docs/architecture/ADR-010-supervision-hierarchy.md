# ADR-010: Supervision Hierarchy para Actor System

## Estado
✅ Aceptado

## Fecha
2025-12-02

## Contexto

El Actor System implementado en Sprint 16 (VELA-578) carece de mecanismos de tolerancia a fallos. Cuando un actor falla durante el procesamiento de un mensaje, actualmente:
- El error se propaga sin control
- El actor queda en estado inconsistente
- No hay reinicio automático
- No hay aislamiento de errores

**Necesitamos un sistema de supervisión** inspirado en Erlang/OTP y Akka que permita:
1. **Contener fallos**: Evitar que errores se propaguen globalmente
2. **Reiniciar actors**: Estrategias automáticas de recuperación
3. **Jerarquías**: Organizar actors en árboles de supervisión
4. **Escalación**: Propagar errores no recuperables al supervisor padre

## Decisión

Implementaremos un **Sistema de Supervisión Jerárquico** con las siguientes características:

### 1. Estrategias de Supervisión (3 tipos)

Inspirado en Erlang/OTP y Akka:

#### A) OneForOne Strategy
```
Supervisor
├── Child A (❌ CRASHED)  → Solo A se reinicia
├── Child B (✅ OK)       → Sigue ejecutando
└── Child C (✅ OK)       → Sigue ejecutando
```

**Cuándo usar**: Cuando los children son independientes entre sí.

#### B) OneForAll Strategy
```
Supervisor
├── Child A (❌ CRASHED)  → Todos se reinician
├── Child B (⚠️ STOP)     → Reiniciado
└── Child C (⚠️ STOP)     → Reiniciado
```

**Cuándo usar**: Cuando los children comparten estado y uno fallido invalida a todos.

#### C) RestForOne Strategy
```
Supervisor
├── Child A (✅ OK)       → Sigue ejecutando
├── Child B (❌ CRASHED)  → B se reinicia
└── Child C (⚠️ STOP)     → C se reinicia (era hijo de B)
```

**Cuándo usar**: Cuando existe dependencia temporal entre children (pipelines).

### 2. Restart Policies (Políticas de Reinicio)

Basado en Akka Supervision Strategies:

```python
@dataclass
class RestartPolicy:
    """Política de reinicio de actors."""
    
    max_retries: int = 3           # Máximo de reinicios
    within_time_window: float = 60.0  # Ventana de tiempo (segundos)
    backoff_strategy: BackoffStrategy = BackoffStrategy.EXPONENTIAL
    initial_delay: float = 1.0     # Delay inicial
    max_delay: float = 30.0        # Delay máximo
```

**Backoff Strategies**:
- **CONSTANT**: Delay fijo entre reinicios
- **LINEAR**: Incremento lineal del delay
- **EXPONENTIAL**: Incremento exponencial (1s → 2s → 4s → 8s...)

### 3. Lifecycle de Supervisión

```
Actor falla → Supervisor detecta → Aplica Strategy → Ejecuta Restart Policy
                                       ↓
                                 Si excede retries
                                       ↓
                                   Escalación al supervisor padre
```

**Hooks del ciclo de vida**:
- `pre_restart()`: Antes de reiniciar (cleanup)
- `post_restart()`: Después de reiniciar (reinit)
- `on_failure_escalated()`: Cuando se escala al padre

### 4. Arquitectura de Clases

```python
# Clase base para supervisores
class SupervisorActor(Actor):
    """Actor que supervisa otros actors."""
    
    def __init__(self, strategy: SupervisionStrategy):
        self.strategy = strategy
        self.children: Dict[str, ActorRef] = {}
        self.restart_counts: Dict[str, RestartStats] = {}
    
    def spawn_child(self, actor_class, name: str) -> ActorRef:
        """Crea actor hijo supervisado."""
        pass
    
    def handle_child_failure(self, child_ref: ActorRef, error: Exception):
        """Maneja fallo de hijo según strategy."""
        pass

# Estrategias concretas
class OneForOneStrategy(SupervisionStrategy):
    def handle_failure(self, supervisor, failed_child, error):
        # Solo reinicia el child fallido
        supervisor.restart_child(failed_child)

class OneForAllStrategy(SupervisionStrategy):
    def handle_failure(self, supervisor, failed_child, error):
        # Reinicia TODOS los children
        for child in supervisor.children.values():
            supervisor.restart_child(child)

class RestForOneStrategy(SupervisionStrategy):
    def handle_failure(self, supervisor, failed_child, error):
        # Reinicia el fallido y los que vienen después
        start_restarting = False
        for child in supervisor.children.values():
            if child == failed_child:
                start_restarting = True
            if start_restarting:
                supervisor.restart_child(child)
```

### 5. Integration con ActorScheduler

El scheduler existente (TASK-041) necesita extenderse:

```python
class ActorScheduler:
    # ... código existente ...
    
    def register_supervisor(self, supervisor: SupervisorActor):
        """Registra supervisor para monitoreo."""
        self.supervisors[supervisor.ref] = supervisor
    
    def notify_actor_failure(self, actor_ref: ActorRef, error: Exception):
        """Notifica fallo a supervisor correspondiente."""
        supervisor = self.find_supervisor_for(actor_ref)
        if supervisor:
            supervisor.handle_child_failure(actor_ref, error)
        else:
            # Sin supervisor → log y detener actor
            self.stop_actor(actor_ref)
```

### 6. Directive System (Decisiones de Supervisión)

Inspirado en Akka Directives:

```python
class SupervisorDirective(Enum):
    """Decisiones que puede tomar un supervisor."""
    
    RESUME = "resume"      # Ignorar error y continuar
    RESTART = "restart"    # Reiniciar actor
    STOP = "stop"          # Detener actor permanentemente
    ESCALATE = "escalate"  # Escalar al supervisor padre
```

**Ejemplo de uso**:

```python
class CustomSupervisor(SupervisorActor):
    def decide_on_failure(self, child_ref: ActorRef, error: Exception) -> SupervisorDirective:
        """Custom logic para decidir acción."""
        if isinstance(error, TransientError):
            return SupervisorDirective.RESTART
        elif isinstance(error, FatalError):
            return SupervisorDirective.ESCALATE
        else:
            return SupervisorDirective.RESUME
```

## Consecuencias

### Positivas

1. ✅ **Tolerancia a fallos**: Sistema resiliente que se auto-recupera
2. ✅ **Aislamiento de errores**: Fallos no afectan al sistema completo
3. ✅ **Flexibilidad**: 3 estrategias para diferentes escenarios
4. ✅ **Configurabilidad**: Políticas de reinicio ajustables
5. ✅ **Inspiración probada**: Erlang/OTP y Akka son referencias sólidas
6. ✅ **Let it crash philosophy**: No defensive programming innecesario
7. ✅ **Escalación controlada**: Errores graves llegan al top-level supervisor

### Negativas

1. ⚠️ **Complejidad adicional**: Más conceptos para aprender
2. ⚠️ **Overhead de supervisión**: Monitoreo continuo consume recursos
3. ⚠️ **Riesgo de restart loops**: Mal configurado puede causar ciclos infinitos
4. ⚠️ **Debugging más difícil**: Reinicios ocultan stack traces originales
5. ⚠️ **Serialización de estado**: Reiniciar actor requiere preservar estado crítico

### Mitigaciones

- **Complejidad**: Proveer defaults sensatos (OneForOne con 3 retries)
- **Overhead**: Supervisión solo cuando es necesario (opt-in)
- **Restart loops**: Circuit breaker automático después de max_retries
- **Debugging**: Log detallado de cada reinicio con stack trace
- **Estado**: Hooks `pre_restart()` / `post_restart()` para serialización

## Alternativas Consideradas

### Alternativa 1: Sin Supervisión (Status Quo)
**Descripción**: Dejar que los actors fallen y terminen.

**Rechazada porque**:
- ❌ No hay tolerancia a fallos
- ❌ Sistema frágil ante errores transitorios
- ❌ No cumple el principio "let it crash" de Erlang

### Alternativa 2: Global Error Handler
**Descripción**: Un único handler global para todos los errores.

**Rechazada porque**:
- ❌ No permite estrategias específicas por tipo de actor
- ❌ Acoplamiento alto (single point of failure)
- ❌ No respeta jerarquía de actors

### Alternativa 3: Solo OneForOne Strategy
**Descripción**: Implementar solo la estrategia más simple.

**Rechazada porque**:
- ❌ Insuficiente para escenarios complejos
- ❌ OneForAll y RestForOne son necesarios para ciertos casos
- ❌ Limitaría la expresividad del sistema

### Alternativa 4: Supervisión Externa (Kubernetes-style)
**Descripción**: Reiniciar procesos desde fuera del runtime.

**Rechazada porque**:
- ❌ Demasiado heavyweight para actors in-process
- ❌ Latencia alta de reinicio
- ❌ No aprovecha el modelo de actors

## Referencias

### Papers y Especificaciones
- **Erlang/OTP Supervision Principles**: [erlang.org/doc/design_principles/sup_princ.html](http://erlang.org/doc/design_principles/sup_princ.html)
- **Akka Supervision**: [doc.akka.io/docs/akka/current/typed/fault-tolerance.html](https://doc.akka.io/docs/akka/current/typed/fault-tolerance.html)
- **"Let it Crash" Philosophy**: [learnyousomeerlang.com/supervisors](http://learnyousomeerlang.com/supervisors)

### Implementaciones de Referencia
- **Erlang Supervisor**: `supervisor.erl` en OTP
- **Akka Typed Supervision**: `akka.actor.typed.SupervisorStrategy`
- **Elixir Supervisor**: `lib/elixir/lib/supervisor.ex`

### Jira
- **Epic**: EPIC-04 (Concurrency - Actors)
- **User Story**: US-09 (Supervision hierarchy para manejo de errores)
- **Sprint**: 17

### Código Relacionado
- **Sprint 16**: Actor System base (VELA-578)
  - `src/concurrency/actor.py` - Clase Actor
  - `src/concurrency/scheduler.py` - ActorScheduler
  - `src/concurrency/message_loop.py` - Message processing

## Implementación

**Archivos a crear** (TASK-042, TASK-043):

1. `src/concurrency/supervision.py`:
   - `SupervisorActor` (base class)
   - `SupervisionStrategy` (interface)
   - `OneForOneStrategy`
   - `OneForAllStrategy`
   - `RestForOneStrategy`
   - `RestartPolicy`
   - `BackoffStrategy` (enum)
   - `SupervisorDirective` (enum)

2. `tests/unit/concurrency/test_supervision.py`:
   - Tests de cada strategy
   - Tests de restart policies
   - Tests de escalación
   - Tests de edge cases (restart loops)

3. Modificaciones a archivos existentes:
   - `src/concurrency/actor.py`: Agregar hooks `pre_restart()`, `post_restart()`
   - `src/concurrency/scheduler.py`: Integrar `notify_actor_failure()`

**Estimación**:
- TASK-042: 40 horas (implementation de strategies)
- TASK-043: 32 horas (restart logic + backoff)
- TASK-044: 48 horas (tests exhaustivos)
- **Total Sprint 17**: ~120 horas

## Notas de Diseño

### 1. Preguntas Frecuentes

**P: ¿Qué pasa si un supervisor falla?**  
R: El supervisor también tiene un supervisor padre que lo reinicia (supervisors all the way up).

**P: ¿Cómo evitar restart loops infinitos?**  
R: `RestartPolicy` con `max_retries` y `within_time_window`. Si se excede, escalación obligatoria.

**P: ¿Se preserva el estado del actor al reiniciar?**  
R: Depende del actor. Hooks `pre_restart()` permiten serializar estado crítico.

**P: ¿Puedo cambiar strategy en runtime?**  
R: No en v1.0. Strategy se define al crear el supervisor y es inmutable.

### 2. Orden de Implementación

1. **TASK-042**: Estructuras base + OneForOne (más simple)
2. **TASK-043**: Restart policies + backoff strategies
3. **TASK-043**: OneForAll y RestForOne (más complejas)
4. **TASK-043**: Integration con ActorScheduler
5. **TASK-044**: Tests exhaustivos de todos los casos

### 3. Testing Strategy

- ✅ Unit tests: Cada strategy aislada
- ✅ Integration tests: Supervisors + Scheduler
- ✅ Fault injection: Simular errores controlados
- ✅ Load tests: Múltiples reinicios simultáneos
- ✅ Edge cases: Restart loops, escalación en cadena

---

**Próximos Pasos**:
1. Implementar `SupervisorActor` base class
2. Implementar `OneForOneStrategy` (más simple)
3. Implementar `RestartPolicy` con backoff
4. Tests de OneForOne
5. Implementar OneForAll y RestForOne
6. Integration tests con ActorScheduler
