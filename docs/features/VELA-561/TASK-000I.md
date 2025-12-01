# TASK-000I: Contratos Formales de la Standard Library

## 📋 Información General
- **Historia:** VELA-561 (Formal Specifications - Phase 0)
- **Epic:** EPIC-00B: Formal Specifications
- **Sprint:** 1
- **Estado:** Pendiente ⏳
- **Prioridad:** P0 (Crítica)
- **Estimación:** 56 horas
- **Dependencias:** TASK-000F, TASK-000G, TASK-000H

---

## 🎯 Objetivo

Documentar contratos formales (precondiciones, postcondiciones, invariantes) para todas las APIs de la **Standard Library** de Vela, incluyendo:

- **Precondiciones y postcondiciones** para cada función
- **Invariantes de tipos** (propiedades que siempre se cumplen)
- **Garantías de performance** (complejidad Big-O)
- **Thread-safety guarantees**
- **Platform-specific behavior**

---

## 📐 Contratos Formales de stdlib

### 1. Collections: `List<T>`

#### 1.1 Contrato de Tipo

```vela
type List<T> = {
  items: Array<T>    # Internal storage (private)
  length: Number     # Number of elements
}

# Invariantes:
I1: length >= 0
I2: length == items.length
I3: ∀ i ∈ [0, length), items[i] is initialized
```

#### 1.2 Operaciones

##### `List.new() -> List<T>`

**Precondiciones:** Ninguna

**Postcondiciones:**
```
P1: result.length == 0
P2: result.items == []
```

**Complejidad:** O(1)

**Thread-safety:** Safe (inmutable)

**Ejemplo:**
```vela
list: List<Number> = List.new()
assert(list.length == 0)  # P1
```

---

##### `List.push(item: T) -> List<T>`

**Precondiciones:** Ninguna

**Postcondiciones:**
```
P1: result.length == this.length + 1
P2: result[result.length - 1] == item
P3: ∀ i ∈ [0, this.length), result[i] == this[i]  (elementos previos intactos)
```

**Complejidad:** 
- Amortized O(1)
- Worst-case O(n) (cuando necesita resize)

**Thread-safety:** Safe (retorna nueva lista)

**Ejemplo:**
```vela
list1: List<Number> = List.new()
list2 = list1.push(42)

assert(list2.length == list1.length + 1)  # P1
assert(list2[0] == 42)  # P2
assert(list1.length == 0)  # list1 inmutable
```

---

##### `List.get(index: Number) -> Option<T>`

**Precondiciones:** Ninguna

**Postcondiciones:**
```
P1: index >= 0 ∧ index < this.length ⟹ result == Some(this.items[index])
P2: index < 0 ∨ index >= this.length ⟹ result == None
```

**Complejidad:** O(1)

**Thread-safety:** Safe (inmutable)

**Ejemplo:**
```vela
list: List<Number> = [1, 2, 3]

assert(list.get(0) == Some(1))  # P1
assert(list.get(10) == None)    # P2
assert(list.get(-1) == None)    # P2
```

---

##### `List.map<U>(fn: (T) -> U) -> List<U>`

**Precondiciones:**
```
PR1: fn debe ser función válida
PR2: fn no debe tener side effects (pureza)
```

**Postcondiciones:**
```
P1: result.length == this.length
P2: ∀ i ∈ [0, length), result[i] == fn(this[i])
P3: Orden de evaluación: left-to-right (garantizado)
```

**Complejidad:** O(n), donde n = this.length

**Thread-safety:** Safe si `fn` es thread-safe

**Ejemplo:**
```vela
list: List<Number> = [1, 2, 3]
doubled = list.map(x => x * 2)

assert(doubled.length == list.length)  # P1
assert(doubled[0] == 2)  # P2 (fn(1) = 2)
assert(doubled[1] == 4)  # P2 (fn(2) = 4)
assert(doubled[2] == 6)  # P2 (fn(3) = 6)
```

