# Vela Language - Visión General

## 1. Descripción

**Vela** es un lenguaje de programación moderno, de alto nivel y multiplataforma, diseñado para:

- **Web** (HTML/JS/WASM)
- **Móvil** (iOS/Android)
- **Desktop** (Windows/macOS/Linux)
- **Backend** (API REST, microservicios)

Vela combina lo mejor de **TypeScript** (tipado estructural) y **Flutter** (UI declarativa y reactiva), integrando:
- Sistema de programación **funcional obligatoria**
- **Programación Orientada a Objetos** (clases, herencia, interfaces)
- **Hooks** y **ciclo de vida** de componentes
- **Runtime reactivo** basado en signals

---

## 2. Características Principales

### 2.1 Sistema de Tipos

- **Tipado estructural**: similar a TypeScript, los tipos se comparan por su estructura, no por su nombre
- **Inferencia de tipos**: el compilador deduce tipos automáticamente
- **Null-safety**: manejo seguro de valores nulos con `?` y operadores safe navigation
- **Generics**: soporte completo para tipos genéricos
- **ADTs**: Algebraic Data Types (Option, Result, enums)

### 2.2 Paradigma Funcional Obligatorio

- **Funciones puras por defecto**: las funciones no pueden tener side effects sin marcarlo explícitamente
- **Inmutabilidad por defecto**: todas las variables son inmutables (NO existen `const` ni `let`)
- **Mutabilidad explícita**: usar `state` para variables mutables (única forma de mutabilidad)
- **Sin loops tradicionales**: NO existen `for`, `while`, `loop` - solo métodos funcionales
- **Sin null**: NO existe `null` - solo `Option<T>` con `Some`/`None`

```vela
# Inmutable por defecto (sin const ni let)
name: String = "Cristian"
PI: Float = 3.14159

# name = "Otro"  # ERROR: no se puede reasignar inmutable

# Mutable explícita con state
state age: Number = 37
age = 38  # OK

# ❌ NO: loops tradicionales
# for i in 0..10 { print(i) }

# ✅ SÍ: métodos funcionales
(0..10).forEach(i => print(i))
numbers.map(x => x * 2)
       .filter(x => x > 5)
       .reduce((a, b) => a + b, 0)

# ❌ NO: null
# value: String? = null

# ✅ SÍ: Option<T>
value: Option<String> = None
value2: Option<String> = Some("hello")
```

### 2.3 Programación Orientada a Objetos

- **Clases**: soporte completo para clases con constructores, propiedades y métodos
- **Clases abstractas**: `abstract class` para clases base
- **Interfaces**: `interface` para contratos de tipos
- **Herencia**: `extends` para heredar de clases
- **Implementación**: `implements` para implementar interfaces
- **Override**: métodos pueden marcarse como `override`
- **Sobrecarga**: métodos pueden sobrecargarse con `overload`

```vela
abstract class Animal {
  name: String
  
  constructor(name: String) {
    this.name = name
  }
  
  abstract fn makeSound() -> String
}

class Dog extends Animal {
  override fn makeSound() -> String {
    return "Woof!"
  }
  
  # Sobrecarga de método
  overload fn bark() -> void {
    print("Woof!")
  }
  
  overload fn bark(times: Number) -> void {
    for i in 0..times {
      print("Woof!")
    }
  }
}

interface Flyable {
  fn fly() -> void
}

class Bird extends Animal implements Flyable {
  override fn makeSound() -> String {
    return "Tweet!"
  }
  
  fn fly() -> void {
    print("${this.name} is flying")
  }
}
```

### 2.4 Decoradores

Los decoradores son anotaciones que modifican el comportamiento de módulos, clases y funciones:

- `@package`: define un paquete
- `@module`: define un módulo
- `@library`: define una biblioteca
- `@deprecated`: marca como obsoleto
- `@override`: marca explícitamente un override (opcional pero recomendado)
- `@test`: marca una función como test

```vela
@package("com.example.myapp")
@module("core")

@deprecated("Use NewComponent instead")
class OldComponent extends Widget {
  # ...
}

@test
fn testUserCreation() {
  # ...
}
```

### 2.5 Sistema de Imports y Visibilidad

Vela soporta múltiples tipos de imports para organizar código:

