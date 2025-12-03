# Vela Compiler User Guide

## 🚀 Getting Started

Esta guía te ayudará a usar el compiler de Vela para compilar código fuente a bytecode ejecutable.

## 📦 Installation

### From Source

```bash
# Clona el repositorio
git clone https://github.com/velalang/vela.git
cd vela

# Compila el compiler
cargo build --release --bin vela-compiler

# El binario estará en target/release/vela-compiler
```

### Using Cargo

```bash
# Instala globalmente
cargo install --path crates/vela-compiler

# Verifica instalación
vela-compiler --version
```

## 💻 Basic Usage

### Compiling a Single File

```bash
# Compila archivo .vela a bytecode
vela-compiler compile source.vela -o output.bytecode

# Compila con optimizaciones
vela-compiler compile source.vela -O2 -o output.bytecode

# Compila con información de debug
vela-compiler compile source.vela --debug -o output.bytecode
```

### Compiling Multiple Files

```bash
# Compila múltiples archivos
vela-compiler compile file1.vela file2.vela -o program.bytecode

# Compila desde directorio
vela-compiler compile src/ -o program.bytecode
```

### Interactive Mode

```bash
# Inicia REPL interactivo
vela-compiler repl

# Dentro del REPL
> let x = 42
> print(x)
42
> exit
```

## ⚙️ Command Line Options

### Global Options

```bash
vela-compiler [COMMAND] [OPTIONS]

Commands:
  compile    Compila código fuente a bytecode
  repl       Inicia REPL interactivo
  check      Verifica sintaxis sin generar bytecode
  help       Muestra ayuda

Options:
  -h, --help          Muestra ayuda
  -V, --version       Muestra versión
  -v, --verbose       Salida detallada
  --quiet             Salida silenciosa
```

### Compile Command Options

```bash
vela-compiler compile [FILES...] [OPTIONS]

Options:
  -o, --output FILE       Archivo de salida (default: a.bytecode)
  -O, --optimize LEVEL    Nivel de optimización (0-3, default: 0)
  -g, --debug             Incluye información de debug
  -t, --target PLATFORM   Plataforma objetivo (native, wasm, cross)
  -W, --warnings          Trata warnings como errores
  --emit-ast              Emite AST en lugar de bytecode
  --emit-tokens           Emite tokens en lugar de bytecode
  --no-color              Desactiva colores en salida
```

## 📝 Writing Vela Code

### Basic Syntax

```vela
// Variables (inmutables por defecto)
let message = "Hello, Vela!"
let count = 42

// Funciones
fn greet(name: String) -> String {
    return "Hello, ${name}!"
}

// Llamadas a funciones
let greeting = greet("World")
print(greeting)  // "Hello, World!"
```

### Control Flow

```vela
// Condicionales
if age >= 18 {
    print("Adult")
} else {
    print("Minor")
}

// Pattern matching
match result {
    Ok(value) => print("Success: ${value}")
    Err(error) => print("Error: ${error}")
}

// Loops funcionales
(1..=10).forEach(i => print(i))

// Map y filter
let doubled = [1, 2, 3, 4].map(x => x * 2)
let evens = [1, 2, 3, 4].filter(x => x % 2 == 0)
```

### Error Handling

```vela
// Result types
fn divide(a: Number, b: Number) -> Result<Number, String> {
    if b == 0 {
        return Err("Division by zero")
    }
    return Ok(a / b)
}

// Using results
match divide(10, 2) {
    Ok(result) => print("Result: ${result}")
    Err(error) => print("Error: ${error}")
}
```

### Classes and Objects

```vela
class Person {
    constructor(name: String, age: Number) {
        this.name = name
        this.age = age
    }

    fn greet() -> String {
        return "Hello, I'm ${this.name}"
    }
}

let person = Person("Alice", 30)
print(person.greet())  // "Hello, I'm Alice"
```

## 🔧 Advanced Features

### Modules and Imports

```vela
// En math.vela
public fn add(a: Number, b: Number) -> Number {
    return a + b
}

// En main.vela
import "math"

let result = math.add(5, 3)
print(result)  // 8
```

### Generics

```vela
// Función genérica
fn identity<T>(value: T) -> T {
    return value
}

// Clase genérica
class Container<T> {
    constructor(value: T) {
        this.value = value
    }

    fn get() -> T {
        return this.value
    }
}

let container = Container<String>("Hello")
print(container.get())  // "Hello"
```

### Async/Await

```vela
async fn fetchData(url: String) -> Result<String, String> {
    // Simulación de fetch
    return Ok("Data from ${url}")
}

async fn main() {
    match await fetchData("https://api.example.com") {
        Ok(data) => print("Received: ${data}")
        Err(error) => print("Error: ${error}")
    }
}
```

## 🐛 Error Messages

### Common Compilation Errors

#### Syntax Errors

```bash
error: expected ')', found '}'
  --> source.vela:5:12
   |
 5 | fn test( {
   |           ^ expected ')'
```

**Fix:** Check parentheses matching.

#### Type Errors

```bash
error: type mismatch
  --> source.vela:8:9
   |
 8 | let x: Number = "hello"
   |         ^^^^^^ expected Number, found String
```

**Fix:** Ensure type consistency.

#### Undefined Variables

```bash
error: undefined variable 'undefined_var'
  --> source.vela:10:5
   |
10 | print(undefined_var)
   |      ^^^^^^^^^^^^^^ variable not defined in this scope
```

**Fix:** Define variables before use.

### Semantic Errors

#### Duplicate Definitions

