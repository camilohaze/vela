# Vela Reserved Keywords

**Version:** 0.1.0 (Phase 0 - Sprint 4)  
**Status:** Draft  
**Date:** 2025-11-30

## Overview

This document defines the complete set of reserved keywords (palabras reservadas) for the Vela programming language. These keywords cannot be used as identifiers (variable names, function names, type names, etc.) as they have special meaning in the language syntax.

## Summary Statistics

| Category | Count | Examples |
|----------|-------|----------|
| **Control Flow** | 11 | `if`, `else`, `match`, `while`, `for` |
| **Declarations** | 8 | `let`, `const`, `fn`, `struct`, `enum`, `trait`, `impl`, `type` |
| **Visibility & Modifiers** | 6 | `pub`, `mut`, `async`, `static`, `unsafe`, `extern` |
| **Types & Values** | 7 | `true`, `false`, `null`, `self`, `Self`, `super`, `crate` |
| **Error Handling** | 3 | `try`, `catch`, `throw` |
| **Async Programming** | 2 | `await`, `yield` |
| **Module System** | 5 | `import`, `export`, `from`, `as`, `module` |
| **Domain-Specific** | 25 | `widget`, `component`, `service`, `repository`, `entity`, `dto`, etc. |
| **Reactive System** | 8 | `Signal`, `Computed`, `Effect`, `Watch`, `store`, `dispatch`, `provide`, `inject` |
| **Reserved (Future)** | 5 | `macro`, `defer`, `go`, `chan`, `select` |
| **TOTAL** | **80** | |

---

## Complete Keyword List (Alphabetical)

```
abstract        actor           adapter         as              async
await           boolean         break           builder         catch
chan            component       Computed        const           continue
controller      crate           decorator       defer           dispatch
dto             Effect          else            entity          enum
export          extern          factory         false           fn
for             from            guard           helper          if
impl            import          in              inject          interceptor
interface       let             loop            macro           mapper
match           middleware      model           module          mut
new             null            number          observer        of
pipe            provider        provide         pub             ref
repository      return          select          self            Self
serializer      service         Signal          singleton       static
store           strategy        string          struct          super
task            throw           trait           true            try
type            typeof          unsafe          usecase         validator
valueObject     watch           Watch           while           widget
yield
```

---

## Keywords by Category

### 1. Control Flow Keywords (11)

| Keyword | Description | Example |
|---------|-------------|---------|
| `if` | Conditional expression | `if x > 0 { ... }` |
| `else` | Alternative branch | `if x > 0 { ... } else { ... }` |
| `match` | Pattern matching | `match value { ... }` |
| `while` | Loop with condition | `while running { ... }` |
| `for` | Iteration loop | `for item in list { ... }` |
| `in` | Iterator keyword (used with `for`) | `for x in 0..10 { ... }` |
| `loop` | Infinite loop | `loop { ... }` |
| `break` | Exit loop | `break;` |
| `continue` | Skip to next iteration | `continue;` |
| `return` | Return from function | `return value;` |
| `yield` | Generator yield (future) | `yield value;` |

**Examples:**
```vela
// if-else
if x > 0 {
    print("positive");
} else if x < 0 {
    print("negative");
} else {
    print("zero");
}

// match
match status {
    Ok(val) => print(val),
    Err(e) => print("Error: ${e}"),
}

// for loop
for i in 0..10 {
    print(i);
}

// while loop
while running {
    process();
}

// loop with break
loop {
    if done { break; }
    work();
}
```

---

### 2. Declaration Keywords (8)

| Keyword | Description | Example |
|---------|-------------|---------|
| `let` | Variable declaration (immutable by default) | `let x = 10;` |
| `const` | Compile-time constant | `const MAX = 100;` |
| `fn` | Function declaration | `fn add(a, b) { a + b }` |
| `struct` | Structure type | `struct Point { x, y }` |
| `enum` | Enumeration type | `enum Color { Red, Green, Blue }` |
| `trait` | Interface/trait definition | `trait Drawable { ... }` |
| `impl` | Implementation block | `impl Point { ... }` |
| `type` | Type alias | `type UserId = i64;` |

**Examples:**
```vela
// Variables
let x = 10;           // immutable
let mut y = 20;       // mutable

// Constants
const PI = 3.14159;
const MAX_SIZE = 1024;

// Function
fn calculate(a: i32, b: i32) -> i32 {
    a + b
}

// Struct
struct User {
    id: UserId,
    name: string,
    email: string,
}

// Enum
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Trait
trait Serialize {
    fn to_json(&self) -> string;
}

// Implementation
impl User {
    fn new(name: string) -> Self {
        User { id: 0, name, email: "" }
    }
}

// Type alias
type UserId = i64;
```

