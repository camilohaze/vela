# Ejemplos de Uso - Sistema de Tipos

## Inferencia Básica

### Variables y Funciones

```rust
// Inferencia automática
let x = 42;        // Type::Int
let y = "hello";   // Type::String
let z = true;      // Type::Bool

// Función con inferencia
fn add(a, b) {     // (Int, Int) -> Int
    a + b
}

// Inferencia polimórfica
fn identity(x) {   // ∀T. T -> T
    x
}
```

### Tipos Explícitos

```rust
// Anotaciones explícitas
let x: Int = 42;
let name: String = "Vela";

// Función con tipos explícitos
fn greet(name: String) -> String {
    "Hello, ${name}"
}

// Tipos complejos
let numbers: [Int] = [1, 2, 3, 4];
let point: (Float, Float) = (3.14, 2.71);
```

## Pattern Matching

### Tuplas y Registros

```rust
// Tupla
let point = (10, 20);
match point {
    (x, y) => println("x: ${x}, y: ${y}")
}

// Registro
let user = {name: "Alice", age: 30};
match user {
    {name, age} => println("${name} is ${age} years old")
}
```

### Variantes

```rust
enum Result<T, E> {
    Ok(T),
    Err(E)
}

let result = Ok(42);
match result {
    Ok(value) => println("Success: ${value}")
    Err(error) => println("Error: ${error}")
}
```

## Funciones de Orden Superior

### Map y Filter

```rust
// Map sobre lista
let numbers = [1, 2, 3, 4, 5];
let doubled = numbers.map(x => x * 2);  // [2, 4, 6, 8, 10]

// Filter con predicado
let evens = numbers.filter(x => x % 2 == 0);  // [2, 4]

// Reduce
let sum = numbers.reduce((acc, x) => acc + x, 0);  // 15
```

### Closures

```rust
// Closure capturando variables
let multiplier = 2;
let multiply = x => x * multiplier;

let result = multiply(5);  // 10
```

## Tipos Genéricos

### Contenedores Genéricos

```rust
// Option<T>
let maybe_value: Option<Int> = Some(42);
match maybe_value {
    Some(v) => println("Value: ${v}")
    None => println("No value")
}

// Result<T, E>
fn divide(a: Int, b: Int) -> Result<Float, String> {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a as Float / b as Float)
    }
}
```

### Tipos Personalizados

```rust
// Struct genérico
struct Container<T> {
    value: T
}

// Función genérica
fn wrap<T>(value: T) -> Container<T> {
    Container { value }
}

let int_container = wrap(42);      // Container<Int>
let str_container = wrap("hello"); // Container<String>
```

## Sistema de Traits (Futuro)

```rust
// Definición de trait
trait Display {
    fn to_string() -> String;
}

// Implementación
impl Display for Int {
    fn to_string() -> String {
        self.toString()
    }
}

// Uso con constraints
fn print<T: Display>(value: T) {
    println(value.to_string());
}
```

## Manejo de Errores

### Result y Option

```rust
// Propagación de errores
fn process_data(data: String) -> Result<ProcessedData, Error> {
    let parsed = parse_json(data)?;
    let validated = validate(parsed)?;
    Ok(transform(validated))
}

// Uso con match
match process_data(raw_data) {
    Ok(result) => println("Success: ${result}")
    Err(error) => println("Error: ${error}")
}

// Option chaining
let user_name = find_user(id)?.name;
```

## Contextos y Scopes

### Variables Locales

```rust
fn example() {
    let x = 10;  // x: Int
    
    {
        let x = "hello";  // Shadowing: x: String
        println(x);       // "hello"
    }
    
    println(x);  // 10 (el original)
}
```

### Closures y Captura

```rust
fn create_counter() -> () -> Int {
    let count = 0;  // Capturado por referencia mutable
    
    () => {
        count = count + 1;
        count
    }
}

let counter = create_counter();
counter();  // 1
counter();  // 2
```

## Integración con Runtime

### Verificación en Runtime

```rust
// Type checking en desarrollo
#[cfg(debug_assertions)]
fn check_types(ast: &AST) {
    let mut checker = TypeChecker::new();
    
    for node in ast.nodes() {
        if let Err(error) = checker.check_node(node) {
            log_error(error);
        }
    }
}

// Optimización en release
#[cfg(not(debug_assertions))]
fn check_types(_ast: &AST) {
    // Type checking omitido en producción
}
```

## Benchmarks y Performance

### Comparación con Python

```rust
// Vela con type checking
let numbers: [Int] = [1, 2, 3, 4, 5];
let sum = numbers.reduce((acc, x) => acc + x, 0);
// Tiempo: ~50ns (con inferencia completa)

// Python equivalente
numbers = [1, 2, 3, 4, 5]
sum_result = sum(numbers)
# Tiempo: ~200ns (sin type checking)
```

### Optimizaciones del Compilador

```rust
// Monomorfización automática
fn generic_add<T: Add>(a: T, b: T) -> T {
    a + b
}

// Se convierte en:
fn add_int(a: i64, b: i64) -> i64 { a + b }
fn add_float(a: f64, b: f64) -> f64 { a + b }
```

## Casos de Uso Avanzados

### DSL para APIs

```rust
// Definición de API con tipos
let api = {
    users: {
        get: (id: Int) -> Result<User, Error>,
        list: (page: Int, limit: Int) -> Result<[User], Error>,
        create: (user: UserInput) -> Result<User, Error>,
    }
};

// Uso type-safe
let user = api.users.get(123)?;
let users = api.users.list(1, 10)?;
```

### State Management Reactivo

```rust
// Sistema reactivo con tipos
let state = reactive({
    count: 0,
    name: "Vela"
});

let doubled = computed(() => state.count * 2);
let greeting = computed(() => "Hello, ${state.name}");

effect(() => {
    println("${greeting()}! Count: ${doubled()}");
});

// Cambios automáticos
state.count = 5;  // Trigger: "Hello, Vela! Count: 10"
state.name = "Rust";  // Trigger: "Hello, Rust! Count: 10"
```