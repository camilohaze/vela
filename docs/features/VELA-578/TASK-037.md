# TASK-037: Implementar Actor Instances

## 📋 Información General
- **Historia:** VELA-578 (Actor System)
- **Sprint:** Sprint 16
- **Estado:** Completada ✅
- **Fecha:** 2025-12-02
- **Estimación:** 48 horas
- **Prioridad:** P0
- **Depende de:** TASK-036

## 🎯 Objetivo

Implementar la clase base `Actor` con:
- Estado privado encapsulado
- Message handlers (método `receive()`)
- Lifecycle hooks (pre_start, post_stop, pre_restart, post_restart)
- ActorRef para location transparency
- Función `spawn()` para crear actores

## 🔨 Implementación

### 1. Actor Base Class (ABC)

```python
class Actor(ABC):
    """Clase base abstracta para todos los actores."""
    
    def __init__(self):
        self._actor_state = ActorState.UNINITIALIZED
        self._actor_ref: Optional[ActorRef] = None
        self._message_count = 0
        self._error_count = 0
    
    @abstractmethod
    def receive(self, message: Any) -> None:
        """DEBE ser implementado por subclases."""
        pass
```

**Características:**
- **Abstracta**: No puede instanciarse directamente
- **Estado privado**: `_actor_state`, `_actor_ref`, contadores
- **Método obligatorio**: `receive()` debe implementarse

### 2. Lifecycle Hooks

```python
def pre_start(self) -> None:
    """Llamado antes de procesar mensajes."""
    pass

def post_stop(self) -> None:
    """Llamado al detener el actor."""
    pass

def pre_restart(self, error: Exception) -> None:
    """Llamado antes de reiniciar por error."""
    pass

def post_restart(self, error: Exception) -> None:
    """Llamado después de reiniciar."""
    pass
```

**Propósito:**
- **pre_start**: Inicializar recursos (DB, files, subscriptions)
- **post_stop**: Cleanup de recursos
- **pre_restart**: Log de error, cleanup parcial
- **post_restart**: Reinicializar estado

### 3. ActorRef (Location-Transparent Reference)

```python
class ActorRef:
    """Referencia a un actor (proxy)."""
    
    def send(self, message: Any) -> None:
        """Enviar mensaje (asíncrono, fire-and-forget)."""
        if self._stopped:
            raise RuntimeError(f"Actor {self._name} is stopped")
        
        self._actor.receive(message)
        self._actor._increment_message_count()
    
    def stop(self) -> None:
        """Detener el actor."""
        self._actor._set_state(ActorState.STOPPING)
        self._actor.post_stop()
        self._actor._set_state(ActorState.STOPPED)
        self._stopped = True
```

**Garantías:**
- **Location transparency**: Mismo API para local y remoto
- **Type-safe**: Puede tipificarse `ActorRef[CounterActor]`
- **Hashable**: Puede usarse en sets/dicts
- **Equality**: Dos refs con mismo nombre son iguales

### 4. Spawn Function

```python
def spawn(actor_class: type, name: Optional[str] = None, **kwargs) -> ActorRef:
    """Crear instancia de un actor."""
    
    # Generar nombre si no se provee
    if name is None:
        name = f"{actor_class.__name__}-{_actor_counter}"
    
    # Crear instancia
    actor_instance = actor_class(**kwargs)
    
    # Crear ActorRef
    actor_ref = ActorRef(name, actor_instance)
    
    # Lifecycle: pre_start
    actor_instance._set_state(ActorState.STARTING)
    actor_instance.pre_start()
    actor_instance._set_state(ActorState.RUNNING)
    
    return actor_ref
```

**Características:**
- Auto-generación de nombres únicos
- Validación de tipo (debe heredar de `Actor`)
- Llamada a `pre_start()` lifecycle hook
- Estado inicial: `RUNNING`

### 5. ActorState Enum

```python
class ActorState(Enum):
    UNINITIALIZED = "uninitialized"  # Actor creado pero no iniciado
    STARTING = "starting"             # En proceso de inicialización
    RUNNING = "running"               # Activo y procesando mensajes
    STOPPING = "stopping"             # En proceso de detención
    STOPPED = "stopped"               # Detenido completamente
    RESTARTING = "restarting"         # En proceso de reinicio
```

### Archivos Generados

1. **src/concurrency/actor.py** (500+ LOC)
   - Clase base `Actor`
   - `ActorRef` implementation
   - `spawn()` function
   - `ActorState` enum
   - `CounterActor` example

2. **tests/unit/concurrency/test_actor.py** (450+ LOC)
   - 42 tests pasando (100%)
   - Cobertura completa de API

3. **docs/features/VELA-578/TASK-037.md** (este archivo)

## ✅ Criterios de Aceptación

- [x] Clase base `Actor` abstracta implementada
- [x] Método `receive()` obligatorio en subclases
- [x] 4 lifecycle hooks implementados (pre_start, post_stop, pre_restart, post_restart)
- [x] `ActorRef` con send(), stop(), equality, hash
- [x] Función `spawn()` con auto-generación de nombres
- [x] `ActorState` enum con 6 estados
- [x] Métrica de mensajes procesados
- [x] Métrica de errores
- [x] 42 tests pasando (100%)
- [x] CounterActor example completo

