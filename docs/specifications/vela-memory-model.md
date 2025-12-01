# Vela Memory Model Specification (Formal)

**Version:** 0.1.0-draft  
**Status:** Work in Progress  
**Last Updated:** 2025-11-30

---

## Table of Contents

1. [Object Lifetime Rules](#1-object-lifetime-rules)
2. [ARC Reference Counting Algorithm](#2-arc-reference-counting-algorithm)
3. [Cycle Detection Algorithm](#3-cycle-detection-algorithm)
4. [Weak Reference Semantics](#4-weak-reference-semantics)
5. [Thread Safety Guarantees](#5-thread-safety-guarantees)
6. [Memory Visibility Across Threads](#6-memory-visibility-across-threads)

---

## 1. Object Lifetime Rules

### 1.1 Object Creation

**Rule:** Objects are created on the heap and managed via Automatic Reference Counting (ARC).

```
alloc(T) → ref(T)
```

**Semantic guarantee:** Every allocation returns a valid reference with refcount = 1.

### 1.2 Ownership Transfer

**Rule:** Assignment transfers ownership without incrementing refcount.

```vela
let a = Object();  // refcount(a) = 1
let b = a;         // refcount(b) = 1, a is now invalid
```

**Formal semantics:**

```
⟨let a = alloc(T), σ⟩ → ⟨(), σ[a ↦ ref(obj, rc=1)]⟩

⟨let b = a, σ[a ↦ ref(obj, rc=1)]⟩ → ⟨(), σ[b ↦ ref(obj, rc=1), a ↦ invalid]⟩
```

### 1.3 Reference Incrementation

**Rule:** Explicit cloning increments refcount.

```vela
let a = Object();   // refcount = 1
let b = a.clone();  // refcount = 2
```

**Formal semantics:**

```
⟨a.clone(), σ[a ↦ ref(obj, rc=n)]⟩ → ⟨ref(obj, rc=n+1), σ[a ↦ ref(obj, rc=n+1)]⟩
```

### 1.4 Object Deallocation

**Rule:** Objects are deallocated when refcount reaches 0.

```
refcount(obj) = 0 → dealloc(obj)
```

**Guarantee:** Deallocation occurs deterministically when last reference goes out of scope.

---

## 2. ARC Reference Counting Algorithm

### 2.1 Reference Count Operations

#### 2.1.1 Increment

```rust
fn retain(obj: *mut Object) {
    atomic_fetch_add(&obj.refcount, 1, Ordering::Relaxed);
}
```

**Properties:**
- Atomic operation (thread-safe)
- Relaxed ordering (no synchronization needed for increment)

#### 2.1.2 Decrement

```rust
fn release(obj: *mut Object) {
    let old_count = atomic_fetch_sub(&obj.refcount, 1, Ordering::Release);
    if old_count == 1 {
        atomic_fence(Ordering::Acquire);  // Synchronize with other threads
        drop(obj);  // Dealloc object
    }
}
```

**Properties:**
- Atomic operation (thread-safe)
- Release ordering on decrement
- Acquire fence before deallocation (ensures visibility of all previous writes)

### 2.2 Complexity Guarantees

| Operation | Time Complexity | Space Overhead |
|-----------|----------------|----------------|
| retain() | O(1) | O(1) per object |
| release() | O(1) amortized | O(1) per object |
| clone() | O(1) | O(n) if deep copy |

### 2.3 Refcount Overflow

**Guarantee:** Runtime panic if refcount exceeds `isize::MAX`.

```vela
// Pseudocode
if refcount >= MAX_REFCOUNT {
    panic("Reference count overflow");
}
```

---

## 3. Cycle Detection Algorithm

### 3.1 Problem Statement

**Issue:** Cyclic references prevent deallocation.

```vela
let a = Object();
let b = Object();
a.ref = b;  // a → b
b.ref = a;  // b → a (cycle!)
```

Both objects have refcount = 2 but are unreachable from root.

### 3.2 Cycle Collection Algorithm (Bacon & Rajan)

#### 3.2.1 Color-Based Marking

**Colors:**
- **Black**: Definitely alive (reachable from root)
- **Gray**: Potentially cyclic garbage
- **White**: Definitely garbage

#### 3.2.2 Algorithm Steps

**Step 1: Mark Candidates**

```
for each obj in possibly_cyclic:
    if obj.refcount > 0:
        mark_gray(obj)
```

**Step 2: Scan Roots**

```
for each root in roots:
    mark_black(root)  // Transitively mark all reachable
```

**Step 3: Collect White Objects**

```
for each obj in possibly_cyclic:
    if obj.color == WHITE:
        dealloc(obj)
```

#### 3.2.3 Complexity

| Phase | Time Complexity | Space Complexity |
|-------|----------------|------------------|
| Mark candidates | O(n) | O(n) |
| Scan roots | O(V + E) | O(d) for depth d |
| Collect | O(n) | O(1) |

**Total:** O(V + E) where V = objects, E = references

### 3.3 Trigger Conditions

**Rule:** Cycle collection runs when:

1. Allocation count > threshold (e.g., 1000 objects)
2. Manual trigger via `System.collectGarbage()`
3. Memory pressure detected

**Guarantee:** Cycle collection is non-incremental but bounded in time.

---

## 4. Weak Reference Semantics

### 4.1 Definition

**Weak reference:** Reference that doesn't increment refcount.

```vela
let strong = Object();      // refcount = 1
let weak = Weak(strong);   // refcount still 1
```

### 4.2 Access Rules

**Rule:** Accessing weak reference may return `null` if object was deallocated.

```vela
let strong = Object();
let weak = Weak(strong);

drop(strong);  // Deallocate object

let upgraded = weak.upgrade();  // Returns null
```

**Formal semantics:**

```
⟨weak.upgrade(), σ⟩ → ⟨
    if refcount(weak.obj) > 0 then Some(ref(weak.obj))
    else null
, σ⟩
```

### 4.3 Thread Safety

**Guarantee:** Weak references are thread-safe.

```rust
struct Weak<T> {
    ptr: *const T,
    weak_count: AtomicUsize,  // Separate weak refcount
}
```

**Properties:**
- Weak references have their own atomic counter
- Object is fully deallocated only when both strong and weak counts reach 0

---

## 5. Thread Safety Guarantees

### 5.1 Send and Sync Traits

**Send:** Type can be transferred across thread boundaries.

```vela
interface Send {}
```

**Sync:** Type can be referenced from multiple threads simultaneously.

```vela
interface Sync {}
```

**Rules:**
- `T: Send` if all fields of `T` are `Send`
- `T: Sync` if `&T: Send`

### 5.2 Atomic Reference Counting

**Guarantee:** ARC operations are atomic and thread-safe.

```vela
let obj = Object();         // Thread A
let obj2 = obj.clone();    // Thread B (safe!)
```

**Implementation:**

```rust
struct Arc<T> {
    ptr: NonNull<ArcInner<T>>,
}

struct ArcInner<T> {
    strong: AtomicUsize,
    weak: AtomicUsize,
    data: T,
}
```

### 5.3 Data Race Freedom

**Theorem:** Vela's type system prevents data races.

**Proof sketch:**
1. Mutable references (`&mut T`) are exclusive (only one can exist)
2. Shared references (`&T`) are read-only
3. Cannot have `&T` and `&mut T` simultaneously

```vela
let x = 42;
let r1 = &x;      // OK
let r2 = &x;      // OK (multiple readers)
let r3 = &mut x;  // ERROR: cannot borrow as mutable while immutable refs exist
```

---

## 6. Memory Visibility Across Threads

### 6.1 Memory Ordering

**Vela uses C++11/Rust memory model:**

| Ordering | Guarantees |
|----------|------------|
| Relaxed | Atomic, no ordering |
| Release | Writes before Release are visible after Acquire |
| Acquire | Reads after Acquire see writes before Release |
| AcqRel | Both Release and Acquire |
| SeqCst | Sequential consistency (total order) |

### 6.2 Happens-Before Relationship

**Definition:** `A happens-before B` if:

1. `A` and `B` are in same thread and `A` comes before `B`
2. `A` is a Release and `B` is an Acquire on same atomic
3. Transitivity: `A happens-before C` and `C happens-before B`

**Guarantee:** If `A happens-before B`, writes in `A` are visible in `B`.

### 6.3 Example: Message Passing

```vela
// Thread 1
let data = [1, 2, 3];
let ready = Atomic(false);

data[0] = 42;
ready.store(true, Release);  // (A)

// Thread 2
while !ready.load(Acquire) {}  // (B)
print(data[0]);  // Guaranteed to see 42
```

**Why it works:**
- (A) Release ensures all writes before it are visible
- (B) Acquire synchronizes with (A)
- Therefore, write to `data[0]` happens-before read in Thread 2

### 6.4 Memory Fences

**Manual synchronization:**

```vela
atomic_fence(Acquire);   // Acquire fence
atomic_fence(Release);   // Release fence
atomic_fence(SeqCst);    // Full fence
```

**Use case:** Synchronize non-atomic operations.

---

## 7. Memory Safety Guarantees

### 7.1 No Dangling Pointers

**Guarantee:** Type system prevents use-after-free.

```vela
let ref = {
    let obj = Object();
    &obj
};  // ERROR: `obj` does not live long enough
// ref.method();  // Would be use-after-free
```

### 7.2 No Double-Free

**Guarantee:** ARC ensures objects are freed exactly once.

```vela
let a = Object();
let b = a.clone();
drop(a);  // refcount = 1
drop(b);  // refcount = 0, object freed
// Object is freed exactly once
```

### 7.3 No Memory Leaks (Except Cycles)

**Guarantee:** Non-cyclic structures are freed deterministically.

**Exception:** Cyclic structures require cycle collection (see Section 3).

---

## Appendix A: Atomic Operations

### A.1 Supported Atomics

```vela
type Atomic<T> where T: Copy + Send + Sync

// Methods
fn load(ordering: Ordering) -> T
fn store(value: T, ordering: Ordering)
fn swap(value: T, ordering: Ordering) -> T
fn compare_exchange(
    current: T,
    new: T,
    success: Ordering,
    failure: Ordering
) -> Result<T, T>
fn fetch_add(value: T, ordering: Ordering) -> T  // For integers
fn fetch_sub(value: T, ordering: Ordering) -> T  // For integers
```

### A.2 Valid Types for Atomic<T>

- `Bool`
- `Int` (i8, i16, i32, i64, isize)
- `UInt` (u8, u16, u32, u64, usize)
- Pointers (`*const T`, `*mut T`)

---

## Appendix B: Memory Layout

### B.1 Object Header

```rust
struct ObjectHeader {
    refcount: AtomicUsize,    // 8 bytes
    weak_count: AtomicUsize,  // 8 bytes
    type_id: usize,           // 8 bytes (for runtime type info)
    flags: u32,               // 4 bytes (GC marks, etc.)
    padding: u32,             // 4 bytes (alignment)
}
// Total: 32 bytes overhead per object
```

### B.2 Alignment Requirements

- Objects are 8-byte aligned
- Atomic operations require natural alignment
- Arrays are densely packed

---

**References:**
- Bacon & Rajan Cycle Collection: https://researcher.watson.ibm.com/researcher/files/us-bacon/Bacon01Concurrent.pdf
- C++11 Memory Model: https://en.cppreference.com/w/cpp/atomic/memory_order
- Rust Nomicon (Memory Model): https://doc.rust-lang.org/nomicon/

---

*Document generated for Sprint 1 (TASK-000G)*  
*Historia: VELA-561 (US-00B)*  
*Last updated: 2025-11-30*
