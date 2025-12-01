# Vela Reserved Keywords

**Version:** 0.1.0 (Phase 0 - Sprint 4) - CORRECTED  
**Status:** Specification  
**Date:** 2025-11-30  
**Paradigm:** FUNCTIONAL PROGRAMMING (Pure Functional)

## Overview

This document defines the complete set of reserved keywords (palabras reservadas) for the Vela programming language. These keywords cannot be used as identifiers (variable names, function names, type names, etc.) as they have special meaning in the language syntax.

**IMPORTANTE:** Vela es un lenguaje **funcional puro** con reactividad integrada. **NO incluye loops imperativos** (`for`, `while`, `loop`) ni valores nulos (`null`, `undefined`, `nil`). La mutabilidad se logra SOLO mediante el keyword `state` (reactivo).

## Summary Statistics

| Category | Count | Examples |
|----------|-------|----------|
| **Control Flow (Functional)** | 3 | `if`, `else`, `match` (NO loops) |
| **Variables** | 1 | `state` (único keyword mutable) |
| **Declarations** | 6 | `fn`, `struct`, `enum`, `type`, `class`, `interface` |
| **Types Primitivos** | 6 | `Number`, `Float`, `String`, `Bool`, `void`, `never` |
| **Valores Booleanos** | 3 | `true`, `false`, `None` (NO null) |
| **Error Handling** | 4 | `try`, `catch`, `throw`, `finally` |
| **Async Programming** | 3 | `async`, `await`, `yield` |
| **Module System** | 4 | `import`, `show`, `hide`, `as` (NO export) |
| **Visibility** | 3 | `public`, `private`, `protected` |
| **OOP** | 7 | `abstract`, `extends`, `implements`, `override`, `overload`, `this`, `super`, `constructor` |
| **Reactive System** | 4 | `computed`, `memo`, `effect`, `watch` |
| **Lifecycle** | 5 | `mount`, `update`, `destroy`, `beforeUpdate`, `afterUpdate` |
| **UI Widgets** | 4 | `StatefulWidget`, `StatelessWidget`, `component`, `widget` |
| **Architecture (DDD)** | 8 | `service`, `repository`, `controller`, `usecase`, `entity`, `dto`, `valueObject`, `model` |
| **Design Patterns** | 7 | `factory`, `builder`, `strategy`, `observer`, `singleton`, `adapter`, `decorator` |
| **Web/API** | 5 | `guard`, `middleware`, `interceptor`, `validator`, `pipe` |
| **Utilities** | 6 | `task`, `helper`, `mapper`, `serializer`, `provider`, `store` |
| **Return** | 1 | `return` |
| **TOTAL** | **~100** | |

---

## ❌ Keywords que NO EXISTEN en Vela

**IMPORTANTE:** Los siguientes keywords comúnmente encontrados en otros lenguajes **NO EXISTEN** en Vela:

### Loops Imperativos (PROHIBIDOS)
```
❌ for        - NO EXISTE (usar métodos funcionales: .map(), .filter(), .forEach())
❌ while      - NO EXISTE (usar recursión o métodos funcionales)
❌ loop       - NO EXISTE (usar recursión tail-call optimizada)
❌ break      - NO EXISTE (no hay loops)
❌ continue   - NO EXISTE (no hay loops)
❌ do         - NO EXISTE (no hay do-while)
```

**Razón:** Vela es funcional puro. Se usan métodos funcionales en su lugar.

### Mutabilidad por Defecto (PROHIBIDOS)
```
❌ let        - NO EXISTE (variables son inmutables por defecto)
❌ const      - NO EXISTE (inmutabilidad es por defecto, NO necesita keyword)
❌ var        - NO EXISTE (jamás)
❌ mut        - NO EXISTE (usar `state` para mutabilidad reactiva)
```

**Razón:** Inmutabilidad por defecto. Solo `state` permite mutabilidad (reactiva).

### Valores Nulos (PROHIBIDOS)
```
❌ null       - NO EXISTE (usar `None` en `Option<T>`)
❌ undefined  - NO EXISTE (usar `Option<T>`)
❌ nil        - NO EXISTE (usar `None`)
```

**Razón:** Vela usa `Option<T>` con `Some(value)` y `None` para seguridad de tipos.

### Exports Explícitos (PROHIBIDO)
```
❌ export     - NO EXISTE (usar modificador `public` en lugar)
❌ module     - NO EXISTE (usar estructura de carpetas)
```

**Razón:** Consistencia con modificadores de acceso (`public`, `private`, `protected`).

### Otros (PROHIBIDOS)
```
❌ switch     - NO EXISTE (usar `match` con pattern matching)
❌ case       - NO EXISTE (usar `match`)
❌ default    - NO EXISTE (usar `_` en match)
❌ goto       - NO EXISTE (jamás)
❌ with       - NO EXISTE
❌ in         - NO EXISTE como keyword standalone
❌ pub        - NO EXISTE (usar `public` completo)
❌ unsafe     - NO EXISTE (Vela es memory-safe siempre)
❌ extern     - NO EXISTE en Phase 0
❌ static     - NO EXISTE (usar singleton pattern)
❌ crate      - NO EXISTE (no es Rust)
❌ self       - NO EXISTE (usar `this`)
❌ Self       - NO EXISTE (usar nombre de clase)
❌ impl       - NO EXISTE (usar `class` o `implements`)
❌ trait      - NO EXISTE (usar `interface`)
```

