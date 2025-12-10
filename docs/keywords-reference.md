# Vela - Palabras Reservadas (Keywords)

Referencia completa de todas las palabras clave del lenguaje Vela, su propósito y uso.

---

## 1. Declaraciones de Variables

### `state`
**Propósito**: Declara una variable **mutable** y **reactiva**.

**Uso**:
```vela
state count: Number = 0
count = count + 1  # OK, es mutable
```

**Contexto**: Se usa para estado local que puede cambiar. **Única forma de tener mutabilidad**.

---

### Declaración Inmutable (por defecto)
**Propósito**: Todas las variables son **inmutables por defecto** sin necesidad de keywords.

**Uso**:
```vela
# Inmutable por defecto (NO necesita const ni let)
name: String = "Cristian"
age: Number = 37
PI: Float = 3.14159

# name = "Otro"  # ERROR: inmutable por defecto

# Para mutabilidad, usar state
state counter: Number = 0
counter = counter + 1  # OK
```

**Nota**: Al ser inmutable por defecto, **no existen** las palabras `const` ni `let`.

---

## 2. Tipos de Datos

### `Number`
**Propósito**: Tipo numérico entero.

**Uso**:
```vela
age: Number = 37
count: Number = 0
```

---

### `Float`
**Propósito**: Tipo numérico de punto flotante.

**Uso**:
```vela
price: Float = 19.99
pi: Float = 3.14159
```

---

### `String`
**Propósito**: Tipo cadena de texto.

**Uso**:
```vela
name: String = "Cristian"
message: String = "Hello, ${name}!"  # interpolación
```

---

### `Bool`
**Propósito**: Tipo booleano.

**Uso**:
```vela
isActive: Bool = true
isEnabled: Bool = false
```

---

### `void`
**Propósito**: Indica que una función no retorna valor.

**Uso**:
```vela
fn greet() -> void {
  print("Hello!")
}
```

---

### `never`
**Propósito**: Tipo para funciones que nunca retornan (lanza excepción o loop infinito).

**Uso**:
```vela
fn panic(message: String) -> never {
  throw Error(message)
}
```

---

## 3. Estructuras de Datos

### `type`
**Propósito**: Define un **alias de tipo** o **type union**.

**Uso**:
```vela
# Alias
type UserId = Number

# Union type
type Status = "pending" | "active" | "completed"

# Structural type
type Point = {
  x: Number
  y: Number
}
```

---

### `enum`
**Propósito**: Define un tipo enumerado.

**Uso**:
```vela
enum Color {
  Red
  Green
  Blue
  Custom(r: Number, g: Number, b: Number)
}

color: Color = Color.Red
custom: Color = Color.Custom(255, 128, 0)
```

---

### `struct`
**Propósito**: Define una estructura de datos (similar a un record).

**Uso**:
```vela
struct User {
  id: Number
  name: String
  email: String
}

user: User = User(id: 1, name: "Cristian", email: "c@example.com")
```

---

## 4. Programación Orientada a Objetos

### `class`
**Propósito**: Define una clase.

**Uso**:
```vela
class Person {
  name: String
  age: Number
  
  constructor(name: String, age: Number) {
    this.name = name
    this.age = age
  }
  
  fn greet() -> String {
    return "Hello, I'm ${this.name}"
  }
}
```

---

### `abstract`
**Propósito**: Define una clase abstracta (no se puede instanciar directamente).

**Uso**:
```vela
abstract class Shape {
  abstract fn area() -> Float
  
  fn describe() -> String {
    return "Shape with area ${this.area()}"
  }
}

class Circle extends Shape {
  radius: Float
  
  override fn area() -> Float {
    return 3.14159 * this.radius * this.radius
  }
}
```

---

### `interface`
**Propósito**: Define un contrato de tipo (métodos que debe implementar una clase).

**Uso**:
```vela
interface Drawable {
  fn draw() -> void
}

interface Clickable {
  fn onClick() -> void
}

class Button implements Drawable, Clickable {
  fn draw() -> void {
    # implementación
  }
  
  fn onClick() -> void {
    # implementación
  }
}
```

---

### `extends`
**Propósito**: Herencia de clases.

**Uso**:
```vela
class Animal {
  name: String
}

class Dog extends Animal {
  breed: String
}
```

---

### `implements`
**Propósito**: Implementación de interfaces.

**Uso**:
```vela
class MyButton implements Drawable, Clickable {
  # debe implementar todos los métodos de las interfaces
}
```

---

### `override`
**Propósito**: Marca explícitamente que un método sobrescribe uno de la clase padre.

**Uso**:
```vela
class Parent {
  fn greet() -> String {
    return "Hello from Parent"
  }
}

class Child extends Parent {
  override fn greet() -> String {
    return "Hello from Child"
  }
}
```

---

### `overload`
**Propósito**: Declara sobrecarga de métodos (múltiples firmas para el mismo método).

**Uso**:
```vela
class Calculator {
  overload fn add(a: Number, b: Number) -> Number {
    return a + b
  }
  
  overload fn add(a: Float, b: Float) -> Float {
    return a + b
  }
  
  overload fn add(numbers: List<Number>) -> Number {
    return numbers.reduce((acc, n) => acc + n, 0)
  }
}
```

---

### `this`
**Propósito**: Referencia a la instancia actual de una clase.

**Uso**:
```vela
class Counter {
  state count: Number = 0
  
  fn increment() -> void {
    this.count += 1
  }
}
```

---

### `super`
**Propósito**: Referencia a la clase padre.

**Uso**:
```vela
class Child extends Parent {
  override fn greet() -> String {
    return super.greet() + " and Child"
  }
}
```

---

### `constructor`
**Propósito**: Define el constructor de una clase.

**Uso**:
```vela
class User {
  name: String
  age: Number
  
  constructor(name: String, age: Number) {
    this.name = name
    this.age = age
  }
}
```

---

## 5. Funciones

### `fn`
**Propósito**: Declara una función.

**Uso**:
```vela
fn add(a: Number, b: Number) -> Number {
  return a + b
}

# Función anónima
callback: (Number) -> Number = fn(x: Number) -> Number {
  return x * 2
}

# Arrow function (sintaxis corta)
double = (x: Number) => x * 2
```

---

### `async`
**Propósito**: Marca una función como asíncrona.

**Uso**:
```vela
async fn fetchData(url: String) -> Result<String, Error> {
  response = await http.get(url)
  return response
}
```

---

### `await`
**Propósito**: Espera el resultado de una operación asíncrona.

**Uso**:
```vela
async fn main() {
  data = await fetchData("https://api.example.com/data")
  print(data)
}
```

---

### `return`
**Propósito**: Retorna un valor de una función.

**Uso**:
```vela
fn square(x: Number) -> Number {
  return x * x
}
```

---

### `yield`
**Propósito**: Produce un valor en un generador (función que retorna múltiples valores).

**Uso**:
```vela
fn* fibonacci() -> Generator<Number> {
  a: Number = 0
  b: Number = 1
  loop {
    yield a
    temp = a
    a = b
    b = temp + b
  }
}
```

---

## 6. Control de Flujo

### `if`
**Propósito**: Condicional if.

**Uso**:
```vela
if age >= 18 {
  print("Adult")
} else if age >= 13 {
  print("Teenager")
} else {
  print("Child")
}

# If expression (retorna valor)
status: String = if isActive { "active" } else { "inactive" }
```

---

### `else`
**Propósito**: Rama alternativa de un `if`.

**Uso**:
```vela
if condition {
  # ...
} else {
  # ...
}
```

---

### `match`
**Propósito**: Pattern matching (similar a switch pero más poderoso).

**Uso**:
```vela
result: Result<Number, Error> = getResult()

match result {
  Ok(value) => print("Success: ${value}")
  Err(error) => print("Error: ${error}")
}

# Match con valores
match status {
  "pending" => print("Waiting...")
  "active" => print("Running")
  "completed" => print("Done!")
  _ => print("Unknown status")
}

# Match con guards
match number {
  n if n < 0 => print("Negative")
  n if n == 0 => print("Zero")
  n if n > 0 => print("Positive")
}
```

