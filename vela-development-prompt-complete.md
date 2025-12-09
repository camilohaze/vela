# PROMPT COMPLETO PARA DESARROLLAR VELA

## 📋 INFORMACIÓN GENERAL

**Fecha:** Diciembre 9, 2025  
**Versión del Prompt:** 3.0  
**Estado del Proyecto:** Desarrollo activo  
**Repositorio:** https://github.com/camilohaze/vela  

---

## 🎯 VISIÓN Y OBJETIVOS DE VELA

Vela es un **lenguaje de programación funcional puro** con las siguientes características principales:

### Paradigma
- **Programación Funcional Pura**: Inmutabilidad por defecto, funciones puras, composición
- **Reactividad Integrada**: Sistema de signals, computed values, effects
- **UI Declarativa**: Widgets basados en composición, similar a Flutter/React
- **Multiplataforma**: Web, Mobile, Desktop, Backend

### Inspiraciones
| Aspecto | Inspirado en |
|---------|--------------|
| **Sintaxis Moderna** | TypeScript, Swift, Kotlin |
| **Sistema de Tipos** | TypeScript, Rust |
| **Funcional** | Haskell, Elm, F# |
| **UI Declarativa** | Flutter, React, SwiftUI |
| **DI/Modular** | NestJS, Angular, Spring Boot |
| **Concurrente** | Go, Erlang, Akka |

### Objetivos Principales
1. **Productividad**: Sintaxis concisa, inferencia de tipos, tooling excelente
2. **Seguridad**: Inmutabilidad, Option<T> en lugar de null, pattern matching exhaustivo
3. **Performance**: Compilación a bytecode optimizado, GC avanzado
4. **Ecosistema**: Paquetes, tooling, comunidad

---

## 🏗️ ARQUITECTURA DEL PROYECTO

### Estructura de Carpetas
```
vela/
├── core/                         # Tipos base, AST, IR (Rust)
├── compiler/                     # Lexer, parser, semantic analyzer, codegen (Rust)
├── vm/                           # Máquina virtual (Rust + Vela)
├── runtime/                      # Runtime system: reactive, concurrency, GC (Rust)
├── stdlib/                       # Librería estándar (Rust + Vela)
├── tooling/                      # CLI, LSP, debugger, devtools (Rust)
├── packages/                     # Paquetes adicionales: i18n, logging, validation (Rust)
│
├── docs/
│   ├── architecture/              # ADRs (Architectural Decision Records)
│   ├── features/                  # Documentación por Historia de Usuario
│   │   └── VELA-XXX/
│   │       ├── README.md
│   │       ├── TASK-001.md
│   │       └── ...
│   ├── api/                       # Especificaciones de API
│   └── design/                    # Diseños de UI/UX
│
├── examples/                      # TODOS los ejemplos van aquí
│   ├── ui/                        # Ejemplos de UI en Vela
│   ├── hello-world/               # Ejemplo básico
│   └── ...
│
├── tests/                         # Tests del proyecto
│   ├── unit/                      # Tests unitarios
│   └── integration/               # Tests de integración
│
├── README.md
├── CHANGELOG.md
├── Cargo.toml                     # Workspace configuration
└── .gitignore
```

### Componentes Principales

#### 1. Core (Rust)
- **AST**: Árbol de sintaxis abstracta
- **IR**: Intermediate Representation
- **Tipos**: Sistema de tipos con inferencia

#### 2. Compiler (Rust)
- **Lexer**: Tokenización
- **Parser**: Análisis sintáctico
- **Semantic Analyzer**: Análisis semántico, type checking
- **Codegen**: Generación de bytecode

#### 3. VM (Rust)
- **VelaVM**: Máquina virtual stack-based
- **Bytecode**: Formato de instrucciones (256 opcodes)
- **GC**: Garbage collector con ARC + cycle detection
- **Loader**: Carga de módulos bytecode

#### 4. Runtime (Rust)
- **Reactive Engine**: Signals, computed, effects
- **Concurrency**: Actores, async/await
- **UI Framework**: Widgets, rendering, layout

#### 5. Stdlib (Rust + Vela)
- **Colecciones**: List, Dict, Set
- **I/O**: File, Network, HTTP
- **Utils**: JSON, Regex, Date/Time

#### 6. Tooling (Rust)
- **CLI**: Build, run, test, format
- **LSP**: Autocompletado, diagnostics
- **Debugger**: Debugging interactivo