---

##### `List.filter(predicate: (T) -> Bool) -> List<T>`

**Precondiciones:**
```
PR1: predicate debe ser función válida
PR2: predicate no debe tener side effects (pureza)
```

**Postcondiciones:**
```
P1: result.length <= this.length
P2: ∀ x ∈ result, predicate(x) == true
P3: ∀ x ∈ this, predicate(x) == true ⟹ x ∈ result
P4: Orden relativo preservado
```

**Complejidad:** O(n)

**Thread-safety:** Safe si `predicate` es thread-safe

**Ejemplo:**
```vela
list: List<Number> = [1, 2, 3, 4, 5]
evens = list.filter(x => x % 2 == 0)

assert(evens.length <= list.length)  # P1
assert(evens == [2, 4])  # P2, P3
```

---

##### `List.reduce<U>(fn: (U, T) -> U, initial: U) -> U`

**Precondiciones:**
```
PR1: fn debe ser función válida
```

**Postcondiciones:**
```
P1: this.length == 0 ⟹ result == initial
P2: this.length == 1 ⟹ result == fn(initial, this[0])
P3: this.length > 1 ⟹ result == fn(...fn(fn(initial, this[0]), this[1]), ..., this[n-1])
P4: Orden de aplicación: left-to-right (fold left)
```

**Complejidad:** O(n)

**Thread-safety:** Safe si `fn` es thread-safe

**Ejemplo:**
```vela
list: List<Number> = [1, 2, 3, 4]
sum = list.reduce((acc, x) => acc + x, 0)

# Evaluación:
# acc=0, x=1 → acc'=1
# acc=1, x=2 → acc'=3
# acc=3, x=3 → acc'=6
# acc=6, x=4 → acc'=10

assert(sum == 10)  # P3
```

---

### 2. Collections: `Map<K, V>`

#### 2.1 Contrato de Tipo

```vela
type Map<K, V> = {
  entries: Array<(K, V)>  # Internal storage (private)
  size: Number            # Number of key-value pairs
}

# Invariantes:
I1: size >= 0
I2: size == entries.length
I3: ∀ i, j donde i ≠ j, entries[i].key ≠ entries[j].key  (keys únicas)
```

#### 2.2 Operaciones

##### `Map.new() -> Map<K, V>`

**Postcondiciones:**
```
P1: result.size == 0
P2: result.entries == []
```

**Complejidad:** O(1)

---

##### `Map.set(key: K, value: V) -> Map<K, V>`

**Postcondiciones:**
```
P1: result.get(key) == Some(value)
P2: key ∈ this ⟹ result.size == this.size  (update)
P3: key ∉ this ⟹ result.size == this.size + 1  (insert)
P4: ∀ k ≠ key, result.get(k) == this.get(k)  (otros keys intactos)
```

**Complejidad:** 
- Average O(1) (hash map)
- Worst O(n) (collision)

---

##### `Map.get(key: K) -> Option<V>`

**Postcondiciones:**
```
P1: key ∈ this ⟹ result == Some(this.value_for(key))
P2: key ∉ this ⟹ result == None
```

**Complejidad:** Average O(1)

---

##### `Map.remove(key: K) -> Map<K, V>`

**Postcondiciones:**
```
P1: result.get(key) == None
P2: key ∈ this ⟹ result.size == this.size - 1
P3: key ∉ this ⟹ result.size == this.size
P4: ∀ k ≠ key, result.get(k) == this.get(k)
```

**Complejidad:** Average O(1)

---

### 3. Option Type: `Option<T>`

#### 3.1 Contrato de Tipo

```vela
enum Option<T> {
  Some(T)
  None
}

# Invariantes:
I1: ∀ opt: Option<T>, opt == Some(_) ∨ opt == None  (exhaustivo)
I2: Some(x) ≠ None  (discriminated union)
```

#### 3.2 Operaciones