---

### ❌ `for` / `while` / `loop` (NO EXISTEN)
**Razón**: Vela es **funcional puro**. No hay loops tradicionales.

**Alternativa - Métodos Funcionales en Listas**:
```vela
# ❌ NO: for loop
# for i in 0..10 { print(i) }

# ✅ SÍ: forEach
(0..10).forEach(i => print(i))

# ❌ NO: while loop
# while condition { doSomething() }

# ✅ SÍ: recursión o métodos funcionales
fn repeatUntil(condition: () -> Bool, action: () -> void) -> void {
  if !condition() {
    action()
    repeatUntil(condition, action)
  }
}

# ❌ NO: loop infinito
# loop { processNext() }

# ✅ SÍ: recursión tail-call optimizada
fn processForever() -> never {
  processNext()
  processForever()
}
```

---

### Métodos Funcionales de Listas

Las listas tienen métodos funcionales avanzados que reemplazan loops:

```vela
numbers: List<Number> = [1, 2, 3, 4, 5]

# map: transformar cada elemento
doubled = numbers.map(x => x * 2)  # [2, 4, 6, 8, 10]

# filter: filtrar elementos
evens = numbers.filter(x => x % 2 == 0)  # [2, 4]

# reduce: reducir a un valor
sum = numbers.reduce((acc, x) => acc + x, 0)  # 15

# forEach: ejecutar acción por cada elemento
numbers.forEach(x => print(x))

# flatMap: mapear y aplanar
nested = [[1, 2], [3, 4]]
flat = nested.flatMap(x => x)  # [1, 2, 3, 4]

# take: tomar primeros N elementos
first3 = numbers.take(3)  # [1, 2, 3]

# drop: saltar primeros N elementos
rest = numbers.drop(2)  # [3, 4, 5]

# takeWhile: tomar mientras condición sea true
lessThan4 = numbers.takeWhile(x => x < 4)  # [1, 2, 3]

# dropWhile: saltar mientras condición sea true
from3 = numbers.dropWhile(x => x < 3)  # [3, 4, 5]

# find: encontrar primer elemento que cumple condición
first_even = numbers.find(x => x % 2 == 0)  # Some(2)

# findIndex: encontrar índice del primer match
index = numbers.findIndex(x => x > 3)  # Some(3)

# every: verificar que todos cumplen condición
all_positive = numbers.every(x => x > 0)  # true

# some: verificar que al menos uno cumple condición
has_even = numbers.some(x => x % 2 == 0)  # true

# partition: dividir en dos listas según condición
(evens, odds) = numbers.partition(x => x % 2 == 0)

# groupBy: agrupar por clave
words = ["hello", "hi", "world", "hey"]
by_length = words.groupBy(w => w.length)
# { 2: ["hi"], 3: ["hey"], 5: ["hello", "world"] }

# sortBy: ordenar por criterio
sorted = numbers.sortBy(x => -x)  # orden descendente

# chunk: dividir en grupos de tamaño N
chunks = numbers.chunk(2)  # [[1, 2], [3, 4], [5]]

# zip: combinar dos listas
names = ["Alice", "Bob"]
ages = [25, 30]
pairs = names.zip(ages)  # [("Alice", 25), ("Bob", 30)]

# scan: como reduce pero retorna pasos intermedios
cumulative = numbers.scan((acc, x) => acc + x, 0)
# [0, 1, 3, 6, 10, 15]

# distinct: eliminar duplicados
unique = [1, 2, 2, 3, 3, 3].distinct()  # [1, 2, 3]

# reverse: invertir orden
reversed = numbers.reverse()  # [5, 4, 3, 2, 1]
```

---

## 7. Manejo de Errores

### `try`
**Propósito**: Bloque para manejo de errores.

**Uso**:
```vela
try {
  data = parseJSON(text)
  process(data)
} catch (error: ParseError) {
  print("Parse error: ${error}")
} catch (error: Error) {
  print("General error: ${error}")
} finally {
  cleanup()
}
```

---

### `catch`
**Propósito**: Captura excepciones en un bloque `try`.

**Uso**:
```vela
try {
  riskyOperation()
} catch (error: MyError) {
  handleError(error)
}
```

---

### `throw`
**Propósito**: Lanza una excepción.

**Uso**:
```vela
fn divide(a: Number, b: Number) -> Float {
  if b == 0 {
    throw Error("Division by zero")
  }
  return a / b
}
```

---

### `finally`
**Propósito**: Bloque que siempre se ejecuta después de try/catch.

**Uso**:
```vela
try {
  file = openFile("data.txt")
  process(file)
} catch (error) {
  print("Error: ${error}")
} finally {
  file.close()  # siempre se ejecuta
}
```

---

## 8. Imports y Módulos

### `import`
**Propósito**: Importa módulos, paquetes, bibliotecas.

**Uso**:
```vela
# Import completo
import 'package:http'
import 'library:math'
import 'module:auth'

# Import selectivo
import 'package:utils' show { HashMap, ArrayList }

# Import con exclusiones
import 'library:math' hide { deprecated_function }

# Import con alias
import 'package:long_name' as ln
```

---

### ❌ `export` (NO EXISTE)
**Razón**: La visibilidad se controla con **modificadores de acceso**.

**Alternativa**:
```vela
# ❌ NO: export
# export fn myFunction() { }

# ✅ SÍ: modificador public (por defecto en módulos)
public fn myFunction() -> void {
  # ...
}

# Privado por defecto en el módulo
private fn internalHelper() -> void {
  # solo accesible dentro del módulo
}
```

---

### `show`
**Propósito**: Importa elementos específicos de un módulo.

**Uso**:
```vela
import 'package:utils' show { HashMap, sort, filter }
```

---

### `hide`
**Propósito**: Importa todo excepto elementos específicos.

**Uso**:
```vela
import 'library:math' hide { deprecated_sqrt }
```

---

### `as`
**Propósito**: Alias para imports.

**Uso**:
```vela
import 'package:very_long_package_name' as vlpn
import 'assets:config.json' as config

vlpn.doSomething()
print(config.version)
```

---

## 9. Modificadores de Acceso

### `public`
**Propósito**: Marca algo como públicamente accesible (por defecto en exports).

**Uso**:
```vela
public class PublicClass {
  public fn publicMethod() -> void {
    # accesible desde fuera
  }
}
```

---

### `private`
**Propósito**: Marca algo como privado (solo accesible dentro de la clase/módulo).

**Uso**:
```vela
class MyClass {
  private state internalState: Number = 0
  
  private fn helperMethod() -> void {
    # solo accesible dentro de MyClass
  }
}
```

---

### `protected`
**Propósito**: Accesible en la clase y sus subclases.

**Uso**:
```vela
class Parent {
  protected fn protectedMethod() -> void {
    # accesible en Parent y subclases
  }
}

class Child extends Parent {
  fn useProtected() -> void {
    this.protectedMethod()  # OK
  }
}
```

---

## 10. Hooks Reactivos

### `computed`
**Propósito**: Define un valor derivado que se recalcula automáticamente.

**Uso**:
```vela
component Counter {
  state count: Number = 0
  
  computed doubled: Number {
    return this.count * 2
  }
  
  computed isEven: Bool {
    return this.count % 2 == 0
  }
}
```

---

### `memo`
**Propósito**: Similar a `computed`, pero con caché más agresivo para cálculos costosos.

**Uso**:
```vela
component DataProcessor {
  state data: List<Number> = []
  
  memo expensiveCalculation: Number {
    # Este cálculo solo se ejecuta si data cambia
    return this.data
      .map(x => x * x)
      .filter(x => x > 100)
      .reduce((a, b) => a + b, 0)
  }
}
```

---