---

## ✅ Complete Keyword List (Alphabetical)

Total: **~100 keywords**

```
abstract        adapter         afterUpdate     as              async
await           beforeUpdate    Bool            builder         catch
class           component       computed        constructor     controller
decorator       destroy         dto             effect          else
entity          enum            extends         factory         false
finally         Float           fn              guard           helper
hide            if              implements      import          interceptor
interface       mapper          match           memo            middleware
model           mount           never           None            Number
observer        override        overload        pipe            private
protected       provider        public          repository      return
serializer      service         show            singleton       state
StatefulWidget  StatelessWidget store           strategy        String
struct          super           task            this            throw
try             type            update          usecase         validator
valueObject     void            watch           widget          yield
```

---

## Keywords by Category (Detailed)

### 1. Control Flow Keywords (3) - FUNCTIONAL ONLY

| Keyword | Description | Example |
|---------|-------------|---------|
| `if` | Conditional expression (también es expression) | `if x > 0 { "positive" } else { "negative" }` |
| `else` | Alternative branch | `if x > 0 { ... } else { ... }` |
| `match` | Pattern matching exhaustivo | `match value { Some(x) => x, None => 0 }` |

**⚠️ NO HAY LOOPS IMPERATIVOS** - Vela es funcional puro.

**Examples:**
```vela
# if-else como expression
status = if age >= 18 { "adult" } else { "minor" }

# match con pattern matching
match result {
  Ok(value) => print("Success: ${value}")
  Err(error) => print("Error: ${error}")
}

# match con guards
match number {
  n if n < 0 => "negative"
  n if n == 0 => "zero"
  n => "positive"
}

# match con destructuring
match point {
  { x: 0, y: 0 } => "origin"
  { x, y } => "point at (${x}, ${y})"
}

# ❌ PROHIBIDO: loops imperativos
# for i in 0..10 { print(i) }  // ERROR
# while condition { work() }    // ERROR

# ✅ CORRECTO: métodos funcionales
(0..10).forEach(i => print(i))
list.map(x => x * 2).filter(x => x > 5)
```

---

### 2. Variables & Mutability (1) - Immutable by Default

| Keyword | Description | Example |
|---------|-------------|---------|
| `state` | Variable mutable y reactiva (ÚNICA mutabilidad) | `state count: Number = 0` |

**⚠️ Variables son INMUTABLES por defecto (sin keyword).**

**Examples:**
```vela
# ✅ Inmutable por defecto (NO necesita const ni let)
name: String = "Vela"
PI: Float = 3.14159

# ❌ NO se puede mutar inmutable
# name = "Otro"  // ERROR de compilación

# ✅ state para mutabilidad reactiva
state counter: Number = 0
counter = counter + 1  # OK, es mutable y reactivo

# ✅ Shadowing (nueva variable, NO mutación)
x: Number = 5
x: Number = x + 1  # Nueva variable x
```

---

### 3. Declaration Keywords (6)

| Keyword | Description | Example |
|---------|-------------|---------|
| `fn` | Function declaration | `fn add(a: Number, b: Number) -> Number { return a + b }` |
| `struct` | Structure type (record/producto) | `struct Point { x: Number, y: Number }` |
| `enum` | Enumeration (con/sin datos) | `enum Color { Red, Green, Blue, Custom(r, g, b) }` |
| `type` | Type alias o union type | `type UserId = Number` o `type Status = "active" \| "inactive"` |
| `class` | Class declaration (OOP) | `class Person { name: String }` |
| `interface` | Interface/contract | `interface Drawable { fn draw() -> void }` |

**Examples:**
```vela
# Function
fn calculate(a: Number, b: Number) -> Number {
  return a + b
}

# Arrow function
add = (a: Number, b: Number) => a + b

# Struct
struct User {
  id: Number
  name: String
  email: String
}

# Enum
enum Result<T, E> {
  Ok(T)
  Err(E)
}

# Type alias
type UserId = Number

# Union type
type Status = "active" | "inactive" | "pending"

# Class
class Rectangle {
  width: Number
  height: Number
  
  constructor(width: Number, height: Number) {
    this.width = width
    this.height = height
  }
  
  fn area() -> Number {
    return this.width * this.height
  }
}

# Interface
interface Serializable {
  fn toJSON() -> String
}
```

---

### 4. Type Keywords (6)

| Keyword | Description | Example |
|---------|-------------|---------|
| `Number` | Integer (64-bit) | `age: Number = 37` |
| `Float` | Floating point (64-bit) | `price: Float = 19.99` |
| `String` | String type | `name: String = "Vela"` |
| `Bool` | Boolean type | `isActive: Bool = true` |
| `void` | No return value | `fn log() -> void { }` |
| `never` | Never returns | `fn panic() -> never { throw Error() }` |

