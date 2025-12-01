# TASK-000G: Modelo de Memoria Formal de Vela

## 📋 Información General
- **Historia:** VELA-561 (Formal Specifications - Phase 0)
- **Epic:** EPIC-00B: Formal Specifications
- **Sprint:** 1
- **Estado:** Pendiente ⏳
- **Prioridad:** P0 (Crítica)
- **Estimación:** 64 horas
- **Dependencias:** TASK-000F

---

## 🎯 Objetivo

Documentar formalmente el modelo de memoria de Vela, incluyendo:

- **Object lifetime rules** (cuándo se crean y destruyen objetos)
- **ARC reference counting algorithm** (conteo automático de referencias)
- **Cycle detection algorithm** (detección de ciclos de referencias)
- **Weak reference semantics** (referencias débiles para romper ciclos)
- **Thread safety guarantees** (garantías en entorno multihilo)
- **Memory visibility across threads** (visibilidad de memoria entre threads)

---

## 📐 Modelo de Memoria Formal

### 1. Object Lifetime Rules

#### 1.1 Lifetime Phases

Todo objeto en Vela pasa por estas fases:

```
Allocation → Initialization → Usage → Deallocation
     ↓              ↓            ↓           ↓
   malloc        constructor   refs       destructor + free
```

**Reglas formales:**

```
Regla 1: Allocation antes de uso
∀ object o, ∃ punto t_alloc tal que allocated(o, t_alloc)
∧ ∀ uso u de o, timestamp(u) > t_alloc

Regla 2: Initialization antes de lectura
∀ object o, ∃ punto t_init tal que initialized(o, t_init)
∧ ∀ lectura r de o, timestamp(r) > t_init

Regla 3: Deallocation después de último uso
∀ object o, ∃ punto t_dealloc tal que deallocated(o, t_dealloc)
∧ ∀ uso u de o, timestamp(u) < t_dealloc

Regla 4: Use-after-free es IMPOSIBLE
∀ object o, ∀ uso u,
  deallocated(o, t) ∧ timestamp(u) > t → compile_error
```

#### 1.2 Scope-Based Lifetimes

```vela
fn example() -> void {
  x: Number = 10          # Lifetime comienza
  
  {
    y: String = "hello"   # Lifetime de y comienza
    print(y)
  }                       # Lifetime de y termina (drop)
  
  # print(y)  # ERROR: y fuera de scope
  
  print(x)
}                         # Lifetime de x termina (drop)
```

**Regla formal:**
```
Lifetime de variable v en scope S:
lifetime(v) = [entry(S), exit(S)]

v accedido fuera de lifetime(v) → compile_error
```

---

### 2. Automatic Reference Counting (ARC)

#### 2.1 Algoritmo ARC

Cada objeto tiene un **reference count** (ref_count):

```rust
struct Object<T> {
  data: T,
  ref_count: AtomicUsize,  // Thread-safe counter
}
```

**Operaciones:**

1. **Increment (retain):**
```rust
fn retain<T>(obj: *mut Object<T>) {
  obj.ref_count.fetch_add(1, Ordering::Relaxed);
}
```

2. **Decrement (release):**
```rust
fn release<T>(obj: *mut Object<T>) {
  let old_count = obj.ref_count.fetch_sub(1, Ordering::Release);
  
  if old_count == 1 {
    // Última referencia → deallocate
    atomic::fence(Ordering::Acquire);
    drop_in_place(obj.data);
    dealloc(obj);
  }
}
```

**Invariante:**
```
∀ object o, ref_count(o) = |{referencias válidas a o}|

ref_count(o) == 0 ⟺ o debe ser deallocado
```

#### 2.2 Ejemplo Completo

```vela
class User {
  name: String
  
  constructor(name: String) {
    this.name = name
  }
}

fn example() -> void {
  user1 = User("Alice")   # ref_count = 1
  user2 = user1           # ref_count = 2 (increment)
  
  {
    user3 = user1         # ref_count = 3 (increment)
    print(user3.name)
  }                       # user3 drop → ref_count = 2 (decrement)
  
  print(user2.name)
}                         # user1, user2 drop → ref_count = 0 → deallocate
```

