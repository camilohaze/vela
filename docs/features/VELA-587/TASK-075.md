# TASK-075: Implementar ARC (Automatic Reference Counting)

## 📋 Información General
- **Historia:** VELA-587 (US-17: Memory Management Automático)
- **Epic:** EPIC-06: Compiler Backend (VelaVM)
- **Sprint:** 24
- **Estado:** En Curso 🔄
- **Estimación:** 64 horas
- **Fecha de Inicio:** 2025-01-30

---

## 🎯 Objetivo

Implementar **Automatic Reference Counting (ARC)** en VelaVM para reemplazar el garbage collector Mark-and-Sweep con un sistema de memoria management determinístico.

**Problema que resuelve:**
- ❌ Mark-and-Sweep tiene pausas STW (Stop-The-World) de 10-100ms
- ❌ Liberación de memoria no determinística (lazy)
- ❌ Picos de memoria altos (acumulación hasta GC run)

**Solución propuesta:**
- ✅ ARC libera memoria **inmediatamente** cuando refCount llega a 0
- ✅ **Sin pausas STW** (mejor latencia)
- ✅ Uso de memoria **determinístico y predecible**
- ✅ Compatible con sistema reactivo de Vela (signals)

---

## 🏗️ Arquitectura

### Decisión Arquitectónica

**ADR-075:** Se eligió **ARC + Cycle Detection** (híbrido) sobre:
1. ❌ Tracing GC puro (Mark-and-Sweep) - Pausas STW
2. ❌ Reference Counting puro - No maneja cycles
3. ✅ **ARC + Cycle Detection** - Balance óptimo
4. ❌ Ownership + Borrow Checker - Demasiado complejo para lenguaje high-level

Ver detalles en: [`docs/architecture/ADR-075-automatic-reference-counting.md`](../../architecture/ADR-075-automatic-reference-counting.md)

### Componentes Principales

```
┌─────────────────────────────────────────────────────────────────┐
│                          VelaVM                                 │
│                                                                 │
│  ┌─────────────┐        ┌──────────────┐      ┌─────────────┐ │
│  │  Opcodes    │───────►│  ARCManager  │◄─────│  VelaHeap   │ │
│  │             │ retain │              │ free │             │ │
│  │ OP_STORE    │ release│ - retain()   │      │ - allocate()│ │
│  │ OP_POP      │        │ - release()  │      │ - deallocate│ │
│  │ OP_RETURN   │        │ - free()     │      │             │ │
│  └─────────────┘        └──────────────┘      └─────────────┘ │
│                                │                               │
│                                │ recursively free children     │
│                                ▼                               │
│                   ┌─────────────────────────┐                 │
│                   │   HeapObjectRef         │                 │
│                   │                         │                 │
│                   │ - refCount: Number = 1  │                 │
│                   │ - isWeak: Bool = false  │                 │
│                   └─────────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘
```

### HeapObjectRef (Modificado)

```vela
public class HeapObjectRef {
  public object: HeapObject
  public metadata: GCMetadata
  
  # ARC (TASK-075): Automatic Reference Counting
  public refCount: Number = 1        # Reference count (empieza en 1)
  public isWeak: Bool = false        # Weak reference flag
  
  constructor(object: HeapObject, size: Number) {
    this.object = object
    this.metadata = GCMetadata(size)
    # refCount ya inicializado a 1
  }
}
```

**Decisiones:**
- `refCount = 1`: Objetos nuevos tienen 1 referencia al crearse
- `isWeak`: Weak refs no afectan refCount (para romper cycles)

### ARCManager (Nuevo)

Clase principal que maneja el reference counting.

**Archivo:** `vm/arc.vela`

#### Campos Principales

```vela
public class ARCManager {
  heap: VelaHeap                              # Referencia al heap
  stats: ARCStats                             # Estadísticas de ARC
  autoreleasePool: List<HeapObjectRef> = []   # Pool de autorelease
  cycleDetectionThreshold: Number = 1000      # Umbral para cycle detection
  allocationsSinceLastCycleCheck: Number = 0
  
  constructor(heap: VelaHeap) {
    this.heap = heap
    this.stats = ARCStats()
  }
}
```

#### Métodos Core

##### 1. `retain(ref: HeapObjectRef) -> void`

**Propósito:** Incrementar refCount de un objeto.