```vela
# Package import (paquetes de terceros)
import 'package:utils'
import 'package:http/client'

# Library import (bibliotecas del sistema)
import 'library:math'
import 'library:collections'

# Module import (módulos del proyecto)
import 'module:network'
import 'module:auth/service'

# Extension import (extensiones)
import 'extension:charts'
import 'extension:animations'

# System import (APIs del sistema operativo)
import 'system:filesystem'
import 'system:process'

# Assets import (recursos estáticos)
import 'assets:images'
import 'assets:fonts'
import 'assets:config.json' as config

# Imports selectivos
import 'package:utils' show { HashMap, ArrayList }
import 'library:math' hide { deprecated_function }

# Alias de imports
import 'package:long_package_name' as lpn
```

**Visibilidad** (sin `export`):

```vela
# Público (accesible desde otros módulos)
public fn publicFunction() -> void {
  # ...
}

public class PublicClass {
  # ...
}

# Privado (solo dentro del módulo)
private fn internalHelper() -> void {
  # ...
}

# Por defecto: público
fn defaultIsPublic() -> void {
  # accesible desde fuera del módulo
}
```

### 2.6 UI Declarativa - StatefulWidget y StatelessWidget (como Flutter)

Inspirado en **Flutter**, Vela tiene dos tipos de widgets:
- **StatefulWidget**: widgets con estado mutable
- **StatelessWidget**: widgets sin estado (inmutables, puros)

#### StatefulWidget

Los **StatefulWidget** contienen lógica de negocio y estado mutable:

```vela
class Counter extends StatefulWidget {
  # Estado local (mutable)
  state count: Number = 0
  
  # Computed values
  computed isEven: Bool {
    return this.count % 2 == 0
  }
  
  # Métodos
  fn increment() -> void {
    this.count += 1
  }
  
  fn decrement() -> void {
    this.count -= 1
  }
  
  # Lifecycle hooks
  mount() {
    print("Counter mounted")
  }
  
  update() {
    print("Counter updated: ${this.count}")
  }
  
  destroy() {
    print("Counter destroyed")
  }
  
  # Build method - retorna el árbol de widgets
  fn build() -> Widget {
    return Column(
      children: [
        Text("Count: ${this.count}"),
        Text(this.isEven ? "Even" : "Odd"),
        Row(
          children: [
            Button(
              text: "−",
              onClick: this.decrement
            ),
            Button(
              text: "+",
              onClick: this.increment
            )
          ]
        )
      ]
    )
  }
}
```

#### StatelessWidget

Los **StatelessWidget** son widgets sin estado (presentacionales):

```vela
class Card extends StatelessWidget {
  # Propiedades inmutables
  title: String
  content: String
  color: Color
  
  constructor(title: String, content: String, color: Color) {
    this.title = title
    this.content = content
    this.color = color
  }
  
  # Build method - retorna el árbol de widgets
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
```

#### Árbol de Widgets Completo

```vela
class MyApp extends StatelessWidget {
  fn build() -> Widget {
    return App(
      title: "My Vela App",
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(
            "Welcome to Vela",
            style: TextStyle(
              fontSize: 24,
              fontWeight: FontWeight.bold,
              color: Color.white
            )
          ),
          SizedBox(height: 20),
          Button(
            text: "Click me",
            variant: ButtonVariant.primary,
            onClick: () => print("Button clicked!")
          ),
          Image(
            src: "assets:logo.png",
            width: 100,
            height: 100,
            fit: ImageFit.contain
          )
        ]
      )
    )
  }
}
```

### 2.7 Hooks y Ciclo de Vida

#### Hooks Reactivos

```vela
class TodoList extends StatefulWidget {
  # state: variable mutable reactiva
  state todos: List<String> = []
  state filter: String = "all"
  
  # computed: valor derivado (se recalcula automáticamente)
  computed filteredTodos: List<String> {
    if this.filter == "all" {
      return this.todos
    }
    return this.todos.filter(t => t.contains(this.filter))
  }
  
  # memo: caché de valor computado costoso
  memo expensiveCalculation: Number {
    return this.todos
      .map(t => t.length)
      .reduce((a, b) => a + b, 0)
  }
  
  # effect: side effect reactivo (se ejecuta cuando cambian dependencias)
  effect {
    print("Todos changed: ${this.todos.length} items")
    # Cleanup automático cuando el componente se destruye
  }
  
  # watch: observador explícito de cambios
  watch(this.filter) {
    print("Filter changed to: ${this.filter}")
  }
  
  fn addTodo(text: String) -> void {
    this.todos = this.todos.append(text)
  }
  
  fn build() -> Widget {
    return ListView(
      children: this.filteredTodos.map(todo => Text(todo))
    )
  }
}
```