---

## 📚 ESPECIFICACIÓN COMPLETA DEL LENGUAJE VELA

### 1. PALABRAS RESERVADAS

#### Variables y Mutabilidad
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| *(sin keyword)* | Inmutable por defecto | `name: String = "Vela"` |
| `signal` | Mutable y reactiva | `signal count: Number = 0` |

**❌ PROHIBIDO:** `let`, `const`, `var`, `mut`

#### Tipos Primitivos
| Keyword | Descripción | Ejemplo |
|---------|-------------|---------|
| `Number` | Entero 64-bit | `age: Number = 37` |
| `Float` | Punto flotante 64-bit | `price: Float = 19.99` |
| `String` | Cadena de texto | `name: String = "Vela"` |
| `Bool` | Booleano | `isActive: Bool = true` |
| `void` | Sin retorno | `fn log() -> void` |
| `never` | Nunca retorna | `fn panic() -> never` |

#### Control de Flujo (Funcional)
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `if` | Condicional (expression) | `if age >= 18 { "adult" } else { "minor" }` |
| `else` | Rama alternativa | `if x > 0 { ... } else { ... }` |
| `match` | Pattern matching exhaustivo | `match result { Ok(val) => ..., Err(e) => ... }` |

**❌ PROHIBIDO:** `for`, `while`, `loop`, `switch`, `case`, `break`, `continue`

#### Funciones
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `fn` | Define función | `fn add(a: Number, b: Number) -> Number` |
| `async` | Función asíncrona | `async fn fetchData() -> Result<String>` |
| `await` | Espera async | `data = await fetchData()` |
| `return` | Retorna valor | `return result` |
| `yield` | Generador | `yield nextValue` |

#### OOP (Orientado a Objetos)
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `class` | Define clase | `class Person { ... }` |
| `abstract` | Clase abstracta | `abstract class Shape { ... }` |
| `interface` | Contrato | `interface Drawable { fn draw() -> void }` |
| `extends` | Herencia | `class Dog extends Animal` |
| `implements` | Implementa interfaz | `class Button implements Clickable` |
| `override` | Sobrescribe | `override fn toString() -> String` |
| `this` | Instancia actual | `this.name` |
| `super` | Clase padre | `super.greet()` |
| `constructor` | Constructor | `constructor(name: String) { this.name = name }` |

#### Estructuras de Datos
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `type` | Alias de tipo | `type UserId = Number` |
| `enum` | Enumeración | `enum Color { Red, Green, Blue }` |
| `struct` | Estructura | `struct User { id: Number, name: String }` |

#### Manejo de Errores
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `try` | Bloque try-catch | `try { riskyOp() } catch (e) { handle(e) }` |
| `catch` | Captura excepción | `catch (e: MyError) { ... }` |
| `throw` | Lanza excepción | `throw Error("failed")` |
| `finally` | Siempre ejecuta | `finally { cleanup() }` |

**Preferido:** `Result<T, E>` sobre excepciones.

#### Imports y Módulos
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `import` | Importar módulo | `import 'package:http'` |
| `show` | Importar específicos | `import 'library:utils' show { sort, filter }` |
| `hide` | Importar excepto | `import 'library:math' hide { deprecated_fn }` |
| `as` | Alias | `import 'package:long_name' as ln` |

**Sistema de Prefijos:**
- `system:*` - APIs internas de Vela
- `package:*` - Dependencias externas
- `module:*` - Módulos del proyecto
- `library:*` - Librerías internas
- `extension:*` - Extensiones
- `assets:*` - Assets

#### Modificadores de Acceso
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `public` | Accesible públicamente | `public class MyClass` |
| `private` | Solo clase/módulo | `private fn helper()` |
| `protected` | Clase y subclases | `protected fn method()` |

#### Reactividad
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `computed` | Valor derivado | `computed doubled: Number { return this.count * 2 }` |
| `memo` | Computed con caché | `memo expensive: Number { /* cálculo */ }` |
| `effect` | Side effect | `effect { print("Count: ${this.count}") }` |
| `watch` | Observar cambios | `watch(this.name) { print("Changed") }` |