```vela
public fn retain(ref: HeapObjectRef) -> void {
  # Weak refs no incrementan refCount
  if ref.isWeak { return }
  
  ref.refCount = ref.refCount + 1
  this.stats.recordRetain()
}
```

**Cuándo se llama:**
- `OP_STORE_LOCAL`: Retain nueva variable
- `OP_DUP`: Retain al duplicar ref en stack
- `OP_STORE_GLOBAL`: Retain nueva global
- `OP_STORE_UPVALUE`: Retain en closure
- `OP_BUILD_LIST`: Retain items de lista

##### 2. `release(ref: HeapObjectRef) -> void`

**Propósito:** Decrementar refCount y liberar si llega a 0.

```vela
public fn release(ref: HeapObjectRef) -> void {
  # Weak refs no afectan refCount
  if ref.isWeak { return }
  
  # Validar refCount > 0
  if ref.refCount <= 0 {
    throw Error("ARC: release() on object with refcount <= 0")
  }
  
  ref.refCount = ref.refCount - 1
  this.stats.recordRelease()
  
  # Free inmediato cuando refCount = 0
  if ref.refCount == 0 {
    this.free(ref)
  }
}
```

**Cuándo se llama:**
- `OP_POP`: Release al eliminar de stack
- `OP_RETURN`: Release locals del frame
- Variable fuera de scope: Release automático
- `OP_STORE_LOCAL`: Release valor antiguo antes de overwrite

##### 3. `autorelease(ref: HeapObjectRef) -> void`

**Propósito:** Diferir release hasta end of scope (autorelease pool).

```vela
public fn autorelease(ref: HeapObjectRef) -> void {
  if ref.isWeak { return }
  this.autoreleasePool.append(ref)
}
```

**Cuándo se usa:**
- Return values que deben sobrevivir más allá del scope actual
- Patrón similar a Objective-C autorelease

##### 4. `drainAutoreleasePool() -> void`

**Propósito:** Liberar todos los objetos en autorelease pool.

```vela
public fn drainAutoreleasePool() -> void {
  this.autoreleasePool.forEach(ref => {
    this.release(ref)
  })
  this.autoreleasePool = []
}
```

**Cuándo se llama:**
- `OP_RETURN`: Al final de cada frame

##### 5. `free(ref: HeapObjectRef) -> void`

**Propósito:** Liberación recursiva de objeto y sus hijos.

```vela
public fn free(ref: HeapObjectRef) -> void {
  # 1. Liberar children recursivamente (type-specific)
  match ref.object {
    HeapObject.Closure(closure) => this.freeClosure(closure)
    HeapObject.Upvalue(upvalue) => this.freeUpvalue(upvalue)
    HeapObject.Instance(instance) => this.freeInstance(instance)
    HeapObject.Class(cls) => this.freeClass(cls)
    HeapObject.BoundMethod(method) => this.freeBoundMethod(method)
    HeapObject.List(list) => this.freeList(list)
    HeapObject.Map(map) => this.freeMap(map)
    HeapObject.String(_) => { /* No children */ }
  }
  
  # 2. Deallocate memoria
  this.heap.deallocate(ref)
  
  # 3. Actualizar estadísticas
  this.stats.recordFree()
}
```

**Flujo de liberación recursiva:**

```
free(Closure) ──► release upvalues
                  └─► free(Upvalue) ──► release closed value
                                        └─► free(Value) si HeapObject

free(Instance) ──► release fields
                   └─► free(Field) si HeapObject

free(List) ──► release items
               └─► free(Item) si HeapObject

free(Map) ──► release keys + values
              └─► free(Key/Value) si HeapObject
```

#### Métodos Type-Specific Free

##### `freeClosure(closure: VelaClosure) -> void`

```vela
fn freeClosure(closure: VelaClosure) -> void {
  # Liberar upvalues
  closure.upvalues.forEach(upvalue => {
    match upvalue.ref {
      Some(ref) => this.release(ref)
      None => {}
    }
  })
}
```

##### `freeInstance(instance: VelaInstance) -> void`

```vela
fn freeInstance(instance: VelaInstance) -> void {
  # Liberar fields
  instance.fields.values().forEach(value => {
    releaseValue(this, value)
  })
}
```

##### `freeList(list: VelaHeapList) -> void`

```vela
fn freeList(list: VelaHeapList) -> void {
  # Liberar items
  list.items.forEach(value => {
    releaseValue(this, value)
  })
}
```