**Examples:**
```vela
# Primitive types
age: Number = 37
price: Float = 19.99
name: String = "Vela"
isActive: Bool = true

# Function return types
fn greet() -> void {
  print("Hello")
}

fn panic(msg: String) -> never {
  throw Error(msg)
}

# Option<T> (NO null)
user: Option<User> = None
result: Option<Number> = Some(42)
```

---

### 5. Visibility & Modifier Keywords (3)

| Keyword | Description | Example |
|---------|-------------|---------|
| `public` | Public visibility (accesible externamente) | `public fn publicFunction() -> void { }` |
| `private` | Private visibility (solo dentro de clase/módulo) | `private fn helper() -> void { }` |
| `protected` | Protected visibility (clase y subclases) | `protected fn method() -> void { }` |

**⚠️ NO EXISTE `export` - usar `public` en su lugar.**

**Examples:**
```vela
# Public (accesible desde otros módulos)
public class MyClass {
  public name: String
  
  public fn greet() -> void {
    print("Hello")
  }
}

# Private (solo dentro del módulo/clase)
private fn internalHelper() -> void {
  # solo accesible aquí
}

class Person {
  private age: Number  # solo dentro de Person
  
  protected fn validate() -> Bool {
    # accesible en subclases
  }
}
```

---

### 6. OOP Keywords (10)

| Keyword | Description | Example |
|---------|-------------|---------|
| `abstract` | Abstract class (no instanciable) | `abstract class Shape { abstract fn area() -> Float }` |
| `extends` | Herencia de clase | `class Dog extends Animal { }` |
| `implements` | Implementa interfaz | `class Button implements Clickable { }` |
| `override` | Sobrescribe método | `override fn toString() -> String { }` |
| `overload` | Sobrecarga de métodos | `overload fn add(a: Number, b: Number) -> Number { }` |
| `this` | Instancia actual | `this.name` |
| `super` | Clase padre | `super.greet()` |
| `constructor` | Constructor de clase | `constructor(name: String) { this.name = name }` |

**Examples:**
```vela
# Abstract class
abstract class Shape {
  abstract fn area() -> Float
  abstract fn perimeter() -> Float
  
  fn describe() -> String {
    return "Area: ${this.area()}, Perimeter: ${this.perimeter()}"
  }
}

# Extends
class Circle extends Shape {
  radius: Float
  
  constructor(radius: Float) {
    this.radius = radius
  }
  
  override fn area() -> Float {
    return 3.14159 * this.radius * this.radius
  }
  
  override fn perimeter() -> Float {
    return 2 * 3.14159 * this.radius
  }
}

# Interface + implements
interface Clickable {
  fn onClick() -> void
}

class Button implements Clickable {
  text: String
  
  fn onClick() -> void {
    print("Button '${this.text}' clicked")
  }
}

# super
class Dog extends Animal {
  constructor(name: String) {
    super(name)  # Llama constructor padre
  }
  
  override fn speak() -> void {
    super.speak()  # Llama método padre
    print("Woof!")
  }
}
```

---

### 7. Error Handling Keywords (4)

| Keyword | Description | Example |
|---------|-------------|---------|
| `try` | Try block | `try { riskyOp() } catch (e) { handle(e) }` |
| `catch` | Catch exception | `catch (e: MyError) { print(e) }` |
| `throw` | Throw exception | `throw Error("failed")` |
| `finally` | Always execute | `finally { cleanup() }` |

**Preferred: `Result<T, E>` type** (más idiomático que excepciones)

**Examples:**
```vela
# Try-catch-finally
try {
  file = openFile("data.txt")
  process(file)
} catch (e: IOException) {
  print("IO Error: ${e}")
} catch (e: Error) {
  print("General Error: ${e}")
} finally {
  cleanup()
}

# Throw
fn validate(age: Number) -> void {
  if age < 0 {
    throw Error("Age cannot be negative")
  }
}

# ✅ Preferido: Result<T, E>
fn divide(a: Number, b: Number) -> Result<Float, Error> {
  if b == 0 {
    return Err(Error("division by zero"))
  }
  return Ok(a / b)
}

# Uso con match
match divide(10, 2) {
  Ok(value) => print("Result: ${value}")
  Err(error) => print("Error: ${error}")
}

# Chaining con ?
fn calculate() -> Result<Number, Error> {
  x = divide(10, 2)?  # Return early si Err
  y = divide(x, 3)?
  return Ok(y)
}
```

---

### 8. Async Programming Keywords (3)

| Keyword | Description | Example |
|---------|-------------|---------|
| `async` | Async function | `async fn fetchData() -> Result<String> { }` |
| `await` | Await async result | `data = await fetchData()` |
| `yield` | Generator yield | `yield nextValue` |