##### `Option.unwrap() -> T`

**Precondiciones:**
```
PR1: this == Some(_)  (NO None)
```

**Postcondiciones:**
```
P1: this == Some(x) ⟹ result == x
P2: this == None ⟹ panic!("unwrap on None")
```

**Complejidad:** O(1)

**⚠️ UNSAFE:** Panic en runtime si `None`

**Ejemplo:**
```vela
opt1: Option<Number> = Some(42)
opt2: Option<Number> = None

x = opt1.unwrap()  # OK: x == 42
y = opt2.unwrap()  # PANIC!
```

---

##### `Option.unwrapOr(default: T) -> T`

**Precondiciones:** Ninguna

**Postcondiciones:**
```
P1: this == Some(x) ⟹ result == x
P2: this == None ⟹ result == default
P3: Nunca panic
```

**Complejidad:** O(1)

**Ejemplo:**
```vela
opt: Option<Number> = None
x = opt.unwrapOr(100)
assert(x == 100)  # P2
```

---

##### `Option.map<U>(fn: (T) -> U) -> Option<U>`

**Precondiciones:**
```
PR1: fn debe ser función válida
```

**Postcondiciones:**
```
P1: this == Some(x) ⟹ result == Some(fn(x))
P2: this == None ⟹ result == None
P3: fn NO ejecutado si this == None
```

**Complejidad:** O(1) + complejidad de `fn`

**Ejemplo:**
```vela
opt1: Option<Number> = Some(5)
opt2: Option<Number> = None

result1 = opt1.map(x => x * 2)  # Some(10)
result2 = opt2.map(x => x * 2)  # None (fn no ejecutado)
```

---

##### `Option.flatMap<U>(fn: (T) -> Option<U>) -> Option<U>`

**Precondiciones:**
```
PR1: fn debe retornar Option<U>
```

**Postcondiciones:**
```
P1: this == Some(x) ⟹ result == fn(x)
P2: this == None ⟹ result == None
P3: No nested Option (Option<Option<U>> → Option<U>)
```

**Complejidad:** O(1) + complejidad de `fn`

**Ejemplo:**
```vela
fn safeDivide(a: Number, b: Number) -> Option<Number> {
  if b == 0 { return None }
  return Some(a / b)
}

opt: Option<Number> = Some(10)

result = opt.flatMap(x => safeDivide(x, 2))  # Some(5)
# Sin flatMap: Some(Some(5)) ❌
```

---

### 4. Result Type: `Result<T, E>`

#### 4.1 Contrato de Tipo

```vela
enum Result<T, E> {
  Ok(T)
  Err(E)
}

# Invariantes:
I1: ∀ res: Result<T, E>, res == Ok(_) ∨ res == Err(_)
I2: Ok(x) ≠ Err(e)
```

#### 4.2 Operaciones

##### `Result.unwrap() -> T`

**Precondiciones:**
```
PR1: this == Ok(_)
```

**Postcondiciones:**
```
P1: this == Ok(x) ⟹ result == x
P2: this == Err(e) ⟹ panic!("unwrap on Err: ${e}")
```

**⚠️ UNSAFE:** Panic si `Err`

---

##### `Result.unwrapOr(default: T) -> T`

**Postcondiciones:**
```
P1: this == Ok(x) ⟹ result == x
P2: this == Err(_) ⟹ result == default
P3: Nunca panic
```

---

##### `Result.map<U>(fn: (T) -> U) -> Result<U, E>`

**Postcondiciones:**
```
P1: this == Ok(x) ⟹ result == Ok(fn(x))
P2: this == Err(e) ⟹ result == Err(e)
P3: fn NO ejecutado si this == Err
```

---

##### `Result.mapErr<F>(fn: (E) -> F) -> Result<T, F>`

**Postcondiciones:**
```
P1: this == Ok(x) ⟹ result == Ok(x)
P2: this == Err(e) ⟹ result == Err(fn(e))
```

