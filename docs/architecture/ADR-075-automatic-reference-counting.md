# ADR-075: Automatic Reference Counting (ARC) para VelaVM

## Estado
✅ Aceptado

## Fecha
2025-12-07

## Contexto

VelaVM actualmente usa **Mark-and-Sweep GC** (TASK-072) para memory management:
- ✅ **Ventajas**: Simple, correcto, maneja ciclos
- ❌ **Desventajas**: Pausas largas (STW), overhead en CPU, memoria no liberada inmediatamente

Para Vela 1.0 necesitamos:
1. **Memory management determinístico**: Liberación inmediata cuando no hay referencias
2. **Menor overhead en runtime**: Sin pausas STW (Stop-The-World)
3. **Mejor para sistemas embebidos**: Predicibilidad en uso de memoria
4. **Compatible con sistemas reactivos**: Signals y computed con lifetime claro

### Problema a Resolver

¿Cómo mejorar el memory management de VelaVM para tener liberación determinística sin sacrificar correctitud?

### Opciones Consideradas

#### Opción 1: **Tracing GC puro** (Status Quo - Mark-and-Sweep)
- ✅ Maneja ciclos automáticamente
- ✅ Simple de implementar
- ❌ Pausas STW
- ❌ Overhead constante de tracing
- ❌ Memoria no liberada inmediatamente

**Ejemplos**: Java, Python, Ruby

#### Opción 2: **Reference Counting puro**
- ✅ Liberación inmediata
- ✅ Sin pausas STW
- ❌ **NO maneja ciclos de referencias** (memory leak crítico)
- ❌ Overhead en cada asignación (refcount++)
- ❌ Cache unfriendly (contadores distribuidos)

**Ejemplos**: Python (CPython usa RC + cyclic GC), Swift (ARC + weak refs)

#### Opción 3: **ARC + Cycle Detection (Híbrido)** ✅ ELEGIDO
- ✅ Liberación inmediata (determinística)
- ✅ Sin pausas STW en operaciones normales
- ✅ Maneja ciclos con detector separado (tracing ocasional)
- ✅ Mejor performance para código sin ciclos (99% de casos)
- ⚠️ Overhead en cada retain/release (pero optimizable con compiler)
- ⚠️ Necesita weak references para evitar ciclos

**Ejemplos**: Swift (ARC + weak/unowned), Objective-C (ARC), Rust (Rc<T> + Weak<T>)

#### Opción 4: **Ownership + Borrow Checker** (Rust-style)
- ✅ Zero-cost en runtime
- ✅ Sin GC
- ❌ **Muy complejo para usuarios** (curva de aprendizaje alta)
- ❌ **Incompatible con paradigma funcional/reactivo** de Vela
- ❌ Requiere reescribir type system completo

**Razón de rechazo**: Vela es un lenguaje **high-level, funcional y reactivo**. Ownership explícito es demasiado complejo para el target audience.

---

## Decisión

Implementamos **ARC (Automatic Reference Counting) + Cycle Detection**:

### Arquitectura

```
┌─────────────────────────────────────────────────────────┐
│                    VelaVM Memory Management              │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │              Heap Allocator (TASK-072)            │  │
│  │  - internString()                                 │  │
│  │  - allocateClosure()                              │  │
│  │  - allocateInstance()                             │  │
│  └──────────────────────────────────────────────────┘  │
│                        │                                 │
│                        ▼                                 │
│  ┌──────────────────────────────────────────────────┐  │
│  │          ARC Manager (TASK-075) ✅ NEW           │  │
│  │  - retain(ref)        // refcount++              │  │
│  │  - release(ref)       // refcount--              │  │
│  │  - autorelease(ref)   // defer release           │  │
│  │  - freeIfZero(ref)    // liberar si refcount=0   │  │
│  └──────────────────────────────────────────────────┘  │
│                        │                                 │
│                        ▼                                 │
│  ┌──────────────────────────────────────────────────┐  │
│  │      Cycle Detector (TASK-076) ✅ FUTURE         │  │
│  │  - detectCycles()     // Tracing para ciclos     │  │
│  │  - breakCycles()      // Weak refs + clear       │  │
│  │  Ejecutado:                                       │  │
│  │    - Periódicamente (threshold)                   │  │
│  │    - Al forzar GC                                 │  │
│  └──────────────────────────────────────────────────┘  │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

### Componentes

#### 1. HeapObjectRef con RefCount

```vela
public class HeapObjectRef {
  public object: HeapObject
  public metadata: GCMetadata
  public refCount: Number = 1      # ✅ NEW: Reference count (empieza en 1)
  public isWeak: Bool = false      # ✅ NEW: Weak reference flag
}
```

#### 2. ARCManager

```vela
public class ARCManager {
  heap: VelaHeap
  
