# 2. Arquitectura de Compiladores e Intérpretes de Vela

**NOTA IMPORTANTE**: Este documento describe la **implementación del compilador** (escrito en Rust/C++), NO el lenguaje Vela en sí. El código de implementación puede usar `for`, `while`, `let`, etc., pero el **lenguaje Vela** NO tiene estos constructos (es puramente funcional, sin loops, sin const/let, sin null).

## 2.1 Pipeline General de Compilación

```
Source Code (.vela)
    ↓
[LEXER] → Tokens
    ↓
[PARSER] → AST (Abstract Syntax Tree)
    ↓
[SEMANTIC ANALYZER] → Validated AST
    ↓
[TYPE CHECKER] → Type-annotated AST
    ↓
[DESUGARING] → Simplified AST
    ↓
[IR GENERATION] → Vela IR (Intermediate Representation)
    ↓
    ├─→ [VelaVM Backend] → Bytecode → VelaVM Interpreter
    ├─→ [VelaNative Backend] → LLVM IR → Native Binary
    ├─→ [VelaWeb JS Backend] → JavaScript → Browser/Node
    └─→ [VelaWeb WASM Backend] → WebAssembly → Browser
```

---

## 2.2 Frontend: Del Código Fuente al IR

### 2.2.1 LEXER (Análisis Léxico)

**Responsabilidades**:
- Convertir caracteres a tokens
- Manejar whitespace y comentarios
- Detectar errores léxicos (caracteres inválidos)
- Tracking de posiciones (línea, columna) para errores

**Estructura de Token**:
```rust
struct Token {
  kind: TokenKind,
  lexeme: String,
  location: SourceLocation,
}

enum TokenKind {
  // Keywords
  Fn, Class, If, Else, Return, Let, Const, State, Signal, Actor, Async, Await,
  
  // Identifiers & Literals
  Identifier, IntLiteral, FloatLiteral, StringLiteral, BoolLiteral, NullLiteral,
  
  // Operators
  Plus, Minus, Star, Slash, Percent, 
  EqualEqual, NotEqual, Less, Greater, LessEqual, GreaterEqual,
  And, Or, Not,
  Equal, PlusEqual, MinusEqual, StarEqual, SlashEqual,
  
  // Delimiters
  LeftParen, RightParen, LeftBrace, RightBrace, LeftBracket, RightBracket,
  Semicolon, Colon, Comma, Dot, Arrow, FatArrow,
  
  // Special
  Eof, Error
}

struct SourceLocation {
  file: String,
  line: Int,
  column: Int,
  offset: Int,
}
```

**Algoritmo**:
- State machine con lookahead
- Reconocimiento de keywords vs identifiers
- String interpolation parsing (`"text ${expr}"`)

**Optimizaciones**:
- String interning para identifiers
- Pre-computed keyword hash map
- Zero-copy lexing cuando sea posible

---

### 2.2.2 PARSER (Análisis Sintáctico)

**Responsabilidades**:
- Construir AST desde tokens
- Validación sintáctica
- Manejo de precedencia de operadores
- Error recovery (continuar después de errores)

**Técnica**: Recursive Descent Parser con Pratt Parsing para expresiones