#### Arquitectura / DDD
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `service` | Lógica de negocio | `service UserService { ... }` |
| `repository` | Acceso a datos | `repository UserRepository { ... }` |
| `controller` | Controlador HTTP | `controller UserController { ... }` |
| `usecase` | Caso de uso | `usecase CreateUser { ... }` |
| `entity` | Entidad de dominio | `entity User { ... }` |
| `dto` | Data Transfer Object | `dto CreateUserDTO { ... }` |
| `valueObject` | Value Object | `valueObject Email { ... }` |
| `model` | Modelo genérico | `model Product { ... }` |

#### UI / Widgets
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `StatefulWidget` | Widget con estado | `class Counter extends StatefulWidget` |
| `StatelessWidget` | Widget puro | `class Label extends StatelessWidget` |
| `component` | Alias de StatefulWidget | `component MyButton { ... }` |
| `widget` | Widget genérico | `widget CustomBox { ... }` |

#### Ciclo de Vida de Componentes
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `mount` | Al montar | `mount() { this.fetchData() }` |
| `update` | Después de update | `update() { print("Updated") }` |
| `destroy` | Al desmontar | `destroy() { this.cleanup() }` |

#### Patrones de Diseño
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `factory` | Factory pattern | `factory UserFactory { ... }` |
| `builder` | Builder pattern | `builder QueryBuilder { ... }` |
| `strategy` | Strategy pattern | `strategy PaymentStrategy { ... }` |
| `observer` | Observer pattern | `observer EventObserver { ... }` |
| `singleton` | Singleton pattern | `singleton Database { ... }` |
| `adapter` | Adapter pattern | `adapter LegacyAdapter { ... }` |
| `decorator` | Decorator pattern | `decorator LogDecorator { ... }` |

#### Web / API
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `guard` | Route guard | `guard AuthGuard { ... }` |
| `middleware` | HTTP middleware | `middleware Logger { ... }` |
| `interceptor` | Request/response | `interceptor AuthInterceptor { ... }` |
| `validator` | Validador | `validator EmailValidator { ... }` |
| `pipe` | Pipeline | `pipe TransformPipe { ... }` |

#### Sistema de Módulos
| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `module` | Módulo funcional | `module AuthModule { declarations: [...], exports: [...] }` |

### 2. DECORADORES / ANNOTATIONS

#### Sistema de Módulos
- `@module({ declarations, exports, providers, imports })`
- `@package` - Paquete publicable
- `@library` - Librería interna
- `@extension` - Extensión del lenguaje

#### Dependency Injection
- `@injectable` - Marca clase como inyectable
- `@inject` - Inyecta dependencia
- `@provides` - Proveedor de dependencia

#### HTTP
- `@get(path)` - Endpoint GET
- `@post(path)` - Endpoint POST
- `@put(path)` - Endpoint PUT
- `@patch(path)` - Endpoint PATCH
- `@delete(path)` - Endpoint DELETE

#### Validación
- `@validate` - Validar input
- `@required` - Campo requerido
- `@min(n)` - Valor mínimo
- `@max(n)` - Valor máximo
- `@email` - Validar email
- `@url` - Validar URL

#### UI
- `@signal` - Estado reactivo
- `@computed` - Valor computado
- `@effect` - Efecto secundario
- `@watch` - Observador

### 3. SINTAXIS ESPECÍFICA

#### Interpolación de Strings
```vela
// ✅ CORRECTO
message: String = "Hello, ${name}!"

// ❌ PROHIBIDO
message = `Hello, ${name}`  // No backticks
message = "Hello, " + name  // No concatenación
```

#### Rangos
```vela
// Rango exclusivo: 0..10 → [0, 1, 2, ..., 9]
(0..10).forEach(i => print(i))

// Rango inclusivo: 0..=10 → [0, 1, 2, ..., 10]
(0..=10).forEach(i => print(i))
```

#### Pattern Matching Exhaustivo
```vela
match value {
  1 => "one"
  2 => "two"
  _ => "other"  // Obligatorio para exhaustividad
}

match result {
  Ok(value) => print("Success: ${value}")
  Err(error) => print("Error: ${error}")
}
```

#### Inmutabilidad por Defecto
```vela
// ✅ Inmutable sin keyword
name: String = "Vela"

// ❌ ERROR: intentar mutar
// name = "Otro"

// ✅ Mutable con signal
signal counter: Number = 0
counter = counter + 1  // OK
```

