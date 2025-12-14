# Building a Todo App with Vela

This tutorial will guide you through building a complete Todo application using Vela's reactive UI system. We'll cover state management, component composition, local storage, and filtering.

## Table of Contents

1. [Project Setup](#project-setup)
2. [Project Structure](#project-structure)
3. [Core Data Types](#core-data-types)
4. [Todo Store](#todo-store)
5. [UI Components](#ui-components)
6. [Main Application](#main-application)
7. [Adding Features](#adding-features)
8. [Testing](#testing)
9. [Complete Code](#complete-code)

---

## Project Setup

First, create a new Vela project:

```bash
vela new todo-app
cd todo-app
```

Your project structure should look like:

```
todo-app/
├── src/
│   ├── main.vela
│   ├── components/
│   ├── stores/
│   └── types/
├── tests/
│   └── unit/
├── vela.yaml
└── README.md
```

---

## Core Data Types

Let's start by defining our data types. Create `src/types/todo.vela`:

```vela
// Todo item structure
struct Todo {
  id: Number,
  text: String,
  completed: Bool,
  createdAt: DateTime,
  updatedAt: DateTime
}

// Filter options for todo list
enum Filter {
  All,
  Active,
  Completed
}

// Application state
struct AppState {
  todos: Array<Todo>,
  filter: Filter,
  newTodoText: String
}
```

---

## Todo Store

Create `src/stores/todo_store.vela` to manage our application state:

```vela
// Global store for todo state
store TodoStore {
  state todos: Array<Todo> = loadTodos()
  state filter: Filter = All
  state newTodoText: String = ""

  // Computed properties
  computed filteredTodos: Array<Todo> {
    return match filter {
      All => todos
      Active => todos.filter(todo => !todo.completed)
      Completed => todos.filter(todo => todo.completed)
    }
  }

  computed activeCount: Number {
    return todos.filter(todo => !todo.completed).length
  }

  computed completedCount: Number {
    return todos.filter(todo => todo.completed).length
  }

  computed allCompleted: Bool {
    return todos.length > 0 && completedCount == todos.length
  }

  // Actions
  fn addTodo(text: String) {
    if text.trim().isEmpty() { return }

    newTodo = Todo {
      id: generateId(),
      text: text.trim(),
      completed: false,
      createdAt: DateTime.now(),
      updatedAt: DateTime.now()
    }

    todos = todos + newTodo
    newTodoText = ""
    saveTodos()
  }

  fn toggleTodo(id: Number) {
    todos = todos.map(todo =>
      if todo.id == id {
        Todo {
          ...todo,
          completed: !todo.completed,
          updatedAt: DateTime.now()
        }
      } else {
        todo
      }
    )
    saveTodos()
  }

  fn deleteTodo(id: Number) {
    todos = todos.filter(todo => todo.id != id)
    saveTodos()
  }

  fn editTodo(id: Number, newText: String) {
    if newText.trim().isEmpty() { return }

    todos = todos.map(todo =>
      if todo.id == id {
        Todo {
          ...todo,
          text: newText.trim(),
          updatedAt: DateTime.now()
        }
      } else {
        todo
      }
    )
    saveTodos()
  }

  fn toggleAll() {
    allCurrentlyCompleted = allCompleted
    todos = todos.map(todo =>
      Todo {
        ...todo,
        completed: !allCurrentlyCompleted,
        updatedAt: DateTime.now()
      }
    )
    saveTodos()
  }

  fn clearCompleted() {
    todos = todos.filter(todo => !todo.completed)
    saveTodos()
  }

  fn setFilter(newFilter: Filter) {
    filter = newFilter
  }

  fn setNewTodoText(text: String) {
    newTodoText = text
  }

  // Persistence
  fn loadTodos() -> Array<Todo> {
    stored = localStorage.getItem("todos")
    return match stored {
      Some(json) => JSON.parse(json).unwrapOr([])
      None => []
    }
  }

  fn saveTodos() {
    json = JSON.stringify(todos)
    localStorage.setItem("todos", json)
  }

  // Utility
  fn generateId() -> Number {
    return Date.now().toNumber() + Math.random().round()
  }
}
```

---

## UI Components

### TodoItem Component

Create `src/components/todo_item.vela`:

```vela
component TodoItem {
  state todo: Todo
  state isEditing: Bool = false
  state editText: String = ""

  computed isCompleted: Bool {
    return todo.completed
  }

  fn startEdit() {
    isEditing = true
    editText = todo.text
  }

  fn cancelEdit() {
    isEditing = false
    editText = ""
  }

  fn saveEdit() {
    if !editText.trim().isEmpty() {
      TodoStore.editTodo(todo.id, editText)
    }
    isEditing = false
    editText = ""
  }

  fn handleKeyPress(event: KeyEvent) {
    match event.key {
      "Enter" => saveEdit()
      "Escape" => cancelEdit()
      _ => {}
    }
  }

  render {
    return Container(
      style: {
        display: "flex",
        alignItems: "center",
        padding: "12px 0",
        borderBottom: "1px solid #e6e6e6"
      },
      children: [
        if isEditing {
          TextInput(
            value: editText,
            onChange: (value) => editText = value,
            onKeyPress: handleKeyPress,
            onBlur: saveEdit,
            autoFocus: true,
            style: {
              flex: 1,
              padding: "6px",
              border: "1px solid #999",
              borderRadius: "4px"
            }
          )
        } else {
          Fragment(
            children: [
              Checkbox(
                checked: isCompleted,
                onChange: () => TodoStore.toggleTodo(todo.id),
                style: { marginRight: "12px" }
              ),
              Text(
                todo.text,
                style: {
                  flex: 1,
                  textDecoration: if isCompleted { "line-through" } else { "none" },
                  color: if isCompleted { "#d9d9d9" } else { "#4d4d4d" },
                  cursor: "pointer"
                },
                onDoubleClick: startEdit
              ),
              Button(
                "×",
                onClick: () => TodoStore.deleteTodo(todo.id),
                style: {
                  background: "none",
                  border: "none",
                  color: "#cc9a9a",
                  fontSize: "30px",
                  marginLeft: "12px",
                  cursor: "pointer",
                  visibility: "hidden"  // Show on hover
                }
              )
            ]
          )
        }
      ]
    )
  }
}
```

### TodoList Component

Create `src/components/todo_list.vela`:

```vela
component TodoList {
  computed todos: Array<Todo> {
    return TodoStore.filteredTodos
  }

  render {
    return if todos.isEmpty() {
      Container(
        style: {
          textAlign: "center",
          padding: "48px",
          color: "#bfbfbf",
          fontStyle: "italic"
        },
        child: Text("No todos yet. Add one above!")
      )
    } else {
      Column(
        children: todos.map(todo => TodoItem(todo: todo))
      )
    }
  }
}
```

### TodoInput Component

Create `src/components/todo_input.vela`:

```vela
component TodoInput {
  computed newTodoText: String {
    return TodoStore.newTodoText
  }

  fn handleSubmit() {
    TodoStore.addTodo(newTodoText)
  }

  fn handleKeyPress(event: KeyEvent) {
    if event.key == "Enter" {
      handleSubmit()
    }
  }

  render {
    return Container(
      style: {
        width: "100%",
        boxShadow: "0 2px 4px rgba(0,0,0,0.2)"
      },
      child: TextInput(
        value: newTodoText,
        onChange: (value) => TodoStore.setNewTodoText(value),
        onKeyPress: handleKeyPress,
        placeholder: "What needs to be done?",
        style: {
          width: "100%",
          padding: "16px 16px 16px 60px",
          border: "none",
          boxShadow: "inset 0 -2px 1px rgba(0,0,0,0.03)",
          fontSize: "24px",
          outline: "none"
        }
      )
    )
  }
}
```

### FilterBar Component

Create `src/components/filter_bar.vela`:

```vela
component FilterBar {
  computed activeCount: Number {
    return TodoStore.activeCount
  }

  computed completedCount: Number {
    return TodoStore.completedCount
  }

  computed currentFilter: Filter {
    return TodoStore.filter
  }

  computed hasCompleted: Bool {
    return completedCount > 0
  }

  fn setFilter(filter: Filter) {
    TodoStore.setFilter(filter)
  }

  render {
    return Container(
      style: {
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "10px 0",
        color: "#777"
      },
      children: [
        Text("${activeCount} item${if activeCount != 1 { "s" } else { "" }} left"),
        Row(
          children: [
            FilterButton("All", All, currentFilter == All),
            FilterButton("Active", Active, currentFilter == Active),
            FilterButton("Completed", Completed, currentFilter == Completed)
          ],
          spacing: 0
        ),
        if hasCompleted {
          Button(
            "Clear completed",
            onClick: () => TodoStore.clearCompleted(),
            style: {
              background: "none",
              border: "none",
              color: "#777",
              cursor: "pointer",
              textDecoration: "underline"
            }
          )
        } else {
          Container()  // Empty space
        }
      ]
    )
  }
}

component FilterButton {
  state text: String
  state filter: Filter
  state isActive: Bool

  fn handleClick() {
    TodoStore.setFilter(filter)
  }

  render {
    return Button(
      text,
      onClick: handleClick,
      style: {
        background: "none",
        border: "none",
        padding: "3px 7px",
        margin: "3px",
        borderRadius: "3px",
        cursor: "pointer",
        color: if isActive { "#3c3c3c" } else { "#777" },
        border: if isActive { "1px solid rgba(175, 47, 47, 0.2)" } else { "none" }
      }
    )
  }
}
```

### TodoHeader Component

Create `src/components/todo_header.vela`:

```vela
component TodoHeader {
  computed allCompleted: Bool {
    return TodoStore.allCompleted
  }

  computed hasTodos: Bool {
    return TodoStore.todos.length > 0
  }

  render {
    return Container(
      style: {
        position: "relative"
      },
      children: [
        Text(
          "todos",
          style: {
            fontSize: "100px",
            fontWeight: "100",
            textAlign: "center",
            color: "rgba(175, 47, 47, 0.15)",
            margin: "0",
            fontFamily: "Helvetica Neue, Helvetica, Arial, sans-serif"
          }
        ),
        if hasTodos {
          Button(
            "❯",
            onClick: () => TodoStore.toggleAll(),
            style: {
              position: "absolute",
              top: "75px",
              left: "20px",
              background: "none",
              border: "none",
              fontSize: "22px",
              color: "#e6e6e6",
              cursor: "pointer",
              transform: if allCompleted { "rotate(90deg)" } else { "rotate(0deg)" },
              transition: "transform 0.2s ease-in-out"
            }
          )
        }
      ]
    )
  }
}
```

---

## Main Application

Create `src/main.vela`:

```vela
component TodoApp {
  render {
    return Container(
      style: {
        maxWidth: "550px",
        margin: "0 auto",
        fontFamily: "Helvetica Neue, Helvetica, Arial, sans-serif"
      },
      child: Column(
        children: [
          TodoHeader(),
          Container(
            style: {
              background: "white",
              boxShadow: "0 2px 4px 0 rgba(0, 0, 0, 0.2), 0 25px 50px 0 rgba(0, 0, 0, 0.1)"
            },
            children: [
              TodoInput(),
              TodoList(),
              FilterBar()
            ]
          )
        ]
      )
    )
  }
}

// Mount the app
fn main() {
  app = TodoApp()
  mount(app, document.body)
}
```

---

## Adding Features

### Local Storage Persistence

Add this to your `vela.yaml`:

```yaml
dependencies:
  - name: localStorage
    version: "1.0.0"
```

The persistence is already implemented in the TodoStore.

### Keyboard Shortcuts

Add keyboard shortcuts to `src/main.vela`:

```vela
component TodoApp {
  effect {
    // Add global keyboard shortcuts
    document.addEventListener("keydown", (event) => {
      if event.ctrlKey || event.metaKey {
        match event.key {
          "a" => {
            event.preventDefault()
            TodoStore.toggleAll()
          }
          "c" => {
            event.preventDefault()
            TodoStore.clearCompleted()
          }
          _ => {}
        }
      }
    })
  }

  // ... rest of component
}
```

### Search Functionality

Add search to the TodoStore:

```vela
store TodoStore {
  state searchQuery: String = ""

  computed filteredTodos: Array<Todo> {
    baseFiltered = match filter {
      All => todos
      Active => todos.filter(todo => !todo.completed)
      Completed => todos.filter(todo => todo.completed)
    }

    return if searchQuery.isEmpty() {
      baseFiltered
    } else {
      baseFiltered.filter(todo =>
        todo.text.toLowerCase().contains(searchQuery.toLowerCase())
      )
    }
  }

  fn setSearchQuery(query: String) {
    searchQuery = query
  }
}
```

Add search input to the UI:

```vela
component TodoApp {
  render {
    return Container(
      // ... existing styles
      child: Column(
        children: [
          TodoHeader(),
          Container(
            style: { /* existing styles */ },
            children: [
              TodoInput(),
              // Add search input
              Container(
                style: { padding: "10px", borderBottom: "1px solid #e6e6e6" },
                child: TextInput(
                  value: TodoStore.searchQuery,
                  onChange: (value) => TodoStore.setSearchQuery(value),
                  placeholder: "Search todos...",
                  style: {
                    width: "100%",
                    padding: "8px",
                    border: "1px solid #e6e6e6",
                    borderRadius: "4px"
                  }
                )
              ),
              TodoList(),
              FilterBar()
            ]
          )
        ]
      )
    )
  }
}
```

---

## Testing

Create `tests/unit/todo_store_test.vela`:

```vela
@test
fn test_add_todo() -> void {
  // Reset store for test
  TodoStore.todos = []

  TodoStore.addTodo("Test todo")

  assert(TodoStore.todos.length == 1)
  assert(TodoStore.todos[0].text == "Test todo")
  assert(TodoStore.todos[0].completed == false)
}

@test
fn test_toggle_todo() -> void {
  TodoStore.todos = [
    Todo { id: 1, text: "Test", completed: false, createdAt: DateTime.now(), updatedAt: DateTime.now() }
  ]

  TodoStore.toggleTodo(1)

  assert(TodoStore.todos[0].completed == true)

  TodoStore.toggleTodo(1)

  assert(TodoStore.todos[0].completed == false)
}

@test
fn test_delete_todo() -> void {
  TodoStore.todos = [
    Todo { id: 1, text: "Test", completed: false, createdAt: DateTime.now(), updatedAt: DateTime.now() }
  ]

  TodoStore.deleteTodo(1)

  assert(TodoStore.todos.isEmpty())
}

@test
fn test_filtering() -> void {
  TodoStore.todos = [
    Todo { id: 1, text: "Active", completed: false, createdAt: DateTime.now(), updatedAt: DateTime.now() },
    Todo { id: 2, text: "Completed", completed: true, createdAt: DateTime.now(), updatedAt: DateTime.now() }
  ]

  TodoStore.filter = Active
  assert(TodoStore.filteredTodos.length == 1)
  assert(TodoStore.filteredTodos[0].text == "Active")

  TodoStore.filter = Completed
  assert(TodoStore.filteredTodos.length == 1)
  assert(TodoStore.filteredTodos[0].text == "Completed")

  TodoStore.filter = All
  assert(TodoStore.filteredTodos.length == 2)
}

@test
fn test_computed_properties() -> void {
  TodoStore.todos = [
    Todo { id: 1, text: "Active 1", completed: false, createdAt: DateTime.now(), updatedAt: DateTime.now() },
    Todo { id: 2, text: "Active 2", completed: false, createdAt: DateTime.now(), updatedAt: DateTime.now() },
    Todo { id: 3, text: "Completed", completed: true, createdAt: DateTime.now(), updatedAt: DateTime.now() }
  ]

  assert(TodoStore.activeCount == 2)
  assert(TodoStore.completedCount == 1)
  assert(TodoStore.allCompleted == false)

  TodoStore.toggleTodo(1)
  TodoStore.toggleTodo(2)

  assert(TodoStore.allCompleted == true)
}
```

Run tests:

```bash
vela test
```

---

## Complete Code

Here's the complete Todo app code for reference:

### src/types/todo.vela
```vela
struct Todo {
  id: Number,
  text: String,
  completed: Bool,
  createdAt: DateTime,
  updatedAt: DateTime
}

enum Filter {
  All,
  Active,
  Completed
}
```

### src/stores/todo_store.vela
```vela
store TodoStore {
  state todos: Array<Todo> = []
  state filter: Filter = All
  state newTodoText: String = ""

  computed filteredTodos: Array<Todo> {
    return match filter {
      All => todos
      Active => todos.filter(todo => !todo.completed)
      Completed => todos.filter(todo => todo.completed)
    }
  }

  computed activeCount: Number {
    return todos.filter(todo => !todo.completed).length
  }

  fn addTodo(text: String) {
    if text.trim().isEmpty() { return }

    newTodo = Todo {
      id: Date.now().toNumber(),
      text: text.trim(),
      completed: false,
      createdAt: DateTime.now(),
      updatedAt: DateTime.now()
    }

    todos = todos + newTodo
    newTodoText = ""
  }

  fn toggleTodo(id: Number) {
    todos = todos.map(todo =>
      if todo.id == id {
        Todo {
          ...todo,
          completed: !todo.completed,
          updatedAt: DateTime.now()
        }
      } else {
        todo
      }
    )
  }

  fn deleteTodo(id: Number) {
    todos = todos.filter(todo => todo.id != id)
  }

  fn setFilter(newFilter: Filter) {
    filter = newFilter
  }

  fn setNewTodoText(text: String) {
    newTodoText = text
  }
}
```

### src/components/todo_item.vela
```vela
component TodoItem {
  state todo: Todo
  state isEditing: Bool = false
  state editText: String = ""

  fn startEdit() {
    isEditing = true
    editText = todo.text
  }

  fn saveEdit() {
    if !editText.trim().isEmpty() {
      TodoStore.editTodo(todo.id, editText)
    }
    isEditing = false
  }

  render {
    return Container(
      style: { padding: "12px 0", borderBottom: "1px solid #e6e6e6" },
      child: Row(
        children: [
          Checkbox(
            checked: todo.completed,
            onChange: () => TodoStore.toggleTodo(todo.id)
          ),
          if isEditing {
            TextInput(
              value: editText,
              onChange: (value) => editText = value,
              onBlur: saveEdit,
              autoFocus: true
            )
          } else {
            Text(
              todo.text,
              onDoubleClick: startEdit,
              style: {
                textDecoration: if todo.completed { "line-through" } else { "none" }
              }
            )
          },
          Button("×", onClick: () => TodoStore.deleteTodo(todo.id))
        ]
      )
    )
  }
}
```

### src/main.vela
```vela
component TodoApp {
  render {
    return Container(
      style: { maxWidth: "550px", margin: "0 auto" },
      child: Column(
        children: [
          Text("Todo App", style: { fontSize: "24px", textAlign: "center" }),
          TodoInput(),
          TodoList(),
          FilterBar()
        ]
      )
    )
  }
}

fn main() {
  mount(TodoApp(), document.body)
}
```

---

## Next Steps

Now that you have a working Todo app, try these enhancements:

1. **Categories/Tags**: Add categories to todos
2. **Due Dates**: Add due date functionality
3. **Priority Levels**: Add priority (high, medium, low)
4. **Drag & Drop**: Reorder todos with drag and drop
5. **Offline Support**: Add service worker for offline functionality
6. **Multi-user**: Add user authentication and shared todos

The complete source code for this tutorial is available in the `examples/todo-app/` directory.