**AST Node Types**:
```rust
enum AstNode {
  // Declarations
  Module(ModuleDecl),
  Function(FunctionDecl),
  Class(ClassDecl),
  Interface(InterfaceDecl),
  Enum(EnumDecl),
  TypeAlias(TypeAliasDecl),
  Actor(ActorDecl),
  Extension(ExtensionDecl),
  
  // Statements
  VarDecl(VarDeclStmt),
  Expression(ExprStmt),
  Return(ReturnStmt),
  If(IfStmt),
  For(ForStmt),
  While(WhileStmt),
  Match(MatchStmt),
  Try(TryStmt),
  StateBlock(StateBlockStmt),
  
  // Expressions
  Binary(BinaryExpr),
  Unary(UnaryExpr),
  Call(CallExpr),
  Member(MemberExpr),
  Index(IndexExpr),
  Lambda(LambdaExpr),
  Literal(LiteralExpr),
  Identifier(IdentifierExpr),
  If(IfExpr),
  Match(MatchExpr),
  Signal(SignalExpr),
  Computed(ComputedExpr),
  Effect(EffectExpr),
  Widget(WidgetExpr),
  Await(AwaitExpr),
  
  // Types
  Type(TypeNode),
}

struct FunctionDecl {
  name: String,
  genericParams: Vec<GenericParam>,
  params: Vec<Param>,
  returnType: Option<TypeNode>,
  body: Option<BlockStmt>,
  attributes: Vec<Attribute>,
  location: SourceLocation,
}

struct ClassDecl {
  name: String,
  genericParams: Vec<GenericParam>,
  superclass: Option<TypeNode>,
  interfaces: Vec<TypeNode>,
  members: Vec<ClassMember>,
  modifiers: Vec<Modifier>,
  location: SourceLocation,
}

struct BinaryExpr {
  left: Box<AstNode>,
  operator: BinaryOp,
  right: Box<AstNode>,
  location: SourceLocation,
}

enum BinaryOp {
  Add, Sub, Mul, Div, Mod,
  Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
  And, Or,
  Assign, AddAssign, SubAssign, MulAssign, DivAssign,
}
```

**Error Recovery Strategies**:
- Panic mode: skip hasta sincronización (`;`, `}`, keywords)
- Inserción de tokens faltantes (`;`, `)`)
- Producción de AST parcial para análisis posteriores

---

### 2.2.3 SEMANTIC ANALYZER

**Responsabilidades**:
- Construcción de Symbol Tables
- Resolución de nombres (name resolution)
- Validación de scopes
- Detección de referencias indefinidas
- Detección de redeclaraciones

**Symbol Table**:
```rust
struct SymbolTable {
  parent: Option<Box<SymbolTable>>,
  symbols: HashMap<String, Symbol>,
}

struct Symbol {
  name: String,
  kind: SymbolKind,
  type: Type,
  location: SourceLocation,
  mutable: Bool,
}

enum SymbolKind {
  Variable,
  Function,
  Class,
  Interface,
  Enum,
  TypeAlias,
  Parameter,
  Field,
  Method,
}
```

**Análisis de Scopes**:
- Global scope
- Module scope
- Function scope
- Block scope
- Class scope

**Validaciones**:
- Variables declaradas antes de uso
- No redeclaración en mismo scope
- Acceso a miembros privados
- Validación de visibilidad (`public`, `private`, etc.)

---

### 2.2.4 TYPE CHECKER

**Responsabilidades**:
- Inferencia de tipos
- Validación de tipos
- Resolución de generics
- Type narrowing (pattern matching, conditionals)
- Null-safety checking

**Sistema de Tipos**:
```rust
enum Type {
  // Primitivos
  Int, Float, String, Bool, Char, Void, Never,
  
  // Compuestos
  Function(FunctionType),
  Tuple(TupleType),
  Array(ArrayType),
  Dict(DictType),
  
  // Nominales
  Class(ClassType),
  Interface(InterfaceType),
  Enum(EnumType),
  
  // Especiales
  Generic(GenericType),
  Union(UnionType),
  Nullable(NullableType),
  Signal(SignalType),
  Actor(ActorType),
  
  // Type variables (para inferencia)
  Var(TypeVar),
  
  // Unknown/Error
  Unknown,
  Error,
}

struct FunctionType {
  params: Vec<Type>,
  returnType: Box<Type>,
  isAsync: Bool,
}

struct GenericType {
  name: String,
  bounds: Vec<Type>,
}

struct UnionType {
  types: Vec<Type>,
}

struct SignalType {
  valueType: Box<Type>,
}
```

**Algoritmo de Inferencia** (basado en Hindley-Milner extendido):
1. Asignar type variables a expresiones sin tipo
2. Generar constraints (ecuaciones de tipos)
3. Unificación (unification)
4. Sustitución de type variables

**Validaciones**:
- Type compatibility en asignaciones
- Argument types en llamadas a funciones
- Return type consistency
- Generic constraints satisfaction
- Null-safety (acceso a nullable sin check)