### `effect`
**Propósito**: Define un side effect reactivo.

**Uso**:
```vela
component Logger {
  state count: Number = 0
  
  effect {
    # Se ejecuta cuando count cambia
    print("Count changed to: ${this.count}")
    
    # Cleanup opcional
    return () => {
      print("Cleaning up old effect")
    }
  }
}
```

---

### `watch`
**Propósito**: Observa cambios en una expresión específica.

**Uso**:
```vela
component Watcher {
  state firstName: String = ""
  state lastName: String = ""
  
  watch(this.firstName) {
    print("First name changed: ${this.firstName}")
  }
  
  watch([this.firstName, this.lastName]) {
    print("Full name: ${this.firstName} ${this.lastName}")
  }
}
```

---

## 11. Ciclo de Vida de Componentes

### `mount`
**Propósito**: Hook que se ejecuta cuando el componente se monta.

**Uso**:
```vela
component MyComponent {
  mount() {
    print("Component mounted")
    this.fetchData()
  }
}
```

---

### `update`
**Propósito**: Hook que se ejecuta después de cada actualización.

**Uso**:
```vela
component MyComponent {
  update() {
    print("Component updated")
  }
}
```

---

### `destroy`
**Propósito**: Hook que se ejecuta cuando el componente se desmonta.

**Uso**:
```vela
component MyComponent {
  destroy() {
    print("Component destroyed - cleanup")
    this.cancelRequests()
  }
}
```

---

### `beforeUpdate`
**Propósito**: Se ejecuta antes de aplicar cambios al DOM.

**Uso**:
```vela
component MyComponent {
  beforeUpdate() {
    print("About to update DOM")
  }
}
```

---

### `afterUpdate`
**Propósito**: Se ejecuta después de aplicar cambios al DOM.

**Uso**:
```vela
component MyComponent {
  afterUpdate() {
    print("DOM updated")
  }
}
```

---

## 12. UI - Widgets (como Flutter)

### `StatefulWidget`
**Propósito**: Widget con estado mutable (similar a Flutter).

**Uso**:
```vela
widget Counter {
  # Estado local
  state count: Number = 0
  
  # Computed values
  computed isEven: Bool {
    return this.count % 2 == 0
  }
  
  # Métodos
  fn increment() -> void {
    this.count += 1
  }
  
  # Lifecycle hooks
  mount() {
    print("Counter mounted")
  }
  
  update() {
    print("Counter updated")
  }
  
  destroy() {
    print("Counter destroyed")
  }
  
  # Build method (obligatorio)
  fn build() -> Widget {
    return Column(
      children: [
        Text("Count: ${this.count}"),
        Text(this.isEven ? "Even" : "Odd"),
        Button(
          text: "Increment",
          onClick: this.increment
        )
      ]
    )
  }
}
```

---

### `StatelessWidget`
**Propósito**: Widget sin estado (inmutable, solo presentacional).

**Uso**:
```vela
component Card {
  # Propiedades inmutables (pasadas en constructor)
  title: String
  content: String
  color: Color
  
  constructor(title: String, content: String, color: Color) {
    this.title = title
    this.content = content
    this.color = color
  }
  
  # Build method (obligatorio)
  fn build() -> Widget {
    return Container(
      padding: EdgeInsets.all(16),
      backgroundColor: this.color,
      child: Column(
        children: [
          Text(
            this.title,
            style: TextStyle(
              fontSize: 20,
              fontWeight: FontWeight.bold
            )
          ),
          SizedBox(height: 8),
          Text(this.content)
        ]
      )
    )
  }
}

# Uso
card = Card(
  title: "Welcome",
  content: "Hello Vela!",
  color: Color.blue
)
```

---

### Comparación: StatefulWidget vs StatelessWidget

```vela
# StatefulWidget: tiene estado mutable
widget TodoList {
  state todos: List<String> = []
  
  fn addTodo(text: String) -> void {
    this.todos = this.todos.append(text)  # muta el estado
  }
  
  fn build() -> Widget {
    return ListView(
      children: this.todos.map(todo => Text(todo))
    )
  }
}

# StatelessWidget: sin estado, puro
component TodoItem {
  text: String
  isCompleted: Bool
  
  constructor(text: String, isCompleted: Bool) {
    this.text = text
    this.isCompleted = isCompleted
  }
  
  fn build() -> Widget {
    return Row(
      children: [
        Checkbox(value: this.isCompleted),
        Text(this.text)
      ]
    )
  }
}
```

---

## 13. Concurrencia (Actors)

### `actor`
**Propósito**: Define un actor para concurrencia con aislamiento de memoria.

**Uso**:
```vela
actor Counter {
  private state count: Number = 0
  
  fn increment() -> void {
    this.count += 1
  }
  
  fn getCount() -> Number {
    return this.count
  }
}

# Uso
counter = Counter.spawn()
await counter.send("increment")
value = await counter.call("getCount")
```

---

### `spawn`
**Propósito**: Crea una nueva instancia de actor.

**Uso**:
```vela
actor Worker {
  fn process(data: String) -> String {
    return data.toUpperCase()
  }
}

worker = Worker.spawn()
result = await worker.call("process", "hello")
```

---

### `send`
**Propósito**: Envía un mensaje a un actor (fire-and-forget).

**Uso**:
```vela
await myActor.send("doSomething")
```

---

### `call`
**Propósito**: Envía un mensaje a un actor y espera respuesta.

**Uso**:
```vela
result = await myActor.call("methodName", arg1, arg2)
```

---

## 14. Genéricos

### `<T>`
**Propósito**: Define parámetros de tipo genérico.

**Uso**:
```vela
# Función genérica
fn identity<T>(value: T) -> T {
  return value
}

# Clase genérica
class Box<T> {
  value: T
  
  constructor(value: T) {
    this.value = value
  }
  
  fn get() -> T {
    return this.value
  }
}

# Type alias genérico
type Result<T, E> = Ok(T) | Err(E)

# Interface genérica
interface Container<T> {
  fn add(item: T) -> void
  fn get(index: Number) -> T
}
```

---

### `where`
**Propósito**: Define constraints en tipos genéricos.

**Uso**:
```vela
fn compare<T>(a: T, b: T) -> Bool where T: Comparable {
  return a < b
}

class SortedList<T> where T: Comparable, T: Hashable {
  items: List<T> = []
  
  fn add(item: T) -> void {
    # ...
  }
}
```

---

## 15. Decoradores

### `@package`
**Propósito**: Define el nombre del paquete.

**Uso**:
```vela
@package("custom-http")
```

---

---

### `@library`
**Propósito**: Define una biblioteca.

**Uso**:
```vela
@library("utils")
```

---

### `@extension`
**Propósito**: Define una extensión.

**Uso**:
```vela
@extension("charts")
```

---

### `@injectable`
**Propósito**: Marca una clase como inyectable en el sistema de DI (Dependency Injection).

**Uso**:
```vela
@injectable
class UserService {
  constructor(@inject api: ApiClient) {
    # ...
  }
}

# Con scope
@injectable(scope: Scope.Singleton)
class ConfigService {
  # Una sola instancia global
}
```

---

### `@inject`
**Propósito**: Marca una propiedad o parámetro para inyección de dependencias.

**Uso**:
```vela
class UserController {
  @inject
  userService: UserService
  
  constructor(@inject db: Database) {
    # ...
  }
}
```

---

### `@provides`
**Propósito**: Marca un método que provee una dependencia en un módulo de DI.

**Uso**:
```vela
@module({
  name: "NetworkModule",
  providers: [HttpClient],
  exports: [HttpClient]
})
module NetworkModule {
  @provides
  fn provideHttpClient() -> HttpClient {
    return HttpClient()
  }
}
```

---

### `@connect`
**Propósito**: Conecta un widget a un Store global de state management.

**Uso**:
```vela
widget UserProfile {
  @connect(AppStore)
  store: AppStore
  
  fn build() -> Widget {
    return Text("User: ${this.store.user}")
  }
}
```