**Examples:**
```vela
# Async function
async fn fetchUser(id: Number) -> Result<User> {
  response = await httpClient.get("/users/${id}")
  return response.json()
}

# Await
async fn process() -> void {
  user = await fetchUser(123)
  profile = await fetchProfile(user.id)
  await saveData(user, profile)
}

# Multiple awaits (sequential)
async fn sequential() -> void {
  result1 = await operation1()
  result2 = await operation2()
  result3 = await operation3()
}

# Parallel awaits
async fn parallel() -> void {
  results = await Promise.all([
    operation1(),
    operation2(),
    operation3()
  ])
}

# Generator (yield)
fn* fibonacci() -> Generator<Number> {
  a = 0
  b = 1
  loop {
    yield a
    temp = a
    a = b
    b = temp + b
  }
}

# Uso del generator
gen = fibonacci()
print(gen.next())  # 0
print(gen.next())  # 1
print(gen.next())  # 1
print(gen.next())  # 2
```

---

### 9. Module System Keywords (4)

| Keyword | Description | Example |
|---------|-------------|---------|
| `import` | Import module/package | `import 'package:http'` |
| `show` | Import specific items | `import 'lib:utils' show { sort, filter }` |
| `hide` | Import all except | `import 'lib:math' hide { deprecated_fn }` |
| `as` | Alias import | `import 'package:long_name' as ln` |

**⚠️ NO EXISTE `export` - usar `public` en su lugar.**

**Examples:**
```vela
# Import package
import 'package:http'
import 'package:flutter/widgets'

# Import local module
import './models/user'
import '../utils/helpers'

# Import specific items (show)
import 'lib:collections' show { List, Map, Set }
import './math' show { add, multiply }

# Import all except (hide)
import 'lib:old_api' hide { deprecatedFunction }

# Alias
import 'package:very_long_package_name' as vlpn
vlpn.someFunction()

# ❌ NO usar export keyword
# export fn myFunction() { }  // ERROR

# ✅ Usar public
public fn myFunction() -> void {
  # Accesible desde otros módulos
}

# Privado por defecto
fn privateHelper() -> void {
  # Solo accesible dentro del módulo
}
```

---

### 10. Reactive System Keywords (4)

| Keyword | Description | Example |
|---------|-------------|---------|
| `computed` | Computed value (reactivo) | `computed doubled: Number { return this.count * 2 }` |
| `memo` | Memoized computed (caché agresivo) | `memo expensive: Number { /* cálculo costoso */ }` |
| `effect` | Side effect reactivo | `effect { print("Count: ${this.count}") }` |
| `watch` | Watch specific changes | `watch(this.name) { print("Name changed") }` |

**Examples:**
```vela
class Counter extends StatefulWidget {
  state count: Number = 0
  
  # Computed (recalcula cuando count cambia)
  computed doubled: Number {
    return this.count * 2
  }
  
  computed tripled: Number {
    return this.count * 3
  }
  
  # Memo (solo recalcula si realmente cambió)
  memo expensiveCalculation: Number {
    # Cálculo costoso
    sum = 0
    (0..this.count).forEach(i => {
      sum = sum + i * i
    })
    return sum
  }
  
  # Effect (ejecuta cuando dependencias cambian)
  effect {
    print("Count changed to: ${this.count}")
  }
  
  # Watch (observa cambio específico)
  watch(this.count) {
    if this.count > 10 {
      print("Count exceeded 10!")
    }
  }
  
  fn increment() -> void {
    this.count = this.count + 1
  }
}
```

---

### 11. UI Lifecycle Keywords (5)

| Keyword | Description | Example |
|---------|-------------|---------|
| `mount` | Component mounted | `mount() { this.fetchData() }` |
| `update` | Component updated | `update() { print("Updated") }` |
| `destroy` | Component unmounted | `destroy() { this.cleanup() }` |
| `beforeUpdate` | Before DOM update | `beforeUpdate() { /* ... */ }` |
| `afterUpdate` | After DOM update | `afterUpdate() { /* ... */ }` |

**Examples:**
```vela
class UserProfile extends StatefulWidget {
  state user: Option<User> = None
  state loading: Bool = false
  
  # Mount: se ejecuta al montar componente
  mount() {
    this.fetchUser()
  }
  
  # Before update: antes de actualizar DOM
  beforeUpdate() {
    print("About to update DOM")
  }
  
  # Update: después de cambio de estado
  update() {
    print("State updated, re-rendering")
  }
  
  # After update: después de actualizar DOM
  afterUpdate() {
    print("DOM has been updated")
  }
  
  # Destroy: al desmontar componente
  destroy() {
    this.cancelRequests()
    this.cleanup()
  }
  
  async fn fetchUser() -> void {
    this.loading = true
    this.user = await apiClient.getUser(123)
    this.loading = false
  }
}
```

---

### 12. UI Widget Keywords (4)

| Keyword | Description | Example |
|---------|-------------|---------|
| `StatefulWidget` | Widget con estado mutable | `class Counter extends StatefulWidget { }` |
| `StatelessWidget` | Widget sin estado (puro) | `class Label extends StatelessWidget { }` |
| `component` | Alias de StatefulWidget | `component MyButton { }` |
| `widget` | Widget genérico | `widget CustomBox { }` |

