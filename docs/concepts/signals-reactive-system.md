# Signals and Reactive System

Vela's reactive system is built around **signals** - observable values that automatically propagate changes to dependent computations and effects. This creates a declarative, efficient, and composable way to handle state and side effects.

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Signals](#signals)
3. [Computed Values](#computed-values)
4. [Effects](#effects)
5. [Advanced Patterns](#advanced-patterns)
6. [Performance](#performance)
7. [Best Practices](#best-practices)

---

## Core Concepts

### Reactive Programming

Reactive programming is a paradigm where programs react to changes in data automatically. Instead of manually updating dependent values, you declare relationships and let the system handle propagation.

**Imperative (manual):**
```javascript
let count = 0
let doubled = count * 2
let display = `Count: ${count}, Doubled: ${doubled}`

function increment() {
  count++
  doubled = count * 2  // Manual update
  display = `Count: ${count}, Doubled: ${doubled}`  // Manual update
  updateUI(display)  // Manual UI update
}
```

**Reactive (automatic):**
```vela
state count: Number = 0
computed doubled: Number { return count * 2 }
computed display: String { return "Count: ${count}, Doubled: ${doubled}" }

effect {
  updateUI(display)
}

fn increment() {
  count = count + 1  // Everything updates automatically
}
```

### Key Benefits

1. **Automatic Updates**: No manual dependency tracking
2. **Declarative**: Express what, not how
3. **Efficient**: Only recompute what's necessary
4. **Composable**: Easy to combine and reuse
5. **Type Safe**: Compile-time guarantees

---

## Signals

Signals are the foundation of Vela's reactive system. They represent observable values that notify subscribers when they change.

### Creating Signals

```vela
// Basic signal
state count: Number = 0

// Signal with initial value
state name: String = "Alice"

// Signal from computation
state userId: Number = getCurrentUserId()
```

### Reading Signals

```vela
// Direct access
currentCount = count

// In expressions
message = "Count is ${count}"
```

### Writing Signals

```vela
// Direct assignment
count = 5

// Functional update
count = count + 1

// Using update method
count.update(current => current * 2)
```

### Signal API

```vela
interface Signal<T> {
  // Read
  get() -> T

  // Write
  set(value: T) -> void
  update(f: (T) -> T) -> void

  // Subscribe
  subscribe(listener: (T) -> void) -> Subscription
}
```

---

## Computed Values

Computed values are signals derived from other signals. They automatically update when their dependencies change.

### Basic Computed

```vela
state firstName: String = "John"
state lastName: String = "Doe"

computed fullName: String {
  return "${firstName} ${lastName}"
}

computed greeting: String {
  return "Hello, ${fullName}!"
}
```

### Computed with Complex Logic

```vela
state items: Array<Item> = []

computed totalPrice: Float {
  return items
    .map(item => item.price * item.quantity)
    .reduce(0, (sum, price) => sum + price)
}

computed hasDiscount: Bool {
  return totalPrice > 100
}

computed finalPrice: Float {
  basePrice = totalPrice
  discount = if hasDiscount { basePrice * 0.1 } else { 0 }
  return basePrice - discount
}
```

### Lazy Evaluation

Computed values are lazy - they only compute when read and cache results until dependencies change.

```vela
computed expensiveCalculation: Number {
  print("Computing...")  // Only prints when accessed
  return heavyComputation()
}

// Nothing happens here
someSignal = Signal.new(42)

// Triggers computation
result = expensiveCalculation  // Prints "Computing..."

// Cached - no recomputation
another = expensiveCalculation  // No print

// Dependency changes - invalidates cache
someSignal.set(43)

// Triggers recomputation
final = expensiveCalculation  // Prints "Computing..." again
```

---

## Effects

Effects are functions that run automatically when their dependencies change. They're used for side effects like DOM updates, API calls, or logging.

### Basic Effects

```vela
state count: Number = 0

effect {
  print("Count changed to: ${count}")
}

// This will print: "Count changed to: 1"
count = 1

// This will print: "Count changed to: 5"
count = 5
```

### Effects with Dependencies

```vela
state user: User = { name: "Alice", age: 30 }
state theme: String = "light"

effect {
  // Only runs when user.name changes
  print("User name: ${user.name}")
}

effect {
  // Only runs when theme changes
  updateTheme(theme)
}
```

### Cleanup Effects

```vela
state isVisible: Bool = true

effect {
  if isVisible {
    startAnimation()
    return () => stopAnimation()  // Cleanup function
  }
}
```

### Effect API

```vela
interface Effect {
  stop() -> void
}

Effect.run(action: () -> void) -> Effect
Effect.run(dependencies: Array<Signal>, action: () -> void) -> Effect
```

---

## Advanced Patterns

### Signal Composition

```vela
// Combine multiple signals
state a: Number = 1
state b: Number = 2
state c: Number = 3

computed sum: Number { return a + b + c }
computed product: Number { return a * b * c }
computed average: Float { return sum / 3.0 }
```

### Reactive Collections

```vela
state todos: Array<Todo> = []

computed completedTodos: Array<Todo> {
  return todos.filter(todo => todo.completed)
}

computed pendingCount: Number {
  return todos.length - completedTodos.length
}

computed progress: Float {
  if todos.isEmpty { return 0.0 }
  return completedTodos.length.toFloat() / todos.length.toFloat()
}
```

### Async Reactivity

```vela
state userId: Number = 0

computed userData: Option<User> {
  if userId == 0 { return None }
  // In real app, this would trigger async fetch
  return fetchUser(userId)
}

effect {
  match userData {
    Some(user) => updateUI(user)
    None => showLoading()
  }
}
```

### Reactive Validation

```vela
state email: String = ""
state password: String = ""

computed isEmailValid: Bool {
  return email.contains("@") && email.contains(".")
}

computed isPasswordValid: Bool {
  return password.length >= 8
}

computed canSubmit: Bool {
  return isEmailValid && isPasswordValid
}

computed validationErrors: Array<String> {
  errors = []
  if !isEmailValid { errors = errors + ["Invalid email"] }
  if !isPasswordValid { errors = errors + ["Password too short"] }
  return errors
}

effect {
  submitButton.enabled = canSubmit
  errorList.items = validationErrors
}
```

### State Machines

```vela
enum AppState {
  Loading,
  Ready,
  Error(message: String)
}

state appState: AppState = Loading

computed isLoading: Bool {
  return match appState {
    Loading => true
    _ => false
  }
}

computed errorMessage: Option<String> {
  return match appState {
    Error(msg) => Some(msg)
    _ => None
  }
}

effect {
  if isLoading {
    showSpinner()
  } else {
    hideSpinner()
  }
}

effect {
  match errorMessage {
    Some(msg) => showError(msg)
    None => hideError()
  }
}
```

---

## Performance

### Change Propagation

Vela uses a **push-pull** model:

1. **Push**: Changes propagate from signals to dependents
2. **Pull**: Computed values recalculate when accessed

### Optimizations

#### Glitch-Free Updates

Vela ensures updates are glitch-free - intermediate inconsistent states are never visible.

```vela
state a: Number = 0
computed b: Number { return a + 1 }
computed c: Number { return b + 1 }

// When a changes to 5:
// - b temporarily becomes 6
// - c temporarily becomes 7
// But observers never see these intermediate states
```

#### Lazy Evaluation

Computed values only recalculate when accessed, and cache results.

#### Batch Updates

Multiple synchronous changes are batched:

```vela
effect {
  print("Effect ran")
}

// Only prints once, not three times
a = 1
b = 2
c = 3
```

### Memory Management

Signals use weak references and automatic cleanup to prevent memory leaks.

---

## Best Practices

### 1. Use Computed for Derived State

```vela
// ✅ Good
state items: Array<Item> = []
computed total: Number { return items.sum(item => item.price) }

// ❌ Avoid
state items: Array<Item> = []
state total: Number = 0

effect {
  total = items.sum(item => item.price)  // Manual sync
}
```

### 2. Avoid Side Effects in Computed

```vela
// ✅ Pure computed
computed displayName: String {
  return "${user.firstName} ${user.lastName}".trim()
}

// ❌ Side effects in computed
computed userData: User {
  logAccess(user.id)  // Side effect!
  return user
}
```

### 3. Use Effects for Side Effects

```vela
// ✅ Effects for side effects
effect {
  saveToLocalStorage("user", user)
}

effect {
  updateDocumentTitle(pageTitle)
}
```

### 4. Prefer Signal Composition

```vela
// ✅ Composable
computed fullName: String { return "${firstName} ${lastName}" }
computed greeting: String { return "Hello, ${fullName}!" }

// ❌ Nested
computed greeting: String {
  fullName = "${firstName} ${lastName}"
  return "Hello, ${fullName}!"
}
```

### 5. Handle Async Properly

```vela
// ✅ Async in effects
effect {
  async fetchUserData(userId).then(data => {
    userData.set(data)
  })
}

// ❌ Async in computed (won't work as expected)
computed userData: User {
  return await fetchUserData(userId)  // Won't update reactively
}
```

### 6. Use Memo for Expensive Computations

```vela
// For expensive computations that don't change often
memo fibonacciCache: Map<Number, Number> { return Map.empty() }

computed fibonacci: Number {
  return memo {
    // Expensive calculation
    if n <= 1 { return n }
    return fibonacci(n - 1) + fibonacci(n - 2)
  }
}
```

### 7. Clean Up Effects

```vela
effect {
  subscription = websocket.subscribe("messages", handleMessage)
  return () => subscription.unsubscribe()  // Cleanup
}
```

### 8. Avoid Deep Nesting

```vela
// ✅ Flat structure
state ui: UIState = { modal: { form: { field: "" } } }
computed fieldValue: String { return ui.modal.form.field }

// ❌ Deep nesting (hard to track)
computed deepValue: String {
  return some.very.deep.nested.value.that.changes.often
}
```

---

## Common Patterns

### Form Validation

```vela
struct FormData {
  email: String,
  password: String,
  confirmPassword: String
}

state form: FormData = { email: "", password: "", confirmPassword: "" }

computed isEmailValid: Bool {
  return form.email.contains("@")
}

computed isPasswordValid: Bool {
  return form.password.length >= 8
}

computed doPasswordsMatch: Bool {
  return form.password == form.confirmPassword
}

computed isFormValid: Bool {
  return isEmailValid && isPasswordValid && doPasswordsMatch
}

computed validationErrors: Array<String> {
  errors = []
  if !isEmailValid { errors = errors + ["Invalid email"] }
  if !isPasswordValid { errors = errors + ["Password too short"] }
  if !doPasswordsMatch { errors = errors + ["Passwords don't match"] }
  return errors
}
```

### API State Management

```vela
enum ApiState<T> {
  Idle,
  Loading,
  Success(data: T),
  Error(message: String)
}

state apiState: ApiState<User> = Idle

computed isLoading: Bool {
  return match apiState {
    Loading => true
    _ => false
  }
}

computed userData: Option<User> {
  return match apiState {
    Success(data) => Some(data)
    _ => None
  }
}

computed errorMessage: Option<String> {
  return match apiState {
    Error(msg) => Some(msg)
    _ => None
  }
}

fn fetchUser(id: Number) {
  apiState = Loading

  async fetchUserFromAPI(id).then(result => {
    apiState = match result {
      Ok(data) => Success(data)
      Err(err) => Error(err.message)
    }
  })
}
```

### Reactive UI Components

```vela
component Counter {
  state count: Number = 0

  computed isEven: Bool { return count % 2 == 0 }

  effect {
    // Update document title
    document.title = "Count: ${count}"
  }

  fn increment() {
    count = count + 1
  }

  fn decrement() {
    count = count - 1
  }

  render {
    return Container(
      padding: 20,
      child: Column(
        children: [
          Text("Counter: ${count}", style: { fontSize: 24 }),
          Text(
            if isEven { "Even" } else { "Odd" },
            style: { color: if isEven { "green" } else { "red" } }
          ),
          Row(
            children: [
              Button("Decrement", onClick: decrement),
              Button("Increment", onClick: increment)
            ]
          )
        ]
      )
    )
  }
}
```

---

Signals provide a powerful, efficient, and declarative way to handle state and side effects in Vela applications. By understanding these concepts and following the best practices, you can build complex reactive applications with minimal boilerplate and maximum maintainability.