#### Arrow Functions
```vela
// Función anónima
callback = (x: Number) => x * 2

// Con bloque
process = (data: String) => {
  cleaned = data.trim()
  return cleaned.toUpperCase()
}
```

#### Opcionalidad con Option<T>
```vela
// ✅ CORRECTO: usar Option<T>
fn findUser(id: Number) -> Option<User> {
  user = database.query(id)
  if user.exists() {
    return Some(user)
  }
  return None
}

// ❌ PROHIBIDO: null, undefined, nil
// user: User? = null
```

#### Métodos Funcionales (OBLIGATORIOS)
```vela
// ✅ CORRECTO: métodos funcionales
(0..10).forEach(i => print(i))
list.map(x => x * 2)
list.filter(x => x > 5)
list.reduce((acc, x) => acc + x, 0)

// ❌ PROHIBIDO: loops imperativos
// for i in 0..10 { print(i) }
// while condition { doSomething() }
```

### 4. SISTEMA DE TIPOS

#### Tipos Básicos
- `Number`: i64
- `Float`: f64
- `String`: UTF-8 string
- `Bool`: true/false
- `void`: Sin retorno
- `never`: Nunca retorna

#### Tipos Compuestos
- `List<T>`: Lista dinámica
- `Dict<K, V>`: Diccionario hash
- `Set<T>`: Conjunto hash
- `Tuple<T1, T2, ...>`: Tupla inmutable
- `Option<T>`: Some(value) | None
- `Result<T, E>`: Ok(value) | Err(error)

#### Generics
```vela
class Stack<T> {
  private items: List<T> = []
  
  fn push(item: T) -> void {
    this.items.add(item)
  }
  
  fn pop() -> Option<T> {
    return this.items.pop()
  }
}
```

#### Type Inference
```vela
// Tipo inferido automáticamente
message = "Hello"  // String
count = 42         // Number
list = [1, 2, 3]   // List<Number>
```

### 5. CONCURRENCIA Y ASYNC

#### Actores
```vela
actor Counter {
  signal count: Number = 0
  
  fn increment() -> void {
    this.count = this.count + 1
  }
  
  fn getCount() -> Number {
    return this.count
  }
}

// Uso
counter = Counter()
counter.increment()
count = counter.getCount()
```

#### Async/Await
```vela
async fn fetchUser(id: Number) -> Result<User> {
  response = await http.get("/users/${id}")
  return response.json()
}

// Uso
user = await fetchUser(123)
```

### 6. REACTIVIDAD

#### Signals
```vela
class Counter {
  signal count: Number = 0
  
  computed doubled: Number {
    return this.count * 2
  }
  
  effect {
    print("Count: ${this.count}, Doubled: ${this.doubled}")
  }
  
  fn increment() -> void {
    this.count = this.count + 1  // Trigger reactivity
  }
}
```

#### Componentes UI
```vela
component CounterWidget {
  state count: Number = 0
  
  computed even: Bool {
    return this.count % 2 == 0
  }
  
  fn render() -> Widget {
    return Column {
      Text("Count: ${this.count}")
      Text("Even: ${this.even}")
      Button(
        text: "Increment",
        onPressed: () => this.count = this.count + 1
      )
    }
  }
}
```

---

## 📊 ESTADO ACTUAL DE IMPLEMENTACIÓN

### ✅ IMPLEMENTADO

#### EPIC-01: Core Language (100% completado)
- ✅ AST y tipos base
- ✅ Sistema de tipos con inferencia
- ✅ IR (Intermediate Representation)

#### EPIC-02: Compiler Frontend (90% completado)
- ✅ Lexer funcional
- ✅ Parser recursivo descendente
- ✅ Semantic analyzer básico
- ✅ Type checker con inferencia
- 🔄 Codegen a bytecode (en desarrollo)

#### EPIC-03: Reactive Engine (85% completado)
- ✅ Signals y computed values
- ✅ Effects y watchers
- ✅ Dependency tracking
- 🔄 Integration con UI (parcial)

#### EPIC-04: UI Framework (80% completado)
- ✅ Widget base classes
- ✅ Layout widgets (Container, Row, Column, Stack)
- ✅ Input widgets (Button, TextField, Checkbox)
- ✅ Display widgets (Text, Image, Icon)
- ✅ Theming system
- 🔄 Navigation y routing (parcial)

