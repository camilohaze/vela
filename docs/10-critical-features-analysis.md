# 10. Análisis de Características Críticas

## Respuestas a Preguntas Clave

### 1. ¿Vela soporta eventos (on, emit)?

**Respuesta: 🟡 PARCIAL**

Actualmente, Vela soporta eventos **solo en el contexto de UI widgets**, pero **NO tiene un sistema de eventos genérico** (`on`, `emit`) para comunicación entre componentes.

#### ✅ Lo que SÍ tiene (eventos de UI):

```vela
# Eventos en widgets
Button {
  text: "Click me",
  onClick: fn() {
    print("Button clicked!")
  }
}

TextField {
  placeholder: "Enter text",
  onChange: fn(value: String) {
    print("Value changed: ${value}")
  }
}
```

**Fuente**: `06-ui-declarative.md` línea 178-180 muestra eventos como `onClick`, `onChange`, etc. en widgets.

#### ❌ Lo que NO tiene (sistema de eventos genérico):

```vela
# ❌ ESTO NO EXISTE EN VELA ACTUALMENTE
class EventEmitter {
  fn on(event: String, handler: (Any) => void) { }
  fn emit(event: String, data: Any) { }
  fn off(event: String, handler: (Any) => void) { }
}

# ❌ TAMPOCO ESTO
@event("userLogin")
fn handleLogin(user: User) { }

emit("userLogin", user: currentUser)
```

#### 🔴 Problema Identificado:

Para ser un lenguaje de alto nivel completo, Vela **NECESITA** un sistema de eventos genérico que permita:

1. **Event Bus global**: Comunicación entre componentes desacoplados
2. **Custom events**: Eventos definidos por el usuario
3. **Event listeners**: Suscripciones tipo-seguras
4. **Event lifecycle**: Propagación, cancelación, bubbling

---

### 2. ¿Cómo evitar conflicto entre `@module` existente y `@module` de DI?

**Respuesta: Usar `@injectable` para DI y reservar `@module` para organización de código**

#### Problema Detectado:

Vela **YA tiene** `@module` definido para **organización de código** (ver `keywords-reference.md` línea 1250):

```vela
# @module existente (organización de código)
@module("auth")
class AuthService { }

# module declaration para paquetes (01-grammar-and-syntax.md línea 758-764)
module com.example.myapp;  # Declaración de paquete del archivo
```

**Fuente**: `01-grammar-and-syntax.md` línea 272 muestra que `module X.Y.Z;` es solo para **declarar el paquete** del archivo, similar a Java/Kotlin.

#### ✅ Solución: Usar `@container` para DI

Para **evitar conflictos**, usamos el estándar de industria:

| Concepto | Decorator | Propósito |
|----------|-----------|-----------|
| Servicio inyectable | `@injectable` | Marca clase como inyectable con scope |
| Parámetro de inyección | `@inject` | Marca parámetro para inyección automática |
| **Container DI** | `@container` | Módulo que agrupa providers (estándar Spring/Angular/NestJS) |
| Provider factory | `@provides` | Marca método factory como provider |

#### Código actualizado SIN conflictos:

```vela
# ============================================
# 1. DECLARACIÓN DE PAQUETE (module existente)
# ============================================
module com.example.myapp.services;

# ============================================
# 2. ORGANIZACIÓN DE CÓDIGO (@module existente)
# ============================================
@module("auth")
class AuthService {
  fn login(username: String, password: String) -> Result<User, Error> { }
}

# ============================================
# 3. INYECCIÓN DE DEPENDENCIAS (DI)
# ============================================

# Servicio inyectable
@injectable(scope: Scope.Singleton)
class UserService {
  fn getUsers() -> List<User> { /* ... */ }
}

# Inyectar dependencias
@injectable
class AuthController {
  constructor(
    @inject userService: UserService,
    @inject logger: Logger
  ) { }
}

# ✅ SOLUCIÓN: Usar @container (estándar de industria)
@container
class AppContainer {
  @provides(scope: Scope.Singleton)
  fn provideDatabase() -> Database {
    return Database(url: "mongodb://localhost")
  }
  
  @provides
  fn provideLogger() -> Logger {
    return ConsoleLogger()
  }
}

# Uso del contenedor DI
injector = Injector(containers: [AppContainer()])
controller = injector.get<AuthController>()
```

#### Alternativas consideradas:

1. **`@container`** ⭐ (RECOMENDADO)
   - Término estándar en DI (Spring, Angular, NestJS)
   - Claro y universalmente reconocido
   - No confunde con `module` existente