---

### 3. Visibility & Modifier Keywords (6)

| Keyword | Description | Example |
|---------|-------------|---------|
| `pub` | Public visibility | `pub fn public_function() { ... }` |
| `mut` | Mutable binding | `let mut x = 10;` |
| `async` | Asynchronous function | `async fn fetch() { ... }` |
| `static` | Static lifetime / global variable | `static COUNTER: i32 = 0;` |
| `unsafe` | Unsafe code block | `unsafe { ... }` |
| `extern` | External function/library | `extern fn c_function();` |

**Examples:**
```vela
// Public visibility
pub struct PublicStruct { ... }
pub fn public_function() { ... }

// Mutable
let mut counter = 0;
counter += 1;

// Async
async fn fetch_data() -> Result<Data> {
    let response = http_client.get(url).await?;
    response.json().await
}

// Static
static mut GLOBAL_COUNTER: i32 = 0;

// Unsafe (future feature)
unsafe {
    // Low-level operations
}

// Extern (future feature)
extern "C" {
    fn external_function(x: i32) -> i32;
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

### 7. Module System Keywords (5)

| Keyword | Description | Example |
|---------|-------------|---------|
| `import` | Import from module | `import { User } from "./models";` |
| `export` | Export from module | `export fn helper() { ... }` |
| `from` | Source module (used with `import`) | `import { x } from "module";` |
| `as` | Rename import/export | `import { User as UserModel } from "models";` |
| `module` | Module declaration | `module utils { ... }` |

**Examples:**
```vela
// Import
import { User, Post } from "./models";
import * as utils from "./utils";
import config from "./config.json";

// Export
export fn helper() { ... }
export struct Data { ... }
export const MAX = 100;

// Rename
import { VeryLongName as Short } from "module";
export { InternalName as PublicName };

// Module
module database {
    pub fn connect() { ... }
    fn internal_query() { ... }
}
```

---

### 8. Domain-Specific Keywords (25)

These keywords provide built-in support for common architectural patterns and domain-driven design.

#### 8.1 UI Components (2)
| Keyword | Description | Example |
|---------|-------------|---------|
| `widget` | UI widget declaration | `widget Button { ... }` |
| `component` | Reusable component | `component NavBar { ... }` |

#### 8.2 Architecture Layers (4)
| Keyword | Description | Example |
|---------|-------------|---------|
| `service` | Business logic service | `service UserService { ... }` |
| `repository` | Data access layer | `repository UserRepository { ... }` |
| `controller` | Request handler | `controller UserController { ... }` |
| `usecase` | Use case / interactor | `usecase CreateUserUseCase { ... }` |

#### 8.3 Domain Models (4)
| Keyword | Description | Example |
|---------|-------------|---------|
| `dto` | Data Transfer Object | `dto UserDTO { ... }` |
| `entity` | Domain entity | `entity User { ... }` |
| `valueObject` | Value object | `valueObject Email { ... }` |
| `model` | Generic model | `model Product { ... }` |

#### 8.4 Design Patterns (7)
| Keyword | Description | Example |
|---------|-------------|---------|
| `factory` | Factory pattern | `factory UserFactory { ... }` |
| `builder` | Builder pattern | `builder QueryBuilder { ... }` |
| `strategy` | Strategy pattern | `strategy PaymentStrategy { ... }` |
| `observer` | Observer pattern | `observer EventObserver { ... }` |
| `singleton` | Singleton pattern | `singleton Database { ... }` |
| `adapter` | Adapter pattern | `adapter LegacyAdapter { ... }` |
| `decorator` | Decorator pattern | `decorator LoggingDecorator { ... }` |

#### 8.5 Web/API (4)
| Keyword | Description | Example |
|---------|-------------|---------|
| `guard` | Route guard | `guard AuthGuard { ... }` |
| `middleware` | HTTP middleware | `middleware Logger { ... }` |
| `interceptor` | Request/response interceptor | `interceptor AuthInterceptor { ... }` |
| `validator` | Input validator | `validator EmailValidator { ... }` |

#### 8.6 Utilities (4)
| Keyword | Description | Example |
|---------|-------------|---------|
| `pipe` | Data transformation pipeline | `pipe TransformPipe { ... }` |
| `task` | Async task/job | `task EmailTask { ... }` |
| `helper` | Helper utility | `helper DateHelper { ... }` |
| `mapper` | Object mapper | `mapper UserMapper { ... }` |
| `serializer` | Data serializer | `serializer JsonSerializer { ... }` |

**Examples:**
```vela
// Widget
widget Button {
    props: {
        text: string,
        onClick: fn(),
    }
    
    render() {
        <button onclick={props.onClick}>{props.text}</button>
    }
}