**Type Narrowing**:
```vela
let x: String? = getValue();

if (x != null) {
  // Aquí x es de tipo String (no nullable)
  let length = x.length;
}

match (result) {
  Result.Ok(value) => {
    // value tiene tipo T
  },
  Result.Err(error) => {
    // error tiene tipo E
  }
}
```

---

### 2.2.5 DESUGARING (Simplificación)

**Responsabilidades**:
- Convertir syntax sugar a formas primitivas
- Simplificar constructos complejos
- Preparar para generación de IR

**Transformaciones**:

1. **For loops** → While loops:
```vela
// Antes
for (item in list) {
  print(item);
}

// Después
let __iter = list.iterator();
while (__iter.hasNext()) {
  let item = __iter.next();
  print(item);
}
```

2. **String interpolation** → Concatenación:
```vela
// Antes
"Hello ${name}!"

// Después
"Hello " + name.toString() + "!"
```

3. **Lambda arrows** → Funciones anónimas:
```vela
// Antes
let double = (x) => x * 2;

// Después
let double = fn(x: Int): Int { return x * 2; };
```

4. **Property access** → Getter/Setter calls:
```vela
// Antes
obj.property = value;

// Después
obj.setProperty(value);
```

5. **Async/Await** → State machines (CPS transform)

6. **Signals** → Reactive graph nodes:
```vela
// Antes
let count = signal(0);
let doubled = computed(() => count.value * 2);

// Después
let count = Signal.create(0);
let doubled = Computed.create(
  () => count.get() * 2,
  [count]  // dependencies
);
```

---

### 2.2.6 IR GENERATION (Vela IR)

**Vela IR**: Intermediate Representation de bajo nivel, independiente de plataforma

**Características**:
- SSA (Static Single Assignment)
- Control flow explícito (basic blocks)
- Type-annotated
- Primitivas para signals y actores

**IR Structure**:
```rust
struct Module {
  name: String,
  functions: Vec<Function>,
  globals: Vec<Global>,
  types: Vec<TypeDef>,
}

struct Function {
  name: String,
  params: Vec<Param>,
  returnType: Type,
  blocks: Vec<BasicBlock>,
  locals: Vec<Local>,
}

struct BasicBlock {
  label: String,
  instructions: Vec<Instruction>,
  terminator: Terminator,
}

enum Instruction {
  // Arithmetic
  Add(Register, Register, Register),
  Sub(Register, Register, Register),
  Mul(Register, Register, Register),
  Div(Register, Register, Register),
  
  // Memory
  Load(Register, Address),
  Store(Address, Register),
  Alloca(Register, Type),
  
  // Calls
  Call(Register, FunctionRef, Vec<Register>),
  
  // Type operations
  Cast(Register, Register, Type),
  
  // Signals
  SignalCreate(Register, Register),      // dest, initial_value
  SignalGet(Register, Register),         // dest, signal
  SignalSet(Register, Register),         // signal, value
  ComputedCreate(Register, FunctionRef, Vec<Register>),  // dest, compute_fn, deps
  EffectCreate(Register, FunctionRef),   // dest, effect_fn
  
  // Actors
  ActorCreate(Register, ActorType),
  ActorSend(Register, MessageId, Vec<Register>),
  
  // Concurrency
  Spawn(Register, FunctionRef, Vec<Register>),
  Await(Register, Register),
  
  // Misc
  Move(Register, Register),
  Copy(Register, Register),
}

enum Terminator {
  Return(Option<Register>),
  Branch(Register, String, String),  // cond, true_block, false_block
  Jump(String),                      // target_block
  Unreachable,
}

struct Register {
  id: Int,
  type: Type,
}
```

**Optimizations en IR**:
- Constant folding
- Dead code elimination
- Common subexpression elimination
- Inline expansion
- Loop optimizations

---

## 2.3 Backend: VelaVM (Bytecode Interpreter)

### 2.3.1 Arquitectura VelaVM