---

### `@select`
**Propósito**: Selecciona una parte específica del store para re-render optimizado.

**Uso**:
```vela
widget UserWidget {
  @connect(AppStore)
  store: AppStore
  
  # Solo re-render si cambia el user
  @select((store) => store.user)
  user: Option<User>
  
  fn build() -> Widget {
    return Text("User: ${this.user}")
  }
}
```

---

### `@persistent`
**Propósito**: Marca un Store para persistencia automática (localStorage/SharedPreferences).

**Uso**:
```vela
@persistent(key: "app_state")
class AppStore extends Store {
  state user: Option<User> = None
  # Se guarda/carga automáticamente
}
```

---

### `@deprecated`
**Propósito**: Marca algo como obsoleto.

**Uso**:
```vela
@deprecated("Use newFunction instead")
fn oldFunction() -> void {
  # ...
}
```

---

### `@test`
**Propósito**: Marca una función como test.

**Uso**:
```vela
@test
fn testAddition() {
  result = add(2, 3)
  expect(result).toBe(5)
}
```

---

## 16. Operadores Especiales

### `?.` (Safe navigation)
**Propósito**: Acceso seguro a propiedades (retorna None si es null).

**Uso**:
```vela
user: Option<User> = getUser()
name: Option<String> = user?.name
email: Option<String> = user?.profile?.email
```

---

### `??` (Null coalescing)
**Propósito**: Valor por defecto si es None/null.

**Uso**:
```vela
name: String = user?.name ?? "Guest"
```

---

### `!` (Non-null assertion)
**Propósito**: Afirma que un valor no es null (puede lanzar error en runtime).

**Uso**:
```vela
user: Option<User> = getUser()
name: String = user!.name  # Peligroso: puede fallar si user es None
```

---

### `=>` (Arrow function)
**Propósito**: Sintaxis corta para funciones.

**Uso**:
```vela
double = (x: Number) => x * 2

numbers.map(x => x * 2)

button.onClick(() => print("Clicked!"))
```

---

### `..` (Range operator)
**Propósito**: Crea un rango (inclusivo en el inicio, exclusivo en el final).

**Uso**:
```vela
for i in 0..10 {  # 0, 1, 2, ..., 9
  print(i)
}
```

---

### `..=` (Inclusive range)
**Propósito**: Crea un rango inclusivo en ambos extremos.

**Uso**:
```vela
for i in 0..=10 {  # 0, 1, 2, ..., 10
  print(i)
}
```

---

### `|` (Pipe operator)
**Propósito**: Composición de funciones (pipe).

**Uso**:
```vela
result = data
  |> filter(x => x > 10)
  |> map(x => x * 2)
  |> reduce((a, b) => a + b, 0)
```

---

## 17. Palabras de Contexto

### `in`
**Propósito**: Usado en loops para iterar sobre colecciones.

**Uso**:
```vela
for item in items {
  print(item)
}
```

---

### `is`
**Propósito**: Type checking en runtime.

**Uso**:
```vela
if value is String {
  print("It's a string: ${value}")
}

match result {
  x if x is Number => print("Number: ${x}")
  x if x is String => print("String: ${x}")
}
```

---

### `new`
**Propósito**: Crea una nueva instancia (opcional, puede omitirse).

**Uso**:
```vela
# Con new (opcional)
user = new User("Cristian", 37)

# Sin new (recomendado)
user = User("Cristian", 37)
```

---

### `true` / `false`
**Propósito**: Literales booleanos.

**Uso**:
```vela
isActive: Bool = true
isDisabled: Bool = false
```

---

### `_` (Underscore)
**Propósito**: Placeholder para valores ignorados.

**Uso**:
```vela
# En pattern matching
match result {
  Ok(value) => print(value)
  _ => print("Error")  # catch-all
}

# En destructuring
(first, _, third) = tuple
```

---

## 18. ADTs y Pattern Matching

### `Some` / `None`
**Propósito**: Variantes del tipo `Option<T>`.

**Uso**:
```vela
value: Option<String> = Some("hello")
empty: Option<String> = None

match value {
  Some(v) => print("Value: ${v}")
  None => print("No value")
}
```

---

### `Ok` / `Err`
**Propósito**: Variantes del tipo `Result<T, E>`.

**Uso**:
```vela
result: Result<Number, String> = Ok(42)
error: Result<Number, String> = Err("Something went wrong")

match result {
  Ok(value) => print("Success: ${value}")
  Err(error) => print("Error: ${error}")
}
---

### `namespace`
**Propósito**: Agrupa definiciones bajo un nombre común.

**Uso**:
```vela
namespace Math {
  const PI: Float = 3.14159
  
  fn sqrt(x: Float) -> Float {
    # ...
  }
}

# Uso
value = Math.sqrt(16)
```

---

### `static`
**Propósito**: Define miembros de clase (no de instancia).

**Uso**:
```vela
class MathUtils {
  static fn add(a: Number, b: Number) -> Number {
    return a + b
  }
}

# Uso
result = MathUtils.add(2, 3)  # no necesita instancia
```

---

### `get` / `set`
**Propósito**: Define getters y setters para propiedades computadas.

**Uso**:
```vela
class Person {
  private state _age: Number = 0
  
  get age() -> Number {
    return this._age
  }
  
  set age(value: Number) {
    if value >= 0 {
      this._age = value
    }
  }
}

person = Person()
person.age = 37  # llama al setter
print(person.age)  # llama al getter
```

---

### `with`
**Propósito**: Copia inmutable con cambios (spread de objetos).

**Uso**:
```vela
user = User(name: "Cristian", age: 37)
updatedUser = user with { age: 38 }

# user.age sigue siendo 37
# updatedUser.age es 38
```

---

## Resumen

Vela tiene **~100 palabras reservadas** que cubren:

- ✅ Declaración de variables (**solo `state`** para mutabilidad, inmutable por defecto sin keywords)
- ✅ Tipos primitivos y compuestos (Number, String, Bool, enum, struct, type)
- ✅ OOP (class, abstract, interface, extends, implements, override, overload)
- ✅ Funciones (fn, async, await, return, yield)
- ✅ Control de flujo (if, else, match) - **❌ SIN loops tradicionales (for/while)**
- ✅ **Programación funcional**: métodos de listas (map, filter, reduce, forEach, etc.)
- ✅ Manejo de errores (try, catch, throw, finally)
- ✅ Imports (import, show, hide, as) - **❌ SIN export** (usar decoradores)
- ✅ Modificadores de acceso (public, private, protected)
- ✅ Hooks reactivos (computed, memo, effect, watch)
- ✅ Ciclo de vida (mount, update, destroy, beforeUpdate, afterUpdate)
- ✅ UI (**widget, component** - como Flutter)
- ✅ Concurrencia (actor, spawn, send, call)
- ✅ Genéricos (<T>, where)
- ✅ Decoradores (@package, @module, @library, @test, @deprecated)
- ✅ Operadores especiales (?., ??, !, =>, .., ..=, |>)
- ✅ ADTs (Some, None, Ok, Err) - **❌ SIN null**

## Filosofía de Vela

Este sistema de palabras clave hace que Vela sea:

- **Funcional puro**: sin loops tradicionales, solo métodos funcionales
- **Inmutable por defecto**: sin `const`/`let`, todo inmutable salvo `state`
- **Null-safety total**: `null` no existe, solo `Option<T>`
- **Expresivo**: sintaxis clara inspirada en TypeScript + Flutter
- **Seguro**: type-safety, immutability, memory-safety
- **Moderno**: async/await, pattern matching, generics
- **Reactivo**: signals integrados en el lenguaje
- **Multiplataforma**: abstracciones para web, móvil, desktop

## Palabras Clave Eliminadas

Para mantener la pureza funcional y simplicidad:

- ❌ `const`, `let` → inmutabilidad por defecto
- ❌ `for`, `while`, `loop`, `break`, `continue` → métodos funcionales
- ❌ `export`, `from` → modificadores de acceso (`public`, `private`, `protected`)
- ❌ `null` → solo `Option<T>` con `Some`/`None`
- ❌ `any` → tipado fuerte obligatorio
- ❌ `@override` → keyword `override` es suficiente
- ❌ `component`, `widget` → `StatefulWidget`, `StatelessWidget`

---

## Nuevas Palabras Clave Propuestas (MVP 1.0)

Para completar las características críticas de Vela, se proponen las siguientes extensiones:

### Sistema de Inyección de Dependencias (DI)

Estas palabras clave permitirán un sistema DI completo y type-safe:

| Palabra Clave | Tipo | Propósito |
|---------------|------|-----------|
| `@injectable` | Decorator | Marca una clase como inyectable en el contenedor DI |
| `@inject` | Decorator | Marca un parámetro como dependencia a inyectar |
| `@container` | Decorator | Define contenedor DI que agrupa providers (estándar Spring/Angular/NestJS) |
| `@provides` | Decorator | Marca un método factory que provee una dependencia |
| `@controller` | Decorator | Define controlador REST/API con routing automático |

**Ejemplo DI completo**:
```vela
# Definir servicio inyectable
@injectable(scope: Scope.Singleton)
class UserService {
  fn getUsers() -> List<User> { /* ... */ }
}