// Service
service UserService {
    repository: UserRepository,
    
    fn create_user(&self, data: CreateUserDTO) -> Result<User> {
        let user = User::new(data);
        self.repository.save(user)
    }
}

// Repository
repository UserRepository {
    fn find_by_id(&self, id: UserId) -> Option<User> {
        // Database query
    }
    
    fn save(&mut self, user: User) -> Result<User> {
        // Insert/update
    }
}

// DTO
dto CreateUserDTO {
    name: string,
    email: string,
    password: string,
}

// Entity
entity User {
    id: UserId,
    name: string,
    email: Email,  // ValueObject
    created_at: DateTime,
}

// Factory
factory UserFactory {
    fn create_from_dto(dto: CreateUserDTO) -> User {
        User {
            id: generate_id(),
            name: dto.name,
            email: Email::new(dto.email),
            created_at: now(),
        }
    }
}

// Middleware
middleware Logger {
    fn handle(&self, req: Request, next: fn(Request) -> Response) -> Response {
        log("Request: ${req.path}");
        let response = next(req);
        log("Response: ${response.status}");
        response
    }
}

// Guard
guard AuthGuard {
    fn can_activate(&self, context: Context) -> bool {
        context.user?.is_authenticated() ?? false
    }
}
```

---

### 9. Reactive System Keywords (8)

| Keyword | Description | Example |
|---------|-------------|---------|
| `Signal` | Reactive signal | `let count = Signal(0);` |
| `Computed` | Computed value | `let double = Computed(() => count() * 2);` |
| `Effect` | Side effect | `Effect(() => { print(count()); });` |
| `Watch` | Watch for changes | `Watch(count, (newVal) => { ... });` |
| `store` | Global state store | `store AppStore { ... }` |
| `dispatch` | Dispatch action | `dispatch(INCREMENT);` |
| `provide` | Provide dependency | `provide(UserService, new UserService());` |
| `inject` | Inject dependency | `let service = inject(UserService);` |

**Examples:**
```vela
// Signals
let count = Signal(0);
let name = Signal("Alice");

// Computed
let doubled = Computed(() => count() * 2);
let greeting = Computed(() => "Hello, ${name()}!");

// Effect (runs when dependencies change)
Effect(() => {
    print("Count is now: ${count()}");
});

// Watch (more control than Effect)
Watch(count, (newValue, oldValue) => {
    print("Changed from ${oldValue} to ${newValue}");
});

// Store
store AppStore {
    state: {
        count: 0,
        users: [],
    }
    
    actions: {
        increment() {
            this.state.count += 1;
        }
        
        add_user(user: User) {
            this.state.users.push(user);
        }
    }
}

// Dispatch
dispatch({ type: "INCREMENT" });
dispatch({ type: "ADD_USER", payload: user });

// Dependency Injection
@injectable
service UserService {
    repository: UserRepository = inject(UserRepository),
}

provide(UserRepository, new UserRepository());
let service = inject(UserService);
```

---

### 10. Reserved Keywords (Future Use) (5)

These keywords are reserved for potential future language features and cannot be used as identifiers.

| Keyword | Description | Potential Use |
|---------|-------------|---------------|
| `macro` | Macro system | Compile-time code generation |
| `defer` | Deferred execution | Cleanup code (like Go's defer) |
| `go` | Goroutine-style concurrency | Lightweight threads |
| `chan` | Channel communication | CSP-style message passing |
| `select` | Channel select | Choose from multiple channels |

**Potential Future Syntax:**
```vela
// Macro (hypothetical)
macro debug(expr) {
    print("Debug: ${expr} = ${eval(expr)}");
}

// Defer (hypothetical)
fn process_file() {
    let file = open("data.txt");
    defer file.close();  // Executes when function returns
    
    // Work with file...
}

// Channels (hypothetical)
let ch = chan<i32>();
go {
    ch.send(42);
}
let value = ch.receive();

// Select (hypothetical)
select {
    case msg = ch1.receive() => {
        print("Received from ch1: ${msg}");
    }
    case ch2.send(value) => {
        print("Sent to ch2");
    }
    default => {
        print("No channel ready");
    }
}
```

---

## Contextual Keywords

Some identifiers are **contextual keywords**: they have special meaning only in specific contexts and can be used as regular identifiers elsewhere.

| Keyword | Context | Can be identifier? |
|---------|---------|-------------------|
| `as` | Import/export, type casting | ❌ No (always reserved) |
| `in` | For loops | ❌ No (always reserved) |
| `of` | Iteration (optional syntax) | ✅ Yes (but rare) |
| `ref` | Pattern matching | ✅ Yes (outside patterns) |
| `typeof` | Type queries | ✅ Yes (outside type context) |

**Examples:**
```vela
// 'as' - always reserved
import { User as UserModel } from "models";
let x = value as i64;