## 📊 Métricas

- **Código:** 500+ LOC en actor.py
- **Tests:** 42 tests (100% passing)
- **Clases:** 3 principales (Actor, ActorRef, ActorState)
- **Cobertura:** ~95% (estimado)
- **Ejemplos:** 1 (CounterActor)

## 🔗 Referencias

- **Jira:** [TASK-037](https://velalang.atlassian.net/browse/TASK-037)
- **Historia:** [VELA-578](https://velalang.atlassian.net/browse/VELA-578)
- **Código:** src/concurrency/actor.py
- **Tests:** tests/unit/concurrency/test_actor.py

## 📝 Decisiones de Diseño

### 1. Actor como ABC (Abstract Base Class)

**Decisión:** Usar `abc.ABC` para forzar implementación de `receive()`.

**Razón:**
- Garantiza que todas las subclases implementen message handler
- Error en tiempo de definición (no en runtime)
- Más explícito que convención

**Alternativa rechazada:** Duck typing (confiar en convención)

---

### 2. Estado Privado con `_` Prefix

**Decisión:** Usar `_` prefix para estado interno del actor.

**Razón:**
- Convención Python para "private"
- No accesible desde fuera del actor
- Evita mutación externa

**Alternativa rechazada:** `__` (name mangling) - demasiado restrictivo

---

### 3. Location Transparency desde v1

**Decisión:** `ActorRef` diseñado para soportar actores remotos.

**Razón:**
- API futura-proof
- Mismo código para local y remoto
- Property `path` ya preparado

**Trade-off:** Complejidad extra ahora, pero simplifica Sprint 20+ (distributed)

---

### 4. Send Temporal (Sin Mailbox)

**Decisión:** `send()` llama `receive()` directamente (por ahora).

**Razón:**
- Mailbox se implementa en TASK-038
- Permite testing inmediato
- Se reemplazará en próxima task

**Nota:** Esto es temporal, NO es el diseño final

---

### 5. Lifecycle Hooks Opcionales

**Decisión:** Hooks con implementación vacía (no abstractos).

**Razón:**
- No todos los actores necesitan hooks
- Override solo si se necesita
- Más conveniente para casos simples

**Alternativa rechazada:** Hooks abstractos (demasiado verboso)

---

## 🎨 Ejemplos de Uso

### Ejemplo 1: Actor Simple

```vela
actor SimpleActor {
  state messages: List<String> = []
  
  fn receive(message: Message) -> void {
    match message {
      Text(content) => {
        this.messages.push(content)
      }
    }
  }
}

# Python equivalent
class SimpleActor(Actor):
    def __init__(self):
        super().__init__()
        self._messages = []
    
    def receive(self, message):
        if isinstance(message, dict) and message.get("type") == "Text":
            self._messages.append(message["content"])
```

### Ejemplo 2: Actor con Lifecycle Hooks

```vela
actor DatabaseActor {
  state connection: Connection = None
  
  fn pre_start() -> void {
    # Conectar a DB
    this.connection = Database.connect()
  }
  
  fn post_stop() -> void {
    # Cerrar conexión
    this.connection.close()
  }
  
  fn receive(message: Message) -> void {
    match message {
      Query(sql) => {
        result = this.connection.execute(sql)
        sender.send(Result(result))
      }
    }
  }
}
```

### Ejemplo 3: Actor Reactivo

```vela
actor ReactiveActor {
  state count: Number = 0
  
  # Computed property (reactivo)
  computed doubled: Number {
    return this.count * 2
  }
  
  # Effect (reactivo)
  effect {
    if this.count > 10 {
      print("Count exceeded 10!")
    }
  }
  
  fn receive(message: Message) -> void {
    match message {
      Increment => this.count = this.count + 1
    }
  }
}
```

## 🚀 Impacto en el Lenguaje

**Nuevas Palabras Reservadas:**
- `actor` - Definir actor class
- `spawn` - Crear instancia de actor

**APIs del Sistema:**
```vela
import 'system:actors' show { Actor, spawn, ActorRef }

# Crear actor
ref: ActorRef<Counter> = spawn Counter()

# Enviar mensaje
ref.send(Increment)

# Detener actor
ref.stop()
```

**Integración con State Management:**
- Actores pueden usar `state` para estado reactivo
- `computed` funciona dentro de actores
- `effect` se dispara en cambios de estado

## 🔄 Próximos Pasos

**TASK-038: Mailbox System**
- Implementar bounded/unbounded/priority mailboxes
- Reemplazar `send()` directo por enqueue a mailbox
- Garantías de ordering (FIFO mismo sender)
- Backpressure (bounded mailbox)

**Cambios en Actor:**
```python
# Actual (TASK-037)
def send(self, message):
    self._actor.receive(message)  # Directo

# Futuro (TASK-038)
def send(self, message):
    self._mailbox.enqueue(message)  # Via mailbox
```

---

**Completado:** 2025-12-02  
**Tiempo:** ~6 horas de implementación y testing  
**Próxima Task:** TASK-038 - Mailbox System  
**Tests:** 42/42 pasando (100%)