#### Ciclo de Vida de Componentes

```vela
class LifecycleExample extends StatefulWidget {
  state data: Option<String> = None
  
  # mount: se ejecuta cuando el componente se monta en el árbol
  mount() {
    print("Component mounted")
    this.fetchData()
  }
  
  # update: se ejecuta después de cada actualización de estado
  update() {
    print("Component updated")
  }
  
  # beforeUpdate: se ejecuta antes de aplicar cambios al DOM
  beforeUpdate() {
    print("About to update")
  }
  
  # afterUpdate: se ejecuta después de aplicar cambios al DOM
  afterUpdate() {
    print("DOM updated")
  }
  
  # destroy: se ejecuta cuando el componente se desmonta
  destroy() {
    print("Component destroyed - cleanup")
    this.cancelRequests()
  }
  
  async fn fetchData() -> void {
    result: Result<String, Error> = await http.get("/api/data")
    match result {
      Ok(data) => this.data = Some(data)
      Err(e) => print("Error: ${e}")
    }
  }
  
  fn cancelRequests() -> void {
    # Cleanup de recursos
  }
  
  fn build() -> Widget {
    return match this.data {
      Some(d) => Text(d)
      None => Text("Loading...")
    }
  }
}
```

---

## 3. Motores y Runtimes

### 3.1 VelaVM (Bytecode Interpreter)

- **Bytecode interpreter** similar a JVM/Python VM
- **ARC híbrido**: Automatic Reference Counting + cycle detection
- **Scheduler reactivo**: manejo de signals, computed, effects
- **Actor system**: concurrencia con aislamiento de memoria
- **Event loop**: manejo de async/await

### 3.2 VelaNative (AOT Compilation)

- **LLVM backend**: compilación a código nativo
- **Optimización agresiva**: inline, dead code elimination, constant folding
- **Runtime mínimo**: runtime library en C/C++ para GC y primitivas
- **Performance**: igual o superior a lenguajes compilados

### 3.3 VelaWeb (Web Platform)

- **JavaScript backend**: transpilación a JS moderno (ES2020+)
- **WebAssembly backend**: compilación a WASM para lógica crítica
- **DOM renderer**: integración directa con DOM del browser
- **Signals reactivity**: sistema reactivo optimizado para web
- **Hot reload**: desarrollo rápido con recarga en caliente

### 3.4 VelaDesktop (Desktop Applications)

- **C++ host engine**: engine nativo embebido
- **Skia renderer**: rendering 2D de alta calidad
- **Native APIs**: acceso a filesystem, process, window management
- **Cross-platform**: Windows, macOS, Linux con un solo codebase

### 3.5 VelaMobile Runtime (iOS/Android)

- **iOS**: Swift bridging + UIKit/SwiftUI renderer
- **Android**: Kotlin bridging + Jetpack Compose renderer
- **Native performance**: compilación AOT para móvil
- **Platform channels**: comunicación bidireccional con código nativo

### 3.6 Cores del Sistema

- **Networking core**: HTTP client, WebSocket, TCP/UDP sockets
- **IO core**: filesystem, streams, buffers
- **Plugin system**: carga dinámica de extensiones nativas
- **FFI**: Foreign Function Interface para interop con C/C++

---

## 4. Herramientas (Tooling)

### 4.1 Vela CLI

