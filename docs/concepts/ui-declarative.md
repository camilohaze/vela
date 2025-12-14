# Declarative UI

Vela's UI system is built around declarative, reactive components that automatically update when their state changes. This approach eliminates manual DOM manipulation and provides a composable, maintainable way to build user interfaces.

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Basic Widgets](#basic-widgets)
3. [Layout System](#layout-system)
4. [State Management](#state-management)
5. [Event Handling](#event-handling)
6. [Styling](#styling)
7. [Advanced Patterns](#advanced-patterns)
8. [Performance](#performance)
9. [Best Practices](#best-practices)

---

## Core Concepts

### Declarative Programming

Instead of manually manipulating DOM elements, you declare what the UI should look like based on current state. The framework handles the rest.

**Imperative (manual):**
```javascript
function updateCounter(count) {
  const element = document.getElementById('counter');
  element.textContent = `Count: ${count}`;
  
  const button = document.getElementById('increment');
  button.addEventListener('click', () => {
    count++;
    updateCounter(count);  // Manual update
  });
}
```

**Declarative (automatic):**
```vela
component Counter {
  state count: Number = 0

  fn increment() {
    count = count + 1
  }

  render {
    return Column(
      children: [
        Text("Count: ${count}"),
        Button("Increment", onClick: increment)
      ]
    )
  }
}
```

### Component Architecture

Components are the building blocks of Vela UIs:

- **Stateless Components**: Pure functions of props
- **Stateful Components**: Manage internal state
- **Composite Components**: Combine other components

### Reactive Updates

UI automatically updates when state changes through Vela's signal system.

---

## Basic Widgets

### Text Widget

```vela
// Simple text
Text("Hello, World!")

// Styled text
Text("Important message", style: {
  fontSize: 18,
  fontWeight: "bold",
  color: "red"
})

// Dynamic text
state name: String = "Alice"
Text("Hello, ${name}!")
```

### Button Widget

```vela
// Basic button
Button("Click me", onClick: () => print("Clicked!"))

// Button with styling
Button("Submit", 
  onClick: submitForm,
  style: {
    backgroundColor: "blue",
    color: "white",
    padding: 10,
    borderRadius: 5
  }
)

// Disabled button
Button("Loading...", disabled: true)
```

### Input Widgets

```vela
// Text input
TextInput(
  value: name,
  onChange: (newValue) => name = newValue,
  placeholder: "Enter your name"
)

// Number input
NumberInput(
  value: age,
  onChange: (newValue) => age = newValue,
  min: 0,
  max: 120
)

// Checkbox
Checkbox(
  checked: agreed,
  onChange: (checked) => agreed = checked,
  label: "I agree to terms"
)

// Select dropdown
Select(
  value: selectedOption,
  options: ["Option 1", "Option 2", "Option 3"],
  onChange: (value) => selectedOption = value
)
```

### Container Widget

```vela
// Basic container
Container(
  child: Text("Centered content"),
  padding: 20,
  alignment: Alignment.center
)

// With background
Container(
  child: Text("Content"),
  style: {
    backgroundColor: "lightgray",
    borderRadius: 8
  }
)
```

---

## Layout System

### Column Layout

```vela
Column(
  children: [
    Text("Header"),
    Text("Content 1"),
    Text("Content 2"),
    Button("Action")
  ],
  spacing: 10,
  alignment: Alignment.start
)
```

### Row Layout

```vela
Row(
  children: [
    Text("Label:"),
    TextInput(value: name, onChange: updateName),
    Button("Submit")
  ],
  spacing: 8,
  alignment: Alignment.center
)
```

### Grid Layout

```vela
Grid(
  children: [
    Text("Cell 1"), Text("Cell 2"),
    Text("Cell 3"), Text("Cell 4")
  ],
  columns: 2,
  spacing: 10
)
```

### Flex Layout

```vela
Flex(
  direction: FlexDirection.row,
  children: [
    Flexible(child: Text("Flexible content"), flex: 1),
    SizedBox(width: 10),
    Button("Fixed button")
  ],
  alignment: Alignment.spaceBetween
)
```

### Stack Layout

```vela
Stack(
  children: [
    // Background
    Container(
      width: 200,
      height: 200,
      style: { backgroundColor: "lightblue" }
    ),
    // Foreground
    Center(child: Text("Overlay text"))
  ]
)
```

---

## State Management

### Component State

```vela
component Counter {
  state count: Number = 0

  fn increment() {
    count = count + 1
  }

  fn decrement() {
    count = count - 1
  }

  render {
    return Column(
      children: [
        Text("Count: ${count}"),
        Row(
          children: [
            Button("-", onClick: decrement),
            Button("+", onClick: increment)
          ]
        )
      ]
    )
  }
}
```

### Computed Properties

```vela
component TodoItem {
  state text: String
  state completed: Bool = false

  computed isEmpty: Bool {
    return text.trim().isEmpty()
  }

  computed displayText: String {
    return if completed { "✓ ${text}" } else { text }
  }

  render {
    return Row(
      children: [
        Checkbox(
          checked: completed,
          onChange: (checked) => completed = checked
        ),
        Text(displayText, 
          style: if completed { 
            { textDecoration: "line-through", color: "gray" } 
          } else { 
            {} 
          }
        )
      ]
    )
  }
}
```

### Shared State

```vela
// Global state
store AppState {
  state theme: String = "light"
  state user: Option<User> = None
}

// Component using global state
component App {
  computed isDark: Bool {
    return AppState.theme == "dark"
  }

  fn toggleTheme() {
    AppState.theme = if AppState.theme == "dark" { "light" } else { "dark" }
  }

  render {
    return Container(
      style: {
        backgroundColor: if isDark { "black" } else { "white" },
        color: if isDark { "white" } else { "black" }
      },
      child: Button("Toggle Theme", onClick: toggleTheme)
    )
  }
}
```

### Form State

```vela
component LoginForm {
  state email: String = ""
  state password: String = ""
  state isSubmitting: Bool = false

  computed isValid: Bool {
    return !email.isEmpty() && !password.isEmpty() && email.contains("@")
  }

  computed canSubmit: Bool {
    return isValid && !isSubmitting
  }

  async fn submit() {
    if !canSubmit { return }
    
    isSubmitting = true
    
    try {
      result = await login(email, password)
      match result {
        Ok(user) => navigateToDashboard(user)
        Err(error) => showError(error.message)
      }
    } finally {
      isSubmitting = false
    }
  }

  render {
    return Column(
      children: [
        TextInput(
          value: email,
          onChange: (value) => email = value,
          placeholder: "Email"
        ),
        TextInput(
          value: password,
          onChange: (value) => password = value,
          placeholder: "Password",
          type: "password"
        ),
        Button(
          if isSubmitting { "Logging in..." } else { "Login" },
          onClick: submit,
          disabled: !canSubmit
        )
      ],
      spacing: 10
    )
  }
}
```

---

## Event Handling

### Click Events

```vela
component ClickExample {
  state clickCount: Number = 0

  fn handleClick() {
    clickCount = clickCount + 1
  }

  fn handleDoubleClick() {
    clickCount = clickCount + 10
  }

  render {
    return Column(
      children: [
        Text("Clicks: ${clickCount}"),
        Button("Click me", 
          onClick: handleClick,
          onDoubleClick: handleDoubleClick
        )
      ]
    )
  }
}
```

### Form Events

```vela
component FormExample {
  state value: String = ""

  fn handleSubmit() {
    print("Submitted: ${value}")
  }

  fn handleKeyPress(event: KeyEvent) {
    if event.key == "Enter" {
      handleSubmit()
    }
  }

  render {
    return TextInput(
      value: value,
      onChange: (newValue) => value = newValue,
      onKeyPress: handleKeyPress,
      onSubmit: handleSubmit,
      placeholder: "Type and press Enter"
    )
  }
}
```

### Mouse Events

```vela
component MouseExample {
  state isHovered: Bool = false
  state position: Point = { x: 0, y: 0 }

  fn handleMouseEnter() {
    isHovered = true
  }

  fn handleMouseLeave() {
    isHovered = false
  }

  fn handleMouseMove(event: MouseEvent) {
    position = { x: event.x, y: event.y }
  }

  render {
    return Container(
      child: Text("Position: (${position.x}, ${position.y})"),
      style: {
        backgroundColor: if isHovered { "lightblue" } else { "white" },
        padding: 20,
        border: "1px solid gray"
      },
      onMouseEnter: handleMouseEnter,
      onMouseLeave: handleMouseLeave,
      onMouseMove: handleMouseMove
    )
  }
}
```

### Custom Events

```vela
// Define custom event
enum TodoEvent {
  Added(text: String),
  Completed(id: Number),
  Deleted(id: Number)
}

// Event emitter
component TodoList {
  state todos: Array<Todo> = []
  state eventListeners: Array<(TodoEvent) -> void> = []

  fn addEventListener(listener: (TodoEvent) -> void) {
    eventListeners = eventListeners + listener
  }

  fn emit(event: TodoEvent) {
    eventListeners.forEach(listener => listener(event))
  }

  fn addTodo(text: String) {
    newTodo = { id: generateId(), text: text, completed: false }
    todos = todos + newTodo
    emit(Added(text))
  }

  fn completeTodo(id: Number) {
    todos = todos.map(todo => 
      if todo.id == id { 
        { ...todo, completed: true } 
      } else { 
        todo 
      }
    )
    emit(Completed(id))
  }

  render {
    return Column(
      children: [
        TextInput(
          placeholder: "Add todo",
          onSubmit: addTodo
        ),
        ...todos.map(todo => TodoItem(todo, completeTodo))
      ]
    )
  }
}
```

---

## Styling

### Inline Styles

```vela
Text("Styled text", style: {
  fontSize: 16,
  fontWeight: "bold",
  color: "blue",
  textAlign: "center"
})
```

### Style Objects

```vela
const buttonStyle = {
  backgroundColor: "blue",
  color: "white",
  padding: 10,
  borderRadius: 5,
  border: "none"
}

const hoverStyle = {
  ...buttonStyle,
  backgroundColor: "darkblue"
}

component StyledButton {
  state isHovered: Bool = false

  render {
    return Button("Click me",
      style: if isHovered { hoverStyle } else { buttonStyle },
      onMouseEnter: () => isHovered = true,
      onMouseLeave: () => isHovered = false
    )
  }
}
```

### Theme System

```vela
// Global theme
const theme = {
  colors: {
    primary: "blue",
    secondary: "gray",
    success: "green",
    error: "red"
  },
  spacing: {
    small: 8,
    medium: 16,
    large: 24
  },
  typography: {
    fontSize: 16,
    fontFamily: "Arial, sans-serif"
  }
}

// Themed component
component ThemedCard {
  state title: String
  state content: String

  render {
    return Container(
      child: Column(
        children: [
          Text(title, style: {
            fontSize: theme.typography.fontSize + 4,
            fontWeight: "bold",
            color: theme.colors.primary
          }),
          Text(content, style: {
            fontSize: theme.typography.fontSize,
            marginTop: theme.spacing.medium
          })
        ]
      ),
      style: {
        padding: theme.spacing.large,
        border: "1px solid ${theme.colors.secondary}",
        borderRadius: 8
      }
    )
  }
}
```

### Responsive Design

```vela
component ResponsiveLayout {
  computed screenWidth: Number {
    return window.innerWidth
  }

  computed isMobile: Bool {
    return screenWidth < 768
  }

  render {
    return if isMobile {
      // Mobile layout
      Column(
        children: [
          Header(),
          Content(),
          Footer()
        ]
      )
    } else {
      // Desktop layout
      Row(
        children: [
          Sidebar(),
          Flex(child: Content(), flex: 1)
        ]
      )
    }
  }
}
```

---

## Advanced Patterns

### Higher-Order Components

```vela
// HOC for loading state
fn withLoading<T>(component: Component<T>) -> Component<T & { loading: Bool }> {
  return (props) => {
    return if props.loading {
      LoadingSpinner()
    } else {
      component(props)
    }
  }
}

// Usage
const UserProfileWithLoading = withLoading(UserProfile)

component App {
  state user: Option<User> = None
  state loading: Bool = true

  effect {
    async fetchUser().then(u => {
      user = Some(u)
      loading = false
    })
  }

  render {
    return UserProfileWithLoading(
      user: user.unwrapOr(defaultUser),
      loading: loading
    )
  }
}
```

### Render Props

```vela
component MouseTracker {
  state position: Point = { x: 0, y: 0 }

  fn handleMouseMove(event: MouseEvent) {
    position = { x: event.x, y: event.y }
  }

  render {
    return Container(
      child: props.render(position),
      onMouseMove: handleMouseMove,
      style: { height: 200, border: "1px solid gray" }
    )
  }
}

// Usage
component App {
  render {
    return MouseTracker(
      render: (position) => 
        Text("Mouse at: (${position.x}, ${position.y})")
    )
  }
}
```

### Compound Components

```vela
component Tabs {
  state activeTab: Number = 0

  fn setActiveTab(index: Number) {
    activeTab = index
  }

  render {
    return Column(
      children: [
        Row(
          children: props.tabs.map((tab, index) => 
            TabHeader(
              title: tab.title,
              active: index == activeTab,
              onClick: () => setActiveTab(index)
            )
          )
        ),
        Container(
          child: props.tabs[activeTab].content,
          padding: 20
        )
      ]
    )
  }
}

component TabHeader {
  state title: String
  state active: Bool
  state onClick: () -> void

  render {
    return Button(title,
      onClick: onClick,
      style: {
        backgroundColor: if active { "blue" } else { "white" },
        color: if active { "white" } else { "black" },
        border: "none",
        padding: 10
      }
    )
  }
}

// Usage
component App {
  render {
    return Tabs(
      tabs: [
        { title: "Home", content: HomePage() },
        { title: "About", content: AboutPage() },
        { title: "Contact", content: ContactPage() }
      ]
    )
  }
}
```

### Custom Hooks

```vela
// Custom hook for local storage
fn useLocalStorage<T>(key: String, initialValue: T) -> (T, (T) -> void) {
  state value: T = loadFromStorage(key, initialValue)

  fn setValue(newValue: T) {
    value = newValue
    saveToStorage(key, newValue)
  }

  effect {
    // Sync with other tabs/windows
    window.addEventListener("storage", (event) => {
      if event.key == key {
        value = JSON.parse(event.newValue)
      }
    })
  }

  return (value, setValue)
}

// Usage
component App {
  (name, setName) = useLocalStorage("name", "Anonymous")

  render {
    return Column(
      children: [
        Text("Name: ${name}"),
        TextInput(
          value: name,
          onChange: setValue
        )
      ]
    )
  }
}
```

### Virtual Scrolling

```vela
component VirtualList {
  state items: Array<Item>
  state scrollTop: Number = 0
  state containerHeight: Number = 400
  itemHeight: Number = 50

  computed visibleRange: Range {
    start = Math.floor(scrollTop / itemHeight)
    end = Math.min(
      start + Math.ceil(containerHeight / itemHeight),
      items.length
    )
    return start..end
  }

  computed visibleItems: Array<Item> {
    return items.slice(visibleRange.start, visibleRange.end)
  }

  computed offsetY: Number {
    return visibleRange.start * itemHeight
  }

  fn handleScroll(event: ScrollEvent) {
    scrollTop = event.scrollTop
  }

  render {
    return Container(
      style: { 
        height: containerHeight, 
        overflow: "auto" 
      },
      onScroll: handleScroll,
      child: Container(
        style: { 
          height: items.length * itemHeight,
          position: "relative"
        },
        child: Container(
          style: { 
            transform: "translateY(${offsetY}px)" 
          },
          children: visibleItems.map(item => 
            Container(
              child: Text(item.text),
              style: { height: itemHeight }
            )
          )
        )
      )
    )
  }
}
```

---

## Performance

### Memoization

```vela
// Memoize expensive computations
memo expensiveList: Array<Item> {
  return items
    .filter(item => item.active)
    .map(item => transformItem(item))
    .sortBy(item => item.priority)
}

// Memoize components
memo UserCard {
  return Container(
    child: Column(
      children: [
        Text(user.name),
        Text(user.email)
      ]
    )
  )
}
```

### Lazy Loading

```vela
component LazyImage {
  state src: String
  state loaded: Bool = false
  state error: Bool = false

  effect {
    async loadImage(src).then(() => {
      loaded = true
    }).catch(() => {
      error = true
    })
  }

  render {
    return if error {
      Text("Failed to load image")
    } else if !loaded {
      LoadingSpinner()
    } else {
      Image(src: src)
    }
  }
}
```

### Debouncing

```vela
component SearchInput {
  state query: String = ""
  state debouncedQuery: String = ""
  debounceTimer: Option<Timer> = None

  effect {
    // Cancel previous timer
    match debounceTimer {
      Some(timer) => timer.cancel()
      None => {}
    }

    // Start new timer
    debounceTimer = Some(schedule(() => {
      debouncedQuery = query
      debounceTimer = None
    }, Duration.milliseconds(300)))
  }

  render {
    return TextInput(
      value: query,
      onChange: (value) => query = value,
      placeholder: "Search..."
    )
  }
}
```

---

## Best Practices

### 1. Keep Components Small

```vela
// ✅ Small, focused components
component TodoItem {
  state todo: Todo

  fn toggle() {
    todo.completed = !todo.completed
  }

  render {
    return Row(
      children: [
        Checkbox(checked: todo.completed, onChange: toggle),
        Text(todo.text)
      ]
    )
  }
}

// ❌ Large, complex components
component TodoApp {
  // ... lots of state and logic
  render {
    return Container(/* huge component tree */)
  }
}
```

### 2. Use Computed for Derived State

```vela
// ✅ Computed properties
component UserProfile {
  state user: User

  computed displayName: String {
    return "${user.firstName} ${user.lastName}"
  }

  computed isAdult: Bool {
    return user.age >= 18
  }

  render {
    return Column(
      children: [
        Text(displayName),
        Text(if isAdult { "Adult" } else { "Minor" })
      ]
    )
  }
}
```

### 3. Lift State Up When Needed

```vela
// ✅ Shared state in parent
component TodoList {
  state todos: Array<Todo> = []

  fn addTodo(text: String) {
    todos = todos + { text: text, completed: false }
  }

  fn toggleTodo(id: Number) {
    todos = todos.map(todo => 
      if todo.id == id { 
        { ...todo, completed: !todo.completed } 
      } else { 
        todo 
      }
    )
  }

  render {
    return Column(
      children: [
        AddTodoForm(onAdd: addTodo),
        ...todos.map(todo => TodoItem(todo, toggleTodo))
      ]
    )
  }
}
```

### 4. Use Keys for Lists

```vela
// ✅ Use keys for stable identity
component TodoList {
  state todos: Array<Todo> = []

  render {
    return Column(
      children: todos.map(todo => 
        TodoItem(key: todo.id, todo: todo)
      )
    )
  }
}
```

### 5. Handle Loading and Error States

```vela
component DataComponent {
  state data: Option<Data> = None
  state loading: Bool = false
  state error: Option<String> = None

  async fn loadData() {
    loading = true
    error = None

    try {
      result = await fetchData()
      data = Some(result)
    } catch (err) {
      error = Some(err.message)
    } finally {
      loading = false
    }
  }

  render {
    return match (loading, error, data) {
      (true, _, _) => LoadingSpinner()
      (_, Some(err), _) => ErrorMessage(err)
      (_, _, Some(d)) => DataView(d)
      _ => EmptyState()
    }
  }
}
```

### 6. Avoid Deep Nesting

```vela
// ✅ Extract components
component Form {
  render {
    return Column(
      children: [
        PersonalInfoSection(),
        AddressSection(),
        SubmitSection()
      ]
    )
  }
}

// ❌ Deep nesting
component Form {
  render {
    return Column(
      children: [
        Container(child: Column(children: [/* deeply nested */]))
      ]
    )
  }
}
```

### 7. Use Fragments for Multiple Roots

```vela
// ✅ Fragment for multiple elements
component MultiElement {
  render {
    return Fragment(
      children: [
        Header(),
        Content(),
        Footer()
      ]
    )
  }
}
```

### 8. Test Components

```vela
@test
fn testCounter() -> void {
  counter = render(Counter())

  // Initial state
  assert(counter.find("text").text == "Count: 0")

  // Click increment
  counter.find("button").click()
  assert(counter.find("text").text == "Count: 1")

  // Click decrement
  counter.find("button[text='-']").click()
  assert(counter.find("text").text == "Count: 0")
}

@test
fn testFormValidation() -> void {
  form = render(LoginForm())

  // Empty form
  submitButton = form.find("button")
  assert(submitButton.disabled == true)

  // Fill email only
  emailInput = form.find("input[placeholder='Email']")
  emailInput.setValue("user@example.com")
  assert(submitButton.disabled == true)

  // Fill password
  passwordInput = form.find("input[type='password']")
  passwordInput.setValue("password123")
  assert(submitButton.disabled == false)
}
```

---

## Common Patterns

### Todo App

```vela
component TodoApp {
  state todos: Array<Todo> = []
  state filter: Filter = All

  enum Filter { All, Active, Completed }

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
    if !text.trim().isEmpty() {
      newTodo = { 
        id: Date.now(), 
        text: text.trim(), 
        completed: false 
      }
      todos = todos + newTodo
    }
  }

  fn toggleTodo(id: Number) {
    todos = todos.map(todo => 
      if todo.id == id { 
        { ...todo, completed: !todo.completed } 
      } else { 
        todo 
      }
    )
  }

  fn deleteTodo(id: Number) {
    todos = todos.filter(todo => todo.id != id)
  }

  fn clearCompleted() {
    todos = todos.filter(todo => !todo.completed)
  }

  render {
    return Column(
      children: [
        Header("Todo App"),
        AddTodoForm(onAdd: addTodo),
        TodoList(
          todos: filteredTodos,
          onToggle: toggleTodo,
          onDelete: deleteTodo
        ),
        Footer(
          activeCount: activeCount,
          filter: filter,
          onFilterChange: (f) => filter = f,
          onClearCompleted: clearCompleted
        )
      ]
    )
  }
}
```

### Data Table

```vela
component DataTable<T> {
  state data: Array<T>
  state sortColumn: Option<String> = None
  state sortDirection: SortDirection = Ascending

  enum SortDirection { Ascending, Descending }

  computed sortedData: Array<T> {
    return match sortColumn {
      Some(column) => {
        sorted = data.sortBy(item => item[column])
        return if sortDirection == Descending { sorted.reverse() } else { sorted }
      }
      None => data
    }
  }

  fn handleSort(column: String) {
    if sortColumn == Some(column) {
      sortDirection = if sortDirection == Ascending { Descending } else { Ascending }
    } else {
      sortColumn = Some(column)
      sortDirection = Ascending
    }
  }

  render {
    return Table(
      headers: props.columns.map(column => 
        TableHeader(
          title: column.title,
          sortable: column.sortable,
          sorted: sortColumn == Some(column.key),
          direction: sortDirection,
          onClick: () => if column.sortable { handleSort(column.key) }
        )
      ),
      rows: sortedData.map(item => 
        TableRow(
          cells: props.columns.map(column => 
            TableCell(column.render(item))
          )
        )
      )
    )
  }
}

// Usage
component UserTable {
  state users: Array<User> = []

  render {
    return DataTable(
      data: users,
      columns: [
        { key: "name", title: "Name", sortable: true, render: (u) => u.name },
        { key: "email", title: "Email", sortable: true, render: (u) => u.email },
        { key: "role", title: "Role", sortable: false, render: (u) => u.role }
      ]
    )
  }
}
```

---

Vela's declarative UI system provides a powerful, efficient way to build reactive user interfaces. By embracing the principles of declarative programming and reactive state management, you can create maintainable, performant applications with less boilerplate code.