##### `freeMap(map: VelaHeapMap) -> void`

```vela
fn freeMap(map: VelaHeapMap) -> void {
  # Liberar keys y values
  map.entries.keys().forEach(key => {
    # Key (String) es primitivo, no necesita release
  })
  
  map.entries.values().forEach(value => {
    releaseValue(this, value)
  })
}
```

#### Helper Functions

```vela
# Helper: Retain value si es HeapObject
public fn retainValue(arc: ARCManager, value: Value) -> void {
  match value {
    Value.HeapObject(ref) => arc.retain(ref)
    _ => {}  # Primitivos no necesitan retain
  }
}

# Helper: Release value si es HeapObject
public fn releaseValue(arc: ARCManager, value: Value) -> void {
  match value {
    Value.HeapObject(ref) => arc.release(ref)
    _ => {}  # Primitivos no necesitan release
  }
}

# Helper: Autorelease value si es HeapObject
public fn autoreleaseValue(arc: ARCManager, value: Value) -> void {
  match value {
    Value.HeapObject(ref) => arc.autorelease(ref)
    _ => {}
  }
}
```

### VelaHeap (Modificado)

**Archivo:** `vm/heap.vela`

#### Cambios realizados:

##### 1. Campo ARC opcional

```vela
public class VelaHeap {
  # ... campos existentes ...
  
  # ARC Manager (TASK-075)
  # NOTE: ARCManager será inyectado después de la creación para evitar
  # dependencia circular (heap.vela → arc.vela → heap.vela)
  arc: Option<ARCManager> = None
}
```

**Razón:** Evitar dependencia circular entre `heap.vela` y `arc.vela`.

##### 2. Método `setARCManager(arcManager: ARCManager)`

```vela
public fn setARCManager(arcManager: ARCManager) -> void {
  this.arc = Some(arcManager)
}
```

**Uso:**
```vela
# Inicialización en VelaVM
heap = VelaHeap()
arc = ARCManager(heap)
heap.setARCManager(arc)
```

##### 3. Método `deallocate(ref: HeapObjectRef)`

```vela
public fn deallocate(ref: HeapObjectRef) -> void {
  # 1. Remover de linked list
  match this.objects {
    Some(headRef) => {
      if headRef === ref {
        # Es el primer objeto
        this.objects = ref.metadata.next
      } else {
        # Buscar en la lista y remover
        current = Some(headRef)
        while let Some(currentRef) = current {
          match currentRef.metadata.next {
            Some(nextRef) => {
              if nextRef === ref {
                currentRef.metadata.next = ref.metadata.next
                break
              }
              current = Some(nextRef)
            }
            None => break
          }
        }
      }
    }
    None => {
      throw Error("deallocate(): heap objects list is empty")
    }
  }
  
  # 2. Remover de interned strings si aplica
  match ref.object {
    HeapObject.String(str) => {
      this.internedStrings.remove(str.value)
    }
    _ => {}
  }
  
  # 3. Actualizar estadísticas
  this.objectCount = this.objectCount - 1
  this.totalFreed = this.totalFreed + ref.metadata.size
}
```

**Llamado por:** `ARCManager.free(ref)` cuando refCount llega a 0.

##### 4. Registro de allocations para cycle detection

```vela
public fn allocate(object: HeapObject, size: Number) -> HeapObjectRef {
  # ... código existente ...
  
  # ARC: Registrar allocation para cycle detection (TASK-075)
  match this.arc {
    Some(arcManager) => arcManager.recordAllocation()
    None => {}  # ARC no inicializado aún
  }
  
  return ref
}
```

**Propósito:** Trigger cycle detection cada N allocations (TASK-076).

### VelaUpvalue (Modificado)

**Archivo:** `vm/heap.vela`

#### Cambio: Campo `ref` para self-reference

```vela
public class VelaUpvalue {
  # ... campos existentes ...
  
  # ARC: Ref a este upvalue en heap
  public ref: Option<HeapObjectRef> = None
  
  # ... métodos existentes ...
}
```

**Propósito:** Permitir que upvalues rastrreen su propia referencia heap para ARC.

**Uso:**
```vela
# Al crear upvalue
upvalue = VelaUpvalue(location, None)
ref = heap.allocateUpvalue(upvalue)
upvalue.ref = Some(ref)
```