2. **`@diModule`**
   - Específico pero menos común
   - Puede confundir con módulos del sistema

3. **`@injectionModule`**
   - Muy explícito pero demasiado verboso

**Decisión final**: Usar **`@container`** para módulos DI (estándar de industria).

---

### 3. ¿Vela soporta patrones de diseño de alto nivel?

**Respuesta: 🟡 PARCIAL - Necesita extensiones críticas**

Vela tiene **buena base** para patrones de diseño, pero le faltan características críticas para soportarlos **completamente**.

#### ✅ Patrones que SÍ soporta bien:

| Patrón | Soporte | Ejemplo |
|--------|---------|---------|
| **Strategy** | ✅ COMPLETO | Funciones de primera clase + interfaces |
| **Observer** | ✅ COMPLETO | Signals + reactive system |
| **Builder** | ✅ COMPLETO | Named parameters + immutability |
| **Factory Method** | ✅ COMPLETO | Static methods + generics |
| **Template Method** | ✅ COMPLETO | Abstract classes + override |
| **Decorator (structural)** | ✅ COMPLETO | Composición + interfaces |
| **Facade** | ✅ COMPLETO | Classes + encapsulation |
| **Adapter** | ✅ COMPLETO | Interfaces + composition |

**Ejemplo - Strategy Pattern**:
```vela
# Strategy Pattern - ✅ FUNCIONA PERFECTAMENTE
interface PaymentStrategy {
  fn process(amount: Number) -> Result<Receipt, Error>;
}

class CreditCardPayment implements PaymentStrategy {
  override fn process(amount: Number) -> Result<Receipt, Error> {
    # Procesar con tarjeta
    return Ok(Receipt(amount: amount, method: "Credit Card"))
  }
}

class PayPalPayment implements PaymentStrategy {
  override fn process(amount: Number) -> Result<Receipt, Error> {
    # Procesar con PayPal
    return Ok(Receipt(amount: amount, method: "PayPal"))
  }
}

# Uso
fn checkout(strategy: PaymentStrategy, amount: Number) {
  result = strategy.process(amount)
  match result {
    Ok(receipt) => print("Payment successful: ${receipt}")
    Err(error) => print("Payment failed: ${error}")
  }
}
```

**Ejemplo - Observer Pattern con Signals**:
```vela
# Observer Pattern - ✅ FUNCIONA CON SIGNALS
class Stock {
  state price = Signal(100.0)
  
  fn updatePrice(newPrice: Number) {
    price.set(newPrice)
  }
}

class StockWatcher {
  fn watch(stock: Stock) {
    effect(fn() {
      currentPrice = stock.price.get()
      print("Price changed to: ${currentPrice}")
    })
  }
}
```

#### 🟡 Patrones con soporte PARCIAL:

| Patrón | Limitación | Qué falta |
|--------|-----------|-----------|
| **Singleton** | Manual | ❌ Necesita DI con `@injectable(scope: Singleton)` |
| **Dependency Injection** | No existe | ❌ Necesita sistema DI completo |
| **Repository** | Manual | ❌ Necesita DI + async/await mejorado |
| **State** | Local | ❌ Necesita State Management global |
| **Command** | Básico | ❌ Necesita Event Bus + undo/redo |
| **Mediator** | No existe | ❌ Necesita Event Bus |
| **Memento** | Manual | ❌ Necesita serialización automática |

**Ejemplo - Singleton sin DI (actual - manual)**:
```vela
# ❌ PROBLEMA: Singleton manual es verboso y error-prone
class Database {
  private static state instance: Option<Database> = None
  
  private constructor() { }
  
  public static fn getInstance() -> Database {
    return match instance {
      Some(db) => db
      None => {
        newDb = Database()
        instance = Some(newDb)
        newDb
      }
    }
  }
}
```

**Ejemplo - Singleton CON DI (propuesto - automático)**:
```vela
# ✅ SOLUCIÓN: Con DI es automático y type-safe
@injectable(scope: Scope.Singleton)
class Database {
  constructor() { }
}

# El contenedor DI garantiza UNA SOLA instancia
injector = Injector()
db1 = injector.get<Database>()
db2 = injector.get<Database>()
# db1 === db2 (misma instancia)
```

#### ❌ Patrones que NO soporta (necesitan características nuevas):

| Patrón | Qué necesita | Prioridad |
|--------|--------------|-----------|
| **Proxy dinámico** | Reflection/Metaprogramming | 🟢 P3 (Vela 3.0) |
| **Interceptor** | AOP (Aspect-Oriented Programming) | 🟢 P3 (Vela 3.0) |
| **Chain of Responsibility** | Event Bus + middleware | 🔴 P1 (Vela 1.0) |
| **Visitor** | Pattern matching avanzado + reflection | 🟡 P2 (Vela 2.0) |
| **Flyweight** | Object pooling + memory profiling | 🟢 P3 (Vela 3.0) |