```bash
# Crear nuevo proyecto
vela create my_app
vela create my_app --template=web
vela create my_app --template=mobile

# Build
vela build                    # bytecode (VelaVM)
vela build --release          # optimizado
vela build --target=native    # LLVM AOT
vela build --target=web       # JS + WASM
vela build --target=ios       # iOS bundle
vela build --target=android   # Android APK

# Run
vela run                      # ejecutar en VelaVM
vela run --watch              # hot reload
vela run --target=web         # abrir en browser

# Format
vela fmt                      # formatear todo el proyecto
vela fmt src/main.vela        # formatear archivo específico

# Lint
vela lint                     # análisis estático
vela lint --fix               # auto-fix de issues

# Test
vela test                     # correr todos los tests
vela test --coverage          # con coverage report
vela test src/utils_test.vela # test específico

# Package management
vela install                  # instalar dependencias
vela add package:lodash       # agregar dependencia
vela remove package:lodash    # remover dependencia
vela publish                  # publicar a registry

# DevTools
vela devtools                 # abrir DevTools
vela doctor                   # diagnóstico de instalación
```

### 4.2 Package Manager

- **Registro centralizado**: npm-like registry para paquetes de Vela
- **Versionado semántico**: SemVer para gestión de versiones
- **Lock file**: `vela.lock` para reproducibilidad
- **Manifest**: `vela.yaml` para definir paquetes

```yaml
# vela.yaml
name: my_app
version: 1.0.0
description: My awesome Vela app

dependencies:
  package:http: ^2.0.0
  package:router: ^1.5.0
  extension:charts: ^3.1.0

dev_dependencies:
  package:test: ^1.0.0
  
targets:
  - web
  - mobile
  - desktop
```

### 4.3 Formatter y Linter

- **Formatter**: estilo consistente automático (similar a Prettier/Black)
- **Linter**: detección de code smells, anti-patterns, vulnerabilidades
- **Auto-fix**: corrección automática de issues simples
- **Integración con editores**: VS Code, IntelliJ, Vim

### 4.4 DevTools

#### UI Inspector
- **Widget tree**: visualización jerárquica de widgets
- **Live editing**: modificar propiedades en tiempo real
- **Layout debugging**: grillas y guías visuales
- **Performance overlay**: FPS, frame time, jank detection

#### Signal Graph Visualizer
- **Dependency graph**: visualizar relaciones entre signals
- **Value tracking**: ver valores actuales de signals/computed
- **Update tracking**: timeline de propagaciones reactivas
- **Memory profiler**: detectar leaks de signals

#### Timeline & Profiler
- **CPU profiler**: identificar bottlenecks
- **Memory profiler**: analizar uso de memoria
- **Network inspector**: monitorear requests HTTP/WebSocket
- **Event timeline**: timeline completo de eventos del app

---

## 5. Filosofía de Diseño

### 5.1 Developer Experience (DX)

- **Aprendizaje gradual**: sintaxis familiar para devs de TypeScript/Dart/Kotlin
- **Error messages claros**: mensajes de error descriptivos y accionables
- **Hot reload**: feedback instantáneo durante desarrollo
- **Type inference**: menos boilerplate, más productividad

### 5.2 Performance

- **Zero-cost abstractions**: abstracciones sin overhead en runtime
- **Lazy evaluation**: computed values se calculan solo cuando se necesitan
- **Memoization**: caché automático de valores costosos
- **Tree shaking**: eliminación de código no usado

### 5.3 Safety

- **Null-safety**: eliminación de null pointer exceptions
- **Memory safety**: ARC + cycle detection previenen leaks
- **Type safety**: sistema de tipos fuerte previene bugs
- **Immutability by default**: reduce bugs de estado mutable

### 5.4 Multiplataforma

- **Write once, run anywhere**: un codebase para todas las plataformas
- **Platform channels**: acceso controlado a APIs nativas
- **Adaptive UI**: componentes que se adaptan a cada plataforma
- **Native performance**: compilación AOT para móvil/desktop

---

## 6. Casos de Uso

### 6.1 Aplicaciones Web (SPA)

```vela
import 'library:ui'
import 'package:router'

class App extends StatelessWidget {
  fn build() -> Widget {
    return Router(
      routes: [
        Route(path: "/", component: HomePage),
        Route(path: "/about", component: AboutPage),
        Route(path: "/user/:id", component: UserPage)
      ]
    )
  }
}
```

### 6.2 Aplicaciones Móviles

```vela
import 'library:ui'
import 'system:camera'

class CameraApp extends StatefulWidget {
  state photo: Option<Image> = None
  
  async fn takePhoto() -> void {
    result = await Camera.capture()
    match result {
      Ok(img) => this.photo = Some(img)
      Err(e) => print("Error: ${e}")
    }
  }
  
  fn build() -> Widget {
    return Scaffold(
      appBar: AppBar(title: "Camera App"),
      body: Center(
        child: match this.photo {
          Some(img) => Image(data: img)
          None => Button(text: "Take Photo", onClick: this.takePhoto)
        }
      )
    )
  }
}
```