---

## 🔧 Integración con VelaVM

### Inicialización

```vela
# vm/velavm.vela
public class VelaVM {
  heap: VelaHeap
  arc: ARCManager
  
  # ... otros campos ...
  
  constructor() {
    # 1. Crear heap
    this.heap = VelaHeap()
    
    # 2. Crear ARCManager
    this.arc = ARCManager(this.heap)
    
    # 3. Configurar heap con ARC
    this.heap.setARCManager(this.arc)
    
    # ... resto de inicialización ...
  }
}
```

### Integración en Opcodes

#### `OP_STORE_LOCAL` (Store en local variable)

```vela
fn opStoreLocal(index: Number, value: Value) -> void {
  # 1. Release valor antiguo (si existe)
  match this.locals[index] {
    Some(oldValue) => releaseValue(this.arc, oldValue)
    None => {}
  }
  
  # 2. Retain nuevo valor
  retainValue(this.arc, value)
  
  # 3. Store
  this.locals[index] = Some(value)
}
```

#### `OP_POP` (Pop from stack)

```vela
fn opPop() -> void {
  match this.stack.pop() {
    Some(value) => {
      # Release valor popped
      releaseValue(this.arc, value)
    }
    None => throw Error("Stack underflow")
  }
}
```

#### `OP_DUP` (Duplicate top of stack)

```vela
fn opDup() -> void {
  match this.stack.peek() {
    Some(value) => {
      # Retain el duplicado
      retainValue(this.arc, value)
      this.stack.push(value)
    }
    None => throw Error("Stack empty on DUP")
  }
}
```

#### `OP_RETURN` (Return from function)

```vela
fn opReturn() -> void {
  # 1. Pop return value (no release, será usado por caller)
  returnValue = this.stack.pop().unwrapOr(Value.None)
  
  # 2. Release todos los locals del frame
  currentFrame = this.callStack.peek().unwrap()
  currentFrame.slots.forEach(value => {
    releaseValue(this.arc, value)
  })
  
  # 3. Drain autorelease pool
  this.arc.drainAutoreleasePool()
  
  # 4. Pop frame
  this.callStack.pop()
  
  # 5. Push return value
  this.stack.push(returnValue)
}
```

#### `OP_STORE_GLOBAL` (Store en variable global)

```vela
fn opStoreGlobal(name: String, value: Value) -> void {
  # 1. Release valor antiguo (si existe)
  match this.globals.get(name) {
    Some(oldValue) => releaseValue(this.arc, oldValue)
    None => {}
  }
  
  # 2. Retain nuevo valor
  retainValue(this.arc, value)
  
  # 3. Store
  this.globals.set(name, value)
}
```

#### `OP_BUILD_LIST` (Construir lista)

```vela
fn opBuildList(itemCount: Number) -> void {
  items: List<Value> = []
  
  # Pop items del stack
  (0..itemCount).forEach(_ => {
    item = this.stack.pop().unwrap()
    items.prepend(item)  # Prepend para mantener orden
  })
  
  # Retain cada item (ahora la lista los posee)
  items.forEach(item => {
    retainValue(this.arc, item)
  })
  
  # Crear lista en heap
  heapList = VelaHeapList()
  heapList.items = items
  ref = this.heap.allocate(HeapObject.List(heapList), 32)
  
  # Push lista al stack (con refCount = 1 desde allocate)
  this.stack.push(Value.HeapObject(ref))
}
```

#### `OP_SET_ATTR` (Set attribute en instance)

```vela
fn opSetAttr(name: String) -> void {
  value = this.stack.pop().unwrap()
  instanceValue = this.stack.pop().unwrap()
  
  match instanceValue {
    Value.HeapObject(ref) => {
      match ref.object {
        HeapObject.Instance(instance) => {
          # 1. Release valor antiguo del field (si existe)
          match instance.getField(name) {
            Some(oldValue) => releaseValue(this.arc, oldValue)
            None => {}
          }
          
          # 2. Retain nuevo valor
          retainValue(this.arc, value)
          
          # 3. Set field
          instance.setField(name, value)
          
          # 4. Push instance de vuelta al stack
          this.stack.push(instanceValue)
        }
        _ => throw Error("SET_ATTR on non-instance")
      }
    }
    _ => throw Error("SET_ATTR on non-heap-object")
  }
}
```