---

### 5. String Operations

#### 5.1 Contrato de Tipo

```vela
type String = {
  bytes: Array<u8>    # UTF-8 encoded bytes
  length: Number      # Length in characters (NOT bytes)
}

# Invariantes:
I1: bytes es válido UTF-8
I2: length == number_of_unicode_characters(bytes)
I3: length <= bytes.length  (multi-byte characters)
```

#### 5.2 Operaciones

##### `String.length() -> Number`

**Postcondiciones:**
```
P1: result == number_of_unicode_characters(this)
P2: result NO es número de bytes
```

**Complejidad:** O(1) (cached)

**Ejemplo:**
```vela
s1: String = "hello"
s2: String = "你好"  # 2 caracteres chinos

assert(s1.length() == 5)  # P1: 5 chars
assert(s2.length() == 2)  # P1: 2 chars (NO 6 bytes)
```

---

##### `String.concat(other: String) -> String`

**Postcondiciones:**
```
P1: result.length == this.length + other.length
P2: result[0..this.length] == this
P3: result[this.length..] == other
```

**Complejidad:** O(n + m), donde n = this.length, m = other.length

---

##### `String.substring(start: Number, end: Number) -> String`

**Precondiciones:**
```
PR1: 0 <= start <= end <= this.length
```

**Postcondiciones:**
```
P1: result.length == end - start
P2: result == this[start..end]
P3: start == end ⟹ result == ""
P4: Violación de PR1 ⟹ panic!
```

**Complejidad:** O(end - start)

---

##### `String.trim() -> String`

**Postcondiciones:**
```
P1: result.length <= this.length
P2: result no tiene whitespace al inicio ni final
P3: Whitespace interno preservado
```

**Complejidad:** O(n)

---

### 6. Async/Await: `Future<T>`

#### 6.1 Contrato de Tipo

```vela
type Future<T> = {
  state: FutureState<T>
}

enum FutureState<T> {
  Pending         # Computación en progreso
  Ready(T)        # Resultado disponible
  Cancelled       # Cancelado
}
```

#### 6.2 Operaciones

##### `Future.await() -> T`

**Precondiciones:** Ninguna

**Postcondiciones:**
```
P1: Suspende ejecución hasta que Future esté Ready
P2: result == value cuando Future == Ready(value)
P3: NO bloquea OS thread (solo suspend task)
P4: Cancelled ⟹ panic!
```

**Complejidad:** Depende de operación asíncrona

**Thread-safety:** Safe (no bloquea threads)

**Ejemplo:**
```vela
async fn fetchData() -> String {
  return "data"
}

async fn process() -> void {
  result = await fetchData()  # Suspend aquí
  print(result)  # Resume cuando fetchData complete
}
```

---

##### `Future.timeout(duration: Duration) -> Result<T, TimeoutError>`

**Postcondiciones:**
```
P1: Future completa antes de duration ⟹ Ok(value)
P2: Future NO completa antes de duration ⟹ Err(TimeoutError)
P3: Garantía de cancelación después de timeout
```

**Ejemplo:**
```vela
future = fetchData().timeout(Duration.seconds(5))

match future {
  Ok(data) => print("Got data: ${data}")
  Err(TimeoutError) => print("Timeout after 5s")
}
```

---

### 7. Platform-Specific Behavior

#### 7.1 File I/O

```vela
fn readFile(path: String) -> Result<String, IOError>
```

**Platform-specific:**
```
Windows:
  - Path separator: \ (backslash)
  - Line ending: \r\n (CRLF)
  - Max path: 260 caracteres (legacy) o 32767 (long paths)

Unix/Linux/macOS:
  - Path separator: / (forward slash)
  - Line ending: \n (LF)
  - Max path: 4096 caracteres

Comportamiento común:
  - Path normalizado automáticamente
  - Line endings convertidos a \n (LF) en memoria
```