```bash
error: duplicate definition of 'test'
  --> source.vela:3:4
   |
 3 | fn test() {}
   |    ^^^^ already defined at source.vela:1:4
```

**Fix:** Rename one of the definitions.

#### Unreachable Code

```bash
warning: unreachable code
  --> source.vela:6:5
   |
 5 |     return 42
 6 |     print("This never executes")
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ code after return is unreachable
```

**Fix:** Remove unreachable code.

## 🚀 Optimization

### Optimization Levels

```bash
# Sin optimizaciones (debug)
vela-compiler compile source.vela -O0

# Optimizaciones básicas
vela-compiler compile source.vela -O1

# Optimizaciones agresivas
vela-compiler compile source.vela -O2

# Optimizaciones máximas
vela-compiler compile source.vela -O3
```

### What Each Level Does

- **O0**: No optimizations, maximum debug info
- **O1**: Basic optimizations (constant folding, dead code elimination)
- **O2**: Advanced optimizations (inlining, loop optimizations)
- **O3**: Aggressive optimizations (may increase compile time)

## 🔍 Debugging

### Debug Information

```bash
# Compila con debug info
vela-compiler compile source.vela --debug -o program.bytecode

# Ejecuta con debugger
vela-vm debug program.bytecode
```

### Source Maps

```bash
# Genera source map
vela-compiler compile source.vela --source-map -o program.bytecode

# El archivo .map contendrá mapeo bytecode -> source
```

### Profiling

```bash
# Compila con profiling
vela-compiler compile source.vela --profile -o program.bytecode

# Ejecuta y genera perfil
vela-vm run program.bytecode --profile > profile.json
```

## 📁 Project Structure

### Recommended Layout

```
my-project/
├── src/
│   ├── main.vela
│   ├── utils.vela
│   └── math.vela
├── tests/
│   ├── main_test.vela
│   └── utils_test.vela
├── vela.toml
└── README.md
```

### Configuration File (vela.toml)

```toml
[project]
name = "my-project"
version = "1.0.0"
authors = ["Your Name <your.email@example.com>"]

[dependencies]
math-lib = { git = "https://github.com/example/math-lib" }

[build]
optimize = 2
debug = true
target = "native"
```

## 🧪 Testing

### Running Tests

```bash
# Ejecuta todos los tests
vela-compiler test

# Ejecuta tests específicos
vela-compiler test tests/math_test.vela

# Ejecuta con cobertura
vela-compiler test --coverage
```

### Writing Tests

```vela
// En math_test.vela
@test
fn test_addition() {
    assert(add(2, 3) == 5)
}

@test
fn test_subtraction() {
    assert(subtract(5, 3) == 2)
}
```

## 📊 Performance Tips

### 1. Use Appropriate Types

```vela
// ✅ Bueno: usa tipos específicos
let count: Number = 42
let name: String = "Alice"

// ❌ Malo: evita tipos genéricos innecesarios
let count: Any = 42  // Más lento en runtime
```

### 2. Prefer Immutable Data

```vela
// ✅ Bueno: inmutabilidad por defecto
let data = [1, 2, 3]
let doubled = data.map(x => x * 2)

// ❌ Malo: evita mutabilidad innecesaria
state data = [1, 2, 3]
data.push(4)  // Side effects
```

### 3. Use Functional Programming

```vela
// ✅ Bueno: programación funcional
let sum = [1, 2, 3, 4, 5]
    .filter(x => x % 2 == 0)
    .map(x => x * 2)
    .reduce((acc, x) => acc + x, 0)

// ❌ Malo: imperativo
let sum = 0
for i in 0..list.length() {
    if list[i] % 2 == 0 {
        sum = sum + (list[i] * 2)
    }
}
```

## 🔗 Integration with Tools

### Build Systems

#### Make

```makefile
.PHONY: build clean test

build:
    vela-compiler compile src/ -o bin/program.bytecode

test:
    vela-compiler test tests/

clean:
    rm -f bin/program.bytecode
```

#### Cargo (for mixed Rust/Vela projects)

```toml
[package]
name = "my-project"
version = "1.0.0"

[build-dependencies]
vela-compiler = "1.0"

[package.metadata.vela]
source-dir = "src"
output-dir = "target/vela"
```

### IDE Integration

#### VS Code Extension

```json
{
    "vela.compiler.path": "/usr/local/bin/vela-compiler",
    "vela.compiler.optimize": 2,
    "vela.compiler.warningsAsErrors": true
}
```

#### Vim/Neovim

```vim
" Configuración básica
let g:vela_compiler_path = '/usr/local/bin/vela-compiler'
let g:vela_compiler_flags = ['-O2', '--debug']

" Compilar archivo actual
nmap <F5> :!vela-compiler compile %<CR>
```

## ❓ Troubleshooting

### Common Issues

#### "Command not found"

```bash
# Asegúrate de que el binario esté en PATH
export PATH=$PATH:/path/to/vela-compiler

# O instala globalmente
cargo install --path crates/vela-compiler
```

#### "Permission denied"

```bash
# Da permisos de ejecución
chmod +x target/release/vela-compiler
```

#### "Out of memory"

```bash
# Reduce nivel de optimización
vela-compiler compile source.vela -O0 -o output.bytecode

# O aumenta límite de memoria
ulimit -v unlimited
```

### Getting Help

```bash
# Muestra ayuda general
vela-compiler --help

# Muestra ayuda de comando específico
vela-compiler compile --help

# Reporta bugs en GitHub
# https://github.com/velalang/vela/issues
```

---

*Documentación generada automáticamente. Última actualización: 2025-12-03*