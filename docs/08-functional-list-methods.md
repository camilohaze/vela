# Vela - Métodos Funcionales de Listas

## Filosofía

Vela es un lenguaje **funcional puro** que **NO tiene loops tradicionales** (`for`, `while`, `loop`).

En su lugar, las listas y colecciones proveen **métodos funcionales** poderosos para iteración, transformación y filtrado.

---

## Métodos Básicos de Transformación

### `map<U>(fn: (T) -> U) -> List<U>`
Transforma cada elemento de la lista aplicando una función.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
doubled = numbers.map(x => x * 2)
# [2, 4, 6, 8, 10]

words = ["hello", "world"]
uppercased = words.map(w => w.toUpperCase())
# ["HELLO", "WORLD"]

# Mapear a otro tipo
lengths = words.map(w => w.length)  # List<Number>
# [5, 5]
```

---

### `filter(fn: (T) -> Bool) -> List<T>`
Filtra elementos que cumplen una condición.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5, 6]
evens = numbers.filter(x => x % 2 == 0)
# [2, 4, 6]

words = ["hello", "hi", "world", "hey"]
short_words = words.filter(w => w.length <= 3)
# ["hi", "hey"]

# Filtrado con múltiples condiciones
filtered = numbers.filter(x => x > 2 && x < 5)
# [3, 4]
```

---

### `reduce<U>(fn: (U, T) -> U, initial: U) -> U`
Reduce la lista a un único valor.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]

# Suma
sum = numbers.reduce((acc, x) => acc + x, 0)
# 15

# Producto
product = numbers.reduce((acc, x) => acc * x, 1)
# 120

# Concatenar strings
words = ["Hello", "Vela", "World"]
sentence = words.reduce((acc, w) => acc + " " + w, "")
# " Hello Vela World"

# Construir objeto complejo
users = [User("Alice", 25), User("Bob", 30)]
by_name = users.reduce((acc, u) => acc.set(u.name, u), {})
# { "Alice": User(...), "Bob": User(...) }
```

---

## Métodos de Iteración

### `forEach(fn: (T) -> void) -> void`
Ejecuta una función por cada elemento (side effects).

```vela
numbers: List<Number> = [1, 2, 3]
numbers.forEach(x => print(x))
# Imprime: 1, 2, 3

# Con índice usando enumerate
numbers.enumerate().forEach((index, value) => {
  print("Index ${index}: ${value}")
})
```

---

### `forEachIndexed(fn: (Number, T) -> void) -> void`
Ejecuta una función con índice y elemento.

```vela
words = ["apple", "banana", "cherry"]
words.forEachIndexed((i, word) => {
  print("${i}: ${word}")
})
# 0: apple
# 1: banana
# 2: cherry
```

---

## Métodos de Búsqueda

### `find(fn: (T) -> Bool) -> Option<T>`
Encuentra el primer elemento que cumple la condición.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
first_even = numbers.find(x => x % 2 == 0)
# Some(2)

not_found = numbers.find(x => x > 10)
# None

# Uso con pattern matching
match first_even {
  Some(n) => print("Found: ${n}")
  None => print("Not found")
}
```

---

### `findIndex(fn: (T) -> Bool) -> Option<Number>`
Encuentra el índice del primer elemento que cumple la condición.

```vela
numbers: List<Number> = [10, 20, 30, 40]
index = numbers.findIndex(x => x > 25)
# Some(2)  (el elemento 30 está en índice 2)
```

---

### `findLast(fn: (T) -> Bool) -> Option<T>`
Encuentra el último elemento que cumple la condición.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
last_even = numbers.findLast(x => x % 2 == 0)
# Some(4)
```

---

### `contains(element: T) -> Bool`
Verifica si la lista contiene un elemento.

```vela
numbers: List<Number> = [1, 2, 3]
has_two = numbers.contains(2)  # true
has_ten = numbers.contains(10)  # false
```

---

## Métodos de Verificación

### `every(fn: (T) -> Bool) -> Bool`
Verifica que **todos** los elementos cumplan la condición.

```vela
numbers: List<Number> = [2, 4, 6, 8]
all_even = numbers.every(x => x % 2 == 0)  # true

all_positive = numbers.every(x => x > 0)  # true
all_large = numbers.every(x => x > 5)  # false
```

---

### `some(fn: (T) -> Bool) -> Bool`
Verifica que **al menos uno** cumpla la condición.

```vela
numbers: List<Number> = [1, 3, 5, 6]
has_even = numbers.some(x => x % 2 == 0)  # true