---

#### 7.2 Process Spawning

```vela
fn spawn(command: String, args: List<String>) -> Result<Process, SpawnError>
```

**Platform-specific:**
```
Windows:
  - Usa CreateProcess API
  - Shell: cmd.exe
  - Environment variables case-insensitive

Unix:
  - Usa fork/exec
  - Shell: /bin/sh
  - Environment variables case-sensitive

Comportamiento común:
  - stdout/stderr capturados en UTF-8
  - Exit codes consistentes (0 = success, non-zero = error)
```

---

## 📊 Tabla de Complejidades

| Operación | Complejidad | Nota |
|-----------|-------------|------|
| **List.new()** | O(1) | - |
| **List.push()** | O(1) amortized | O(n) en resize |
| **List.get()** | O(1) | - |
| **List.map()** | O(n) | n = length |
| **List.filter()** | O(n) | - |
| **List.reduce()** | O(n) | - |
| **Map.new()** | O(1) | - |
| **Map.set()** | O(1) average | O(n) worst case |
| **Map.get()** | O(1) average | O(n) worst case |
| **Map.remove()** | O(1) average | - |
| **Option.unwrap()** | O(1) | Panic si None |
| **Option.map()** | O(1) + fn | - |
| **Result.unwrap()** | O(1) | Panic si Err |
| **Result.map()** | O(1) + fn | - |
| **String.length()** | O(1) | Cached |
| **String.concat()** | O(n + m) | - |
| **String.substring()** | O(k) | k = end - start |
| **String.trim()** | O(n) | - |
| **Future.await()** | Variable | Non-blocking |

---

## 📋 Thread-Safety Summary

| Tipo/Operación | Thread-Safety | Razón |
|----------------|---------------|-------|
| **List** | ✅ Safe | Inmutable (retorna nuevas listas) |
| **Map** | ✅ Safe | Inmutable (retorna nuevos maps) |
| **Option** | ✅ Safe | Inmutable |
| **Result** | ✅ Safe | Inmutable |
| **String** | ✅ Safe | Inmutable |
| **Future** | ✅ Safe | Non-blocking suspend |
| **Atomic<T>** | ✅ Safe | Operaciones atómicas |
| **state variables** | ✅ Safe | Sistema reactivo sincroniza |
| **Actores** | ✅ Safe | No shared state, message passing |

---

## ✅ Criterios de Aceptación

- [x] Contratos formales definidos para todas las APIs principales
- [x] Precondiciones y postcondiciones especificadas
- [x] Invariantes de tipos documentadas
- [x] Complejidades Big-O especificadas
- [x] Thread-safety guarantees documentadas
- [x] Platform-specific behavior explicado
- [x] Ejemplos de uso incluidos

---

## 🔗 Referencias

### Papers y Estándares
- **Design by Contract:** [Bertrand Meyer, 1992](https://en.wikipedia.org/wiki/Design_by_contract)
- **Hoare Logic:** [An Axiomatic Basis for Computer Programming (Hoare, 1969)](https://www.cs.cmu.edu/~crary/819-f09/Hoare69.pdf)
- **Option Type:** [Rust Option<T> documentation](https://doc.rust-lang.org/std/option/)
- **Result Type:** [Rust Result<T, E> documentation](https://doc.rust-lang.org/std/result/)

### Implementaciones de Referencia
- **Rust std::collections:** [Collections API](https://doc.rust-lang.org/std/collections/)
- **Haskell Prelude:** [Standard Library](https://hackage.haskell.org/package/base-4.17.0.0/docs/Prelude.html)
- **Swift Standard Library:** [API Design Guidelines](https://www.swift.org/documentation/api-design-guidelines/)

---

**Estado:** ⏳ Pendiente de implementación  
**Prioridad:** P0 - Crítico para desarrollo seguro  
**Siguiente paso:** Implementar stdlib con tests basados en contratos