#### EPIC-05: State Management (100% completado)
- ✅ Store pattern Redux-style
- ✅ Actions y reducers
- ✅ Dispatch system
- ✅ @connect y @select decorators
- ✅ Middleware system
- ✅ DevTools integration

#### EPIC-06: Compiler Backend / VelaVM (70% completado)
- ✅ VelaVM stack-based (256 opcodes)
- ✅ Bytecode format con metadata
- ✅ Loader de módulos
- ✅ GC con ARC básico
- ✅ Cycle detection básico
- 🔄 Full mark-and-sweep cycle detection
- 🔄 Integration completa con VM

#### EPIC-07: Standard Library (30% completado)
- ✅ List<T> básico
- 🔄 Dict<K,V> (en desarrollo)
- 🔄 Set<T> (pendiente)
- 🔄 I/O APIs (pendiente)
- 🔄 HTTP client (pendiente)
- 🔄 JSON serialization (pendiente)

#### EPIC-08: Tooling (20% completado)
- 🔄 CLI básico (build/run/test)
- 🔄 LSP server (autocompletado básico)
- 🔄 Formatter (pendiente)
- 🔄 Debugger (pendiente)

### ❌ NO IMPLEMENTADO / PENDIENTE

#### EPIC-07: Standard Library (Continuación)
- ❌ Dict<K,V> implementation
- ❌ Set<T> implementation
- ❌ Queue y Stack
- ❌ File I/O API
- ❌ Directory operations
- ❌ HTTP client
- ❌ WebSocket support
- ❌ JSON parser/encoder
- ❌ Regex support
- ❌ Date/Time utilities

#### EPIC-08: Tooling (Continuación)
- ❌ vela build command
- ❌ vela run command
- ❌ vela test command
- ❌ vela fmt command
- ❌ vela doctor command
- ❌ LSP completion
- ❌ LSP hover
- ❌ LSP definition
- ❌ LSP diagnostics
- ❌ LSP rename
- ❌ LSP publishDiagnostics

#### EPIC-09: Package Manager
- ❌ vela install
- ❌ vela publish
- ❌ Dependency resolution
- ❌ Package registry

#### EPIC-10: Advanced Features
- ❌ Macros system
- ❌ Operator overloading
- ❌ Traits (similar a interfaces)
- ❌ Advanced generics
- ❌ Reflection
- ❌ FFI (Foreign Function Interface)

#### EPIC-11: Performance & Optimization
- ❌ JIT compilation
- ❌ Advanced GC (generational)
- ❌ Parallel compilation
- ❌ Incremental compilation

#### EPIC-12: Ecosystem
- ❌ Package ecosystem
- ❌ Documentation generator
- ❌ Benchmarking tools
- ❌ Profiling tools

---

## 📋 BACKLOG COMPLETO DE TAREAS

### EPIC-06: Compiler Backend (Continuación)
- **TASK-077**: Integrar ARC con sistema reactivo - GC de signals y computed
- **TASK-078**: Tests de memory management - Tests de leaks y correctness
- **TASK-079**: Implementar module resolution - Resolución de imports y paths
- **TASK-080**: Implementar bytecode loader - Carga de bytecode compilado
- **TASK-081**: Tests de module system - Tests de imports y carga

### EPIC-07: Standard Library
- **TASK-083**: Implementar Set<T> - Set con hash table
- **TASK-084**: Implementar Dict<K,V> - Dictionary con hash table
- **TASK-085**: Implementar Queue y Stack - Estructuras adicionales
- **TASK-086**: Tests de colecciones - Tests exhaustivos
- **TASK-087**: Implementar File API - Lectura/escritura de archivos
- **TASK-088**: Implementar Directory API - Operaciones con directorios
- **TASK-089**: Implementar HttpClient - Cliente HTTP básico
- **TASK-090**: Implementar WebSocket - Soporte para WebSockets
- **TASK-091**: Tests de I/O y networking - Tests de correctness
- **TASK-092**: Implementar JSON parser - Parser de JSON
- **TASK-093**: Implementar JSON encoder - Serialización a JSON
- **TASK-094**: Implementar JSON decorators - Serialización automática
- **TASK-095**: Tests de JSON - Tests de parsing y encoding

### EPIC-08: Tooling (CLI)
- **TASK-097**: Implementar vela build - Compilación de proyectos
- **TASK-098**: Implementar vela run - Ejecución de proyectos
- **TASK-099**: Implementar vela test - Runner de tests
- **TASK-100**: Implementar vela fmt - Formatter de código
- **TASK-101**: Implementar vela doctor - Diagnóstico de instalación