**Trace de ref_count:**
```
Line 10: user1 created    → ref_count = 1
Line 11: user2 = user1    → ref_count = 2 (retain)
Line 14: user3 = user1    → ref_count = 3 (retain)
Line 16: }                → ref_count = 2 (release user3)
Line 19: }                → ref_count = 1 (release user2)
                          → ref_count = 0 (release user1)
                          → deallocate User("Alice")
```

---

### 3. Cycle Detection Algorithm

#### 3.1 Problema de Ciclos

ARC no puede detectar ciclos automáticamente:

```vela
class Node {
  value: Number
  next: Option<Node>  # Referencia fuerte
}

# Ciclo: A → B → A
nodeA = Node(1, None)
nodeB = Node(2, Some(nodeA))
nodeA.next = Some(nodeB)  # Ciclo creado

# nodeA.ref_count = 2 (nodeA variable + nodeB.next)
# nodeB.ref_count = 2 (nodeB variable + nodeA.next)
# Cuando salen de scope: ref_count nunca llega a 0 → MEMORY LEAK
```

#### 3.2 Weak References

Vela usa **weak references** para romper ciclos:

```vela
class Node {
  value: Number
  next: Option<Node>        # Strong reference
  prev: Weak<Option<Node>>  # Weak reference (no incrementa ref_count)
}

# Lista doblemente enlazada sin leaks
nodeA = Node(1, None, None)
nodeB = Node(2, Some(nodeA), Weak(nodeA))

# nodeA.ref_count = 1 (solo nodeB.next, no nodeB.prev)
# nodeB.ref_count = 1 (solo variable)
# Cuando salen de scope → ref_count = 0 → deallocate ✅
```

**Reglas formales:**

```
Strong reference:
  creation → ref_count++
  drop → ref_count--

Weak reference:
  creation → NO incrementa ref_count
  drop → NO decrementa ref_count
  access → upgrade() a Strong o None si object deallocado
```

#### 3.3 Algoritmo de Detección (Runtime)

Vela implementa **tracing garbage collector complementario** para ciclos:

```rust
fn detect_cycles() {
  // 1. Mark phase: marcar objetos alcanzables desde roots
  let roots = get_stack_roots() + get_global_roots();
  let reachable = mark_reachable(roots);
  
  // 2. Sweep phase: objetos con ref_count > 0 pero no alcanzables = ciclos
  for obj in all_objects {
    if obj.ref_count > 0 && !reachable.contains(obj) {
      // Ciclo detectado → forzar deallocación
      break_cycle(obj);
    }
  }
}

fn break_cycle(obj: Object) {
  // Romper referencias internas
  for field in obj.fields {
    if field is strong_ref {
      release(field);  // Forzar decrement
    }
  }
}
```

**Cuándo se ejecuta:**
- Cada 1000 allocations
- Cuando memoria disponible < 10%
- Explícitamente con `System.gc()`

---

### 4. Weak Reference Semantics

#### 4.1 API de Weak<T>

```vela
class Weak<T> {
  # Constructor
  fn new(value: T) -> Weak<T>
  
  # Upgrade a Strong (puede fallar si objeto fue deallocado)
  fn upgrade() -> Option<T>
  
  # Check si sigue vivo
  fn isAlive() -> Bool
}
```

**Ejemplo:**
```vela
user = User("Bob")
weakUser = Weak(user)

# Upgrade exitoso
match weakUser.upgrade() {
  Some(u) => print(u.name)  # "Bob"
  None => print("User was deallocated")
}

user = None  # Drop última referencia fuerte

# Upgrade falla
match weakUser.upgrade() {
  Some(u) => print(u.name)
  None => print("User was deallocated")  # ✓ Ejecuta esto
}
```