**Examples:**
```vela
# StatefulWidget (con estado)
class Counter extends StatefulWidget {
  state count: Number = 0
  
  fn render() -> Widget {
    return Column([
      Text("Count: ${this.count}"),
      Button(
        text: "Increment",
        onClick: () => this.count = this.count + 1
      )
    ])
  }
}

# StatelessWidget (sin estado, puro)
class Greeting extends StatelessWidget {
  name: String
  
  constructor(name: String) {
    this.name = name
  }
  
  fn render() -> Widget {
    return Text("Hello, ${this.name}!")
  }
}

# component (alias)
component TodoItem {
  task: String
  completed: Bool
  
  fn render() -> Widget {
    return Row([
      Checkbox(checked: this.completed),
      Text(this.task)
    ])
  }
}

# widget (genérico)
widget Card {
  child: Widget
  
  fn render() -> Widget {
    return Container(
      padding: 16,
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey)
      ),
      child: this.child
    )
  }
}
```

---

### 4. Types & Special Values (7)

| Keyword | Description | Example |
|---------|-------------|---------|
| `true` | Boolean true literal | `let valid = true;` |
| `false` | Boolean false literal | `let valid = false;` |
| `null` | Null/undefined value | `let optional = null;` |
| `self` | Current instance (lowercase) | `self.name` |
| `Self` | Current type (capitalized) | `fn new() -> Self { ... }` |
| `super` | Parent module | `super::parent_function();` |
| `crate` | Current crate root | `crate::root_module::function();` |

**Examples:**
```vela
// Booleans
let is_valid = true;
let is_ready = false;

// Null
let maybe_value: i32? = null;

// self (instance)
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn distance(&self) -> f64 {
        ((self.x ** 2 + self.y ** 2) as f64).sqrt()
    }
}

// Self (type)
impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

// super (parent module)
mod parent {
    pub fn parent_fn() { ... }
    
    mod child {
        fn child_fn() {
            super::parent_fn();  // Call parent module function
        }
    }
}

// crate (root)
crate::utils::helper_function();
```

---

### 5. Error Handling Keywords (3)

| Keyword | Description | Example |
|---------|-------------|---------|
| `try` | Try block (error handling) | `try { ... } catch (e) { ... }` |
| `catch` | Catch exceptions | `try { ... } catch (e) { print(e); }` |
| `throw` | Throw/raise exception | `throw Error("something went wrong");` |

**Examples:**
```vela
// Try-catch
try {
    let file = open_file("data.txt")?;
    process(file);
} catch (e) {
    print("Error: ${e}");
}

// Throw
fn validate(x: i32) {
    if x < 0 {
        throw Error("value must be positive");
    }
}

// ? operator (error propagation)
fn read_config() -> Result<Config> {
    let file = open_file("config.toml")?;  // Returns early on error
    let content = file.read_to_string()?;
    parse_config(content)
}
```

---

### 6. Async Programming Keywords (2)

| Keyword | Description | Example |
|---------|-------------|---------|
| `async` | Async function modifier | `async fn fetch() { ... }` |
| `await` | Await async operation | `let result = async_call().await;` |

**Examples:**
```vela
// Async function
async fn fetch_user(id: UserId) -> Result<User> {
    let response = http_client.get("/users/${id}").await?;
    response.json().await
}

// Await in async context
async fn process() {
    let user = fetch_user(123).await?;
    let profile = fetch_profile(user.id).await?;
    save_data(user, profile).await?;
}

// Parallel async
async fn fetch_all() {
    let (users, posts, comments) = await! [
        fetch_users(),
        fetch_posts(),
        fetch_comments(),
    ];
}
```

---

### 13. Architecture/DDD Keywords (8)

| Keyword | Description | Example |
|---------|-------------|---------|
| `service` | Business logic service | `service UserService { }` |
| `repository` | Data access layer | `repository UserRepository { }` |
| `controller` | Request handler | `controller UserController { }` |
| `usecase` | Use case/interactor | `usecase CreateUser { }` |
| `entity` | Domain entity | `entity User { }` |
| `dto` | Data Transfer Object | `dto CreateUserDTO { }` |
| `valueObject` | Value object (immutable) | `valueObject Email { }` |
| `model` | Generic model | `model Product { }` |