  # Retain: Incrementar refcount
  fn retain(ref: HeapObjectRef) -> void {
    if !ref.isWeak {
      ref.refCount = ref.refCount + 1
    }
  }
  
  # Release: Decrementar refcount y liberar si es 0
  fn release(ref: HeapObjectRef) -> void {
    if ref.isWeak {
      return  # Weak refs no afectan refcount
    }
    
    ref.refCount = ref.refCount - 1
    
    if ref.refCount == 0 {
      this.free(ref)
    }
  }
  
  # Free: Liberar objeto y sus hijos recursivamente
  fn free(ref: HeapObjectRef) -> void {
    # 1. Release children (closures, upvalues, fields)
    match ref.object {
      HeapObject.Closure(c) => {
        # Release upvalues
        c.upvalues.forEach(uv => this.release(uv.ref))
      }
      HeapObject.Instance(inst) => {
        # Release fields
        inst.fields.values().forEach(val => {
          match val {
            Value.HeapObject(r) => this.release(r)
            _ => {}
          }
        })
      }
      # ... otros tipos
    }
    
    # 2. Liberar memoria
    this.heap.deallocate(ref)
  }
}
```

#### 3. Integración con VM Opcodes

**Operaciones que incrementan refcount (retain)**:
- `OP_LOAD_CONST`: Cargar función/closure → retain
- `OP_DUP`: Duplicar ref en stack → retain
- `OP_STORE_LOCAL`: Guardar ref en local → retain
- `OP_STORE_GLOBAL`: Guardar ref en global → retain
- `OP_STORE_UPVALUE`: Guardar ref en upvalue → retain
- `OP_SET_ATTR`: Asignar ref a field → retain
- `OP_BUILD_LIST`: Agregar ref a lista → retain

**Operaciones que decrementan refcount (release)**:
- `OP_POP`: Eliminar ref del stack → release
- `OP_RETURN`: Salir de frame → release locals
- `OP_CLOSE_UPVALUE`: Cerrar upvalue → release stack ref
- Variable local sale de scope → release

**Ejemplo de código VM:**

```vela
# OP_STORE_LOCAL
fn opStoreLocal(index: Number, value: Value) -> void {
  # Release old value
  match this.locals[index] {
    Some(Value.HeapObject(oldRef)) => this.arc.release(oldRef)
    _ => {}
  }
  
  # Retain new value
  match value {
    Value.HeapObject(ref) => this.arc.retain(ref)
    _ => {}
  }
  
  this.locals[index] = Some(value)
}

# OP_POP
fn opPop() -> Result<void> {
  match this.stack.pop() {
    Some(Value.HeapObject(ref)) => this.arc.release(ref)
    _ => {}
  }
  return Ok(void)
}
```

#### 4. Weak References (TASK-076)

Para **evitar ciclos**, usamos weak references:

```vela
# Ejemplo de ciclo:
class Node {
  parent: Option<Node> = None  # ❌ Strong ref → CYCLE
  children: List<Node> = []
}