---

## 🔴 Características CRÍTICAS que faltan (MVP 1.0)

Para que Vela sea un lenguaje de **alto nivel completo** que soporte la **mayoría de patrones de diseño**, necesita:

### 1. **Sistema de Inyección de Dependencias (DI)** 🔴 CRÍTICO

**Prioridad**: P0 (MVP 1.0)

**Keywords nuevos**:
- `@injectable` - Marca clase como inyectable
- `@inject` - Marca parámetro para inyección
- `@container` - Define contenedor DI que agrupa providers (estándar Spring/Angular/NestJS)
- `@provides` - Factory method para providers
- `@controller` - Define controlador REST/API con routing automático

**Código de ejemplo DI**:
```vela
# Container DI
@container
class AppContainer {
  @provides(scope: Scope.Singleton)
  fn provideDatabase() -> Database {
    return Database(url: "mongodb://localhost")
  }
}
```

**Código de ejemplo REST API**:
```vela
# Controlador REST con DI
@controller("/api/users")
class UserController {
  constructor(@inject userService: UserService) { }
  
  @get("/")
  fn getAll() -> Result<List<User>, Error> {
    return Ok(userService.getUsers())
  }
  
  @get("/:id")
  fn getById(id: String) -> Result<User, Error> {
    return userService.getUserById(id)
  }
  
  @post("/")
  fn create(user: User) -> Result<User, Error> {
    return userService.createUser(user)
  }
  
  @put("/:id")
  fn update(id: String, user: User) -> Result<User, Error> {
    return userService.updateUser(id, user)
  }
  
  @delete("/:id")
  fn delete(id: String) -> Result<void, Error> {
    return userService.deleteUser(id)
  }
}
```

**Patrones que desbloquea**:
- ✅ Singleton (automático)
- ✅ Factory (automático)
- ✅ Dependency Injection
- ✅ Repository
- ✅ Service Layer
- ✅ Inversion of Control
- ✅ Controller Pattern (REST APIs)
- ✅ MVC/MVVM arquitectura

---

### 2. **Sistema de Eventos Genérico (Event Bus)** 🔴 CRÍTICO

**Prioridad**: P0 (MVP 1.0)

**Keywords nuevos**:
- `EventBus` - Class base para event bus
- `EventEmitter` - Interface para emisores
- `EventListener` - Type para listeners
- `on` - Keyword para suscribirse
- `emit` - Keyword para emitir
- `off` - Keyword para desuscribirse

**Código propuesto**:
```vela
# Event Bus genérico type-safe
class EventBus<T> {
  private state listeners: Dict<String, List<(T) => void>> = {}
  
  fn on(event: String, handler: (T) => void) -> Subscription {
    listeners[event] = (listeners[event] ?? []).push(handler)
    return Subscription(
      unsubscribe: fn() { off(event, handler) }
    )
  }
  
  fn emit(event: String, data: T) {
    listeners[event]?.forEach(fn(handler) {
      handler(data)
    })
  }
  
  fn off(event: String, handler: (T) => void) {
    listeners[event] = listeners[event]?.filter(fn(h) => h != handler) ?? []
  }
}

# Uso type-safe
type UserEvent = UserLogin(user: User) | UserLogout(userId: String)

userEventBus = EventBus<UserEvent>()

# Suscribirse
userEventBus.on("login", fn(event: UserEvent) {
  match event {
    UserLogin(user) => print("User logged in: ${user.name}")
    _ => {}
  }
})

# Emitir
userEventBus.emit("login", UserLogin(user: currentUser))
```

**Patrones que desbloquea**:
- ✅ Observer (mejorado)
- ✅ Event-Driven Architecture
- ✅ Mediator
- ✅ Command (con eventos)
- ✅ Chain of Responsibility
- ✅ Publish-Subscribe

---

### 3. **State Management Global** 🔴 CRÍTICO

**Prioridad**: P0 (MVP 1.0)

**Keywords nuevos**:
- `Store<T>` - Clase base para stores
- `Action` - Type para acciones
- `Reducer` - Type para reducers
- `dispatch` - Keyword para enviar acciones
- `@connect` - Conectar widget a store
- `@select` - Optimización de subscripción
- `@persistent` - Persistencia automática

