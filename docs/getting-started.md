# Getting Started with Vela

**Time to complete:** 25 minutes
**Prerequisites:** None (Vela CLI will be installed)

Welcome to Vela! This guide will get you up and running with your first Vela program in under 30 minutes.

---

## Table of Contents

1. [Installation](#1-installation)
2. [Your First Program](#2-your-first-program)
3. [Basic Concepts](#3-basic-concepts)
4. [Functional Programming](#4-functional-programming)
5. [Reactivity](#5-reactivity)
6. [UI Basics](#6-ui-basics)
7. [Your First Project](#7-your-first-project)
8. [Next Steps](#8-next-steps)

---

## 1. Installation

### 1.1 System Requirements

- **Operating System:** Windows 10+, macOS 10.15+, Ubuntu 18.04+
- **Memory:** 4GB RAM minimum
- **Disk Space:** 500MB free space

### 1.2 Install Vela CLI

#### Windows (PowerShell)
```powershell
# Download and install
Invoke-WebRequest -Uri "https://github.com/velalang/vela/releases/latest/download/vela-windows-x64.msi" -OutFile "vela.msi"
Start-Process msiexec.exe -ArgumentList "/i vela.msi /quiet" -Wait

# Add to PATH (if not done automatically)
$env:Path += ";C:\Program Files\Vela\bin"
```

#### macOS
```bash
# Using Homebrew
brew install velalang/tap/vela

# Or download manually
curl -L https://github.com/velalang/vela/releases/latest/download/vela-macos-x64.pkg -o vela.pkg
sudo installer -pkg vela.pkg -target /
```

#### Linux (Ubuntu/Debian)
```bash
# Add repository
curl -fsSL https://github.com/velalang/vela/releases/latest/download/vela-linux-x64.deb -o vela.deb
sudo dpkg -i vela.deb
sudo apt-get install -f
```

### 1.3 Verify Installation

Open a terminal and run:

```bash
vela --version
```

You should see output like:
```
Vela 1.0.0
```

---

## 2. Your First Program

### 2.1 Create Your First File

Create a file called `hello.vela`:

```vela
// hello.vela
fn main() -> void {
  print("Hello, Vela!")
}
```

### 2.2 Run Your Program

```bash
vela run hello.vela
```

**Output:**
```
Hello, Vela!
```

**Congratulations!** You've just run your first Vela program.

### 2.3 Understanding the Code

- `fn main() -> void`: Defines a function named `main` that returns nothing
- `print("Hello, Vela!")`: Calls the built-in `print` function
- No semicolons needed (except in some cases)
- Functions are declared with `fn`, parameters in parentheses, return type after `->`

---

## 3. Basic Concepts

### 3.1 Variables and Types

Vela variables are **immutable by default**:

```vela
// Variables are immutable by default
name: String = "Alice"
age: Number = 30

// This would cause a compile error:
// name = "Bob"  // Error: cannot mutate immutable variable

// For mutable variables, use 'state'
state counter: Number = 0
counter = counter + 1  // OK: state variables are mutable
```

**Built-in Types:**
- `Number`: 64-bit integers
- `Float`: 64-bit floating point
- `String`: UTF-8 text
- `Bool`: true/false

### 3.2 Functions

```vela
// Simple function
fn greet(name: String) -> String {
  return "Hello, ${name}!"
}

// Function with multiple parameters
fn add(x: Number, y: Number) -> Number {
  return x + y
}

// Calling functions
fn main() -> void {
  message = greet("World")
  result = add(5, 3)
  print("${message} The answer is ${result}")
}
```

**Output:**
```
Hello, World! The answer is 8
```

### 3.3 Control Flow

#### If Expressions

```vela
fn checkAge(age: Number) -> String {
  if age >= 18 {
    return "Adult"
  } else {
    return "Minor"
  }
}
```

#### Match Expressions (Pattern Matching)

```vela
fn describeNumber(n: Number) -> String {
  match n {
    0 => "zero"
    1 => "one"
    2 => "two"
    _ => "other"  // catch-all pattern
  }
}
```

### 3.4 Collections

```vela
fn main() -> void {
  // Arrays
  numbers: Array<Number> = [1, 2, 3, 4, 5]

  // Array operations
  doubled = numbers.map(x => x * 2)
  evens = numbers.filter(x => x % 2 == 0)

  print("Original: ${numbers}")
  print("Doubled: ${doubled}")
  print("Evens: ${evens}")
}
```

**Output:**
```
Original: [1, 2, 3, 4, 5]
Doubled: [2, 4, 6, 8, 10]
Evens: [2, 4]
```

---

## 4. Functional Programming

Vela is designed for functional programming. Instead of loops, use functional methods:

### 4.1 Pure Functions

```vela
// Pure function: no side effects, same input = same output
fn calculateTax(amount: Float, rate: Float) -> Float {
  return amount * rate
}

// Impure function (has side effect)
fn printReceipt(amount: Float) -> void {
  print("Total: $${amount}")
}
```

### 4.2 Functional Collection Operations

```vela
fn main() -> void {
  products = [
    { name: "Laptop", price: 999.99 },
    { name: "Mouse", price: 29.99 },
    { name: "Keyboard", price: 79.99 }
  ]

  // Filter expensive products
  expensive = products.filter(p => p.price > 100)

  // Calculate total
  total = products.map(p => p.price).reduce((sum, price) => sum + price, 0)

  // Transform to names
  names = products.map(p => p.name)

  print("Expensive products: ${expensive}")
  print("Total: $${total}")
  print("Product names: ${names}")
}
```

**Output:**
```
Expensive products: [{name: "Laptop", price: 999.99}]
Total: $1109.97
Product names: ["Laptop", "Mouse", "Keyboard"]
```

### 4.3 No Loops, Only Functions

**❌ Don't do this (imperative loops don't exist):**
```vela
// This won't compile - no for/while loops!
for i in 0..10 {
  print(i)
}
```

**✅ Do this (functional approach):**
```vela
// Use functional methods instead
(0..10).forEach(i => print(i))

// Or recursion for complex cases
fn printNumbers(n: Number) -> void {
  if n <= 10 {
    print(n)
    printNumbers(n + 1)
  }
}
```

---

## 5. Reactivity

Vela has built-in reactivity for handling state changes:

### 5.1 Signals

```vela
fn main() -> void {
  // Reactive state
  state count: Number = 0

  // Computed value (reacts to changes)
  computed doubled: Number { return count * 2 }

  // Effect (runs when dependencies change)
  effect {
    print("Count: ${count}, Doubled: ${doubled}")
  }

  // Change the state
  count = 5
  // Effect automatically runs: "Count: 5, Doubled: 10"

  count = 10
  // Effect automatically runs: "Count: 10, Doubled: 20"
}
```

### 5.2 Reactive Data Flow

```vela
fn main() -> void {
  // User data
  state user: { name: String, age: Number } = {
    name: "Alice",
    age: 25
  }

  // Computed properties
  computed displayName: String { return "User: ${user.name}" }
  computed isAdult: Bool { return user.age >= 18 }

  // Watch for changes
  watch(user.name) {
    print("Name changed to: ${user.name}")
  }

  effect {
    if isAdult {
      print("${displayName} is an adult")
    } else {
      print("${displayName} is a minor")
    }
  }

  // Change triggers everything
  user.name = "Bob"
  user.age = 30
}
```

---

## 6. UI Basics

Vela includes a declarative UI system:

### 6.1 Basic Widgets

```vela
import 'system:ui'

fn main() -> void {
  // Create a simple UI
  app = Container(
    padding: 20,
    child: Column(
      children: [
        Text("Welcome to Vela!", style: { fontSize: 24 }),
        SizedBox(height: 20),
        Text("This is your first UI app"),
        SizedBox(height: 20),
        ElevatedButton(
          text: "Click me!",
          onPressed: () => print("Button clicked!")
        )
      ]
    )
  )

  // Run the app
  runApp(app)
}
```

### 6.2 Reactive UI

```vela
import 'system:ui'

fn main() -> void {
  state counter: Number = 0

  app = Container(
    padding: 20,
    child: Column(
      children: [
        Text("Counter: ${counter}", style: { fontSize: 24 }),
        SizedBox(height: 20),
        Row(
          children: [
            ElevatedButton(
              text: "Increment",
              onPressed: () => { counter = counter + 1 }
            ),
            SizedBox(width: 10),
            ElevatedButton(
              text: "Decrement",
              onPressed: () => { counter = counter - 1 }
            )
          ]
        )
      ]
    )
  )

  runApp(app)
}
```

---

## 7. Your First Project

Let's create a complete Todo application:

### 7.1 Project Structure

```bash
my-todo-app/
├── vela.yaml
├── src/
│   ├── main.vela
│   ├── todo.vela
│   └── ui.vela
└── tests/
    └── todo_test.vela
```

### 7.2 vela.yaml

```yaml
name: my-todo-app
version: 0.1.0
main: src/main.vela

dependencies:
  - system:ui
  - system:reactive
```

### 7.3 Todo Logic (src/todo.vela)

```vela
// Todo item structure
struct TodoItem {
  id: Number,
  text: String,
  completed: Bool
}

// Todo list state
state todos: Array<TodoItem> = []
state nextId: Number = 1

// Computed: incomplete todos
computed incompleteTodos: Array<TodoItem> {
  return todos.filter(todo => !todo.completed)
}

// Add a new todo
fn addTodo(text: String) -> void {
  newTodo = TodoItem {
    id: nextId,
    text: text,
    completed: false
  }
  todos = todos + [newTodo]  // Functional append
  nextId = nextId + 1
}

// Toggle todo completion
fn toggleTodo(id: Number) -> void {
  todos = todos.map(todo =>
    if todo.id == id {
      TodoItem { id: todo.id, text: todo.text, completed: !todo.completed }
    } else {
      todo
    }
  )
}

// Remove completed todos
fn clearCompleted() -> void {
  todos = todos.filter(todo => !todo.completed)
}
```

### 7.4 UI (src/ui.vela)

```vela
import 'system:ui'
import 'todo'

fn buildUI() -> Widget {
  return Container(
    padding: 20,
    child: Column(
      children: [
        Text("My Todo App", style: { fontSize: 24, fontWeight: "bold" }),
        SizedBox(height: 20),

        // Add todo input
        Row(
          children: [
            Expanded(
              child: TextField(
                placeholder: "What needs to be done?",
                onSubmitted: (text) => addTodo(text)
              )
            )
          ]
        ),
        SizedBox(height: 20),

        // Todo list
        Expanded(
          child: ListView(
            children: todos.map(todo => TodoItemWidget(todo))
          )
        ),

        // Footer
        if todos.length > 0 {
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text("${incompleteTodos.length} items left"),
              TextButton(
                text: "Clear completed",
                onPressed: () => clearCompleted()
              )
            ]
          )
        }
      ]
    )
  )
}

component TodoItemWidget(todo: TodoItem) -> Widget {
  return Row(
    children: [
      Checkbox(
        value: todo.completed,
        onChanged: (checked) => toggleTodo(todo.id)
      ),
      SizedBox(width: 10),
      Expanded(
        child: Text(
          todo.text,
          style: if todo.completed {
            { textDecoration: "line-through", color: Colors.grey }
          } else {
            {}
          }
        )
      )
    ]
  )
}
```

### 7.5 Main App (src/main.vela)

```vela
import 'system:ui'
import 'ui'

fn main() -> void {
  app = MaterialApp(
    title: "My Todo App",
    home: buildUI()
  )

  runApp(app)
}
```

### 7.6 Run the App

```bash
cd my-todo-app
vela run
```

### 7.7 Add Tests (tests/todo_test.vela)

```vela
import 'system:testing'
import 'todo'

describe("Todo App", () => {
  beforeEach(() => {
    // Reset state before each test
    todos = []
    nextId = 1
  })

  it("should add a todo", () => {
    addTodo("Learn Vela")

    expect(todos.length).toBe(1)
    expect(todos[0].text).toBe("Learn Vela")
    expect(todos[0].completed).toBe(false)
  })

  it("should toggle todo completion", () => {
    addTodo("Test toggling")
    toggleTodo(1)

    expect(todos[0].completed).toBe(true)
  })

  it("should clear completed todos", () => {
    addTodo("Task 1")
    addTodo("Task 2")
    toggleTodo(1)  // Complete first task

    clearCompleted()

    expect(todos.length).toBe(1)
    expect(todos[0].text).toBe("Task 2")
  })

  it("should compute incomplete todos", () => {
    addTodo("Task 1")
    addTodo("Task 2")
    addTodo("Task 3")
    toggleTodo(1)
    toggleTodo(3)

    expect(incompleteTodos.length).toBe(1)
    expect(incompleteTodos[0].text).toBe("Task 2")
  })
})
```

### 7.8 Run Tests

```bash
vela test
```

---

## 8. Next Steps

Now that you know the basics, here are some next steps:

### 8.1 Learn More

- **Language Reference:** Read the [complete language specification](language-specification.md)
- **API Documentation:** Explore the [standard library APIs](api-reference.md)
- **Advanced Topics:** Learn about [actors](actors-concurrency.md) and [signals](signals-reactive-system.md)

### 8.2 Build Something

Try building:
- A calculator app
- A weather dashboard
- A simple game
- A REST API

### 8.3 Join the Community

- **GitHub:** [github.com/velalang/vela](https://github.com/velalang/vela)
- **Discord:** Join our community server
- **Documentation:** [docs.velalang.org](https://docs.velalang.org)

### 8.4 Example Projects

Check out these example projects in the `examples/` directory:
- `examples/hello-world.vela` - Basic examples
- `examples/calculator-app.vela` - Complete calculator
- `examples/todo-app.vela` - Todo application
- `examples/reactive-ui.vela` - Reactive UI examples

---

**Congratulations!** You've completed the Vela Getting Started guide. You now know how to:

- ✅ Install and run Vela
- ✅ Write basic programs
- ✅ Use functional programming
- ✅ Build reactive applications
- ✅ Create user interfaces
- ✅ Structure projects
- ✅ Write and run tests

Happy coding with Vela! 🚀