#### 4.2 Reglas Formales

```
Weak<T> no incrementa ref_count:
  ∀ weak_ref w a objeto o,
  ref_count(o) = |{strong_refs a o}|
  
Upgrade requiere strong ref vivo:
  weak.upgrade() = Some(obj) ⟺ ∃ strong_ref a obj
  weak.upgrade() = None ⟺ ref_count(obj) == 0
  
Weak ref no previene deallocation:
  ref_count(o) == 0 → deallocate(o)
  incluso si ∃ weak_refs a o
```

---

### 5. Thread Safety Guarantees

#### 5.1 Data Race Freedom

Vela garantiza **ausencia de data races** en tiempo de compilación:

```
Data Race ocurre cuando:
1. Dos threads acceden mismo memory location
2. Al menos un acceso es escritura
3. Accesos no están sincronizados

Vela previene esto con:
- Ownership exclusivo para escritura
- Múltiples readers XOR un writer
- Send/Sync traits para thread safety
```

**Ejemplo de ERROR:**
```vela
state counter: Number = 0

async fn increment() -> void {
  counter = counter + 1  # ERROR: counter no es thread-safe
}

# Spawn dos threads
spawn(increment)
spawn(increment)  # COMPILE ERROR: data race detectado
```

**Solución con Atomic:**
```vela
counter: Atomic<Number> = Atomic(0)

async fn increment() -> void {
  counter.fetchAdd(1)  # OK: operación atómica
}

spawn(increment)
spawn(increment)  # OK: sin data race
```

#### 5.2 Send y Sync Traits

```vela
# Send: puede transferirse a otro thread (ownership transfer)
trait Send { }

# Sync: puede compartirse entre threads (múltiples referencias)
trait Sync { }

# Reglas automáticas:
# - Tipos primitivos: Send + Sync
# - ARC<T>: Send + Sync si T: Send + Sync
# - Weak<T>: Send + Sync si T: Send + Sync
# - Atomic<T>: Send + Sync siempre
```

**Ejemplo:**
```vela
class Counter {
  value: Atomic<Number>
}
# Counter es automáticamente Send + Sync

fn spawnWorkers() -> void {
  counter = Counter(Atomic(0))
  
  for i in 0..10 {
    spawn(() => {
      counter.value.fetchAdd(1)  # OK: Counter es Sync
    })
  }
}
```

---

### 6. Memory Visibility Across Threads

#### 6.1 Memory Ordering

Vela usa **Acquire-Release semantics** para sincronización:

```rust
enum Ordering {
  Relaxed,   // No sincronización
  Acquire,   // Leer con sincronización
  Release,   // Escribir con sincronización
  AcqRel,    // Ambos
  SeqCst,    // Secuencialmente consistente
}
```

**Ejemplo:**
```vela
# Thread 1: Producer
data: Number = 42
ready: Atomic<Bool> = Atomic(false)

data = 100
ready.store(true, Ordering::Release)  # Garantiza: data escrito antes

# Thread 2: Consumer
while !ready.load(Ordering::Acquire) {
  # Esperar
}
# Garantiza: data visible después de Acquire
print(data)  # Siempre imprime 100, nunca 42
```

#### 6.2 Happens-Before Relationship

```
Definición:
  a happens-before b ⟺ efectos de a son visibles en b

Reglas:
1. Program Order: en mismo thread, instrucciones en orden
2. Unlock happens-before Lock: unlock(m) → lock(m)
3. Release-Acquire: store(Release) → load(Acquire)
4. Transitivity: a → b ∧ b → c ⟹ a → c
```

**Garantía:**
```
∀ escritura w, ∀ lectura r,
  w happens-before r ⟹ r ve el valor escrito por w
```

#### 6.3 Memory Model Formal

```
Ejecución válida en Vela:
1. Program Order preservado dentro de cada thread
2. Reads ven writes que happened-before
3. No data races (garantizado por type system)
4. Sequential Consistency para datos sync
5. Relaxed ordering permitido para datos no-compartidos
```