# Solución con weak refs:
class Node {
  parent: Option<Weak<Node>> = None  # ✅ Weak ref → NO CYCLE
  children: List<Node> = []
}
```

**Weak<T>**: Referencia que NO incrementa refcount.
- No previene liberación del objeto
- Se convierte en `None` automáticamente cuando objeto es liberado
- Se usa para parent refs, delegates, observers

---

## Consecuencias

### Positivas

1. **✅ Liberación determinística**: Objetos liberados inmediatamente cuando refcount=0
   - Mejor para sistemas embebidos
   - Predictibilidad en memoria

2. **✅ Sin pausas STW**: ARC no detiene el mundo
   - Mejor latencia
   - Mejor para real-time systems

3. **✅ Menor overhead para código sin ciclos**: 99% de código no tiene ciclos
   - Performance superior a Mark-and-Sweep

4. **✅ Memoria liberada inmediatamente**: No esperar a GC run
   - Menor memoria peak
   - Mejor para dispositivos con poca RAM

5. **✅ Compatible con sistemas reactivos**:
   - Signals con refcount → auto-cleanup cuando no hay subscribers
   - Computed values → liberados cuando no se usan

6. **✅ Migración gradual**: Podemos mantener Mark-and-Sweep como fallback

### Negativas

1. **❌ Overhead en cada asignación**: `refcount++` y `refcount--` en cada operación
   - **Mitigación**: Compiler optimizations (eliminar retain/release redundantes)
   - **Mitigación**: Batch operations en loops

2. **❌ No maneja ciclos automáticamente**: Requiere weak refs explícitas
   - **Mitigación**: Cycle detector (TASK-076) ejecutado periódicamente
   - **Mitigación**: Linter warnings para posibles ciclos

3. **❌ Cache unfriendly**: Contadores dispersos en memoria
   - **Mitigación**: Alinear refcount con metadata (locality)
   - **Mitigación**: Usar atomic operations eficientes (x86 LOCK prefix)

4. **❌ Thread-safety complejo**: Refcount++ debe ser atómico en multithreading
   - **Mitigación**: Atomic operations (AtomicNumber)
   - **Mitigación**: Per-thread arenas para evitar contención

### Trade-offs Aceptados

| Aspecto | Mark-and-Sweep (Antes) | ARC (Después) |
|---------|------------------------|---------------|
| **Liberación** | Lazy (GC run) | Eager (inmediata) |
| **Pausas** | STW (10-100ms) | Sin pausas |
| **Ciclos** | Automático | Manual (weak refs) |
| **Overhead** | Constante (tracing) | Por operación (refcount) |
| **Memoria peak** | Alta (acumula basura) | Baja (libera inmediato) |
| **Predicibilidad** | Baja (GC impredecible) | Alta (determinístico) |

---

## Alternativas Consideradas y Rechazadas

### Alternativa 1: Tracing GC mejorado (Generational GC)
- **Descripción**: Dividir heap en generaciones (young/old), solo tracing en young gen
- **Ventajas**: Menor overhead que full GC, más eficiente
- **Razón de rechazo**: Aún tiene pausas STW, no es determinístico

### Alternativa 2: Concurrent GC (como Go)
- **Descripción**: GC corre en paralelo con mutator threads
- **Ventajas**: Menor pausa (solo stop-the-world para sync points)
- **Razón de rechazo**: Muy complejo, requiere write barriers, overhead constante

### Alternativa 3: Escape Analysis + Stack Allocation
- **Descripción**: Alocar objetos en stack si no escapan función
- **Ventajas**: Zero cost para objetos locales
- **Razón de rechazo**: NO reemplaza GC, solo optimización complementaria (futuro)

### Alternativa 4: Manual Memory Management (C/C++ style)
- **Descripción**: malloc/free explícito por usuario
- **Razón de rechazo**: Vela es high-level language, no compatible con filosofía

---

## Implementación

### Fase 1: ARC Básico (TASK-075) ✅ Este Sprint

**Archivos a crear/modificar:**

1. **vm/arc.vela** (nuevo):
   - `ARCManager` class
   - `retain()`, `release()`, `autorelease()`
   - `freeIfZero()` recursivo

2. **vm/heap.vela** (modificar):
   - Agregar `refCount: Number = 1` a `HeapObjectRef`
   - Agregar `isWeak: Bool = false`
   - Integrar `ARCManager` en `VelaHeap`

3. **vm/velavm.vela** (modificar):
   - Agregar `retain()` en `OP_STORE_LOCAL`, `OP_DUP`, etc.
   - Agregar `release()` en `OP_POP`, `OP_RETURN`, etc.
   - Agregar `release()` al salir de scope de locals

4. **tests/unit/vm/test_arc.vela** (nuevo):
   - Tests de refcount correctness
   - Tests de liberación inmediata
   - Tests de retain/release balance

**Métricas:**
- Estimación: 64 horas
- Cobertura: >= 85%
- Performance: Overhead < 10% vs Mark-and-Sweep

### Fase 2: Weak References (TASK-076) ✅ Siguiente Sprint

1. **vm/weak.vela** (nuevo):
   - `Weak<T>` type
   - `WeakRef` class
   - `lock()` method (convert weak → strong temporalmente)

2. **Cycle detector** (tracing ocasional):
   - Ejecutar cada N allocations
   - Mark-and-sweep solo para objetos con refcount > 0 no alcanzables

### Fase 3: Integración Reactiva (TASK-077) ✅ Siguiente Sprint

1. **reactive/signals.vela** (modificar):
   - Signals con refcount
   - Auto-cleanup cuando no hay subscribers
   - Computed values con lifecycle ARC

---

## Referencias

### Papers y Recursos

1. **Swift ARC**: [Swift Language Guide - ARC](https://docs.swift.org/swift-book/LanguageGuide/AutomaticReferenceCounting.html)
   - Modelo de ARC + weak/unowned references
   - Inspiration principal para Vela

2. **Objective-C Memory Management**: [Apple - Memory Management Programming Guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/MemoryMgmt/Articles/MemoryMgmt.html)
   - retain/release/autorelease pattern
   - Retain cycles y cómo evitarlos

3. **Python CPython RC**: [Python Memory Management](https://docs.python.org/3/c-api/refcounting.html)
   - Reference counting + cyclic GC
   - Modelo híbrido similar a nuestra propuesta

4. **Rust Rc<T>**: [Rust Std - Rc](https://doc.rust-lang.org/std/rc/struct.Rc.html)
   - Single-threaded reference counting
   - Weak<T> para ciclos

5. **MMTk (Memory Management Toolkit)**: [MMTk Project](https://www.mmtk.io/)
   - Framework para implementar GC algorithms
   - Benchmarks de RC vs Tracing GC

### Documentación Vela

- **TASK-072**: [Heap & GC](../../features/VELA-586/TASK-072.md) - Sistema actual Mark-and-Sweep
- **TASK-071**: [VelaVM Core](../../features/VELA-586/TASK-071.md) - Intérprete VM
- **ADR-069**: [Bytecode ISA](ADR-069-bytecode-instruction-set.md) - Opcodes relacionados

### Benchmarks

Comparación esperada (basado en papers y otras VMs):

| Benchmark | Mark-and-Sweep | ARC | Mejora |
|-----------|---------------|-----|--------|
| Latency (avg) | 5ms | 1ms | 5x |
| Latency (p99) | 100ms (GC pause) | 2ms | 50x |
| Throughput | 100% | 92% | -8% |
| Memory peak | 100% | 60% | 40% less |
| Allocation rate | 1M obj/s | 900K obj/s | -10% |

**Conclusión**: ARC gana en latencia y memoria, pierde ~10% en throughput puro.

---

## Preguntas Frecuentes

### 1. ¿Por qué no usar solo Tracing GC (Mark-and-Sweep)?

**Respuesta**: Tracing GC tiene pausas STW impredecibles. Para Vela (lenguaje reactivo, UI, embebidos), necesitamos **determinismo** en liberación de memoria. ARC da liberación inmediata sin pausas.

### 2. ¿Qué pasa con ciclos de referencias?

**Respuesta**: ARC **no maneja ciclos automáticamente**. Soluciones:
1. **Weak references** (TASK-076): Usar `Weak<T>` para romper ciclos (parent refs, delegates)
2. **Cycle detector** (TASK-076): Tracing ocasional para detectar y romper ciclos

En práctica, 99% del código no tiene ciclos. Los ciclos son patrones conocidos (parent-child, observer) y se resuelven con weak refs.

### 3. ¿Cuál es el overhead de ARC?

**Respuesta**: Cada operación de asignación tiene `refcount++` y `refcount--`:
- **Overhead en CPU**: ~5-10% vs Mark-and-Sweep
- **Overhead en memoria**: 8 bytes por objeto (refcount)

**Mitigación**:
- Compiler optimizations (eliminar retain/release redundantes)
- Batch operations en loops

### 4. ¿ARC es thread-safe?

**Respuesta**: Para Sprint 24, **ARC es single-threaded**. En futuro (TASK-XXX: Multithreading):
- Usar `AtomicNumber` para refcount (atomic increment/decrement)
- Usar per-thread arenas para reducir contención

### 5. ¿Cómo se compara con Rust?

**Respuesta**: Rust usa **ownership + borrow checker** (zero-cost). Vela usa **ARC** (runtime overhead). Trade-off:
- **Rust**: Zero cost, pero complejo (curva de aprendizaje alta)
- **Vela**: Runtime overhead (~5-10%), pero simple (high-level language)

Vela es más similar a **Swift** (ARC + weak refs) que a Rust.

### 6. ¿Puedo forzar GC manualmente?

**Respuesta**: Sí, `gc.collect()` ejecuta:
1. Cycle detector (encuentra ciclos no alcanzables)
2. Libera objetos en ciclos

En operación normal, ARC libera automáticamente (sin `gc.collect()`).

---

## Próximos Pasos

1. **TASK-075** ✅ Este Sprint (Sprint 24):
   - Implementar `ARCManager` con `retain()`, `release()`
   - Integrar en VelaVM opcodes
   - Tests de correctness

2. **TASK-076** ⏭️ Sprint 25:
   - Implementar `Weak<T>` references
   - Cycle detector (tracing ocasional)
   - Tests de memory leaks

3. **TASK-077** ⏭️ Sprint 25:
   - Integrar ARC con reactive system (signals, computed)
   - Auto-cleanup de signals sin subscribers

4. **TASK-078** ⏭️ Sprint 25:
   - Test suite completo de memory management
   - Benchmarks de performance

---

## Decisión Final

**Implementamos ARC (Automatic Reference Counting) + Cycle Detection** para VelaVM.

**Razón**: Balance óptimo entre **determinismo, performance y simplicidad** para Vela 1.0.

---

**Fecha de decisión:** 2025-12-07  
**Autor:** GitHub Copilot Agent  
**Revisores:** Equipo Core Vela  
**Estado:** ✅ Aceptado e implementándose en Sprint 24