### EPIC-08: Tooling (LSP)
- **TASK-108**: Implementar textDocument/completion - Autocompletado
- **TASK-109**: Implementar textDocument/hover - Tooltips
- **TASK-110**: Implementar textDocument/definition - Go to definition
- **TASK-111**: Implementar textDocument/publishDiagnostics - Errores en tiempo real
- **TASK-112**: Implementar textDocument/rename - Refactoring
- **TASK-113**: Tests de LSP - Tests de protocolo

### EPIC-09: Package Manager
- **TASK-103**: Implementar vela install - Instalación de dependencias
- **TASK-104**: Implementar dependency resolution - Resolución de versiones
- **TASK-105**: Implementar vela publish - Publicación de paquetes
- **TASK-106**: Tests de package manager - Tests de instalación

### EPIC-10: Advanced Language Features
- **TASK-114**: Implementar macros system - Macros para metaprogramming
- **TASK-115**: Implementar operator overloading - Sobrecarga de operadores
- **TASK-116**: Implementar traits - Sistema de traits
- **TASK-117**: Advanced generics - Generics avanzados
- **TASK-118**: Reflection API - Introspección de tipos
- **TASK-119**: FFI support - Foreign Function Interface

### EPIC-11: Performance
- **TASK-120**: JIT compilation - Just-In-Time compiler
- **TASK-121**: Generational GC - GC generacional avanzado
- **TASK-122**: Parallel compilation - Compilación paralela
- **TASK-123**: Incremental compilation - Compilación incremental

### EPIC-12: Ecosystem
- **TASK-124**: Package ecosystem - Sistema de paquetes
- **TASK-125**: Documentation generator - Generador de docs
- **TASK-126**: Benchmarking tools - Herramientas de benchmarking
- **TASK-127**: Profiling tools - Herramientas de profiling

---

## 🚀 PROCESO DE DESARROLLO

### 1. INICIAR SPRINT
```bash
# 1. Leer la Historia en Jira
# 2. Obtener lista de Subtasks
# 3. Crear rama: git checkout -b feature/VELA-XXX-descripcion
# 4. Crear carpeta: docs/features/VELA-XXX/
```

### 2. DESARROLLAR SUBTASKS
Por cada Subtask en orden:
```bash
# 1. Mover a "En curso" en Jira
# 2. Identificar tipo de Subtask:
#    - Decisión arquitectónica → Generar ADR
#    - Feature nueva → Generar código + tests
#    - Refactoring → Código + tests regresión
#    - Documentación → Docs

# 3. Generar entregables:
#    - docs/architecture/ADR-XXX.md (si decisión)
#    - src/feature.py (código)
#    - tests/unit/test_feature.py (tests)
#    - docs/features/VELA-XXX/TASK-XXX.md (documentación)

# 4. Commit atómico:
git add .
git commit -m "feat(VELA-XXX): implementar TASK-XXX

- Código en src/
- Tests en tests/unit/
- Documentación en docs/
- ADR en docs/architecture/ (si aplica)

Refs: VELA-XXX"

# 5. Mover a "Finalizada" en Jira
```

### 3. COMPLETAR HISTORIA
```bash
# 1. Generar README.md de la Historia
# 2. Crear Pull Request
# 3. Mover Historia a "En revisión"
# 4. Esperar aprobación del usuario
# 5. Merge a main
# 6. Mover Historia a "Finalizada"
```

### 4. CERRAR SPRINT
```bash
# 1. Generar Release Notes: docs/releases/sprint-N.md
# 2. Actualizar CHANGELOG.md
# 3. Crear tag: git tag sprint-N
# 4. Cerrar Sprint en Jira
```

---

## 📝 TEMPLATE DE ARCHIVOS

### ADR Template
```markdown
# ADR-XXX: [Título de la Decisión]

## Estado
✅ Aceptado | 🔄 Propuesto | ❌ Rechazado | ⏸️ Obsoleto

## Fecha
YYYY-MM-DD

## Contexto
[Problema que resolvemos]

## Decisión
[Solución elegida]

## Consecuencias
### Positivas
- [Beneficio 1]
- [Beneficio 2]

### Negativas
- [Trade-off 1]
- [Trade-off 2]

## Alternativas Consideradas
1. **Alternativa 1**: [Descripción] - Rechazada porque [razón]
2. **Alternativa 2**: [Descripción] - Rechazada porque [razón]

## Referencias
- Jira: [VELA-XXX]
- Documentación: [link]

## Implementación
Ver código en: `src/nombre-archivo.py`
```