**Características**:
- Stack-based VM
- Bytecode compacto
- Garbage collection integrado (ARC híbrido)
- Scheduler reactivo integrado
- JIT compilation opcional (futuro)

**Bytecode Format**:
```rust
enum Opcode {
  // Stack operations
  Push(Value),
  Pop,
  Dup,
  Swap,
  
  // Local variables
  LoadLocal(u16),
  StoreLocal(u16),
  LoadGlobal(u16),
  StoreGlobal(u16),
  
  // Arithmetic
  Add, Sub, Mul, Div, Mod, Neg,
  
  // Comparison
  Eq, Ne, Lt, Gt, Le, Ge,
  
  // Logic
  And, Or, Not,
  
  // Control flow
  Jump(i32),
  JumpIfFalse(i32),
  Call(u16, u8),       // function_idx, arg_count
  Return,
  
  // Objects
  NewObject(u16),      // class_idx
  GetField(u16),       // field_idx
  SetField(u16),
  
  // Collections
  NewArray(u32),       // size
  GetIndex,
  SetIndex,
  NewDict,
  
  // Signals
  SignalNew,
  SignalGet,
  SignalSet,
  ComputedNew(u16),    // compute_fn_idx
  EffectNew(u16),      // effect_fn_idx
  
  // Actors
  ActorNew(u16),       // actor_class_idx
  ActorSend(u16),      // message_id
  
  // Async
  Await,
  Yield,
  
  // Misc
  Halt,
}
```

**VM Runtime Structure**:
```rust
struct VelaVM {
  // Execution
  stack: Vec<Value>,
  callStack: Vec<CallFrame>,
  globals: HashMap<String, Value>,
  
  // Memory management
  heap: Heap,
  arcManager: ArcManager,
  
  // Reactive system
  signalGraph: SignalGraph,
  scheduler: ReactiveScheduler,
  
  // Concurrency
  actors: HashMap<ActorId, Actor>,
  taskQueue: TaskQueue,
  
  // Metadata
  module: Module,
  constantPool: Vec<Value>,
}

struct CallFrame {
  function: FunctionRef,
  locals: Vec<Value>,
  returnAddress: usize,
  stackBase: usize,
}

enum Value {
  Int(i64),
  Float(f64),
  Bool(bool),
  String(StringRef),
  Object(ObjectRef),
  Array(ArrayRef),
  Dict(DictRef),
  Function(FunctionRef),
  Signal(SignalRef),
  Actor(ActorRef),
  Null,
}
```

**Execution Loop**:
```rust
fn execute(vm: &mut VelaVM) {
  loop {
    let opcode = vm.fetchOpcode();
    
    match opcode {
      Opcode::Add => {
        let b = vm.stack.pop();
        let a = vm.stack.pop();
        vm.stack.push(a + b);
      },
      
      Opcode::Call(fn_idx, arg_count) => {
        let args = vm.stack.popN(arg_count);
        let function = vm.module.functions[fn_idx];
        vm.callStack.push(CallFrame::new(function, args));
        vm.pc = function.entryPoint;
      },
      
      Opcode::SignalGet => {
        let signal = vm.stack.pop();
        let value = vm.signalGraph.getValue(signal);
        vm.stack.push(value);
      },
      
      // ... otros opcodes
      
      Opcode::Halt => break,
    }
  }
}
```

---

### 2.3.2 Reactive Scheduler en VelaVM

**Responsabilidades**:
- Tracking de dependencias entre signals
- Propagación de cambios
- Batching de updates
- Priorización de efectos

**Algoritmo**:
1. Signal value cambia → marca como dirty
2. Notifica a dependientes (computed, effects)
3. Encola updates en scheduler
4. Procesa en orden topológico (evita cálculos redundantes)
5. Ejecuta effects después de todos los computed