has_negative = numbers.some(x => x < 0)  # false
```

---

### `none(fn: (T) -> Bool) -> Bool`
Verifica que **ninguno** cumpla la condición.

```vela
numbers: List<Number> = [1, 3, 5, 7]
no_evens = numbers.none(x => x % 2 == 0)  # true
```

---

## Métodos de Particionamiento

### `partition(fn: (T) -> Bool) -> (List<T>, List<T>)`
Divide la lista en dos según una condición.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5, 6]
(evens, odds) = numbers.partition(x => x % 2 == 0)
# evens: [2, 4, 6]
# odds: [1, 3, 5]
```

---

### `groupBy<K>(fn: (T) -> K) -> Dict<K, List<T>>`
Agrupa elementos por una clave.

```vela
words = ["apple", "banana", "cherry", "avocado", "blueberry"]
by_first_letter = words.groupBy(w => w[0])
# {
#   "a": ["apple", "avocado"],
#   "b": ["banana", "blueberry"],
#   "c": ["cherry"]
# }

numbers: List<Number> = [1, 2, 3, 4, 5, 6]
by_parity = numbers.groupBy(x => if x % 2 == 0 { "even" } else { "odd" })
# {
#   "even": [2, 4, 6],
#   "odd": [1, 3, 5]
# }
```

---

### `chunk(size: Number) -> List<List<T>>`
Divide la lista en grupos de tamaño N.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5, 6, 7]
chunks = numbers.chunk(3)
# [[1, 2, 3], [4, 5, 6], [7]]
```

---

## Métodos de Toma y Descarte

### `take(n: Number) -> List<T>`
Toma los primeros N elementos.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
first3 = numbers.take(3)
# [1, 2, 3]
```

---

### `takeLast(n: Number) -> List<T>`
Toma los últimos N elementos.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
last2 = numbers.takeLast(2)
# [4, 5]
```

---

### `takeWhile(fn: (T) -> Bool) -> List<T>`
Toma elementos mientras la condición sea verdadera.

```vela
numbers: List<Number> = [1, 2, 3, 4, 1, 2]
result = numbers.takeWhile(x => x < 4)
# [1, 2, 3]  (se detiene en 4)
```

---

### `drop(n: Number) -> List<T>`
Descarta los primeros N elementos.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
rest = numbers.drop(2)
# [3, 4, 5]
```

---

### `dropLast(n: Number) -> List<T>`
Descarta los últimos N elementos.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
without_last2 = numbers.dropLast(2)
# [1, 2, 3]
```

---

### `dropWhile(fn: (T) -> Bool) -> List<T>`
Descarta elementos mientras la condición sea verdadera.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
result = numbers.dropWhile(x => x < 3)
# [3, 4, 5]
```

---

## Métodos de Ordenamiento

### `sort() -> List<T>` (donde T: Comparable)
Ordena la lista en orden ascendente.

```vela
numbers: List<Number> = [3, 1, 4, 1, 5, 9]
sorted = numbers.sort()
# [1, 1, 3, 4, 5, 9]
```

---

### `sortBy<U>(fn: (T) -> U) -> List<T>` (donde U: Comparable)
Ordena la lista por un criterio.

```vela
words = ["banana", "apple", "cherry"]
by_length = words.sortBy(w => w.length)
# ["apple", "banana", "cherry"]

# Orden descendente
desc = numbers.sortBy(x => -x)
# [9, 5, 4, 3, 1, 1]
```

---

### `reverse() -> List<T>`
Invierte el orden de la lista.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
reversed = numbers.reverse()
# [5, 4, 3, 2, 1]
```

---

## Métodos de Aplanamiento

### `flatMap<U>(fn: (T) -> List<U>) -> List<U>`
Mapea y aplana en una sola operación.

```vela
numbers: List<Number> = [1, 2, 3]
result = numbers.flatMap(x => [x, x * 10])
# [1, 10, 2, 20, 3, 30]