### Resumen de Integración

| Opcode | Retain | Release | Notas |
|--------|--------|---------|-------|
| `OP_STORE_LOCAL` | ✅ Nuevo valor | ✅ Valor antiguo | Overwrite |
| `OP_POP` | ❌ | ✅ Valor popped | Stack pop |
| `OP_DUP` | ✅ Duplicado | ❌ | Duplicate ref |
| `OP_RETURN` | ❌ | ✅ Locals | Drain autorelease pool |
| `OP_STORE_GLOBAL` | ✅ Nuevo valor | ✅ Valor antiguo | Overwrite |
| `OP_STORE_UPVALUE` | ✅ Nuevo valor | ✅ Valor antiguo | Closure capture |
| `OP_SET_ATTR` | ✅ Nuevo field | ✅ Field antiguo | Instance field set |
| `OP_BUILD_LIST` | ✅ Cada item | ❌ | Lista posee items |
| `OP_SET_ITEM` | ✅ Nuevo item | ✅ Item antiguo | List/map item set |

---

## 📊 Estadísticas y Debugging

### ARCStats

```vela
public class ARCStats {
  public retainCount: Number = 0        # Total retain() calls
  public releaseCount: Number = 0       # Total release() calls
  public freedCount: Number = 0         # Total free() calls
  public leakedCount: Number = 0        # Objects con refCount > 0 al shutdown
  public peakLiveObjects: Number = 0    # Max objetos vivos simultáneos
  public currentLiveObjects: Number = 0 # Objetos vivos actuales
  
  public fn recordRetain() -> void {
    this.retainCount = this.retainCount + 1
    this.currentLiveObjects = this.currentLiveObjects + 1
    if this.currentLiveObjects > this.peakLiveObjects {
      this.peakLiveObjects = this.currentLiveObjects
    }
  }
  
  public fn recordRelease() -> void {
    this.releaseCount = this.releaseCount + 1
  }
  
  public fn recordFree() -> void {
    this.freedCount = this.freedCount + 1
    this.currentLiveObjects = this.currentLiveObjects - 1
  }
  
  public fn recordLeak() -> void {
    this.leakedCount = this.leakedCount + 1
  }
  
  override fn toString() -> String {
    return "ARCStats {
      retains: ${this.retainCount},
      releases: ${this.releaseCount},
      freed: ${this.freedCount},
      leaked: ${this.leakedCount},
      peak: ${this.peakLiveObjects},
      current: ${this.currentLiveObjects}
    }"
  }
}
```

### Métodos de Debugging

#### `getStats() -> ARCStats`

Retorna estadísticas actuales.

```vela
stats = vm.arc.getStats()
print(stats)
```

#### `validateRefcount(ref: HeapObjectRef) -> void`

Valida que refCount >= 0.

```vela
public fn validateRefcount(ref: HeapObjectRef) -> void {
  if ref.refCount < 0 {
    throw Error("ARC: Invalid refcount ${ref.refCount} for ${ref.object}")
  }
}
```

#### `dumpLiveObjects() -> void`

Lista todos los objetos con refCount > 0 (placeholder para TASK-078).

```vela
public fn dumpLiveObjects() -> void {
  print("=== Live Objects (refCount > 0) ===")
  # TODO TASK-078: Traverse heap, list objects with refCount > 0
}
```

#### `findLeaks() -> List<HeapObjectRef>`

Detecta objetos unreachable con refCount > 0 (placeholder para TASK-078).

```vela
public fn findLeaks() -> List<HeapObjectRef> {
  # TODO TASK-078: Mark-and-sweep desde roots, find unmarked con refCount > 0
  return []
}
```

---

## 🧪 Testing

### Tests Pendientes (TASK-078)

#### Unit Tests para ARCManager

1. **Test retain/release básico**
```vela
@test
fn testRetainRelease() -> void {
  heap = VelaHeap()
  arc = ARCManager(heap)
  
  ref = heap.allocate(HeapObject.String(VelaString("test")), 32)
  assert(ref.refCount == 1, "Initial refCount should be 1")
  
  arc.retain(ref)
  assert(ref.refCount == 2, "After retain should be 2")
  
  arc.release(ref)
  assert(ref.refCount == 1, "After release should be 1")
}
```