```rust
struct ReactiveScheduler {
  dirtySignals: HashSet<SignalId>,
  dirtyComputed: Vec<ComputedId>,
  pendingEffects: Vec<EffectId>,
  updateBatch: bool,
}

fn processBatch(scheduler: &mut ReactiveScheduler, graph: &mut SignalGraph) {
  // 1. Procesar computed en orden topológico
  while (!scheduler.dirtyComputed.isEmpty()) {
    let computed = scheduler.dirtyComputed.pop();
    let newValue = computed.recompute(graph);
    if (newValue != computed.value) {
      computed.value = newValue;
      graph.notifyDependents(computed);
    }
  }
  
  // 2. Ejecutar effects
  for (effect in scheduler.pendingEffects) {
    effect.run();
  }
  
  scheduler.pendingEffects.clear();
}
```

---

## 2.4 Backend: VelaNative (LLVM)

### 2.4.1 LLVM IR Generation

**Proceso**:
1. Vela IR → LLVM IR translation
2. Type mapping (Vela types → LLVM types)
3. Runtime function calls (para GC, signals, actors)
4. Linking con Vela runtime library

**Type Mapping**:
```
Vela Type       → LLVM Type
Int             → i64
Float           → double
Bool            → i1
String          → %String* (struct pointer)
Object          → %Object* (struct pointer)
Signal<T>       → %Signal* (opaque pointer)
Actor<T>        → %Actor* (opaque pointer)
Function        → function pointer
```

**LLVM IR Example**:
```llvm
; Vela function: fn add(a: Int, b: Int): Int => a + b;

define i64 @add(i64 %a, i64 %b) {
entry:
  %result = add i64 %a, %b
  ret i64 %result
}

; Vela function with signal:
; fn counter(): Int {
;   let count = signal(0);
;   state { count.value += 1; }
;   return count.value;
; }

define i64 @counter() {
entry:
  ; Crear signal
  %signal_ptr = call %Signal* @vela_signal_create_int(i64 0)
  
  ; Obtener valor actual
  %old_value = call i64 @vela_signal_get_int(%Signal* %signal_ptr)
  
  ; Incrementar
  %new_value = add i64 %old_value, 1
  
  ; Actualizar signal
  call void @vela_signal_set_int(%Signal* %signal_ptr, i64 %new_value)
  
  ; Retornar
  ret i64 %new_value
}
```

**Runtime Functions**:
```c
// En vela_runtime.c

Signal* vela_signal_create_int(int64_t initial_value);
int64_t vela_signal_get_int(Signal* signal);
void vela_signal_set_int(Signal* signal, int64_t value);

Object* vela_object_allocate(ClassInfo* class_info);
void vela_arc_retain(Object* obj);
void vela_arc_release(Object* obj);

Actor* vela_actor_create(ActorClass* actor_class);
void vela_actor_send(Actor* actor, Message* message);
```

---

### 2.4.2 Optimizaciones LLVM

**Passes aplicados**:
- Inlining de funciones pequeñas
- Loop unrolling
- Dead code elimination
- Constant propagation
- Tail call optimization
- SIMD vectorization (cuando aplicable)

**Link-Time Optimization (LTO)**:
- Whole program optimization
- Cross-module inlining
- Devirtualization

---

## 2.5 Backend: VelaWeb (JavaScript + WebAssembly)

### 2.5.1 JavaScript Backend

**Para**: UI rendering, DOM manipulation, async I/O

**Translation Strategy**:
```vela
// Vela code
fn greet(name: String): String => "Hello, ${name}!";

// Generated JavaScript
function greet(name) {
  return `Hello, ${name}!`;
}
```

**Signals → JavaScript Proxies**:
```javascript
class Signal {
  constructor(initialValue) {
    this._value = initialValue;
    this._subscribers = new Set();
  }
  
  get value() {
    if (currentEffect) {
      this._subscribers.add(currentEffect);
    }
    return this._value;
  }
  
  set value(newValue) {
    this._value = newValue;
    this._notify();
  }
  
  _notify() {
    for (const sub of this._subscribers) {
      scheduler.schedule(sub);
    }
  }
}
```

**UI Rendering**:
```javascript
// Vela Widget → Virtual DOM
function renderTodoApp() {
  const todos = signal([]);
  
  return h('div', { class: 'container' }, [
    h('h1', {}, 'Todo App'),
    h('input', { 
      value: inputValue.value,
      onInput: (e) => inputValue.value = e.target.value
    }),
    ...todos.value.map(todo => h('div', {}, todo))
  ]);
}
```