words = ["hello world", "foo bar"]
all_words = words.flatMap(s => s.split(" "))
# ["hello", "world", "foo", "bar"]
```

---

### `flatten() -> List<U>` (donde T = List<U>)
Aplana una lista de listas.

```vela
nested: List<List<Number>> = [[1, 2], [3, 4], [5]]
flat = nested.flatten()
# [1, 2, 3, 4, 5]
```

---

## Métodos de Combinación

### `zip<U>(other: List<U>) -> List<(T, U)>`
Combina dos listas en tuplas.

```vela
names = ["Alice", "Bob", "Charlie"]
ages = [25, 30, 35]
pairs = names.zip(ages)
# [("Alice", 25), ("Bob", 30), ("Charlie", 35)]

# Si las listas tienen diferentes tamaños, se trunca al más corto
short = [1, 2]
long = [10, 20, 30, 40]
result = short.zip(long)
# [(1, 10), (2, 20)]
```

---

### `zipWith<U, V>(other: List<U>, fn: (T, U) -> V) -> List<V>`
Combina dos listas con una función.

```vela
a = [1, 2, 3]
b = [10, 20, 30]
sums = a.zipWith(b, (x, y) => x + y)
# [11, 22, 33]
```

---

### `concat(other: List<T>) -> List<T>`
Concatena dos listas.

```vela
a = [1, 2, 3]
b = [4, 5, 6]
combined = a.concat(b)
# [1, 2, 3, 4, 5, 6]
```

---

### `append(element: T) -> List<T>`
Agrega un elemento al final.

```vela
numbers = [1, 2, 3]
with_four = numbers.append(4)
# [1, 2, 3, 4]
```

---

### `prepend(element: T) -> List<T>`
Agrega un elemento al inicio.

```vela
numbers = [2, 3, 4]
with_one = numbers.prepend(1)
# [1, 2, 3, 4]
```

---

## Métodos de Reducción Especializados

### `sum() -> Number` (donde T = Number)
Suma todos los elementos.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
total = numbers.sum()  # 15
```

---

### `product() -> Number` (donde T = Number)
Multiplica todos los elementos.

```vela
numbers: List<Number> = [1, 2, 3, 4]
result = numbers.product()  # 24
```

---

### `min() -> Option<T>` (donde T: Comparable)
Encuentra el elemento mínimo.

```vela
numbers: List<Number> = [3, 1, 4, 1, 5]
minimum = numbers.min()  # Some(1)

empty: List<Number> = []
no_min = empty.min()  # None
```

---

### `max() -> Option<T>` (donde T: Comparable)
Encuentra el elemento máximo.

```vela
numbers: List<Number> = [3, 1, 4, 1, 5]
maximum = numbers.max()  # Some(5)
```

---

### `average() -> Option<Float>` (donde T = Number)
Calcula el promedio.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
avg = numbers.average()  # Some(3.0)

empty: List<Number> = []
no_avg = empty.average()  # None
```

---

## Métodos de Escaneo

### `scan<U>(fn: (U, T) -> U, initial: U) -> List<U>`
Como `reduce`, pero retorna todos los valores intermedios.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
cumulative = numbers.scan((acc, x) => acc + x, 0)
# [0, 1, 3, 6, 10, 15]

# Útil para calcular sumas acumulativas
running_total = [10, 20, 30].scan((acc, x) => acc + x, 0)
# [0, 10, 30, 60]
```

---

## Métodos de Ventana

### `windows(size: Number) -> List<List<T>>`
Crea ventanas deslizantes de tamaño N.

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]
windows = numbers.windows(3)
# [[1, 2, 3], [2, 3, 4], [3, 4, 5]]
```

---

## Métodos de Unicidad

### `distinct() -> List<T>` (donde T: Hashable)
Elimina duplicados (preserva orden de primera aparición).

```vela
numbers: List<Number> = [1, 2, 2, 3, 3, 3, 4]
unique = numbers.distinct()
# [1, 2, 3, 4]
```

---

### `distinctBy<K>(fn: (T) -> K) -> List<T>` (donde K: Hashable)
Elimina duplicados según un criterio.

```vela
words = ["hello", "world", "hi", "hey"]
by_length = words.distinctBy(w => w.length)
# ["hello", "hi"]  (preserva primer elemento de cada longitud)
```

---

## Métodos de Acceso

### `get(index: Number) -> Option<T>`
Obtiene elemento por índice (seguro).

```vela
numbers: List<Number> = [10, 20, 30]
second = numbers.get(1)  # Some(20)
invalid = numbers.get(10)  # None
```

---

### `first() -> Option<T>`
Obtiene el primer elemento.

```vela
numbers: List<Number> = [1, 2, 3]
first = numbers.first()  # Some(1)