**Examples:**
```vela
# Service (business logic)
service UserService {
  repository: UserRepository
  
  constructor(repository: UserRepository) {
    this.repository = repository
  }
  
  async fn createUser(dto: CreateUserDTO) -> Result<User, Error> {
    # Validate
    if dto.email.length == 0 {
      return Err(Error("Email required"))
    }
    
    # Create entity
    user = User {
      id: generateId(),
      name: dto.name,
      email: Email.create(dto.email)
    }
    
    # Save
    return await this.repository.save(user)
  }
}

# Repository (data access)
repository UserRepository {
  db: DatabaseConnection
  
  async fn findById(id: Number) -> Option<User> {
    return await this.db.query("SELECT * FROM users WHERE id = ?", [id])
  }
  
  async fn save(user: User) -> Result<User, Error> {
    return await this.db.insert("users", user)
  }
}

# Controller (HTTP handler)
controller UserController {
  service: UserService = inject(UserService)
  
  @post("/users")
  @validate
  async fn create(dto: CreateUserDTO) -> Response {
    match await this.service.createUser(dto) {
      Ok(user) => Response.json(user, 201)
      Err(error) => Response.error(error, 400)
    }
  }
}

# Use Case
usecase CreateUser {
  repository: UserRepository
  eventBus: EventBus
  
  async fn execute(dto: CreateUserDTO) -> Result<User, Error> {
    user = User.create(dto)
    savedUser = await this.repository.save(user)
    await this.eventBus.publish(UserCreatedEvent(savedUser))
    return Ok(savedUser)
  }
}

# Entity (domain model)
entity User {
  id: UserId
  name: String
  email: Email  # ValueObject
  createdAt: DateTime
  
  fn changeEmail(newEmail: Email) -> Result<void, Error> {
    if !newEmail.isValid() {
      return Err(Error("Invalid email"))
    }
    this.email = newEmail
    return Ok(void)
  }
}

# DTO (data transfer)
dto CreateUserDTO {
  name: String
  email: String
  password: String
}

dto UserResponseDTO {
  id: Number
  name: String
  email: String
}

# Value Object (immutable)
valueObject Email {
  value: String
  
  static fn create(value: String) -> Result<Email, Error> {
    if !Email.isValidFormat(value) {
      return Err(Error("Invalid email format"))
    }
    return Ok(Email { value })
  }
  
  static fn isValidFormat(value: String) -> Bool {
    return value.contains("@")
  }
}
```

---

### 14. Design Pattern Keywords (7)

| Keyword | Description | Example |
|---------|-------------|---------|
| `factory` | Factory pattern | `factory UserFactory { }` |
| `builder` | Builder pattern | `builder QueryBuilder { }` |
| `strategy` | Strategy pattern | `strategy PaymentStrategy { }` |
| `observer` | Observer pattern | `observer EventObserver { }` |
| `singleton` | Singleton pattern | `singleton Database { }` |
| `adapter` | Adapter pattern | `adapter LegacyAdapter { }` |
| `decorator` | Decorator pattern | `decorator LogDecorator { }` |

**Examples:**
```vela
# Factory
factory UserFactory {
  static fn createFromDTO(dto: CreateUserDTO) -> User {
    return User {
      id: generateId(),
      name: dto.name,
      email: Email.create(dto.email),
      createdAt: DateTime.now()
    }
  }
  
  static fn createAdmin(name: String) -> User {
    return User {
      id: generateId(),
      name: name,
      email: Email.create("${name}@admin.com"),
      role: Role.Admin,
      createdAt: DateTime.now()
    }
  }
}

# Builder
builder QueryBuilder {
  table: String
  conditions: List<Condition> = []
  orderBy: Option<String> = None
  limit: Option<Number> = None
  
  fn where(field: String, value: Any) -> Self {
    this.conditions.push(Condition { field, value })
    return this
  }
  
  fn order(field: String) -> Self {
    this.orderBy = Some(field)
    return this
  }
  
  fn take(n: Number) -> Self {
    this.limit = Some(n)
    return this
  }
  
  fn build() -> String {
    query = "SELECT * FROM ${this.table}"
    if this.conditions.length > 0 {
      query = query + " WHERE ${this.buildConditions()}"
    }
    match this.orderBy {
      Some(field) => query = query + " ORDER BY ${field}"
      None => { }
    }
    match this.limit {
      Some(n) => query = query + " LIMIT ${n}"
      None => { }
    }
    return query
  }
}

# Strategy
strategy PaymentStrategy {
  abstract fn pay(amount: Float) -> Result<PaymentResult, Error>
}

class CreditCardStrategy implements PaymentStrategy {
  cardNumber: String
  
  override fn pay(amount: Float) -> Result<PaymentResult, Error> {
    # Process credit card payment
    return Ok(PaymentResult { success: true })
  }
}

class PayPalStrategy implements PaymentStrategy {
  email: String
  
  override fn pay(amount: Float) -> Result<PaymentResult, Error> {
    # Process PayPal payment
    return Ok(PaymentResult { success: true })
  }
}

# Singleton
singleton Database {
  connection: Option<Connection> = None
  
  fn getInstance() -> Database {
    # Returns same instance always
    return Database
  }
  
  async fn connect() -> Result<void, Error> {
    if this.connection.isNone() {
      this.connection = Some(await createConnection())
    }
    return Ok(void)
  }
}

# Observer
observer EventObserver {
  abstract fn notify(event: Event) -> void
}

class LoggerObserver implements EventObserver {
  override fn notify(event: Event) -> void {
    print("Event: ${event.type} at ${event.timestamp}")
  }
}

class MetricsObserver implements EventObserver {
  override fn notify(event: Event) -> void {
    metrics.record(event)
  }
}
```