# Inyectar dependencias (controllers NO usan @injectable)
@controller("/api/auth")
controller AuthController {
  constructor(
    @inject userService: UserService,
    @inject logger: Logger
  ) { }
}

# Contenedor DI (✅ usar @container, estándar de industria)
@container
class AppContainer {
  @provides(scope: Scope.Singleton)
  fn provideDatabase() -> Database {
    return Database(url: "mongodb://localhost")
  }
}
```

**Ejemplo REST API completo**:
```vela
# Controlador REST (✅ NO usa @injectable)
@controller("/api/users")
controller UserController {
  constructor(@inject userService: UserService) { }
  
  @get("/")
  fn getAll() -> Response<List<User>> { /* ... */ }
  
  @post("/")
  fn create(user: User) -> Response<User> { /* ... */ }
  
  @get("/:id")
  fn getById(id: String) -> Response<User> { /* ... */ }
}
```

### Sistema de State Management Global

Estas palabras clave permitirán gestión de estado global (Redux-style):

| Palabra Clave | Tipo | Propósito |
|---------------|------|-----------|
| `@connect` | Decorator | Conecta un widget a un Store global |
| `@select` | Decorator | Selecciona una porción del estado (optimización) |
| `@persistent` | Decorator | Hace el estado persistente automáticamente |
| `Store` | Class base | Clase base para stores globales |
| `Action` | Type | Tipo base para acciones del store |
| `Reducer` | Type | Tipo para funciones reductoras |
| `dispatch` | Keyword | Envía una acción al store |
| `subscribe` | Keyword | Se suscribe a cambios del store |

**Ejemplo completo**:
```vela
# Definir Store
class AppStore extends Store<AppState> {
  constructor() {
    super(initialState: AppState(counter: 0, user: None))
  }
  
  # Reducer
  reducer = fn (state: AppState, action: Action) -> AppState {
    return match action {
      IncrementAction => state with { counter: state.counter + 1 }
      SetUserAction(user) => state with { user: Some(user) }
      _ => state
    }
  }
}

# Conectar widget al store
@connect(AppStore)
@select((store) => store.counter) # Solo re-renderiza si counter cambia
widget CounterWidget {
  fn render(context: Context) -> Widget {
    dispatch(IncrementAction()) # Enviar acción
    
    return Button(
      text: "Count: ${props.counter}",
      onClick: fn() { dispatch(IncrementAction()) }
    )
  }
}

# Store persistente
@persistent(key: "app_state")
class PersistentStore extends Store<AppState> { }
```

### Testing Avanzado

Palabras clave adicionales para testing completo:

| Palabra Clave | Tipo | Propósito |
|---------------|------|-----------|
| `@mock` | Decorator | Crea un mock de una clase/interfaz |
| `@spy` | Decorator | Espía las llamadas a un método |
| `verify` | Keyword | Verifica que un mock fue llamado |
| `when` | Keyword | Define comportamiento de un mock |
| `testWidget` | Keyword | Test específico para widgets |

**Ejemplo**:
```vela
@test
fn shouldCallUserService() {
  # Mock del servicio
  @mock userService = MockUserService()
  
  # Definir comportamiento
  when(userService.getUsers()).thenReturn([
    User(name: "John", age: 30)
  ])
  
  controller = AuthController(userService: userService)
  controller.loadUsers()
  
  # Verificar llamada
  verify(userService.getUsers()).calledOnce()
}