2. **Test free cuando refCount = 0**
```vela
@test
fn testFreeOnZeroRefcount() -> void {
  heap = VelaHeap()
  arc = ARCManager(heap)
  
  ref = heap.allocate(HeapObject.String(VelaString("test")), 32)
  initialCount = heap.objectCount
  
  arc.release(ref)  # refCount: 1 → 0, debería liberar
  
  assert(heap.objectCount == initialCount - 1, "Object should be freed")
}
```

3. **Test autorelease pool**
```vela
@test
fn testAutoreleasePool() -> void {
  heap = VelaHeap()
  arc = ARCManager(heap)
  
  ref1 = heap.allocate(HeapObject.String(VelaString("a")), 32)
  ref2 = heap.allocate(HeapObject.String(VelaString("b")), 32)
  
  arc.autorelease(ref1)
  arc.autorelease(ref2)
  
  assert(arc.autoreleasePool.length() == 2, "Pool should have 2 objects")
  
  arc.drainAutoreleasePool()
  
  assert(arc.autoreleasePool.length() == 0, "Pool should be empty")
  assert(heap.objectCount == 0, "Objects should be freed")
}
```

4. **Test recursive free (Closure → Upvalues)**
```vela
@test
fn testRecursiveFreeClosure() -> void {
  heap = VelaHeap()
  arc = ARCManager(heap)
  
  # Crear closure con upvalue
  upvalue = VelaUpvalue(0, None)
  upvalueRef = heap.allocateUpvalue(upvalue)
  upvalue.ref = Some(upvalueRef)
  
  closure = VelaClosure(someFunction, [upvalue])
  closureRef = heap.allocateClosure(closure)
  
  initialCount = heap.objectCount  # 2 objects: closure + upvalue
  
  # Free closure debería liberar upvalue también
  arc.free(closureRef)
  
  assert(heap.objectCount < initialCount, "Upvalue should be freed")
}
```

5. **Test recursive free (List → Items)**
```vela
@test
fn testRecursiveFreeList() -> void {
  heap = VelaHeap()
  arc = ARCManager(heap)
  
  # Crear lista con strings
  str1Ref = heap.allocate(HeapObject.String(VelaString("a")), 32)
  str2Ref = heap.allocate(HeapObject.String(VelaString("b")), 32)
  
  heapList = VelaHeapList()
  heapList.items = [Value.HeapObject(str1Ref), Value.HeapObject(str2Ref)]
  
  listRef = heap.allocate(HeapObject.List(heapList), 32)
  
  # Retain items (lista los posee)
  arc.retain(str1Ref)
  arc.retain(str2Ref)
  
  initialCount = heap.objectCount  # 3 objects: list + 2 strings
  
  # Free list debería liberar items
  arc.free(listRef)
  
  assert(heap.objectCount < initialCount, "Items should be freed")
}
```

6. **Test weak refs (no afectan refCount)**
```vela
@test
fn testWeakRefsDoNotAffectRefcount() -> void {
  heap = VelaHeap()
  arc = ARCManager(heap)
  
  ref = heap.allocate(HeapObject.String(VelaString("test")), 32)
  weakRef = arc.createWeakRef(ref)
  
  assert(weakRef.isWeak == true, "Should be weak")
  assert(weakRef.refCount == 0, "Weak refs don't affect refCount")
  
  arc.retain(weakRef)  # No debería incrementar
  assert(weakRef.refCount == 0, "Weak retain should be no-op")
  
  arc.release(weakRef)  # No debería decrementar
  assert(weakRef.refCount == 0, "Weak release should be no-op")
}
```

7. **Test memory leak detection**
```vela
@test
fn testFindLeaks() -> void {
  heap = VelaHeap()
  arc = ARCManager(heap)
  
  # Crear objeto órfano (refCount > 0 pero unreachable)
  orphan = heap.allocate(HeapObject.String(VelaString("orphan")), 32)
  
  # Simular que no hay referencias desde roots
  leaks = arc.findLeaks()
  
  assert(leaks.length() > 0, "Should detect leak")
  assert(leaks.contains(orphan), "Should detect orphan object")
}
```

8. **Test estadísticas**
```vela
@test
fn testStatistics() -> void {
  heap = VelaHeap()
  arc = ARCManager(heap)
  
  ref = heap.allocate(HeapObject.String(VelaString("test")), 32)
  
  arc.retain(ref)
  arc.retain(ref)
  arc.release(ref)
  arc.release(ref)
  arc.release(ref)  # Free
  
  stats = arc.getStats()
  
  assert(stats.retainCount == 2, "Should record 2 retains")
  assert(stats.releaseCount == 3, "Should record 3 releases")
  assert(stats.freedCount == 1, "Should record 1 free")
}
```