// 'in' - always reserved
for item in list { ... }

// 'ref' - contextual
match value {
    ref x => { /* x is a reference */ }
}
let ref = 10;  // OK: 'ref' as variable name

// 'typeof' - contextual
type MyType = typeof(some_value);
let typeof = "string";  // OK: 'typeof' as variable name
```

---

## Conflict Prevention

### Cannot be used as:
- ❌ Variable names: `let if = 10;` → **ERROR**
- ❌ Function names: `fn while() { ... }` → **ERROR**
- ❌ Type names: `struct match { ... }` → **ERROR**
- ❌ Field names: `struct User { let: string }` → **ERROR**
- ❌ Module names: `module fn { ... }` → **ERROR**

### Can be used as:
- ✅ String literals: `let keyword = "if";` → **OK**
- ✅ In comments: `// This is an if statement` → **OK**
- ✅ Raw identifiers (future): `let r#type = 10;` → **OK** (like Rust)

---

## Keyword Grouping by First Letter

| Letter | Keywords |
|--------|----------|
| **A** | `abstract`, `actor`, `adapter`, `as`, `async`, `await` |
| **B** | `boolean`, `break`, `builder` |
| **C** | `catch`, `chan`, `component`, `Computed`, `const`, `continue`, `controller`, `crate` |
| **D** | `decorator`, `defer`, `dispatch`, `dto` |
| **E** | `Effect`, `else`, `entity`, `enum`, `export`, `extern` |
| **F** | `factory`, `false`, `fn`, `for`, `from` |
| **G** | `go`, `guard` |
| **H** | `helper` |
| **I** | `if`, `impl`, `import`, `in`, `inject`, `interceptor`, `interface` |
| **L** | `let`, `loop` |
| **M** | `macro`, `mapper`, `match`, `middleware`, `model`, `module`, `mut` |
| **N** | `new`, `null`, `number` |
| **O** | `observer`, `of` |
| **P** | `pipe`, `provide`, `provider`, `pub` |
| **R** | `ref`, `repository`, `return` |
| **S** | `select`, `self`, `Self`, `serializer`, `service`, `Signal`, `singleton`, `static`, `store`, `strategy`, `string`, `struct`, `super` |
| **T** | `task`, `throw`, `trait`, `true`, `try`, `type`, `typeof` |
| **U** | `unsafe`, `usecase` |
| **V** | `validator`, `valueObject` |
| **W** | `watch`, `Watch`, `while`, `widget` |
| **Y** | `yield` |

---

## Design Rationale

### Why so many domain-specific keywords?

**Pros:**
- ✅ **Clarity:** `service UserService` is clearer than `struct UserService` with comments
- ✅ **Enforces architecture:** Prevents mixing concerns
- ✅ **IDE support:** Better autocomplete and navigation
- ✅ **Code generation:** Can generate boilerplate based on keyword

**Cons:**
- ⚠️ More keywords to learn
- ⚠️ Less flexibility in naming

**Decision:** Vela prioritizes **clarity and architecture enforcement** over minimalism.

### Why separate `Signal` vs `store`?

- `Signal`: Fine-grained reactivity (single value)
- `store`: Global state management (multiple values + actions)

Different use cases warrant different keywords.

### Why reserve keywords for future features?

Prevents breaking changes when features are added. Better to reserve early than break existing code later.

---

## Comparison with Other Languages

| Language | Total Keywords | Notes |
|----------|----------------|-------|
| Vela | 80 | Domain-specific + reactive |
| Rust | 53 | Systems programming focus |
| Python | 35 | Minimal, dynamic |
| JavaScript | 63 (ES2022) | Includes contextual |
| Java | 50 | OOP-focused |
| C++ | 95 | Large, complex |
| Go | 25 | Minimalist philosophy |
| TypeScript | 65+ | JavaScript + types |

Vela has **more keywords** than most languages due to domain-specific and reactive system support, but each keyword serves a clear purpose.

---

## Future Considerations

### Potential Additions
- `macro` - Macro system
- `defer` - Deferred execution
- `go` / `chan` / `select` - CSP-style concurrency
- `union` - Tagged unions (distinct from enum?)
- `interface` - Separate from trait?

### Deprecated/Removed
None yet (Phase 0)

---

**TASK:** TASK-003  
**Historia:** VELA-566 (US-01)  
**Sprint:** Sprint 4 (Phase 0)  
**Status:** Completed ✅  
**Date:** 2025-11-30