empty: List<Number> = []
no_first = empty.first()  # None
```

---

### `last() -> Option<T>`
Obtiene el último elemento.

```vela
numbers: List<Number> = [1, 2, 3]
last = numbers.last()  # Some(3)
```

---

## Métodos de Tamaño

### `length() -> Number`
Retorna el número de elementos.

```vela
numbers: List<Number> = [1, 2, 3]
size = numbers.length()  # 3
```

---

### `isEmpty() -> Bool`
Verifica si la lista está vacía.

```vela
empty: List<Number> = []
is_empty = empty.isEmpty()  # true

numbers: List<Number> = [1, 2, 3]
is_empty2 = numbers.isEmpty()  # false
```

---

## Encadenamiento de Métodos (Method Chaining)

Todos estos métodos se pueden encadenar para crear **pipelines funcionales**:

```vela
# Ejemplo complejo: procesamiento de datos
users = [
  User("Alice", 25, "Engineering"),
  User("Bob", 30, "Sales"),
  User("Charlie", 35, "Engineering"),
  User("Diana", 28, "Sales"),
  User("Eve", 32, "Engineering")
]

# Pipeline funcional
result = users
  .filter(u => u.department == "Engineering")  # Solo ingenieros
  .map(u => u.age)                             # Extraer edades
  .filter(age => age > 26)                     # Mayores de 26
  .sortBy(age => age)                          # Ordenar
  .take(2)                                     # Tomar primeros 2
  .map(age => age * 12)                        # Convertir a meses
  .sum()                                       # Sumar todo

# result: suma de edades (en meses) de los 2 ingenieros más jóvenes mayores de 26
```

---

## Rangos como Listas

Los rangos son listas lazy que se pueden usar con todos estos métodos:

```vela
# Rango exclusivo
(0..10)
  .filter(x => x % 2 == 0)
  .map(x => x * x)
  .forEach(x => print(x))
# Imprime: 0, 4, 16, 36, 64

# Rango inclusivo
(1..=5)
  .map(x => x * 10)
  .reduce((a, b) => a + b, 0)
# 150

# Generar lista desde rango
numbers = (0..10).toList()  # [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
```

---

## Reemplazo de Loops Tradicionales

### ❌ Antes (con loops - NO PERMITIDO en Vela):

```javascript
// ❌ Este código NO es válido en Vela
let sum = 0
for (let i = 0; i < numbers.length; i++) {
  sum += numbers[i]
}
```

### ✅ Ahora (funcional - CORRECTO en Vela):

```vela
# ✅ Código válido en Vela
sum = numbers.reduce((acc, x) => acc + x, 0)

# O más simple:
sum = numbers.sum()
```

---

### ❌ Antes (while loop - NO PERMITIDO):

```javascript
// ❌ Este código NO es válido en Vela
let count = 0
while (count < 10) {
  console.log(count)
  count++
}
```

### ✅ Ahora (funcional - CORRECTO):

```vela
# ✅ Código válido en Vela
(0..10).forEach(i => print(i))

# O con recursión (tail-call optimizada):
fn printNumbers(n: Number, max: Number) -> void {
  if n < max {
    print(n)
    printNumbers(n + 1, max)
  }
}
printNumbers(0, 10)
```

---

## Performance y Lazy Evaluation

Muchos métodos son **lazy** (evaluación perezosa), lo que significa que no ejecutan hasta que se necesita el resultado:

```vela
# Esto NO ejecuta map/filter inmediatamente
lazy = numbers
  .map(x => expensiveOperation(x))
  .filter(x => x > 100)

# Se ejecuta SOLO cuando se consume:
result = lazy.take(1)  # Solo procesa hasta obtener 1 elemento
```

---

## Resumen

Vela elimina completamente los loops tradicionales en favor de:

✅ **Métodos funcionales** (map, filter, reduce, etc.)  
✅ **Encadenamiento** (method chaining)  
✅ **Inmutabilidad** (cada operación retorna nueva lista)  
✅ **Lazy evaluation** (evaluación perezosa cuando es posible)  
✅ **Recursión tail-call optimizada** (para casos avanzados)  

Esto hace el código más:
- **Declarativo**: qué hacer, no cómo hacerlo
- **Seguro**: sin mutaciones inesperadas
- **Composable**: funciones se combinan fácilmente
- **Legible**: intención clara