---

### 2.5.2 WebAssembly Backend

**Para**: Lógica computacionalmente intensiva, algoritmos

**Compilation Path**:
```
Vela IR → WASM IR → WASM Binary
```

**Type Mapping**:
```
Vela Type       → WASM Type
Int             → i64
Float           → f64
Bool            → i32
String          → (externref or i32 pointer to linear memory)
Object          → externref
```

**Example**:
```wasm
;; Vela: fn fibonacci(n: Int): Int
(func $fibonacci (param $n i64) (result i64)
  (if (result i64)
    (i64.le_u (local.get $n) (i64.const 1))
    (then (local.get $n))
    (else
      (i64.add
        (call $fibonacci (i64.sub (local.get $n) (i64.const 1)))
        (call $fibonacci (i64.sub (local.get $n) (i64.const 2)))
      )
    )
  )
)
```

**Interfacing JS ↔ WASM**:
```javascript
// Glue code generado automáticamente
const wasmModule = await WebAssembly.instantiate(wasmBytes, {
  env: {
    signal_create: (value) => { /* JS implementation */ },
    signal_get: (signal) => { /* JS implementation */ },
    // ...
  }
});

export const fibonacci = wasmModule.instance.exports.fibonacci;
```

---

## 2.6 Análisis de Flujo y Optimizaciones Avanzadas

### 2.6.1 Control Flow Analysis

- **CFG (Control Flow Graph)**: Representación de todos los caminos de ejecución
- **Dominator Tree**: Para optimizaciones de code motion
- **Loop Detection**: Identificación de loops naturales

### 2.6.2 Data Flow Analysis

- **Reaching Definitions**: Qué definiciones llegan a cada uso
- **Live Variables**: Qué variables están vivas en cada punto
- **Available Expressions**: Para CSE (Common Subexpression Elimination)

### 2.6.3 Signal Graph Optimization

**Optimizaciones específicas para reactivity**:
1. **Dead Signal Elimination**: Eliminar signals nunca leídos
2. **Signal Fusion**: Combinar computed adjacentes
3. **Memo Insertion**: Memoization automática de computed costosos
4. **Batch Consolidation**: Reducir número de batches reactivos

---

## 2.7 Manejo de Errores en Compilación

### 2.7.1 Error Reporting

**Estructura de Error**:
```rust
struct CompileError {
  kind: ErrorKind,
  message: String,
  location: SourceLocation,
  severity: Severity,
  hints: Vec<String>,
}

enum Severity {
  Error,
  Warning,
  Info,
}
```

**Pretty Printing**:
```
Error: Type mismatch in function call
  ┌─ src/main.vela:15:10
  │
15│   let x = add("hello", 5);
  │           ^^^ expected Int, found String
  │
  = note: function 'add' expects (Int, Int) → Int
  = help: consider using toString() to convert the string
```

### 2.7.2 Error Recovery

**Estrategias**:
- Continuar compilación después de errores
- Generar AST/IR parcial
- Inferir tipos para minimizar errores en cascada

---

## 2.8 Debugging Support

### 2.8.1 Debug Information

**Generación**:
- Source maps (para backends web)
- DWARF debug info (para backends nativos)
- Bytecode debug symbols (para VelaVM)

**Información incluida**:
- Mapping de instrucciones → líneas de código
- Variable names y types
- Call stack frames
- Scope boundaries

### 2.8.2 Breakpoints & Stepping

**VelaVM Support**:
- Breakpoint hooks en bytecode interpreter
- Single-step execution
- Variable inspection
- Expression evaluation en runtime

---

**FIN DEL DOCUMENTO: Arquitectura de Compiladores e Intérpretes**

Esta especificación cubre toda la arquitectura de compilación y será base para:
- Implementación del compiler frontend
- Implementación de backends (VM, Native, Web)
- Optimizaciones
- Herramientas de debugging
- Testing del compilador