### Código Fuente Template
```python
"""
[Título de la Subtask]

Implementación de: VELA-XXX
Historia: VELA-YYY
Fecha: YYYY-MM-DD

Descripción:
[Qué hace este código]
"""

class NombreClase:
    """
    Clase principal para [funcionalidad].
    
    Esta implementación resuelve [problema].
    """
    
    def __init__(self):
        """Inicializar la clase."""
        pass
    
    def metodo_principal(self):
        """
        Método principal de la funcionalidad.
        
        Returns:
            dict: Resultado de la ejecución
        """
        return {"success": True}


if __name__ == "__main__":
    instance = NombreClase()
    result = instance.metodo_principal()
    print(f"Resultado: {result}")
```

### Tests Template
```python
"""
Tests unitarios para [nombre de la feature]

Jira: VELA-XXX
Historia: VELA-YYY
"""

import pytest
from src.nombre_archivo import NombreClase


class TestNombreClase:
    """Suite de tests para NombreClase."""
    
    def setup_method(self):
        """Configurar cada test."""
        self.instance = NombreClase()
    
    def test_initialization(self):
        """Test de inicialización."""
        assert self.instance is not None
    
    def test_metodo_principal(self):
        """Test del método principal."""
        result = self.instance.metodo_principal()
        assert result["success"] == True
    
    def test_metodo_principal_returns_dict(self):
        """Test que verifica el tipo de retorno."""
        result = self.instance.metodo_principal()
        assert isinstance(result, dict)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
```

### Documentación de Subtask Template
```markdown
# TASK-XXX: [Título]

## 📋 Información General
- **Historia:** VELA-YYY
- **Estado:** Completada ✅
- **Fecha:** YYYY-MM-DD

## 🎯 Objetivo
[Qué problema resuelve esta Subtask]

## 🔨 Implementación
[Cómo se resolvió]

### Archivos generados
- `src/archivo.py` - Implementación principal
- `tests/unit/test_archivo.py` - Tests unitarios
- `docs/architecture/ADR-XXX.md` - Decisión arquitectónica (si aplica)

## ✅ Criterios de Aceptación
- [x] Código implementado
- [x] Tests escritos y pasando
- [x] Documentación generada
- [x] ADR creado (si aplica)

## 🔗 Referencias
- **Jira:** [TASK-XXX](https://velalang.atlassian.net/browse/VELA-XXX)
- **Historia:** [VELA-YYY](https://velalang.atlassian.net/browse/VELA-YYY)
```

### README de Historia Template
```markdown
# VELA-XXX: [Título de la Historia]

## 📋 Información General
- **Epic:** VELA-ZZZ
- **Sprint:** Sprint N
- **Estado:** Completada ✅
- **Fecha:** YYYY-MM-DD

## 🎯 Descripción
[Descripción de la Historia de Usuario]

## 📦 Subtasks Completadas
1. **TASK-XXX**: [Título] ✅
2. **TASK-YYY**: [Título] ✅

## 🔨 Implementación
Ver archivos en:
- `src/` - Código fuente
- `tests/unit/` - Tests
- `docs/features/VELA-XXX/` - Documentación

## 📊 Métricas
- **Subtasks:** X completadas
- **Archivos creados:** Y
- **Tests escritos:** Z

## ✅ Definición de Hecho
- [x] Todas las Subtasks completadas
- [x] Código funcional
- [x] Tests pasando (>= 80% cobertura)
- [x] Documentación completa
- [x] Pull Request merged

## 🔗 Referencias
- **Jira:** [VELA-XXX](https://velalang.atlassian.net/browse/VELA-XXX)
```

---

## ⚠️ REGLAS ABSOLUTAS

### 1. CADA TAREA GENERA ENTREGABLES TANGIBLES
**NUNCA** solo cambiar estados en Jira. SIEMPRE generar:
- ✅ Código fuente funcional
- ✅ Tests unitarios (>= 80% cobertura)
- ✅ Documentación completa
- ✅ ADRs para decisiones arquitectónicas
- ✅ Commits en Git