testWidget("Counter increments on button press", fn(tester) {
  widget = CounterWidget()
  tester.pumpWidget(widget)
  
  tester.tap(find.byText("Increment"))
  tester.pump()
  
  expect(find.byText("Count: 1")).toExist()
})
```

### Resumen de Extensiones Propuestas

**Estado actual de Vela**:
- ✅ ~100 keywords existentes (funcional, OOP, reactivo, UI)
- 🟡 State Management LOCAL (signals)
- ❌ Sin DI system
- ❌ Sin State Management GLOBAL
- 🟡 Testing básico (sin mocks ni widget testing)

**Con las extensiones propuestas** (+15 keywords):
- ✅ Sistema DI completo (4 decorators: @injectable, @inject, @container, @provides)
- ✅ REST/API Support (7 decorators: @controller, @get, @post, @put, @delete, @patch, @middleware, @guard)
- ✅ State Management global (8 keywords: @connect, @select, @persistent, Store, Action, Reducer, dispatch, subscribe)
- ✅ Testing avanzado (5 keywords: @mock, @spy, verify, when, testWidget)
- ✅ Event System (5 keywords: EventBus, EventEmitter, on, emit, off)

**Total estimado: ~126 palabras reservadas** para un lenguaje de alto nivel completo y productivo.

**⚠️ Nota sobre decoradores**: 
- `@module("name")` → organización de código (ya existe en keywords-reference.md línea 1250)
- `@container` → contenedor DI (nuevo, estándar Spring/Angular/NestJS)
- `@controller("/path")` → controlador REST (nuevo, estándar NestJS/Spring)
- `module X.Y.Z;` → declaración de paquete del archivo (ya existe en 01-grammar-and-syntax.md línea 272)

### Prioridad de Implementación

Según análisis en `09-language-completeness-analysis.md`:

1. 🔴 **CRÍTICO (MVP 1.0)**: DI + State Management Global
2. 🟡 **IMPORTANTE (2.0)**: Testing avanzado (mocks, widget testing)
3. 🟢 **DESEABLE (3.0)**: Metaprogramming, reflection

Ver documento completo para detalles de implementación, ejemplos y comparación con otros lenguajes.

---

## 20. Type-Specific Keywords

**Propósito**: Vela usa **keywords específicos** en lugar de un `class` genérico para forzar claridad semántica y mejores prácticas.

### 🎨 UI Components

#### `widget`
**Propósito**: Componente de UI con estado (stateful).

**Características obligatorias**:
- DEBE implementar `build(context: Context): Widget`
- DEBE implementar lifecycle: `init()`, `dispose()`
- PUEDE tener `state` mutable
- NO puede ser `@injectable`

**Uso**:
```vela
widget LoginWidget {
  state email: String = ""
  state password: String = ""
  
  fn init(): void {
    print("Widget initialized")
  }
  
  fn dispose(): void {
    print("Widget disposed")
  }
  
  fn build(context: Context): Widget {
    return Container {
      child: Column {
        children: [
          TextField { value: this.email },
          TextField { value: this.password },
          Button { label: "Login", click: () => this.login() }
        ]
      }
    }
  }
  
  fn login(): void {
    # Lógica de login
  }
}
```

---

#### `component`
**Propósito**: Componente de UI sin estado (stateless).

**Características obligatorias**:
- DEBE implementar `build(context: Context): Widget`
- SOLO puede tener `props` readonly
- NO puede tener `state`
- NO puede tener lifecycle hooks

**Uso**:
```vela
component Button {
  props {
    label: String
    click: () => void
    disabled: Bool = false
  }
  
  fn build(context: Context): Widget {
    return Container {
      backgroundColor: this.disabled ? Colors.grey : Colors.blue,
      click: this.click,
      child: Text(this.label)
    }
  }
}
```

---

### 🏢 Business Logic

#### `service`
**Propósito**: Lógica de negocio (singleton, injectable).

**Características obligatorias**:
- DEBE ser `@injectable`
- DEBE tener `scope: Singleton` o `Transient`
- SOLO lógica de negocio pura
- NO puede tener `state` mutable
- NO puede tener UI

**Uso**:
```vela
@injectable(scope: Scope.Singleton)
service AuthService {
  constructor(@inject private repository: AuthRepository) { }
  
  public fn login(email: String, password: String): Result<User, Error> {
    return this.repository.findByEmail(email)
      .andThen(user => this.validatePassword(user, password))
  }
}
```

---

#### `repository`
**Propósito**: Acceso a datos (CRUD, async).

**Características obligatorias**:
- DEBE ser `@injectable`
- DEBE implementar: `findAll`, `findById`, `save`, `delete`
- TODOS los métodos DEBEN ser `async`
- DEBE retornar `Promise<T>` o `Result<T, Error>`
- NO puede tener lógica de negocio

**Uso**:
```vela
@injectable
repository UserRepository {
  constructor(@inject private db: Database) { }
  
  public async fn findAll(): Promise<List<User>> {
    return this.db.query("SELECT * FROM users")
  }
  
  public async fn findById(id: String): Promise<Option<User>> {
    return this.db.query("SELECT * FROM users WHERE id = ?", [id])
  }
  
  public async fn save(user: User): Promise<User> {
    return this.db.insert("users", user)
  }
  
  public async fn delete(id: String): Promise<Bool> {
    return this.db.delete("users", id)
  }
}
```

---

#### `controller`
**Propósito**: Controlador REST API (como NestJS).

**Características obligatorias**:
- **NO necesita** `@injectable` (se registra en `controllers: []`, NO en `providers: []`)
- DEBE tener decorador `@controller(path)` con path base
- Métodos públicos DEBEN tener decorador HTTP (`@get`, `@post`, `@put`, `@patch`, `@delete`)
- DEBE retornar `Response<T>` o `Promise<Response<T>>`
- PUEDE recibir dependencias con `@inject` en constructor (sin necesitar `@injectable`)
- NO puede tener lógica de negocio (solo orchestración y delegación a services)

**Uso**:
```vela
# ✅ CORRECTO: Controller NO usa @injectable
@controller("/api/users")
controller UserController {
  constructor(@inject private service: UserService) { }
  
  @get("/")
  public async fn getAll(): Response<List<User>> {
    return this.service.findAll()
      .map(users => Response.ok(users))
  }
  
  @post("/")
  public async fn create(@body dto: CreateUserDto): Response<User> {
    return this.service.create(dto)
      .map(user => Response.created(user))
  }
  
  @get("/:id")
  public async fn getById(@param id: String): Response<User> {
    return this.service.findById(id)
      .map(user => Response.ok(user))
  }
}
```

---

#### `usecase`
**Propósito**: Caso de uso (orquestación, single responsibility).

**Características obligatorias**:
- DEBE ser `@injectable`
- DEBE tener UN SOLO método público: `execute(...): Result<T, Error>`
- PUEDE orquestar múltiples servicios
- NO puede tener múltiples métodos públicos

**Uso**:
```vela
@injectable
usecase LoginUseCase {
  constructor(
    @inject private authService: AuthService,
    @inject private auditService: AuditService
  ) { }
  
  public async fn execute(email: String, password: String): Result<User, Error> {
    return this.authService.login(email, password)
      .andThen(user => this.auditService.log("User logged in", user))
  }
}
```

---

### 📦 Data Transfer & Models

#### `dto`
**Propósito**: Data Transfer Object (immutable, serializable).

**Características obligatorias**:
- TODAS las propiedades readonly
- DEBE implementar `validate(): Result<void, ValidationError>`
- DEBE implementar `toJson(): JsonObject`
- DEBE implementar `static fromJson(json): Result<Self, Error>`
- NO puede tener métodos de negocio

**Uso**:
```vela
dto CreateUserDto {
  public readonly email: String
  public readonly password: String
  public readonly name: String
  
  fn validate(): Result<void, ValidationError> {
    if (!this.email.contains("@")) {
      return Result.err(ValidationError("Invalid email"))
    }
    return Result.ok(())
  }
  
  fn toJson(): JsonObject {
    return { "email": this.email, "password": this.password, "name": this.name }
  }
  
  static fn fromJson(json: JsonObject): Result<CreateUserDto, Error> {
    return Result.ok(CreateUserDto {
      email: json["email"],
      password: json["password"],
      name: json["name"]
    })
  }
}
```

---

#### `entity`
**Propósito**: Entidad de dominio (identidad, mutable controlado).

**Características obligatorias**:
- DEBE tener `readonly id` único
- DEBE implementar `equals(other: Self): Bool` (por ID)
- DEBE usar factory method `static create(...): Result<Self, Error>`
- Constructor DEBE ser privado
- Mutaciones DEBEN retornar `Result<void, Error>`

**Uso**:
```vela
entity User {
  public readonly id: String
  private email: String
  private name: String
  
  private constructor(id: String, email: String, name: String) {
    this.id = id
    this.email = email
    this.name = name
  }
  
  static fn create(email: String, name: String): Result<User, Error> {
    return validateEmail(email)
      .andThen(() => validateName(name))
      .map(() => User(generateId(), email, name))
  }
  
  fn changeName(newName: String): Result<void, Error> {
    return validateName(newName).map(() => {
      this.name = newName
    })
  }
  
  fn equals(other: User): Bool {
    return this.id == other.id
  }
}
```

---

#### `valueObject`
**Propósito**: Value Object (immutable, sin identidad).

**Características obligatorias**:
- TODAS las propiedades readonly
- DEBE implementar `equals(other: Self): Bool` (por valor)
- DEBE implementar `static create(...): Result<Self, Error>`
- Constructor DEBE ser privado
- DEBE ser completamente inmutable

**Uso**:
```vela
valueObject Email {
  private readonly value: String
  
  private constructor(value: String) {
    this.value = value
  }
  
  static fn create(value: String): Result<Email, Error> {
    if (!value.contains("@")) {
      return Result.err(Error("Invalid email"))
    }
    return Result.ok(Email(value))
  }
  
  fn equals(other: Email): Bool {
    return this.value == other.value
  }
  
  fn toString(): String {
    return this.value
  }
}
```

---

#### `model`
**Propósito**: Modelo de datos (estructura, validación).

**Características obligatorias**:
- Propiedades públicas (schema explícito)
- DEBE implementar `validate(): Result<void, ValidationError>`
- DEBE implementar `toEntity(): Result<Entity, Error>`
- DEBE implementar `static fromEntity(entity): Self`

**Uso**:
```vela
model UserModel {
  public id: String
  public email: String
  public name: String
  public age: Int
  
  fn validate(): Result<void, ValidationError> {
    if (this.age < 0 || this.age > 150) {
      return Result.err(ValidationError("Invalid age"))
    }
    return Result.ok(())
  }
  
  fn toEntity(): Result<User, Error> {
    return User.create(this.email, this.name)
  }
  
  static fn fromEntity(user: User): UserModel {
    return UserModel {
      id: user.id,
      email: user.email,
      name: user.name,
      age: user.age
    }
  }
}
```

---

### 🏗️ Design Patterns

#### `factory`
**Propósito**: Factory (creación de objetos).

**Características obligatorias**:
- DEBE tener método `static create(...): Result<T, Error>`
- TODOS los métodos DEBEN ser `static`
- NO puede tener instancias

**Uso**:
```vela
factory UserFactory {
  static fn create(type: UserType, data: CreateUserData): Result<User, Error> {
    return match type {
      UserType.Admin => this.createAdmin(data),
      UserType.Customer => this.createCustomer(data)
    }
  }
  
  private static fn createAdmin(data: CreateUserData): Result<User, Error> {
    return User.create(data.email, data.name)
      .map(user => user.grantRole(Role.Admin))
  }
}
```

---

#### `builder`
**Propósito**: Builder (construcción fluida).

**Características obligatorias**:
- DEBE tener método `build(): Result<T, Error>`
- Métodos de construcción DEBEN retornar `Self`
- DEBE validar en `build()`, no antes

**Uso**:
```vela
builder UserBuilder {
  private email: Option<String> = None
  private name: Option<String> = None
  
  public fn withEmail(email: String): Self {
    this.email = Some(email)
    return this
  }
  
  public fn withName(name: String): Self {
    this.name = Some(name)
    return this
  }
  
  public fn build(): Result<User, Error> {
    return User.create(
      this.email.expect("Email required"),
      this.name.expect("Name required")
    )
  }
}
```

---

#### `strategy`
**Propósito**: Strategy (algoritmos intercambiables).

**Características obligatorias**:
- Base DEBE tener método `abstract`
- Implementaciones DEBEN extender base
- DEBEN ser intercambiables (LSP)

**Uso**:
```vela
strategy PaymentStrategy {
  abstract fn pay(amount: Decimal): Result<Receipt, Error>
}