---

### 15. Web/API Keywords (5)

| Keyword | Description | Example |
|---------|-------------|---------|
| `guard` | Route guard/authorization | `guard AuthGuard { }` |
| `middleware` | HTTP middleware | `middleware Logger { }` |
| `interceptor` | Request/response interceptor | `interceptor AuthInterceptor { }` |
| `validator` | Input validator | `validator EmailValidator { }` |
| `pipe` | Data transformation pipeline | `pipe TransformPipe { }` |

**Examples:**
```vela
# Guard (route protection)
guard AuthGuard {
  fn canActivate(context: Context) -> Result<Bool, Error> {
    match context.user {
      Some(user) => return Ok(user.isAuthenticated())
      None => return Err(Error("Unauthorized"))
    }
  }
}

guard AdminGuard {
  fn canActivate(context: Context) -> Result<Bool, Error> {
    match context.user {
      Some(user) => {
        if user.role == Role.Admin {
          return Ok(true)
        } else {
          return Err(Error("Forbidden: Admin only"))
        }
      }
      None => return Err(Error("Unauthorized"))
    }
  }
}

# Middleware (HTTP pipeline)
middleware Logger {
  fn handle(req: Request, next: fn(Request) -> Response) -> Response {
    print("→ ${req.method} ${req.path}")
    response = next(req)
    print("← ${response.status}")
    return response
  }
}

middleware CORS {
  fn handle(req: Request, next: fn(Request) -> Response) -> Response {
    response = next(req)
    response.headers.set("Access-Control-Allow-Origin", "*")
    return response
  }
}

# Interceptor (transform request/response)
interceptor AuthInterceptor {
  fn intercept(req: Request) -> Request {
    token = getAuthToken()
    req.headers.set("Authorization", "Bearer ${token}")
    return req
  }
}

# Validator
validator EmailValidator {
  fn validate(value: String) -> Result<void, Error> {
    if !value.contains("@") {
      return Err(Error("Invalid email format"))
    }
    return Ok(void)
  }
}

validator AgeValidator {
  fn validate(value: Number) -> Result<void, Error> {
    if value < 18 {
      return Err(Error("Must be 18 or older"))
    }
    if value > 120 {
      return Err(Error("Invalid age"))
    }
    return Ok(void)
  }
}

# Pipe (data transformation)
pipe UpperCasePipe {
  fn transform(value: String) -> String {
    return value.toUpperCase()
  }
}

pipe DateFormatPipe {
  fn transform(date: DateTime) -> String {
    return date.format("YYYY-MM-DD")
  }
}
```

---

### 16. Utility Keywords (6)

| Keyword | Description | Example |
|---------|-------------|---------|
| `task` | Async task/background job | `task EmailTask { }` |
| `helper` | Helper/utility class | `helper DateHelper { }` |
| `mapper` | Object mapper | `mapper UserMapper { }` |
| `serializer` | Data serializer | `serializer JsonSerializer { }` |
| `provider` | Dependency provider | `provider ServiceProvider { }` |
| `store` | Global state store | `store AppStore { }` |

**Examples:**
```vela
# Task (background job)
task EmailTask {
  async fn run(recipient: String, subject: String, body: String) -> Result<void, Error> {
    await emailService.send(recipient, subject, body)
    return Ok(void)
  }
}

task ProcessImagesTask {
  async fn run(imageIds: List<Number>) -> Result<void, Error> {
    imageIds.forEach(id => {
      await processImage(id)
    })
    return Ok(void)
  }
}

# Helper (utilities)
helper DateHelper {
  static fn format(date: DateTime, pattern: String) -> String {
    # Format date according to pattern
    return formatted
  }
  
  static fn addDays(date: DateTime, days: Number) -> DateTime {
    return date.addDays(days)
  }
}

helper StringHelper {
  static fn slugify(text: String) -> String {
    return text.toLowerCase().replace(" ", "-")
  }
}

# Mapper (object transformation)
mapper UserMapper {
  static fn toDTO(user: User) -> UserDTO {
    return UserDTO {
      id: user.id,
      name: user.name,
      email: user.email.value
    }
  }
  
  static fn toEntity(dto: CreateUserDTO) -> User {
    return User {
      id: generateId(),
      name: dto.name,
      email: Email.create(dto.email)
    }
  }
}

# Serializer
serializer JsonSerializer {
  static fn serialize(obj: Any) -> String {
    return JSON.stringify(obj)
  }
  
  static fn deserialize<T>(json: String) -> Result<T, Error> {
    try {
      return Ok(JSON.parse(json) as T)
    } catch (e) {
      return Err(Error("Invalid JSON"))
    }
  }
}

# Provider (dependency injection)
provider ServiceProvider {
  static fn provide() -> Map<String, Any> {
    return {
      "UserService": UserService(UserRepository()),
      "EmailService": EmailService(),
      "Database": Database.getInstance()
    }
  }
}

# Store (global state)
store AppStore {
  state user: Option<User> = None
  state theme: String = "light"
  state notifications: List<Notification> = []
  
  fn setUser(user: User) -> void {
    this.user = Some(user)
  }
  
  fn addNotification(notification: Notification) -> void {
    this.notifications.push(notification)
  }
  
  fn toggleTheme() -> void {
    this.theme = if this.theme == "light" { "dark" } else { "light" }
  }
}
```