**Patrones que desbloquea**:
- ✅ State (global)
- ✅ Command (con actions)
- ✅ Memento (con time-travel)
- ✅ Undo/Redo
- ✅ Event Sourcing

---

### 4. **Pattern Matching Avanzado** 🟡 IMPORTANTE

**Prioridad**: P1 (Vela 2.0)

**Extensiones necesarias**:
```vela
# Guard clauses avanzados
match value {
  x if x > 0 && x < 10 => "Small positive"
  x if x >= 10 => "Large positive"
  _ => "Other"
}

# Destructuring avanzado
match person {
  User(name: "John", age: age, ..rest) => "John is ${age}"
  User(name: name, ..rest) => "User ${name}"
  _ => "Unknown"
}

# Pattern en lambdas
users.filter(fn(User(age: age, ..)) => age >= 18)
```

**Patrones que desbloquea**:
- ✅ Visitor (mejorado)
- ✅ Interpreter
- ✅ Expression Problem

---

### 5. **Reflection/Metaprogramming** 🟢 DESEABLE

**Prioridad**: P2 (Vela 3.0)

**Keywords nuevos**:
- `typeof` - Obtener tipo en runtime
- `reflect` - API de reflection
- `@meta` - Metadatos en clases

**Patrones que desbloquea**:
- ✅ Proxy dinámico
- ✅ Interceptor
- ✅ Decorator dinámico
- ✅ Serialization genérica

---

## 📊 Resumen: Cobertura de Patrones de Diseño

### Estado Actual (sin extensiones):

| Categoría | Patrones Soportados | Total | % |
|-----------|---------------------|-------|---|
| **Creacionales** | 3/5 (Factory, Builder, Prototype) | 5 | 60% |
| **Estructurales** | 6/7 (Adapter, Facade, Decorator, Composite, Bridge, Flyweight) | 7 | 86% |
| **Comportamiento** | 7/11 (Strategy, Observer, Template, Command, Iterator, State, Visitor) | 11 | 64% |
| **TOTAL** | **16/23** | **23** | **70%** |

### Con extensiones propuestas (DI + Event Bus + State Management):

| Categoría | Patrones Soportados | Total | % |
|-----------|---------------------|-------|---|
| **Creacionales** | 5/5 ✅ | 5 | **100%** |
| **Estructurales** | 7/7 ✅ | 7 | **100%** |
| **Comportamiento** | 11/11 ✅ | 11 | **100%** |
| **TOTAL** | **23/23 ✅** | **23** | **100%** |

---

## 🎯 Conclusión

**Respuestas resumidas**:

1. **¿Eventos?** → 🟡 PARCIAL (solo UI, falta Event Bus genérico)
2. **¿Conflicto @module?** → ✅ RESUELTO (usar `@container` para DI, `@module` para organización)
3. **¿Patrones de diseño?** → 🟡 70% actual → 100% con extensiones
4. **¿REST APIs?** → 🆕 Agregar `@controller` con decoradores HTTP (`@get`, `@post`, etc.)

**Para ser un lenguaje de alto nivel completo**, Vela necesita implementar en **MVP 1.0**:

1. 🔴 **DI System** (con `@injectable`, `@inject`, `@container`, `@provides`)
2. 🔴 **REST/API Support** (con `@controller`, `@get`, `@post`, `@put`, `@delete`, `@patch`)
3. 🔴 **Event Bus genérico** (con `EventBus`, `on`, `emit`, `off`)
4. 🔴 **State Management global** (con `Store`, `Action`, `dispatch`, `@connect`)

Estas características son **CRÍTICAS** y desbloquearan:
- ✅ **100% de patrones de diseño** estándar
- ✅ **Backend APIs** (REST/microservicios)
- ✅ **Arquitecturas modernas** (DDD, Clean Architecture, Hexagonal)

---

## 🆕 4. Sistema REST/API Controllers 🔴 CRÍTICO

**Prioridad**: P0 (MVP 1.0)

**Keywords nuevos**:
- `@controller(basePath)` - Define controlador REST con ruta base
- `@get(path)` - Endpoint HTTP GET
- `@post(path)` - Endpoint HTTP POST
- `@put(path)` - Endpoint HTTP PUT
- `@delete(path)` - Endpoint HTTP DELETE
- `@patch(path)` - Endpoint HTTP PATCH
- `@middleware` - Middleware para validación/auth
- `@guard` - Guard para autorización
- `Request<T>` - Tipo para requests HTTP
- `Response<T>` - Tipo para responses HTTP