strategy CreditCardPayment extends PaymentStrategy {
  fn pay(amount: Decimal): Result<Receipt, Error> {
    return Result.ok(Receipt { amount, method: "Credit Card" })
  }
}
```

---

#### `observer`
**Propósito**: Observer (patrón observador).

**Características obligatorias**:
- DEBE tener método `update(event: T): void`
- NO puede retornar valores (solo side effects)

**Uso**:
```vela
observer StockObserver {
  abstract fn update(event: StockEvent): void
}

observer EmailNotifier extends StockObserver {
  fn update(event: StockEvent): void {
    print("Stock ${event.symbol} is now ${event.price}")
  }
}
```

---

#### `singleton`
**Propósito**: Singleton (instancia única global).

**Características obligatorias**:
- Constructor DEBE ser privado
- DEBE tener `static getInstance(): Self`
- DEBE garantizar una sola instancia
- DEBE ser thread-safe

**Uso**:
```vela
singleton AppConfig {
  private static instance: Option<AppConfig> = None
  
  private constructor() {
    this.loadConfig()
  }
  
  public static fn getInstance(): AppConfig {
    if (AppConfig.instance.isNone()) {
      AppConfig.instance = Some(AppConfig())
    }
    return AppConfig.instance.unwrap()
  }
  
  public fn getApiUrl(): String {
    return this.apiUrl
  }
}
```

---

#### `adapter`
**Propósito**: Adapter (interfaz adaptadora).

**Características obligatorias**:
- DEBE tener método `adapt(external: T): Internal`
- NO puede tener lógica de negocio
- NO puede modificar datos (solo adaptar estructura)

**Uso**:
```vela
adapter HttpAdapter {
  fn adapt(request: HttpRequest): AxiosRequest {
    return AxiosRequest {
      url: request.url,
      method: request.method.toString(),
      headers: request.headers
    }
  }
}
```

---

#### `decorator`
**Propósito**: Decorator pattern (extensión de funcionalidad).

**Características obligatorias**:
- DEBE recibir objeto del mismo tipo en constructor
- DEBE delegar llamadas al objeto interno
- DEBE ser composable (stackable)

**Uso**:
```vela
decorator LoggingDecorator {
  constructor(private wrapped: Service) { }
  
  fn execute(data: String): Result<String, Error> {
    print("Before execution")
    let result = this.wrapped.execute(data)
    print("After execution")
    return result
  }
}
```

---

### 🔐 Security & Middleware

#### `guard`
**Propósito**: Guard de autorización.

**Características obligatorias**:
- DEBE ser `@injectable`
- DEBE implementar `canActivate(context): Promise<Result<Bool, Error>>`
- NO puede tener lógica de negocio

**Uso**:
```vela
@injectable
guard AuthGuard {
  constructor(@inject private authService: AuthService) { }
  
  async fn canActivate(context: ExecutionContext): Promise<Result<Bool, Error>> {
    let token = context.getRequest().headers["Authorization"]
    return this.authService.validateToken(token)
      .map(user => true)
      .mapErr(() => false)
  }
}
```

---

#### `middleware`
**Propósito**: Middleware HTTP.

**Características obligatorias**:
- DEBE ser `@injectable`
- DEBE implementar `apply(req, res, next): Promise<void>`
- DEBE llamar `next()` para continuar pipeline

**Uso**:
```vela
@injectable
middleware LoggingMiddleware {
  async fn apply(req: Request, res: Response, next: NextFunction): Promise<void> {
    print("Request: ${req.method} ${req.url}")
    await next()
    print("Response: ${res.statusCode}")
  }
}
```

---

#### `interceptor`
**Propósito**: Interceptor HTTP.

**Características obligatorias**:
- DEBE ser `@injectable`
- DEBE implementar `intercept(context, next): Promise<Result<Any, Error>>`
- DEBE llamar `next.handle()`

**Uso**:
```vela
@injectable
interceptor TransformInterceptor {
  async fn intercept(ctx: ExecutionContext, next: CallHandler): Promise<Result<Any, Error>> {
    return next.handle()
      .map(data => ({ success: true, data: data, timestamp: DateTime.now() }))
  }
}
```

---

#### `validator`
**Propósito**: Validator (validación de datos).

**Características obligatorias**:
- DEBE implementar `validate(value: T): Result<void, ValidationError>`
- DEBE retornar `Result` (no excepciones)
- NO puede tener estado mutable

**Uso**:
```vela
validator EmailValidator {
  fn validate(value: String): Result<void, ValidationError> {
    if (!value.contains("@")) {
      return Result.err(ValidationError("Invalid email"))
    }
    return Result.ok(())
  }
}
```

---

### 🏗️ Architecture

#### `module`
**Propósito**: Módulo funcional (MULTIPLATAFORMA: Angular + NestJS style).

**Características obligatorias**:
- DEBE tener decorador `@module({ name: "...", ... })`
- DEBE declarar `name`, `declarations`, `controllers`, `providers`, `imports`, `exports`
- `name`: Nombre único del módulo (string)
- `declarations`: Widgets, components, services (frontend/general)
- `controllers`: Controllers REST (backend)
- `providers`: Services, repositories, guards, middleware, pipes (con `@injectable`)
- `exports` ⊆ (`declarations` ∪ `providers`) (puede exportar widgets O providers)

**Uso (Backend Module)**:
```vela
@module({
  name: "UserModule",
  controllers: [UserController],  # REST endpoints
  providers: [UserService, UserRepository],  # Business logic
  imports: [DatabaseModule, HttpModule],  # Otros módulos
  exports: [UserService]
})
module UserModule { }
```

**Uso (Frontend Module)**:
```vela
@module({
  name: "AuthModule",
  declarations: [LoginWidget, HeaderWidget],  # UI components
  providers: [AuthService],  # Shared services
  imports: [UiModule, FormsModule],  # Otros módulos UI
  exports: [AuthService, LoginWidget]
})
module AuthModule { }
```

**Uso (Hybrid Module - TÍPICO EN VELA)**:
```vela
@module({
  name: "UserModule",
  declarations: [UserWidget, UserCard],  # UI components
  controllers: [UserController],  # REST API
  providers: [UserService, UserRepository],  # Business logic
  imports: [DatabaseModule],
  exports: [UserService, UserWidget]  # Exporta AMBOS: service + widget
})
module UserModule { }
```

**⚠️ IMPORTANTE**: 
- **Vela es MULTIPLATAFORMA**: soporta frontend (declarations) + backend (controllers)
- **Controllers** se registran en `controllers: []`, NO en `providers: []`
- **Declarations** para widgets/components (frontend)
- **Providers** son clases con `@injectable` (services, repositories, guards, middleware, pipes)

---

#### `store`
**Propósito**: Store de state management.

**Características obligatorias**:
- DEBE ser `@injectable(scope: Singleton)`
- DEBE implementar `initialState(): State`
- DEBE implementar `reducer(state, action): State`
- Estado DEBE ser inmutable
- Reducer DEBE ser puro

**Uso**:
```vela
@injectable(scope: Scope.Singleton)
store CounterStore {
  fn initialState(): CounterState {
    return CounterState { count: 0 }
  }
  
  fn reducer(state: CounterState, action: CounterAction): CounterState {
    return match action {
      CounterAction.Increment => CounterState { count: state.count + 1 },
      CounterAction.Decrement => CounterState { count: state.count - 1 }
    }
  }
}
```

---

#### `provider`
**Propósito**: Provider de inyección de dependencias.

**Características obligatorias**:
- DEBE implementar `provide(): T`
- NO puede retornar `null`

**Uso**:
```vela
provider HttpClientProvider {
  fn provide(): HttpClient {
    return HttpClient {
      baseUrl: "https://api.example.com",
      timeout: 5000
    }
  }
}
```

---

### ⚡ Concurrency & Transformation

#### `actor`
**Propósito**: Actor de concurrencia.

**Características obligatorias**:
- DEBE tener al menos un handler con pattern matching de mensajes
- Handlers DEBEN ser thread-safe
- NO comparte estado mutable (solo mensajes)

**Uso**:
```vela
actor EmailActor {
  private mailbox: Queue<EmailMessage> = Queue()
  
  fn receive(message: Message): Result<void, Error> {
    return match message {
      SendEmail(to, subject, body) => this.handleSend(to, subject, body),
      ScheduleEmail(to, scheduledAt) => this.handleSchedule(to, scheduledAt)
    }
  }
  
  private fn handleSend(to: String, subject: String, body: String): Result<void, Error> {
    print("Sending email to ${to}")
    return Result.ok(())
  }
}
```

---

#### `pipe`
**Propósito**: Pipe de transformación.

**Características obligatorias**:
- DEBE implementar `transform(value: T): R`
- DEBE ser puro (sin side effects)
- DEBE ser composable

**Uso**:
```vela
pipe UpperCasePipe {
  fn transform(value: String): String {
    return value.toUpperCase()
  }
}