---

## Return Keyword

| Keyword | Description | Example |
|---------|-------------|---------|
| `return` | Return from function | `return value` |

**Examples:**
```vela
fn add(a: Number, b: Number) -> Number {
  return a + b
}

fn findUser(id: Number) -> Option<User> {
  user = database.query(id)
  if user.exists() {
    return Some(user)
  }
  return None
}

# Early return
fn validate(age: Number) -> Result<void, Error> {
  if age < 0 {
    return Err(Error("Age cannot be negative"))
  }
  if age > 150 {
    return Err(Error("Invalid age"))
  }
  return Ok(void)
}
```

---

## Functional List Methods (NOT Keywords, but important)

**Vela NO tiene loops imperativos. Se usan métodos funcionales:**

- `.map()` - Transform elements
- `.filter()` - Filter elements
- `.reduce()` - Reduce to single value
- `.forEach()` - Execute action for each
- `.flatMap()` - Map and flatten
- `.find()` - Find first match
- `.findIndex()` - Find index
- `.every()` - All match
- `.some()` - At least one matches
- `.take()` - First N elements
- `.drop()` - Skip first N
- `.takeWhile()` - Take while condition
- `.dropWhile()` - Drop while condition
- `.partition()` - Split into two lists
- `.groupBy()` - Group by key
- `.sortBy()` - Sort by criteria
- `.chunk()` - Split into chunks
- `.zip()` - Combine two lists
- `.scan()` - Reduce with intermediate steps
- `.distinct()` - Remove duplicates
- `.reverse()` - Reverse order

**Examples:**
```vela
# Map
numbers = [1, 2, 3, 4, 5]
doubled = numbers.map(x => x * 2)  # [2, 4, 6, 8, 10]

# Filter
evens = numbers.filter(x => x % 2 == 0)  # [2, 4]

# Reduce
sum = numbers.reduce((acc, x) => acc + x, 0)  # 15

# ForEach (side effects)
numbers.forEach(x => print(x))

# Chaining
result = numbers
  .filter(x => x > 2)
  .map(x => x * x)
  .reduce((acc, x) => acc + x, 0)  # 50

# Find
users = [user1, user2, user3]
admin = users.find(u => u.role == Role.Admin)  # Option<User>

# GroupBy
users.groupBy(u => u.age)  # Map<Number, List<User>>
```

---

## Summary

**Total Keywords: ~100**

- **Control Flow**: 3 (`if`, `else`, `match`) - NO loops
- **Variables**: 1 (`state`) - immutable by default
- **Declarations**: 6 (`fn`, `struct`, `enum`, `type`, `class`, `interface`)
- **Types**: 6 (`Number`, `Float`, `String`, `Bool`, `void`, `never`)
- **Values**: 3 (`true`, `false`, `None`) - NO null
- **Error Handling**: 4 (`try`, `catch`, `throw`, `finally`)
- **Async**: 3 (`async`, `await`, `yield`)
- **Modules**: 4 (`import`, `show`, `hide`, `as`) - NO export
- **Visibility**: 3 (`public`, `private`, `protected`)
- **OOP**: 10 (`abstract`, `extends`, `implements`, `override`, `overload`, `this`, `super`, `constructor`)
- **Reactive**: 4 (`computed`, `memo`, `effect`, `watch`)
- **Lifecycle**: 5 (`mount`, `update`, `destroy`, `beforeUpdate`, `afterUpdate`)
- **UI**: 4 (`StatefulWidget`, `StatelessWidget`, `component`, `widget`)
- **Architecture**: 8 (`service`, `repository`, `controller`, `usecase`, `entity`, `dto`, `valueObject`, `model`)
- **Patterns**: 7 (`factory`, `builder`, `strategy`, `observer`, `singleton`, `adapter`, `decorator`)
- **Web/API**: 5 (`guard`, `middleware`, `interceptor`, `validator`, `pipe`)
- **Utilities**: 6 (`task`, `helper`, `mapper`, `serializer`, `provider`, `store`)
- **Return**: 1 (`return`)

---

## Design Philosophy

Vela prioritizes:

1. **Functional Purity**: NO loops imperativos, immutability by default
2. **Type Safety**: NO null, usar `Option<T>`
3. **Clarity**: Domain-specific keywords hacen código más legible
4. **Architecture**: Keywords enforce arquitectura limpia (DDD, patterns)
5. **Reactivity**: Sistema reactivo integrado (computed, effect, watch)
6. **Modern**: Inspirado en Flutter, React, Rust, TypeScript

---

**END OF SPECIFICATION**