**Código completo de ejemplo**:
```vela
# ============================================
# 1. Definir modelos
# ============================================
type User = {
  id: String,
  name: String,
  email: String,
  role: String
}

type CreateUserDto = {
  name: String,
  email: String,
  password: String
}

# ============================================
# 2. Servicio con lógica de negocio
# ============================================
@injectable(scope: Scope.Singleton)
class UserService {
  constructor(@inject db: Database) { }
  
  fn getUsers() -> Result<List<User>, Error> {
    return db.query("SELECT * FROM users")
  }
  
  fn getUserById(id: String) -> Result<User, Error> {
    return db.queryOne("SELECT * FROM users WHERE id = ?", [id])
  }
  
  fn createUser(dto: CreateUserDto) -> Result<User, Error> {
    hashedPassword = hash(dto.password)
    return db.insert("users", {
      name: dto.name,
      email: dto.email,
      password: hashedPassword
    })
  }
}

# ============================================
# 3. Middleware de autenticación
# ============================================
@middleware
class AuthMiddleware {
  fn handle(req: Request, next: () => Response) -> Response {
    token = req.headers.get("Authorization")
    
    return match token {
      Some(t) if isValidToken(t) => next()
      _ => Response.unauthorized("Invalid token")
    }
  }
}

# ============================================
# 4. Guard de autorización
# ============================================
@guard
class RoleGuard {
  fn canActivate(req: Request, requiredRole: String) -> Bool {
    user = req.user  # Inyectado por AuthMiddleware
    return user?.role == requiredRole
  }
}

# ============================================
# 5. Controlador REST
# ============================================
@controller("/api/users")
@use([AuthMiddleware])  # Aplicar middleware a todas las rutas
class UserController {
  constructor(
    @inject userService: UserService
  ) { }
  
  # GET /api/users
  @get("/")
  fn getAll(req: Request) -> Response<List<User>> {
    return match userService.getUsers() {
      Ok(users) => Response.ok(users)
      Err(error) => Response.internalError(error.message)
    }
  }
  
  # GET /api/users/:id
  @get("/:id")
  fn getById(req: Request, id: String) -> Response<User> {
    return match userService.getUserById(id) {
      Ok(user) => Response.ok(user)
      Err(error) => Response.notFound("User not found")
    }
  }
  
  # POST /api/users
  @post("/")
  @guard(RoleGuard, role: "admin")  # Solo admins pueden crear
  fn create(req: Request<CreateUserDto>) -> Response<User> {
    dto = req.body
    
    return match userService.createUser(dto) {
      Ok(user) => Response.created(user)
      Err(error) => Response.badRequest(error.message)
    }
  }
  
  # PUT /api/users/:id
  @put("/:id")
  @guard(RoleGuard, role: "admin")
  fn update(req: Request<User>, id: String) -> Response<User> {
    return match userService.updateUser(id, req.body) {
      Ok(user) => Response.ok(user)
      Err(error) => Response.badRequest(error.message)
    }
  }
  
  # DELETE /api/users/:id
  @delete("/:id")
  @guard(RoleGuard, role: "admin")
  fn delete(req: Request, id: String) -> Response<void> {
    return match userService.deleteUser(id) {
      Ok(_) => Response.noContent()
      Err(error) => Response.badRequest(error.message)
    }
  }
}

# ============================================
# 6. Bootstrap de la aplicación
# ============================================
@container
class AppContainer {
  @provides(scope: Scope.Singleton)
  fn provideDatabase() -> Database {
    return Database(url: "mongodb://localhost:27017/myapp")
  }
}

# Main
fn main() {
  app = Application(
    containers: [AppContainer()],
    controllers: [UserController],
    middlewares: [AuthMiddleware],
    guards: [RoleGuard]
  )
  
  app.listen(port: 3000)
  print("Server running on http://localhost:3000")
}
```

**Características del sistema REST**:
- ✅ Routing automático basado en decoradores
- ✅ Inyección de dependencias en controllers
- ✅ Path parameters (`:id`)
- ✅ Query parameters (`?page=1&limit=10`)
- ✅ Request body parsing automático
- ✅ Response helpers (ok, created, notFound, etc.)
- ✅ Middleware system (auth, logging, cors, etc.)
- ✅ Guards para autorización
- ✅ Type-safe requests y responses
- ✅ Error handling integrado

**Patrones adicionales desbloqueados**:
- ✅ Controller Pattern
- ✅ MVC/MVVM
- ✅ Middleware Pattern
- ✅ Guard Pattern
- ✅ DTO Pattern
- ✅ Repository Pattern (con DI)

---

**Próximos pasos**: Actualizar `vela-roadmap-scrum.csv` para incluir estas características en MVP 1.0.