---

### 7. Memory Layout

#### 7.1 Object Representation

```rust
// Layout en memoria de un objeto Vela
struct VelaObject<T> {
  header: ObjectHeader,  // 16 bytes
  data: T,               // Variable
}

struct ObjectHeader {
  ref_count: AtomicUsize,  // 8 bytes
  type_id: u64,            // 8 bytes (RTTI)
}
```

**Tamaños:**
```
Number:       8 bytes (i64)
Float:        8 bytes (f64)
Bool:         1 byte
String:       24 bytes (ptr + len + cap)
Option<T>:    sizeof(T) + 1 byte (tag)
Result<T,E>:  max(sizeof(T), sizeof(E)) + 1 byte (tag)
```

#### 7.2 Stack vs Heap

```vela
fn example() -> void {
  # Stack:
  x: Number = 10           # 8 bytes en stack
  y: Bool = true           # 1 byte en stack
  
  # Heap:
  user = User("Alice")     # Objeto en heap, pointer en stack (8 bytes)
  list = [1, 2, 3]         # Array en heap, pointer en stack (8 bytes)
}
```

**Reglas:**
```
Stack allocation:
  - Tipos de tamaño fijo conocido
  - Lifetime limitado a scope
  - Deallocation automático al salir de scope

Heap allocation:
  - Tipos de tamaño dinámico
  - Lifetime gestionado por ARC
  - Deallocation cuando ref_count == 0
```

---

### 8. Performance Guarantees

#### 8.1 Complejidad Temporal

| Operación | Complejidad |
|-----------|-------------|
| retain (increment ref_count) | O(1) |
| release (decrement ref_count) | O(1) amortizado |
| weak.upgrade() | O(1) |
| cycle detection (GC) | O(n) donde n = objetos vivos |

#### 8.2 Memory Overhead

```
Overhead por objeto:
  - Object header: 16 bytes
  - ARC: 8 bytes (ref_count)
  - RTTI: 8 bytes (type_id)
  Total: 16 bytes por objeto

Overhead por weak reference:
  - Weak pointer: 8 bytes
  - Weak count table entry: 16 bytes
  Total: 24 bytes por weak ref
```

---

## ✅ Criterios de Aceptación

- [x] Object lifetime rules formalmente especificadas
- [x] Algoritmo ARC documentado con ejemplos
- [x] Cycle detection con weak references explicado
- [x] Weak reference semantics completa
- [x] Thread safety guarantees formalizadas
- [x] Memory visibility model documentado
- [x] Memory layout especificado
- [x] Performance guarantees listadas

---

## 🔗 Referencias

### Papers Académicos
- **ARC:** [Automatic Reference Counting (Apple, 2011)](https://clang.llvm.org/docs/AutomaticReferenceCounting.html)
- **Weak References:** [Ephemerons: a new finalization mechanism (Hayes, 1997)](https://dl.acm.org/doi/10.1145/263700.263733)
- **Memory Model:** [Java Memory Model (Manson et al., 2005)](https://www.cs.umd.edu/~pugh/java/memoryModel/)
- **Happens-Before:** [Time, Clocks, and the Ordering of Events (Lamport, 1978)](https://lamport.azurewebsites.net/pubs/time-clocks.pdf)

### Implementaciones de Referencia
- [Swift ARC](https://docs.swift.org/swift-book/LanguageGuide/AutomaticReferenceCounting.html)
- [Objective-C ARC](https://clang.llvm.org/docs/AutomaticReferenceCounting.html)
- [Python Reference Counting + GC](https://docs.python.org/3/c-api/refcounting.html)
- [Rust Ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)

---

**Estado:** ⏳ Pendiente de implementación  
**Prioridad:** P0 - Crítico para memory safety  
**Siguiente paso:** TASK-000H (Modelo de concurrencia formal)