#### Integration Tests con VelaVM

9. **Test OP_STORE_LOCAL retain/release**
```vela
@test
fn testOpStoreLocalARC() -> void {
  vm = VelaVM()
  
  # local x = "hello"
  vm.opPushConstant(Value.String("hello"))
  vm.opStoreLocal(0)
  
  # Verificar refCount
  match vm.locals[0] {
    Some(Value.HeapObject(ref)) => {
      assert(ref.refCount == 1, "Local should have refCount 1")
    }
    _ => throw Error("Expected HeapObject in local 0")
  }
  
  # local x = "world"  (overwrite)
  vm.opPushConstant(Value.String("world"))
  vm.opStoreLocal(0)
  
  # "hello" debería haberse liberado
  assert(vm.heap.objectCount == 1, "Old value should be freed")
}
```

10. **Test OP_RETURN drain autorelease pool**
```vela
@test
fn testOpReturnDrainsPool() -> void {
  vm = VelaVM()
  
  # Simular función que retorna string
  vm.arc.autorelease(someRef)
  
  assert(vm.arc.autoreleasePool.length() == 1, "Pool should have 1 object")
  
  vm.opReturn()
  
  assert(vm.arc.autoreleasePool.length() == 0, "Pool should be empty after return")
}
```

### Coverage Goal

**Target:** >= 80% coverage de `vm/arc.vela`

**Critical Paths:**
- ✅ retain/release correctness
- ✅ Free cuando refCount = 0
- ✅ Recursive freeing (Closure, Instance, List, Map)
- ✅ Autorelease pool
- ✅ Weak refs (no-op en retain/release)
- ✅ Statistics tracking

---

## ⚡ Performance

### Overhead Esperado

**CPU:**
- ~5-10% overhead por retain/release en cada assignment
- Compensado por eliminación de pausas STW de GC

**Memoria:**
- Liberación inmediata → **menor peak memory** (vs Mark-and-Sweep)
- `refCount` agrega 8 bytes por objeto (Number)
- `isWeak` agrega 1 byte por objeto (Bool)
- Total: **~9 bytes overhead por objeto**

### Benchmarks Pendientes (TASK-078)

1. **Memory Peak Comparison**
   - Benchmark: Crear 10K objetos sin liberar
   - Métrica: Peak memory (ARC vs Mark-and-Sweep)
   - Expectativa: ARC usa ~30-50% menos peak memory

2. **Latency (No STW Pauses)**
   - Benchmark: Crear y liberar objetos en loop
   - Métrica: P99 latency
   - Expectativa: ARC tiene latency consistente, sin spikes de GC

3. **Throughput (Assignments per second)**
   - Benchmark: Assignment-heavy workload
   - Métrica: Assignments/sec
   - Expectativa: ARC ~5-10% slower que Mark-and-Sweep (por overhead)

4. **Recursive Free Performance**
   - Benchmark: Liberar árbol profundo (nested objects)
   - Métrica: Time to free
   - Expectativa: ARC O(N) recursivo, similar a Mark-and-Sweep sweep phase

---

## 🔮 Trabajo Futuro (TASK-076, TASK-077, TASK-078)

### TASK-076: Cycle Detection + Weak Refs (Sprint 25)

**Pendiente:**
1. Implementar `Weak<T>` type en `vm/weak.vela`
2. Implementar `ARCManager.lockWeakRef()` (convertir weak→strong temporalmente)
3. Implementar `ARCManager.checkForCycles()` (mark-and-sweep periódico para cycles)
4. Trigger cycle detection cada N allocations (threshold configurable)

**Ejemplo de uso:**
```vela
# Crear weak ref para romper cycle
parent.child = child       # Strong ref
child.parent = weak(parent)  # Weak ref (no afecta refCount)

# Al liberar parent, child.parent se invalida automáticamente
```

### TASK-077: Integración con Sistema Reactivo (Sprint 25)

**Pendiente:**
1. Integrar ARC con `signal<T>` (signals deben retain valores)
2. Integrar ARC con `computed<T>` (computed retiene dependencias)
3. Integrar ARC con `effect` (effects retienen señales observadas)