# Uso en templates
Text("hello" | UpperCasePipe)  # "HELLO"
```

---

#### `task`
**Propósito**: Tarea asíncrona.

**Características obligatorias**:
- DEBE implementar `async execute(...): Promise<Result<T, Error>>`
- DEBE implementar `cancel(): void`
- DEBE ser cancelable

**Uso**:
```vela
task FetchUserTask {
  constructor(@inject private repository: UserRepository) { }
  
  async fn execute(userId: String): Promise<Result<User, Error>> {
    return this.repository.findById(userId)
  }
  
  fn cancel(): void {
    print("Task cancelled")
  }
}
```

---

### 🔧 Utilities

#### `helper`
**Propósito**: Helper functions (funciones utilitarias estáticas).

**Características obligatorias**:
- TODOS los métodos DEBEN ser `static`
- TODOS los métodos DEBEN ser puros
- NO puede tener instancias ni constructor

**Uso**:
```vela
helper StringHelper {
  static fn capitalize(str: String): String {
    return str[0].toUpperCase() + str.substring(1)
  }
  
  static fn slugify(str: String): String {
    return str.toLowerCase().replaceAll(" ", "-")
  }
}

# Uso
let title = StringHelper.capitalize("hello")
```

---

#### `mapper`
**Propósito**: Object Mapper (transformación de objetos).

**Características obligatorias**:
- DEBE tener métodos `toX` y `fromX`
- TODOS los métodos DEBEN ser `static`
- DEBE ser bidireccional cuando sea posible

**Uso**:
```vela
mapper UserMapper {
  static fn toDto(entity: User): UserDto {
    return UserDto {
      id: entity.id,
      email: entity.email,
      name: entity.name
    }
  }
  
  static fn toEntity(dto: UserDto): Result<User, Error> {
    return User.create(dto.email, dto.name)
  }
}
```

---

#### `serializer`
**Propósito**: Serializador (conversión de formatos).

**Características obligatorias**:
- DEBE implementar `static serialize<T>(obj: T): Result<String, Error>`
- DEBE implementar `static deserialize<T>(str: String): Result<T, Error>`
- DEBE retornar `Result`

**Uso**:
```vela
serializer JsonSerializer {
  static fn serialize<T>(obj: T): Result<String, Error> {
    return Result.ok(JSON.stringify(obj))
  }
  
  static fn deserialize<T>(json: String): Result<T, Error> {
    return Result.tryCatch(() => JSON.parse(json) as T)
  }
}
```

---

### Resumen de Type-Specific Keywords

**Total**: 30 keywords específicos

**Por categoría**:
- 🎨 UI: 2 (widget, component)
- 🏢 Business Logic: 4 (service, repository, controller, usecase)
- 📦 Data: 4 (dto, entity, valueObject, model)
- 🏗️ Patterns: 7 (factory, builder, strategy, observer, singleton, adapter, decorator)
- 🔐 Security: 4 (guard, middleware, interceptor, validator)
- 🏗️ Architecture: 3 (module, store, provider)
- ⚡ Concurrency: 3 (actor, pipe, task)
- 🔧 Utilities: 3 (helper, mapper, serializer)

**Beneficios**:
- ✅ Claridad semántica inmediata
- ✅ Validación en tiempo de compilación
- ✅ Reglas inquebrantables por tipo
- ✅ Prevención de errores
- ✅ Mejores prácticas forzadas
- ✅ IDE inteligente con autocomplete contextual

---

## 📋 Reglas de `@injectable`

**Cuándo SÍ usar `@injectable`:**

| Keyword | Requiere `@injectable` | Razón |
|---------|----------------------|-------|
| `service` | ✅ SÍ | Lógica de negocio, necesita DI |
| `repository` | ✅ SÍ | Acceso a datos, necesita DI |
| `usecase` | ✅ SÍ | Orquestación, necesita DI |
| `guard` | ✅ SÍ | Autorización, necesita DI |
| `middleware` | ✅ SÍ | Interceptores HTTP, necesita DI |
| `interceptor` | ✅ SÍ | Interceptores, necesita DI |
| `validator` | ✅ SÍ | Validación, necesita DI |
| `store` | ✅ SÍ | State management, necesita DI |
| `provider` | ✅ SÍ | Proveedores de DI, necesita DI |

**Cuándo NO usar `@injectable`:**

| Keyword | Requiere `@injectable` | Razón |
|---------|----------------------|-------|
| `controller` | ❌ NO | Se registra en `controllers: []`, no en `providers: []` |
| `widget` | ❌ NO | Componentes UI no necesitan DI |
| `component` | ❌ NO | Componentes UI no necesitan DI |
| `dto` | ❌ NO | Objetos de transferencia, no necesitan DI |
| `entity` | ❌ NO | Entidades de dominio, no necesitan DI |
| `valueObject` | ❌ NO | Value objects, no necesitan DI |
| `model` | ❌ NO | Modelos de datos, no necesitan DI |

**IMPORTANTE**: NO se usa prefijo `use` ni `on` en lifecycle hooks. Los hooks son: `init()`, `dispose()`, `mount()`, `update()`, etc.