### 2. POLÍTICA DE GIT (OBLIGATORIA)
#### ✅ UNA RAMA POR HISTORIA
```bash
git checkout -b feature/VELA-XXX-descripcion
```

#### ✅ UN COMMIT POR SUBTASK
```bash
git commit -m "feat(VELA-XXX): TASK-XXX @titulo

- Descripción del cambio
- Archivos modificados
- Tests agregados

Refs: VELA-XXX"
```

### 3. ESTRUCTURA DE ARCHIVOS OBLIGATORIA
```
docs/features/VELA-XXX/
├── README.md              # Resumen de la Historia
├── TASK-001.md            # Doc de Subtask 1
├── TASK-002.md            # Doc de Subtask 2

docs/architecture/
├── ADR-XXX-titulo.md      # Decisión arquitectónica

src/
├── feature-name.py        # Código fuente

tests/unit/
├── test_feature-name.py   # Tests unitarios
```

### 4. CHECKLIST ANTES DE MARCAR SUBTASK COMO "FINALIZADA"
- [ ] ✅ Código creado en src/
- [ ] ✅ Tests creados en tests/unit/
- [ ] ✅ Tests pasando (ejecutar con pytest)
- [ ] ✅ Documentación creada en docs/features/
- [ ] ✅ ADR creado (si es decisión arquitectónica)
- [ ] ✅ Commit realizado con mensaje descriptivo

---

## 🎯 ESTRATEGIA DE IMPLEMENTACIÓN

### Prioridades
1. **Core Language** (AST, tipos, IR) - Base sólida
2. **Compiler Frontend** (lexer, parser, semantic) - Compilación funcional
3. **VM Backend** (VelaVM, bytecode, GC) - Ejecución
4. **UI Framework** - Diferenciador principal
5. **Standard Library** - Utilidades
6. **Tooling** - Developer experience

### Enfoque de Desarrollo
- **Incremental**: Funcionalidad básica primero, optimizaciones después
- **Test-Driven**: Tests antes del código
- **Documentado**: Todo documentado con ADRs
- **Modular**: Componentes desacoplados
- **Performante**: Optimizaciones desde el inicio

### Métricas de Éxito
- **Compilación**: Código compila sin errores
- **Tests**: Cobertura >= 80%, todos pasando
- **Performance**: Benchmarks competitivos
- **Ecosystem**: Comunidad creciente
- **Adopción**: Proyectos reales usando Vela

---

## 📞 INTERACCIÓN CON USUARIO

### Cuando el usuario dice "Desarrolla Historia VELA-XXX"
1. Leer la historia en el backlog
2. Obtener lista de subtasks
3. Crear rama feature/VELA-XXX
4. Desarrollar cada subtask en orden
5. Generar entregables completos
6. Crear PR y esperar aprobación
7. Merge a main

### Cuando el usuario pregunta algo
- Responder con información precisa del contexto
- Si no sabes, investigar con herramientas
- Mantener conversación técnica pero clara

### Cuando hay errores
- Diagnosticar el problema
- Proponer solución con código
- Implementar fix si es claro
- Preguntar si no está claro

---

## 🔧 HERRAMIENTAS DISPONIBLES

### Para investigación
- `grep_search`: Buscar en código
- `read_file`: Leer archivos
- `list_dir`: Listar directorios
- `run_in_terminal`: Ejecutar comandos

### Para desarrollo
- `create_file`: Crear archivos
- `replace_string_in_file`: Editar archivos
- `run_in_terminal`: Git, build, test

### Para documentación
- `create_file`: Crear docs
- `replace_string_in_file`: Actualizar docs

---

## 🎯 OBJETIVO FINAL

Crear un lenguaje de programación moderno que combine:
- **La expresividad de TypeScript**
- **La performance de Rust**
- **La reactividad de Flutter**
- **La simplicidad de Go**
- **El ecosystem de Node.js**

Con un enfoque en **developer experience** y **productividad**, permitiendo construir aplicaciones web, mobile y backend con una sola sintaxis unificada.

**Estado Actual**: ~60% completado, con base sólida en compiler, VM y UI framework. Listo para expansión del ecosystem y tooling.

---

*Prompt generado el 9 de diciembre de 2025. Versión 3.0 - Completo y actualizado con el estado actual del proyecto.*