**Ejemplo:**
```vela
state user: User = fetchUser()  # ARC: retain user object

computed fullName: String {
  return "${this.user.firstName} ${this.user.lastName}"
}  # ARC: retain user while computed is active

effect {
  print("User: ${this.user.name}")
}  # ARC: retain user while effect is active
```

### TASK-078: Tests Completos y Optimizaciones (Sprint 25)

**Pendiente:**
1. 40+ unit tests para ARCManager
2. Integration tests con VelaVM opcodes
3. Benchmarks de performance (memory, latency, throughput)
4. Leak detection tools (`dumpLiveObjects`, `findLeaks`)
5. Optimizaciones:
   - Inline retain/release en hot paths
   - Batch autorelease drain
   - Fast path para refCount == 1 (no need to retain/release)

---

## ✅ Criterios de Aceptación (DoD)

- [x] **ADR-075 creado** - Decisión arquitectónica documentada
- [x] **ARCManager implementado** - Clase principal con retain/release/free
- [x] **HeapObjectRef modificado** - Agregado refCount + isWeak
- [x] **VelaHeap modificado** - Agregado deallocate() + ARC integration
- [x] **VelaUpvalue modificado** - Agregado campo ref
- [x] **Documentación TASK-075** - Este archivo completo
- [ ] **VelaVM opcodes modificados** - Retain/release en ~15 opcodes (EN PROGRESO)
- [ ] **Tests unitarios** - >= 40 tests con >= 80% coverage (PENDIENTE TASK-078)
- [ ] **Tests de integración** - VelaVM opcodes (PENDIENTE TASK-078)
- [ ] **Benchmarks** - Memory, latency, throughput (PENDIENTE TASK-078)
- [ ] **Code review aprobado** - (PENDIENTE)
- [ ] **Commit realizado** - (PENDIENTE)

---

## 📁 Archivos Generados

### Código Fuente

1. **`docs/architecture/ADR-075-automatic-reference-counting.md`** (665 lines)
   - Decisión arquitectónica completa
   - Análisis de 4 opciones
   - Arquitectura ARC + Cycle Detection
   - Consecuencias y trade-offs

2. **`vm/arc.vela`** (542 lines)
   - ARCStats class
   - ARCManager class con 20+ métodos
   - Type-specific free methods
   - Helper functions

3. **`vm/heap.vela`** (modificado, +70 lines)
   - Agregado refCount + isWeak a HeapObjectRef
   - Agregado ref a VelaUpvalue
   - Agregado campo arc a VelaHeap
   - Agregado método deallocate()
   - Agregado método setARCManager()

### Documentación

4. **`docs/features/VELA-587/TASK-075.md`** (este archivo, ~1,000 lines)
   - Especificación completa de ARC
   - Arquitectura y componentes
   - Integración con VelaVM
   - Tests y benchmarks
   - Trabajo futuro

### Tests (Pendiente TASK-078)

5. **`tests/unit/vm/test_arc.vela`** (PENDIENTE, ~600 lines estimadas)
   - 40+ tests unitarios
   - Integration tests

---

## 🔗 Referencias

### ADR y Documentación
- [ADR-075: Automatic Reference Counting](../../architecture/ADR-075-automatic-reference-counting.md)
- [VELA-587: Memory Management Automático](./README.md)

### Jira
- **Historia:** [VELA-587](https://velalang.atlassian.net/browse/VELA-587)
- **Epic:** [EPIC-06: Compiler Backend](https://velalang.atlassian.net/browse/EPIC-06)
- **Task:** [TASK-075](https://velalang.atlassian.net/browse/TASK-075)

### Referencias Externas
- **Swift ARC:** https://docs.swift.org/swift-book/LanguageGuide/AutomaticReferenceCounting.html
- **Objective-C Memory Management:** https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/MemoryMgmt/
- **Python CPython RC:** https://devguide.python.org/internals/garbage-collector/
- **Rust Rc<T>:** https://doc.rust-lang.org/std/rc/struct.Rc.html

---

**ESTADO ACTUAL:** En Curso 🔄  
**PRÓXIMO PASO:** Modificar VelaVM opcodes para integrar retain/release  
**BLOQUEADORES:** Ninguno  
**FECHA DE ACTUALIZACIÓN:** 2025-01-30