### 6.3 Backend / API REST

```vela
import 'package:http/server'
import 'module:database'

fn main() async {
  server = HttpServer.create(port: 8080)
  
  server.get("/users", async (req, res) => {
    users = await Database.query("SELECT * FROM users")
    res.json(users)
  })
  
  server.post("/users", async (req, res) => {
    user = User.fromJson(req.body)
    saved = await Database.insert(user)
    res.status(201).json(saved)
  })
  
  await server.listen()
  print("Server running on port 8080")
}
```

### 6.4 Desktop Applications

```vela
import 'library:ui'
import 'system:filesystem'

class TextEditor extends StatefulWidget {
  state content: String = ""
  state filePath: Option<String> = None
  
  async fn open() -> void {
    path = await FileSystem.openDialog()
    match path {
      Some(p) => {
        this.content = await FileSystem.readText(p)
        this.filePath = path
      }
      None => {}
    }
  }
  
  async fn save() -> void {
    match this.filePath {
      Some(p) => await FileSystem.writeText(p, this.content)
      None => await this.saveAs()
    }
  }
  
  fn build() -> Widget {
    return Window(
      title: "Vela Text Editor",
      menu: MenuBar(
        items: [
          MenuItem(label: "Open", shortcut: "Ctrl+O", onClick: this.open),
          MenuItem(label: "Save", shortcut: "Ctrl+S", onClick: this.save)
        ]
      ),
      child: TextField(
        value: this.content,
        multiline: true,
        onChange: (text) => this.content = text
      )
    )
  }
}
```

---

## 7. Comparación con Otros Lenguajes

| Feature | Vela | TypeScript | Dart/Flutter | Rust |
|---------|------|------------|--------------|------|
| Tipado estructural | ✅ | ✅ | ❌ | ❌ |
| Null-safety | ✅ | ✅ | ✅ | ✅ |
| Inmutabilidad por defecto | ✅ | ❌ | ❌ | ❌ |
| UI declarativa integrada | ✅ | ❌ | ✅ | ❌ |
| Sistema reactivo built-in | ✅ | ❌ | ✅ (parcial) | ❌ |
| Actor model concurrency | ✅ | ❌ | ❌ (isolates) | ❌ |
| Compilación AOT | ✅ | ❌ | ✅ | ✅ |
| Hot reload | ✅ | ✅ (con tools) | ✅ | ❌ |
| Multiplataforma | ✅ | ✅ (web) | ✅ | ❌ |

---

## 8. Roadmap

### Vela 1.0 (MVP) - Q1-Q2 2026
- ✅ Core language (grammar, lexer, parser, type checker)
- ✅ VelaVM bytecode interpreter
- ✅ Sistema reactivo (signals, computed, effects)
- ✅ Actor system básico
- ✅ UI framework core
- ✅ Standard library básica
- ✅ CLI básico (build, run, fmt)
- ✅ Documentación fundamental

### Vela 2.0 (Platform Complete) - Q3-Q4 2026
- ✅ VelaNative (LLVM backend)
- ✅ VelaWeb (JS + WASM)
- ✅ LSP (Language Server Protocol)
- ✅ Testing framework
- ✅ Package manager completo
- ✅ DevTools básico
- ✅ Documentación completa

### Vela 3.0 (Maturity) - Q1-Q2 2027
- ✅ VelaMobile (iOS/Android runtimes)
- ✅ VelaDesktop (native apps)
- ✅ FFI system
- ✅ Optimizaciones avanzadas
- ✅ DevTools avanzado (profiler, inspector)
- ✅ Cloud deployment tools
- ✅ Ecosystem maduro

---

## 9. Comunidad y Recursos

- **Website**: https://velalang.org
- **Docs**: https://docs.velalang.org
- **GitHub**: https://github.com/velalang/vela
- **Discord**: https://discord.gg/velalang
- **Package Registry**: https://packages.velalang.org
- **Playground**: https://play.velalang.org

---

## 10. Licencia

Vela es open-source bajo licencia **MIT**.
